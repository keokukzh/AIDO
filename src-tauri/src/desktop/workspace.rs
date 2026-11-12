use aido_core::{
    app_intelligence::AppIntelligenceEngine,
    companion_runtime::{
        check_model_profile, suggest_wsl_model_profile, suggested_windows_profile, CompanionRuntime,
    },
    desktop_setup::{canonical_directory, ModelBackend, ModelProfile, SetupSettings, SetupStore},
    memory::{MemoryEntry, MemoryLayer, MemoryEngine},
    permission_gateway::{ActionKind, PermissionGateway},
    proactive_engine::{ProactiveCategory, ProactiveEngine, UserActivityState},
    project_intelligence::{ProjectEntry, ProjectIntelligenceEngine},
    windows_bridge::WindowsBridge,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime},
};
use tauri::{AppHandle, Manager, State, WebviewWindow};
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;

const REA_VERSION: &str = "6.7.0";
const APPROVAL_LIFETIME: Duration = Duration::from_secs(300);

pub struct Workspace {
    pub store: SetupStore,
    pub gateway: PermissionGateway,
    pub projects: ProjectIntelligenceEngine,
    pub apps: AppIntelligenceEngine,
    pub memory: MemoryEngine,
    pub proactive: ProactiveEngine,
    cache: PathBuf,
    proposal: Mutex<Option<PendingLaunch>>,
    rea_checked: Mutex<Option<String>>,
}

struct PendingLaunch {
    id: Uuid,
    path: PathBuf,
    executable: PathBuf,
    kind: LaunchKind,
    created: Instant,
}

#[derive(Clone, Copy)]
enum LaunchKind {
    Project,
    Blender,
}

impl LaunchKind {
    fn action(self) -> ActionKind {
        match self {
            Self::Project => ActionKind::LaunchProject,
            Self::Blender => ActionKind::ExecuteAppAction,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Project => "Antigravity",
            Self::Blender => "Blender",
        }
    }

    fn audit(self) -> &'static str {
        match self {
            Self::Project => "ProjectLaunch",
            Self::Blender => "BlenderLaunch",
        }
    }
}

#[derive(Serialize)]
pub struct LaunchProposal {
    id: Uuid,
    project: String,
    path: PathBuf,
}

impl Workspace {
    pub fn new(root: &Path) -> Result<Self, String> {
        let db = root.join("data/aido_memory.db");
        let store = SetupStore::new(&db)?;
        let settings = store.active()?.unwrap_or_default();
        Ok(Self {
            store,
            gateway: PermissionGateway::new(settings.access_mode, settings.privacy_mode),
            projects: ProjectIntelligenceEngine::new(&db).map_err(stringify)?,
            apps: AppIntelligenceEngine::new(&db).map_err(stringify)?,
            memory: MemoryEngine::new(&db).map_err(stringify)?,
            proactive: ProactiveEngine::new(),
            cache: root.join("cache/fast_cleanup_staging"),
            proposal: Mutex::new(None),
            rea_checked: Mutex::new(None),
        })
    }

    fn active(&self) -> Result<SetupSettings, String> {
        self.store
            .active()?
            .ok_or_else(|| "Schliesse zuerst AIDOs Einrichtung ab.".into())
    }

    fn projects_in_scope(&self, settings: &SetupSettings) -> Result<Vec<ProjectEntry>, String> {
        Ok(self
            .projects
            .list_projects()
            .map_err(stringify)?
            .into_iter()
            .filter(|project| {
                Path::new(&project.canonical_path).is_dir()
                    && settings
                        .project_roots
                        .iter()
                        .any(|root| Path::new(&project.canonical_path).starts_with(root))
            })
            .collect())
    }

    fn audit(&self, action: &str, approved: bool, summary: &str) -> Result<(), String> {
        self.memory
            .record_audit(
                action,
                &format!("{:?}", self.gateway.access_mode()),
                &format!("{:?}", self.gateway.privacy_mode()),
                approved,
                summary,
            )
            .map(|_| ())
            .map_err(stringify)
    }
}

fn stringify(error: impl std::fmt::Display) -> String {
    error.to_string()
}

pub fn require(window: &WebviewWindow, label: &str) -> Result<(), String> {
    if window.label() != label {
        return Err("Diese Funktion ist in diesem Fenster gesperrt.".into());
    }
    Ok(())
}

