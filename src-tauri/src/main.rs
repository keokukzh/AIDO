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
        "demo" => {
            println!("=== AIDO (Artificial Intelligence Development Operator) — Kern-Prototyp & Systemcheck ===");
            println!(
                "Datenschutzmodus: {:?} | Zugriffsmodus: {:?}",
                core.gateway.privacy_mode(),
                core.gateway.access_mode()
            );

            // Seed 3 harmless temporary cache files in AIDO's isolated staging folder
            let seeded = core.bridge.seed_demo_cache_files()?;
            println!(
                "[1/5] Test-Staging vorbereitet: {} KB in '{}'",
                seeded / 1024,
                core.bridge.safe_temp_dir().display()
            );

            // Step 1: Read-only health check + AI proposal + Permission Gateway request
            let api_key = std::env::var("AIDO_LOCAL_API_KEY").ok();
            let inspection = core.inspect_and_propose_fast_cleanup(api_key.as_deref())?;
            println!("\n[2/5] AIDO Erklärungs- & Freigabevorschlag:");
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
            println!("\n[3/5] Ausführung nach Freigabe:");
            println!("  {}", cleanup_result.human_report);

            // Step 3: Verify SQLite/FTS5 Memory & Audit Log recall
            let memories = core.memory.search_memory("Fast AND Cleanup", 5)?;
            let audits = core.memory.list_audit_log(5)?;
            println!(
                "\n[4/5] Memory-Abruf (SQLite + FTS5): {} passende Erinnerungen, {} Audit-Einträge.",
                memories.len(),
                audits.len()
            );
            if let Some(latest) = memories.first() {
                println!("  Letzte Erinnerung: [{}] {}", latest.created_at, latest.summary);
            }

            // Step 5: App Intelligence & Projektkatalog
            println!("\n[5/5] App Intelligence & Projektkatalog:");
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
        }
        _ => {
            println!("Verwendung: aido-cli [demo | health | apps | projects]");
        }
    }

    Ok(())
}
