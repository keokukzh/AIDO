use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::ai_client::LocalAiClient;
use crate::permission_gateway::{ActionKind, ApprovalToken, PermissionGateway};
use crate::phone_bridge::{PhoneAction, PhoneBridge, PhoneError, PhoneKey, PhoneObservation};

/// Record of an executed agent step on the phone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneStepRecord {
    pub step: usize,
    pub action: PhoneAction,
    pub outcome_de: String,
    pub foreground_package: String,
}

/// Final report returned after attempting to execute a user goal on a phone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoneRunReport {
    pub goal: String,
    pub serial: Option<String>,
    pub steps: Vec<PhoneStepRecord>,
    pub completed: bool,
    pub handed_off: bool,
    pub summary_de: String,
}

/// Strategy deciding the next action given user goal, screen observation, and execution history.
pub trait PhonePlanner: Send + Sync {
    fn next_action(
        &self,
        goal: &str,
        obs: &PhoneObservation,
        history: &[PhoneStepRecord],
    ) -> PhoneAction;
}

/// Deterministic, fast rule-based planner for common intents in German and English.
#[derive(Debug, Default, Clone)]
pub struct RuleBasedPhonePlanner;

impl RuleBasedPhonePlanner {
    pub fn new() -> Self {
        Self
    }

    /// Resolves common natural language app names to canonical Android package IDs.
    pub fn map_app_name_to_package(name: &str) -> Option<&'static str> {
        let n = name.trim().to_lowercase();
        match n.as_str() {
            "einstellungen" | "settings" | "android settings" => Some("com.android.settings"),
            "whatsapp" => Some("com.whatsapp"),
            "chrome" | "browser" | "google chrome" => Some("com.android.chrome"),
            "kamera" | "camera" => Some("com.android.camera"),
            "youtube" => Some("com.google.android.youtube"),
            "telegram" => Some("org.telegram.messenger"),
            "spotify" => Some("com.spotify.music"),
            "maps" | "karten" | "google maps" => Some("com.google.android.apps.maps"),
            "telefon" | "phone" | "anrufe" => Some("com.google.android.dialer"),
            "nachrichten" | "messages" | "sms" => Some("com.google.android.apps.messaging"),
            "play store" | "playstore" | "google play" => Some("com.android.vending"),
            "gmail" | "mail" | "e-mail" => Some("com.google.android.gm"),
            "fotos" | "photos" | "galerie" | "gallery" => Some("com.google.android.apps.photos"),
            "rechner" | "calculator" => Some("com.google.android.calculator"),
            "uhr" | "clock" | "wecker" => Some("com.google.android.deskclock"),
            "kalender" | "calendar" => Some("com.google.android.calendar"),
            "dateien" | "files" => Some("com.google.android.apps.nbu.files"),
            _ => None,
        }
    }
}

