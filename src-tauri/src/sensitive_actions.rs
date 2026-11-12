use crate::memory::MemoryEngine;
use crate::permission_gateway::{ActionKind, ApprovalToken, PermissionGateway};
use crate::runtime_supervisor::RuntimeSupervisor;
use crate::ui_automation::UiAutomationEngine;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use uuid::Uuid;

const DEFAULT_LIFETIME: Duration = Duration::from_secs(300);

#[derive(Debug, Clone, Serialize)]
pub struct ProposalChoice {
    pub id: String,
    pub label: String,
    pub is_default: bool,
}

/// Preview returned to the existing decision-card UI. Execution is a second step.
#[derive(Debug, Clone, Serialize)]
pub struct UserProposal {
    pub phase: &'static str,
    pub id: Uuid,
    pub title: String,
    pub human_explanation: String,
    pub preview_de: String,
    pub blast_summary_de: String,
    pub blast_radius: crate::permission_gateway::BlastRadius,
    pub risk_level: String,
    pub action_payload_json: String,
    pub options: Vec<ProposalChoice>,
    pub created_at: String,
    pub state: String,
    pub execute_command: String,
    pub execute_args: Value,
}

#[derive(Debug, Clone)]
pub struct PreparedAction {
    pub id: Uuid,
    pub action: ActionKind,
    pub targets: Vec<String>,
    title: String,
    explanation: String,
    created: Instant,
    created_at: String,
    execute_args: Value,
}

impl PreparedAction {
    /// Launch target stored at proposal time. Later command arguments are ignored.
    pub fn approved_launch_target(&self) -> Result<(String, String), String> {
        if self.action != ActionKind::LaunchProject {
            return Err("Diese Vorschau ist kein Projektstart.".into());
        }
        let name = self
            .targets
            .first()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .ok_or("Die freigegebene Vorschau enthält kein Projekt.")?;
        let mode = self
            .targets
            .get(1)
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .unwrap_or("antigravity");
        Ok((name.to_string(), mode.to_string()))
    }
}

struct ProposalDraft<'a> {
    gateway: &'a PermissionGateway,
    action: ActionKind,
    title: String,
    explanation: String,
    targets: Vec<String>,
    execute_args: Value,
    freed_bytes: u64,
    freed_vram: u64,
}

#[derive(Debug)]
pub struct SensitiveActionQueue {
    pending: Mutex<HashMap<Uuid, PreparedAction>>,
    lifetime: Duration,
}

impl Default for SensitiveActionQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl SensitiveActionQueue {
    pub fn new() -> Self {
        Self::with_lifetime(DEFAULT_LIFETIME)
    }

    pub fn with_lifetime(lifetime: Duration) -> Self {
        Self {
            pending: Mutex::new(HashMap::new()),
            lifetime,
        }
    }

    pub fn discard(&self, request_id: Uuid) {
        self.pending
            .lock()
            .expect("Freigabe-Queue")
            .remove(&request_id);
    }

    pub fn list_proposals(&self) -> Vec<UserProposal> {
        let mut pending = self.pending.lock().expect("Freigabe-Queue");
        let lifetime = self.lifetime;
        pending.retain(|_, item| item.created.elapsed() < lifetime);
        let mut views: Vec<_> = pending.values().map(view_of).collect();
        drop(pending);
        views.sort_by(|left, right| left.created_at.cmp(&right.created_at));
        views
    }

    pub fn propose_unload(&self, gateway: &PermissionGateway) -> Result<UserProposal, String> {
        let action = ActionKind::UnloadIdleModelSlot;
        let explanation = action.preview_de().to_string();
        self.insert(ProposalDraft {
            gateway,
            action,
            title: "Modell aus dem Grafikspeicher entladen".into(),
            explanation,
            targets: vec!["127.0.0.1".into()],
            execute_args: json!({}),
            freed_bytes: 0,
            freed_vram: 16_000,
        })
    }

