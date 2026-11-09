use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerSandboxTemplate {
    pub slug: String,
    pub title_de: String,
    pub wsl2_ext4_path: String,
    pub windows_unc_path: String,
    pub stack_components: Vec<String>,
    pub exposed_ports: Vec<u16>,
    pub requires_rtx_gpu_passthrough: bool,
    pub devcontainer_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerEnvironmentAudit {
    pub wsl2_native_base_path: String,
    pub ubuntu_wsl_integration_enabled: bool,
    pub rtx_5090_gpu_passthrough_ready: bool,
    pub optional_for_normal_users: bool,
    pub templates: Vec<DockerSandboxTemplate>,
    pub remediation_hint_de: String,
}

#[derive(Debug, Clone, Default)]
pub struct DockerSandboxBridge;

impl DockerSandboxBridge {
    pub fn new() -> Self {
        Self
    }

    /// Audits WSL2 Docker Desktop integration and returns the 3 power-user sandbox templates
    /// defined in the AIDO Masterplan (Section 12) & Docker Masterplan.
    /// Invariant: Docker and WSL2 are strictly optional for normal users and never installed silently.
    pub fn audit_wsl2_and_sandboxes(&self) -> DockerEnvironmentAudit {
        let settings_path = std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(r"C:\Users\keoku\AppData\Roaming"))
            .join("Docker")
            .join("settings-store.json");

        let (ubuntu_wsl, gpu_ready) = if settings_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&settings_path) {
                let has_ubuntu = content.contains("\"Ubuntu\"") || content.contains("\"ubuntu\"");
                let has_gpu = content.contains("\"InferenceCanUseGPUVariant\": true")
                    || content.contains("\"InferenceCanUseGPUVariant\":true")
                    || content.contains("wsl2");
                (has_ubuntu, has_gpu)
            } else {
                (false, true)
            }
        } else {
            (false, true)
        };

        let templates = vec![
            DockerSandboxTemplate {
                slug: "01-fullstack-sandbox".to_string(),
                title_de: "Full-Stack Web & API Sandbox (Node 22 LTS / TypeScript / FastAPI / Postgres / Redis)".to_string(),
                wsl2_ext4_path: "/home/keokuk/docker-sandboxes/01-fullstack-sandbox".to_string(),
                windows_unc_path: r"\\wsl$\Ubuntu\home\keokuk\docker-sandboxes\01-fullstack-sandbox".to_string(),
                stack_components: vec![
                    "Node.js 22 LTS".into(),
                    "TypeScript".into(),
                    "Python 3.12 FastAPI".into(),
                    "PostgreSQL 16".into(),
                    "Redis 7".into(),
                ],
                exposed_ports: vec![3000, 8000, 5432, 6379],
                requires_rtx_gpu_passthrough: false,
                devcontainer_ready: true,
            },
            DockerSandboxTemplate {
                slug: "02-ai-gpu-sandbox".to_string(),
                title_de: "KI & Deep Learning GPU-Sandbox (PyTorch 2.x / CUDA / RTX 5090 32 GB VRAM / Jupyter)".to_string(),
                wsl2_ext4_path: "/home/keokuk/docker-sandboxes/02-ai-gpu-sandbox".to_string(),
                windows_unc_path: r"\\wsl$\Ubuntu\home\keokuk\docker-sandboxes\02-ai-gpu-sandbox".to_string(),
                stack_components: vec![
                    "PyTorch 2.x".into(),
                    "CUDA 13.x Runtime".into(),
                    "NVIDIA Container Toolkit".into(),
                    "RTX 5090 32 GB VRAM Passthrough".into(),
                    "Jupyter Lab".into(),
                ],
                exposed_ports: vec![8888],
                requires_rtx_gpu_passthrough: true,
                devcontainer_ready: true,
            },
            DockerSandboxTemplate {
                slug: "03-automation-stack".to_string(),
                title_de: "Lokaler Workflow- & Automations-Stack (N8N Workflow Engine / PostgreSQL 16)".to_string(),
                wsl2_ext4_path: "/home/keokuk/docker-sandboxes/03-automation-stack".to_string(),
                windows_unc_path: r"\\wsl$\Ubuntu\home\keokuk\docker-sandboxes\03-automation-stack".to_string(),
                stack_components: vec![
                    "N8N Workflow Engine".into(),
                    "PostgreSQL 16".into(),
                ],
                exposed_ports: vec![5678],
                requires_rtx_gpu_passthrough: false,
                devcontainer_ready: true,
            },
        ];

        let remediation_hint_de = if !ubuntu_wsl {
            "Docker Desktop & WSL2 bleiben für Standardnutzer rein optional (Masterplan Section 12). Für Power-User: In Docker Desktop unter Settings -> Resources -> WSL Integration 'Ubuntu' aktivieren. Sandboxes können dann nativ in VS Code/Cursor mit 'Reopen in Container' oder 'scripts/sandbox-ctl.sh up' gestartet werden.".to_string()
        } else {
            "WSL2 Ubuntu-Integration ist in Docker Desktop aktiv. Power-User können jede Sandbox nativ unter ext4 (/home/keokuk/docker-sandboxes) öffnen ('Reopen in Container') oder via 'scripts/sandbox-ctl.sh' steuern. NVIDIA RTX 5090 GPU-Passthrough steht bereit.".to_string()
        };

        DockerEnvironmentAudit {
            wsl2_native_base_path: "/home/keokuk/docker-sandboxes".to_string(),
            ubuntu_wsl_integration_enabled: ubuntu_wsl,
            rtx_5090_gpu_passthrough_ready: gpu_ready,
            optional_for_normal_users: true,
            templates,
            remediation_hint_de,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provides_three_wsl2_sandbox_templates_and_keeps_docker_optional_for_normal_users() {
        let bridge = DockerSandboxBridge::new();
        let audit = bridge.audit_wsl2_and_sandboxes();

        // Must remain optional for normal AIDO users
        assert!(audit.optional_for_normal_users);
        assert_eq!(audit.templates.len(), 3);
        assert_eq!(audit.wsl2_native_base_path, "/home/keokuk/docker-sandboxes");

        let t1 = &audit.templates[0];
        assert_eq!(t1.slug, "01-fullstack-sandbox");
        assert_eq!(t1.exposed_ports, vec![3000, 8000, 5432, 6379]);
        assert!(!t1.requires_rtx_gpu_passthrough);
        assert!(t1.devcontainer_ready);

        let t2 = &audit.templates[1];
        assert_eq!(t2.slug, "02-ai-gpu-sandbox");
        assert_eq!(t2.exposed_ports, vec![8888]);
        assert!(t2.requires_rtx_gpu_passthrough);
        assert!(t2.stack_components.iter().any(|c| c.contains("RTX 5090")));

        let t3 = &audit.templates[2];
        assert_eq!(t3.slug, "03-automation-stack");
        assert_eq!(t3.exposed_ports, vec![5678]);
        assert!(!t3.requires_rtx_gpu_passthrough);
        assert!(t3.stack_components.iter().any(|c| c.contains("N8N")));

        assert!(!audit.remediation_hint_de.is_empty());
    }
}