impl PhonePlanner for RuleBasedPhonePlanner {
    fn next_action(
        &self,
        goal: &str,
        obs: &PhoneObservation,
        history: &[PhoneStepRecord],
    ) -> PhoneAction {
        let trimmed_goal = goal.trim();
        let lower = trimmed_goal.to_lowercase();

        // If history already has an action and it succeeded, check if goal is satisfied
        if !history.is_empty() {
            let last_step = &history[history.len() - 1];
            match &last_step.action {
                PhoneAction::LaunchApp { package }
                    if obs.foreground_package == *package || !package.is_empty() =>
                {
                    return PhoneAction::Done {
                        summary: format!("App '{}' erfolgreich geöffnet", package),
                    };
                }
                PhoneAction::StartCall { number } => {
                    return PhoneAction::Done {
                        summary: format!("Anruf an '{}' erfolgreich gestartet", number),
                    };
                }
                PhoneAction::SendSms { number, .. } => {
                    return PhoneAction::Done {
                        summary: format!("SMS an '{}' erfolgreich vorbereitet", number),
                    };
                }
                PhoneAction::OpenUrl { url } => {
                    return PhoneAction::Done {
                        summary: format!("Webadresse '{}' erfolgreich aufgerufen", url),
                    };
                }
                PhoneAction::TapNode { .. }
                | PhoneAction::Tap { .. }
                | PhoneAction::Key { .. }
                | PhoneAction::Swipe { .. }
                | PhoneAction::TypeText { .. } => {
                    return PhoneAction::Done {
                        summary: format!("Aktion für Ziel '{}' erfolgreich ausgeführt", trimmed_goal),
                    };
                }
                _ => {}
            }
        }

        // 1. Open URL: "öffne <url>", "open <url>", "http://...", "https://...", "www...."
        let is_url = lower.starts_with("http://")
            || lower.starts_with("https://")
            || lower.starts_with("öffne http://")
            || lower.starts_with("öffne https://")
            || lower.starts_with("open http://")
            || lower.starts_with("open https://")
            || lower.starts_with("öffne www.")
            || lower.starts_with("open www.")
            || lower.starts_with("www.");

        if is_url {
            let raw_url = if let Some(u) = trimmed_goal.strip_prefix("öffne ") {
                u.trim()
            } else if let Some(u) = trimmed_goal.strip_prefix("open ") {
                u.trim()
            } else {
                trimmed_goal
            };
            let url = if raw_url.starts_with("www.") {
                format!("https://{}", raw_url)
            } else {
                raw_url.to_string()
            };
            return PhoneAction::OpenUrl { url };
        }

        // 2. Swipes and Scrolling: "wische nach oben", "scroll down", etc.
        if lower == "wische nach oben" || lower == "swipe up" || lower == "nach unten scrollen" || lower == "scroll down" {
            return PhoneAction::Swipe { x1: 540, y1: 1600, x2: 540, y2: 600, duration_ms: 300 };
        }
        if lower == "wische nach unten" || lower == "swipe down" || lower == "nach oben scrollen" || lower == "scroll up" {
            return PhoneAction::Swipe { x1: 540, y1: 600, x2: 540, y2: 1600, duration_ms: 300 };
        }
        if lower == "wische nach links" || lower == "swipe left" {
            return PhoneAction::Swipe { x1: 800, y1: 1200, x2: 200, y2: 1200, duration_ms: 300 };
        }
        if lower == "wische nach rechts" || lower == "swipe right" {
            return PhoneAction::Swipe { x1: 200, y1: 1200, x2: 800, y2: 1200, duration_ms: 300 };
        }

        // 3. Text Typing: "schreibe <text>", "tippe text <text> ein", "type <text>"
        let type_prefixes = &[
            "schreibe text ", "schreibe ", "tippe text ", "type text ", "type ", "gebe ein ", "eingabe von "
        ];
        for prefix in type_prefixes {
            if let Some(text_to_type) = trimmed_goal.strip_prefix(prefix) {
                let clean_text = text_to_type.trim();
                if !clean_text.is_empty() {
                    return PhoneAction::TypeText { text: clean_text.to_string() };
                }
            }
        }

        // 4. Hardware keys: "zurück", "back", "home", "recents", "enter"
        if lower == "zurück" || lower == "zurueck" || lower == "back" {
            return PhoneAction::Key {
                key: PhoneKey::Back,
            };
        }
        if lower == "home" || lower == "startbildschirm" || lower == "hauptbildschirm" {
            return PhoneAction::Key {
                key: PhoneKey::Home,
            };
        }
        if lower == "recents" || lower == "apps" || lower == "app-übersicht" || lower == "übersicht" {
            return PhoneAction::Key {
                key: PhoneKey::Recents,
            };
        }
        if lower == "enter" || lower == "eingabe" || lower == "bestätigen" || lower == "bestaetigen" {
            return PhoneAction::Key {
                key: PhoneKey::Enter,
            };
        }

        // 5. App Launch: "öffne <app>", "starte <app>", "open <app>"
        let app_prefixes = &["öffne ", "oeffne ", "starte ", "open ", "launch "];
        for prefix in app_prefixes {
            if let Some(rest) = lower.strip_prefix(prefix) {
                let app_query = rest.trim();
                let package = if let Some(known_pkg) = Self::map_app_name_to_package(app_query) {
                    known_pkg.to_string()
                } else if app_query.contains('.') {
                    app_query.to_string()
                } else {
                    return PhoneAction::HandOffToUser {
                        reason: format!(
                            "App '{}' ist nicht bekannt. Bitte Paketnamen angeben.",
                            app_query
                        ),
                    };
                };
                return PhoneAction::LaunchApp { package };
            }
        }

        // 6. Phone calls: "rufe <nummer> an", "call <nummer>"
        if lower.starts_with("rufe ") && lower.ends_with(" an") {
            let number = &trimmed_goal[5..trimmed_goal.len() - 3].trim();
            return PhoneAction::StartCall {
                number: number.to_string(),
            };
        }
        if let Some(rest) = lower.strip_prefix("call ") {
            return PhoneAction::StartCall {
                number: rest.trim().to_string(),
            };
        }

        // 7. SMS sending: "sms an <nummer>: <text>", "text <nummer>: <text>"
        if let Some(rest) = lower.strip_prefix("sms an ") {
            if let Some(colon) = rest.find(':') {
                let number = rest[..colon].trim().to_string();
                let body = trimmed_goal[trimmed_goal.len() - (rest.len() - colon - 1)..].trim().to_string();
                return PhoneAction::SendSms { number, body };
            }
        }
        if let Some(rest) = lower.strip_prefix("sms to ") {
            if let Some(colon) = rest.find(':') {
                let number = rest[..colon].trim().to_string();
                let body = trimmed_goal[trimmed_goal.len() - (rest.len() - colon - 1)..].trim().to_string();
                return PhoneAction::SendSms { number, body };
            }
        }

        // 8. Tap on text: "tippe auf <text>", "klicke <text>", "tap on <text>"
        let tap_prefixes = &["tippe auf ", "tippe ", "klicke auf ", "klicke ", "tap on ", "tap ", "click "];
        for prefix in tap_prefixes {
            if let Some(target) = lower.strip_prefix(prefix) {
                let target_clean = target.trim();
                // Search in ui_nodes
                for node in &obs.ui_nodes {
                    let text_match = node.text.to_lowercase().contains(target_clean);
                    let desc_match = node.content_desc.to_lowercase().contains(target_clean);
                    if text_match || desc_match {
                        return PhoneAction::TapNode { index: node.index };
                    }
                }
                return PhoneAction::HandOffToUser {
                    reason: format!(
                        "Element '{}' nicht auf dem aktuellen Bildschirm gefunden.",
                        target_clean
                    ),
                };
            }
        }

        // Fallback: Hand off to human
        PhoneAction::HandOffToUser {
            reason: format!(
                "Ziel '{}' konnte vom regelbasierten Planer nicht eindeutig interpretiert werden.",
                trimmed_goal
            ),
        }
    }
}

