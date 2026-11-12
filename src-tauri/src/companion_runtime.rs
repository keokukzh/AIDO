use crate::ai_client::{
    loopback_agent, read_model_json, ChatMessage, ChatRole, LocalAiClient, LLAMA_URL, QWEN_MODEL_ID,
};
use crate::desktop_setup::{ModelBackend, ModelProfile};
use crate::permission_gateway::{AccessMode, PermissionGateway, PrivacyMode};
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant};

const LLAMA_ADDRESS: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 18434);
const LLAMA_BINARY: &str = r"D:\AI_Models\unsloth\llama.cpp\build\bin\Release\llama-server.exe";
const DEFAULT_MODEL: &str = r"D:\AI_Models\llm\Qwen3.8-27B-UD-Q4_K_M.gguf";

pub struct CompanionRuntime {
    owned: Mutex<Option<OwnedServer>>,
    request: Mutex<()>,
    stopping: AtomicBool,
}

struct OwnedServer {
    process: Child,
    key: String,
    url: String,
    lease: Option<ChildStdin>,
    profile: ModelProfile,
    #[cfg(windows)]
    _job: Option<WindowsJob>,
}

#[cfg(windows)]
struct WindowsJob(std::os::windows::io::OwnedHandle);

#[cfg(windows)]
impl WindowsJob {
    fn new() -> std::io::Result<Self> {
        use std::os::windows::io::{AsRawHandle, FromRawHandle};
        use windows_sys::Win32::System::JobObjects::{
            CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        };
        let raw = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if raw.is_null() {
            return Err(std::io::Error::last_os_error());
        }
        // The returned job handle is uniquely owned and not inheritable.
        let handle = unsafe { std::os::windows::io::OwnedHandle::from_raw_handle(raw) };
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let configured = unsafe {
            SetInformationJobObject(
                handle.as_raw_handle(),
                JobObjectExtendedLimitInformation,
                (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of_val(&limits) as u32,
            )
        };
        if configured == 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Self(handle))
    }

    fn assign(&self, child: &Child) -> std::io::Result<()> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::JobObjects::AssignProcessToJobObject;
        // Both handles stay valid for the call; the job owns the cleanup policy.
        let assigned =
            unsafe { AssignProcessToJobObject(self.0.as_raw_handle(), child.as_raw_handle()) };
        if assigned == 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(())
    }
}

