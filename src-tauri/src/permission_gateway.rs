use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
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
    /// Read-only inspection of deep cleanup staging artifacts
    DeepCleanupScan,
    /// State-modifying Deep Cleanup of stale build/shader/crash artifacts (requires explicit user ApprovalToken)
    DeepCleanupExecute,
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
    /// Read-only synthesis and isolated verification of unknown app adapter
    SynthesizeAndVerifyAdapter,
    /// State-modifying clean uninstallation of AIDO caches and database (requires explicit user ApprovalToken)
    ExecuteCleanUninstall,
    /// State-modifying pairing of a smartphone remote cockpit (requires explicit user confirmation / PIN)
    PairRemoteDevice,
    /// State-modifying revocation of a paired smartphone remote session
    RevokeRemoteDevice,
    /// Read-only on-demand desktop snapshot and vision summary for paired smartphone
    CaptureAidoBlickSnapshot,
    /// State-modifying elevation to scoped admin task via AdminBroker (requires explicit ApprovalToken and UAC isolation)
    ElevateAdminBrokerTask,
    /// State-modifying management of WSL2 / RTX 5090 Docker Sandboxes (requires explicit ApprovalToken)
    ManageDockerSandbox,
    /// Read-only observation of an Android phone via ADB (device listing, UI hierarchy dump, screenshot)
    PhoneObserve,
    /// State-modifying action on an Android phone via ADB (tap, swipe, type, key, launch app, open URL)
    PhoneAct,
    /// Sensitive state-modifying action on an Android phone via ADB (send SMS, start call, install APK, Wi-Fi pairing/connect)
    PhoneSensitiveAct,
    /// Read-only observation of desktop windows and UI accessibility trees via agent-desktop / UIA
    UiObserve,
    /// State-modifying action on desktop UI via agent-desktop / UIA (click, type, press key, set-value)
    UiAct,
    /// Privacy-sensitive capture of the visible desktop. Requires a one-time user
    /// approval because the image can contain personal data.
    CaptureDesktopScreen,
}

/// How far a gated action can reach. Shown to the user together with the German preview.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlastRadius {
    pub summary_de: String,
    pub leaves_device: bool,
    pub privacy_sensitive: bool,
    pub affects_other_programs: bool,
}

impl ActionKind {
    pub fn is_mutating(&self) -> bool {
        matches!(
            self,
            ActionKind::FastCleanupExecute
                | ActionKind::DeepCleanupExecute
                | ActionKind::UnloadIdleModelSlot
                | ActionKind::ExecuteAppAction
                | ActionKind::LaunchProject
                | ActionKind::RunInstallerStep
                | ActionKind::RollbackInstaller
                | ActionKind::ExecuteCleanUninstall
                | ActionKind::PairRemoteDevice
                | ActionKind::RevokeRemoteDevice
                | ActionKind::ElevateAdminBrokerTask
                | ActionKind::ManageDockerSandbox
                | ActionKind::PhoneAct
                | ActionKind::PhoneSensitiveAct
                | ActionKind::UiAct
                | ActionKind::CaptureDesktopScreen
        )
    }

