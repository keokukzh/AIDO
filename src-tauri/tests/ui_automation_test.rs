use aido_core::{
    memory::MemoryEngine,
    permission_gateway::{AccessMode, ActionKind, GatewayError, PermissionGateway, PrivacyMode},
    ui_automation::{UiAction, UiAutomationEngine, UiAutomationError},
};
use std::process::Command;
use uuid::Uuid;

#[test]
fn test_ui_automation_driver_resolution_and_security_gateway() {
    let temp_root = std::env::temp_dir().join(format!("aido-ui-test-{}", Uuid::new_v4()));
    let db_path = temp_root.join("data/aido_memory.db");
    std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();

    let memory = MemoryEngine::new(&db_path).unwrap();
    let engine = UiAutomationEngine::new();

    // 1. Resolve Driver
    let driver = engine.resolve_driver().expect("driver must be resolvable");
    assert!(driver.exists(), "driver binary must physically exist on disk");

    // 2. ReadOnly Access Mode: Observe is allowed, Act is strictly blocked
    let readonly_gateway = PermissionGateway::new(AccessMode::ReadOnly, PrivacyMode::PrivateOffline);
    let windows = engine.list_windows(&readonly_gateway, None);
    assert!(windows.is_ok(), "list_windows should succeed in ReadOnly mode");

    let blocked_action = engine.execute_action(
        &readonly_gateway,
        "notepad",
        &UiAction::TypeText {
            element_ref: "@test:e1".to_string(),
            text: "AIDO Test".to_string(),
        },
        None,
        Some(&memory),
    );

    match blocked_action {
        Err(UiAutomationError::Gateway(GatewayError::ReadOnlyBlocked(msg))) => {
            assert!(msg.contains("UiAct"));
        }
        other => panic!("Expected ReadOnlyBlocked, got: {:?}", other),
    }

    // 3. StandardAccess: Action without token must be rejected
    let standard_gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);
    let missing_token_err = engine.execute_action(
        &standard_gateway,
        "notepad",
        &UiAction::Click {
            element_ref: "@test:e2".to_string(),
        },
        None,
        Some(&memory),
    );
    assert!(matches!(missing_token_err, Err(UiAutomationError::ApprovalRequired)));

    // 4. Token Creation & Approval Flow
    let approval_req = engine
        .prepare_action_approval(
            &standard_gateway,
            "notepad",
            &UiAction::TypeText {
                element_ref: "@test:e3".to_string(),
                text: "Hallo aus AIDO UI Automation".to_string(),
            },
        )
        .expect("approval request should be generated");

    assert_eq!(approval_req.action, ActionKind::UiAct);
    assert!(approval_req.human_title.to_lowercase().contains("notepad"));

    let token = standard_gateway
        .approve_request(approval_req.request_id, Some("Freigabe durch Nutzer".to_string()))
        .expect("approval should yield valid token");

    // Replay attack check: Token consumption
    let token_clone = token.clone();
    let consume_res = standard_gateway.consume_approval_token(&token, &ActionKind::UiAct);
    assert!(consume_res.is_ok());

    // Second consumption must fail (single-use token protection)
    let replay_res = standard_gateway.consume_approval_token(&token_clone, &ActionKind::UiAct);
    assert!(replay_res.is_err(), "Approval token replay must be rejected");

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_root);
}

#[test]
fn test_ui_automation_notepad_lifecycle_and_audit_log() {
    let temp_root = std::env::temp_dir().join(format!("aido-ui-notepad-{}", Uuid::new_v4()));
    let db_path = temp_root.join("data/aido_memory.db");
    std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();

    let memory = MemoryEngine::new(&db_path).unwrap();
    let engine = UiAutomationEngine::new();
    let gateway = PermissionGateway::new(AccessMode::FullAccess, PrivacyMode::PrivateOffline);

    // Launch Notepad as target test window
    let mut child = Command::new("notepad.exe")
        .spawn()
        .expect("notepad should launch");

    // Give the window a moment to initialize
    std::thread::sleep(std::time::Duration::from_millis(1500));

    // Observe: Take Snapshot
    let snapshot_res = engine.take_snapshot(&gateway, "Notepad");
    if let Ok(ref snapshot) = snapshot_res {
        assert_eq!(snapshot.app, "Notepad");
        assert!(snapshot.human_summary_de.contains("UI-Snapshot für 'Notepad'"));
    }

    // Act: Focus window in FullAccess mode
    let _focus_res = engine.execute_action(
        &gateway,
        "Notepad",
        &UiAction::FocusWindow {
            app: "Notepad".to_string(),
        },
        None,
        Some(&memory),
    );

    // Terminate test notepad
    let _ = child.kill();
    let _ = child.wait();

    // Verify Audit Log recorded the action
    let audit_entries = memory.list_audit_log(10).unwrap();
    assert!(
        audit_entries.iter().any(|e| e.action == "UiAutomation"),
        "Audit log must contain UiAutomation execution entry"
    );

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_root);
}