    pub fn propose_launch(
        &self,
        gateway: &PermissionGateway,
        project_name: &str,
        mode: &str,
    ) -> Result<UserProposal, String> {
        let name = project_name.trim();
        if name.is_empty() {
            return Err("Bitte nenne das Projekt, das AIDO starten soll.".into());
        }
        if name.chars().count() > 240 {
            return Err("Der Projektname ist zu lang.".into());
        }
        let mode = {
            let trimmed = mode.trim();
            if trimmed.is_empty() {
                "antigravity"
            } else {
                trimmed
            }
        };
        if mode.chars().count() > 64 {
            return Err("Der Startmodus ist zu lang.".into());
        }
        let action = ActionKind::LaunchProject;
        let title = format!("Projekt «{name}» starten");
        let explanation = format!(
            "{} Projekt: «{name}». Programm: «{mode}».",
            action.preview_de()
        );
        self.insert(ProposalDraft {
            gateway,
            action,
            title,
            explanation,
            targets: vec![name.to_string(), mode.to_string()],
            execute_args: json!({ "projectName": name, "mode": mode }),
            freed_bytes: 0,
            freed_vram: 0,
        })
    }

    pub fn propose_capture(&self, gateway: &PermissionGateway) -> Result<UserProposal, String> {
        let action = ActionKind::CaptureDesktopScreen;
        let explanation = action.preview_de().to_string();
        self.insert(ProposalDraft {
            gateway,
            action,
            title: "Bildschirm aufnehmen".into(),
            explanation,
            targets: vec!["Bildschirm 0".into()],
            execute_args: json!({}),
            freed_bytes: 0,
            freed_vram: 0,
        })
    }

    /// Runs `effect` only when a real, unexpired preview matches a still-valid token.
    ///
    /// The token is not consumed here. The effect must consume it so a replay cannot
    /// enter the effect a second time. A missing or forged token never calls `effect`.
    pub fn run_after_peek<T>(
        &self,
        gateway: &PermissionGateway,
        memory: &MemoryEngine,
        token: Option<&ApprovalToken>,
        expected: ActionKind,
        audit_action: &str,
        effect: impl FnOnce(&PreparedAction, &ApprovalToken) -> Result<T, String>,
    ) -> Result<T, String> {
        let outcome = (|| {
            let token = token.ok_or_else(|| {
                "Diese Aktion braucht eine einmalige Freigabe. Es liegt kein Token vor.".to_string()
            })?;
            let prepared = self.prepared_for(token, &expected)?;
            gateway
                .peek_approval_token(token, &expected)
                .map_err(|error| error.to_string())?;
            let result = effect(&prepared, token);
            if result.is_ok() {
                self.discard(prepared.id);
            }
            result
        })();
        write_audit(memory, gateway, audit_action, &expected, &outcome);
        outcome
    }

    pub fn execute_unload(
        &self,
        gateway: &PermissionGateway,
        memory: &MemoryEngine,
        token: Option<&ApprovalToken>,
    ) -> Result<String, String> {
        self.run_after_peek(
            gateway,
            memory,
            token,
            ActionKind::UnloadIdleModelSlot,
            "UnloadModelVram",
            |_, approved| {
                RuntimeSupervisor::new()
                    .force_unload_model(gateway, approved)
                    .map_err(|error| error.to_string())
            },
        )
    }

    pub fn execute_capture<F>(
        &self,
        gateway: &PermissionGateway,
        memory: &MemoryEngine,
        engine: &UiAutomationEngine,
        token: Option<&ApprovalToken>,
        runner: F,
    ) -> Result<String, String>
    where
        F: FnOnce(&Path) -> Result<String, String>,
    {
        self.run_after_peek(
            gateway,
            memory,
            token,
            ActionKind::CaptureDesktopScreen,
            "CaptureDesktopScreen",
            |_, approved| {
                let driver = engine.resolve_driver().map_err(|error| error.to_string())?;
                gateway
                    .consume_approval_token(approved, &ActionKind::CaptureDesktopScreen)
                    .map_err(|error| error.to_string())?;
                runner(&driver)
            },
        )
    }

