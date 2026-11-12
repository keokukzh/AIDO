use aido_core::desktop_setup::{SetupSettings, SetupStore};
use aido_core::permission_gateway::{AccessMode, PrivacyMode};

#[test]
fn aido_setup_resumes_without_marking_a_draft_complete() {
    let directory = std::env::temp_dir().join(format!("aido-setup-{}", uuid::Uuid::new_v4()));
    let database = directory.join("setup.db");
    let store = SetupStore::new(&database).unwrap();
    let mut draft = store.load().unwrap();
    assert_eq!(draft.access_mode, AccessMode::StandardAccess);
    assert_eq!(draft.privacy_mode, PrivacyMode::PrivateOffline);
    assert!(!draft.completed);
    draft.step = 2;
    store.save(&draft).unwrap();
    let reopened = SetupStore::new(&database).unwrap().load().unwrap();
    assert_eq!(reopened.step, 2);
    assert!(!reopened.completed);
    assert!(SetupSettings::default().project_roots.is_empty());
    assert!(store.active().unwrap().is_none());
    draft.completed = true;
    store.activate(&draft).unwrap();
    draft.completed = false;
    draft.access_mode = AccessMode::ReadOnly;
    store.save(&draft).unwrap();
    assert_eq!(store.load().unwrap().access_mode, AccessMode::ReadOnly);
    let active = store.active().unwrap().unwrap();
    assert!(active.completed);
    assert_eq!(active.access_mode, AccessMode::StandardAccess);
}
