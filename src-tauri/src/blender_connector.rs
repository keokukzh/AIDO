use crate::permission_gateway::{ActionKind, ApprovalToken, GatewayError, PermissionGateway};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BlenderError {
    #[error("Blender wurde auf diesem System nicht gefunden")]
    NotFound,
    #[error("Ungültiger Pfad zur .blend-Datei: {0}")]
    InvalidBlendPath(String),
    #[error("Fehler beim Ausführen von Blender: {0}")]
    ExecutionFailed(String),
    #[error("Fehler beim Parsen der Blender-Szenen-Metadaten: {0}")]
    JsonParseFailed(String),
    #[error("Sicherheitsverletzung im Python-Skript: {0}")]
    ScriptSecurityViolation(String),
    #[error("Berechtigungsfehler: {0}")]
    Gateway(#[from] GatewayError),
    #[error("IO-Fehler: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderInfo {
    pub executable_path: String,
    pub version: String,
    pub is_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderObjectEntry {
    pub name: String,
    pub object_type: String,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderSceneSummary {
    pub blend_file: Option<String>,
    pub blender_version: String,
    pub active_engine: String,
    pub active_camera: Option<String>,
    pub object_count: usize,
    pub mesh_count: usize,
    pub light_count: usize,
    pub camera_count: usize,
    pub objects: Vec<BlenderObjectEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderRenderSummary {
    pub blend_file: String,
    pub output_path: String,
    pub frame_start: u32,
    pub frame_end: u32,
    pub exit_code: i32,
    pub stdout_summary: String,
}

#[derive(Debug, Clone, Default)]
pub struct BlenderConnector;

impl BlenderConnector {
    pub fn new() -> Self {
        Self
    }

    /// Discovers installed Blender executable across Program Files and LocalAppData
    pub fn find_blender() -> Option<PathBuf> {
        ["ProgramFiles", "LOCALAPPDATA"]
            .into_iter()
            .filter_map(std::env::var_os)
            .flat_map(|base| {
                let base = PathBuf::from(base);
                [
                    base.join("Blender Foundation"),
                    base.join("Programs/Blender Foundation"),
                ]
            })
            .find_map(|base| {
                let mut candidates = std::fs::read_dir(base)
                    .ok()?
                    .flatten()
                    .filter(|entry| entry.file_name().to_string_lossy().starts_with("Blender "))
                    .map(|entry| entry.path().join("blender.exe"))
                    .filter(|path| path.is_file())
                    .collect::<Vec<_>>();

                candidates.sort_by_cached_key(|path| {
                    path.parent()
                        .and_then(Path::file_name)
                        .and_then(|name| name.to_str())
                        .and_then(|name| name.strip_prefix("Blender "))
                        .map(|version| {
                            version
                                .split('.')
                                .filter_map(|part| part.parse::<u32>().ok())
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default()
                });
                candidates.pop().and_then(|path| path.canonicalize().ok())
            })
    }

    /// Probes Blender availability and returns version details
    pub fn probe_info(&self) -> Result<BlenderInfo, BlenderError> {
        let binary = Self::find_blender().ok_or(BlenderError::NotFound)?;
        let output = Command::new(&binary)
            .arg("--version")
            .output()
            .map_err(|e| BlenderError::ExecutionFailed(e.to_string()))?;

        let text = String::from_utf8_lossy(&output.stdout);
        let first_line = text.lines().next().unwrap_or("Blender (unbekannte Version)");

        Ok(BlenderInfo {
            executable_path: binary.to_string_lossy().to_string(),
            version: first_line.trim().to_string(),
            is_available: true,
        })
    }

    /// Read-only inspection of Blender scene (either the default blank scene or a specific .blend file)
    pub fn inspect_scene(
        &self,
        blend_file: Option<&Path>,
        gateway: &PermissionGateway,
    ) -> Result<BlenderSceneSummary, BlenderError> {
        gateway.check_readonly_allowed(&ActionKind::DiscoverApps)?;

        let binary = Self::find_blender().ok_or(BlenderError::NotFound)?;
        let mut cmd = Command::new(&binary);
        cmd.arg("--background");

        if let Some(file) = blend_file {
            if !file.is_file() {
                return Err(BlenderError::InvalidBlendPath(file.display().to_string()));
            }
            cmd.arg(file);
        }

        let python_snippet = r#"
import bpy, json
objs = []
for o in bpy.data.objects:
    objs.append({"name": o.name, "object_type": o.type, "visible": not o.hide_viewport})
cam_name = bpy.context.scene.camera.name if bpy.context.scene.camera else None
data = {
    "blender_version": bpy.app.version_string,
    "active_engine": bpy.context.scene.render.engine,
    "active_camera": cam_name,
    "object_count": len(objs),
    "mesh_count": sum(1 for o in objs if o["object_type"] == "MESH"),
    "light_count": sum(1 for o in objs if o["object_type"] == "LIGHT"),
    "camera_count": sum(1 for o in objs if o["object_type"] == "CAMERA"),
    "objects": objs
}
print("AIDO_BLENDER_JSON:" + json.dumps(data))
"#;

        cmd.arg("--python-expr").arg(python_snippet);

        let output = cmd
            .output()
            .map_err(|e| BlenderError::ExecutionFailed(e.to_string()))?;
        let stdout = String::from_utf8_lossy(&output.stdout);

        for line in stdout.lines() {
            if let Some(json_str) = line.strip_prefix("AIDO_BLENDER_JSON:") {
                #[derive(Deserialize)]
                struct RawInspection {
                    blender_version: String,
                    active_engine: String,
                    active_camera: Option<String>,
                    object_count: usize,
                    mesh_count: usize,
                    light_count: usize,
                    camera_count: usize,
                    objects: Vec<BlenderObjectEntry>,
                }
                let raw: RawInspection = serde_json::from_str(json_str)
                    .map_err(|e| BlenderError::JsonParseFailed(e.to_string()))?;

                return Ok(BlenderSceneSummary {
                    blend_file: blend_file.map(|p| p.display().to_string()),
                    blender_version: raw.blender_version,
                    active_engine: raw.active_engine,
                    active_camera: raw.active_camera,
                    object_count: raw.object_count,
                    mesh_count: raw.mesh_count,
                    light_count: raw.light_count,
                    camera_count: raw.camera_count,
                    objects: raw.objects,
                });
            }
        }

        Err(BlenderError::JsonParseFailed(
            "Blender hat keine AIDO_BLENDER_JSON-Antwort zurückgegeben".to_string(),
        ))
    }

    /// Evaluates a controlled Python script inside Blender in headless mode
    /// Requires an explicit ApprovalToken for ActionKind::ExecuteAppAction.
    pub fn execute_script_approved(
        &self,
        script: &str,
        blend_file: Option<&Path>,
        gateway: &PermissionGateway,
        token: &ApprovalToken,
    ) -> Result<String, BlenderError> {
        // Enforce safety against destructive prompt injection
        let lower = script.to_lowercase();
        let dangerous = ["os.system", "shutil.rmtree", "subprocess.popen", "__import__('os').system"];
        for d in dangerous {
            if lower.contains(d) {
                return Err(BlenderError::ScriptSecurityViolation(format!(
                    "Unzulässiger Systembefehl im Blender-Skript erkannt: {}",
                    d
                )));
            }
        }

        gateway.consume_token_for_execution(token, &ActionKind::ExecuteAppAction)?;

        let binary = Self::find_blender().ok_or(BlenderError::NotFound)?;
        let mut cmd = Command::new(&binary);
        cmd.arg("--background");

        if let Some(file) = blend_file {
            if !file.is_file() {
                return Err(BlenderError::InvalidBlendPath(file.display().to_string()));
            }
            cmd.arg(file);
        }

        cmd.arg("--python-expr").arg(script);

        let output = cmd
            .output()
            .map_err(|e| BlenderError::ExecutionFailed(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    }

    /// Renders a blend file to an output folder in headless background mode
    /// Requires an explicit ApprovalToken for ActionKind::ExecuteAppAction.
    pub fn render_approved(
        &self,
        blend_file: &Path,
        output_prefix: &Path,
        frame_start: u32,
        frame_end: u32,
        gateway: &PermissionGateway,
        token: &ApprovalToken,
    ) -> Result<BlenderRenderSummary, BlenderError> {
        gateway.consume_token_for_execution(token, &ActionKind::ExecuteAppAction)?;

        if !blend_file.is_file() {
            return Err(BlenderError::InvalidBlendPath(blend_file.display().to_string()));
        }

        let binary = Self::find_blender().ok_or(BlenderError::NotFound)?;
        let mut cmd = Command::new(&binary);
        cmd.arg("--background")
            .arg(blend_file)
            .arg("-o")
            .arg(output_prefix.display().to_string())
            .arg("-s")
            .arg(frame_start.to_string())
            .arg("-e")
            .arg(frame_end.to_string())
            .arg("-a");

        let output = cmd
            .output()
            .map_err(|e| BlenderError::ExecutionFailed(e.to_string()))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let summary_lines = stdout
            .lines()
            .filter(|l| l.contains("Saved:") || l.contains("Append frame") || l.contains("Time:"))
            .take(5)
            .collect::<Vec<_>>()
            .join("\n");

        Ok(BlenderRenderSummary {
            blend_file: blend_file.display().to_string(),
            output_path: output_prefix.display().to_string(),
            frame_start,
            frame_end,
            exit_code: output.status.code().unwrap_or(-1),
            stdout_summary: if summary_lines.is_empty() {
                "Render-Befehl abgeschlossen".to_string()
            } else {
                summary_lines
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permission_gateway::{AccessMode, PrivacyMode};

    #[test]
    #[ignore = "braucht installiertes Blender auf dem Referenz-PC (Discovery)"]
    fn test_blender_discovery_and_info() {
        let connector = BlenderConnector::new();
        let binary = BlenderConnector::find_blender();
        assert!(binary.is_some(), "Blender should be discovered on this workstation");
        let info = connector.probe_info().expect("Blender probe should succeed");
        assert!(info.is_available);
        assert!(info.version.contains("Blender"));
    }

    #[test]
    #[ignore = "braucht installiertes Blender auf dem Referenz-PC (Headless-Szenenanalyse)"]
    fn test_blender_scene_inspection_blank_scene() {
        let connector = BlenderConnector::new();
        let gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);
        let scene = connector.inspect_scene(None, &gateway).expect("Blank scene inspection should succeed");
        assert_eq!(scene.object_count, 3);
        assert_eq!(scene.mesh_count, 1); // Cube
        assert_eq!(scene.camera_count, 1); // Camera
        assert_eq!(scene.light_count, 1); // Light
    }

    #[test]
    fn test_blender_script_security_block() {
        let connector = BlenderConnector::new();
        let gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);
        let req = gateway
            .request_approval(
                ActionKind::ExecuteAppAction,
                "Blender",
                "Python-Skript ausführen",
                vec!["D:\\Test".to_string()],
                0,
                0,
            )
            .unwrap();
        let token = gateway.approve_request(req.request_id, None).unwrap();

        let malicious_script = "import os; os.system('echo hacked')";
        let result = connector.execute_script_approved(malicious_script, None, &gateway, &token);
        assert!(result.is_err());
        match result {
            Err(BlenderError::ScriptSecurityViolation(_)) => {}
            _ => panic!("Expected ScriptSecurityViolation"),
        }
    }
}