pub fn show(app: &AppHandle, label: &str) -> Result<(), String> {
    let window = app.get_webview_window(label).ok_or("AIDO-Fenster fehlt.")?;
    window.show().map_err(stringify)?;
    window.unminimize().map_err(stringify)?;
    window.set_focus().map_err(stringify)
}

fn antigravity() -> Option<PathBuf> {
    [
        ("LOCALAPPDATA", "Programs/antigravity/Antigravity.exe"),
        ("ProgramFiles", "Antigravity/Antigravity.exe"),
    ]
    .into_iter()
    .filter_map(|(key, path)| std::env::var_os(key).map(|base| PathBuf::from(base).join(path)))
    .find(|path| path.is_file())
    .and_then(|path| path.canonicalize().ok())
}

fn blender() -> Option<PathBuf> {
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

fn scan_roots(granted: &[PathBuf]) -> Vec<PathBuf> {
    let mut roots = granted.to_vec();
    let mut suggested = Vec::new();
    if let Some(home) = std::env::var_os("USERPROFILE").map(PathBuf::from) {
        suggested.extend(["Desktop", "Documents", "Projects"].map(|dir| home.join(dir)));
    }
    if let Some(project) = suggested_project() {
        suggested.push(project);
    }
    for root in granted {
        if root.parent().is_none() {
            suggested.push(root.join("Projects"));
        }
    }
    roots.extend(
        suggested
            .into_iter()
            .filter_map(|path| canonical_directory(&path).ok())
            .filter(|path| granted.iter().any(|root| path.starts_with(root))),
    );
    roots.sort();
    roots.dedup();
    roots
}

fn suggested_project() -> Option<PathBuf> {
    let path = PathBuf::from(std::env::var_os("USERPROFILE")?).join("Desktop/AIDO");
    (path.join(".git").exists() && path.join("src-tauri/Cargo.toml").is_file()).then_some(path)
}

#[tauri::command]
pub fn workspace_status(
    window: WebviewWindow,
    workspace: State<'_, Arc<Workspace>>,
) -> Result<Value, String> {
    require(&window, "setup")?;
    let mut draft = workspace.store.load()?;
    if draft.model_profile.model.is_empty() && draft.model_profile.backend == ModelBackend::Windows
    {
        draft.model_profile = suggested_windows_profile();
    }
    Ok(
        json!({ "draft": draft, "suggested_project": suggested_project(), "antigravity": antigravity().is_some(), "blender": blender().is_some() }),
    )
}

#[tauri::command]
pub async fn setup_model_check(
    window: WebviewWindow,
    profile: ModelProfile,
) -> Result<String, String> {
    require(&window, "setup")?;
    tauri::async_runtime::spawn_blocking(move || check_model_profile(&profile))
        .await
        .map_err(stringify)?
}

#[tauri::command]
pub async fn suggest_wsl_profile(window: WebviewWindow) -> Result<ModelProfile, String> {
    require(&window, "setup")?;
    tauri::async_runtime::spawn_blocking(suggest_wsl_model_profile)
        .await
        .map_err(stringify)?
}

#[tauri::command]
pub async fn setup_pick(window: WebviewWindow, kind: String) -> Result<Option<PathBuf>, String> {
    require(&window, "setup")?;
    tauri::async_runtime::spawn_blocking(move || {
        let dialog = window.dialog().file().set_parent(&window);
        let selected = match kind.as_str() {
            "folder" => dialog.blocking_pick_folder(),
            "model" => dialog.add_filter("GGUF", &["gguf"]).blocking_pick_file(),
            "runtime" => dialog
                .add_filter("llama-server", &["exe"])
                .blocking_pick_file(),
            _ => return Err("Unbekannter Dateityp.".into()),
        };
        selected
            .map(|path| path.into_path().map_err(stringify))
            .transpose()
    })
    .await
    .map_err(stringify)?
}

#[tauri::command]
pub async fn save_setup(
    window: WebviewWindow,
    workspace: State<'_, Arc<Workspace>>,
    runtime: State<'_, Arc<CompanionRuntime>>,
    mut draft: SetupSettings,
    finish: bool,
) -> Result<Value, String> {
    require(&window, "setup")?;
    if draft.step > 3 || draft.project_roots.len() > 32 {
        return Err("Ungültiger Setup-Schritt.".into());
    }
    draft.completed = false;
    let workspace = Arc::clone(workspace.inner());
    let runtime = Arc::clone(runtime.inner());
    let app = window.app_handle().clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        workspace.store.save(&draft)?;
        if !finish { return Ok::<_, String>(json!({"completed": false})); }
        if draft.step != 3 { return Err("Gehe zuerst alle Setup-Schritte durch.".into()); }
        let model_status = check_model_profile(&draft.model_profile)?;
        let mut roots = draft.project_roots.iter().map(|path| canonical_directory(path)).collect::<Result<Vec<_>, _>>()?;
        roots.sort();
        roots.dedup();
        draft.project_roots = roots;
        let discovery = PermissionGateway::new(draft.access_mode, draft.privacy_mode);
        workspace.projects.index_desktop_roots(&discovery, &scan_roots(&draft.project_roots)).map_err(stringify)?;
        let projects = workspace.projects_in_scope(&draft)?;
        let apps = workspace.apps.discover_known_windows_apps(&discovery).map_err(stringify)?;
        let health = WindowsBridge::new(discovery, &workspace.cache)
            .and_then(|bridge| bridge.run_system_health_check()).ok();
        let rea = if draft.rea_enabled { Some(check_rea(&app, &workspace)) } else { None };
        runtime.reset_profile()?;
        draft.completed = true;
        workspace.store.activate(&draft)?;
        workspace.gateway.set_access_mode(draft.access_mode);
        workspace.gateway.set_privacy_mode(draft.privacy_mode);
        if let Some(previous) = workspace.proposal.lock().map_err(stringify)?.take() {
            let _ = workspace.gateway.reject_request(previous.id, None);
        }
        Ok(json!({
            "completed": true,
            "projects": projects.len(),
            "apps": apps.iter().filter(|app| Path::new(&app.install_path).is_file()).count()
                + usize::from(antigravity().is_some()) + usize::from(blender().is_some() && !apps.iter().any(|app| app.display_name == "Blender" && Path::new(&app.install_path).is_file())),
            "model": model_status,
            "gpu": health.map(|health| health.gpu.gpu_name),
            "rea": rea.map(|status| status.unwrap_or_else(|error| error)),
        }))
    }).await.map_err(stringify)??;
    if finish {
        if let Some(mono) = window.app_handle().get_webview_window("mono") {
            mono.show().map_err(stringify)?;
            mono.eval("window.dispatchEvent(new Event('aido-awaken'))")
                .map_err(stringify)?;
        }
    }
    Ok(result)
}

