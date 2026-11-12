use aido_core::{
    memory::{MemoryEngine, MemoryLayer},
    proactive_engine::{
        ProactiveCategory, ProactiveEngine, UserActivityState,
    },
    project_intelligence::ProjectEntry,
    windows_bridge::{GpuStatus, SystemHealthReport},
};
use uuid::Uuid;

#[test]
fn test_memory_engine_7_layers_and_forget_control() {
    let temp_root = std::env::temp_dir().join(format!("aido-mem-test-{}", Uuid::new_v4()));
    let db_path = temp_root.join("data/aido_memory.db");
    std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();

    let memory = MemoryEngine::new(&db_path).unwrap();

    // 1. Store entries in different layers
    let id_user = memory
        .store_memory(
            MemoryLayer::UserMemory.as_str(),
            "preferences",
            "Lieblingsprogramm",
            "Der Nutzer arbeitet bevorzugt mit Blender 3D für Visualisierungen.",
            "chat",
        )
        .expect("user memory should be stored");

    let _id_project = memory
        .store_memory(
            MemoryLayer::ProjectMemory.as_str(),
            "projects",
            "Projektpfad AIDO",
            "Das Hauptprojekt AIDO befindet sich unter C:\\Users\\keoku\\Desktop\\AIDO.",
            "project_scanner",
        )
        .expect("project memory should be stored");

    let _id_semantic = memory
        .store_memory(
            MemoryLayer::SemanticMemory.as_str(),
            "hardware",
            "GPU Spezifikation",
            "Die Arbeitsstation ist mit einer NVIDIA GeForce RTX 5090 (32 GB VRAM) ausgestattet.",
            "health_monitor",
        )
        .expect("semantic memory should be stored");

    // 2. FTS5 Search verification
    let results = memory.search_memory("Blender", 10).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].summary, "Lieblingsprogramm");
    assert_eq!(results[0].layer, MemoryLayer::UserMemory.as_str());

    let gpu_results = memory.search_memory("RTX 5090", 10).unwrap();
    assert_eq!(gpu_results.len(), 1);
    assert!(gpu_results[0].details.contains("RTX 5090"));

    // 3. Storage Quota report
    let quota = memory.inspect_storage_quota(10 * 1024 * 1024).unwrap();
    assert!(quota.db_size_bytes > 0);
    assert!(quota.usage_percent < 1.0);
    assert!(quota.quota_healthy);

    // 4. Forget Memory Entry
    let forgotten = memory.forget_memory_entry(id_user).unwrap();
    assert!(forgotten, "forget_memory_entry must return true for existing entry");

    // After forgetting, searching for "Blender" must yield 0 active results
    let after_forget = memory.search_memory("Blender", 10).unwrap();
    assert_eq!(after_forget.len(), 0, "Forgotten memory must not appear in search");

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_root);
}

#[test]
fn test_proactive_engine_evaluation_and_focus_guard() {
    let engine = ProactiveEngine::new();

    let project = ProjectEntry {
        name: "AIDO".to_string(),
        canonical_path: "C:\\Users\\keoku\\Desktop\\AIDO".to_string(),
        stack: vec!["Rust".to_string(), "Tauri".to_string()],
        git_branch: "main".to_string(),
        git_status: "dirty".to_string(),
        recommended_action: "git commit".to_string(),
        human_summary: "AIDO Desktop Companion".to_string(),
    };

    let health = SystemHealthReport {
        timestamp: "2026-11-12T00:00:00Z".to_string(),
        gpu: GpuStatus {
            gpu_name: "NVIDIA GeForce RTX 5090".to_string(),
            vram_used_mib: 1800,
            vram_free_mib: 30000,
            vram_total_mib: 32607,
        },
        system_ram_free_mb: 40000,
        system_ram_total_mb: 64902,
        top_processes: vec![],
        aido_temp_cache_path: "".to_string(),
        aido_temp_cache_files: 0,
        aido_temp_cache_bytes: 0,
        human_summary: "".to_string(),
    };

    let projects = vec![project];

    // 1. Normal Idle State: Proactive suggestions generated for dirty git project
    let report_idle = engine.evaluate_context(
        UserActivityState::NormalIdle,
        &health,
        &projects,
        0,
        0,
    );
    assert!(!report_idle.suppressed_by_focus_guard);
    assert!(!report_idle.suggestions.is_empty(), "Should generate suggestion for dirty git project");

    let first = &report_idle.suggestions[0];
    assert_eq!(first.category, ProactiveCategory::DirtyGitWorkspace);
    assert!(first.human_message.contains("AIDO"));

    // 2. Fullscreen or Gaming State: Focus guard must suppress all notifications
    let report_gaming = engine.evaluate_context(
        UserActivityState::FullscreenOrGaming,
        &health,
        &projects,
        0,
        0,
    );
    assert!(report_gaming.suppressed_by_focus_guard, "Focus guard must suppress during gaming/fullscreen");
    assert!(report_gaming.suggestions.is_empty(), "No suggestions emitted when suppressed");

    // 3. Category Muting
    engine.mute_category(ProactiveCategory::DirtyGitWorkspace);
    assert!(engine.is_category_muted(ProactiveCategory::DirtyGitWorkspace));

    let report_after_mute = engine.evaluate_context(
        UserActivityState::NormalIdle,
        &health,
        &projects,
        0,
        0,
    );
    assert!(report_after_mute.suggestions.iter().all(|s| s.category != ProactiveCategory::DirtyGitWorkspace));

    // Unmute
    engine.unmute_category(ProactiveCategory::DirtyGitWorkspace);
    assert!(!engine.is_category_muted(ProactiveCategory::DirtyGitWorkspace));
}
