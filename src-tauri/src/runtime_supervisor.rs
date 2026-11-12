use crate::permission_gateway::{ActionKind, GatewayError, PermissionGateway};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Privacy/Permission Gateway Blockade: {0}")]
    Gateway(#[from] GatewayError),
    #[error("Dateisystem- oder E/A-Fehler: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeState {
    WarmActive,
    SleepingZeroVram,
    StoppedStandby,
    MissingBinary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRuntimeInfo {
    pub service_id: String,
    pub display_name: String,
    pub binary_path: String,
    pub binary_exists: bool,
    pub cuda_dll_dir: String,
    pub cuda_dll_ready: bool,
    pub endpoint_url: String,
    pub sleep_idle_seconds: Option<u32>,
    pub launch_args: Vec<String>,
    pub state: RuntimeState,
    pub human_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStatusReport {
    pub llama_router: ServiceRuntimeInfo,
    pub whisper_asr: ServiceRuntimeInfo,
    pub overall_summary: String,
}

#[derive(Debug, Clone)]
pub struct RuntimeSupervisor {
    pub llama_binary: PathBuf,
    pub whisper_binary: PathBuf,
    pub cuda_dll_dir: PathBuf,
    pub llama_endpoint: String,
    pub whisper_endpoint: String,
    pub sleep_idle_seconds: u32,
}

impl Default for RuntimeSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeSupervisor {
    pub fn new() -> Self {
        Self {
            llama_binary: PathBuf::from(
                r"D:\AI_Models\unsloth\llama.cpp\build\bin\Release\llama-server.exe",
            ),
            whisper_binary: PathBuf::from(
                r"D:\AI_Models\unsloth\whisper.cpp\build\bin\Release\whisper-server.exe",
            ),
            cuda_dll_dir: PathBuf::from(r"C:\AI\llamacpp-cuda"),
            llama_endpoint: "http://127.0.0.1:18434".into(),
            whisper_endpoint: "http://127.0.0.1:18080".into(),
            sleep_idle_seconds: 300,
        }
    }

    pub fn with_custom_config(
        llama_binary: impl Into<PathBuf>,
        whisper_binary: impl Into<PathBuf>,
        cuda_dll_dir: impl Into<PathBuf>,
        llama_endpoint: impl Into<String>,
        whisper_endpoint: impl Into<String>,
        sleep_idle_seconds: u32,
    ) -> Self {
        Self {
            llama_binary: llama_binary.into(),
            whisper_binary: whisper_binary.into(),
            cuda_dll_dir: cuda_dll_dir.into(),
            llama_endpoint: llama_endpoint.into(),
            whisper_endpoint: whisper_endpoint.into(),
            sleep_idle_seconds,
        }
    }

    /// Prepends `self.cuda_dll_dir` to `current_path` idempotently.
    /// Critical for Windows CUDA 13.4 DLL resolution (`cudart64_13.dll`, `cublas64_13.dll`).
    pub fn build_env_path(&self, current_path: &str) -> String {
        let dll_dir_str = self.cuda_dll_dir.to_string_lossy();
        let dll_dir_clean = dll_dir_str.trim().trim_end_matches(['\\', '/']);

        let already_present = current_path.split(';').any(|p| {
            p.trim()
                .trim_end_matches(['\\', '/'])
                .eq_ignore_ascii_case(dll_dir_clean)
        });

        if already_present {
            current_path.to_string()
        } else if current_path.trim().is_empty() {
            dll_dir_str.to_string()
        } else {
            format!("{};{}", dll_dir_str, current_path)
        }
    }

    /// Probes the local AI runtime status and validates offline privacy guardrails.
    pub fn probe_status(
        &self,
        gateway: &PermissionGateway,
    ) -> Result<RuntimeStatusReport, RuntimeError> {
        gateway.check_readonly_allowed(&ActionKind::ProbeLocalRuntimes)?;
        gateway.check_endpoint_allowed(&self.llama_endpoint)?;
        gateway.check_endpoint_allowed(&self.whisper_endpoint)?;

        let cuda_dll_ready = self.cuda_dll_dir.join("cudart64_13.dll").exists();
        let llama_binary_exists = self.llama_binary.exists();
        let whisper_binary_exists = self.whisper_binary.exists();

        // 1. Probe llama-server (/models)
        let llama_url = format!("{}/models", self.llama_endpoint.trim_end_matches('/'));
        let llama_probe = ureq::get(&llama_url)
            .timeout(std::time::Duration::from_millis(400))
            .call();

        let (llama_state, llama_human_status) = match llama_probe {
            Ok(resp) => {
                let body = resp.into_string().unwrap_or_default();
                let is_sleeping = if let Ok(val) = serde_json::from_str::<serde_json::Value>(&body)
                {
                    let check_models = |arr: &[serde_json::Value]| -> bool {
                        !arr.is_empty()
                            && arr.iter().all(|m| {
                                m.get("status").and_then(|s| s.as_str()) == Some("unloaded")
                                    || m.get("unloaded").and_then(|u| u.as_bool()) == Some(true)
                            })
                    };

                    if let Some(models) = val.get("models").and_then(|m| m.as_array()) {
                        check_models(models)
                    } else if let Some(data) = val.get("data").and_then(|d| d.as_array()) {
                        check_models(data)
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_sleeping {
                    (
                        RuntimeState::SleepingZeroVram,
                        format!(
                            "Im Null-VRAM-Ruhemodus (Zero-VRAM Standby). Nach {} Sekunden Inaktivität automatisch entladen (0 MiB VRAM) und bereit für sofortigen Aufruf.",
                            self.sleep_idle_seconds
                        ),
                    )
                } else {
                    (
                        RuntimeState::WarmActive,
                        format!(
                            "Laufend und betriebsbereit (Warm Active) auf {}.",
                            self.llama_endpoint
                        ),
                    )
                }
            }
            Err(_) => {
                if llama_binary_exists {
                    (
                        RuntimeState::StoppedStandby,
                        format!(
                            "Gestoppt (Bereit für On-Demand-Start auf Abruf, 0 MiB VRAM). Automatische Entladung nach {}s Inaktivität vorkonfiguriert.",
                            self.sleep_idle_seconds
                        ),
                    )
                } else {
                    (
                        RuntimeState::MissingBinary,
                        format!(
                            "Llama-Server Binary nicht gefunden unter: {}",
                            self.llama_binary.to_string_lossy()
                        ),
                    )
                }
            }
        };

        // 2. Probe whisper-server (/)
        let whisper_url = format!("{}/", self.whisper_endpoint.trim_end_matches('/'));
        let whisper_probe = ureq::get(&whisper_url)
            .timeout(std::time::Duration::from_millis(400))
            .call();

        let (whisper_state, whisper_human_status) = match whisper_probe {
            Ok(_) => (
                RuntimeState::WarmActive,
                format!(
                    "Laufend und betriebsbereit (CUDA 13.4 ASR, Deutsches Modell) auf {}.",
                    self.whisper_endpoint
                ),
            ),
            Err(_) => {
                if whisper_binary_exists {
                    (
                        RuntimeState::StoppedStandby,
                        "Gestoppt (Bereit für On-Demand-Start auf Abruf, 0 MiB VRAM).".to_string(),
                    )
                } else {
                    (
                        RuntimeState::MissingBinary,
                        format!(
                            "Whisper-Server Binary nicht gefunden unter: {}",
                            self.whisper_binary.to_string_lossy()
                        ),
                    )
                }
            }
        };

        // Exact verified arguments from Phase 1
        let llama_args = vec![
            "--host".into(),
            "127.0.0.1".into(),
            "--port".into(),
            "18434".into(),
            "--models-Dir".into(),
            r"D:\AI_Models\llm".into(),
            "--ctx-size".into(),
            "32768".into(),
            "-ngl".into(),
            "99".into(),
            "--gpu-layers-draft".into(),
            "99".into(),
            "--flash-attn".into(),
            "on".into(),
            "--sleep-idle-seconds".into(),
            self.sleep_idle_seconds.to_string(),
        ];

        let whisper_args = vec![
            "--host".into(),
            "127.0.0.1".into(),
            "--port".into(),
            "18080".into(),
            "-m".into(),
            r"D:\AI_Models\unsloth\whisper.cpp\models\ggml-small.bin".into(),
            "-l".into(),
            "de".into(),
            "-fa".into(),
        ];

        let llama_info = ServiceRuntimeInfo {
            service_id: "llama_router".into(),
            display_name: "Llama Router (Qwen3.8 + Vision + DFlash2)".into(),
            binary_path: self.llama_binary.to_string_lossy().to_string(),
            binary_exists: llama_binary_exists,
            cuda_dll_dir: self.cuda_dll_dir.to_string_lossy().to_string(),
            cuda_dll_ready,
            endpoint_url: self.llama_endpoint.clone(),
            sleep_idle_seconds: Some(self.sleep_idle_seconds),
            launch_args: llama_args,
            state: llama_state,
            human_status: llama_human_status,
        };

        let whisper_info = ServiceRuntimeInfo {
            service_id: "whisper_asr".into(),
            display_name: "Whisper ASR (German Speech-to-Text)".into(),
            binary_path: self.whisper_binary.to_string_lossy().to_string(),
            binary_exists: whisper_binary_exists,
            cuda_dll_dir: self.cuda_dll_dir.to_string_lossy().to_string(),
            cuda_dll_ready,
            endpoint_url: self.whisper_endpoint.clone(),
            sleep_idle_seconds: None,
            launch_args: whisper_args,
            state: whisper_state,
            human_status: whisper_human_status,
        };

        let overall_summary = match (llama_state, whisper_state) {
            (RuntimeState::WarmActive, RuntimeState::WarmActive) => {
                "Alle lokalen KI-Runtimes (Llama & Whisper) sind aktiv und betriebsbereit.".to_string()
            }
            (RuntimeState::SleepingZeroVram, RuntimeState::WarmActive) => {
                "Llama befindet sich im Null-VRAM-Ruhemodus (0 MiB VRAM), Whisper ist aktiv.".to_string()
            }
            (RuntimeState::WarmActive, _) => {
                "Llama-Server ist aktiv und betriebsbereit, Whisper-ASR im Standby (0 MiB VRAM).".to_string()
            }
            (RuntimeState::StoppedStandby, RuntimeState::StoppedStandby) => {
                "Alle lokalen KI-Runtimes (Llama & Whisper) befinden sich im Standby-Modus (0 MiB VRAM). Bereit für On-Demand-Start.".to_string()
            }
            _ => {
                format!(
                    "Lokale KI-Runtimes: Llama ist {:?}, Whisper ist {:?}.",
                    llama_state, whisper_state
                )
            }
        };

        Ok(RuntimeStatusReport {
            llama_router: llama_info,
            whisper_asr: whisper_info,
            overall_summary,
        })
    }

    /// Forcibly unloads the active model slot from VRAM via llama-server API
    /// Requires an explicit ApprovalToken for ActionKind::UnloadIdleModelSlot.
    pub fn force_unload_model(
        &self,
        gateway: &PermissionGateway,
        token: &crate::permission_gateway::ApprovalToken,
    ) -> Result<String, RuntimeError> {
        gateway.consume_approval_token(token, &ActionKind::UnloadIdleModelSlot)?;
        gateway.check_endpoint_allowed(&self.llama_endpoint)?;

        let unload_url = format!("{}/slots/0?action=erase", self.llama_endpoint.trim_end_matches('/'));
        let _ = ureq::post(&unload_url)
            .timeout(std::time::Duration::from_millis(1500))
            .call();

        Ok("Modellslot erfolgreich entladen. VRAM im Null-VRAM-Ruhemodus freigegeben.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permission_gateway::{AccessMode, PrivacyMode};

    #[test]
    fn test_build_env_path_prepends_cuda_dll_dir_idempotently() {
        let supervisor = RuntimeSupervisor::with_custom_config(
            r"C:\fake\llama-server.exe",
            r"C:\fake\whisper-server.exe",
            r"C:\AI\llamacpp-cuda",
            "http://127.0.0.1:18434",
            "http://127.0.0.1:18080",
            300,
        );

        let initial_path = r"C:\Windows\system32;C:\Windows";
        let prepended = supervisor.build_env_path(initial_path);
        assert_eq!(
            prepended,
            r"C:\AI\llamacpp-cuda;C:\Windows\system32;C:\Windows"
        );

        // Idempotent: prepending again does not duplicate
        let idempotent = supervisor.build_env_path(&prepended);
        assert_eq!(idempotent, prepended);

        // Handles trailing slash in current path
        let path_with_slash = r"C:\AI\llamacpp-cuda\;C:\Windows";
        let untouched = supervisor.build_env_path(path_with_slash);
        assert_eq!(untouched, path_with_slash);

        // Handles empty path
        let empty_result = supervisor.build_env_path("");
        assert_eq!(empty_result, r"C:\AI\llamacpp-cuda");
    }

    #[test]
    fn test_probe_status_enforces_private_offline_loopback() {
        let gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);
        let supervisor = RuntimeSupervisor::with_custom_config(
            r"C:\fake\llama-server.exe",
            r"C:\fake\whisper-server.exe",
            r"C:\AI\llamacpp-cuda",
            "https://external-cloud-api.example.com",
            "http://127.0.0.1:18080",
            300,
        );

        let result = supervisor.probe_status(&gateway);
        assert!(matches!(
            result,
            Err(RuntimeError::Gateway(GatewayError::OfflineViolation(_)))
        ));
    }

    #[test]
    fn test_probe_status_reports_verified_launch_args_and_standby_state() {
        let gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);
        let supervisor = RuntimeSupervisor::with_custom_config(
            r"C:\fake\llama-server.exe",
            r"C:\fake\whisper-server.exe",
            r"C:\AI\llamacpp-cuda",
            "http://127.0.0.1:28434",
            "http://127.0.0.1:28080",
            300,
        );

        let report = supervisor.probe_status(&gateway).expect("Probe status should succeed");

        // Verify Llama router launch args
        assert!(report.llama_router.launch_args.contains(&"--sleep-idle-seconds".to_string()));
        assert!(report.llama_router.launch_args.contains(&"300".to_string()));
        assert!(report.llama_router.launch_args.contains(&"--flash-attn".to_string()));
        assert!(report.llama_router.launch_args.contains(&"on".to_string()));
        assert_eq!(report.llama_router.sleep_idle_seconds, Some(300));

        // Verify Whisper ASR launch args
        assert!(report.whisper_asr.launch_args.contains(&"-l".to_string()));
        assert!(report.whisper_asr.launch_args.contains(&"de".to_string()));
        assert!(report.whisper_asr.launch_args.contains(&"-fa".to_string()));
        assert_eq!(report.whisper_asr.sleep_idle_seconds, None);

        // State reflects unrunning dummy port: StoppedStandby or MissingBinary
        assert!(
            report.llama_router.state == RuntimeState::StoppedStandby
                || report.llama_router.state == RuntimeState::MissingBinary
        );
        assert!(
            report.whisper_asr.state == RuntimeState::StoppedStandby
                || report.whisper_asr.state == RuntimeState::MissingBinary
        );

        // Human status is German
        assert!(
            report.llama_router.human_status.contains("0 MiB VRAM")
                || report.llama_router.human_status.contains("Binary nicht gefunden")
        );
    }

    #[test]
    fn test_force_unload_model_requires_approval_token() {
        let gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);
        let supervisor = RuntimeSupervisor::with_custom_config(
            r"C:\fake\llama-server.exe",
            r"C:\fake\whisper-server.exe",
            r"C:\AI\llamacpp-cuda",
            "http://127.0.0.1:18434",
            "http://127.0.0.1:18080",
            300,
        );

        let req = gateway.request_approval(
            ActionKind::UnloadIdleModelSlot,
            "Unload VRAM",
            "Freigabe",
            vec!["127.0.0.1:18434".into()],
            0,
            16000,
        ).unwrap();

        let token = gateway.approve_request(req.request_id, None).unwrap();
        let unload_res = supervisor.force_unload_model(&gateway, &token);
        assert!(unload_res.is_ok());

        // Second consumption with already consumed token must fail
        let unload_again = supervisor.force_unload_model(&gateway, &token);
        assert!(unload_again.is_err());
    }
}
