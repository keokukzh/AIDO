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
                "[1/7] Test-Staging vorbereitet: {} KB in '{}'",
                seeded / 1024,
                core.bridge.safe_temp_dir().display()
            );

            // Step 1: Read-only health check + AI proposal + Permission Gateway request
            let api_key = std::env::var("AIDO_LOCAL_API_KEY").ok();
            let inspection = core.inspect_and_propose_fast_cleanup(api_key.as_deref())?;
            println!("\n[2/7] AIDO Erklärungs- & Freigabevorschlag:");
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
            println!("\n[3/7] Ausführung nach Freigabe:");
            println!("  {}", cleanup_result.human_report);

            // Step 3: Verify SQLite/FTS5 Memory & Audit Log recall
            let memories = core.memory.search_memory("Fast AND Cleanup", 5)?;
            let audits = core.memory.list_audit_log(5)?;
            println!(
                "\n[4/7] Memory-Abruf (SQLite + FTS5): {} passende Erinnerungen, {} Audit-Einträge.",
                memories.len(),
                audits.len()
            );
            if let Some(latest) = memories.first() {
                println!("  Letzte Erinnerung: [{}] {}", latest.created_at, latest.summary);
            }

            // Step 5: App Intelligence & Projektkatalog
            println!("\n[5/7] App Intelligence & Projektkatalog:");
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
            println!("\n[6/7] Smart Installer & Modell-Wiederverwendung:");
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
            println!("\n[7/7] Live-Runtime-Supervisor, Vision/ASR & 4-Stufen-Memory-Kompaktierung:");
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
        }
        _ => {
            println!("Verwendung: aido-cli [demo | health | apps | projects | installer | runtime]");
        }
    }

    Ok(())
}