    fn insert(&self, draft: ProposalDraft) -> Result<UserProposal, String> {
        let ProposalDraft {
            gateway,
            action,
            title,
            explanation,
            targets,
            execute_args,
            freed_bytes,
            freed_vram,
        } = draft;
        let request = gateway
            .request_approval(
                action.clone(),
                title.clone(),
                explanation.clone(),
                targets.clone(),
                freed_bytes,
                freed_vram,
            )
            .map_err(|error| error.to_string())?;
        let prepared = PreparedAction {
            id: request.request_id,
            action,
            targets,
            title,
            explanation,
            created: Instant::now(),
            created_at: request.created_at.to_rfc3339(),
            execute_args,
        };
        let view = view_of(&prepared);
        self.pending
            .lock()
            .expect("Freigabe-Queue")
            .insert(prepared.id, prepared);
        Ok(view)
    }

    fn prepared_for(
        &self,
        token: &ApprovalToken,
        expected: &ActionKind,
    ) -> Result<PreparedAction, String> {
        if &token.action != expected {
            return Err("Diese Freigabe gehört zu einer anderen Aktion.".into());
        }
        let (item, expired) = {
            let pending = self.pending.lock().expect("Freigabe-Queue");
            match pending.get(&token.request_id) {
                Some(item) => (item.clone(), item.created.elapsed() >= self.lifetime),
                None => {
                    return Err("Zu dieser Freigabe gibt es keine offene Vorschau.".into());
                }
            }
        };
        if &item.action != expected {
            return Err("Die Vorschau passt nicht zur freigegebenen Aktion.".into());
        }
        if expired {
            self.discard(token.request_id);
            return Err("Die Freigabe ist abgelaufen. Bitte erstelle eine neue Vorschau.".into());
        }
        Ok(item)
    }
}

fn view_of(prepared: &PreparedAction) -> UserProposal {
    let blast = prepared.action.blast_radius();
    UserProposal {
        phase: "proposal",
        id: prepared.id,
        title: prepared.title.clone(),
        human_explanation: prepared.explanation.clone(),
        preview_de: prepared.action.preview_de().to_string(),
        blast_summary_de: blast.summary_de.clone(),
        blast_radius: blast,
        risk_level: risk_level(&prepared.action).to_string(),
        action_payload_json: "{}".into(),
        options: vec![
            ProposalChoice {
                id: "allow".into(),
                label: "Einmal erlauben".into(),
                is_default: true,
            },
            ProposalChoice {
                id: "cancel".into(),
                label: "Abbrechen".into(),
                is_default: false,
            },
        ],
        created_at: prepared.created_at.clone(),
        state: "PENDING".into(),
        execute_command: command_name(&prepared.action).to_string(),
        execute_args: prepared.execute_args.clone(),
    }
}

fn command_name(action: &ActionKind) -> &'static str {
    match action {
        ActionKind::UnloadIdleModelSlot => "unload_model_vram",
        ActionKind::LaunchProject => "launch_project_orchestrated",
        ActionKind::CaptureDesktopScreen => "capture_desktop_screen",
        _ => "",
    }
}

fn risk_level(action: &ActionKind) -> &'static str {
    match action {
        ActionKind::CaptureDesktopScreen => "RED_SYSTEM",
        ActionKind::UnloadIdleModelSlot | ActionKind::LaunchProject => "YELLOW_PROJECT",
        _ => "YELLOW_PROJECT",
    }
}

fn write_audit<T>(
    memory: &MemoryEngine,
    gateway: &PermissionGateway,
    audit_action: &str,
    kind: &ActionKind,
    outcome: &Result<T, String>,
) {
    let (approved, summary) = match outcome {
        Ok(_) => (
            true,
            format!("Freigegebene Aktion ausgeführt. {}", kind.preview_de()),
        ),
        Err(error) => (false, format!("Aktion nicht ausgeführt: {error}")),
    };
    let _ = memory.record_audit(
        audit_action,
        &format!("{:?}", gateway.access_mode()),
        &format!("{:?}", gateway.privacy_mode()),
        approved,
        &summary,
    );
}
