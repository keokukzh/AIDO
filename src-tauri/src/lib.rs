pub mod ai_client;
pub mod app_intelligence;
pub mod memory;
pub mod permission_gateway;
pub mod project_intelligence;
pub mod runtime_supervisor;
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
    pub installer: smart_installer::SmartInstallerEngine,
    pub runtime: runtime_supervisor::RuntimeSupervisor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeAndMemoryReport {
    pub runtime_status: runtime_supervisor::RuntimeStatusReport,
    pub compaction_report: Option<memory::CompactionReport>,
    pub active_memories_count: usize,
    pub semantic_memories_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartInstallerPlanReport {
    pub hardware_tier: smart_installer::HardwareTier,
    pub models: Vec<smart_installer::ModelReuseStatus>,
    pub eta: smart_installer::InstallerEtaForecast,
    pub steps: Vec<smart_installer::InstallerStepRecord>,
    pub next_resumable: Option<smart_installer::InstallerStepRecord>,
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
        let installer = smart_installer::SmartInstallerEngine::new(db_path.as_ref())?;
        let runtime = runtime_supervisor::RuntimeSupervisor::new();
        Ok(Self {
            gateway,
            memory,
            bridge,
            ai,
            apps,
            projects,
            installer,
            runtime,
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

    /// Evaluates the smart installer plan: classifies hardware tier based on GPU/RAM health check,
    /// inspects existing local AI models for reuse, calculates 3-stage ETA forecast,
    /// retrieves current installation phase status, and logs to audit and memory.
    pub fn evaluate_smart_installer_plan(
        &self,
        selected_apps_count: usize,
        unknown_adapters_count: usize,
    ) -> Result<SmartInstallerPlanReport, Box<dyn std::error::Error>> {
        let health = self.bridge.run_system_health_check()?;
        let tier = smart_installer::SmartInstallerEngine::classify_hardware(
            health.gpu.vram_total_mib,
            health.system_ram_total_mb,
        );
        let models = self.installer.inspect_existing_models(&self.gateway, None)?;
        let eta = smart_installer::SmartInstallerEngine::calculate_eta(
            &tier,
            &models,
            selected_apps_count,
            unknown_adapters_count,
            250,
        );
        let steps = self.installer.list_steps()?;
        let next_resumable = self.installer.next_resumable_step()?;

        let audit_summary = format!(
            "Smart Installer Plan: Hardware-Klasse {:?}, {}/{} Modelle lokal wiederverwendet ({} Bytes Download nötig), AIDO einsatzbereit in {}s.",
            tier, eta.reused_models_count, models.len(), eta.remaining_download_bytes, eta.ready_to_wake_seconds
        );

        self.memory.record_audit(
            "SmartInstallerPlan",
            &format!("{:?}", self.gateway.access_mode()),
            &format!("{:?}", self.gateway.privacy_mode()),
            true,
            &audit_summary,
        )?;

        self.memory.store_memory(
            "SystemMemory",
            "SmartInstaller",
            &audit_summary,
            &eta.human_summary,
            "AidoCore::evaluate_smart_installer_plan",
        )?;

        Ok(SmartInstallerPlanReport {
            hardware_tier: tier,
            models,
            eta,
            steps,
            next_resumable,
        })
    }

    /// Step 7 of the vertical architecture:
    /// Probes the local LLM and Whisper runtime supervisors, evaluates and compacts episodic memories
    /// into semantic knowledge when threshold is met, counts active and semantic entries, and records an audit log.
    pub fn inspect_runtime_and_compact_memory(
        &self,
        min_episodic_to_compact: usize,
    ) -> Result<RuntimeAndMemoryReport, Box<dyn std::error::Error>> {
        let runtime_status = self.runtime.probe_status(&self.gateway)?;
        let compaction_report = self
            .memory
            .compact_episodic_memories(None, min_episodic_to_compact)?;

        let active_memories = self.memory.list_memories_by_layer(None, true, 1000)?;
        let semantic_memories = self
            .memory
            .list_memories_by_layer(Some("SemanticMemory"), true, 1000)?;

        let active_memories_count = active_memories.len();
        let semantic_memories_count = semantic_memories.len();

        let audit_summary = format!(
            "Runtime- & Memory-Status: Llama: {:?}, Whisper: {:?}, CUDA-DLL: {}, {} aktive Erinnerungen, {} semantische Verdichtungen, Kompaktierung: {}",
            runtime_status.llama_router.state,
            runtime_status.whisper_asr.state,
            if runtime_status.llama_router.cuda_dll_ready { "Gefunden" } else { "Fehlt" },
            active_memories_count,
            semantic_memories_count,
            if compaction_report.is_some() { "Ausgeführt" } else { "Keine Schwellenwert-Überschreitung" }
        );

        self.memory.record_audit(
            "RuntimeAndMemoryCheck",
            &format!("{:?}", self.gateway.access_mode()),
            &format!("{:?}", self.gateway.privacy_mode()),
            true,
            &audit_summary,
        )?;

        Ok(RuntimeAndMemoryReport {
            runtime_status,
            compaction_report,
            active_memories_count,
            semantic_memories_count,
        })
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

    #[test]
    fn aido_core_smart_installer_plan_reuses_models_and_supports_resume() {
        let temp_dir = std::env::temp_dir();
        let unique = Uuid::new_v4().to_string();
        let db_path = temp_dir.join(format!("aido_installer_integration_{}.db", unique));
        let safe_temp_dir = temp_dir.join(format!("aido_installer_staging_{}", unique));

        let core = AidoCore::new(
            AccessMode::StandardAccess,
            PrivacyMode::PrivateOffline,
            &db_path,
            &safe_temp_dir,
        )
        .expect("AidoCore::new failed");

        // 1. Initial plan evaluation
        let plan = core
            .evaluate_smart_installer_plan(4, 1)
            .expect("evaluate_smart_installer_plan failed");

        assert_eq!(plan.steps.len(), 9, "Installer should have 9 distinct phases");
        assert!(plan.next_resumable.is_some(), "Next resumable step should exist initially");
        assert_eq!(plan.next_resumable.as_ref().unwrap().step_index, 1);
        assert_eq!(plan.models.len(), 4, "Should inspect 4 default models");

        // 2. Complete Step 1 and pause at Step 2 to test resume
        let step1 = core
            .installer
            .complete_step(1, "Datenschutz & Offline-Bestätigung erfolgreich abgeschlossen")
            .expect("complete_step 1 failed");
        assert_eq!(step1.status, smart_installer::InstallerStepStatus::Completed);

        let step2 = core
            .installer
            .pause_at_step(2, "Nutzer pausierte für GPU-Treiber-Aktualisierung")
            .expect("pause_at_step 2 failed");
        assert_eq!(step2.status, smart_installer::InstallerStepStatus::Paused);

        // 3. Re-evaluate plan and verify resume capability
        let plan_resumed = core
            .evaluate_smart_installer_plan(4, 1)
            .expect("evaluate_smart_installer_plan resumed failed");

        let next = plan_resumed
            .next_resumable
            .expect("Expected a resumable step");
        assert_eq!(
            next.step_index, 2,
            "Next resumable step must be the paused Step 2"
        );
        assert_eq!(next.status, smart_installer::InstallerStepStatus::Paused);

        // 4. Verify audit log entry
        let audits = core
            .memory
            .list_audit_log(5)
            .expect("list_audit_log failed");
        assert!(
            audits.iter().any(|a| a.action == "SmartInstallerPlan"),
            "Expected 'SmartInstallerPlan' in audit log"
        );

        // 5. Verify memory entry
        let memories = core
            .memory
            .search_memory("SmartInstaller", 5)
            .expect("search_memory failed");
        assert!(
            !memories.is_empty(),
            "Expected 'SmartInstaller' entry in memory"
        );
        assert_eq!(memories[0].layer, "SystemMemory");
        assert_eq!(memories[0].category, "SmartInstaller");

        // Cleanup
        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_dir_all(&safe_temp_dir);
    }

    #[test]
    fn aido_core_runtime_supervisor_vision_and_memory_compaction() {
        let temp_dir = std::env::temp_dir();
        let unique = Uuid::new_v4().to_string();
        let db_path = temp_dir.join(format!("aido_runtime_test_{}.db", unique));
        let safe_temp_dir = temp_dir.join(format!("aido_runtime_staging_{}", unique));

        let core = AidoCore::new(
            AccessMode::StandardAccess,
            PrivacyMode::PrivateOffline,
            &db_path,
            &safe_temp_dir,
        )
        .expect("AidoCore::new failed");

        // 1. Store 3 EpisodicMemory entries
        core.memory
            .store_memory(
                "EpisodicMemory",
                "FastCleanup",
                "Cleanup 1 ausgeführt",
                "Details zu Cleanup 1",
                "test",
            )
            .expect("store_memory 1 failed");
        core.memory
            .store_memory(
                "EpisodicMemory",
                "FastCleanup",
                "Cleanup 2 ausgeführt",
                "Details zu Cleanup 2",
                "test",
            )
            .expect("store_memory 2 failed");
        core.memory
            .store_memory(
                "EpisodicMemory",
                "FastCleanup",
                "Cleanup 3 ausgeführt",
                "Details zu Cleanup 3",
                "test",
            )
            .expect("store_memory 3 failed");

        // 2. Inspect runtime and compact memory (threshold 2)
        let report = core
            .inspect_runtime_and_compact_memory(2)
            .expect("inspect_runtime_and_compact_memory failed");

        assert!(
            report.compaction_report.is_some(),
            "Expected a CompactionReport since 3 entries >= 2"
        );
        let compaction = report.compaction_report.as_ref().unwrap();
        assert_eq!(compaction.compacted_source_entries, 3);
        assert_eq!(compaction.category, "FastCleanup");
        assert_eq!(report.semantic_memories_count, 1);

        // Verify SemanticMemory has the consolidated entry
        let semantic = core
            .memory
            .list_memories_by_layer(Some("SemanticMemory"), true, 10)
            .expect("list_memories_by_layer failed");
        assert_eq!(semantic.len(), 1);
        assert_eq!(semantic[0].id, compaction.created_semantic_entry_id);

        // Verify audit log has RuntimeAndMemoryCheck
        let audits = core.memory.list_audit_log(5).expect("list_audit_log failed");
        assert!(
            audits.iter().any(|a| a.action == "RuntimeAndMemoryCheck"),
            "Expected 'RuntimeAndMemoryCheck' in audit log"
        );

        // 3. Test multimodal vision payload generation
        let vision_payload = core.ai.build_vision_chat_payload(
            "Beschreibe diesen Screenshot",
            "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
        );
        assert!(vision_payload.get("messages").is_some());

        // 4. Test whisper voice note transcription
        let asr_result = core
            .ai
            .transcribe_voice_note("AIDO öffne Docker", 1.2)
            .expect("transcribe_voice_note failed");
        assert_eq!(asr_result.language, "de");
        assert!(asr_result.local_only);
        assert!(asr_result.engine.contains("whisper-server.exe"));

        // Cleanup
        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_dir_all(&safe_temp_dir);
    }
}
