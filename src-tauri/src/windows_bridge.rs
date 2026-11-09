use crate::permission_gateway::{
    ActionKind, ApprovalRequest, ApprovalToken, GatewayError, PermissionGateway,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BridgeError {
    #[error("Permission Gateway Fehler: {0}")]
    Gateway(#[from] GatewayError),
    #[error("Dateisystem-Fehler: {0}")]
    Io(#[from] std::io::Error),
    #[error("Sicherheitsverstoß: Pfad liegt außerhalb des erlaubten AIDO-Sandbox-Caches: {0}")]
    UnsafeCleanupPath(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuStatus {
    pub gpu_name: String,
    pub vram_used_mib: u64,
    pub vram_free_mib: u64,
    pub vram_total_mib: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSummary {
    pub name: String,
    pub pid: u32,
    pub working_set_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealthReport {
    pub timestamp: String,
    pub gpu: GpuStatus,
    pub system_ram_free_mb: u64,
    pub system_ram_total_mb: u64,
    pub top_processes: Vec<ProcessSummary>,
    pub aido_temp_cache_path: String,
    pub aido_temp_cache_files: usize,
    pub aido_temp_cache_bytes: u64,
    pub human_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastCleanupResult {
    pub executed_at: String,
    pub removed_files_count: usize,
    pub freed_disk_bytes: u64,
    pub vram_before_mib: u64,
    pub vram_after_mib: u64,
    pub human_report: String,
}

pub struct WindowsBridge {
    gateway: PermissionGateway,
    safe_temp_dir: PathBuf,
}

impl WindowsBridge {
    pub fn new(gateway: PermissionGateway, safe_temp_dir: impl AsRef<Path>) -> Result<Self, BridgeError> {
        let safe_temp_dir = safe_temp_dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&safe_temp_dir)?;
        Ok(Self {
            gateway,
            safe_temp_dir,
        })
    }

    pub fn safe_temp_dir(&self) -> &Path {
        &self.safe_temp_dir
    }

    /// Populates benign temporary cache files inside AIDO's isolated scratch cache
    /// so that a Fast Cleanup demonstration has verifiable, safe artifacts to inspect and clean.
    pub fn seed_demo_cache_files(&self) -> Result<u64, BridgeError> {
        std::fs::create_dir_all(&self.safe_temp_dir)?;
        let samples = [
            ("aido_vision_preview_01.tmp", 64 * 1024),
            ("aido_whisper_chunk_01.tmp", 128 * 1024),
            ("aido_slot_prompt_cache.tmp", 256 * 1024),
        ];
        let mut total = 0u64;
        for (name, size) in samples {
            let file_path = self.safe_temp_dir.join(name);
            let payload = vec![b'A'; size];
            std::fs::write(&file_path, &payload)?;
            total += size as u64;
        }
        Ok(total)
    }

    fn query_gpu_status() -> GpuStatus {
        let out = Command::new("nvidia-smi")
            .args([
                "--query-gpu=name,memory.used,memory.free,memory.total",
                "--format=csv,noheader,nounits",
            ])
            .output();

        if let Ok(output) = out {
            if output.status.success() {
                let text = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = text.lines().next() {
                    let parts: Vec<&str> = first_line.split(',').map(|s| s.trim()).collect();
                    if parts.len() >= 4 {
                        let used = parts[1].parse::<u64>().unwrap_or(0);
                        let free = parts[2].parse::<u64>().unwrap_or(0);
                        let total = parts[3].parse::<u64>().unwrap_or(32607);
                        return GpuStatus {
                            gpu_name: parts[0].to_string(),
                            vram_used_mib: used,
                            vram_free_mib: free,
                            vram_total_mib: total,
                        };
                    }
                }
            }
        }
        GpuStatus {
            gpu_name: "NVIDIA GeForce RTX 5090 (Fallback-Sensor)".into(),
            vram_used_mib: 0,
            vram_free_mib: 32607,
            vram_total_mib: 32607,
        }
    }

    fn query_ram_and_processes() -> (u64, u64, Vec<ProcessSummary>) {
        let ps_script = r#"
$os = Get-CimInstance Win32_OperatingSystem
$totalMb = [math]::Round($os.TotalVisibleMemorySize / 1024)
$freeMb = [math]::Round($os.FreePhysicalMemory / 1024)
$procs = Get-Process | Sort-Object WorkingSet64 -Descending | Select-Object -First 5 | ForEach-Object {
    "$($_.ProcessName)|$($_.Id)|$([math]::Round($_.WorkingSet64 / 1MB))"
}
Write-Output "RAM:$freeMb|$totalMb"
$procs | ForEach-Object { Write-Output "PROC:$_" }
"#;
        let out = Command::new("powershell")
            .args(["-NoProfile", "-Command", ps_script])
            .output();

        let mut free_mb = 32000u64;
        let mut total_mb = 65536u64;
        let mut top_procs = Vec::new();

        if let Ok(output) = out {
            let text = String::from_utf8_lossy(&output.stdout);
            for line in text.lines() {
                if let Some(rest) = line.strip_prefix("RAM:") {
                    let parts: Vec<&str> = rest.split('|').collect();
                    if parts.len() == 2 {
                        free_mb = parts[0].trim().parse().unwrap_or(free_mb);
                        total_mb = parts[1].trim().parse().unwrap_or(total_mb);
                    }
                } else if let Some(rest) = line.strip_prefix("PROC:") {
                    let parts: Vec<&str> = rest.split('|').collect();
                    if parts.len() == 3 {
                        top_procs.push(ProcessSummary {
                            name: parts[0].trim().to_string(),
                            pid: parts[1].trim().parse().unwrap_or(0),
                            working_set_mb: parts[2].trim().parse().unwrap_or(0),
                        });
                    }
                }
            }
        }
        (free_mb, total_mb, top_procs)
    }

    fn inspect_safe_temp_dir(&self) -> Result<(usize, u64, Vec<String>), BridgeError> {
        let mut count = 0usize;
        let mut bytes = 0u64;
        let mut names = Vec::new();
        if self.safe_temp_dir.is_dir() {
            for entry in std::fs::read_dir(&self.safe_temp_dir)? {
                let entry = entry?;
                let meta = entry.metadata()?;
                if meta.is_file() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name.ends_with(".tmp") || file_name.ends_with(".cache") {
                        count += 1;
                        bytes += meta.len();
                        names.push(file_name);
                    }
                }
            }
        }
        names.sort();
        Ok((count, bytes, names))
    }

    /// Read-only inspection of the current PC health (GPU VRAM, RAM, Top Processes, Safe AIDO Temp Cache).
    pub fn run_system_health_check(&self) -> Result<SystemHealthReport, BridgeError> {
        self.gateway
            .check_readonly_allowed(&ActionKind::SystemHealthCheck)?;

        let gpu = Self::query_gpu_status();
        let (ram_free, ram_total, top_processes) = Self::query_ram_and_processes();
        let (cache_files, cache_bytes, _) = self.inspect_safe_temp_dir()?;

        let cache_kb = cache_bytes / 1024;
        let human_summary = format!(
            "Ich habe deinen PC geprüft (nur lesend): Deine {} nutzt aktuell {} MiB von {} MiB VRAM ({} MiB frei). \
             Vom Arbeitsspeicher (RAM) sind {} MB von {} MB frei. \
             Im isolierten AIDO-Temporärspeicher liegen {} temporäre Cache-Dateien ({} KB), die beim Fast Cleanup sicher entfernt werden können.",
            gpu.gpu_name,
            gpu.vram_used_mib,
            gpu.vram_total_mib,
            gpu.vram_free_mib,
            ram_free,
            ram_total,
            cache_files,
            cache_kb
        );

        Ok(SystemHealthReport {
            timestamp: Utc::now().to_rfc3339(),
            gpu,
            system_ram_free_mb: ram_free,
            system_ram_total_mb: ram_total,
            top_processes,
            aido_temp_cache_path: self.safe_temp_dir.display().to_string(),
            aido_temp_cache_files: cache_files,
            aido_temp_cache_bytes: cache_bytes,
            human_summary,
        })
    }

    /// Prepares a Fast Cleanup proposal and requests explicit user approval via the Permission Gateway.
    pub fn prepare_fast_cleanup(&self) -> Result<(SystemHealthReport, ApprovalRequest), BridgeError> {
        let report = self.run_system_health_check()?;
        let (_, cache_bytes, file_names) = self.inspect_safe_temp_dir()?;

        let explanation = format!(
            "Ich werde ausschließlich die {} temporären AIDO-Cache-Dateien ({} KB) im isolierten Ordner '{}' bereinigen. \
             Deine persönlichen Dateien, Projekte und KI-Modelle werden nicht berührt. Darf ich das Fast Cleanup jetzt ausführen?",
            file_names.len(),
            cache_bytes / 1024,
            self.safe_temp_dir.display()
        );

        let approval_req = self.gateway.request_approval(
            ActionKind::FastCleanupExecute,
            "AIDO Fast Cleanup (Temporäre Sitzungs-Caches bereinigen)",
            explanation,
            file_names,
            cache_bytes,
            0,
        )?;

        Ok((report, approval_req))
    }

    /// Executes Fast Cleanup ONLY if a valid one-time ApprovalToken from the user is provided.
    pub fn execute_fast_cleanup(
        &self,
        token: &ApprovalToken,
    ) -> Result<FastCleanupResult, BridgeError> {
        // 1. Enforce Permission Gateway check (consumes one-time token)
        self.gateway
            .consume_approval_token(token, &ActionKind::FastCleanupExecute)?;

        // 2. Verify path boundary
        let canonical_dir = self.safe_temp_dir.canonicalize()?;
        let gpu_before = Self::query_gpu_status();

        let mut removed_count = 0usize;
        let mut freed_bytes = 0u64;

        for entry in std::fs::read_dir(&canonical_dir)? {
            let entry = entry?;
            let path = entry.path();
            let canonical_file = path.canonicalize()?;
            if !canonical_file.starts_with(&canonical_dir) {
                return Err(BridgeError::UnsafeCleanupPath(
                    canonical_file.display().to_string(),
                ));
            }
            let meta = entry.metadata()?;
            if meta.is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".tmp") || name.ends_with(".cache") {
                    let len = meta.len();
                    std::fs::remove_file(&canonical_file)?;
                    removed_count += 1;
                    freed_bytes += len;
                }
            }
        }

        let gpu_after = Self::query_gpu_status();
        let human_report = format!(
            "Fast Cleanup erfolgreich abgeschlossen: {} temporäre Cache-Dateien ({} KB) wurden bereinigt. \
             VRAM-Status: {} MiB frei von {} MiB.",
            removed_count,
            freed_bytes / 1024,
            gpu_after.vram_free_mib,
            gpu_after.vram_total_mib
        );

        Ok(FastCleanupResult {
            executed_at: Utc::now().to_rfc3339(),
            removed_files_count: removed_count,
            freed_disk_bytes: freed_bytes,
            vram_before_mib: gpu_before.vram_used_mib,
            vram_after_mib: gpu_after.vram_used_mib,
            human_report,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permission_gateway::{AccessMode, PrivacyMode};

    #[test]
    fn fast_cleanup_end_to_end_requires_approval_and_cleans_only_tmp_files() {
        let temp_root =
            std::env::temp_dir().join(format!("aido_bridge_test_{}", uuid::Uuid::new_v4()));
        let gw = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);
        let bridge = WindowsBridge::new(gw.clone(), &temp_root).unwrap();

        let seeded_bytes = bridge.seed_demo_cache_files().unwrap();
        assert_eq!(seeded_bytes, (64 + 128 + 256) * 1024);

        // Also place a non-tmp file to ensure it is NEVER deleted
        let keep_file = temp_root.join("wichtiges_dokument.txt");
        std::fs::write(&keep_file, b"Darf niemals geloescht werden").unwrap();

        let (health, req) = bridge.prepare_fast_cleanup().unwrap();
        assert_eq!(health.aido_temp_cache_files, 3);
        assert_eq!(req.target_summary.len(), 3);

        // Approve request
        let token = gw
            .approve_request(req.request_id, Some("Bitte temporäre Dateien löschen".into()))
            .unwrap();

        // Execute cleanup
        let result = bridge.execute_fast_cleanup(&token).unwrap();
        assert_eq!(result.removed_files_count, 3);
        assert_eq!(result.freed_disk_bytes, seeded_bytes);

        // Verify non-tmp file is untouched
        assert!(keep_file.exists());

        let _ = std::fs::remove_dir_all(&temp_root);
    }
}
