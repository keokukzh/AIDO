use aido_core::memory::MemoryEngine;
use aido_core::permission_gateway::{
    AccessMode, ActionKind, ApprovalToken, PermissionGateway, PrivacyMode,
};
use aido_core::sensitive_actions::SensitiveActionQueue;
use aido_core::ui_automation::UiAutomationEngine;
use std::path::{Path, PathBuf};
use std::time::Duration;
use uuid::Uuid;

struct TempRoot(PathBuf);

impl TempRoot {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("aido-gate-{}", Uuid::new_v4()));
        std::fs::create_dir_all(path.join("data")).unwrap();
        Self(path)
    }

    fn memory(&self) -> MemoryEngine {
        MemoryEngine::new(self.0.join("data/aido_memory.db")).unwrap()
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn standard_gateway() -> PermissionGateway {
    PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline)
}

#[test]
fn unload_requires_a_real_one_time_token_and_writes_an_audit_entry() {
    let root = TempRoot::new();
    let memory = root.memory();
    let gateway = standard_gateway();
    let queue = SensitiveActionQueue::new();
    let proposal = queue.propose_unload(&gateway).expect("preview");
    assert_eq!(proposal.phase, "proposal");
    assert_eq!(proposal.execute_command, "unload_model_vram");
    assert!(proposal.blast_summary_de.contains("Grafikspeicher"));

    let mut ran = false;
    let missing = queue.run_after_peek(
        &gateway,
        &memory,
        None,
        ActionKind::UnloadIdleModelSlot,
        "UnloadModelVram",
        |_, _| {
            ran = true;
            Ok(())
        },
    );
    assert!(missing.is_err());
    assert!(!ran, "missing token must not execute");

    let forged = ApprovalToken {
        token_id: Uuid::new_v4(),
        request_id: proposal.id,
        action: ActionKind::UnloadIdleModelSlot,
        issued_at: chrono::Utc::now(),
        user_note: Some("selbst freigegeben".into()),
    };
    let forged_result = queue.run_after_peek(
        &gateway,
        &memory,
        Some(&forged),
        ActionKind::UnloadIdleModelSlot,
        "UnloadModelVram",
        |_, _| {
            ran = true;
            Ok(())
        },
    );
    assert!(forged_result.is_err());
    assert!(
        !ran,
        "a preview id without a user-issued token must not execute"
    );

    let token = gateway
        .approve_request(proposal.id, Some("Freigabe durch Nutzer".into()))
        .unwrap();
    let roundtrip: ApprovalToken =
        serde_json::from_value(serde_json::to_value(&token).unwrap()).unwrap();
    let once = queue.execute_unload(&gateway, &memory, Some(&roundtrip));
    assert!(
        once.is_ok(),
        "first approved unload should pass the gate: {once:?}"
    );
    let replay = queue.execute_unload(&gateway, &memory, Some(&roundtrip));
    assert!(replay.is_err(), "the same token must not unload twice");

    let audit = memory.list_audit_log(20).unwrap();
    assert!(
        audit
            .iter()
            .any(|entry| entry.action == "UnloadModelVram" && entry.approved),
        "successful unload must be audited: {audit:?}"
    );
    assert!(
        audit
            .iter()
            .any(|entry| entry.action == "UnloadModelVram" && !entry.approved),
        "a refused unload must also leave an audit entry"
    );
}

#[test]
fn launch_uses_only_the_approved_target_and_rejects_replay() {
    let root = TempRoot::new();
    let memory = root.memory();
    let gateway = standard_gateway();
    let queue = SensitiveActionQueue::new();
    let proposal = queue
        .propose_launch(&gateway, "Alpha", "cursor")
        .expect("preview");
    assert!(proposal.human_explanation.contains("Alpha"));
    assert!(proposal.human_explanation.contains("cursor"));

    let token = gateway.approve_request(proposal.id, None).unwrap();
    let mut calls = 0;
    let mut seen = String::new();
    let started = queue.run_after_peek(
        &gateway,
        &memory,
        Some(&token),
        ActionKind::LaunchProject,
        "LaunchProjectOrchestrated",
        |prepared, approved| {
            calls += 1;
            let (name, mode) = prepared.approved_launch_target()?;
            seen = format!("{name}:{mode}");
            gateway
                .consume_approval_token(approved, &ActionKind::LaunchProject)
                .map_err(|error| error.to_string())?;
            Ok(())
        },
    );
    assert!(started.is_ok());
    assert_eq!(calls, 1);
    assert_eq!(seen, "Alpha:cursor");

    let replay = queue.run_after_peek(
        &gateway,
        &memory,
        Some(&token),
        ActionKind::LaunchProject,
        "LaunchProjectOrchestrated",
        |_, _| {
            calls += 1;
            Ok(())
        },
    );
    assert!(replay.is_err());
    assert_eq!(calls, 1, "replay must not launch again");
    let audit = memory.list_audit_log(10).unwrap();
    assert!(audit
        .iter()
        .any(|entry| entry.action == "LaunchProjectOrchestrated" && entry.approved));
}

