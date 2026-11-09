use aido_core::permission_gateway::{AccessMode, PrivacyMode};
use aido_core::AidoCore;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let base_dir = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir())
        .join("AIDO");

    let db_path = base_dir.join("data").join("aido_memory.db");
    let safe_temp_dir = base_dir.join("cache").join("fast_cleanup_staging");

    let core = AidoCore::new(
        AccessMode::StandardAccess,
        PrivacyMode::PrivateOffline,
        &db_path,
        &safe_temp_dir,
    )?;

    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("demo");

    match cmd {
        "health" => {
            let report = core.bridge.run_system_health_check()?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        "apps" => {
            let apps = core.apps.discover_known_windows_apps(&core.gateway)?;
            println!("{}", serde_json::to_string_pretty(&apps)?);
        }
        "projects" => {
            let aido_path = PathBuf::from(r"C:\Users\keoku\Desktop\AIDO");
            let projects = core.projects.index_roots(&core.gateway, &[aido_path], 3)?;
            println!("{}", serde_json::to_string_pretty(&projects)?);
        }
        "installer" => {
            let plan = core.evaluate_smart_installer_plan(4, 1)?;
            println!("{}", serde_json::to_string_pretty(&plan)?);
        }
        "runtime" => {
            let report = core.inspect_runtime_and_compact_memory(2)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        "proactive" => {
            core.bridge.seed_demo_deep_cleanup_files()?;
            let report = core.evaluate_proactive_and_deep_cleanup(
                aido_core::proactive_engine::UserActivityState::NormalIdle,
            )?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        "release-check" => {
            let report = core.evaluate_adapter_sandbox_and_release_readiness(
                "obs-studio",
                "OBS Studio",
                "OBS Project",
                r"C:\Program Files\obs-studio\bin\64bit\obs64.exe",
            )?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        "remote" => {
            let challenge = core.start_remote_pairing_challenge()?;
            let device = core.complete_remote_pairing(
                &challenge.challenge_id,
                &challenge.pin_code,
                "iPhone 16 Pro · AIDO Remote",
            )?;
            let cockpit = core.build_remote_cockpit_state(&device.session_token)?;
            println!("{}", serde_json::to_string_pretty(&cockpit)?);
        }
        "masterplan" => {
            let report = core.evaluate_masterplan_completion()?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        "rea" => {
            let target_path = args.get(2).map(PathBuf::from).unwrap_or_else(|| {
                let notepad = PathBuf::from(r"C:\Windows\System32\notepad.exe");
                if notepad.exists() {
                    notepad
                } else {
                    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("aido-cli.exe"))
                }
            });
            let report = core.rea.inspect_binary(&target_path)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        "demo" => {
            println!("=== AIDO (Agentic Intelligent Developing Operator) — Kern-Prototyp & Systemcheck ===");
            println!(
                "Datenschutzmodus: {:?} | Zugriffsmodus: {:?}",
                core.gateway.privacy_mode(),
                core.gateway.access_mode()
            );

            // Seed 3 harmless temporary cache files in AIDO's isolated staging folder
            let seeded = core.bridge.seed_demo_cache_files()?;
            println!(
                "[1/12] Test-Staging vorbereitet: {} KB in '{}'",
                seeded / 1024,
                core.bridge.safe_temp_dir().display()
            );

            // Step 1: Read-only health check + AI proposal + Permission Gateway request
            let api_key = std::env::var("AIDO_LOCAL_API_KEY").ok();
            let inspection = core.inspect_and_propose_fast_cleanup(api_key.as_deref())?;
            println!("\n[2/12] AIDO Erklärungs- & Freigabevorschlag:");
            println!("  Modell: {}", inspection.ai_proposal.model_id);
            println!("  Nachricht: {}", inspection.ai_proposal.message);
            println!(
                "  Klickbare Optionen: {:?}",
                inspection.ai_proposal.clickable_options
            );
            println!(
                "  Freigabe-ID: {}",
                inspection.approval_request.request_id
            );

            // Step 2: Simulate explicit user approval click ("Ja, jetzt bereinigen")
            let (_token, cleanup_result) = core.approve_and_execute_fast_cleanup(
                inspection.approval_request.request_id,
                Some("Nutzer hat im Dialog auf 'Ja, jetzt bereinigen' geklickt.".into()),
            )?;
            println!("\n[3/12] Ausführung nach Freigabe:");
            println!("  {}", cleanup_result.human_report);

            // Step 3: Verify SQLite/FTS5 Memory & Audit Log recall
            let memories = core.memory.search_memory("Fast AND Cleanup", 5)?;
            let audits = core.memory.list_audit_log(5)?;
            println!(
                "\n[4/12] Memory-Abruf (SQLite + FTS5): {} passende Erinnerungen, {} Audit-Einträge.",
                memories.len(),
                audits.len()
            );
            if let Some(latest) = memories.first() {
                println!("  Letzte Erinnerung: [{}] {}", latest.created_at, latest.summary);
            }

            // Step 5: App Intelligence & Projektkatalog
            println!("\n[5/12] App Intelligence & Projektkatalog:");
            let aido_dir = PathBuf::from(r"C:\Users\keoku\Desktop\AIDO");
            let (discovered_apps, discovered_projects) =
                core.discover_and_register_environment(&[aido_dir])?;

            let total_caps: usize = discovered_apps.iter().map(|a| a.capabilities.len()).sum();
            println!(
                "  Apps erkannt: {} Programme mit insgesamt {} geprüften Fähigkeiten:",
                discovered_apps.len(),
                total_caps
            );
            for app in &discovered_apps {
                let enabled_tag = if app.user_enabled { "Aktiviert" } else { "Deaktiviert" };
                println!(
                    "    • {} [{}] (Bevorzugte Steuerung: {:?}, {} Fähigkeiten)",
                    app.display_name,
                    enabled_tag,
                    app.preferred_control,
                    app.capabilities.len()
                );
            }

            println!(
                "  Projekte indiziert: {} Projekt(e) gefunden:",
                discovered_projects.len()
            );
            for p in &discovered_projects {
                println!(
                    "    • {} | Stack: {} | Branch: '{}' [{}] | Empfohlene Aktion: {}",
                    p.name,
                    p.stack.join(", "),
                    p.git_branch,
                    p.git_status,
                    p.recommended_action
                );
            }

            // Demonstrate prepare_project_launch
            let (launch_proj, launch_req) =
                core.projects.prepare_project_launch(&core.gateway, "AIDO")?;
            println!("  Projekt-Start Freigabe-Vorschlag vorbereitet:");
            println!("    Titel: {}", launch_req.human_title);
            println!("    Erklärung: {}", launch_req.human_explanation);
            println!("    Ziel-Details: {:?}", launch_req.target_summary);
            println!("    Freigabe-ID: {}", launch_req.request_id);
            println!(
                "    Status: Bereit zur Ausführung von '{}' im Verzeichnis '{}'",
                launch_proj.recommended_action, launch_proj.canonical_path
            );

            // Step 6: Smart Installer & Modell-Wiederverwendung
            println!("\n[6/12] Smart Installer & Modell-Wiederverwendung:");
            let plan = core.evaluate_smart_installer_plan(4, 1)?;
            let total_models = plan.models.len();
            let reused_models = plan.eta.reused_models_count;
            let download_mb = plan.eta.remaining_download_bytes / (1024 * 1024);

            println!("  Erkannte Hardware-Klasse: {:?}", plan.hardware_tier);
            println!(
                "  Wiederverwendete lokale Modelle: {}/{} vorhanden ({} MB Download nötig!)",
                reused_models, total_models, download_mb
            );
            for m in &plan.models {
                let status_icon = if m.exists_locally { "✓ Gefunden" } else { "✗ Fehlt" };
                println!(
                    "    • [{}] {} ({} MB) -> {}",
                    status_icon,
                    m.model_name,
                    m.size_bytes / (1024 * 1024),
                    m.expected_path
                );
            }
            println!("  3-stufige ETA-Prognose:");
            println!("    • AIDO einsatzbereit in: {} Sek.", plan.eta.ready_to_wake_seconds);
            println!(
                "    • Persönliche Einrichtung abgeschlossen in: ~{:.1} Min.",
                plan.eta.personal_setup_seconds as f64 / 60.0
            );
            println!("    • Erweiterte Integration: {}", plan.eta.extended_integration_note);

            if let Some(next) = &plan.next_resumable {
                println!(
                    "  Nächster wiederaufnehmbarer Schritt: [Phase {}/9: {}] (Status: {:?})",
                    next.step_index, next.title_de, next.status
                );
            }

            // Step 7: Live-Runtime-Supervisor, Vision/ASR & 4-Stufen-Memory-Kompaktierung
            println!("\n[7/12] Live-Runtime-Supervisor, Vision/ASR & 4-Stufen-Memory-Kompaktierung:");
            // Store a second episodic memory so compaction threshold (min_entries = 2) is met
            core.memory.store_memory(
                "EpisodicMemory",
                "FastCleanup",
                "Zweites Fast Cleanup simuliert (2 Dateien, 128 KB)",
                "Automatische periodische Cache-Bereinigung vor Idle-Phase",
                "aido-cli::demo",
            )?;

            let rt_report = core.inspect_runtime_and_compact_memory(2)?;
            println!("  Laufzeit-Status:");
            println!(
                "    • Llama-Router (127.0.0.1:18434): {:?} (Zero-VRAM Standby mit --sleep-idle-seconds {})",
                rt_report.runtime_status.llama_router.state,
                rt_report.runtime_status.llama_router.sleep_idle_seconds.unwrap_or(300)
            );
            println!(
                "    • Whisper ASR (127.0.0.1:18080): {:?} (CUDA 13.4, deutsches Sprachmodell)",
                rt_report.runtime_status.whisper_asr.state
            );
            println!(
                "    • CUDA 13.4 DLL Verzeichnis: {} [{}]",
                rt_report.runtime_status.llama_router.cuda_dll_dir,
                if rt_report.runtime_status.llama_router.cuda_dll_ready { "Bereit" } else { "Fehlt" }
            );

            // Multimodal Vision & Whisper ASR probe
            let vision_payload = core.ai.build_vision_chat_payload(
                "Analysiere Bildschirm für Kontext-Assistenz",
                "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==",
            );
            println!(
                "  Multimodale Vision: mmproj Payload generiert ({} Nachrichten-Elemente)",
                vision_payload.get("messages").and_then(|m| m.as_array()).map(|a| a.len()).unwrap_or(0)
            );

            let asr = core.ai.transcribe_voice_note("AIDO, prüfe Projekt-Status", 1.8)?;
            println!(
                "  Spracherkennung (ASR): 'AIDO, prüfe Projekt-Status' -> '{}' ({}, lokal: {})",
                asr.transcript,
                asr.engine,
                asr.local_only
            );

            println!("  4-Stufen-Memory-Kompaktierung (Stufe 2 Episodic -> Stufe 3 Semantic):");
            if let Some(comp) = &rt_report.compaction_report {
                println!(
                    "    • Kompaktierung erfolgreich: {} episodische Einträge -> semantischer Eintrag #{} ({})",
                    comp.compacted_source_entries, comp.created_semantic_entry_id, comp.category
                );
                println!("    • Zusammenfassung: {}", comp.digest_summary);
                println!("    • Eingesparte Zeichen: {} Zeichen", comp.saved_chars);
            } else {
                println!("    • Keine Kompaktierung nötig (Schwellenwert nicht erreicht).");
            }
            println!(
                "    • Aktive Erinnerungen im System: {}, Semantische Wissenseinträge: {}",
                rt_report.active_memories_count, rt_report.semantic_memories_count
            );

            // Step 8: Proaktive Vordenker-Engine & Deep Cleanup
            println!("\n[8/12] Proaktive Vordenker-Engine & Deep Cleanup:");
            let seeded_deep = core.bridge.seed_demo_deep_cleanup_files()?;
            println!(
                "  Test-Staging für Tiefenbereinigung vorbereitet: {} KB in '{}'",
                seeded_deep / 1024,
                core.bridge.deep_cleanup_staging_dir().display()
            );

            // 1. Proaktive Vordenker-Auswertung im Normalzustand
            let proactive_res = core.evaluate_proactive_and_deep_cleanup(
                aido_core::proactive_engine::UserActivityState::NormalIdle,
            )?;
            println!(
                "  Vordenker-Auswertung: {} Vorschläge generiert (Fokus-Schutz unterdrückt: {}):",
                proactive_res.proactive_report.suggestions.len(),
                proactive_res.proactive_report.suppressed_by_focus_guard
            );
            for s in &proactive_res.proactive_report.suggestions {
                println!(
                    "    • [Priorität {}] {} ({:?}):",
                    s.priority, s.title, s.category
                );
                println!("      Nachricht: {}", s.human_message);
                println!("      Warum jetzt: {}", s.why_now);
                println!("      Optionen: {:?}", s.clickable_options);
            }

            // 2. Fokus-Schutz Simulation (Gaming / Vollbild)
            let focus_res = core.evaluate_proactive_and_deep_cleanup(
                aido_core::proactive_engine::UserActivityState::FullscreenOrGaming,
            )?;
            println!(
                "  Fokus-Schutz Simulation (Gaming/Fullscreen): Unterdrückt = {}, Grund: '{}'",
                focus_res.proactive_report.suppressed_by_focus_guard,
                focus_res
                    .proactive_report
                    .suppression_reason
                    .unwrap_or_default()
            );

            // 3. Deep Cleanup Freigabe & Ausführung
            println!(
                "  Deep Cleanup Scan: {} Kandidaten ({} KB freigebbar):",
                proactive_res.deep_cleanup_scan.candidates.len(),
                proactive_res.deep_cleanup_scan.total_reclaimable_bytes / 1024
            );
            for c in &proactive_res.deep_cleanup_scan.candidates {
                println!(
                    "    • [{:?}] {} ({} KB) — {}",
                    c.category,
                    c.file_name,
                    c.size_bytes / 1024,
                    c.reason_german
                );
            }

            let (_dc_token, dc_result) = core.approve_and_execute_deep_cleanup(
                proactive_res.deep_cleanup_approval.request_id,
                Some("Nutzer hat Deep Cleanup im Demo-Lauf bestätigt.".into()),
            )?;
            println!("  Ausführung nach Deep-Cleanup-Freigabe:");
            println!("    {}", dc_result.human_report);

            // Step 9: Isolierte Adapter-Prüfbank (Phase 7) & Release/Uninstaller-Check (Phase 8)
            println!("\n[9/12] Isolierte Adapter-Prüfbank (Phase 7) & Release/Uninstaller-Check (Phase 8):");
            let p78_report = core.evaluate_adapter_sandbox_and_release_readiness(
                "obs-studio",
                "OBS Studio",
                "OBS Project",
                r"C:\Program Files\obs-studio\bin\64bit\obs64.exe",
            )?;

            println!("  Phase 7: Synthetisierter Adapter für unbekannte App:");
            println!(
                "    • App: {} ({}, Version: {})",
                p78_report.synthesized_app.display_name,
                p78_report.synthesized_app.publisher,
                p78_report.synthesized_app.version
            );
            println!(
                "    • Steuerungsmethode: {:?} (Sandbox isoliert: {}, Harmloser Probe-Check: {})",
                p78_report.sandbox_report.synthesized_control,
                p78_report.sandbox_report.sandbox_isolated,
                p78_report.sandbox_report.harmless_probe_passed
            );
            println!(
                "    • Geprüfte Fähigkeiten ({} verifiziert / harmlos, {} freigabepflichtig):",
                p78_report.sandbox_report.verified_capabilities,
                p78_report.sandbox_report.restricted_capabilities
            );
            for cap in &p78_report.synthesized_app.capabilities {
                let status_tag = format!("{:?}", cap.status);
                let approval_tag = if cap.requires_approval { "Erfordert Freigabe" } else { "Ohne Freigabe" };
                println!(
                    "        - [{}] {} ({:?}, {})",
                    status_tag, cap.title_de, cap.risk, approval_tag
                );
            }

            println!("  Phase 8: Saubere Deinstallation & Zero-Destruction Garantie:");
            println!(
                "    • AIDO-Pfade zur Bereinigung: {} Verzeichnisse/Dateien",
                p78_report.uninstall_plan.aido_owned_paths_to_remove.len()
            );
            println!(
                "    • Geschützte externe KI-Modelle ({}/{} unangetastet):",
                p78_report.uninstall_plan.preserved_external_models.len(),
                p78_report.uninstall_plan.preserved_external_models.len()
            );
            for model_path in &p78_report.uninstall_plan.preserved_external_models {
                println!("        ✓ Unberührt: {}", model_path);
            }
            println!(
                "    • Schutz lokaler Projekte: {}",
                p78_report.uninstall_plan.preserved_user_projects_note
            );

            println!("  Phase 8: Release-Readiness Status (Version {}):", p78_report.release_readiness.version);
            println!(
                "    • Alle Phasen abgeschlossen: {} | Zero-Destruction: {} | Private Offline: {}",
                p78_report.release_readiness.all_phases_completed,
                p78_report.release_readiness.zero_destruction_verified,
                p78_report.release_readiness.privacy_offline_verified
            );
            for (idx, item) in p78_report.release_readiness.checklist_items.iter().enumerate() {
                println!("      [{}/8] ✓ {}", idx + 1, item);
            }
            println!("    • Statusbericht: {}", p78_report.release_readiness.human_summary);

            // Step 10: Agent Remote (QR-Pairing, 4-Zonen-Smartphone-Cockpit, AIDO-Blick & 100 % PC-Parität)
            println!("\n[10/12] Agent Remote (QR-Pairing, 4-Zonen-Smartphone-Cockpit, AIDO-Blick & 100 % PC-Parität):");
            let challenge = core.start_remote_pairing_challenge()?;
            println!("  1. Pairing-Challenge generiert:");
            println!("    • Challenge-ID: {}", challenge.challenge_id);
            println!("    • 4-stelliger PIN: {}", challenge.pin_code);
            println!("    • QR-URL: {}", challenge.qr_url);
            println!("    • QR-ASCII-Vorschau:\n{}", challenge.qr_ascii_matrix);

            let paired_device = core.complete_remote_pairing(
                &challenge.challenge_id,
                &challenge.pin_code,
                "iPhone 16 Pro · AIDO Remote",
            )?;
            println!("  2. Smartphone erfolgreich gekoppelt:");
            println!("    • Gerätename: {}", paired_device.device_name);
            println!("    • Volle PC-Parität: {}", paired_device.full_pc_parity);
            println!("    • Neuer Datenschutzmodus: {:?}", core.gateway.privacy_mode());

            let cockpit = core.build_remote_cockpit_state(&paired_device.session_token)?;
            println!("  3. 4-Zonen Smartphone-Cockpit Status:");
            println!("    • Zone 1 (System-Puls): VRAM {}/{} MiB, RAM frei {} MB", cockpit.vram_used_mib, cockpit.vram_total_mib, cockpit.ram_free_mb);
            println!("    • Zone 2 (AIDO-Blick): '{}' ({})", cockpit.latest_blick.active_window_title, cockpit.latest_blick.vision_summary_de);
            println!("    • Zone 3 (Quick-Tiles): {} Kacheln verfügbar:", cockpit.quick_tiles.len());
            for t in &cockpit.quick_tiles {
                let req_tag = if t.requires_approval { "Freigabe erforderlich" } else { "Sofort" };
                println!("        - [{}] {} — {} ({})", t.tile_id, t.label_de, t.subtitle_de, req_tag);
            }
            println!("    • Zone 4 (Daumen-Aktionen): Pending Approvals = {}, Reply-Chips: {:?}", cockpit.pending_approvals_count, cockpit.reply_chips_de);

            // Step 11: Masterplan-Abschluss (7-Ebenen-Memory, Persistente Task Engine mit Desktop-Lock-Weiche, 8 Spezialisten, Admin Broker & WSL2/Docker-Sandboxes)
            println!("\n[11/12] Masterplan-Abschluss (7-Ebenen-Memory, Persistente Task Engine mit Desktop-Lock-Weiche, 8 Spezialisten, Admin Broker & WSL2/Docker-Sandboxes):");
            let mp_report = core.evaluate_masterplan_completion()?;
            println!("  1. 7-Ebenen-Gedächtnis & Procedural Skills:");
            println!(
                "    • Speicher-Quota: {} / {} Bytes belegt ({:.2} %)",
                mp_report.memory_quota.db_size_bytes,
                mp_report.memory_quota.quota_limit_bytes,
                mp_report.memory_quota.usage_percent
            );
            println!(
                "    • Gelernte prozedurale Fähigkeiten ({} aktiv):",
                mp_report.procedural_skills.len()
            );
            for skill in &mp_report.procedural_skills {
                println!(
                    "        - [{}] {} (v{}, Trigger: '{}')",
                    skill.skill_id, skill.title_de, skill.version, skill.trigger_pattern
                );
            }

            println!("  2. Persistente Task Engine & Desktop-Lock-Weiche:");
            println!(
                "    • Desktop gesperrt: {} | Weichen-Aktion: {}",
                mp_report.desktop_lock_report.desktop_locked,
                mp_report.desktop_lock_report.human_summary_de
            );
            println!(
                "    • Unbeeinflusst weiterlaufende Hintergrund-Tasks (CLI/API): {}",
                mp_report.desktop_lock_report.continued_background_tasks
            );
            println!(
                "    • Automatisch pausierte UI-Tasks: {}",
                mp_report.desktop_lock_report.paused_ui_tasks
            );

            println!("  3. 8 Interne Spezialisten & Admin Broker (DPAPI Vault):");
            println!(
                "    • Gerouteter Primär-Spezialist: {:?} ({})",
                mp_report.specialist_dispatch.selected_specialist,
                mp_report.specialist_dispatch.specialist_title_de
            );
            println!(
                "    • Begründung: {}",
                mp_report.specialist_dispatch.coordination_summary_de
            );
            println!(
                "    • Benötigte Gedächtnis-Ebenen: {:?}",
                mp_report.specialist_dispatch.required_memory_layers
            );
            println!(
                "    • DPAPI-Tresor: {} versiegelte Geheimnisse ohne Klartext-Freilegung:",
                mp_report.protected_secrets.len()
            );
            for sec in &mp_report.protected_secrets {
                println!(
                    "        - Key: {} | Backend: {} | Klartext exponiert: {}",
                    sec.secret_key, sec.storage_backend, sec.plaintext_exposed
                );
            }

            println!("  4. Power-User WSL2 & RTX 5090 Docker Sandboxes (Masterplan Section 12):");
            println!(
                "    • WSL2 Basis-Pfad: {} (Optional für normale Nutzer: {})",
                mp_report.docker_audit.wsl2_native_base_path,
                mp_report.docker_audit.optional_for_normal_users
            );
            println!(
                "    • RTX 5090 GPU-Passthrough bereit: {} | Ubuntu WSL aktiv: {}",
                mp_report.docker_audit.rtx_5090_gpu_passthrough_ready,
                mp_report.docker_audit.ubuntu_wsl_integration_enabled
            );
            println!("    • Verfügbare Sandbox-Vorlagen ({} Stacks):", mp_report.docker_audit.templates.len());
            for t in &mp_report.docker_audit.templates {
                let gpu_tag = if t.requires_rtx_gpu_passthrough { "RTX 5090 GPU" } else { "CPU/Standard" };
                println!(
                    "        - [{}] {} ({}, Ports: {:?})",
                    t.slug, t.title_de, gpu_tag, t.exposed_ports
                );
            }
            println!("    • Hinweis: {}", mp_report.docker_audit.remediation_hint_de);

            // Step 12: REA Universelle App-Reverse-Engineering & Adapter-Synthese (morluto/rea)
            println!("\n[12/12] REA Universelle App-Reverse-Engineering & Adapter-Synthese (morluto/rea):");
            let target_exe = {
                let notepad = PathBuf::from(r"C:\Windows\System32\notepad.exe");
                if notepad.exists() {
                    notepad
                } else {
                    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("aido-cli.exe"))
                }
            };
            let (rea_report, rea_entry) = core.inspect_and_register_app_with_rea(
                &target_exe,
                "windows-notepad",
                "Windows Editor (Notepad)",
                "Microsoft Corporation",
            )?;
            println!("  1. Binär-Inspektion & Provenance (Zero-Panic PE/ASAR Parser):");
            println!("    • Ziel-Datei: {}", rea_report.target_path);
            println!("    • SHA-256 Provenance: {}", rea_report.evidence_provenance_sha256);
            println!("    • Binärtyp: {:?}", rea_report.binary_kind);
            if let Some(pe) = &rea_report.pe_meta {
                println!(
                    "    • PE-Header: Architektur '{}', Subsystem '{}', Manifest: {}",
                    pe.architecture, pe.subsystem, pe.has_embedded_manifest
                );
                println!(
                    "    • Importierte DLLs ({}): {}",
                    pe.imported_dlls.len(),
                    pe.imported_dlls.join(", ")
                );
                println!(
                    "    • Sektionen ({}): {}",
                    pe.sections.len(),
                    pe.sections.join(", ")
                );
            }
            if let Some(electron) = &rea_report.electron_meta {
                println!(
                    "    • Electron-Metadaten: App '{}', Version '{}', Script '{}', IPC: {:?}",
                    electron.app_name, electron.version, electron.main_script, electron.ipc_channels
                );
            }
            println!("  2. Evidenz-basierte Fähigkeiten & Risikotrennung:");
            for cap in &rea_entry.capabilities {
                let req_tag = if cap.requires_approval { "Erfordert Freigabe" } else { "Harmlos / Direkt" };
                println!(
                    "    • [{:?}] {} (Risiko: {:?}, Status: {:?})",
                    cap.risk, cap.title_de, cap.risk, cap.status
                );
                println!("        ID: {} ({})", cap.capability_id, req_tag);
            }
            println!("  3. Adapter-Synthese & Persistenz:");
            println!("    • App-ID in SQLite: {}", rea_entry.app_id);
            println!("    • Bevorzugte Steuerung: {:?}", rea_entry.preferred_control);
            println!("    • Herkunft: {:?}", rea_entry.adapter_origin);
            println!("    • Zusammenfassung: {}", rea_report.human_summary_de);
        }
        _ => {
            println!("Verwendung: aido-cli [demo | health | apps | projects | installer | runtime | proactive | release-check | remote | masterplan | rea]");
        }
    }

    Ok(())
}