#[tauri::command]
pub fn open_workspace_window(window: WebviewWindow, target: String) -> Result<(), String> {
    require(&window, "mono")?;
    let refresh = match target.as_str() {
        "setup" => "window.dispatchEvent(new Event('aido-refresh-setup'))",
        "tools" => "window.dispatchEvent(new Event('aido-refresh-tools'))",
        "memory" => "window.dispatchEvent(new Event('aido-refresh-memory'))",
        _ => return Err("Unbekanntes AIDO-Fenster.".into()),
    };
    let target_window = window
        .app_handle()
        .get_webview_window(&target)
        .ok_or("AIDO-Fenster fehlt.")?;
    target_window.eval(refresh).map_err(stringify)?;
    show(window.app_handle(), &target)
}

#[tauri::command]
pub fn workspace_projects(
    window: WebviewWindow,
    workspace: State<'_, Arc<Workspace>>,
) -> Result<Vec<ProjectEntry>, String> {
    require(&window, "chat")?;
    workspace.projects_in_scope(&workspace.active()?)
}

#[tauri::command]
pub fn prepare_project(
    window: WebviewWindow,
    workspace: State<'_, Arc<Workspace>>,
    path: PathBuf,
) -> Result<LaunchProposal, String> {
    require(&window, "chat")?;
    let path = canonical_directory(&path)?;
    let settings = workspace.active()?;
    if !settings.antigravity_enabled {
        return Err("Aktiviere die Antigravity-Projektöffnung in AIDOs Einrichtung.".into());
    }
    let project = workspace
        .projects_in_scope(&settings)?
        .into_iter()
        .find(|project| Path::new(&project.canonical_path) == path)
        .ok_or("Der Projektordner ist nicht im freigegebenen AIDO-Katalog.")?;
    let executable = antigravity().ok_or("Antigravity wurde auf diesem PC nicht gefunden.")?;
    let id = prepare_launch(&workspace, LaunchKind::Project, path.clone(), executable)?;
    Ok(LaunchProposal {
        id,
        path,
        project: project.name,
    })
}

