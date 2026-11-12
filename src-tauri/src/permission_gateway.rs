use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use url::{Host, SyntaxViolation, Url};
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
    ///
    /// The decision uses a parsed URL (scheme, userinfo, host, port), not a
    /// string prefix. Userinfo, unexpected schemes, backslashes, and embedded
    /// whitespace or control characters are rejected. In PrivateOffline and
    /// LocalConnected the host spelling must be the parser's canonical form,
    /// so decimal, octal, hex, and compressed IPv6 tricks cannot impersonate
    /// a loopback or LAN address.
    ///
    /// PrivateOffline allows loopback only (`127.0.0.0/8`, `::1`, `localhost`).
    /// LocalConnected also allows RFC 1918 addresses and the exact names
    /// `https://tunnel.aido.local` and `http://aido-remote.local`.
    /// HybridAI allows any http(s) host without userinfo.
    pub fn check_endpoint_allowed(&self, url: &str) -> Result<(), GatewayError> {
        if !endpoint_allowed(url, self.privacy_mode()) {
            self.blocked_external_requests
                .fetch_add(1, Ordering::SeqCst);
            return Err(GatewayError::OfflineViolation(url.to_string()));
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

    /// Alias for consuming a one-time ApprovalToken right before task execution.
    pub fn consume_token_for_execution(
        &self,
        token: &ApprovalToken,
        expected_action: &ActionKind,
    ) -> Result<(), GatewayError> {
        self.consume_approval_token(token, expected_action)
    }
}

fn endpoint_allowed(raw: &str, mode: PrivacyMode) -> bool {
    let trimmed = raw.trim();
    if trimmed.is_empty() || contains_disallowed_char(trimmed) || authority_has_userinfo(trimmed) {
        return false;
    }
    let Ok(parsed) = parse_strict_url(trimmed) else {
        return false;
    };
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return false;
    }
    if !parsed.username().is_empty() || parsed.password().is_some() || parsed.host().is_none() {
        return false;
    }
    host_permitted(&parsed, trimmed, mode)
}

fn contains_disallowed_char(input: &str) -> bool {
    input
        .chars()
        .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
}

fn authority_has_userinfo(url: &str) -> bool {
    let Some((_, after_scheme)) = url.split_once("://") else {
        return true;
    };
    let authority = after_scheme.split(['/', '?', '#']).next().unwrap_or("");
    authority.contains('@')
}

fn parse_strict_url(input: &str) -> Result<Url, ()> {
    let blocked = Cell::new(false);
    let parsed = Url::options()
        .syntax_violation_callback(Some(&|violation| {
            if matches!(
                violation,
                SyntaxViolation::EmbeddedCredentials
                    | SyntaxViolation::UnencodedAtSign
                    | SyntaxViolation::Backslash
                    | SyntaxViolation::TabOrNewlineIgnored
                    | SyntaxViolation::C0SpaceIgnored
            ) {
                blocked.set(true);
            }
        }))
        .parse(input)
        .map_err(|_| ())?;
    if blocked.get() {
        return Err(());
    }
    Ok(parsed)
}

fn host_permitted(parsed: &Url, raw: &str, mode: PrivacyMode) -> bool {
    let Some(host) = parsed.host() else {
        return false;
    };
    if mode != PrivacyMode::HybridAI && !host_spelling_is_canonical(raw, parsed) {
        return false;
    }
    match mode {
        PrivacyMode::HybridAI => true,
        PrivacyMode::PrivateOffline => is_loopback_host(&host),
        PrivacyMode::LocalConnected => {
            is_loopback_host(&host) || is_local_name_or_lan(&host, parsed.scheme())
        }
    }
}

fn host_spelling_is_canonical(raw: &str, parsed: &Url) -> bool {
    match (raw_host_token(raw), parsed.host_str()) {
        (Some(raw_host), Some(parsed_host)) if !raw_host.is_empty() => {
            raw_host.eq_ignore_ascii_case(parsed_host)
        }
        _ => false,
    }
}

