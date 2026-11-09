use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessMode {
    ReadOnly,
    StandardAccess,
    FullAccess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivacyMode {
    PrivateOffline,
    LocalConnected,
    HybridAI,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionKind {
    /// Read-only inspection of hardware, VRAM, RAM, and processes
    SystemHealthCheck,
    /// Read-only dry-run preview of Fast Cleanup
    FastCleanupPreview,
    /// State-modifying Fast Cleanup (requires explicit user ApprovalToken)
    FastCleanupExecute,
    /// Unload idle local AI model slot on 127.0.0.1
    UnloadIdleModelSlot,
    /// Read-only discovery of installed apps and capabilities
    DiscoverApps,
    /// Read-only harmless capability verification
    VerifyAppCapability,
    /// State-modifying app action execution (requires explicit user ApprovalToken)
    ExecuteAppAction,
    /// Read-only scanning of project directories and workspaces
    ScanProjects,
    /// State-modifying project launch (requires explicit user ApprovalToken)
    LaunchProject,
    /// Read-only inspection of hardware specs and existing model files
    InspectHardwareAndModels,
    /// State-modifying execution of an installer step
    RunInstallerStep,
    /// State-modifying rollback of completed installer steps
    RollbackInstaller,
    /// Read-only inspection and health probing of local AI runtimes
    ProbeLocalRuntimes,
}

impl ActionKind {
    pub fn is_mutating(&self) -> bool {
        matches!(
            self,
            ActionKind::FastCleanupExecute
                | ActionKind::UnloadIdleModelSlot
                | ActionKind::ExecuteAppAction
                | ActionKind::LaunchProject
                | ActionKind::RunInstallerStep
                | ActionKind::RollbackInstaller
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub request_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub action: ActionKind,
    pub human_title: String,
    pub human_explanation: String,
    pub target_summary: Vec<String>,
    pub estimated_freed_bytes: u64,
    pub estimated_freed_vram_mib: u64,
    pub approved: Option<bool>,
    pub user_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalToken {
    pub token_id: Uuid,
    pub request_id: Uuid,
    pub action: ActionKind,
    pub issued_at: DateTime<Utc>,
    pub user_note: Option<String>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GatewayError {
    #[error("Aktion im ReadOnly-Modus gesperrt: {0}")]
    ReadOnlyBlocked(String),
    #[error("Fehlende oder ungültige Nutzerfreigabe für Aktion: {0}")]
    ApprovalRequired(String),
    #[error("Netzwerkziel im Private-Offline-Modus verboten (nur 127.0.0.1 erlaubt): {0}")]
    OfflineViolation(String),
    #[error("Freigabe-Anfrage nicht gefunden oder bereits verwendet: {0}")]
    InvalidRequest(Uuid),
}

#[derive(Debug, Clone)]
pub struct PermissionGateway {
    access_mode: Arc<Mutex<AccessMode>>,
    privacy_mode: Arc<Mutex<PrivacyMode>>,
    pending_requests: Arc<Mutex<HashMap<Uuid, ApprovalRequest>>>,
    valid_tokens: Arc<Mutex<HashMap<Uuid, ApprovalToken>>>,
}

impl Default for PermissionGateway {
    fn default() -> Self {
        Self::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline)
    }
}

impl PermissionGateway {
    pub fn new(access_mode: AccessMode, privacy_mode: PrivacyMode) -> Self {
        Self {
            access_mode: Arc::new(Mutex::new(access_mode)),
            privacy_mode: Arc::new(Mutex::new(privacy_mode)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
            valid_tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn access_mode(&self) -> AccessMode {
        *self.access_mode.lock().unwrap()
    }

    pub fn set_access_mode(&self, mode: AccessMode) {
        *self.access_mode.lock().unwrap() = mode;
    }

    pub fn privacy_mode(&self) -> PrivacyMode {
        *self.privacy_mode.lock().unwrap()
    }

    pub fn set_privacy_mode(&self, mode: PrivacyMode) {
        *self.privacy_mode.lock().unwrap() = mode;
    }

    /// Verifies that an outgoing endpoint complies with the active PrivacyMode.
    /// In PrivateOffline mode, only loopback addresses (127.0.0.1 / localhost) are permitted.
    pub fn check_endpoint_allowed(&self, url: &str) -> Result<(), GatewayError> {
        let mode = self.privacy_mode();
        if mode == PrivacyMode::PrivateOffline {
            let lower = url.trim().to_ascii_lowercase();
            let is_loopback = lower.starts_with("http://127.0.0.1:")
                || lower.starts_with("http://127.0.0.1/")
                || lower == "http://127.0.0.1"
                || lower.starts_with("http://localhost:")
                || lower.starts_with("http://localhost/")
                || lower == "http://localhost";
            if !is_loopback {
                return Err(GatewayError::OfflineViolation(url.to_string()));
            }
        }
        Ok(())
    }

    /// Checks whether a read-only action is allowed immediately.
    pub fn check_readonly_allowed(&self, action: &ActionKind) -> Result<(), GatewayError> {
        if action.is_mutating() {
            return Err(GatewayError::ApprovalRequired(format!("{:?}", action)));
        }
        Ok(())
    }

    /// Creates a human-understandable ApprovalRequest for a mutating action.
    pub fn request_approval(
        &self,
        action: ActionKind,
        human_title: impl Into<String>,
        human_explanation: impl Into<String>,
        target_summary: Vec<String>,
        estimated_freed_bytes: u64,
        estimated_freed_vram_mib: u64,
    ) -> Result<ApprovalRequest, GatewayError> {
        if self.access_mode() == AccessMode::ReadOnly && action.is_mutating() {
            return Err(GatewayError::ReadOnlyBlocked(format!("{:?}", action)));
        }
        let req = ApprovalRequest {
            request_id: Uuid::new_v4(),
            created_at: Utc::now(),
            action,
            human_title: human_title.into(),
            human_explanation: human_explanation.into(),
            target_summary,
            estimated_freed_bytes,
            estimated_freed_vram_mib,
            approved: None,
            user_note: None,
        };
        self.pending_requests
            .lock()
            .unwrap()
            .insert(req.request_id, req.clone());
        Ok(req)
    }

    /// Called when the user explicitly clicks "Ja, jetzt bereinigen" in the UI.
    pub fn approve_request(
        &self,
        request_id: Uuid,
        user_note: Option<String>,
    ) -> Result<ApprovalToken, GatewayError> {
        if self.access_mode() == AccessMode::ReadOnly {
            return Err(GatewayError::ReadOnlyBlocked(
                "Genehmigung im ReadOnly-Modus nicht möglich".into(),
            ));
        }
        let mut pending = self.pending_requests.lock().unwrap();
        let req = pending
            .get_mut(&request_id)
            .ok_or(GatewayError::InvalidRequest(request_id))?;
        req.approved = Some(true);
        req.user_note = user_note.clone();

        let token = ApprovalToken {
            token_id: Uuid::new_v4(),
            request_id,
            action: req.action.clone(),
            issued_at: Utc::now(),
            user_note,
        };
        self.valid_tokens
            .lock()
            .unwrap()
            .insert(token.token_id, token.clone());
        Ok(token)
    }

    /// Called when the user clicks "Nein, abbrechen".
    pub fn reject_request(
        &self,
        request_id: Uuid,
        user_note: Option<String>,
    ) -> Result<ApprovalRequest, GatewayError> {
        let mut pending = self.pending_requests.lock().unwrap();
        let req = pending
            .get_mut(&request_id)
            .ok_or(GatewayError::InvalidRequest(request_id))?;
        req.approved = Some(false);
        req.user_note = user_note;
        Ok(req.clone())
    }

    /// Consumes a one-time ApprovalToken right before executing a mutating action.
    pub fn consume_approval_token(
        &self,
        token: &ApprovalToken,
        expected_action: &ActionKind,
    ) -> Result<(), GatewayError> {
        if self.access_mode() == AccessMode::ReadOnly {
            return Err(GatewayError::ReadOnlyBlocked(format!(
                "{:?}",
                expected_action
            )));
        }
        let mut tokens = self.valid_tokens.lock().unwrap();
        let stored = tokens
            .remove(&token.token_id)
            .ok_or_else(|| GatewayError::ApprovalRequired(format!("{:?}", expected_action)))?;

        if &stored.action != expected_action {
            return Err(GatewayError::ApprovalRequired(format!(
                "Token gilt für {:?}, nicht für {:?}",
                stored.action, expected_action
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_offline_blocks_external_urls() {
        let gw = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);
        assert!(gw.check_endpoint_allowed("http://127.0.0.1:18434/v1/chat/completions").is_ok());
        assert!(gw.check_endpoint_allowed("http://localhost:18080/inference").is_ok());
        assert_eq!(
            gw.check_endpoint_allowed("https://api.openai.com/v1/chat/completions"),
            Err(GatewayError::OfflineViolation(
                "https://api.openai.com/v1/chat/completions".into()
            ))
        );
    }

    #[test]
    fn readonly_mode_blocks_mutating_approval_and_execution() {
        let gw = PermissionGateway::new(AccessMode::ReadOnly, PrivacyMode::PrivateOffline);
        assert!(gw.check_readonly_allowed(&ActionKind::SystemHealthCheck).is_ok());
        let res = gw.request_approval(
            ActionKind::FastCleanupExecute,
            "Fast Cleanup",
            "Bereinigt temporäre AIDO-Caches",
            vec!["temp".into()],
            1024,
            0,
        );
        assert!(matches!(res, Err(GatewayError::ReadOnlyBlocked(_))));
    }

    #[test]
    fn mutating_action_requires_one_time_approval_token() {
        let gw = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);
        let req = gw
            .request_approval(
                ActionKind::FastCleanupExecute,
                "Fast Cleanup",
                "Bereinigt temporäre AIDO-Testdateien",
                vec!["aido_scratch_cache".into()],
                4096,
                0,
            )
            .unwrap();

        let token = gw.approve_request(req.request_id, Some("Freigegeben".into())).unwrap();

        // First consumption succeeds
        assert!(gw
            .consume_approval_token(&token, &ActionKind::FastCleanupExecute)
            .is_ok());

        // Replay of the same token must fail
        assert!(matches!(
            gw.consume_approval_token(&token, &ActionKind::FastCleanupExecute),
            Err(GatewayError::ApprovalRequired(_))
        ));
    }

    #[test]
    fn action_kind_mutating_classification() {
        assert!(!ActionKind::SystemHealthCheck.is_mutating());
        assert!(!ActionKind::FastCleanupPreview.is_mutating());
        assert!(!ActionKind::DiscoverApps.is_mutating());
        assert!(!ActionKind::VerifyAppCapability.is_mutating());
        assert!(!ActionKind::ScanProjects.is_mutating());
        assert!(!ActionKind::InspectHardwareAndModels.is_mutating());
        assert!(!ActionKind::ProbeLocalRuntimes.is_mutating());

        assert!(ActionKind::FastCleanupExecute.is_mutating());
        assert!(ActionKind::UnloadIdleModelSlot.is_mutating());
        assert!(ActionKind::ExecuteAppAction.is_mutating());
        assert!(ActionKind::LaunchProject.is_mutating());
        assert!(ActionKind::RunInstallerStep.is_mutating());
        assert!(ActionKind::RollbackInstaller.is_mutating());
    }
}