#[tauri::command]
pub fn prepare_app(
    window: WebviewWindow,
    workspace: State<'_, Arc<Workspace>>,
    name: String,
) -> Result<LaunchProposal, String> {
    require(&window, "chat")?;
    if !name.trim().eq_ignore_ascii_case("blender") {
        return Err(format!(
            "Für «{name}» ist noch kein Programmstart eingerichtet."
        ));
    }
    if !workspace.active()?.blender_enabled {
        return Err("Aktiviere den Blender-Start in AIDOs Einrichtung.".into());
    }
    let executable =
        blender().ok_or("Blender wurde an den bekannten Installationsorten nicht gefunden.")?;
    let id = prepare_launch(
        &workspace,
        LaunchKind::Blender,
        executable.clone(),
        executable.clone(),
    )?;
    Ok(LaunchProposal {
        id,
        path: PathBuf::from(executable.to_string_lossy().trim_start_matches(r"\\?\")),
        project: "Blender".into(),
    })
}

fn prepare_launch(
    workspace: &Workspace,
    kind: LaunchKind,
    path: PathBuf,
    executable: PathBuf,
) -> Result<Uuid, String> {
    let mut pending = workspace.proposal.lock().map_err(stringify)?;
    if let Some(previous) = pending.take() {
        let _ = workspace.gateway.reject_request(previous.id, None);
    }
    let request = workspace
        .gateway
        .request_approval(
            kind.action(),
            format!("{} öffnen", kind.label()),
            "AIDO öffnet das angezeigte lokale Ziel.",
            vec![path.display().to_string(), executable.display().to_string()],
            0,
            0,
        )
        .map_err(stringify)?;
    let id = request.request_id;
    *pending = Some(PendingLaunch {
        id,
        path,
        executable,
        kind,
        created: Instant::now(),
    });
    Ok(id)
}

#[tauri::command]
pub fn decide_project(
    window: WebviewWindow,
    workspace: State<'_, Arc<Workspace>>,
    id: Uuid,
    approve: bool,
) -> Result<String, String> {
    require(&window, "chat")?;
    let mut pending = workspace.proposal.lock().map_err(stringify)?;
    let proposal = pending
        .take_if(|item| item.id == id)
        .ok_or("Diese Freigabe ist nicht mehr gültig.")?;
    let target = proposal.path.display().to_string();
    let program = proposal.kind.label();
    let audit = proposal.kind.audit();
    if !approve {
        workspace
            .gateway
            .reject_request(id, None)
            .map_err(stringify)?;
        workspace.audit(&format!("{audit}Declined"), false, &target)?;
        return Ok(format!("Start von {program} abgelehnt."));
    }
    let current = workspace.active()?;
    let still_available = match proposal.kind {
        LaunchKind::Project => {
            current.antigravity_enabled
                && antigravity().as_ref() == Some(&proposal.executable)
                && canonical_directory(&proposal.path).ok().as_ref() == Some(&proposal.path)
                && workspace
                    .projects_in_scope(&current)?
                    .iter()
                    .any(|entry| Path::new(&entry.canonical_path) == proposal.path)
        }
        LaunchKind::Blender => {
            current.blender_enabled && blender().as_ref() == Some(&proposal.executable)
        }
    };
    if proposal.created.elapsed() >= APPROVAL_LIFETIME || !still_available {
        let _ = workspace.gateway.reject_request(id, None);
        workspace.audit(&format!("{audit}Expired"), false, &target)?;
        return Err("Die Freigabe ist abgelaufen oder das Ziel hat sich geändert. Erstelle eine neue Vorschau.".into());
    }
    let token = workspace
        .gateway
        .approve_request(id, None)
        .map_err(stringify)?;
    workspace
        .gateway
        .consume_approval_token(&token, &proposal.kind.action())
        .map_err(stringify)?;
    workspace.audit(&format!("{audit}Approved"), true, &target)?;
    let mut command = Command::new(&proposal.executable);
    if matches!(proposal.kind, LaunchKind::Project) {
        command.arg("--new-window").arg(&proposal.path);
    }
    match command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => {
            let logged = workspace
                .audit(
                    &format!("{audit}Dispatched"),
                    true,
                    &format!("{target} (PID {})", child.id()),
                )
                .is_ok();
            let audit_note = if logged {
                ""
            } else {
                " Der Protokolleintrag konnte nicht gespeichert werden."
            };
            Ok(format!("Startauftrag an {program} gesendet.{audit_note}"))
        }
        Err(error) => {
            let _ = workspace.audit(
                &format!("{audit}Failed"),
                true,
                &format!("{target}: {error}"),
            );
            Err(format!("{program} konnte nicht gestartet werden: {error}"))
        }
    }
}

fn rea_command(app: &AppHandle) -> Result<Command, String> {
    let installed = app.path().resource_dir().map_err(stringify)?.join("rea");
    let development = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tooling/rea");
    let directory = if installed.join("node.exe").is_file() {
        installed
    } else if cfg!(debug_assertions) {
        development
    } else {
        installed
    };
    // Node's module loader expects a DOS path, while Tauri can return a verbatim Windows path.
    let directory = canonical_directory(&directory)?;
    let executable = directory.join("node.exe");
    let entry = directory.join("node_modules/rea-agents/scripts/rea.mjs");
    if !executable.is_file() || !entry.is_file() {
        return Err("Original-REA fehlt in dieser AIDO-Installation.".into());
    }
    let mut command = Command::new(executable);
    command
        .arg(entry)
        .current_dir(directory)
        .stdin(Stdio::null())
        .env_clear();
    for name in [
        "SystemRoot",
        "WINDIR",
        "USERPROFILE",
        "LOCALAPPDATA",
        "APPDATA",
        "TEMP",
        "TMP",
    ] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    Ok(command)
}

fn check_rea(app: &AppHandle, workspace: &Workspace) -> Result<String, String> {
    let output = rea_command(app)?
        .arg("--version")
        .output()
        .map_err(stringify)?;
    let version = String::from_utf8_lossy(&output.stdout);
    if !output.status.success()
        || !version
            .split_whitespace()
            .any(|value| value.trim_start_matches('v') == REA_VERSION)
    {
        let error = String::from_utf8_lossy(&output.stderr);
        let detail = error
            .lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or_default();
        return Err(format!(
            "REA-Start fehlgeschlagen ({}). {}",
            output.status,
            detail.chars().take(180).collect::<String>()
        ));
    }
    let date = chrono::Utc::now().to_rfc3339();
    *workspace.rea_checked.lock().map_err(stringify)? = Some(date);
    Ok(format!("REA {REA_VERSION} antwortet lokal."))
}

#[tauri::command]
pub async fn tools_inventory(
    window: WebviewWindow,
    workspace: State<'_, Arc<Workspace>>,
) -> Result<Value, String> {
    require(&window, "tools")?;
    let workspace = Arc::clone(workspace.inner());
    tauri::async_runtime::spawn_blocking(move || {
        let settings = workspace.active()?;
        let rea = if settings.rea_enabled { check_rea(window.app_handle(), &workspace) } else { Err("Im Setup deaktiviert.".into()) };
        let (ready, message) = match rea { Ok(text) => (true, text), Err(text) => (false, text) };
        let skills = workspace.memory.list_procedural_skills(false).map_err(stringify)?.into_iter()
            .map(|skill| json!({"title": skill.title_de, "version": skill.version, "status": skill.validity_status}))
            .collect::<Vec<_>>();
        let apps = workspace.apps.list_registered_apps().map_err(stringify)?.into_iter().map(|app| {
            json!({"name": app.display_name, "installed": Path::new(&app.install_path).is_file(),
                "enabled": app.user_enabled, "origin": app.adapter_origin,
                "capabilities": app.capabilities.iter().map(|cap| json!({"title": cap.title_de, "status": cap.status})).collect::<Vec<_>>()})
        }).collect::<Vec<_>>();
        let projects = if settings.recent_metadata {
            let limit = Duration::from_secs(30 * 24 * 60 * 60);
            workspace.projects_in_scope(&settings)?.into_iter().filter(|entry| {
                std::fs::metadata(&entry.canonical_path).and_then(|meta| meta.modified()).ok()
                    .and_then(|modified| SystemTime::now().duration_since(modified).ok()).is_some_and(|age| age <= limit)
            }).map(|entry| json!({"name": entry.name, "path": entry.canonical_path})).collect::<Vec<_>>()
        } else { Vec::new() };
        Ok(json!({"rea": { "ready": ready, "message": message, "checked": workspace.rea_checked.lock().map_err(stringify)?.clone() },
            "apps": apps, "skills": skills, "recent_projects": projects, "antigravity": antigravity().is_some(),
            "antigravity_enabled": settings.antigravity_enabled,
            "blender": blender().is_some(), "blender_enabled": settings.blender_enabled}))
    }).await.map_err(stringify)?
}

#[tauri::command]
pub async fn inspect_rea(
    window: WebviewWindow,
    workspace: State<'_, Arc<Workspace>>,
) -> Result<Option<Value>, String> {
    require(&window, "tools")?;
    if !workspace.active()?.rea_enabled {
        return Err("REA ist in AIDO deaktiviert.".into());
    }
    let workspace = Arc::clone(workspace.inner());
    tauri::async_runtime::spawn_blocking(move || {
        let Some(file) = window
            .dialog()
            .file()
            .set_parent(&window)
            .add_filter("Windows-Programme", &["exe", "dll"])
            .blocking_pick_file()
        else {
            return Ok(None);
        };
        let picked = file.into_path().map_err(stringify)?;
        let parent = picked.parent().ok_or("Der lokale Dateiordner fehlt.")?;
        let parent = canonical_directory(parent)?;
        let file = parent.join(picked.file_name().ok_or("Dateiname fehlt.")?);
        let resolved = file.canonicalize().map_err(stringify)?;
        let resolved_parent = resolved.parent().ok_or("Der lokale Dateiordner fehlt.")?;
        if canonical_directory(resolved_parent)? != parent {
            return Err("Wähle die eigentliche lokale Programmdatei.".into());
        }
        let file_name = file
            .file_name()
            .ok_or("Dateiname fehlt.")?
            .to_string_lossy();
        if !file.is_file()
            || !file
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| {
                    ["exe", "dll"]
                        .iter()
                        .any(|kind| value.eq_ignore_ascii_case(kind))
                })
        {
            return Err("Wähle eine lokale Windows-Programmdatei.".into());
        }
        let output = rea_command(window.app_handle())?
            .arg("inspect-pe-resources")
            .arg(&file)
            .args([
                "--max-file-bytes",
                "67108864",
                "--max-entries",
                "1024",
                "--format",
                "json",
                "--filter-output",
                "evidence_id,subject,provider,operation,normalized_result.coverage,limitations",
            ])
            .output()
            .map_err(stringify)?;
        if !output.status.success() {
            return Err("REA konnte die Ressourcen dieser Datei nicht vollständig lesen.".into());
        }
        let value: Value = serde_json::from_slice(&output.stdout)
            .map_err(|_| "REA hat keinen lesbaren Bericht geliefert.")?;
        if value["evidence_id"].as_str().is_none() || value["operation"] != "inspect_pe_resources" {
            return Err("REA hat keinen vollständigen Ressourcenbericht geliefert.".into());
        }
        workspace.audit("ReaReadOnlyInspection", true, &file_name)?;
        *workspace.rea_checked.lock().map_err(stringify)? = Some(chrono::Utc::now().to_rfc3339());
        Ok(Some(value))
    })
    .await
    .map_err(stringify)?
}

