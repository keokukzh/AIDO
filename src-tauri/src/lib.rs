pub mod ai_client;
pub mod app_intelligence;
pub mod memory;
pub mod permission_gateway;
pub mod project_intelligence;
pub mod windows_bridge;

use ai_client::{AiResponseWithOptions, LocalAiClient};
use memory::MemoryEngine;
use permission_gateway::{
    AccessMode, ApprovalRequest, ApprovalToken, PermissionGateway, PrivacyMode,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use windows_bridge::{FastCleanupResult, SystemHealthReport, WindowsBridge};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AidoWorkflowInspection {
    pub health_report: SystemHealthReport,
    pub ai_proposal: AiResponseWithOptions,
    pub approval_request: ApprovalRequest,
}

pub struct AidoCore {
    pub gateway: PermissionGateway,
    pub memory: MemoryEngine,
    pub bridge: WindowsBridge,
    pub ai: LocalAiClient,
}

impl AidoCore {
    pub fn new(
        access_mode: AccessMode,
        privacy_mode: PrivacyMode,
        db_path: impl AsRef<Path>,
        safe_temp_dir: impl AsRef<Path>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let gateway = PermissionGateway::new(access_mode, privacy_mode);
        let memory = MemoryEngine::new(db_path)?;
        let bridge = WindowsBridge::new(gateway.clone(), safe_temp_dir)?;
        let ai = LocalAiClient::new(gateway.clone());
        Ok(Self {
            gateway,
            memory,
            bridge,
            ai,
        })
    }

    /// Step 1 of the MVP vertical slice:
    /// Performs a read-only System Health Check, asks the local AI to explain it in plain German,
    /// creates a formal ApprovalRequest in the Permission Gateway, and logs the inspection.
    pub fn inspect_and_propose_fast_cleanup(
        &self,
        api_key: Option<&str>,
    ) -> Result<AidoWorkflowInspection, Box<dyn std::error::Error>> {
        let (health_report, approval_request) = self.bridge.prepare_fast_cleanup()?;
        let ai_proposal = self
            .ai
            .explain_health_and_propose_cleanup(&health_report.human_summary, api_key)?;

        self.memory.record_audit(
            "FastCleanupPreview",
            &format!("{:?}", self.gateway.access_mode()),
            &format!("{:?}", self.gateway.privacy_mode()),
            true,
            &format!(
                "Systemcheck durchgeführt: {} Dateien ({} Bytes) im AIDO-Temp-Cache gefunden.",
                health_report.aido_temp_cache_files, health_report.aido_temp_cache_bytes
            ),
        )?;

        Ok(AidoWorkflowInspection {
            health_report,
            ai_proposal,
            approval_request,
        })
    }

    /// Step 2 of the MVP vertical slice:
    /// After the user clicks "Ja, jetzt bereinigen", issues a one-time ApprovalToken,
    /// executes the Fast Cleanup via the Windows Bridge, and stores the outcome in SQLite/FTS5 Memory.
    pub fn approve_and_execute_fast_cleanup(
        &self,
        request_id: uuid::Uuid,
        user_note: Option<String>,
    ) -> Result<(ApprovalToken, FastCleanupResult), Box<dyn std::error::Error>> {
        let token = self.gateway.approve_request(request_id, user_note.clone())?;
        let result = self.bridge.execute_fast_cleanup(&token)?;

        self.memory.record_audit(
            "FastCleanupExecute",
            &format!("{:?}", self.gateway.access_mode()),
            &format!("{:?}", self.gateway.privacy_mode()),
            true,
            &result.human_report,
        )?;

        self.memory.store_memory(
            "EpisodicMemory",
            "FastCleanup",
            &format!(
                "Fast Cleanup nach Nutzerfreigabe ausgeführt ({} Dateien, {} KB bereinigt)",
                result.removed_files_count,
                result.freed_disk_bytes / 1024
            ),
            &format!(
                "{} | Nutzer-Notiz: {}",
                result.human_report,
                user_note.unwrap_or_else(|| "Keine".into())
            ),
            "AidoCore::approve_and_execute_fast_cleanup",
        )?;

        Ok((token, result))
    }
}