impl Drop for OwnedServer {
    fn drop(&mut self) {
        let lease = self.lease.take();
        let was_wsl = lease.is_some();
        drop(lease);
        if was_wsl {
            for _ in 0..30 {
                if matches!(self.process.try_wait(), Ok(Some(_))) {
                    return;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

impl Default for CompanionRuntime {
    fn default() -> Self {
        Self {
            owned: Mutex::new(None),
            request: Mutex::new(()),
            stopping: AtomicBool::new(false),
        }
    }
}

impl CompanionRuntime {
    pub fn chat(&self, messages: &[ChatMessage]) -> Result<String, String> {
        self.chat_with_profile(messages, &ModelProfile::default())
    }

    pub fn chat_with_profile(
        &self,
        messages: &[ChatMessage],
        profile: &ModelProfile,
    ) -> Result<String, String> {
        validate_conversation(messages)?;
        let _request = self
            .request
            .try_lock()
            .map_err(|_| "Eine Anfrage läuft bereits.")?;
        let (url, model_id, key) = self.ensure_server(profile)?;
        let client = LocalAiClient::with_endpoints(
            PermissionGateway::new(AccessMode::ReadOnly, PrivacyMode::PrivateOffline),
            url,
            "http://127.0.0.1:18080",
            model_id,
        );
        client
            .companion_chat(messages, key.as_deref())
            .map_err(|error| error.to_string())
    }

    pub fn reset_profile(&self) -> Result<(), String> {
        let _request = self
            .request
            .try_lock()
            .map_err(|_| "Warte, bis die laufende Modellanfrage abgeschlossen ist.")?;
        drop(self.owned().take());
        Ok(())
    }

    fn owned(&self) -> MutexGuard<'_, Option<OwnedServer>> {
        self.owned.lock().unwrap_or_else(|error| error.into_inner())
    }

    pub fn shutdown(&self) {
        self.stopping.store(true, Ordering::Release);
        drop(self.owned().take());
    }

    fn ensure_server(
        &self,
        profile: &ModelProfile,
    ) -> Result<(String, String, Option<String>), String> {
        let (url, key, newly_started) = {
            let mut owned = self.owned();
            if self.stopping.load(Ordering::Acquire) {
                return Err("AIDO wird beendet.".into());
            }
            if let Some(server) = owned.as_mut() {
                if server.profile != *profile
                    || server
                        .process
                        .try_wait()
                        .map_err(|error| error.to_string())?
                        .is_some()
                {
                    owned.take();
                }
            }
            if let Some(server) = owned.as_ref() {
                (server.url.clone(), server.key.clone(), false)
            } else {
                let occupied =
                    TcpStream::connect_timeout(&LLAMA_ADDRESS.into(), Duration::from_millis(250))
                        .is_ok();
                if profile.backend == ModelBackend::Windows
                    && occupied
                    && profile.model.is_empty()
                    && profile.executable.is_empty()
                {
                    let key = std::env::var("AIDO_LOCAL_API_KEY").ok();
                    if let Some(model_id) = find_qwen(LLAMA_URL, key.as_deref()) {
                        return Ok((LLAMA_URL.into(), model_id, key));
                    }
                }
                let port = if occupied || profile.backend == ModelBackend::Wsl {
                    TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
                        .and_then(|listener| listener.local_addr())
                        .map_err(|error| format!("Kein freier lokaler Port für Qwen: {error}"))?
                        .port()
                } else {
                    LLAMA_ADDRESS.port()
                };
                let server = start_server(port, profile)?;
                let connection = (server.url.clone(), server.key.clone(), true);
                *owned = Some(server);
                connection
            }
        };

        let deadline = Instant::now() + Duration::from_secs(90);
        loop {
            if self.stopping.load(Ordering::Acquire) {
                return Err("AIDO wird beendet.".into());
            }
            if let Some(model_id) = find_qwen(&url, Some(&key)) {
                return Ok((url, model_id, Some(key)));
            }
            if !newly_started {
                return Err(
                    "Der AIDO-Modelldienst ist gerade nicht bereit. Versuche es erneut.".into(),
                );
            }
            {
                let mut owned = self.owned();
                let status = owned
                    .as_mut()
                    .and_then(|server| server.process.try_wait().ok().flatten());
                if let Some(status) = status {
                    owned.take();
                    return Err(format!("Qwen konnte nicht gestartet werden ({status})."));
                }
                if Instant::now() >= deadline {
                    owned.take();
                    return Err(
                        "Qwen wurde innerhalb von 90 Sekunden nicht bereit. Versuche es erneut."
                            .into(),
                    );
                }
            }
            std::thread::sleep(Duration::from_millis(300));
        }
    }
}

fn validate_conversation(messages: &[ChatMessage]) -> Result<(), String> {
    if messages.is_empty()
        || messages.len() > 15
        || messages.len() % 2 == 0
        || messages
            .iter()
            .map(|message| message.content.len())
            .sum::<usize>()
            > 6000
    {
        return Err("Die Unterhaltung ist leer oder zu lang.".into());
    }
    for (index, message) in messages.iter().enumerate() {
        let role = if index % 2 == 0 {
            ChatRole::User
        } else {
            ChatRole::Assistant
        };
        if message.role != role || message.content.trim().is_empty() {
            return Err("Die Unterhaltung enthält eine ungültige Nachricht.".into());
        }
    }
    Ok(())
}

fn find_qwen(url: &str, key: Option<&str>) -> Option<String> {
    let mut request = loopback_agent()
        .get(&format!("{url}/v1/models"))
        .timeout(Duration::from_secs(2));
    if let Some(key) = key {
        request = request.set("Authorization", &format!("Bearer {key}"));
    }
    let response = read_model_json(request.call().ok()?).ok()?;
    let mut ids = response["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|entry| entry["id"].as_str());
    let file_name = format!("{QWEN_MODEL_ID}.gguf");
    ids.find(|id| {
        let name = id.rsplit(['/', '\\']).next().unwrap_or(id);
        name.eq_ignore_ascii_case(QWEN_MODEL_ID) || name.eq_ignore_ascii_case(&file_name)
    })
    .map(str::to_owned)
}

pub fn suggested_windows_profile() -> ModelProfile {
    let binary = PathBuf::from(LLAMA_BINARY);
    let model = default_windows_model();
    ModelProfile {
        executable: if binary.is_file() {
            LLAMA_BINARY.into()
        } else {
            String::new()
        },
        model: if model.is_file() {
            model.to_string_lossy().into_owned()
        } else {
            String::new()
        },
        ..ModelProfile::default()
    }
}

fn default_windows_model() -> PathBuf {
    std::env::var_os("AIDO_MODEL_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_MODEL))
}

fn windows_paths(profile: &ModelProfile) -> (PathBuf, PathBuf) {
    let binary = if profile.executable.is_empty() {
        PathBuf::from(LLAMA_BINARY)
    } else {
        PathBuf::from(&profile.executable)
    };
    let model = if profile.model.is_empty() {
        default_windows_model()
    } else {
        PathBuf::from(&profile.model)
    };
    (binary, model)
}

fn wsl_executable() -> Result<PathBuf, String> {
    let system_root = std::env::var_os("SystemRoot").ok_or("Windows-Systempfad fehlt.")?;
    Ok(PathBuf::from(system_root).join("System32/wsl.exe"))
}

pub fn suggest_wsl_model_profile() -> Result<ModelProfile, String> {
    let mut command = Command::new(wsl_executable()?);
    hide_console(&mut command);
    let output = command
        .args(["-d", "Ubuntu", "--exec", "/usr/bin/getent", "passwd"])
        .output()
        .map_err(|error| format!("WSL2 ist nicht erreichbar: {error}"))?;
    if !output.status.success() {
        return Err("Ubuntu konnte nicht geöffnet werden. Wähle das Windows-Profil oder trage dein WSL-Profil ein.".into());
    }
    let model = default_windows_model();
    let path = model.to_string_lossy().replace('\\', "/");
    let (drive, rest) = path
        .split_once(':')
        .ok_or("Keine lokale GGUF-Datei gefunden.")?;
    if drive.len() != 1 || !drive.as_bytes()[0].is_ascii_alphabetic() || !model.is_file() {
        return Err("Wähle die Modelldatei für Ubuntu manuell.".into());
    }
    let model = format!("/mnt/{}{rest}", drive.to_ascii_lowercase());
    let passwd = String::from_utf8_lossy(&output.stdout);
    for line in passwd.lines() {
        let fields: Vec<_> = line.split(':').collect();
        if fields.len() != 7
            || !fields[2]
                .parse::<u32>()
                .is_ok_and(|id| (1000..60000).contains(&id))
            || !fields[5].starts_with("/home/")
        {
            continue;
        }
        for executable in [
            format!("{}/.local/share/aido/llama-b12284/llama-server", fields[5]),
            format!("{}/.unsloth/llama.cpp/llama-server", fields[5]),
        ] {
            let profile = ModelProfile {
                backend: ModelBackend::Wsl,
                executable,
                model: model.clone(),
                distro: "Ubuntu".into(),
                user: fields[0].into(),
            };
            if check_model_profile(&profile).is_ok() {
                return Ok(profile);
            }
        }
    }
    Err("AIDO hat unter Ubuntu keinen CUDA-fähigen llama-server gefunden. Wähle das Windows-Profil oder trage die Linux-Pfade ein.".into())
}

fn wsl_command(profile: &ModelProfile) -> Result<Command, String> {
    let identifier = |name: &str| {
        !name.is_empty()
            && name.len() <= 64
            && name
                .bytes()
                .next()
                .is_some_and(|c| c.is_ascii_alphanumeric())
            && name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
    };
    if !identifier(&profile.distro)
        || !identifier(&profile.user)
        || profile.user.eq_ignore_ascii_case("root")
    {
        return Err("Wähle eine WSL2-Distribution und einen Benutzer ohne Root-Rechte.".into());
    }
    if !profile.executable.ends_with("/llama-server")
        || !profile.model.to_ascii_lowercase().ends_with(".gguf")
        || ![&profile.executable, &profile.model]
            .into_iter()
            .all(|path| path.starts_with('/') && !path.contains(['\r', '\n', '\0']))
    {
        return Err("Gib absolute Linux-Pfade für llama-server und das GGUF-Modell an.".into());
    }
    let mut command = Command::new(wsl_executable()?);
    command.args([
        "--distribution",
        &profile.distro,
        "--user",
        &profile.user,
        "--exec",
    ]);
    hide_console(&mut command);
    Ok(command)
}

pub fn check_model_profile(profile: &ModelProfile) -> Result<String, String> {
    if profile.backend == ModelBackend::Windows {
        let (binary, model) = windows_paths(profile);
        if !binary.is_absolute()
            || !model.is_absolute()
            || !binary.is_file()
            || !model.is_file()
            || !binary
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case("llama-server.exe"))
            || !model
                .extension()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case("gguf"))
        {
            return Err("Wähle eine vorhandene llama-server.exe und eine GGUF-Modelldatei.".into());
        }
        return Ok("Windows-Laufzeit und Modelldatei gefunden.".into());
    }
    let output = wsl_command(profile)?
        .args(["/bin/sh", "-c", r#"test "$(id -u)" != 0 && test -x "$1" && test -r "$2" && LD_LIBRARY_PATH="${1%/*}:/usr/lib/wsl/lib" "$1" --list-devices 2>&1"#,
            "aido-probe", &profile.executable, &profile.model])
        .output().map_err(|error| format!("WSL2 ist nicht erreichbar: {error}"))?;
    let devices = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        return Err(
            "WSL konnte den Server oder die GGUF-Datei mit diesem Benutzer nicht lesen.".into(),
        );
    }
    let device = devices.lines().find(|line| line.contains("CUDA") && line.contains("NVIDIA"))
        .ok_or("Die WSL-Laufzeit meldet keine CUDA-GPU. Wähle eine CUDA-fähige Linux-Laufzeit oder das Windows-Profil.")?;
    Ok(device.trim().into())
}

fn hide_console(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
}

fn start_server(port: u16, profile: &ModelProfile) -> Result<OwnedServer, String> {
    let key = uuid::Uuid::new_v4().to_string();
    let port_argument = port.to_string();
    let args = [
        "--host",
        "127.0.0.1",
        "--port",
        &port_argument,
        "--alias",
        QWEN_MODEL_ID,
        "--ctx-size",
        "8192",
        "--parallel",
        "1",
        "--n-gpu-layers",
        "99",
        "--sleep-idle-seconds",
        "300",
    ];
    if profile.backend == ModelBackend::Wsl {
        let mut process = wsl_command(profile)?
            .args([
                "/bin/sh",
                "-c",
                r#"
                test "$(id -u)" != 0 || exit 1
                IFS= read -r LLAMA_API_KEY || exit 1
                export LLAMA_API_KEY
                export LD_LIBRARY_PATH="${1%/*}:/usr/lib/wsl/lib"
                exec 3<&0
                "$@" </dev/null & server=$!
                (cat <&3 >/dev/null; kill -TERM "$server" 2>/dev/null) & lease=$!
                wait "$server"; status=$?
                kill "$lease" 2>/dev/null
                exit "$status"
            "#,
                "aido-model",
                &profile.executable,
                "--model",
                &profile.model,
            ])
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("WSL-Modellstart fehlgeschlagen: {error}"))?;
        let mut lease = process
            .stdin
            .take()
            .ok_or("Der WSL-Server konnte nicht verbunden werden.")?;
        if writeln!(lease, "{key}").is_err() {
            let _ = process.kill();
            let _ = process.wait();
            return Err("Der WSL-Server wurde beim Start beendet.".into());
        }
        return Ok(OwnedServer {
            process,
            key,
            url: format!("http://127.0.0.1:{port}"),
            lease: Some(lease),
            profile: profile.clone(),
            #[cfg(windows)]
            _job: None,
        });
    }
    let (binary, model) = windows_paths(profile);
    if !model.is_file() {
        return Err(format!("Qwen-Modell fehlt unter {}.", model.display()));
    }
    if !binary.is_file() {
        return Err(format!("llama-server fehlt unter {}.", binary.display()));
    }

    let mut command = Command::new(binary);
    for (name, _) in std::env::vars_os() {
        if name
            .to_string_lossy()
            .to_ascii_uppercase()
            .starts_with("LLAMA_ARG_")
        {
            command.env_remove(name);
        }
    }
    command
        .args(args)
        .arg("--model")
        .arg(model)
        .env("LLAMA_API_KEY", &key)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use crate::runtime_supervisor::RuntimeSupervisor;
        let path = std::env::var("PATH").unwrap_or_default();
        command.env("PATH", RuntimeSupervisor::new().build_env_path(&path));
        hide_console(&mut command);
    }
    #[cfg(windows)]
    let job = WindowsJob::new()
        .map_err(|error| format!("AIDOs Modellprozess konnte nicht abgesichert werden: {error}"))?;
    let mut process = command
        .spawn()
        .map_err(|error| format!("Qwen konnte nicht gestartet werden: {error}"))?;
    #[cfg(windows)]
    if let Err(error) = job.assign(&process) {
        let _ = process.kill();
        let _ = process.wait();
        return Err(format!(
            "AIDO konnte den Modellprozess nicht übernehmen: {error}"
        ));
    }
    Ok(OwnedServer {
        process,
        key,
        url: format!("http://127.0.0.1:{port}"),
        lease: None,
        profile: profile.clone(),
        #[cfg(windows)]
        _job: Some(job),
    })
}