/// Host token from `scheme://host[:port]...`, including IPv6 brackets.
fn raw_host_token(url: &str) -> Option<&str> {
    let authority = url.split_once("://")?.1.split(['/', '?', '#']).next()?;
    if authority.is_empty() || authority.contains('@') {
        return None;
    }
    if let Some(rest) = authority.strip_prefix('[') {
        let end = rest.find(']')?;
        let after = &rest[end + 1..];
        if !after.is_empty() {
            let port = after.strip_prefix(':')?;
            if !is_numeric_port(port) {
                return None;
            }
        }
        let host_len = end + 2;
        return authority.get(..host_len);
    }
    match authority.rsplit_once(':') {
        Some((host, port)) if is_numeric_port(port) => Some(host),
        _ => Some(authority),
    }
}

fn is_numeric_port(port: &str) -> bool {
    !port.is_empty() && port.bytes().all(|byte| byte.is_ascii_digit())
}

fn is_loopback_host(host: &Host<&str>) -> bool {
    match host {
        Host::Ipv4(ip) => ip.is_loopback(),
        Host::Ipv6(ip) => ip.is_loopback(),
        Host::Domain(domain) => *domain == "localhost",
    }
}

fn is_local_name_or_lan(host: &Host<&str>, scheme: &str) -> bool {
    match host {
        Host::Ipv4(ip) => is_rfc1918(*ip),
        Host::Ipv6(_) => false,
        Host::Domain(domain) => {
            (*domain == "tunnel.aido.local" && scheme == "https")
                || (*domain == "aido-remote.local" && scheme == "http")
        }
    }
}