/// LLM-powered planner for complex instructions and dynamic visual navigation.
pub struct LlmPhonePlanner {
    pub ai: LocalAiClient,
}

impl LlmPhonePlanner {
    pub fn new(ai: LocalAiClient) -> Self {
        Self { ai }
    }

    /// Builds a compact prompt string describing the screen and goal.
    pub fn build_prompt(
        goal: &str,
        obs: &PhoneObservation,
        history: &[PhoneStepRecord],
    ) -> String {
        let mut prompt = format!(
            "Du bist ein Phone-Agent. Dein Ziel: \"{}\"\nAktive Vordergrund-App: {}\n\nSichtbare UI-Elemente:\n",
            goal, obs.foreground_package
        );

        for node in obs.ui_nodes.iter().take(40) {
            let label = if !node.text.is_empty() {
                &node.text
            } else {
                &node.content_desc
            };
            let id = if !node.resource_id.is_empty() {
                &node.resource_id
            } else {
                "-"
            };
            prompt.push_str(&format!(
                "[{}] \"{}\" | id: {} | clickable: {}\n",
                node.index, label, id, node.clickable
            ));
        }

        if !history.is_empty() {
            prompt.push_str("\nBisherige Schritte:\n");
            for step in history {
                prompt.push_str(&format!(
                    "Schritt {}: {:?} -> {}\n",
                    step.step, step.action, step.outcome_de
                ));
            }
        }

        prompt.push_str(
            "\nAntworte mit GENAU EINEM JSON-Objekt für die nächste Aktion im folgenden Format (ohne Markdown):\n\
             {\"TapNode\": {\"index\": 0}} oder {\"LaunchApp\": {\"package\": \"...\"}} oder {\"TypeText\": {\"text\": \"...\"}} oder {\"Key\": {\"key\": \"Back\"}} oder {\"Done\": {\"summary\": \"...\"}} oder {\"HandOffToUser\": {\"reason\": \"...\"}}\n"
        );

        prompt
    }

