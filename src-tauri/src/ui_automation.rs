use crate::memory::MemoryEngine;
use crate::permission_gateway::{
    AccessMode, ActionKind, ApprovalRequest, ApprovalToken, GatewayError, PermissionGateway,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UiAutomationError {
    #[error("I/O- oder Ausführungsfehler des UI-Treibers: {0}")]
    Io(#[from] std::io::Error),
    #[error(
        "Treiber 'agent-desktop.exe' wurde nicht gefunden. Bitte Installation oder Tooling prüfen."
    )]
    DriverNotFound,
    #[error(
        "SHA-256 von agent-desktop.exe stimmt nicht ({path}). Die Datei wird nicht gestartet."
    )]
    HashMismatch { path: String },
    #[error(
        "agent-desktop.exe muss ein absoluter Pfad aus dem Ressourcenordner oder dem bekannten Installationspfad sein."
    )]
    UntrustedDriverPath,
    #[error("Treiber meldet Fehler: {0}")]
    DriverFailed(String),
    #[error("JSON-Parsing-Fehler des UI-Snapshots: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Berechtigungsfehler des Gateways: {0}")]
    Gateway(#[from] GatewayError),
    #[error("Aktion verlangt ein gültiges ApprovalToken im StandardAccess-Modus.")]
    ApprovalRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub title: String,
    pub app: String,
    pub bounds: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiElement {
    #[serde(rename = "ref")]
    pub element_ref: String,
    pub role: String,
    pub name: Option<String>,
    pub value: Option<String>,
    pub enabled: Option<bool>,
    pub focused: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSnapshot {
    pub app: String,
    pub snapshot_id: Option<String>,
    pub elements: Vec<UiElement>,
    pub raw_json: String,
    pub human_summary_de: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiAction {
    Click { element_ref: String },
    TypeText { element_ref: String, text: String },
    SetValue { element_ref: String, value: String },
    PressKey { combo: String },
    FocusWindow { app: String },
    CloseApp { app: String },
}

impl UiAction {
    pub fn description_de(&self) -> String {
        match self {
            UiAction::Click { element_ref } => {
                format!("Klick auf Element-Referenz '{}'", element_ref)
            }
            UiAction::TypeText { element_ref, text } => {
                format!("Texteingabe in '{}': \"{}\"", element_ref, text)
            }
            UiAction::SetValue { element_ref, value } => {
                format!("Wert setzen in '{}': \"{}\"", element_ref, value)
            }
            UiAction::PressKey { combo } => format!("Tastenkombination drücken: '{}'", combo),
            UiAction::FocusWindow { app } => format!("Fenster aktivieren: '{}'", app),
            UiAction::CloseApp { app } => format!("Anwendung schließen: '{}'", app),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiActionResult {
    pub success: bool,
    pub app: String,
    pub action_summary: String,
    pub disposition: Option<String>,
    pub human_summary_de: String,
}

/// SHA-256 of the bundled `tooling/agent-desktop/agent-desktop.exe`.
/// Keep this in sync with `scripts/stage-rea.mjs`.
pub const AGENT_DESKTOP_SHA256: &str =
    "d274667031322e50b9c2b1ab81c2ca46835d52cc932769a5c34bd4d481fd94c3";

/// Optional absolute override. The file is started only when its SHA-256 matches the pin.
pub const AGENT_DESKTOP_ENV: &str = "AIDO_AGENT_DESKTOP_PATH";

const BUNDLED_AGENT_DESKTOP: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../tooling/agent-desktop/agent-desktop.exe"
);

#[derive(Debug, Clone, Default)]
pub struct UiAutomationEngine {
    custom_driver_path: Option<PathBuf>,
    resource_dir: Option<PathBuf>,
}

impl UiAutomationEngine {
    pub fn new() -> Self {
        Self {
            custom_driver_path: None,
            resource_dir: None,
        }
    }

    pub fn with_driver_path(path: impl Into<PathBuf>) -> Self {
        Self {
            custom_driver_path: Some(path.into()),
            resource_dir: None,
        }
    }

    pub fn with_resource_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.resource_dir = Some(dir.into());
        self
    }

    /// Resolves `agent-desktop.exe` only from an absolute, hash-pinned location.
    ///
    /// Order, with no CWD or PATH fallback:
    /// 1. `AIDO_AGENT_DESKTOP_PATH`, if set. A relative path or hash mismatch fails closed.
    /// 2. An explicit driver path on this engine, with the same hash check.
    /// 3. `resource_dir/agent-desktop/agent-desktop.exe` when that file exists.
    /// 4. The absolute path of the binary bundled in this repository.
    pub fn resolve_driver(&self) -> Result<PathBuf, UiAutomationError> {
        let env_override = std::env::var_os(AGENT_DESKTOP_ENV).map(PathBuf::from);
        self.resolve_for_launch(self.resource_dir.as_deref(), env_override.as_deref())
    }

    pub fn resolve_for_launch(
        &self,
        resource_dir: Option<&Path>,
        env_override: Option<&Path>,
    ) -> Result<PathBuf, UiAutomationError> {
        if let Some(path) = env_override {
            return verify_pinned_driver(path);
        }
        if let Some(path) = &self.custom_driver_path {
            return verify_pinned_driver(path);
        }
        verify_installed_driver(resource_dir.or(self.resource_dir.as_deref()))
    }

    /// Read-only discovery: Lists all currently visible application windows via agent-desktop.
    pub fn list_windows(
        &self,
        gateway: &PermissionGateway,
        app_filter: Option<&str>,
    ) -> Result<Vec<WindowInfo>, UiAutomationError> {
        gateway.check_readonly_allowed(&ActionKind::UiObserve)?;
        let driver = self.resolve_driver()?;

        let mut cmd = Command::new(driver);
        cmd.arg("list-windows");
        if let Some(app) = app_filter {
            cmd.args(["--app", app]);
        }

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let output = cmd.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        let parsed: serde_json::Value = serde_json::from_str(&stdout)
            .unwrap_or(serde_json::json!({ "data": { "windows": [] } }));

        let mut windows = Vec::new();
        if let Some(arr) = parsed
            .get("data")
            .and_then(|d| d.get("windows"))
            .and_then(|w| w.as_array())
        {
            for item in arr {
                let title = item
                    .get("title")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();
                let app = item
                    .get("app")
                    .and_then(|a| a.as_str())
                    .unwrap_or("")
                    .to_string();
                let bounds = item.get("bounds").map(|b| b.to_string());
                windows.push(WindowInfo { title, app, bounds });
            }
        }

        Ok(windows)
    }

    /// Read-only snapshot of an application's UI accessibility tree.
    pub fn take_snapshot(
        &self,
        gateway: &PermissionGateway,
        app: &str,
    ) -> Result<UiSnapshot, UiAutomationError> {
        gateway.check_readonly_allowed(&ActionKind::UiObserve)?;
        let driver = self.resolve_driver()?;

        let mut cmd = Command::new(driver);
        cmd.args(["snapshot", "--app", app]);

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let output = cmd.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        let parsed: serde_json::Value = serde_json::from_str(&stdout).map_err(|e| {
            UiAutomationError::DriverFailed(format!(
                "Snapshot-Ausgabe konnte nicht als JSON geparst werden: {} (Ausgabe: {})",
                e, stdout
            ))
        })?;

        let snapshot_id = parsed
            .get("data")
            .and_then(|d| d.get("snapshot_id"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        let mut elements = Vec::new();
        if let Some(nodes) = parsed
            .get("data")
            .and_then(|d| d.get("nodes"))
            .and_then(|n| n.as_array())
        {
            for node in nodes {
                if let Some(r) = node.get("ref").and_then(|r| r.as_str()) {
                    let role = node
                        .get("role")
                        .and_then(|r| r.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let name = node
                        .get("name")
                        .and_then(|n| n.as_str())
                        .map(|n| n.to_string());
                    let value = node
                        .get("value")
                        .and_then(|v| v.as_str())
                        .map(|v| v.to_string());
                    let enabled = node.get("enabled").and_then(|e| e.as_bool());
                    let focused = node.get("focused").and_then(|f| f.as_bool());

                    elements.push(UiElement {
                        element_ref: r.to_string(),
                        role,
                        name,
                        value,
                        enabled,
                        focused,
                    });
                }
            }
        }

        let human_summary_de = format!(
            "UI-Snapshot für '{}' erfolgreich erfasst ({} interaktive Bedienelemente erkannt).",
            app,
            elements.len()
        );

        Ok(UiSnapshot {
            app: app.to_string(),
            snapshot_id,
            elements,
            raw_json: stdout.to_string(),
            human_summary_de,
        })
    }

    /// Prepares an approval request for a mutating UI action (StandardAccess).
    pub fn prepare_action_approval(
        &self,
        gateway: &PermissionGateway,
        app: &str,
        action: &UiAction,
    ) -> Result<ApprovalRequest, UiAutomationError> {
        let title = format!("AIDO möchte eine Eingabe in '{}' ausführen", app);
        let explanation = format!(
            "Geplante Aktion: {}. Im Standard-Zugriffsmodus ist für jede mutierende Eingabe eine Bestätigung erforderlich.",
            action.description_de()
        );

        let req = gateway.request_approval(
            ActionKind::UiAct,
            title,
            explanation,
            vec![format!("App: {}", app), action.description_de()],
            0,
            0,
        )?;

        Ok(req)
    }

    /// Executes a UI action with strict Permission Gateway enforcement and optional SQLite audit logging.
    pub fn execute_action(
        &self,
        gateway: &PermissionGateway,
        app: &str,
        action: &UiAction,
        token: Option<ApprovalToken>,
        memory: Option<&MemoryEngine>,
    ) -> Result<UiActionResult, UiAutomationError> {
        // Enforce ReadOnly check
        if gateway.access_mode() == AccessMode::ReadOnly {
            return Err(UiAutomationError::Gateway(GatewayError::ReadOnlyBlocked(
                format!("{:?}", ActionKind::UiAct),
            )));
        }

        // Enforce token consumption in StandardAccess
        if gateway.access_mode() == AccessMode::StandardAccess {
            let tok = token.as_ref().ok_or(UiAutomationError::ApprovalRequired)?;
            gateway.consume_approval_token(tok, &ActionKind::UiAct)?;
        }

        let driver = self.resolve_driver()?;
        let mut cmd = Command::new(driver);

        match action {
            UiAction::Click { element_ref } => {
                cmd.args(["click", element_ref]);
            }
            UiAction::TypeText { element_ref, text } => {
                cmd.args(["type", element_ref, text]);
            }
            UiAction::SetValue { element_ref, value } => {
                cmd.args(["set-value", element_ref, value]);
            }
            UiAction::PressKey { combo } => {
                cmd.args(["press", combo]);
            }
            UiAction::FocusWindow { app: target_app } => {
                cmd.args(["focus-window", "--app", target_app]);
            }
            UiAction::CloseApp { app: target_app } => {
                cmd.args(["close-app", "--app", target_app]);
            }
        }

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        let output = cmd.output()?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        let parsed: serde_json::Value =
            serde_json::from_str(&stdout).unwrap_or(serde_json::Value::Null);
        let ok = parsed
            .get("ok")
            .and_then(|o| o.as_bool())
            .unwrap_or(output.status.success());
        let disposition = parsed
            .get("data")
            .and_then(|d| d.get("disposition"))
            .map(|d| d.to_string());

        let summary = format!(
            "Aktion '{}' in App '{}' ausgeführt (Status: {})",
            action.description_de(),
            app,
            if ok { "Erfolgreich" } else { "Fehlgeschlagen" }
        );

        // Record to SQLite audit_log if memory engine is provided
        if let Some(mem) = memory {
            let mode_str = format!("{:?}", gateway.access_mode());
            let target_str = format!("app:{}", app);
            let _ = mem.record_audit("UiAutomation", &mode_str, &target_str, ok, &summary);
        }

        if !ok {
            return Err(UiAutomationError::DriverFailed(format!(
                "Aktion schlug fehl: {} (stderr: {})",
                stdout, stderr
            )));
        }

        Ok(UiActionResult {
            success: true,
            app: app.to_string(),
            action_summary: action.description_de(),
            disposition,
            human_summary_de: summary,
        })
    }
}

/// Accepts only the bundled resource copy or the known repository binary.
pub fn verify_installed_driver(resource_dir: Option<&Path>) -> Result<PathBuf, UiAutomationError> {
    if let Some(dir) = resource_dir {
        let candidate = dir.join("agent-desktop").join("agent-desktop.exe");
        if candidate.exists() {
            return verify_pinned_driver(&candidate);
        }
    }
    let bundled = PathBuf::from(BUNDLED_AGENT_DESKTOP);
    if bundled.is_file() {
        return verify_pinned_driver(&bundled);
    }
    Err(UiAutomationError::DriverNotFound)
}

/// Rejects relative paths, missing files, and any SHA-256 other than the pin.
pub fn verify_pinned_driver(path: &Path) -> Result<PathBuf, UiAutomationError> {
    if !path.is_absolute() {
        return Err(UiAutomationError::UntrustedDriverPath);
    }
    let canonical = path
        .canonicalize()
        .map_err(|_| UiAutomationError::DriverNotFound)?;
    let bytes = std::fs::read(&canonical)?;
    let digest = crate::rea_bridge::sha256_digest(&bytes);
    if !digest.eq_ignore_ascii_case(AGENT_DESKTOP_SHA256) {
        return Err(UiAutomationError::HashMismatch {
            path: canonical.display().to_string(),
        });
    }
    Ok(strip_verbatim_prefix(canonical))
}

fn strip_verbatim_prefix(path: PathBuf) -> PathBuf {
    let text = path.to_string_lossy();
    if let Some(local) = text.strip_prefix(r"\\?\") {
        PathBuf::from(local)
    } else {
        path
    }
}

/// Runs a desktop screenshot only after the driver path has been hash-checked again.
pub fn run_screenshot(driver: &Path) -> Result<String, UiAutomationError> {
    let driver = verify_pinned_driver(driver)?;
    let mut cmd = Command::new(&driver);
    cmd.args(["screenshot", "--screen", "0"]);
    cmd.stdin(std::process::Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let output = cmd.output()?;
    if !output.status.success() {
        return Err(UiAutomationError::DriverFailed(
            "Screenshot-Aufruf fehlgeschlagen.".into(),
        ));
    }
    let base64 = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if base64.is_empty() {
        return Err(UiAutomationError::DriverFailed(
            "Screenshot lieferte keine Bilddaten.".into(),
        ));
    }
    Ok(base64)
}

#[cfg(test)]
mod driver_pin_tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let path =
            std::env::temp_dir().join(format!("aido-driver-{name}-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn relative_and_path_fallbacks_are_rejected() {
        let relative = Path::new("tooling/agent-desktop/agent-desktop.exe");
        assert!(matches!(
            verify_pinned_driver(relative),
            Err(UiAutomationError::UntrustedDriverPath)
        ));
        assert!(matches!(
            verify_pinned_driver(Path::new("agent-desktop.exe")),
            Err(UiAutomationError::UntrustedDriverPath)
        ));
        let engine = UiAutomationEngine::new();
        assert!(matches!(
            engine.resolve_for_launch(
                None,
                Some(Path::new("tooling/agent-desktop/agent-desktop.exe"))
            ),
            Err(UiAutomationError::UntrustedDriverPath)
        ));
    }

    #[test]
    fn hash_mismatch_rejects_absolute_file_without_falling_through() {
        let dir = temp_dir("mismatch");
        let resource = dir.join("agent-desktop");
        std::fs::create_dir_all(&resource).unwrap();
        let tampered = resource.join("agent-desktop.exe");
        std::fs::write(&tampered, b"this is not the pinned driver").unwrap();

        let mismatch = verify_installed_driver(Some(&dir));
        assert!(
            matches!(&mismatch, Err(UiAutomationError::HashMismatch { .. })),
            "a tampered resource must not fall through to another binary: {mismatch:?}"
        );

        let direct = verify_pinned_driver(&tampered);
        assert!(matches!(
            direct,
            Err(UiAutomationError::HashMismatch { .. })
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn bundled_driver_matches_the_pinned_sha256_and_is_absolute() {
        let resolved = verify_installed_driver(None).expect("bundled driver");
        assert!(resolved.is_absolute());
        assert!(resolved.ends_with("agent-desktop.exe"));
        let again = verify_pinned_driver(&resolved).expect("recheck");
        assert_eq!(again, resolved);

        let empty = temp_dir("empty-resource");
        let from_empty = verify_installed_driver(Some(&empty)).expect("fallback to known path");
        assert_eq!(from_empty, resolved);
        let _ = std::fs::remove_dir_all(&empty);
    }
}