    /// German preview shown before the user grants a one-time token.
    pub fn preview_de(&self) -> &'static str {
        match self {
            ActionKind::CaptureDesktopScreen => {
                "AIDO nimmt den sichtbaren Bildschirm dieses PCs auf. Das Bild kann Fenster, Texte und personenbezogene Inhalte enthalten. Es bleibt auf diesem Gerät und wird nicht versendet."
            }
            ActionKind::UnloadIdleModelSlot => {
                "AIDO entlädt das lokale KI-Modell aus dem Grafikspeicher dieses PCs. Es werden keine Dateien gelöscht und keine Daten gesendet."
            }
            ActionKind::LaunchProject => {
                "AIDO startet das in der Vorschau genannte lokale Projekt mit dem genannten Programm. Es wird nichts hochgeladen."
            }
            _ => "AIDO möchte eine Aktion ausführen, die deine einmalige Freigabe braucht.",
        }
    }

    /// Reach of the action, independent of the concrete target names.
    pub fn blast_radius(&self) -> BlastRadius {
        let radius = |summary_de: &str,
                      leaves_device,
                      privacy_sensitive,
                      affects_other_programs| BlastRadius {
            summary_de: summary_de.to_string(),
            leaves_device,
            privacy_sensitive,
            affects_other_programs,
        };
        match self {
            ActionKind::SystemHealthCheck => radius(
                "Nur lesende Abfrage von Hardware und Auslastung auf diesem PC.",
                false,
                false,
                false,
            ),
            ActionKind::FastCleanupPreview => {
                radius("Nur eine Vorschau. Es wird nichts gelöscht.", false, false, false)
            }
            ActionKind::FastCleanupExecute => radius(
                "Löscht nur die vorher gezeigten temporären Dateien auf diesem PC.",
                false,
                false,
                true,
            ),
            ActionKind::DeepCleanupScan => radius(
                "Nur lesende Suche nach alten Zwischenspeichern auf diesem PC.",
                false,
                false,
                false,
            ),
            ActionKind::DeepCleanupExecute => radius(
                "Löscht die vorher gezeigten alten Build-, Shader- und Absturzdateien auf diesem PC.",
                false,
                false,
                true,
            ),
            ActionKind::UnloadIdleModelSlot => radius(
                "Gibt nur den Grafikspeicher des lokalen KI-Modells auf diesem PC frei. Keine Dateien, keine anderen Programme, kein Netzwerk.",
                false,
                false,
                false,
            ),
            ActionKind::DiscoverApps => {
                radius("Liest nur, welche Programme lokal erkannt sind.", false, false, false)
            }
            ActionKind::VerifyAppCapability => radius(
                "Prüft nur lokal, ob eine Fähigkeit benutzbar ist.",
                false,
                false,
                false,
            ),
            ActionKind::ExecuteAppAction => radius(
                "Startet oder steuert ein lokales Programm auf diesem PC.",
                false,
                false,
                true,
            ),
            ActionKind::ScanProjects => radius(
                "Liest nur freigegebene Projektordner auf diesem PC.",
                false,
                false,
                false,
            ),
            ActionKind::LaunchProject => radius(
                "Startet das angezeigte lokale Projekt mit einem lokalen Programm. Es wird nichts hochgeladen.",
                false,
                false,
                true,
            ),
            ActionKind::InspectHardwareAndModels => radius(
                "Liest nur Hardware und vorhandene Modelldateien.",
                false,
                false,
                false,
            ),
            ActionKind::RunInstallerStep => radius(
                "Führt einen Installationsschritt lokal auf diesem PC aus.",
                false,
                false,
                true,
            ),
            ActionKind::RollbackInstaller => radius(
                "Macht einen lokalen Installationsschritt rückgängig.",
                false,
                false,
                true,
            ),
            ActionKind::ProbeLocalRuntimes => radius(
                "Prüft nur lokale KI-Dienste auf 127.0.0.1.",
                false,
                false,
                false,
            ),
            ActionKind::SynthesizeAndVerifyAdapter => radius(
                "Prüft einen unbekannten Adapter nur lesend und isoliert.",
                false,
                false,
                false,
            ),
            ActionKind::ExecuteCleanUninstall => radius(
                "Entfernt lokale AIDO-Zwischenspeicher und die lokale Datenbank auf diesem PC.",
                false,
                false,
                false,
            ),
            ActionKind::PairRemoteDevice => radius(
                "Koppelt ein Gerät. Dabei können Verbindungsdaten diesen PC verlassen.",
                true,
                false,
                false,
            ),
            ActionKind::RevokeRemoteDevice => {
                radius("Widerruft eine Koppelung auf diesem PC.", false, false, false)
            }
            ActionKind::CaptureAidoBlickSnapshot => radius(
                "Liest den Bildschirm für ein gekoppeltes Gerät. Der Inhalt kann personenbezogen sein und das Gerät erreichen.",
                true,
                true,
                false,
            ),
            ActionKind::ElevateAdminBrokerTask => radius(
                "Führt eine eng begrenzte Aufgabe mit erhöhten Rechten auf diesem PC aus.",
                false,
                false,
                true,
            ),
            ActionKind::ManageDockerSandbox => radius(
                "Steuert eine lokale Docker-Umgebung auf diesem PC.",
                false,
                false,
                true,
            ),
            ActionKind::PhoneObserve => radius(
                "Liest den Bildschirm oder die Oberfläche eines verbundenen Telefons.",
                false,
                true,
                false,
            ),
            ActionKind::PhoneAct => radius(
                "Bedient ein verbundenes Telefon, zum Beispiel durch Tippen, Wischen oder Text.",
                false,
                false,
                true,
            ),
            ActionKind::PhoneSensitiveAct => radius(
                "Führt eine heikle Telefonaktion aus, zum Beispiel Anruf, SMS oder Installation.",
                true,
                true,
                true,
            ),
            ActionKind::UiObserve => radius(
                "Liest Fenster und Bedienelemente auf diesem PC.",
                false,
                true,
                false,
            ),
            ActionKind::UiAct => radius(
                "Klickt oder tippt in ein anderes Programm auf diesem PC.",
                false,
                false,
                true,
            ),
            ActionKind::CaptureDesktopScreen => radius(
                "Nimmt den sichtbaren Bildschirm dieses PCs auf. Das Bild kann Fenster, Texte und personenbezogene Inhalte enthalten. Es bleibt lokal und wird nicht versendet.",
                false,
                true,
                false,
            ),
        }
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

#[derive(Debug, Clone, Error, PartialEq, Eq)]
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
    external_bytes_sent: Arc<AtomicU64>,
    blocked_external_requests: Arc<AtomicU64>,
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
            external_bytes_sent: Arc::new(AtomicU64::new(0)),
            blocked_external_requests: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn external_bytes_sent(&self) -> u64 {
        self.external_bytes_sent.load(Ordering::SeqCst)
    }

    pub fn blocked_external_requests(&self) -> u64 {
        self.blocked_external_requests.load(Ordering::SeqCst)
    }

    pub fn record_external_bytes(&self, bytes: u64) {
        self.external_bytes_sent.fetch_add(bytes, Ordering::SeqCst);
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

    /// Returns the number of currently pending approval requests.
    pub fn pending_requests_count(&self) -> usize {
        self.pending_requests.lock().unwrap().len()
    }

    /// Returns the most recently created pending approval request, optionally filtered by ActionKind.
    pub fn latest_pending_request(&self, action: Option<&ActionKind>) -> Option<ApprovalRequest> {
        let pending = self.pending_requests.lock().unwrap();
        pending
            .values()
            .filter(|r| r.approved.is_none() && action.is_none_or(|a| &r.action == a))
            .max_by_key(|r| r.created_at)
            .cloned()
    }

    /// Verifies that an outgoing endpoint complies with the active PrivacyMode.
    /// In PrivateOffline mode, only loopback addresses (127.0.0.1 / localhost) are permitted.
    /// In LocalConnected mode, loopback, private LAN addresses (RFC 1918), and local tunnel endpoints are permitted.
    pub fn check_endpoint_allowed(&self, url: &str) -> Result<(), GatewayError> {
        let mode = self.privacy_mode();
        let lower = url.trim().to_ascii_lowercase();

        let is_loopback = lower.starts_with("http://127.0.0.1:")
            || lower.starts_with("http://127.0.0.1/")
            || lower == "http://127.0.0.1"
            || lower.starts_with("https://127.0.0.1:")
            || lower.starts_with("https://127.0.0.1/")
            || lower == "https://127.0.0.1"
            || lower.starts_with("http://localhost:")
            || lower.starts_with("http://localhost/")
            || lower == "http://localhost"
            || lower.starts_with("https://localhost:")
            || lower.starts_with("https://localhost/")
            || lower == "https://localhost";

        match mode {
            PrivacyMode::PrivateOffline => {
                if !is_loopback {
                    self.blocked_external_requests.fetch_add(1, Ordering::SeqCst);
                    return Err(GatewayError::OfflineViolation(url.to_string()));
                }
            }
            PrivacyMode::LocalConnected => {
                let is_lan_or_tunnel = lower.starts_with("http://192.168.")
                    || lower.starts_with("https://192.168.")
                    || lower.starts_with("http://10.")
                    || lower.starts_with("https://10.")
                    || lower.starts_with("https://tunnel.aido.local")
                    || lower.starts_with("http://aido-remote.local")
                    || (16..=31).any(|octet| {
                        lower.starts_with(&format!("http://172.{}.", octet))
                            || lower.starts_with(&format!("https://172.{}.", octet))
                    });

                if !is_loopback && !is_lan_or_tunnel {
                    self.blocked_external_requests.fetch_add(1, Ordering::SeqCst);
                    return Err(GatewayError::OfflineViolation(url.to_string()));
                }
            }
            PrivacyMode::HybridAI => {}
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

    /// Checks that a one-time token is present and matches the action without consuming it.
    pub fn peek_approval_token(
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
        let tokens = self.valid_tokens.lock().unwrap();
        let stored = tokens
            .get(&token.token_id)
            .ok_or_else(|| GatewayError::ApprovalRequired(format!("{:?}", expected_action)))?;
        if &stored.action != expected_action {
            return Err(GatewayError::ApprovalRequired(format!(
                "Token gilt für {:?}, nicht für {:?}",
                stored.action, expected_action
            )));
        }
        Ok(())
    }

    /// Alias for consuming a one-time ApprovalToken right before task execution.
    pub fn consume_token_for_execution(
        &self,
        token: &ApprovalToken,
        expected_action: &ActionKind,
    ) -> Result<(), GatewayError> {
        self.consume_approval_token(token, expected_action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_offline_blocks_external_urls() {
        let gw = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);
        assert!(gw
            .check_endpoint_allowed("http://127.0.0.1:18434/v1/chat/completions")
            .is_ok());
        assert!(gw
            .check_endpoint_allowed("http://localhost:18080/inference")
            .is_ok());
        assert_eq!(
            gw.check_endpoint_allowed("https://api.openai.com/v1/chat/completions"),
            Err(GatewayError::OfflineViolation(
                "https://api.openai.com/v1/chat/completions".into()
            ))
        );
    }

    #[test]
    fn local_connected_allows_lan_and_tunnel_but_blocks_external_cloud() {
        let gw = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::LocalConnected);
        assert!(gw
            .check_endpoint_allowed("http://127.0.0.1:18434/v1/chat/completions")
            .is_ok());
        assert!(gw
            .check_endpoint_allowed("http://192.168.1.42:18435/remote")
            .is_ok());
        assert!(gw
            .check_endpoint_allowed("http://10.0.0.5:8080/api")
            .is_ok());
        assert!(gw.check_endpoint_allowed("http://172.20.1.10:3000").is_ok());
        assert!(gw
            .check_endpoint_allowed("https://tunnel.aido.local/cockpit")
            .is_ok());
        assert!(gw
            .check_endpoint_allowed("http://aido-remote.local")
            .is_ok());
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
        assert!(gw
            .check_readonly_allowed(&ActionKind::SystemHealthCheck)
            .is_ok());
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

        let token = gw
            .approve_request(req.request_id, Some("Freigegeben".into()))
            .unwrap();

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
        assert!(!ActionKind::DeepCleanupScan.is_mutating());
        assert!(!ActionKind::DiscoverApps.is_mutating());
        assert!(!ActionKind::VerifyAppCapability.is_mutating());
        assert!(!ActionKind::ScanProjects.is_mutating());
        assert!(!ActionKind::InspectHardwareAndModels.is_mutating());
        assert!(!ActionKind::ProbeLocalRuntimes.is_mutating());
        assert!(!ActionKind::SynthesizeAndVerifyAdapter.is_mutating());
        assert!(!ActionKind::CaptureAidoBlickSnapshot.is_mutating());
        assert!(!ActionKind::PhoneObserve.is_mutating());

        assert!(ActionKind::FastCleanupExecute.is_mutating());
        assert!(ActionKind::DeepCleanupExecute.is_mutating());
        assert!(ActionKind::UnloadIdleModelSlot.is_mutating());
        assert!(ActionKind::ExecuteAppAction.is_mutating());
        assert!(ActionKind::LaunchProject.is_mutating());
        assert!(ActionKind::RunInstallerStep.is_mutating());
        assert!(ActionKind::RollbackInstaller.is_mutating());
        assert!(ActionKind::ExecuteCleanUninstall.is_mutating());
        assert!(ActionKind::PairRemoteDevice.is_mutating());
        assert!(ActionKind::RevokeRemoteDevice.is_mutating());
        assert!(ActionKind::PhoneAct.is_mutating());
        assert!(ActionKind::PhoneSensitiveAct.is_mutating());
        assert!(ActionKind::UiAct.is_mutating());
        assert!(ActionKind::CaptureDesktopScreen.is_mutating());
        assert!(!ActionKind::UiObserve.is_mutating());
    }

    #[test]
    fn peek_approval_token_does_not_consume_it() {
        let gw = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);
        let req = gw
            .request_approval(
                ActionKind::CaptureDesktopScreen,
                "Bildschirm aufnehmen",
                ActionKind::CaptureDesktopScreen.preview_de(),
                vec!["Bildschirm 0".into()],
                0,
                0,
            )
            .unwrap();
        let token = gw
            .approve_request(req.request_id, Some("Nutzer".into()))
            .unwrap();
        assert!(gw
            .peek_approval_token(&token, &ActionKind::CaptureDesktopScreen)
            .is_ok());
        assert!(gw
            .peek_approval_token(&token, &ActionKind::CaptureDesktopScreen)
            .is_ok());
        assert!(gw
            .consume_approval_token(&token, &ActionKind::CaptureDesktopScreen)
            .is_ok());
        assert!(matches!(
            gw.peek_approval_token(&token, &ActionKind::CaptureDesktopScreen),
            Err(GatewayError::ApprovalRequired(_))
        ));
    }

    #[test]
    fn capture_desktop_screen_blast_radius_stays_on_device_and_is_private() {
        let radius = ActionKind::CaptureDesktopScreen.blast_radius();
        assert!(radius.privacy_sensitive);
        assert!(!radius.leaves_device);
        assert!(radius.summary_de.contains("Bildschirm"));
        assert!(ActionKind::CaptureDesktopScreen
            .preview_de()
            .contains("nicht versendet"));
        let gw = PermissionGateway::new(AccessMode::ReadOnly, PrivacyMode::PrivateOffline);
        assert!(matches!(
            gw.check_readonly_allowed(&ActionKind::CaptureDesktopScreen),
            Err(GatewayError::ApprovalRequired(_))
        ));
    }

    #[test]
    fn test_telemetry_counters_and_offline_guard() {
        let gw = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);
        assert_eq!(gw.external_bytes_sent(), 0);
        assert_eq!(gw.blocked_external_requests(), 0);

        // Allowed loopback request
        assert!(gw
            .check_endpoint_allowed("http://127.0.0.1:18434/models")
            .is_ok());
        assert_eq!(gw.blocked_external_requests(), 0);

        // Blocked external request
        assert!(gw
            .check_endpoint_allowed("https://api.openai.com/v1/chat")
            .is_err());
        assert_eq!(gw.blocked_external_requests(), 1);

        // In PrivateOffline mode, external bytes sent remains strictly 0
        assert_eq!(gw.external_bytes_sent(), 0);
    }
}