    /// Robustly parses a JSON action from LLM output, extracting JSON substring if needed.
    pub fn parse_action(raw_output: &str) -> PhoneAction {
        let trimmed = raw_output.trim();

        // Direct parse
        if let Ok(act) = serde_json::from_str::<PhoneAction>(trimmed) {
            return act;
        }

        // Substring extraction
        if let Some(start) = trimmed.find('{') {
            if let Some(end) = trimmed.rfind('}') {
                if end > start {
                    let slice = &trimmed[start..=end];
                    if let Ok(act) = serde_json::from_str::<PhoneAction>(slice) {
                        return act;
                    }
                }
            }
        }

        PhoneAction::HandOffToUser {
            reason: format!(
                "Antwort des Sprachmodells konnte nicht als PhoneAction geparst werden: {}",
                trimmed
            ),
        }
    }
}

impl PhonePlanner for LlmPhonePlanner {
    fn next_action(
        &self,
        goal: &str,
        obs: &PhoneObservation,
        history: &[PhoneStepRecord],
    ) -> PhoneAction {
        let prompt = Self::build_prompt(goal, obs, history);

        // Try querying the local model via explain_health_and_propose_cleanup or direct endpoint
        match self.ai.explain_health_and_propose_cleanup(&prompt, None) {
            Ok(resp) => Self::parse_action(&resp.message),
            Err(e) => PhoneAction::HandOffToUser {
                reason: format!("Lokales KI-Modell nicht erreichbar: {}", e),
            },
        }
    }
}

/// Agent running the observe → plan → approve → act control loop on Android.
pub struct PhoneAgent {
    pub bridge: Arc<PhoneBridge>,
    pub planner: Arc<dyn PhonePlanner>,
    pub max_steps: usize,
}

impl PhoneAgent {
    pub fn new(bridge: Arc<PhoneBridge>, planner: Arc<dyn PhonePlanner>) -> Self {
        Self {
            bridge,
            planner,
            max_steps: 12,
        }
    }

    pub fn with_max_steps(mut self, max: usize) -> Self {
        self.max_steps = max;
        self
    }