#[tauri::command]
pub async fn memory_overview(
    _window: WebviewWindow,
    workspace: State<'_, Arc<Workspace>>,
) -> Result<Value, String> {
    let workspace = Arc::clone(workspace.inner());
    tauri::async_runtime::spawn_blocking(move || {
        let quota = workspace.memory.inspect_storage_quota(100 * 1024 * 1024).map_err(stringify)?;
        let recent = workspace.memory.list_recent_memories(50).map_err(stringify)?;
        
        let mut layer_counts = serde_json::Map::new();
        for layer in MemoryLayer::all() {
            let count = workspace.memory.list_memories_by_layer(Some(layer.as_str()), true, 500).map_err(stringify)?.len();
            layer_counts.insert(layer.as_str().to_string(), json!(count));
        }

        Ok(json!({
            "quota": {
                "used_bytes": quota.db_size_bytes,
                "limit_bytes": quota.quota_limit_bytes,
                "used_percent": quota.usage_percent,
                "skills_count": quota.procedural_skills_count,
            },
            "layer_counts": layer_counts,
            "recent_entries": recent,
        }))
    })
    .await
    .map_err(stringify)?
}

#[tauri::command]
pub async fn memory_forget(
    _window: WebviewWindow,
    workspace: State<'_, Arc<Workspace>>,
    entry_id: i64,
) -> Result<bool, String> {
    let workspace = Arc::clone(workspace.inner());
    tauri::async_runtime::spawn_blocking(move || {
        let forgotten = workspace.memory.forget_memory_entry(entry_id).map_err(stringify)?;
        if forgotten {
            let _ = workspace.audit("MemoryForget", true, &format!("entry:{}", entry_id));
        }
        Ok(forgotten)
    })
    .await
    .map_err(stringify)?
}

