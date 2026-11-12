use crate::docker_sandbox_bridge::{DockerEnvironmentAudit, DockerSandboxBridge, DockerSandboxTemplate};
use crate::permission_gateway::{ActionKind, ApprovalRequest, ApprovalToken, GatewayError, PermissionGateway};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DockerError {
    #[error("Docker CLI wurde auf diesem System nicht gefunden")]
    CliNotFound,
    #[error("Docker Daemon läuft nicht oder ist nicht erreichbar: {0}")]
    DaemonNotRunning(String),
    #[error("Befehlsausführung fehlgeschlagen: {0}")]
    ExecutionFailed(String),
    #[error("Fehler beim Parsen der Container-Informationen: {0}")]
    ParseFailed(String),
    #[error("Ungültiger Container- oder Sandbox-Name: {0}")]
    InvalidTarget(String),
    #[error("Berechtigungsfehler: {0}")]
    Gateway(#[from] GatewayError),
    #[error("IO-Fehler: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DockerActionType {
    Start,
    Stop,
    Restart,
    PruneUnused,
}

impl DockerActionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DockerActionType::Start => "start",
            DockerActionType::Stop => "stop",
            DockerActionType::Restart => "restart",
            DockerActionType::PruneUnused => "system prune -f",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerContainerSummary {
    pub id: String,
    pub names: String,
    pub image: String,
    pub state: String,
    pub status: String,
    pub ports: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerEnvironmentStatus {
    pub cli_available: bool,
    pub cli_version: Option<String>,
    pub daemon_running: bool,
    pub audit: DockerEnvironmentAudit,
    pub container_count: usize,
    pub running_count: usize,
    pub human_message_de: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerActionResult {
    pub action: DockerActionType,
    pub target: String,
    pub exit_code: i32,
    pub output_summary: String,
}

#[derive(Debug, Clone, Default)]
pub struct DockerController {
    bridge: DockerSandboxBridge,
}

impl DockerController {
    pub fn new() -> Self {
        Self {
            bridge: DockerSandboxBridge::new(),
        }
    }

    /// Discovers Docker CLI binary
    pub fn find_docker_cli() -> Option<PathBuf> {
        if let Ok(output) = Command::new("docker").arg("--version").output() {
            if output.status.success() {
                return Some(PathBuf::from("docker"));
            }
        }
        let pf = std::env::var_os("ProgramFiles").map(PathBuf::from)?;
        let candidate = pf.join("Docker\\Docker\\resources\\bin\\docker.exe");
        if candidate.is_file() {
            Some(candidate)
        } else {
            None
        }
    }

    /// Read-only environment and daemon status check
    pub fn check_status(&self, gateway: &PermissionGateway) -> Result<DockerEnvironmentStatus, DockerError> {
        gateway.check_readonly_allowed(&ActionKind::SystemHealthCheck)?;

        let audit = self.bridge.audit_wsl2_and_sandboxes();
        let cli_path = Self::find_docker_cli();

        if cli_path.is_none() {
            return Ok(DockerEnvironmentStatus {
                cli_available: false,
                cli_version: None,
                daemon_running: false,
                audit,
                container_count: 0,
                running_count: 0,
                human_message_de: "Docker Desktop ist auf diesem System nicht installiert. Für Normalnutzer ist Docker rein optional.".to_string(),
            });
        }

        let cli = cli_path.unwrap();
        let version_str = Command::new(&cli)
            .arg("--version")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

        // Probe daemon connection
        let info_out = Command::new(&cli).arg("info").output();
        let daemon_running = match &info_out {
            Ok(o) => o.status.success(),
            Err(_) => false,
        };

        if !daemon_running {
            return Ok(DockerEnvironmentStatus {
                cli_available: true,
                cli_version: version_str,
                daemon_running: false,
                audit,
                container_count: 0,
                running_count: 0,
                human_message_de: "Docker CLI ist installiert, aber der Docker Desktop Hintergrunddienst läuft nicht.".to_string(),
            });
        }

        let containers = self.list_containers_internal(&cli).unwrap_or_default();
        let running_count = containers.iter().filter(|c| c.state.eq_ignore_ascii_case("running")).count();

        Ok(DockerEnvironmentStatus {
            cli_available: true,
            cli_version: version_str,
            daemon_running: true,
            container_count: containers.len(),
            running_count,
            audit,
            human_message_de: format!(
                "Docker Desktop läuft aktiv. {} Container gefunden ({} aktiv).",
                containers.len(),
                running_count
            ),
        })
    }

    /// Read-only container listing
    pub fn list_containers(&self, gateway: &PermissionGateway) -> Result<Vec<DockerContainerSummary>, DockerError> {
        gateway.check_readonly_allowed(&ActionKind::DiscoverApps)?;
        let cli = Self::find_docker_cli().ok_or(DockerError::CliNotFound)?;
        self.list_containers_internal(&cli)
    }

    fn list_containers_internal(&self, cli: &std::path::Path) -> Result<Vec<DockerContainerSummary>, DockerError> {
        let output = Command::new(cli)
            .args(["ps", "-a", "--format", "{{json .}}"])
            .output()
            .map_err(|e| DockerError::ExecutionFailed(e.to_string()))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(DockerError::DaemonNotRunning(err.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut list = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            #[derive(Deserialize)]
            #[serde(rename_all = "PascalCase")]
            struct RawDockerPs {
                #[serde(alias = "ID")]
                id: Option<String>,
                names: Option<String>,
                image: Option<String>,
                state: Option<String>,
                status: Option<String>,
                ports: Option<String>,
            }

            if let Ok(raw) = serde_json::from_str::<RawDockerPs>(line) {
                list.push(DockerContainerSummary {
                    id: raw.id.unwrap_or_default(),
                    names: raw.names.unwrap_or_default(),
                    image: raw.image.unwrap_or_default(),
                    state: raw.state.unwrap_or_default(),
                    status: raw.status.unwrap_or_default(),
                    ports: raw.ports.unwrap_or_default(),
                });
            }
        }

        Ok(list)
    }

    /// Read-only fetch of container logs
    pub fn get_logs(
        &self,
        target: &str,
        tail_lines: usize,
        gateway: &PermissionGateway,
    ) -> Result<String, DockerError> {
        gateway.check_readonly_allowed(&ActionKind::DiscoverApps)?;
        Self::validate_target_name(target)?;

        let cli = Self::find_docker_cli().ok_or(DockerError::CliNotFound)?;
        let output = Command::new(&cli)
            .args(["logs", "--tail", &tail_lines.to_string(), target])
            .output()
            .map_err(|e| DockerError::ExecutionFailed(e.to_string()))?;

        let text = if output.status.success() {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            String::from_utf8_lossy(&output.stderr).to_string()
        };

        Ok(text)
    }

    /// Prepares an ApprovalRequest for container mutation (Start / Stop / Restart / Prune)
    pub fn prepare_action_approval(
        &self,
        action: DockerActionType,
        target: &str,
        gateway: &PermissionGateway,
    ) -> Result<ApprovalRequest, DockerError> {
        Self::validate_target_name(target)?;

        let (title, explanation) = match action {
            DockerActionType::Start => (
                format!("Docker Container '{}' starten", target),
                format!("AIDO startet den Docker-Container '{}'. Dadurch werden die zugehörigen Netzwerkports geöffnet.", target),
            ),
            DockerActionType::Stop => (
                format!("Docker Container '{}' stoppen", target),
                format!("AIDO stoppt den Docker-Container '{}'. Laufende Prozesse im Container werden sicher beendet.", target),
            ),
            DockerActionType::Restart => (
                format!("Docker Container '{}' neustarten", target),
                format!("AIDO startet den Docker-Container '{}' neu.", target),
            ),
            DockerActionType::PruneUnused => (
                "Ungenutzte Docker-Ressourcen bereinigen".to_string(),
                "AIDO entfernt gestoppte Container, nicht genutzte Netzwerke und dangling Images, um Speicherplatz freizugeben.".to_string(),
            ),
        };

        let req = gateway.request_approval(
            ActionKind::ManageDockerSandbox,
            title,
            explanation,
            vec![target.to_string()],
            0,
            0,
        )?;

        Ok(req)
    }

    /// Executes an approved container action with a consumed one-time ApprovalToken
    pub fn execute_action_approved(
        &self,
        action: DockerActionType,
        target: &str,
        gateway: &PermissionGateway,
        token: &ApprovalToken,
    ) -> Result<DockerActionResult, DockerError> {
        Self::validate_target_name(target)?;
        gateway.consume_token_for_execution(token, &ActionKind::ManageDockerSandbox)?;

        let cli = Self::find_docker_cli().ok_or(DockerError::CliNotFound)?;
        let mut cmd = Command::new(&cli);

        match action {
            DockerActionType::Start => {
                cmd.args(["start", target]);
            }
            DockerActionType::Stop => {
                cmd.args(["stop", target]);
            }
            DockerActionType::Restart => {
                cmd.args(["restart", target]);
            }
            DockerActionType::PruneUnused => {
                cmd.args(["system", "prune", "-f"]);
            }
        }

        let output = cmd
            .output()
            .map_err(|e| DockerError::ExecutionFailed(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = if output.status.success() {
            stdout.trim().to_string()
        } else {
            format!("Fehler: {}", stderr.trim())
        };

        Ok(DockerActionResult {
            action,
            target: target.to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            output_summary: combined,
        })
    }

    /// Returns Masterplan templates (01-fullstack, 02-ai-gpu, 03-automation)
    pub fn get_templates(&self) -> Vec<DockerSandboxTemplate> {
        self.bridge.audit_wsl2_and_sandboxes().templates
    }

    fn validate_target_name(target: &str) -> Result<(), DockerError> {
        let t = target.trim();
        if t.is_empty() || t.contains(';') || t.contains('&') || t.contains('|') || t.contains('`') || t.contains('$') {
            return Err(DockerError::InvalidTarget(format!(
                "Ungültiger oder potenziell schädlicher Containername: '{}'",
                target
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permission_gateway::{AccessMode, PrivacyMode};

    #[test]
    #[ignore = "braucht die Docker-CLI auf dem Referenz-PC (Discovery und Status)"]
    fn test_docker_cli_discovery_and_status() {
        let controller = DockerController::new();
        let gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);
        let status = controller.check_status(&gateway).expect("Status check should not fail");
        assert!(status.cli_available, "Docker CLI should be found on this machine");
        assert_eq!(status.audit.templates.len(), 3, "Must have 3 Masterplan templates");
    }

    #[test]
    fn test_docker_target_sanitization() {
        assert!(DockerController::validate_target_name("my-container-1").is_ok());
        assert!(DockerController::validate_target_name("02-ai-gpu-sandbox").is_ok());
        assert!(DockerController::validate_target_name("cont; rm -rf /").is_err());
        assert!(DockerController::validate_target_name("cont & calc.exe").is_err());
    }

    #[test]
    fn test_docker_prepare_and_execute_approved_token() {
        let controller = DockerController::new();
        let gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

        let req = controller
            .prepare_action_approval(DockerActionType::Start, "test-box", &gateway)
            .unwrap();
        assert_eq!(req.action, ActionKind::ManageDockerSandbox);

        let token = gateway.approve_request(req.request_id, None).unwrap();

        // Execution with valid token consumes token (even if daemon is stopped, token consumption is verified)
        let _ = controller.execute_action_approved(DockerActionType::Start, "test-box", &gateway, &token);

        // Reusing the same token must fail with ApprovalRequired
        let reuse_result =
            controller.execute_action_approved(DockerActionType::Start, "test-box", &gateway, &token);
        assert!(reuse_result.is_err());
        match reuse_result {
            Err(DockerError::Gateway(GatewayError::ApprovalRequired(_))) => {}
            other => panic!("Expected ApprovalRequired error, got {:?}", other),
        }
    }
}