    /// Runs the loop until the goal is achieved, handed off to the user, or step limit is reached.
    pub fn run_goal(
        &self,
        gateway: &PermissionGateway,
        serial: Option<&str>,
        goal: &str,
        approver: &dyn Fn(&PhoneAction) -> Option<ApprovalToken>,
    ) -> PhoneRunReport {
        let mut steps = Vec::new();

        for step_num in 1..=self.max_steps {
            // 1. Observe screen state
            let obs = match self.bridge.observe(gateway, serial, false) {
                Ok(o) => o,
                Err(e) => {
                    return PhoneRunReport {
                        goal: goal.to_string(),
                        serial: serial.map(|s| s.to_string()),
                        steps,
                        completed: false,
                        handed_off: true,
                        summary_de: format!("Fehler bei der Bildschirmbeobachtung: {}", e),
                    };
                }
            };

            // 2. Immediate security check: Hand off if banking/password screen is active
            if PhoneBridge::is_sensitive_package(&obs.foreground_package) {
                let reason = format!(
                    "Sicherheitsrelevante App '{}' im Vordergrund aktiv.",
                    obs.foreground_package
                );
                steps.push(PhoneStepRecord {
                    step: step_num,
                    action: PhoneAction::HandOffToUser {
                        reason: reason.clone(),
                    },
                    outcome_de: format!("{}. Steuerung automatisch an Nutzer übergeben.", reason),
                    foreground_package: obs.foreground_package.clone(),
                });
                return PhoneRunReport {
                    goal: goal.to_string(),
                    serial: serial.map(|s| s.to_string()),
                    steps,
                    completed: false,
                    handed_off: true,
                    summary_de: format!("{}. Steuerung an Nutzer übergeben.", reason),
                };
            }

            // 3. Plan next action
            let action = self.planner.next_action(goal, &obs, &steps);

            // 4. Handle terminal planner decisions
            match &action {
                PhoneAction::Done { summary } => {
                    steps.push(PhoneStepRecord {
                        step: step_num,
                        action: action.clone(),
                        outcome_de: summary.clone(),
                        foreground_package: obs.foreground_package,
                    });
                    return PhoneRunReport {
                        goal: goal.to_string(),
                        serial: serial.map(|s| s.to_string()),
                        steps,
                        completed: true,
                        handed_off: false,
                        summary_de: summary.clone(),
                    };
                }
                PhoneAction::HandOffToUser { reason } => {
                    steps.push(PhoneStepRecord {
                        step: step_num,
                        action: action.clone(),
                        outcome_de: format!("An Nutzer übergeben: {}", reason),
                        foreground_package: obs.foreground_package,
                    });
                    return PhoneRunReport {
                        goal: goal.to_string(),
                        serial: serial.map(|s| s.to_string()),
                        steps,
                        completed: false,
                        handed_off: true,
                        summary_de: format!("An Nutzer übergeben: {}", reason),
                    };
                }
                _ => {}
            }

            // 5. Formulate Permission Gateway Approval Request
            let (action_kind, human_title, human_exp) = match &action {
                PhoneAction::SendSms { number, body } => (
                    ActionKind::PhoneSensitiveAct,
                    "SMS auf Smartphone vorbereiten",
                    format!("SMS an '{}' vorbereiten mit Text: \"{}\"", number, body),
                ),
                PhoneAction::StartCall { number } => (
                    ActionKind::PhoneSensitiveAct,
                    "Telefonanruf starten",
                    format!("Anruf an Rufnummer '{}' initiieren", number),
                ),
                PhoneAction::LaunchApp { package } => (
                    ActionKind::PhoneAct,
                    "App auf Smartphone öffnen",
                    format!("Android-App '{}' im Vordergrund starten", package),
                ),
                PhoneAction::Tap { x, y } => (
                    ActionKind::PhoneAct,
                    "Auf Display tippen",
                    format!("Tippen auf Koordinate ({}, {})", x, y),
                ),
                PhoneAction::TapNode { index } => (
                    ActionKind::PhoneAct,
                    "Auf UI-Element tippen",
                    format!("Tippen auf UI-Element #{}", index),
                ),
                PhoneAction::Swipe {
                    x1,
                    y1,
                    x2,
                    y2,
                    duration_ms,
                } => (
                    ActionKind::PhoneAct,
                    "Wischgeste ausführen",
                    format!(
                        "Wischen von ({},{}) nach ({},{}) ({} ms)",
                        x1, y1, x2, y2, duration_ms
                    ),
                ),
                PhoneAction::TypeText { text } => (
                    ActionKind::PhoneAct,
                    "Text eingeben",
                    format!("Eingabe von \"{}\"", text),
                ),
                PhoneAction::Key { key } => (
                    ActionKind::PhoneAct,
                    "Navigationstaste drücken",
                    format!("Taste '{:?}' senden", key),
                ),
                PhoneAction::OpenUrl { url } => (
                    ActionKind::PhoneAct,
                    "Webadresse aufrufen",
                    format!("URL '{}' auf dem Smartphone öffnen", url),
                ),
                PhoneAction::Wait { ms } => (
                    ActionKind::PhoneObserve,
                    "Warten",
                    format!("{} ms warten", ms),
                ),
                _ => (
                    ActionKind::PhoneAct,
                    "Aktion auf Smartphone ausführen",
                    format!("{:?}", action),
                ),
            };

            // Non-mutating wait can run without token
            if let PhoneAction::Wait { .. } = &action {
                let outcome = self
                    .bridge
                    .execute(gateway, serial, &action, None, Some(&obs))
                    .unwrap_or_else(|e| format!("Fehler: {}", e));
                steps.push(PhoneStepRecord {
                    step: step_num,
                    action,
                    outcome_de: outcome,
                    foreground_package: obs.foreground_package,
                });
                continue;
            }

            // Create formal request in gateway
            if let Err(e) = gateway.request_approval(
                action_kind,
                human_title,
                human_exp,
                vec![format!("{:?}", action)],
                0,
                0,
            ) {
                steps.push(PhoneStepRecord {
                    step: step_num,
                    action: action.clone(),
                    outcome_de: format!("Freigabe-Erstellung blockiert: {}", e),
                    foreground_package: obs.foreground_package,
                });
                return PhoneRunReport {
                    goal: goal.to_string(),
                    serial: serial.map(|s| s.to_string()),
                    steps,
                    completed: false,
                    handed_off: true,
                    summary_de: format!("Aktion durch Sicherheitsrichtlinie gesperrt: {}", e),
                };
            }

            // Ask user-defined approver callback for approval token
            let token = approver(&action);
            if token.is_none() {
                steps.push(PhoneStepRecord {
                    step: step_num,
                    action,
                    outcome_de: "Nutzer hat Freigabe verweigert. Vorgang abgebrochen.".into(),
                    foreground_package: obs.foreground_package,
                });
                return PhoneRunReport {
                    goal: goal.to_string(),
                    serial: serial.map(|s| s.to_string()),
                    steps,
                    completed: false,
                    handed_off: true,
                    summary_de: "Nutzer hat die erforderliche Freigabe verweigert.".into(),
                };
            }

            // Execute on phone bridge
            match self
                .bridge
                .execute(gateway, serial, &action, token.as_ref(), Some(&obs))
            {
                Ok(outcome) => {
                    steps.push(PhoneStepRecord {
                        step: step_num,
                        action,
                        outcome_de: outcome,
                        foreground_package: obs.foreground_package,
                    });
                }
                Err(PhoneError::SafetyBlocked(reason)) => {
                    steps.push(PhoneStepRecord {
                        step: step_num,
                        action,
                        outcome_de: format!(
                            "Sicherheitssperre: {}. Automatisch an Nutzer übergeben.",
                            reason
                        ),
                        foreground_package: obs.foreground_package,
                    });
                    return PhoneRunReport {
                        goal: goal.to_string(),
                        serial: serial.map(|s| s.to_string()),
                        steps,
                        completed: false,
                        handed_off: true,
                        summary_de: format!("Sicherheitssperre: {}. An Nutzer übergeben.", reason),
                    };
                }
                Err(err) => {
                    steps.push(PhoneStepRecord {
                        step: step_num,
                        action,
                        outcome_de: format!("Aktionsfehler: {}", err),
                        foreground_package: obs.foreground_package,
                    });
                    return PhoneRunReport {
                        goal: goal.to_string(),
                        serial: serial.map(|s| s.to_string()),
                        steps,
                        completed: false,
                        handed_off: true,
                        summary_de: format!("Aktionsfehler: {}", err),
                    };
                }
            }
        }

        PhoneRunReport {
            goal: goal.to_string(),
            serial: serial.map(|s| s.to_string()),
            steps,
            completed: false,
            handed_off: true,
            summary_de: format!(
                "Maximales Schrittlimit ({}) erreicht, ohne dass das Ziel als abgeschlossen markiert wurde.",
                self.max_steps
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::permission_gateway::{AccessMode, PrivacyMode};
    use crate::phone_bridge::{MockAdbRunner, UiNode};

    fn mock_obs(pkg: &str) -> PhoneObservation {
        PhoneObservation {
            serial: "test-dev".into(),
            foreground_package: pkg.into(),
            ui_nodes: vec![
                UiNode {
                    index: 0,
                    text: "WLAN".into(),
                    resource_id: "com.android.settings:id/title".into(),
                    content_desc: "".into(),
                    class_name: "android.widget.TextView".into(),
                    package: pkg.into(),
                    clickable: true,
                    focusable: true,
                    scrollable: false,
                    password: false,
                    bounds: (100, 200, 500, 400),
                    center: (300, 300),
                },
                UiNode {
                    index: 1,
                    text: "Bluetooth".into(),
                    resource_id: "com.android.settings:id/bluetooth".into(),
                    content_desc: "".into(),
                    class_name: "android.widget.TextView".into(),
                    package: pkg.into(),
                    clickable: true,
                    focusable: true,
                    scrollable: false,
                    password: false,
                    bounds: (100, 420, 500, 620),
                    center: (300, 520),
                },
            ],
            screenshot_png_base64: None,
            secure_screen_suspected: false,
            captured_at: Utc::now(),
        }
    }

    #[test]
    fn test_rule_planner_intents() {
        let planner = RuleBasedPhonePlanner::new();
        let obs = mock_obs("com.android.launcher");

        // 1. Launch Settings
        let a1 = planner.next_action("Öffne Einstellungen", &obs, &[]);
        assert_eq!(
            a1,
            PhoneAction::LaunchApp {
                package: "com.android.settings".into()
            }
        );

        // 2. Launch WhatsApp
        let a2 = planner.next_action("open WhatsApp", &obs, &[]);
        assert_eq!(
            a2,
            PhoneAction::LaunchApp {
                package: "com.whatsapp".into()
            }
        );

        // 3. Navigation keys
        assert_eq!(
            planner.next_action("zurück", &obs, &[]),
            PhoneAction::Key {
                key: PhoneKey::Back
            }
        );
        assert_eq!(
            planner.next_action("home", &obs, &[]),
            PhoneAction::Key {
                key: PhoneKey::Home
            }
        );

        // 4. Calls and SMS
        assert_eq!(
            planner.next_action("rufe +49170123456 an", &obs, &[]),
            PhoneAction::StartCall {
                number: "+49170123456".into()
            }
        );
        assert_eq!(
            planner.next_action("sms an 0170123: Hallo AIDO", &obs, &[]),
            PhoneAction::SendSms {
                number: "0170123".into(),
                body: "Hallo AIDO".into()
            }
        );

        // 5. Tap on UI element
        assert_eq!(
            planner.next_action("tippe auf WLAN", &obs, &[]),
            PhoneAction::TapNode { index: 0 }
        );

        // 6. Element not found -> HandOff
        let not_found = planner.next_action("tippe auf Flugmodus", &obs, &[]);
        assert!(matches!(not_found, PhoneAction::HandOffToUser { .. }));

        // 7. Second step after app launch -> Done
        let step_record = PhoneStepRecord {
            step: 1,
            action: PhoneAction::LaunchApp {
                package: "com.android.settings".into(),
            },
            outcome_de: "App gestartet".into(),
            foreground_package: "com.android.settings".into(),
        };
        let obs_in_settings = mock_obs("com.android.settings");
        let a_done = planner.next_action("Öffne Einstellungen", &obs_in_settings, &[step_record]);
        assert!(matches!(a_done, PhoneAction::Done { .. }));

        // 8. Open URL (prioritized before app launch)
        assert_eq!(
            planner.next_action("öffne https://github.com/keokukzh/AIDO", &obs, &[]),
            PhoneAction::OpenUrl {
                url: "https://github.com/keokukzh/AIDO".into()
            }
        );

        // 9. Swiping and scrolling
        assert_eq!(
            planner.next_action("wische nach oben", &obs, &[]),
            PhoneAction::Swipe {
                x1: 540,
                y1: 1600,
                x2: 540,
                y2: 600,
                duration_ms: 300
            }
        );

        // 10. Typing text
        assert_eq!(
            planner.next_action("schreibe Hallo Welt", &obs, &[]),
            PhoneAction::TypeText {
                text: "Hallo Welt".into()
            }
        );
    }

    #[test]
    fn test_phone_agent_runs_open_settings_to_done() {
        let runner = Arc::new(MockAdbRunner::new());
        // Mock dumpsys returning launcher first, then settings
        runner.set_response_for_substr(
            "dumpsys window",
            Ok(b"mCurrentFocus=Window{123 u0 com.android.settings/com.android.settings.Settings}\n".to_vec()),
        );

        let bridge = Arc::new(PhoneBridge::new(runner));
        let planner = Arc::new(RuleBasedPhonePlanner::new());
        let agent = PhoneAgent::new(bridge, planner);

        let gw = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

        let report = agent.run_goal(
            &gw,
            Some("test-dev"),
            "Öffne Einstellungen",
            &|act| {
                // Auto-approve mutating action
                let req = gw
                    .request_approval(
                        ActionKind::PhoneAct,
                        "Auto",
                        "Auto",
                        vec![format!("{:?}", act)],
                        0,
                        0,
                    )
                    .ok()?;
                gw.approve_request(req.request_id, None).ok()
            },
        );

        assert!(report.completed);
        assert!(!report.handed_off);
        assert!(report.steps.len() >= 2); // 1: LaunchApp, 2: Done
    }

    #[test]
    fn test_declined_approval_stops_agent() {
        let runner = Arc::new(MockAdbRunner::new());
        let bridge = Arc::new(PhoneBridge::new(runner));
        let planner = Arc::new(RuleBasedPhonePlanner::new());
        let agent = PhoneAgent::new(bridge, planner);

        let gw = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

        let report = agent.run_goal(
            &gw,
            Some("test-dev"),
            "Öffne Einstellungen",
            &|_act| None, // User declines!
        );

        assert!(!report.completed);
        assert!(report.handed_off);
        assert_eq!(report.steps.len(), 1);
        assert!(report.summary_de.contains("verweigert"));
    }

    #[test]
    fn test_banking_foreground_hands_off_immediately() {
        let runner = Arc::new(MockAdbRunner::new());
        runner.set_response_for_substr(
            "dumpsys window",
            Ok(b"mCurrentFocus=Window{123 u0 de.n26.android/de.n26.MainActivity}\n".to_vec()),
        );

        let bridge = Arc::new(PhoneBridge::new(runner));
        let planner = Arc::new(RuleBasedPhonePlanner::new());
        let agent = PhoneAgent::new(bridge, planner);

        let gw = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

        let report = agent.run_goal(
            &gw,
            Some("test-dev"),
            "tippe auf WLAN",
            &|_act| None,
        );

        assert!(!report.completed);
        assert!(report.handed_off);
        assert!(report.summary_de.contains("Sicherheitsrelevante App"));
    }

    #[test]
    fn test_max_steps_enforced() {
        // Planner that never yields Done
        struct EndlessPlanner;
        impl PhonePlanner for EndlessPlanner {
            fn next_action(&self, _goal: &str, _obs: &PhoneObservation, _history: &[PhoneStepRecord]) -> PhoneAction {
                PhoneAction::Wait { ms: 1 }
            }
        }

        let runner = Arc::new(MockAdbRunner::new());
        let bridge = Arc::new(PhoneBridge::new(runner));
        let agent = PhoneAgent::new(bridge, Arc::new(EndlessPlanner)).with_max_steps(3);

        let gw = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

        let report = agent.run_goal(&gw, Some("test-dev"), "Unendliches Warten", &|_| None);

        assert!(!report.completed);
        assert!(report.handed_off);
        assert_eq!(report.steps.len(), 3);
        assert!(report.summary_de.contains("Schrittlimit"));
    }

    #[test]
    fn test_llm_planner_parses_json_and_falls_back_on_garbage() {
        // 1. Valid JSON action
        let valid_json = r#"{"TapNode": {"index": 5}}"#;
        let act1 = LlmPhonePlanner::parse_action(valid_json);
        assert_eq!(act1, PhoneAction::TapNode { index: 5 });

        // 2. Embedded JSON in markdown / conversational text
        let wrapped_json = "Sicher! Hier ist die nächste Aktion:\n```json\n{\"Key\": {\"key\": \"Home\"}}\n```\nViel Erfolg!";
        let act2 = LlmPhonePlanner::parse_action(wrapped_json);
        assert_eq!(act2, PhoneAction::Key { key: PhoneKey::Home });

        // 3. Garbage text
        let garbage = "Entschuldigung, ich weiß nicht wie ich das machen soll.";
        let act3 = LlmPhonePlanner::parse_action(garbage);
        assert!(matches!(act3, PhoneAction::HandOffToUser { .. }));
    }
}