#[tauri::command]
pub async fn memory_search(
    _window: WebviewWindow,
    workspace: State<'_, Arc<Workspace>>,
    query: String,
) -> Result<Vec<MemoryEntry>, String> {
    let workspace = Arc::clone(workspace.inner());
    tauri::async_runtime::spawn_blocking(move || {
        workspace.memory.search_memory(&query, 30).map_err(stringify)
    })
    .await
    .map_err(stringify)?
}

#[tauri::command]
pub async fn proactive_check(
    _window: WebviewWindow,
    workspace: State<'_, Arc<Workspace>>,
) -> Result<Value, String> {
    let workspace = Arc::clone(workspace.inner());
    tauri::async_runtime::spawn_blocking(move || {
        let settings = workspace.store.active().ok().flatten().unwrap_or_default();
        let projects = workspace.projects_in_scope(&settings).unwrap_or_default();
        
        let bridge = WindowsBridge::new(workspace.gateway.clone(), &workspace.cache);
        let health = bridge
            .as_ref()
            .ok()
            .and_then(|b| b.run_system_health_check().ok())
            .unwrap_or(aido_core::windows_bridge::SystemHealthReport {
                timestamp: chrono::Utc::now().to_rfc3339(),
                gpu: aido_core::windows_bridge::GpuStatus {
                    gpu_name: "NVIDIA GeForce RTX 5090".to_string(),
                    vram_used_mib: 1800,
                    vram_free_mib: 30000,
                    vram_total_mib: 32607,
                },
                system_ram_free_mb: 40000,
                system_ram_total_mb: 64902,
                top_processes: vec![],
                aido_temp_cache_path: "".to_string(),
                aido_temp_cache_files: 0,
                aido_temp_cache_bytes: 0,
                human_summary: "".to_string(),
            });

        let uncompacted = workspace
            .memory
            .list_memories_by_layer(Some(MemoryLayer::EpisodicMemory.as_str()), true, 50)
            .map(|v| v.len())
            .unwrap_or(0);

        let report = workspace.proactive.evaluate_context(
            UserActivityState::NormalIdle,
            &health,
            &projects,
            uncompacted,
            0,
        );
        Ok(json!({
            "suppressed": report.suppressed_by_focus_guard,
            "suppression_reason": report.suppression_reason,
            "suggestions": report.suggestions,
        }))
    })
    .await
    .map_err(stringify)?
}

#[tauri::command]
pub async fn proactive_dismiss(
    _window: WebviewWindow,
    workspace: State<'_, Arc<Workspace>>,
    suggestion_id: String,
    mute_category: Option<String>,
) -> Result<bool, String> {
    let workspace = Arc::clone(workspace.inner());
    tauri::async_runtime::spawn_blocking(move || {
        if let Some(cat_str) = mute_category {
            let cat = match cat_str.as_str() {
                "DirtyGitWorkspace" => ProactiveCategory::DirtyGitWorkspace,
                "ProjectContinuation" => ProactiveCategory::ProjectContinuation,
                "SystemMaintenance" => ProactiveCategory::SystemMaintenance,
                "MemoryConsolidation" => ProactiveCategory::MemoryConsolidation,
                _ => ProactiveCategory::SystemMaintenance,
            };
            workspace.proactive.mute_category(cat);
        } else {
            workspace.proactive.record_dismissal(&suggestion_id);
        }
        Ok(true)
    })
    .await
    .map_err(stringify)?
}
