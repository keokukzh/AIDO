use aido_core::blender_connector::BlenderConnector;
use aido_core::docker_controller::{DockerActionType, DockerController};
use aido_core::permission_gateway::{AccessMode, ActionKind, PermissionGateway, PrivacyMode};
use aido_core::windows_security::{SystemSecurityGuard, WindowsDpapiVault};

#[test]
fn test_blender_connector_headless_inspection() {
    let connector = BlenderConnector::new();
    let gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

    let info = connector.probe_info().expect("Blender must be probeable");
    assert!(info.is_available);
    println!("Detected Blender version: {}", info.version);

    let summary = connector
        .inspect_scene(None, &gateway)
        .expect("Headless blank scene inspection must succeed");
    assert_eq!(summary.object_count, 3);
    assert_eq!(summary.mesh_count, 1);
    assert_eq!(summary.camera_count, 1);
    assert_eq!(summary.light_count, 1);
    println!(
        "Blender scene inspection verified: Engine={}, Objects={:?}",
        summary.active_engine,
        summary.objects.iter().map(|o| &o.name).collect::<Vec<_>>()
    );
}

#[test]
fn test_docker_controller_environment_and_gateway_guard() {
    let ctrl = DockerController::new();
    let gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

    let status = ctrl.check_status(&gateway).expect("Docker status check must succeed");
    assert!(status.cli_available, "Docker CLI should be found on reference host");
    println!("Docker status: {}", status.human_message_de);

    let templates = ctrl.get_templates();
    assert_eq!(templates.len(), 3, "All 3 Masterplan templates must be present");
    assert_eq!(templates[0].slug, "01-fullstack-sandbox");
    assert_eq!(templates[1].slug, "02-ai-gpu-sandbox");
    assert_eq!(templates[2].slug, "03-automation-stack");

    // Test token requirement for container start
    let req = ctrl
        .prepare_action_approval(DockerActionType::Start, "02-ai-gpu-sandbox", &gateway)
        .unwrap();
    assert_eq!(req.action, ActionKind::ManageDockerSandbox);
    let token = gateway.approve_request(req.request_id, None).unwrap();

    // Token is consumed
    let _ = ctrl.execute_action_approved(DockerActionType::Start, "02-ai-gpu-sandbox", &gateway, &token);

    // Replay with consumed token must fail
    let replay = ctrl.execute_action_approved(DockerActionType::Start, "02-ai-gpu-sandbox", &gateway, &token);
    assert!(replay.is_err(), "Replaying a consumed token must be rejected");
}

#[test]
fn test_windows_dpapi_vault_and_system_security() {
    let vault = WindowsDpapiVault::new();
    let secret_credential = b"mock_token_AIDO_SEALED_PAT_CREDENTIAL_KEY_2026_RTX5090";

    // Encrypt with DPAPI
    let ciphertext = vault.protect(secret_credential).expect("DPAPI protect must succeed");
    assert_ne!(ciphertext, secret_credential);

    // Decrypt with DPAPI
    let plaintext = vault.unprotect(&ciphertext).expect("DPAPI unprotect must succeed");
    assert_eq!(plaintext, secret_credential);

    // Test security guard
    let guard = SystemSecurityGuard::new();
    let report = guard.generate_report();
    assert!(!report.security_context.user_name.is_empty());
    assert_eq!(report.monitored_ports.len(), 5);

    // Check that Hermes on port 18434 is marked as protected
    let hermes = report.monitored_ports.iter().find(|p| p.port == 18434).unwrap();
    assert!(hermes.is_protected_service);
    assert!(hermes.note_de.contains("Hermes"));
    println!("Windows Security Guard Report: {:?}", report.security_context);
}