fn is_rfc1918(ip: Ipv4Addr) -> bool {
    match ip.octets() {
        [10, ..] => true,
        [172, second, ..] if (16..=31).contains(&second) => true,
        [192, 168, ..] => true,
        _ => false,
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

    fn assert_all_allowed(mode: PrivacyMode, urls: &[&str]) {
        let gw = PermissionGateway::new(AccessMode::StandardAccess, mode);
        for url in urls {
            assert!(
                gw.check_endpoint_allowed(url).is_ok(),
                "expected {url:?} to be allowed in {mode:?}"
            );
        }
        assert_eq!(gw.blocked_external_requests(), 0);
    }

    fn assert_all_blocked(mode: PrivacyMode, urls: &[&str]) {
        let gw = PermissionGateway::new(AccessMode::StandardAccess, mode);
        for url in urls {
            assert_eq!(
                gw.check_endpoint_allowed(url),
                Err(GatewayError::OfflineViolation((*url).to_string())),
                "expected {url:?} to be blocked in {mode:?}"
            );
        }
        assert_eq!(gw.blocked_external_requests(), urls.len() as u64);
    }

    #[test]
    fn endpoint_guard_allows_legitimate_loopback_lan_and_tunnels() {
        let loopback = [
            "http://127.0.0.1:18434/v1/chat/completions",
            "http://127.0.0.1:18434",
            "http://127.0.0.1/",
            "http://127.0.0.1",
            "https://127.0.0.1",
            "https://127.0.0.1:18434/models",
            "http://127.0.0.2:18434/models",
            "http://localhost:18080/inference",
            "http://localhost",
            "https://localhost/",
            "http://LocalHost:18080/inference",
            "HTTP://127.0.0.1:18434/models",
            "  http://127.0.0.1:18434/models  ",
            "\r\nhttp://localhost:18080/inference\t",
            "http://[::1]:18434/v1/chat/completions",
            "https://[::1]/",
        ];
        assert_all_allowed(PrivacyMode::PrivateOffline, &loopback);
        assert_all_allowed(PrivacyMode::LocalConnected, &loopback);

        assert_all_allowed(
            PrivacyMode::LocalConnected,
            &[
                "http://192.168.1.42:18435/remote",
                "https://192.168.0.1/",
                "http://192.168.255.255/api",
                "http://10.0.0.5:8080/api",
                "https://10.255.255.255/",
                "http://172.16.0.1/",
                "http://172.20.1.10:3000",
                "https://172.31.255.254/",
                "https://tunnel.aido.local/cockpit",
                "https://tunnel.aido.local",
                "HTTPS://TUNNEL.AIDO.LOCAL/cockpit",
                "https://tunnel.aido.local:8443/cockpit",
                "http://aido-remote.local",
                "http://aido-remote.local:8080/api",
                "HTTP://AIDO-REMOTE.LOCAL",
            ],
        );

        assert_all_allowed(
            PrivacyMode::HybridAI,
            &[
                "https://api.openai.com/v1/chat/completions",
                "http://127.0.0.1:18434/v1/chat/completions",
                "HTTP://API.OPENAI.COM/v1",
            ],
        );
    }

    #[test]
    fn endpoint_guard_rejects_prefix_and_obfuscation_bypasses() {
        let shared = [
            "http://localhost:1@evil.example",
            "http://127.0.0.1:18434@evil.example",
            "http://user:secret@127.0.0.1:18434/v1/chat/completions",
            "http://@127.0.0.1:18434/",
            "https://tunnel.aido.local@evil.example/cockpit",
            "http://127.0.0.1.evil.example/",
            "http://localhost.evil.example/",
            "http://LOCALHOST.evil.example/",
            "http://localhost.localdomain/",
            "http://2130706433/",
            "http://2130706433",
            "http://0x7f000001/",
            "http://0x7f.0.0.1/",
            "http://127.1/",
            "http://127.0.1/",
            "http://0177.0.0.1/",
            "http://127.00.0.1/",
            "http://[::ffff:127.0.0.1]/",
            "http://[0:0:0:0:0:0:0:1]/",
            "http://[::ffff:8.8.8.8]/",
            "http://[::ffff:10.0.0.1]/",
            "http://%6c%6f%63%61%6c%68%6f%73%74/",
            "http://127.0.0.1%2eevil.example/",
            "http://127.0.0.1\t.evil.example",
            "http://127.0.0.1 .evil.example",
            "http://localhost\n@evil.example",
            "http://127.0.0.1:18434\\@evil.example",
            "http://evil.com\\@127.0.0.1",
            "ftp://127.0.0.1/",
            "ws://127.0.0.1:18434",
            "file://127.0.0.1/etc/passwd",
            "javascript:alert(1)",
            "",
            "   ",
            "not a url",
            "http://",
            "://127.0.0.1",
            "http://0.0.0.0:18434/",
        ];
        assert_all_blocked(PrivacyMode::PrivateOffline, &shared);
        assert_all_blocked(PrivacyMode::LocalConnected, &shared);

        assert_all_blocked(
            PrivacyMode::PrivateOffline,
            &[
                "http://192.168.1.42:18435/remote",
                "http://10.0.0.5:8080/api",
                "http://172.20.1.10:3000",
                "https://tunnel.aido.local/cockpit",
                "http://aido-remote.local",
                "https://api.openai.com/v1/chat/completions",
            ],
        );

        assert_all_blocked(
            PrivacyMode::LocalConnected,
            &[
                "http://10.evil.example",
                "http://10.evil.example:8080/",
                "https://10.evil.example",
                "http://10.EVIL.example",
                "http://192.168.evil.example",
                "https://192.168.evil.example/path",
                "https://tunnel.aido.local.evil",
                "https://tunnel.aido.local.evil/cockpit",
                "https://TUNNEL.AIDO.LOCAL.EVIL",
                "https://evil.tunnel.aido.local/cockpit",
                "http://tunnel.aido.local/cockpit",
                "https://aido-remote.local",
                "http://aido-remote.local.evil",
                "http://foo.aido-remote.local",
                "https://tunnel.aido.localhost/",
                "http://172.15.255.255/",
                "http://172.32.0.1/",
                "http://192.167.1.1/",
                "http://192.169.1.1/",
                "http://11.0.0.1/",
                "http://9.255.255.255/",
                "http://169.254.1.1/",
                "http://100.64.0.1/",
                "http://8.8.8.8/",
                "http://167772161/",
                "http://10.1/",
                "http://192.168.1/",
                "http://[fc00::1]/",
                "http://[fe80::1]/",
                "https://api.openai.com/v1/chat/completions",
            ],
        );

        assert_all_blocked(
            PrivacyMode::HybridAI,
            &[
                "http://localhost:1@evil.example",
                "http://user:secret@api.openai.com/v1",
                "file:///etc/passwd",
                "ftp://127.0.0.1/",
                "http://127.0.0.1:18434\\@evil.example",
            ],
        );
    }
}
