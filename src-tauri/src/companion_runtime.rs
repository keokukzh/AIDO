use crate::ai_client::{
    loopback_agent, read_model_json, ChatMessage, ChatRole, LocalAiClient, LLAMA_URL, QWEN_MODEL_ID,
};
use crate::permission_gateway::{AccessMode, PermissionGateway, PrivacyMode};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
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
}

impl Drop for OwnedServer {
    fn drop(&mut self) {
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
        validate_conversation(messages)?;
        let _request = self
            .request
            .try_lock()
            .map_err(|_| "Eine Anfrage läuft bereits.")?;
        let (url, model_id, key) = self.ensure_server()?;
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

    fn owned(&self) -> MutexGuard<'_, Option<OwnedServer>> {
        self.owned.lock().unwrap_or_else(|error| error.into_inner())
    }

    pub fn shutdown(&self) {
        self.stopping.store(true, Ordering::Release);
        drop(self.owned().take());
    }

    fn ensure_server(&self) -> Result<(String, String, Option<String>), String> {
        let (url, key, newly_started) = {
            let mut owned = self.owned();
            if self.stopping.load(Ordering::Acquire) {
                return Err("AIDO wird beendet.".into());
            }
            if let Some(server) = owned.as_mut() {
                if server
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
                let port = if TcpStream::connect_timeout(
                    &LLAMA_ADDRESS.into(),
                    Duration::from_millis(250),
                )
                .is_ok()
                {
                    let key = std::env::var("AIDO_LOCAL_API_KEY").ok();
                    if let Some(model_id) = find_qwen(LLAMA_URL, key.as_deref()) {
                        return Ok((LLAMA_URL.into(), model_id, key));
                    }
                    // Keep the other service running and reserve a candidate loopback port.
                    TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
                        .and_then(|listener| listener.local_addr())
                        .map_err(|error| format!("Kein freier lokaler Port für Qwen: {error}"))?
                        .port()
                } else {
                    LLAMA_ADDRESS.port()
                };
                let server = start_server(port)?;
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

fn start_server(port: u16) -> Result<OwnedServer, String> {
    let binary = PathBuf::from(LLAMA_BINARY);
    let model = std::env::var_os("AIDO_MODEL_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_MODEL));
    if !model.is_file() {
        return Err(format!("Qwen-Modell fehlt unter {}.", model.display()));
    }
    if !binary.is_file() {
        return Err(format!("llama-server fehlt unter {}.", binary.display()));
    }

    let key = uuid::Uuid::new_v4().to_string();
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
        .args(["--host", "127.0.0.1", "--port"])
        .arg(port.to_string())
        .arg("--model")
        .arg(model)
        .args([
            "--alias",
            QWEN_MODEL_ID,
            "--ctx-size",
            "8192",
            "--parallel",
            "1",
        ])
        .args(["--n-gpu-layers", "99", "--sleep-idle-seconds", "300"])
        .env("LLAMA_API_KEY", &key)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use crate::runtime_supervisor::RuntimeSupervisor;
        use std::os::windows::process::CommandExt;
        let path = std::env::var("PATH").unwrap_or_default();
        command
            .env("PATH", RuntimeSupervisor::new().build_env_path(&path))
            .creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    let process = command
        .spawn()
        .map_err(|error| format!("Qwen konnte nicht gestartet werden: {error}"))?;
    Ok(OwnedServer {
        process,
        key,
        url: format!("http://127.0.0.1:{port}"),
    })
}