#[test]
fn capture_rejects_hash_mismatch_and_audits_a_one_time_success_without_the_image() {
    let root = TempRoot::new();
    let memory = root.memory();
    let gateway = standard_gateway();
    let queue = SensitiveActionQueue::new();

    let readonly = PermissionGateway::new(AccessMode::ReadOnly, PrivacyMode::PrivateOffline);
    assert!(SensitiveActionQueue::new()
        .propose_capture(&readonly)
        .is_err());

    let proposal = queue.propose_capture(&gateway).expect("preview");
    assert_eq!(proposal.risk_level, "RED_SYSTEM");
    assert!(proposal.blast_radius.privacy_sensitive);
    assert!(!proposal.blast_radius.leaves_device);
    let token = gateway
        .approve_request(proposal.id, Some("Bildschirm einmalig".into()))
        .unwrap();

    let tampered_dir = root.0.join("tampered");
    std::fs::create_dir_all(&tampered_dir).unwrap();
    let tampered = tampered_dir.join("not-agent-desktop.exe");
    std::fs::write(&tampered, b"not the pinned driver").unwrap();
    assert!(tampered.is_absolute());

    let mut ran = false;
    let mismatch = queue.execute_capture(
        &gateway,
        &memory,
        &UiAutomationEngine::with_driver_path(&tampered),
        Some(&token),
        |_| {
            ran = true;
            Ok("SECRETPIXEL".into())
        },
    );
    assert!(mismatch.is_err(), "hash mismatch must be rejected");
    assert!(!ran, "a mismatched binary must not be started");
    assert!(
        gateway
            .peek_approval_token(&token, &ActionKind::CaptureDesktopScreen)
            .is_ok(),
        "a failed integrity check must not burn the user's token"
    );

    let image = queue.execute_capture(
        &gateway,
        &memory,
        &UiAutomationEngine::new(),
        Some(&token),
        |driver| {
            ran = true;
            assert!(
                driver.is_absolute(),
                "driver path must not be PATH/CWD relative"
            );
            assert_ne!(driver, Path::new("agent-desktop.exe"));
            Ok("SECRETPIXEL".into())
        },
    );
    assert_eq!(image.unwrap(), "SECRETPIXEL");
    assert!(ran);

    ran = false;
    let replay = queue.execute_capture(
        &gateway,
        &memory,
        &UiAutomationEngine::new(),
        Some(&token),
        |_| {
            ran = true;
            Ok("SECRETPIXEL".into())
        },
    );
    assert!(replay.is_err());
    assert!(!ran, "screenshot token is single-use");

    let audit = memory.list_audit_log(20).unwrap();
    let success = audit
        .iter()
        .find(|entry| entry.action == "CaptureDesktopScreen" && entry.approved)
        .expect("capture audit entry");
    assert!(!success.summary.contains("SECRETPIXEL"));
    assert!(success.summary.contains("Bildschirm"));
}

#[test]
fn expired_preview_does_not_execute() {
    let root = TempRoot::new();
    let memory = root.memory();
    let gateway = standard_gateway();
    let queue = SensitiveActionQueue::with_lifetime(Duration::ZERO);
    let proposal = queue
        .propose_launch(&gateway, "Alpha", "antigravity")
        .unwrap();
    let token = gateway.approve_request(proposal.id, None).unwrap();
    let mut ran = false;
    let expired = queue.run_after_peek(
        &gateway,
        &memory,
        Some(&token),
        ActionKind::LaunchProject,
        "LaunchProjectOrchestrated",
        |_, _| {
            ran = true;
            Ok(())
        },
    );
    assert!(expired.is_err());
    assert!(!ran);
}
