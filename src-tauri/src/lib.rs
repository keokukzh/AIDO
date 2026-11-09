pub mod agent_remote;
pub mod ai_client;
pub mod app_intelligence;
pub mod memory;
pub mod permission_gateway;
pub mod proactive_engine;
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
    pub proactive: proactive_engine::ProactiveEngine,
    pub remote: agent_remote::AgentRemoteEngine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeAndMemoryReport {
    pub runtime_status: runtime_supervisor::RuntimeStatusReport,
    pub compaction_report: Option<memory::CompactionReport>,
    pub active_memories_count: usize,
    pub semantic_memories_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProactiveAndDeepCleanupReport {
    pub deep_cleanup_scan: windows_bridge::DeepCleanupScanReport,
    pub deep_cleanup_approval: permission_gateway::ApprovalRequest,
    pub proactive_report: proactive_engine::ProactiveEvaluationReport,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartInstallerPlanReport {
    pub hardware_tier: smart_installer::HardwareTier,
    pub models: Vec<smart_installer::ModelReuseStatus>,
    pub eta: smart_installer::InstallerEtaForecast,
    pub steps: Vec<smart_installer::InstallerStepRecord>,
    pub next_resumable: Option<smart_installer::InstallerStepRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase7And8Report {
    pub synthesized_app: app_intelligence::AppRegistryEntry,
    pub sandbox_report: app_intelligence::AdapterSandboxReport,
    pub uninstall_plan: smart_installer::CleanUninstallPlan,
    pub release_readiness: smart_installer::ReleaseReadinessReport,
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
        let proactive = proactive_engine::ProactiveEngine::new();
        let remote = agent_remote::AgentRemoteEngine::new(db_path.as_ref())?;
        Ok(Self {
            gateway,
            memory,
            bridge,
            ai,
            apps,
            projects,
            installer,
            runtime,
            proactive,
            remote,
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

    /// Evaluates system context, active projects, episodic memories, and reclaimable build/shader caches
    /// to generate prioritized proactive suggestions under Focus-Guard rules, while also staging Deep Cleanup.
    pub fn evaluate_proactive_and_deep_cleanup(
        &self,
        activity: proactive_engine::UserActivityState,
    ) -> Result<ProactiveAndDeepCleanupReport, Box<dyn std::error::Error>> {
        let health = self.bridge.run_system_health_check()?;
        let (deep_cleanup_scan, deep_cleanup_approval) = self.bridge.prepare_deep_cleanup()?;
        let projects = self.projects.list_projects()?;
        let active_episodic_count = self
            .memory
            .list_memories_by_layer(Some("EpisodicMemory"), true, 100)?
            .len();

        let proactive_report = self.proactive.evaluate_context(
            activity,
            &health,
            &projects,
            active_episodic_count,
            deep_cleanup_scan.total_reclaimable_bytes,
        );

        let audit_summary = format!(
            "Vordenker-Prüfung: Aktivitätszustand {:?}, {} proaktive Vorschläge generiert (Fokus-Schutz unterdrückt: {}), {} KB Deep-Cleanup-Kandidaten vorbereitet.",
            activity,
            proactive_report.suggestions.len(),
            proactive_report.suppressed_by_focus_guard,
            deep_cleanup_scan.total_reclaimable_bytes / 1024
        );

        self.memory.record_audit(
            "ProactiveAndDeepCleanupCheck",
            &format!("{:?}", self.gateway.access_mode()),
            &format!("{:?}", self.gateway.privacy_mode()),
            true,
            &audit_summary,
        )?;

        Ok(ProactiveAndDeepCleanupReport {
            deep_cleanup_scan,
            deep_cleanup_approval,
            proactive_report,
        })
    }

    /// After explicit user approval, issues an ApprovalToken, deletes categorized stale build/shader/crash caches
    /// within safe sandbox boundaries, logs to audit and records an EpisodicMemory entry.
    pub fn approve_and_execute_deep_cleanup(
        &self,
        request_id: uuid::Uuid,
        user_note: Option<String>,
    ) -> Result<(ApprovalToken, FastCleanupResult), Box<dyn std::error::Error>> {
        let token = self.gateway.approve_request(request_id, user_note.clone())?;
        let result = self.bridge.execute_deep_cleanup(&token)?;

        self.memory.record_audit(
            "DeepCleanupExecute",
            &format!("{:?}", self.gateway.access_mode()),
            &format!("{:?}", self.gateway.privacy_mode()),
            true,
            &result.human_report,
        )?;

        self.memory.store_memory(
            "EpisodicMemory",
            "DeepCleanup",
            &format!(
                "Deep Cleanup nach Nutzerfreigabe ausgeführt ({} Dateien, {} KB bereinigt)",
                result.removed_files_count,
                result.freed_disk_bytes / 1024
            ),
            &format!(
                "{} | Nutzer-Notiz: {}",
                result.human_report,
                user_note.unwrap_or_else(|| "Keine".into())
            ),
            "AidoCore::approve_and_execute_deep_cleanup",
        )?;

        Ok((token, result))
    }
    /// Evaluates Phase 7 (Isolated Adapter Sandbox synthesis for unknown apps) and Phase 8 (Clean Uninstall plan
    /// and 8-Phase Release Readiness evaluation), records audit and system memory, and returns `Phase7And8Report`.
    pub fn evaluate_adapter_sandbox_and_release_readiness(
        &self,
        unknown_app_id: &str,
        unknown_app_name: &str,
        publisher: &str,
        install_path: &str,
    ) -> Result<Phase7And8Report, Box<dyn std::error::Error>> {
        let (synthesized_app, sandbox_report) = self.apps.synthesize_and_verify_unknown_app_adapter(
            &self.gateway,
            unknown_app_id,
            unknown_app_name,
            publisher,
            install_path,
            true,
        )?;

        let (uninstall_plan, _approval_req) = self.installer.prepare_clean_uninstall(
            &self.gateway,
            self.bridge.safe_temp_dir(),
            None,
        )?;

        let release_readiness = self.installer.evaluate_release_readiness(&self.gateway)?;

        let audit_summary = format!(
            "Adapter-Sandbox & Release-Readiness: App '{}' ({:?}, {} verifizierte Fähigkeiten, isoliert: {}), Release-Readiness: {} (Alle Phasen: {}, Zero-Destruction: {}).",
            synthesized_app.display_name,
            sandbox_report.synthesized_control,
            sandbox_report.verified_capabilities,
            sandbox_report.sandbox_isolated,
            release_readiness.version,
            release_readiness.all_phases_completed,
            release_readiness.zero_destruction_verified,
        );

        self.memory.record_audit(
            "AdapterSandboxAndReleaseCheck",
            &format!("{:?}", self.gateway.access_mode()),
            &format!("{:?}", self.gateway.privacy_mode()),
            true,
            &audit_summary,
        )?;

        self.memory.store_memory(
            "SystemMemory",
            "ReleaseReadiness",
            &audit_summary,
            &format!(
                "Adapter-Sandbox: {} | Uninstall-Plan: {} Pfade bereinigen, {} Modelle geschützt | Release-Status: {}",
                sandbox_report.human_summary,
                uninstall_plan.aido_owned_paths_to_remove.len(),
                uninstall_plan.preserved_external_models.len(),
                release_readiness.human_summary,
            ),
            "AidoCore::evaluate_adapter_sandbox_and_release_readiness",
        )?;

        Ok(Phase7And8Report {
            synthesized_app,
            sandbox_report,
            uninstall_plan,
            release_readiness,
        })
    }

    /// Starts a pairing challenge with a scannable QR payload, a 4-digit PIN, and a 60s TTL.
    pub fn start_remote_pairing_challenge(
        &self,
    ) -> Result<agent_remote::PairingChallenge, Box<dyn std::error::Error>> {
        let challenge = self.remote.generate_pairing_challenge("192.168.1.42", 18435, 60)?;
        self.memory.record_audit(
            "RemotePairingChallenge",
            &format!("{:?}", self.gateway.access_mode()),
            &format!("{:?}", self.gateway.privacy_mode()),
            true,
            &format!(
                "Pairing-Challenge generiert: ID={}, PIN={}, URL={}",
                challenge.challenge_id, challenge.pin_code, challenge.qr_url
            ),
        )?;
        Ok(challenge)
    }

    /// Completes pairing using the 4-digit PIN, issues a revocable session token,
    /// sets privacy mode to LocalConnected, and stores an audit & memory entry.
    pub fn complete_remote_pairing(
        &self,
        challenge_id: &str,
        pin_code: &str,
        device_name: &str,
    ) -> Result<agent_remote::PairedRemoteDevice, Box<dyn std::error::Error>> {
        let device = self.remote.verify_pairing_and_issue_session(
            &self.gateway,
            challenge_id,
            pin_code,
            device_name,
        )?;
        self.memory.record_audit(
            "PairRemoteDevice",
            &format!("{:?}", self.gateway.access_mode()),
            &format!("{:?}", self.gateway.privacy_mode()),
            true,
            &format!(
                "Smartphone-Kopplung erfolgreich: Gerät='{}', ID={}",
                device.device_name, device.device_id
            ),
        )?;
        self.memory.store_memory(
            "SystemMemory",
            "AgentRemote",
            &format!(
                "Smartphone '{}' erfolgreich per QR+PIN gekoppelt.",
                device.device_name
            ),
            &format!(
                "Geräte-ID: {}, Volle PC-Parität: {}, Token: aido_rmt_***",
                device.device_id, device.full_pc_parity
            ),
            "AidoCore::complete_remote_pairing",
        )?;
        Ok(device)
    }

    /// Authenticates a smartphone session token, runs a system health check, lists indexed projects,
    /// captures an on-demand AIDO-Blick snapshot, and builds the 4-zone RemoteCockpitState.
    pub fn build_remote_cockpit_state(
        &self,
        session_token: &str,
    ) -> Result<agent_remote::RemoteCockpitState, Box<dyn std::error::Error>> {
        let device = self.remote.authenticate_session(session_token)?;
        let health = self.bridge.run_system_health_check()?;
        let projects = self.projects.list_projects()?;
        let latest_blick = self
            .remote
            .capture_aido_blick(&self.gateway, session_token, &health, &projects)?;

        let quick_tiles = vec![
            agent_remote::RemoteQuickTile {
                tile_id: "tile_blick".into(),
                label_de: "AIDO-Blick".into(),
                subtitle_de: "Desktop-Snapshot anfordern".into(),
                action_kind: "CaptureAidoBlickSnapshot".into(),
                requires_approval: false,
            },
            agent_remote::RemoteQuickTile {
                tile_id: "tile_fast_cleanup".into(),
                label_de: "Fast Cleanup".into(),
                subtitle_de: "Temp-Caches freigeben".into(),
                action_kind: "FastCleanupExecute".into(),
                requires_approval: true,
            },
            agent_remote::RemoteQuickTile {
                tile_id: "tile_deep_cleanup".into(),
                label_de: "Deep Cleanup".into(),
                subtitle_de: "Stale Artefakte prüfen".into(),
                action_kind: "DeepCleanupExecute".into(),
                requires_approval: true,
            },
            agent_remote::RemoteQuickTile {
                tile_id: "tile_project_launch".into(),
                label_de: "Projekt Start".into(),
                subtitle_de: "Empfohlene Aktion starten".into(),
                action_kind: "LaunchProject".into(),
                requires_approval: true,
            },
        ];

        let reply_chips_de = vec![
            "Ja, jetzt freigeben".to_string(),
            "AIDO-Blick aktualisieren".to_string(),
            "Später erinnern".to_string(),
        ];

        let pending_approvals_count = self.gateway.pending_requests_count();

        Ok(agent_remote::RemoteCockpitState {
            device_name: device.device_name,
            privacy_mode: format!("{:?}", self.gateway.privacy_mode()),
            full_pc_parity: device.full_pc_parity,
            vram_used_mib: health.gpu.vram_used_mib,
            vram_total_mib: health.gpu.vram_total_mib,
            ram_free_mb: health.system_ram_free_mb,
            pending_approvals_count,
            quick_tiles,
            latest_blick,
            reply_chips_de,
        })
    }

    /// Executes an explicit user approval initiated via the paired smartphone cockpit,
    /// enforcing full PC parity, issuing an ApprovalToken, recording an audit log and an EpisodicMemory entry.
    pub fn execute_remote_approval(
        &self,
        session_token: &str,
        request_id: uuid::Uuid,
        user_note: Option<String>,
    ) -> Result<(ApprovalToken, String), Box<dyn std::error::Error>> {
        let device = self.remote.authenticate_session(session_token)?;
        if !device.full_pc_parity {
            return Err("Gekoppeltes Gerät verfügt über keine volle PC-Parität".into());
        }
        let note = Some(format!(
            "[Remote:{}] {}",
            device.device_name,
            user_note.unwrap_or_else(|| "Per Daumen-Cockpit freigegeben".into())
        ));
        let token = self.gateway.approve_request(request_id, note)?;
        let confirmation = format!(
            "Aktion '{:?}' erfolgreich per Smartphone-Cockpit ('{}') freigegeben.",
            token.action, device.device_name
        );
        self.memory.record_audit(
            "RemoteApprovalExecute",
            &format!("{:?}", self.gateway.access_mode()),
            &format!("{:?}", self.gateway.privacy_mode()),
            true,
            &format!(
                "Remote-Freigabe erteilt für Request-ID {}: Gerät='{}'",
                request_id, device.device_name
            ),
        )?;
        self.memory.store_memory(
            "EpisodicMemory",
            "AgentRemote",
            &format!("Remote-Freigabe für {:?} erteilt.", token.action),
            &confirmation,
            "AidoCore::execute_remote_approval",
        )?;
        Ok((token, confirmation))
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

    #[test]
    fn aido_core_proactive_engine_and_deep_cleanup_end_to_end() {
        let temp_dir = std::env::temp_dir();
        let unique = Uuid::new_v4().to_string();
        let db_path = temp_dir.join(format!("aido_proactive_test_{}.db", unique));
        let safe_temp_dir = temp_dir.join(format!("aido_proactive_staging_{}", unique));
        let workspace_root = temp_dir.join(format!("aido_proactive_workspace_{}", unique));
        std::fs::create_dir_all(&workspace_root).expect("Failed to create workspace root");

        // Create a project inside workspace_root with git init and untracked file to trigger dirty status
        let project_dir = workspace_root.join("AidoDirtyProject");
        std::fs::create_dir_all(&project_dir).expect("Failed to create project dir");
        std::fs::write(
            project_dir.join("Cargo.toml"),
            "[package]\nname = \"aido_dirty\"\nversion = \"0.1.0\"\n",
        )
        .expect("Failed to write Cargo.toml");
        let _ = std::process::Command::new("git")
            .args(["init"])
            .current_dir(&project_dir)
            .output();
        let _ = std::fs::write(project_dir.join("uncommitted.txt"), "dirty worktree state");

        let core = AidoCore::new(
            AccessMode::StandardAccess,
            PrivacyMode::PrivateOffline,
            &db_path,
            &safe_temp_dir,
        )
        .expect("AidoCore::new failed");

        // Index project so list_projects has it
        let (_, indexed_projects) = core
            .discover_and_register_environment(std::slice::from_ref(&workspace_root))
            .expect("discover_and_register_environment failed");
        assert!(!indexed_projects.is_empty());

        // Seed 3 deep cleanup files (1152 KB)
        let seeded_bytes = core
            .bridge
            .seed_demo_deep_cleanup_files()
            .expect("seed_demo_deep_cleanup_files failed");
        assert_eq!(seeded_bytes, 1152 * 1024);

        // Store 4 episodic memories to trigger memory consolidation suggestion
        for i in 1..=4 {
            core.memory
                .store_memory(
                    "EpisodicMemory",
                    "FastCleanup",
                    &format!("Cleanup {}", i),
                    "Details",
                    "test",
                )
                .expect("store_memory failed");
        }

        // 1. Test Focus-Guard suppression during Gaming/Fullscreen
        let suppressed = core
            .evaluate_proactive_and_deep_cleanup(
                proactive_engine::UserActivityState::FullscreenOrGaming,
            )
            .expect("evaluate_proactive_and_deep_cleanup suppressed failed");
        assert!(suppressed.proactive_report.suppressed_by_focus_guard);
        assert!(suppressed.proactive_report.suggestions.is_empty());
        assert!(suppressed
            .proactive_report
            .suppression_reason
            .as_ref()
            .unwrap()
            .contains("Fokus-Schutz"));

        // 2. Test NormalIdle evaluation with proactive suggestions and deep cleanup scan
        let normal = core
            .evaluate_proactive_and_deep_cleanup(proactive_engine::UserActivityState::NormalIdle)
            .expect("evaluate_proactive_and_deep_cleanup normal failed");
        assert!(!normal.proactive_report.suppressed_by_focus_guard);
        assert!(!normal.proactive_report.suggestions.is_empty());
        assert_eq!(normal.deep_cleanup_scan.candidates.len(), 3);
        assert_eq!(
            normal.deep_cleanup_scan.total_reclaimable_bytes,
            1152 * 1024
        );
        assert_eq!(
            normal.deep_cleanup_approval.action,
            permission_gateway::ActionKind::DeepCleanupExecute
        );

        // 3. Test Deep Cleanup execution with approval
        let (token, cleanup_result) = core
            .approve_and_execute_deep_cleanup(
                normal.deep_cleanup_approval.request_id,
                Some("Nutzer hat Tiefenbereinigung freigegeben".into()),
            )
            .expect("approve_and_execute_deep_cleanup failed");

        assert_eq!(
            token.action,
            permission_gateway::ActionKind::DeepCleanupExecute
        );
        assert_eq!(cleanup_result.removed_files_count, 3);
        assert_eq!(cleanup_result.freed_disk_bytes, 1152 * 1024);
        assert!(cleanup_result.human_report.contains("Deep Cleanup erfolgreich"));

        // 4. Verify audit log & episodic memory
        let audits = core.memory.list_audit_log(10).expect("list_audit_log failed");
        assert!(audits.iter().any(|a| a.action == "DeepCleanupExecute"));
        assert!(audits
            .iter()
            .any(|a| a.action == "ProactiveAndDeepCleanupCheck"));

        let memories = core
            .memory
            .search_memory("DeepCleanup", 5)
            .expect("search_memory failed");
        assert!(!memories.is_empty());
        assert_eq!(memories[0].layer, "EpisodicMemory");
        assert_eq!(memories[0].category, "DeepCleanup");

        // Cleanup
        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_dir_all(&safe_temp_dir);
        let _ = std::fs::remove_dir_all(&workspace_root);
    }

    #[test]
    fn aido_core_adapter_sandbox_and_clean_uninstall_preserve_models() {
        let temp_dir = std::env::temp_dir();
        let unique = Uuid::new_v4().to_string();
        let db_path = temp_dir.join(format!("aido_release_test_{}.db", unique));
        let safe_temp_dir = temp_dir.join(format!("aido_release_staging_{}", unique));
        let external_models_dir = temp_dir.join(format!("aido_external_models_{}", unique));
        let test_cache_dir = temp_dir.join(format!("aido_test_cache_{}", unique));

        std::fs::create_dir_all(&external_models_dir).expect("Failed to create external models dir");
        std::fs::create_dir_all(&test_cache_dir).expect("Failed to create test cache dir");

        let dummy_model_path = external_models_dir.join("Qwen3.8-27B-UD-Q4_K_M.gguf");
        let model_bytes = b"MOCK_MODEL_WEIGHTS_DO_NOT_DELETE";
        std::fs::write(&dummy_model_path, model_bytes).expect("Failed to write mock model");

        let test_cache_file = test_cache_dir.join("staging_cache.tmp");
        std::fs::write(&test_cache_file, b"TEMP_DATA_TO_CLEAN").expect("Failed to write test cache file");

        let core = AidoCore::new(
            AccessMode::StandardAccess,
            PrivacyMode::PrivateOffline,
            &db_path,
            &safe_temp_dir,
        )
        .expect("AidoCore::new failed");

        // 1. Evaluate Adapter Sandbox & Release Readiness
        let report = core
            .evaluate_adapter_sandbox_and_release_readiness(
                "obs-studio",
                "OBS Studio",
                "OBS Project",
                r"C:\Program Files\obs-studio\bin\64bit\obs64.exe",
            )
            .expect("evaluate_adapter_sandbox_and_release_readiness failed");

        // Assert sandbox verification metrics
        assert!(report.sandbox_report.harmless_probe_passed);
        assert!(report.sandbox_report.sandbox_isolated);
        assert_eq!(report.sandbox_report.verified_capabilities, 2);
        assert_eq!(report.sandbox_report.restricted_capabilities, 1);
        assert_eq!(report.synthesized_app.capabilities.len(), 3);
        assert_eq!(
            report.synthesized_app.adapter_origin,
            app_intelligence::AdapterOrigin::AutoGenerated
        );

        // Assert release readiness report
        assert_eq!(report.release_readiness.version, "0.1.0-mvp");
        assert!(report.release_readiness.all_phases_completed);
        assert!(report.release_readiness.security_gate_passed);
        assert!(report.release_readiness.privacy_offline_verified);
        assert!(report.release_readiness.zero_destruction_verified);
        assert_eq!(report.release_readiness.checklist_items.len(), 8);

        // 2. Prepare and execute clean uninstall with custom model specs
        let custom_specs = [(
            "LLM",
            "Qwen3.8-27B",
            dummy_model_path.as_path(),
            model_bytes.len() as u64,
        )];

        let (uninstall_plan, approval_req) = core
            .installer
            .prepare_clean_uninstall(&core.gateway, &test_cache_dir, Some(&custom_specs))
            .expect("prepare_clean_uninstall failed");

        assert_eq!(
            approval_req.action,
            permission_gateway::ActionKind::ExecuteCleanUninstall
        );
        assert!(uninstall_plan
            .preserved_external_models
            .contains(&dummy_model_path.display().to_string()));

        let token = core
            .gateway
            .approve_request(
                approval_req.request_id,
                Some("Freigabe für saubere Deinstallation im Integrationstest".into()),
            )
            .expect("approve_request failed");

        let uninstall_result = core
            .installer
            .execute_clean_uninstall(&core.gateway, &token, &test_cache_dir)
            .expect("execute_clean_uninstall failed");

        assert!(uninstall_result.contains("Saubere Deinstallation abgeschlossen"));
        // AIDO test cache dir must be removed
        assert!(!test_cache_dir.exists(), "AIDO test cache directory must be removed");

        // External model file MUST be 100% intact!
        assert!(dummy_model_path.exists(), "External model file MUST exist and NOT be deleted!");
        let read_back_bytes = std::fs::read(&dummy_model_path).expect("Failed to read dummy model");
        assert_eq!(read_back_bytes, model_bytes, "Model contents must remain 100 % intact!");

        // 3. Verify audit log & memory
        let audits = core.memory.list_audit_log(10).expect("list_audit_log failed");
        assert!(
            audits.iter().any(|a| a.action == "AdapterSandboxAndReleaseCheck"),
            "Expected 'AdapterSandboxAndReleaseCheck' in audit log"
        );

        let memories = core
            .memory
            .search_memory("ReleaseReadiness", 5)
            .expect("search_memory failed");
        assert!(
            !memories.is_empty(),
            "Expected 'ReleaseReadiness' in memory"
        );
        assert_eq!(memories[0].layer, "SystemMemory");
        assert_eq!(memories[0].category, "ReleaseReadiness");

        // Cleanup
        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_dir_all(&safe_temp_dir);
        let _ = std::fs::remove_dir_all(&external_models_dir);
    }

    #[test]
    fn aido_core_agent_remote_qr_pairing_full_parity_approval_and_revoke() {
        let temp_dir = std::env::temp_dir();
        let unique = Uuid::new_v4().to_string();
        let db_path = temp_dir.join(format!("aido_remote_integration_{}.db", unique));
        let safe_temp_dir = temp_dir.join(format!("aido_remote_staging_{}", unique));

        let core = AidoCore::new(
            AccessMode::StandardAccess,
            PrivacyMode::PrivateOffline,
            &db_path,
            &safe_temp_dir,
        )
        .expect("AidoCore::new failed");

        // 1. Initial mode is PrivateOffline
        assert_eq!(core.gateway.privacy_mode(), PrivacyMode::PrivateOffline);

        // 2. Generate pairing challenge
        let challenge = core
            .start_remote_pairing_challenge()
            .expect("start_remote_pairing_challenge failed");
        assert_eq!(challenge.pin_code.len(), 4);
        assert!(challenge.qr_url.contains("192.168.1.42:18435/remote?ticket="));

        // 3. Complete pairing
        let device = core
            .complete_remote_pairing(
                &challenge.challenge_id,
                &challenge.pin_code,
                "iPhone 16 Pro · Test Remote",
            )
            .expect("complete_remote_pairing failed");

        assert_eq!(device.device_name, "iPhone 16 Pro · Test Remote");
        assert!(device.full_pc_parity);
        assert_eq!(core.gateway.privacy_mode(), PrivacyMode::LocalConnected);

        // 4. Create an approval request and build cockpit state
        let req = core
            .gateway
            .request_approval(
                permission_gateway::ActionKind::FastCleanupExecute,
                "Fast Cleanup",
                "Bereinigt temporäre Caches",
                vec!["temp_cache".into()],
                1024,
                0,
            )
            .expect("request_approval failed");

        let cockpit = core
            .build_remote_cockpit_state(&device.session_token)
            .expect("build_remote_cockpit_state failed");

        assert_eq!(cockpit.pending_approvals_count, 1);
        assert!(cockpit.full_pc_parity);
        assert_eq!(cockpit.quick_tiles.len(), 4);
        assert_eq!(cockpit.reply_chips_de.len(), 3);
        assert!(!cockpit.latest_blick.vision_summary_de.is_empty());

        // 5. Remote approval execution
        let (token, confirmation) = core
            .execute_remote_approval(
                &device.session_token,
                req.request_id,
                Some("Freigabe vom Sofa per Daumen".into()),
            )
            .expect("execute_remote_approval failed");

        assert_eq!(token.action, permission_gateway::ActionKind::FastCleanupExecute);
        assert!(confirmation.contains("iPhone 16 Pro · Test Remote"));

        // Consume approval token
        assert!(core
            .gateway
            .consume_approval_token(&token, &permission_gateway::ActionKind::FastCleanupExecute)
            .is_ok());

        // Verify memory & audit log
        let audits = core.memory.list_audit_log(10).expect("list_audit_log failed");
        assert!(audits.iter().any(|a| a.action == "RemotePairingChallenge"));
        assert!(audits.iter().any(|a| a.action == "PairRemoteDevice"));
        assert!(audits.iter().any(|a| a.action == "RemoteApprovalExecute"));

        let memories = core
            .memory
            .search_memory("AgentRemote", 5)
            .expect("search_memory failed");
        assert!(!memories.is_empty());

        // 6. Revoke device
        let revoked = core
            .remote
            .revoke_device(&core.gateway, &device.device_id)
            .expect("revoke_device failed");
        assert!(revoked);

        // Privacy mode restored to PrivateOffline
        assert_eq!(core.gateway.privacy_mode(), PrivacyMode::PrivateOffline);

        // Session no longer authorized
        assert!(core.remote.authenticate_session(&device.session_token).is_err());

        // Cleanup
        let _ = std::fs::remove_file(&db_path);
        let _ = std::fs::remove_dir_all(&safe_temp_dir);
    }
}
