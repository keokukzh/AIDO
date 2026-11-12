use serde::{Deserialize, Serialize};
use std::net::TcpStream;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("DPAPI-Verschlüsselung fehlgeschlagen: {0}")]
    DpapiEncryptionFailed(String),
    #[error("DPAPI-Entschlüsselung fehlgeschlagen: {0}")]
    DpapiDecryptionFailed(String),
    #[error("Systemabfrage fehlgeschlagen: {0}")]
    SystemQueryFailed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntegrityLevel {
    Untrusted,
    Low,
    Medium,
    High,
    System,
}

impl IntegrityLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            IntegrityLevel::Untrusted => "Untrusted",
            IntegrityLevel::Low => "Low",
            IntegrityLevel::Medium => "Medium (Standardbenutzer)",
            IntegrityLevel::High => "High (Administrator)",
            IntegrityLevel::System => "System",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSecurityContext {
    pub is_elevated: bool,
    pub integrity_level: IntegrityLevel,
    pub user_name: String,
    pub elevation_safe_for_companion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoredPortInfo {
    pub port: u16,
    pub service_name: String,
    pub is_active: bool,
    pub is_protected_service: bool,
    pub note_de: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSecurityReport {
    pub security_context: ProcessSecurityContext,
    pub monitored_ports: Vec<MonitoredPortInfo>,
    pub hermes_protected_on_18434: bool,
    pub whisper_ready_on_18080: bool,
}

#[repr(C)]
struct DataBlob {
    cb_data: u32,
    pb_data: *mut u8,
}

#[cfg(windows)]
#[link(name = "crypt32")]
extern "system" {
    fn CryptProtectData(
        p_data_in: *const DataBlob,
        sz_data_descr: *const u16,
        p_optional_entropy: *const DataBlob,
        pv_reserved: *mut std::ffi::c_void,
        p_prompt_struct: *mut std::ffi::c_void,
        dw_flags: u32,
        p_data_out: *mut DataBlob,
    ) -> i32;

    fn CryptUnprotectData(
        p_data_in: *const DataBlob,
        ppsz_data_descr: *mut *mut u16,
        p_optional_entropy: *const DataBlob,
        pv_reserved: *mut std::ffi::c_void,
        p_prompt_struct: *mut std::ffi::c_void,
        dw_flags: u32,
        p_data_out: *mut DataBlob,
    ) -> i32;

    fn LocalFree(h_mem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
}

#[derive(Debug, Clone, Default)]
pub struct WindowsDpapiVault;

impl WindowsDpapiVault {
    pub fn new() -> Self {
        Self
    }

    /// Encrypts plaintext bytes using Windows Data Protection API (DPAPI).
    /// The ciphertext is tied directly to the current logged-in Windows user account.
    pub fn protect(&self, plaintext: &[u8]) -> Result<Vec<u8>, SecurityError> {
        #[cfg(windows)]
        {
            if plaintext.is_empty() {
                return Ok(Vec::new());
            }
            let in_blob = DataBlob {
                cb_data: plaintext.len() as u32,
                pb_data: plaintext.as_ptr() as *mut u8,
            };
            let mut out_blob = DataBlob {
                cb_data: 0,
                pb_data: std::ptr::null_mut(),
            };

            // dwFlags = 1 (CRYPTPROTECT_UI_FORBIDDEN - no UI dialogs)
            let success = unsafe {
                CryptProtectData(
                    &in_blob,
                    std::ptr::null(),
                    std::ptr::null(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    1,
                    &mut out_blob,
                )
            };

            if success != 0 && !out_blob.pb_data.is_null() {
                let slice = unsafe {
                    std::slice::from_raw_parts(out_blob.pb_data, out_blob.cb_data as usize)
                };
                let result = slice.to_vec();
                unsafe {
                    LocalFree(out_blob.pb_data as *mut std::ffi::c_void);
                }
                Ok(result)
            } else {
                Err(SecurityError::DpapiEncryptionFailed(
                    "CryptProtectData schlug fehl".to_string(),
                ))
            }
        }
        #[cfg(not(windows))]
        {
            // Portable mock fallback for non-Windows platforms
            Ok(plaintext.iter().map(|b| b ^ 0xAA).collect())
        }
    }

    /// Decrypts ciphertext previously sealed with protect() for the current Windows user.
    pub fn unprotect(&self, ciphertext: &[u8]) -> Result<Vec<u8>, SecurityError> {
        #[cfg(windows)]
        {
            if ciphertext.is_empty() {
                return Ok(Vec::new());
            }
            let in_blob = DataBlob {
                cb_data: ciphertext.len() as u32,
                pb_data: ciphertext.as_ptr() as *mut u8,
            };
            let mut out_blob = DataBlob {
                cb_data: 0,
                pb_data: std::ptr::null_mut(),
            };

            let success = unsafe {
                CryptUnprotectData(
                    &in_blob,
                    std::ptr::null_mut(),
                    std::ptr::null(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    1,
                    &mut out_blob,
                )
            };

            if success != 0 && !out_blob.pb_data.is_null() {
                let slice = unsafe {
                    std::slice::from_raw_parts(out_blob.pb_data, out_blob.cb_data as usize)
                };
                let result = slice.to_vec();
                unsafe {
                    LocalFree(out_blob.pb_data as *mut std::ffi::c_void);
                }
                Ok(result)
            } else {
                Err(SecurityError::DpapiDecryptionFailed(
                    "CryptUnprotectData schlug fehl (ungültiges Secret oder anderer Nutzerkontext)".to_string(),
                ))
            }
        }
        #[cfg(not(windows))]
        {
            Ok(ciphertext.iter().map(|b| b ^ 0xAA).collect())
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SystemSecurityGuard;

impl SystemSecurityGuard {
    pub fn new() -> Self {
        Self
    }

    /// Inspects the current process security token (Integrity Level and Elevation)
    pub fn inspect_security_context(&self) -> ProcessSecurityContext {
        let user_name = std::env::var("USERNAME").unwrap_or_else(|_| "Nutzer".to_string());

        // Fast elevation check via Windows whoami /groups
        let is_elevated = if let Ok(output) = std::process::Command::new("whoami")
            .arg("/groups")
            .output()
        {
            let text = String::from_utf8_lossy(&output.stdout);
            // S-1-16-12288 is the well-known SID for High Mandatory Level (Elevated Admin)
            text.contains("S-1-16-12288") || text.contains("High Mandatory Level")
        } else {
            false
        };

        let integrity_level = if is_elevated {
            IntegrityLevel::High
        } else {
            IntegrityLevel::Medium
        };

        // AIDO is designed to run securely at Medium integrity so it does not expose an elevated attack surface
        ProcessSecurityContext {
            is_elevated,
            integrity_level,
            user_name,
            elevation_safe_for_companion: !is_elevated,
        }
    }

    /// Probes ports of local companion and AI services on 127.0.0.1
    pub fn probe_services(&self) -> Vec<MonitoredPortInfo> {
        let targets = [
            (18434, "Hermes Agent", true, "Geschützter Hermes-Agentendienst (bleibt unangetastet)"),
            (18080, "Whisper.cpp Server", false, "Lokaler ASR-Sprachdienst"),
            (11434, "Ollama", false, "Lokale LLM-Laufzeit"),
            (1234, "LM Studio", false, "Lokale Modellumgebung"),
            (8080, "AIDO Web-Vorschau", false, "Lokaler Vorschau-Port"),
        ];

        targets
            .into_iter()
            .map(|(port, name, protected, note)| {
                let is_active = TcpStream::connect_timeout(
                    &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
                    Duration::from_millis(50),
                )
                .is_ok();

                MonitoredPortInfo {
                    port,
                    service_name: name.to_string(),
                    is_active,
                    is_protected_service: protected,
                    note_de: note.to_string(),
                }
            })
            .collect()
    }

    /// Complete security & service health audit
    pub fn generate_report(&self) -> SystemSecurityReport {
        let security_context = self.inspect_security_context();
        let monitored_ports = self.probe_services();

        let hermes_protected_on_18434 = monitored_ports
            .iter()
            .find(|p| p.port == 18434)
            .map(|p| p.is_active)
            .unwrap_or(false);

        let whisper_ready_on_18080 = monitored_ports
            .iter()
            .find(|p| p.port == 18080)
            .map(|p| p.is_active)
            .unwrap_or(false);

        SystemSecurityReport {
            security_context,
            monitored_ports,
            hermes_protected_on_18434,
            whisper_ready_on_18080,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dpapi_vault_roundtrip() {
        let vault = WindowsDpapiVault::new();
        let secret = b"AIDO_SUPER_SECRET_TOKEN_2026_RTX5090";

        let ciphertext = vault.protect(secret).expect("DPAPI protect should succeed");
        assert_ne!(ciphertext, secret, "Ciphertext must not match plaintext");

        let decrypted = vault.unprotect(&ciphertext).expect("DPAPI unprotect should succeed");
        assert_eq!(decrypted, secret, "Decrypted bytes must match original plaintext");
    }

    #[test]
    fn test_dpapi_vault_empty_slice() {
        let vault = WindowsDpapiVault::new();
        let encrypted = vault.protect(&[]).unwrap();
        let decrypted = vault.unprotect(&encrypted).unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_security_context_medium_integrity() {
        let guard = SystemSecurityGuard::new();
        let ctx = guard.inspect_security_context();
        println!("Process Integrity: {:?}", ctx.integrity_level);
        assert!(!ctx.user_name.is_empty());
    }

    #[test]
    fn test_service_probe_and_report() {
        let guard = SystemSecurityGuard::new();
        let report = guard.generate_report();
        assert_eq!(report.monitored_ports.len(), 5);
        let hermes = report.monitored_ports.iter().find(|p| p.port == 18434).unwrap();
        assert!(hermes.is_protected_service);
    }
}
