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
        "demo" => {
            println!("=== AIDO Kern-Prototyp: Systemcheck & Fast Cleanup Durchstich ===");
            println!(
                "Datenschutzmodus: {:?} | Zugriffsmodus: {:?}",
                core.gateway.privacy_mode(),
                core.gateway.access_mode()
            );

            // Seed 3 harmless temporary cache files in AIDO's isolated staging folder
            let seeded = core.bridge.seed_demo_cache_files()?;
            println!(
                "[1/4] Test-Staging vorbereitet: {} KB in '{}'",
                seeded / 1024,
                core.bridge.safe_temp_dir().display()
            );

            // Step 1: Read-only health check + AI proposal + Permission Gateway request
            let api_key = std::env::var("AIDO_LOCAL_API_KEY").ok();
            let inspection = core.inspect_and_propose_fast_cleanup(api_key.as_deref())?;
            println!("\n[2/4] AIDO Erklärungs- & Freigabevorschlag:");
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
            println!("\n[3/4] Ausführung nach Freigabe:");
            println!("  {}", cleanup_result.human_report);

            // Step 3: Verify SQLite/FTS5 Memory & Audit Log recall
            let memories = core.memory.search_memory("Fast AND Cleanup", 5)?;
            let audits = core.memory.list_audit_log(5)?;
            println!(
                "\n[4/4] Memory-Abruf (SQLite + FTS5): {} passende Erinnerungen, {} Audit-Einträge.",
                memories.len(),
                audits.len()
            );
            if let Some(latest) = memories.first() {
                println!("  Letzte Erinnerung: [{}] {}", latest.created_at, latest.summary);
            }
        }
        _ => {
            println!("Verwendung: aido-cli [demo | health]");
        }
    }

    Ok(())
}
