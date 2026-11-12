#![cfg(all(feature = "desktop", windows))]

use aido_core::{
    desktop_setup::{ModelProfile, SetupSettings, SetupStore},
    memory::MemoryEngine,
    permission_gateway::{AccessMode, ActionKind, PermissionGateway, PrivacyMode},
};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[test]
fn test_desktop_actions_and_rea_flow() {
    let temp_root = std::env::temp_dir().join(format!("aido-actions-test-{}", Uuid::new_v4()));
    let db_path = temp_root.join("data/aido_memory.db");
    std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();

    let store = SetupStore::new(&db_path).unwrap();
    let desktop_aido = PathBuf::from(std::env::var_os("USERPROFILE").unwrap()).join("Desktop/AIDO");
    let canonical_desktop_aido = desktop_aido.canonicalize().unwrap();
    let text = canonical_desktop_aido.to_string_lossy();
    let normalized_desktop_aido = if let Some(local) = text.strip_prefix(r"\\?\") {
        PathBuf::from(local)
    } else {
        canonical_desktop_aido
    };

    let settings = SetupSettings {
        step: 3,
        completed: true,
        access_mode: AccessMode::FullAccess,
        privacy_mode: PrivacyMode::PrivateOffline,
        project_roots: vec![PathBuf::from("C:\\"), PathBuf::from("D:\\")],
        recent_metadata: true,
        model_profile: ModelProfile::default(),
        rea_enabled: true,
        antigravity_enabled: true,
        blender_enabled: true,
    };
    store.activate(&settings).unwrap();

    // Verify draft save and active settings persistence
    let active = store.active().unwrap().expect("active settings exist");
    assert!(active.completed);
    assert_eq!(active.access_mode, AccessMode::FullAccess);
    assert!(active.antigravity_enabled);
    assert!(active.blender_enabled);
    assert!(active.rea_enabled);

    // Verify REA node executable and entrypoint in installed or tooling location
    let rea_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tooling/rea");
    let node_exe = rea_dir.join("node.exe");
    let rea_entry = rea_dir.join("node_modules/rea-agents/scripts/rea.mjs");
    assert!(node_exe.is_file(), "tooling/rea/node.exe must exist");
    assert!(rea_entry.is_file(), "tooling/rea rea.mjs must exist");

    // Run REA --version
    let version_output = std::process::Command::new(&node_exe)
        .arg(&rea_entry)
        .arg("--version")
        .output()
        .expect("rea --version must execute");
    assert!(version_output.status.success());
    let version_str = String::from_utf8_lossy(&version_output.stdout);
    assert!(version_str.contains("6.7.0"), "REA version must be 6.7.0, got {version_str}");

    // Run REA inspect-pe-resources on C:\Windows\System32\notepad.exe
    let notepad_path = Path::new("C:\\Windows\\System32\\notepad.exe");
    assert!(notepad_path.is_file());
    let inspect_output = std::process::Command::new(&node_exe)
        .arg(&rea_entry)
        .arg("inspect-pe-resources")
        .arg(notepad_path)
        .args([
            "--max-file-bytes",
            "67108864",
            "--max-entries",
            "1024",
            "--format",
            "json",
            "--filter-output",
            "evidence_id,subject,provider,operation,normalized_result.coverage,limitations",
        ])
        .output()
        .expect("rea inspect-pe-resources must execute");
    assert!(inspect_output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&inspect_output.stdout).expect("valid JSON report");
    assert!(report["evidence_id"].as_str().is_some());
    assert_eq!(report["operation"], "inspect_pe_resources");
    assert_eq!(report["subject"]["name"], "notepad.exe");
    assert_eq!(report["normalized_result"]["coverage"]["status"], "complete");
    assert!(report["normalized_result"]["coverage"]["resources"].as_u64().unwrap() > 0);

    // Verify target directory normalization
    assert_eq!(normalized_desktop_aido, PathBuf::from(r"C:\Users\keoku\Desktop\AIDO"));

    // Verify PermissionGateway approval lifecycle & audit logging for Antigravity & Blender
    let memory = MemoryEngine::new(&db_path).unwrap();
    let gateway = PermissionGateway::new(AccessMode::FullAccess, PrivacyMode::PrivateOffline);

    // 1. Decline workflow
    let req1 = gateway
        .request_approval(
            ActionKind::LaunchProject,
            "Antigravity öffnen",
            "AIDO öffnet das angezeigte lokale Ziel.",
            vec![normalized_desktop_aido.display().to_string()],
            0,
            0,
        )
        .unwrap();
    gateway.reject_request(req1.request_id, None).unwrap();
    memory
        .record_audit(
            "ProjectLaunchDeclined",
            "FullAccess",
            "PrivateOffline",
            false,
            &normalized_desktop_aido.display().to_string(),
        )
        .unwrap();

    // 2. Approve and consume workflow
    let req2 = gateway
        .request_approval(
            ActionKind::LaunchProject,
            "Antigravity öffnen",
            "AIDO öffnet das angezeigte lokale Ziel.",
            vec![normalized_desktop_aido.display().to_string()],
            0,
            0,
        )
        .unwrap();
    let token = gateway.approve_request(req2.request_id, None).unwrap();
    gateway
        .consume_approval_token(&token, &ActionKind::LaunchProject)
        .unwrap();
    memory
        .record_audit(
            "ProjectLaunchApproved",
            "FullAccess",
            "PrivateOffline",
            true,
            &normalized_desktop_aido.display().to_string(),
        )
        .unwrap();

    // Replay / reuse of already consumed token MUST fail
    let replay_err = gateway.consume_approval_token(&token, &ActionKind::LaunchProject);
    assert!(replay_err.is_err(), "Already consumed token must be rejected");

    // 3. Blender launch lifecycle
    let req3 = gateway
        .request_approval(
            ActionKind::ExecuteAppAction,
            "Blender öffnen",
            "AIDO öffnet das angezeigte lokale Ziel.",
            vec!["Blender".to_string()],
            0,
            0,
        )
        .unwrap();
    let token_blender = gateway.approve_request(req3.request_id, None).unwrap();
    gateway
        .consume_approval_token(&token_blender, &ActionKind::ExecuteAppAction)
        .unwrap();
    memory
        .record_audit(
            "BlenderLaunchApproved",
            "FullAccess",
            "PrivateOffline",
            true,
            "C:\\Program Files\\Blender Foundation\\Blender 5.1\\blender.exe",
        )
        .unwrap();

    // Verify Audit log persistence and integrity in SQLite
    let audit_entries = memory.list_audit_log(10).unwrap();
    assert!(audit_entries.len() >= 3);
    assert_eq!(audit_entries[0].action, "BlenderLaunchApproved");
    assert!(audit_entries[0].approved);
    assert_eq!(audit_entries[1].action, "ProjectLaunchApproved");
    assert!(audit_entries[1].approved);
    assert_eq!(audit_entries[2].action, "ProjectLaunchDeclined");
    assert!(!audit_entries[2].approved);
    assert!(audit_entries[2].summary.contains("AIDO"));
}
