pub mod ai_client;
pub mod app_intelligence;
pub mod memory;
pub mod permission_gateway;
pub mod project_intelligence;
pub mod smart_installer;
pub mod windows_bridge;

use ai_client::{AiResponseWithOptions, LocalAiClient};
use app_intelligence::{AppIntelligenceEngine, AppRegistryEntry};
use memory::MemoryEngine;
use permission_gateway::{
    AccessMode, ApprovalRequest, ApprovalToken, PermissionGateway, PrivacyMode,
};
use project_intelligence::{ProjectEntry, ProjectIntelligenceEngine};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
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
    pub apps: AppIntelligenceEngine,
    pub projects: ProjectIntelligenceEngine,
}

impl AidoCore {
    pub fn new(
        access_mode: AccessMode,
        privacy_mode: PrivacyMode,
        db_path: impl AsRef<Path>,
        safe_temp_dir: impl AsRef<Path>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let gateway = PermissionGateway::new(access_mode, privacy_mode);
        let memory = MemoryEngine::new(db_path.as_ref())?;
        let bridge = WindowsBridge::new(gateway.clone(), safe_temp_dir)?;
        let ai = LocalAiClient::new(gateway.clone());
        let apps = AppIntelligenceEngine::new(db_path.as_ref())?;
        let projects = ProjectIntelligenceEngine::new(db_path.as_ref())?;
        Ok(Self {
            gateway,
            memory,
            bridge,
            ai,
            apps,
            projects,
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

    /// Discovers known Windows developer applications and indexes project directories,
    /// logs the discovery to audit and memory, and returns the discovered entries.
    pub fn discover_and_register_environment(
        &self,
        project_roots: &[PathBuf],
    ) -> Result<(Vec<AppRegistryEntry>, Vec<ProjectEntry>), Box<dyn std::error::Error>> {
        let discovered_apps = self.apps.discover_known_windows_apps(&self.gateway)?;
        let discovered_projects = self.projects.index_roots(&self.gateway, project_roots, 3)?;

        let audit_summary = format!(
            "Umgebungserkennung abgeschlossen: {} bekannte Apps, {} Projekte indiziert.",
            discovered_apps.len(),
            discovered_projects.len()
        );

        self.memory.record_audit(
            "EnvironmentDiscovery",
            &format!("{:?}", self.gateway.access_mode()),
            &format!("{:?}", self.gateway.privacy_mode()),
            true,
            &audit_summary,
        )?;

        let app_names: Vec<String> = discovered_apps.iter().map(|a| a.display_name.clone()).collect();
        let project_names: Vec<String> = discovered_projects.iter().map(|p| p.name.clone()).collect();
        let memory_details = format!(
            "Erkannte Apps: {} | Indizierte Projekte: {}",
            app_names.join(", "),
            project_names.join(", ")
        );

        self.memory.store_memory(
            "SystemMemory",
            "Discovery",
            &audit_summary,
            &memory_details,
            "AidoCore::discover_and_register_environment",
        )?;

        Ok((discovered_apps, discovered_projects))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn aido_core_discovers_apps_and_projects_and_logs_to_memory() {
        let temp_dir = std::env::temp_dir();
        let unique = Uuid::new_v4().to_string();
        let db_path = temp_dir.join(format!("aido_integration_test_{}.db", unique));
        let safe_temp_dir = temp_dir.join(format!("aido_integration_staging_{}", unique));
        let workspace_root = temp_dir.join(format!("aido_integration_workspace_{}", unique));
        std::fs::create_dir_all(&workspace_root).expect("Failed to create workspace root");

        // Create a dummy project inside workspace_root
        let project_dir = workspace_root.join("AidoTestProject");
        std::fs::create_dir_all(&project_dir).expect("Failed to create project dir");
        std::fs::write(
            project_dir.join("Cargo.toml"),
            "[package]\nname = \"aido_test\"\nversion = \"0.1.0\"\n",
        )
        .expect("Failed to write Cargo.toml");

        let core = AidoCore::new(
            AccessMode::StandardAccess,
            PrivacyMode::PrivateOffline,
            &db_path,
            &safe_temp_dir,
        )
        .expect("AidoCore::new failed");

        // 1. discover_and_register_environment
        let (apps, projects) = core
            .discover_and_register_environment(std::slice::from_ref(&workspace_root))
            .expect("discover_and_register_environment failed");

        assert_eq!(apps.len(), 4, "Should discover 4 registered known apps");
        assert!(
            projects.iter().any(|p| p.name == "AidoTestProject"),
            "Should find AidoTestProject in projects"
        );

        // 2. prepare_project_launch
        let (launch_project, approval_request) = core
            .projects
            .prepare_project_launch(&core.gateway, "AidoTestProject")
            .expect("prepare_project_launch failed");

        assert_eq!(launch_project.name, "AidoTestProject");
        assert_eq!(
            approval_request.action,
            permission_gateway::ActionKind::LaunchProject
        );
        assert!(approval_request
            .target_summary
            .contains(&"AidoTestProject".to_string()));

        // 3. memory.search_memory
        let memories = core
            .memory
            .search_memory("Umgebungserkennung", 5)
            .expect("search_memory failed");
        assert!(
            !memories.is_empty(),
            "Expected to find 'Umgebungserkennung' in memory"
        );
        assert_eq!(memories[0].layer, "SystemMemory");
        assert_eq!(memories[0].category, "Discovery");

        let audits = core
            .memory
            .list_audit_log(5)
            .expect("list_audit_log failed");
        assert!(
            audits.iter().any(|a| a.action == "EnvironmentDiscovery"),
            "Expected 'EnvironmentDiscovery' in audit log"
        );

        // Cleanup
        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_dir_all(&safe_temp_dir);
        let _ = std::fs::remove_dir_all(&workspace_root);
    }
}
