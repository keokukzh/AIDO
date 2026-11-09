use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;

use crate::permission_gateway::{ActionKind, ApprovalToken, GatewayError, PermissionGateway};

/// Trait abstracting execution of ADB commands so both real system processes and unit-test mocks can be used.
pub trait AdbRunner: Send + Sync {
    fn run(&self, serial: Option<&str>, args: &[&str]) -> Result<Vec<u8>, PhoneError>;
}

/// Fallback runner when no ADB installation is available on the host.
#[derive(Debug, Default)]
pub struct NoAdbRunner;

impl AdbRunner for NoAdbRunner {
    fn run(&self, _serial: Option<&str>, _args: &[&str]) -> Result<Vec<u8>, PhoneError> {
        Err(PhoneError::AdbNotFound)
    }
}

/// Real system ADB runner invoking `adb.exe` / `adb` via `std::process::Command`.
#[derive(Debug, Clone)]
pub struct SystemAdbRunner {
    pub adb_path: PathBuf,
}

impl SystemAdbRunner {
    pub fn new(adb_path: PathBuf) -> Self {
        Self { adb_path }
    }
}

impl AdbRunner for SystemAdbRunner {
    fn run(&self, serial: Option<&str>, args: &[&str]) -> Result<Vec<u8>, PhoneError> {
        let mut cmd = std::process::Command::new(&self.adb_path);

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        if let Some(s) = serial {
            cmd.arg("-s").arg(s);
        }
        for arg in args {
            cmd.arg(arg);
        }

        let output = cmd.output().map_err(|e| {
            PhoneError::CommandFailed(format!("Fehler beim Starten von ADB ('{}'): {}", self.adb_path.display(), e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let err_msg = if !stderr.is_empty() {
                stderr
            } else if !stdout.is_empty() {
                stdout
            } else {
                format!("ADB-Prozess beendet mit Exit-Code {:?}", output.status.code())
            };
            return Err(PhoneError::CommandFailed(err_msg));
        }

        Ok(output.stdout)
    }
}

type MockMatcher = Box<dyn Fn(Option<&str>, &[&str]) -> bool + Send + Sync>;
type MockRule = (MockMatcher, Result<Vec<u8>, PhoneError>);

/// Test double runner for deterministic unit testing without a connected device.
#[derive(Default)]
pub struct MockAdbRunner {
    pub recorded: Arc<Mutex<Vec<Vec<String>>>>,
    responses: Arc<Mutex<Vec<MockRule>>>,
    fallback: Arc<Mutex<Option<Vec<u8>>>>,
}

impl MockAdbRunner {
    pub fn new() -> Self {
        Self {
            recorded: Arc::new(Mutex::new(Vec::new())),
            responses: Arc::new(Mutex::new(Vec::new())),
            fallback: Arc::new(Mutex::new(Some(Vec::new()))),
        }
    }

    pub fn recorded_commands(&self) -> Vec<Vec<String>> {
        self.recorded.lock().unwrap().clone()
    }

    pub fn set_response_for_substr(&self, substr: &str, result: Result<Vec<u8>, PhoneError>) {
        let sub = substr.to_string();
        let matcher = move |_: Option<&str>, args: &[&str]| {
            let full = args.join(" ");
            full.contains(&sub)
        };
        self.responses
            .lock()
            .unwrap()
            .push((Box::new(matcher), result));
    }

    pub fn set_response_for_exact_args(&self, exact_args: &[&str], result: Result<Vec<u8>, PhoneError>) {
        let expected: Vec<String> = exact_args.iter().map(|s| s.to_string()).collect();
        let matcher = move |_: Option<&str>, args: &[&str]| {
            args == expected.as_slice()
        };
        self.responses
            .lock()
            .unwrap()
            .push((Box::new(matcher), result));
    }

    pub fn set_fallback(&self, bytes: Option<Vec<u8>>) {
        *self.fallback.lock().unwrap() = bytes;
    }
}

impl AdbRunner for MockAdbRunner {
    fn run(&self, serial: Option<&str>, args: &[&str]) -> Result<Vec<u8>, PhoneError> {
        let mut cmd_record = Vec::new();
        if let Some(s) = serial {
            cmd_record.push("-s".to_string());
            cmd_record.push(s.to_string());
        }
        for a in args {
            cmd_record.push(a.to_string());
        }
        self.recorded.lock().unwrap().push(cmd_record);

        let guard = self.responses.lock().unwrap();
        for (matcher, res) in guard.iter().rev() {
            if matcher(serial, args) {
                return res.clone();
            }
        }

        if let Some(fb) = self.fallback.lock().unwrap().as_ref() {
            return Ok(fb.clone());
        }

        Err(PhoneError::CommandFailed("MockAdbRunner: Kein passender Antwort-Eintrag".into()))
    }
}

/// Locates `adb.exe` / `adb` in standard Android SDK locations, %LOCALAPPDATA%, or PATH.
pub fn locate_adb() -> Option<PathBuf> {
    // 1. Check ANDROID_HOME or ANDROID_SDK_ROOT
    for env_var in &["ANDROID_HOME", "ANDROID_SDK_ROOT"] {
        if let Ok(dir) = std::env::var(env_var) {
            let candidate = PathBuf::from(&dir)
                .join("platform-tools")
                .join(if cfg!(windows) { "adb.exe" } else { "adb" });
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    // 2. Check standard Windows LocalAppData path
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        let candidate = PathBuf::from(local_appdata)
            .join("Android")
            .join("Sdk")
            .join("platform-tools")
            .join("adb.exe");
        if candidate.exists() {
            return Some(candidate);
        }
    }

    // 3. Search in PATH
    if let Ok(path_var) = std::env::var("PATH") {
        let binary_name = if cfg!(windows) { "adb.exe" } else { "adb" };
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(binary_name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    None
}

/// Information about a connected Android device.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PhoneDevice {
    pub serial: String,
    pub state: String,
    pub model: String,
    pub transport: String, // "usb" | "wifi"
}

/// Parsed UI node from Android uiautomator XML dump.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UiNode {
    pub index: usize,
    pub text: String,
    pub resource_id: String,
    pub content_desc: String,
    pub class_name: String,
    pub package: String,
    pub clickable: bool,
    pub focusable: bool,
    pub scrollable: bool,
    pub password: bool,
    pub bounds: (i32, i32, i32, i32),
    pub center: (i32, i32),
}

/// Complete observation snapshot of a phone's current screen and state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneObservation {
    pub serial: String,
    pub foreground_package: String,
    pub ui_nodes: Vec<UiNode>,
    pub screenshot_png_base64: Option<String>,
    pub secure_screen_suspected: bool,
    pub captured_at: DateTime<Utc>,
}

/// Standard Android hardware / navigation keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhoneKey {
    Back,
    Home,
    Recents,
    Enter,
    Delete,
}

/// Atomic phone action executable by the agent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PhoneAction {
    Tap { x: i32, y: i32 },
    TapNode { index: usize },
    Swipe { x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32 },
    TypeText { text: String },
    Key { key: PhoneKey },
    LaunchApp { package: String },
    OpenUrl { url: String },
    StartCall { number: String },
    SendSms { number: String, body: String },
    Wait { ms: u64 },
    Done { summary: String },
    HandOffToUser { reason: String },
}

/// Domain errors for phone bridge operations.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum PhoneError {
    #[error("ADB wurde nicht gefunden: Bitte Android SDK oder platform-tools installieren")]
    AdbNotFound,
    #[error("Kein Android-Gerät verbunden oder autorisiert")]
    NoDevice,
    #[error("ADB-Befehl fehlgeschlagen: {0}")]
    CommandFailed(String),
    #[error("Parsing-Fehler: {0}")]
    Parse(String),
    #[error("Gateway-Sicherheitsfehler: {0}")]
    Gateway(#[from] GatewayError),
    #[error("Sicherheitssperre (Schutz vertraulicher Daten): {0}")]
    SafetyBlocked(String),
}

/// Encodes raw bytes to standard RFC 4648 Base64 without third-party crates.
pub fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARSET[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARSET[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARSET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARSET[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// Bridge coordinating ADB communication, UI inspection, input validation, and safety controls.
#[derive(Clone)]
pub struct PhoneBridge {
    runner: Arc<dyn AdbRunner>,
}

impl PhoneBridge {
    pub fn new(runner: Arc<dyn AdbRunner>) -> Self {
        Self { runner }
    }

    pub fn runner(&self) -> &Arc<dyn AdbRunner> {
        &self.runner
    }

    /// Discovers connected devices via `adb devices -l`.
    pub fn list_devices(&self) -> Result<Vec<PhoneDevice>, PhoneError> {
        let output_bytes = self.runner.run(None, &["devices", "-l"])?;
        let output = String::from_utf8_lossy(&output_bytes);
        Ok(parse_adb_devices_output(&output))
    }

    /// Pairs an Android phone over Wi-Fi (`adb pair <host_port> <code>`).
    /// Requires user ApprovalToken for `ActionKind::PhoneSensitiveAct`.
    pub fn pair_wifi(
        &self,
        gateway: &PermissionGateway,
        token: &ApprovalToken,
        host_port: &str,
        code: &str,
    ) -> Result<String, PhoneError> {
        gateway.consume_approval_token(token, &ActionKind::PhoneSensitiveAct)?;

        let host_port = host_port.trim();
        let code = code.trim();

        // Strict input validation: host:port and digits
        if !is_valid_host_port(host_port) {
            return Err(PhoneError::SafetyBlocked(
                "Ungültige IP/Port-Angabe für Wi-Fi-Pairing".into(),
            ));
        }
        if !code.chars().all(|c| c.is_ascii_digit()) || code.len() < 6 {
            return Err(PhoneError::SafetyBlocked(
                "Ungültiger Pairing-Code: Mindestens 6 Ziffern erforderlich".into(),
            ));
        }

        let res = self.runner.run(None, &["pair", host_port, code])?;
        Ok(String::from_utf8_lossy(&res).trim().to_string())
    }

    /// Connects to a previously paired Wi-Fi device (`adb connect <host_port>`).
    /// Requires user ApprovalToken for `ActionKind::PhoneSensitiveAct`.
    pub fn connect_wifi(
        &self,
        gateway: &PermissionGateway,
        token: &ApprovalToken,
        host_port: &str,
    ) -> Result<String, PhoneError> {
        gateway.consume_approval_token(token, &ActionKind::PhoneSensitiveAct)?;

        let host_port = host_port.trim();
        if !is_valid_host_port(host_port) {
            return Err(PhoneError::SafetyBlocked(
                "Ungültige IP/Port-Angabe für Wi-Fi-Verbindung".into(),
            ));
        }

        let res = self.runner.run(None, &["connect", host_port])?;
        Ok(String::from_utf8_lossy(&res).trim().to_string())
    }

    /// Inspects the active screen: fetches the UI tree via uiautomator dump, detects the foreground app,
    /// and optionally captures a screenshot.
    pub fn observe(
        &self,
        gateway: &PermissionGateway,
        serial: Option<&str>,
        with_screenshot: bool,
    ) -> Result<PhoneObservation, PhoneError> {
        // Read-only check in gateway: fails if system is somehow misconfigured
        gateway.check_readonly_allowed(&ActionKind::PhoneObserve)?;

        // 1. Dump UI hierarchy
        let _ = self
            .runner
            .run(serial, &["shell", "uiautomator", "dump", "/sdcard/aido_ui.xml"]);

        let xml_bytes = self
            .runner
            .run(serial, &["exec-out", "cat", "/sdcard/aido_ui.xml"])
            .or_else(|_| self.runner.run(serial, &["shell", "cat", "/sdcard/aido_ui.xml"]))
            .or_else(|_| self.runner.run(serial, &["exec-out", "uiautomator", "dump", "/dev/tty"]))
            .unwrap_or_default();

        let xml_str = String::from_utf8_lossy(&xml_bytes);
        let ui_nodes = parse_uiautomator_xml(&xml_str);

        // 2. Query foreground app via dumpsys window
        let dumpsys_bytes = self
            .runner
            .run(serial, &["shell", "dumpsys", "window"])
            .unwrap_or_default();
        let dumpsys_str = String::from_utf8_lossy(&dumpsys_bytes);
        let foreground_package = parse_foreground_package(&dumpsys_str);

        // 3. Check for security screen indicators
        let secure_screen_suspected = dumpsys_str.contains("FLAG_SECURE")
            || Self::is_sensitive_package(&foreground_package)
            || ui_nodes.iter().any(|n| n.password);

        // 4. Screenshot (if requested)
        let screenshot_png_base64 = if with_screenshot {
            self.runner
                .run(serial, &["exec-out", "screencap", "-p"])
                .ok()
                .map(|b| base64_encode(&b))
        } else {
            None
        };

        Ok(PhoneObservation {
            serial: serial.unwrap_or("default").to_string(),
            foreground_package,
            ui_nodes,
            screenshot_png_base64,
            secure_screen_suspected,
            captured_at: Utc::now(),
        })
    }

    /// Hard blocklist check for banking, payment, wallet, 2FA, password manager, and lockscreen packages.
    pub fn is_sensitive_package(pkg: &str) -> bool {
        let p = pkg.to_lowercase();
        const SENSITIVE_KEYWORDS: &[&str] = &[
            "bank",
            "paypal",
            "revolut",
            "n26",
            "wallet",
            "pay",
            "twint",
            "1password",
            "bitwarden",
            "lastpass",
            "authenticator",
            "authy",
            "sparkasse",
            "postbank",
            "commerzbank",
            "dkb",
            "klarna",
            "stripe",
            "keyguard",
            "lockscreen",
        ];

        SENSITIVE_KEYWORDS.iter().any(|kw| p.contains(kw))
    }

    /// Checks if ADB Keyboard IME is installed and enabled on the phone.
    pub fn adb_keyboard_status(&self, serial: Option<&str>) -> bool {
        let res = self
            .runner
            .run(serial, &["shell", "ime", "list", "-a", "-s"])
            .or_else(|_| {
                self.runner
                    .run(serial, &["shell", "pm", "list", "packages", "com.android.adbkeyboard"])
            });

        res.map(|bytes| String::from_utf8_lossy(&bytes).contains("com.android.adbkeyboard"))
            .unwrap_or(false)
    }

    /// Installs ADB Keyboard APK from a local file path and sets it as active IME.
    /// Requires user ApprovalToken for `ActionKind::PhoneSensitiveAct`.
    pub fn install_adb_keyboard(
        &self,
        gateway: &PermissionGateway,
        token: &ApprovalToken,
        serial: Option<&str>,
        apk_path: &Path,
    ) -> Result<String, PhoneError> {
        gateway.consume_approval_token(token, &ActionKind::PhoneSensitiveAct)?;

        if !apk_path.exists() {
            return Err(PhoneError::CommandFailed(format!(
                "Lokale APK-Datei '{}' existiert nicht",
                apk_path.display()
            )));
        }

        let path_str = apk_path
            .to_str()
            .ok_or_else(|| PhoneError::CommandFailed("Ungültiger APK-Dateipfad".into()))?;

        self.runner.run(serial, &["install", "-r", path_str])?;
        let _ = self
            .runner
            .run(serial, &["shell", "ime", "enable", "com.android.adbkeyboard/.AdbIME"]);
        let _ = self
            .runner
            .run(serial, &["shell", "ime", "set", "com.android.adbkeyboard/.AdbIME"]);

        Ok("ADB Keyboard erfolgreich installiert und aktiviert".into())
    }

    /// Executes an atomic phone action with permission enforcement, strict input validation,
    /// and safety blocking against banking/password targets.
    pub fn execute(
        &self,
        gateway: &PermissionGateway,
        serial: Option<&str>,
        action: &PhoneAction,
        token: Option<&ApprovalToken>,
        last_observation: Option<&PhoneObservation>,
    ) -> Result<String, PhoneError> {
        // --- 1. SAFETY CHECKS BEFORE EXECUTION ---
        if let Some(obs) = last_observation {
            if Self::is_sensitive_package(&obs.foreground_package) {
                return Err(PhoneError::SafetyBlocked(format!(
                    "Sicherheitsrelevante App '{}' im Vordergrund aktiv. Steuerung automatisch gesperrt.",
                    obs.foreground_package
                )));
            }
        }

        match action {
            PhoneAction::LaunchApp { package } if Self::is_sensitive_package(package) => {
                return Err(PhoneError::SafetyBlocked(format!(
                    "Starten der sicherheitsrelevanten App '{}' blockiert.",
                    package
                )));
            }
            PhoneAction::TapNode { index } => {
                if let Some(obs) = last_observation {
                    if let Some(node) = obs.ui_nodes.iter().find(|n| n.index == *index) {
                        if node.password {
                            return Err(PhoneError::SafetyBlocked(format!(
                                "Tippen auf Passwortfeld #{} blockiert.",
                                index
                            )));
                        }
                    }
                }
            }
            PhoneAction::TypeText { .. } => {
                if let Some(obs) = last_observation {
                    if obs.ui_nodes.iter().any(|n| n.password) {
                        return Err(PhoneError::SafetyBlocked(
                            "Texteingabe blockiert: Bildschirm enthält ein Passwortfeld.".into(),
                        ));
                    }
                }
            }
            _ => {}
        }

        // --- 2. NO-OP / CONTROL FLOW ACTIONS (No ADB, No Token) ---
        match action {
            PhoneAction::Wait { ms } => {
                std::thread::sleep(std::time::Duration::from_millis(*ms));
                return Ok(format!("{} ms gewartet", ms));
            }
            PhoneAction::Done { summary } => {
                return Ok(format!("Ziel erreicht: {}", summary));
            }
            PhoneAction::HandOffToUser { reason } => {
                return Ok(format!("An Nutzer übergeben: {}", reason));
            }
            _ => {}
        }

        // --- 3. GATEWAY TOKEN ENFORCEMENT & CONSUMPTION ---
        let required_token = token.ok_or_else(|| {
            PhoneError::Gateway(GatewayError::ApprovalRequired(format!("{:?}", action)))
        })?;

        match action {
            PhoneAction::SendSms { .. } | PhoneAction::StartCall { .. } => {
                gateway.consume_approval_token(required_token, &ActionKind::PhoneSensitiveAct)?;
            }
            _ => {
                gateway.consume_approval_token(required_token, &ActionKind::PhoneAct)?;
            }
        }

        // --- 4. INPUT VALIDATION (ANTI-INJECTION) & ADB DISPATCH ---
        match action {
            PhoneAction::Tap { x, y } => {
                self.runner
                    .run(serial, &["shell", "input", "tap", &x.to_string(), &y.to_string()])?;
                Ok(format!("Tippen bei ({}, {}) ausgeführt", x, y))
            }
            PhoneAction::TapNode { index } => {
                let obs = last_observation.ok_or_else(|| {
                    PhoneError::Parse("TapNode erfordert eine vorherige Beobachtung".into())
                })?;
                let node = obs
                    .ui_nodes
                    .iter()
                    .find(|n| n.index == *index)
                    .ok_or_else(|| {
                        PhoneError::Parse(format!(
                            "Node mit Index {} nicht in letzter Beobachtung gefunden",
                            index
                        ))
                    })?;
                let (cx, cy) = node.center;
                self.runner
                    .run(serial, &["shell", "input", "tap", &cx.to_string(), &cy.to_string()])?;
                Ok(format!(
                    "Tippen auf Node #{} ('{}') bei ({}, {}) ausgeführt",
                    index, node.text, cx, cy
                ))
            }
            PhoneAction::Swipe {
                x1,
                y1,
                x2,
                y2,
                duration_ms,
            } => {
                self.runner.run(
                    serial,
                    &[
                        "shell",
                        "input",
                        "swipe",
                        &x1.to_string(),
                        &y1.to_string(),
                        &x2.to_string(),
                        &y2.to_string(),
                        &duration_ms.to_string(),
                    ],
                )?;
                Ok(format!(
                    "Wischen von ({},{}) nach ({},{}) in {}ms ausgeführt",
                    x1, y1, x2, y2, duration_ms
                ))
            }
            PhoneAction::Key { key } => {
                let keycode = match key {
                    PhoneKey::Back => "KEYCODE_BACK",
                    PhoneKey::Home => "KEYCODE_HOME",
                    PhoneKey::Recents => "KEYCODE_APP_SWITCH",
                    PhoneKey::Enter => "KEYCODE_ENTER",
                    PhoneKey::Delete => "KEYCODE_DEL",
                };
                self.runner
                    .run(serial, &["shell", "input", "keyevent", keycode])?;
                Ok(format!("Taste '{:?}' gesendet", key))
            }
            PhoneAction::LaunchApp { package } => {
                if !is_valid_package_name(package) {
                    return Err(PhoneError::SafetyBlocked(format!(
                        "Ungültiger Paketname '{}' (unerlaubte Zeichen / Shell-Injection)",
                        package
                    )));
                }
                self.runner.run(
                    serial,
                    &[
                        "shell",
                        "monkey",
                        "-p",
                        package,
                        "-c",
                        "android.intent.category.LAUNCHER",
                        "1",
                    ],
                )?;
                Ok(format!("App '{}' gestartet", package))
            }
            PhoneAction::OpenUrl { url } => {
                if !is_valid_url(url) {
                    return Err(PhoneError::SafetyBlocked(format!(
                        "Ungültige oder unsichere URL '{}'",
                        url
                    )));
                }
                self.runner.run(
                    serial,
                    &["shell", "am", "start", "-a", "android.intent.action.VIEW", "-d", url],
                )?;
                Ok(format!("URL '{}' geöffnet", url))
            }
            PhoneAction::StartCall { number } => {
                let clean_num: String = number.chars().filter(|c| !c.is_whitespace()).collect();
                if !is_valid_phone_number(&clean_num) {
                    return Err(PhoneError::SafetyBlocked(format!(
                        "Ungültige Telefonnummer '{}' (unerlaubte Zeichen)",
                        number
                    )));
                }
                let tel_uri = format!("tel:{}", clean_num);
                self.runner.run(
                    serial,
                    &["shell", "am", "start", "-a", "android.intent.action.CALL", "-d", &tel_uri],
                )?;
                Ok(format!("Anruf an '{}' gestartet", number))
            }
            PhoneAction::SendSms { number, body } => {
                let clean_num: String = number.chars().filter(|c| !c.is_whitespace()).collect();
                if !is_valid_phone_number(&clean_num) {
                    return Err(PhoneError::SafetyBlocked(format!(
                        "Ungültige Telefonnummer '{}' (unerlaubte Zeichen)",
                        number
                    )));
                }
                let sms_uri = format!("sms:{}", clean_num);
                self.runner.run(
                    serial,
                    &[
                        "shell",
                        "am",
                        "start",
                        "-a",
                        "android.intent.action.SENDTO",
                        "-d",
                        &sms_uri,
                        "--es",
                        "sms_body",
                        body,
                    ],
                )?;
                Ok(format!("SMS an '{}' vorbereitet (Composer geöffnet)", number))
            }
            PhoneAction::TypeText { text } => {
                let has_adb_kb = self.adb_keyboard_status(serial);
                if has_adb_kb {
                    let b64 = base64_encode(text.as_bytes());
                    self.runner.run(
                        serial,
                        &[
                            "shell",
                            "am",
                            "broadcast",
                            "-a",
                            "ADB_INPUT_B64",
                            "--es",
                            "msg",
                            &b64,
                        ],
                    )?;
                    Ok(format!("Text via ADB Keyboard gesendet: '{}'", text))
                } else {
                    if !text.is_ascii() {
                        return Err(PhoneError::CommandFailed(
                            "Texteingabe mit Umlauten/Sonderzeichen erfordert ADB Keyboard (com.android.adbkeyboard). Bitte 'install_adb_keyboard' ausführen.".into(),
                        ));
                    }
                    if text
                        .chars()
                        .any(|c| matches!(c, ';' | '&' | '|' | '`' | '$' | '\\' | '"' | '\n' | '\r'))
                    {
                        return Err(PhoneError::SafetyBlocked(
                            "Sicherheitsrisiko: Ungültige Sonderzeichen im Eingabetext".into(),
                        ));
                    }
                    let escaped = text.replace(' ', "%s");
                    self.runner
                        .run(serial, &["shell", "input", "text", &escaped])?;
                    Ok(format!("Text via ASCII-Fallback gesendet: '{}'", text))
                }
            }
            _ => unreachable!(),
        }
    }
}

/// Validates phone numbers: only digits, spaces, and leading '+' allowed.
fn is_valid_phone_number(num: &str) -> bool {
    if num.is_empty() {
        return false;
    }
    let chars: Vec<char> = num.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if *c == '+' {
            if i != 0 {
                return false;
            }
        } else if !c.is_ascii_digit() && *c != '-' && *c != ' ' {
            return false;
        }
    }
    true
}

/// Validates package names: only [A-Za-z0-9._] allowed.
fn is_valid_package_name(pkg: &str) -> bool {
    !pkg.is_empty()
        && pkg
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_')
}

/// Validates URLs: must start with http://, https://, tel:, or sms: and contain no control chars.
fn is_valid_url(url: &str) -> bool {
    let lower = url.to_lowercase();
    let valid_prefix = lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("tel:")
        || lower.starts_with("sms:");
    if !valid_prefix {
        return false;
    }
    !url
        .chars()
        .any(|c| c.is_whitespace() || matches!(c, ';' | '&' | '|' | '`' | '$' | '"' | '\''))
}

/// Validates host:port notation for Wi-Fi pairing/connecting.
fn is_valid_host_port(hp: &str) -> bool {
    if hp.is_empty() {
        return false;
    }
    hp.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == ':' || c == '-')
}

/// Parses the output of `adb devices -l`.
pub fn parse_adb_devices_output(output: &str) -> Vec<PhoneDevice> {
    let mut devices = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty()
            || line.starts_with("List of devices attached")
            || line.starts_with('*')
        {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let serial = parts[0].to_string();
        let state = parts.get(1).unwrap_or(&"unknown").to_string();

        let mut model = "Unknown".to_string();
        for part in &parts[2..] {
            if let Some(m) = part.strip_prefix("model:") {
                model = m.to_string();
                break;
            }
        }

        let transport = if serial.contains(':') || line.contains("transport:wifi") {
            "wifi".to_string()
        } else {
            "usb".to_string()
        };

        devices.push(PhoneDevice {
            serial,
            state,
            model,
            transport,
        });
    }
    devices
}

/// Extracts the active foreground package name from `dumpsys window` output.
pub fn parse_foreground_package(dumpsys: &str) -> String {
    for line in dumpsys.lines() {
        if line.contains("mCurrentFocus=Window{")
            || line.contains("mFocusedApp=AppWindowToken{")
            || line.contains("mFocusedApp=ActivityRecord{")
            || line.contains("mCurrentFocus=")
            || line.contains("mFocusedApp=")
        {
            // Format example: mCurrentFocus=Window{8bb1e24 u0 com.android.settings/com.android.settings.Settings}
            if let Some(slash_idx) = line.find('/') {
                let prefix = &line[..slash_idx];
                // Find token preceding slash
                if let Some(last_space) = prefix.rfind(|c: char| c.is_whitespace() || c == '{') {
                    let candidate = &prefix[last_space + 1..];
                    let clean = candidate.trim_start_matches(|c: char| c.is_alphanumeric() && candidate.starts_with("u0"));
                    let clean = clean.trim();
                    let final_pkg = if let Some(stripped) = clean.strip_prefix("u0 ") {
                        stripped.trim()
                    } else {
                        clean
                    };
                    if is_valid_package_name(final_pkg) {
                        return final_pkg.to_string();
                    }
                }
            }
        }
    }
    "unknown".to_string()
}

/// Unescapes standard XML entities.
fn unescape_xml(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
}

/// Parses uiautomator XML dump into structured `UiNode` items.
pub fn parse_uiautomator_xml(xml: &str) -> Vec<UiNode> {
    let mut nodes = Vec::new();
    let mut cursor = 0;

    while let Some(start_node) = xml[cursor..].find("<node ") {
        let abs_start = cursor + start_node;
        let end_node = match xml[abs_start..].find('>') {
            Some(e) => abs_start + e,
            None => break,
        };

        let node_tag = &xml[abs_start..=end_node];
        cursor = end_node + 1;

        // Parse attributes
        let text = extract_attr(node_tag, "text");
        let resource_id = extract_attr(node_tag, "resource-id");
        let content_desc = extract_attr(node_tag, "content-desc");
        let class_name = extract_attr(node_tag, "class");
        let package = extract_attr(node_tag, "package");
        let clickable = extract_attr(node_tag, "clickable") == "true";
        let focusable = extract_attr(node_tag, "focusable") == "true";
        let scrollable = extract_attr(node_tag, "scrollable") == "true";
        let password = extract_attr(node_tag, "password") == "true";
        let bounds_str = extract_attr(node_tag, "bounds");

        let (bounds, center) = parse_bounds(&bounds_str);

        // Keep only actionable or informative nodes
        let has_text = !text.trim().is_empty();
        let has_desc = !content_desc.trim().is_empty();
        if clickable || scrollable || has_text || has_desc {
            let index = nodes.len();
            nodes.push(UiNode {
                index,
                text,
                resource_id,
                content_desc,
                class_name,
                package,
                clickable,
                focusable,
                scrollable,
                password,
                bounds,
                center,
            });
        }
    }

    nodes
}

/// Extracts a named attribute value from an XML tag string like `<node text="foo" ...>`.
fn extract_attr(tag: &str, attr_name: &str) -> String {
    let pattern = format!("{}=\"", attr_name);
    if let Some(pos) = tag.find(&pattern) {
        let val_start = pos + pattern.len();
        if let Some(val_end) = tag[val_start..].find('"') {
            return unescape_xml(&tag[val_start..val_start + val_end]);
        }
    }
    String::new()
}

/// Parses bounds format `[x1,y1][x2,y2]` and calculates center coordinate.
fn parse_bounds(bounds_str: &str) -> ((i32, i32, i32, i32), (i32, i32)) {
    // Expected: [0,0][1080,2400]
    let s = bounds_str.trim();
    if let Some(first_end) = s.find(']') {
        let first_box = &s[1..first_end]; // x1,y1
        if let Some(second_start) = s[first_end + 1..].find('[') {
            let second_abs = first_end + 1 + second_start;
            let second_box = &s[second_abs + 1..s.len().saturating_sub(1)]; // x2,y2

            let p1: Vec<i32> = first_box
                .split(',')
                .filter_map(|p| p.trim().parse().ok())
                .collect();
            let p2: Vec<i32> = second_box
                .split(',')
                .filter_map(|p| p.trim().parse().ok())
                .collect();

            if p1.len() == 2 && p2.len() == 2 {
                let bounds = (p1[0], p1[1], p2[0], p2[1]);
                let center = ((p1[0] + p2[0]) / 2, (p1[1] + p2[1]) / 2);
                return (bounds, center);
            }
        }
    }
    ((0, 0, 0, 0), (0, 0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permission_gateway::{AccessMode, PrivacyMode};

    #[test]
    fn test_device_list_parsing_usb_and_wifi() {
        let adb_out = r#"
List of devices attached
R58N123ABC             device usb:1-1 product:dm3qxxx model:SM_S918B device:dm3q transport_id:1
192.168.1.50:5555      device product:panther model:Pixel_7 device:panther transport_id:2
emulator-5554          offline transport_id:3
"#;
        let devices = parse_adb_devices_output(adb_out);
        assert_eq!(devices.len(), 3);

        assert_eq!(devices[0].serial, "R58N123ABC");
        assert_eq!(devices[0].state, "device");
        assert_eq!(devices[0].model, "SM_S918B");
        assert_eq!(devices[0].transport, "usb");

        assert_eq!(devices[1].serial, "192.168.1.50:5555");
        assert_eq!(devices[1].state, "device");
        assert_eq!(devices[1].model, "Pixel_7");
        assert_eq!(devices[1].transport, "wifi");

        assert_eq!(devices[2].serial, "emulator-5554");
        assert_eq!(devices[2].state, "offline");
        assert_eq!(devices[2].model, "Unknown");
    }

    #[test]
    fn test_uiautomator_xml_parsing_with_bounds_center_and_password() {
        let xml = r#"<?xml version='1.0' encoding='UTF-8' standalone='yes' ?>
<hierarchy rotation="0">
  <node index="0" text="" resource-id="" class="android.widget.FrameLayout" package="com.android.settings" content-desc="" checkable="false" checked="false" clickable="false" enabled="true" focusable="false" focused="false" scrollable="false" long-clickable="false" password="false" selected="false" bounds="[0,0][1080,2400]">
    <node index="0" text="WLAN &amp; Internet" resource-id="com.android.settings:id/title" class="android.widget.TextView" package="com.android.settings" content-desc="Netzwerk &lt;WLAN&gt;" checkable="false" checked="false" clickable="true" enabled="true" focusable="true" focused="false" scrollable="false" long-clickable="false" password="false" selected="false" bounds="[100,200][500,400]" />
    <node index="1" text="" resource-id="com.android.settings:id/pin" class="android.widget.EditText" package="com.android.settings" content-desc="PIN eingeben" checkable="false" checked="false" clickable="true" enabled="true" focusable="true" focused="false" scrollable="false" long-clickable="false" password="true" selected="false" bounds="[100,500][900,600]" />
  </node>
</hierarchy>"#;

        let nodes = parse_uiautomator_xml(xml);
        assert_eq!(nodes.len(), 2);

        // Node 0
        assert_eq!(nodes[0].index, 0);
        assert_eq!(nodes[0].text, "WLAN & Internet");
        assert_eq!(nodes[0].content_desc, "Netzwerk <WLAN>");
        assert_eq!(nodes[0].bounds, (100, 200, 500, 400));
        assert_eq!(nodes[0].center, (300, 300));
        assert!(!nodes[0].password);

        // Node 1
        assert_eq!(nodes[1].index, 1);
        assert_eq!(nodes[1].content_desc, "PIN eingeben");
        assert_eq!(nodes[1].bounds, (100, 500, 900, 600));
        assert_eq!(nodes[1].center, (500, 550));
        assert!(nodes[1].password);
    }

    #[test]
    fn test_foreground_package_parsing() {
        let dumpsys_a = r#"
  mCurrentFocus=Window{8bb1e24 u0 com.android.settings/com.android.settings.Settings}
  mFocusedApp=AppWindowToken{123 u0 com.android.settings/com.android.settings.Settings}
"#;
        assert_eq!(parse_foreground_package(dumpsys_a), "com.android.settings");

        let dumpsys_b = r#"
  mFocusedApp=ActivityRecord{a4f15d8 u0 com.google.android.youtube/com.google.android.apps.youtube.app.WatchWhileActivity}
"#;
        assert_eq!(parse_foreground_package(dumpsys_b), "com.google.android.youtube");
    }

    #[test]
    fn test_tap_node_mapping_uses_correct_center() {
        let runner = Arc::new(MockAdbRunner::new());
        let bridge = PhoneBridge::new(runner.clone());
        let gw = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

        let req = gw
            .request_approval(ActionKind::PhoneAct, "Tap", "Tippen", vec![], 0, 0)
            .unwrap();
        let token = gw.approve_request(req.request_id, None).unwrap();

        let obs = PhoneObservation {
            serial: "test-dev".into(),
            foreground_package: "com.android.settings".into(),
            ui_nodes: vec![UiNode {
                index: 0,
                text: "WLAN".into(),
                resource_id: "".into(),
                content_desc: "".into(),
                class_name: "".into(),
                package: "com.android.settings".into(),
                clickable: true,
                focusable: true,
                scrollable: false,
                password: false,
                bounds: (100, 200, 500, 400),
                center: (300, 300),
            }],
            screenshot_png_base64: None,
            secure_screen_suspected: false,
            captured_at: Utc::now(),
        };

        let res = bridge.execute(
            &gw,
            Some("test-dev"),
            &PhoneAction::TapNode { index: 0 },
            Some(&token),
            Some(&obs),
        );
        assert!(res.is_ok());

        let cmds = runner.recorded_commands();
        assert_eq!(cmds.len(), 1);
        assert_eq!(
            cmds[0],
            vec!["-s", "test-dev", "shell", "input", "tap", "300", "300"]
        );
    }

    #[test]
    fn test_type_text_via_adb_keyboard_broadcast_with_umlauts() {
        let runner = Arc::new(MockAdbRunner::new());
        // Mock keyboard IME present
        runner.set_response_for_substr(
            "ime list",
            Ok(b"com.android.adbkeyboard/.AdbIME\n".to_vec()),
        );

        let bridge = PhoneBridge::new(runner.clone());
        let gw = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

        let req = gw
            .request_approval(ActionKind::PhoneAct, "Type", "Text", vec![], 0, 0)
            .unwrap();
        let token = gw.approve_request(req.request_id, None).unwrap();

        let text = "Grüße vom AIDO Phone Agent! äöü";
        let res = bridge.execute(
            &gw,
            Some("test-dev"),
            &PhoneAction::TypeText {
                text: text.to_string(),
            },
            Some(&token),
            None,
        );
        assert!(res.is_ok());

        let cmds = runner.recorded_commands();
        let last_cmd = cmds.last().unwrap();
        assert_eq!(last_cmd[2], "shell");
        assert_eq!(last_cmd[3], "am");
        assert_eq!(last_cmd[4], "broadcast");
        assert_eq!(last_cmd[6], "ADB_INPUT_B64");
        assert_eq!(&last_cmd[9], &base64_encode(text.as_bytes()));
    }

    #[test]
    fn test_ascii_fallback_escaping_and_umlaut_rejection_without_adb_keyboard() {
        let runner = Arc::new(MockAdbRunner::new());
        // No adb keyboard
        runner.set_response_for_substr("ime list", Ok(b"com.google.android.inputmethod.latin\n".to_vec()));

        let bridge = PhoneBridge::new(runner.clone());
        let gw = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

        // 1. Umlauts rejected
        let req1 = gw
            .request_approval(ActionKind::PhoneAct, "Type", "Text", vec![], 0, 0)
            .unwrap();
        let token1 = gw.approve_request(req1.request_id, None).unwrap();
        let res1 = bridge.execute(
            &gw,
            Some("test-dev"),
            &PhoneAction::TypeText {
                text: "Schöne Grüße".to_string(),
            },
            Some(&token1),
            None,
        );
        assert!(matches!(res1, Err(PhoneError::CommandFailed(_))));

        // 2. ASCII text with spaces converted to %s
        let req2 = gw
            .request_approval(ActionKind::PhoneAct, "Type", "Text", vec![], 0, 0)
            .unwrap();
        let token2 = gw.approve_request(req2.request_id, None).unwrap();
        let res2 = bridge.execute(
            &gw,
            Some("test-dev"),
            &PhoneAction::TypeText {
                text: "Hello World".to_string(),
            },
            Some(&token2),
            None,
        );
        assert!(res2.is_ok());
        let cmds = runner.recorded_commands();
        let last_cmd = cmds.last().unwrap();
        assert_eq!(
            last_cmd,
            &vec!["-s", "test-dev", "shell", "input", "text", "Hello%sWorld"]
        );
    }

    #[test]
    fn test_injection_inputs_rejected() {
        let runner = Arc::new(MockAdbRunner::new());
        let bridge = PhoneBridge::new(runner);
        let gw = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

        // Dangerous phone number
        let req1 = gw
            .request_approval(ActionKind::PhoneSensitiveAct, "Call", "Call", vec![], 0, 0)
            .unwrap();
        let tok1 = gw.approve_request(req1.request_id, None).unwrap();
        let res1 = bridge.execute(
            &gw,
            None,
            &PhoneAction::StartCall {
                number: "1234; rm -rf /".into(),
            },
            Some(&tok1),
            None,
        );
        assert!(matches!(res1, Err(PhoneError::SafetyBlocked(_))));

        // Dangerous package name
        let req2 = gw
            .request_approval(ActionKind::PhoneAct, "Launch", "Launch", vec![], 0, 0)
            .unwrap();
        let tok2 = gw.approve_request(req2.request_id, None).unwrap();
        let res2 = bridge.execute(
            &gw,
            None,
            &PhoneAction::LaunchApp {
                package: "com.pkg && reboot".into(),
            },
            Some(&tok2),
            None,
        );
        assert!(matches!(res2, Err(PhoneError::SafetyBlocked(_))));

        // Dangerous URL
        let req3 = gw
            .request_approval(ActionKind::PhoneAct, "Url", "Url", vec![], 0, 0)
            .unwrap();
        let tok3 = gw.approve_request(req3.request_id, None).unwrap();
        let res3 = bridge.execute(
            &gw,
            None,
            &PhoneAction::OpenUrl {
                url: "javascript:alert(1)".into(),
            },
            Some(&tok3),
            None,
        );
        assert!(matches!(res3, Err(PhoneError::SafetyBlocked(_))));
    }

    #[test]
    fn test_banking_package_and_password_field_blocked() {
        let runner = Arc::new(MockAdbRunner::new());
        let bridge = PhoneBridge::new(runner);
        let gw = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

        // 1. Launching banking app blocked
        let req1 = gw
            .request_approval(ActionKind::PhoneAct, "Launch", "Launch", vec![], 0, 0)
            .unwrap();
        let tok1 = gw.approve_request(req1.request_id, None).unwrap();
        let res1 = bridge.execute(
            &gw,
            None,
            &PhoneAction::LaunchApp {
                package: "de.n26.android".into(),
            },
            Some(&tok1),
            None,
        );
        assert!(matches!(res1, Err(PhoneError::SafetyBlocked(_))));

        // 2. Action blocked when foreground package is PayPal
        let req2 = gw
            .request_approval(ActionKind::PhoneAct, "Tap", "Tap", vec![], 0, 0)
            .unwrap();
        let tok2 = gw.approve_request(req2.request_id, None).unwrap();
        let obs_banking = PhoneObservation {
            serial: "s1".into(),
            foreground_package: "com.paypal.android.p2pmobile".into(),
            ui_nodes: vec![],
            screenshot_png_base64: None,
            secure_screen_suspected: true,
            captured_at: Utc::now(),
        };
        let res2 = bridge.execute(
            &gw,
            None,
            &PhoneAction::Tap { x: 100, y: 100 },
            Some(&tok2),
            Some(&obs_banking),
        );
        assert!(matches!(res2, Err(PhoneError::SafetyBlocked(_))));

        // 3. Tapping password field blocked
        let req3 = gw
            .request_approval(ActionKind::PhoneAct, "Tap", "Tap", vec![], 0, 0)
            .unwrap();
        let tok3 = gw.approve_request(req3.request_id, None).unwrap();
        let obs_pwd = PhoneObservation {
            serial: "s1".into(),
            foreground_package: "com.android.settings".into(),
            ui_nodes: vec![UiNode {
                index: 0,
                text: "".into(),
                resource_id: "".into(),
                content_desc: "".into(),
                class_name: "".into(),
                package: "com.android.settings".into(),
                clickable: true,
                focusable: true,
                scrollable: false,
                password: true,
                bounds: (10, 10, 100, 50),
                center: (55, 30),
            }],
            screenshot_png_base64: None,
            secure_screen_suspected: true,
            captured_at: Utc::now(),
        };
        let res3 = bridge.execute(
            &gw,
            None,
            &PhoneAction::TapNode { index: 0 },
            Some(&tok3),
            Some(&obs_pwd),
        );
        assert!(matches!(res3, Err(PhoneError::SafetyBlocked(_))));
    }

    #[test]
    fn test_sms_requires_sensitive_token_and_maps_to_sendto_intent() {
        let runner = Arc::new(MockAdbRunner::new());
        let bridge = PhoneBridge::new(runner.clone());
        let gw = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

        // Regular PhoneAct token must be rejected for SendSms (requires PhoneSensitiveAct)
        let req_wrong = gw
            .request_approval(ActionKind::PhoneAct, "SMS", "SMS", vec![], 0, 0)
            .unwrap();
        let tok_wrong = gw.approve_request(req_wrong.request_id, None).unwrap();
        let res_wrong = bridge.execute(
            &gw,
            Some("dev1"),
            &PhoneAction::SendSms {
                number: "+491701234567".into(),
                body: "Hallo AIDO".into(),
            },
            Some(&tok_wrong),
            None,
        );
        assert!(matches!(res_wrong, Err(PhoneError::Gateway(_))));

        // Sensitive token works
        let req_ok = gw
            .request_approval(ActionKind::PhoneSensitiveAct, "SMS", "SMS", vec![], 0, 0)
            .unwrap();
        let tok_ok = gw.approve_request(req_ok.request_id, None).unwrap();
        let res_ok = bridge.execute(
            &gw,
            Some("dev1"),
            &PhoneAction::SendSms {
                number: "+491701234567".into(),
                body: "Hallo AIDO".into(),
            },
            Some(&tok_ok),
            None,
        );
        assert!(res_ok.is_ok());

        let cmds = runner.recorded_commands();
        let last_cmd = cmds.last().unwrap();
        assert_eq!(
            last_cmd,
            &vec![
                "-s",
                "dev1",
                "shell",
                "am",
                "start",
                "-a",
                "android.intent.action.SENDTO",
                "-d",
                "sms:+491701234567",
                "--es",
                "sms_body",
                "Hallo AIDO"
            ]
        );
    }

    #[test]
    fn test_readonly_mode_allows_observe_but_blocks_act() {
        let runner = Arc::new(MockAdbRunner::new());
        let bridge = PhoneBridge::new(runner);
        let gw = PermissionGateway::new(AccessMode::ReadOnly, PrivacyMode::PrivateOffline);

        // Observe works
        let obs_res = bridge.observe(&gw, None, false);
        assert!(obs_res.is_ok());

        // Mutating approval creation is blocked in ReadOnly
        let req_res = gw.request_approval(ActionKind::PhoneAct, "Tap", "Tap", vec![], 0, 0);
        assert!(matches!(req_res, Err(GatewayError::ReadOnlyBlocked(_))));
    }
}
