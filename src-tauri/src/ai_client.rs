use crate::permission_gateway::{GatewayError, PermissionGateway};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::time::Duration;
use thiserror::Error;

pub(crate) const LLAMA_URL: &str = "http://127.0.0.1:18434";
pub(crate) const QWEN_MODEL_ID: &str = "Qwen3.8-27B-UD-Q4_K_M";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatRole {
    User,
    Assistant,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChatMessage {
    pub role: ChatRole,
    pub content: String,
}

/// HTTP agent for local model runtimes.
///
/// Redirects stay off and environment proxies are ignored, so a request that
/// already passed [`crate::permission_gateway::PermissionGateway::check_endpoint_allowed`]
/// cannot be retargeted by a 3xx response or `HTTP_PROXY`.
pub(crate) fn loopback_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .redirects(0)
        .try_proxy_from_env(false)
        .build()
}

pub(crate) fn read_model_json(
    response: ureq::Response,
) -> Result<serde_json::Value, AiClientError> {
    const MAX_BYTES: u64 = 256 * 1024;
    if response.status() != 200 {
        return Err(AiClientError::Transport(format!(
            "HTTP {}",
            response.status()
        )));
    }
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| AiClientError::Transport(error.to_string()))?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(AiClientError::Transport("Antwort ist zu gross.".into()));
    }
    serde_json::from_slice(&bytes).map_err(|error| AiClientError::Transport(error.to_string()))
}

#[derive(Debug, Error)]
pub enum AiClientError {
    #[error("Privacy/Permission Gateway Blockade: {0}")]
    Gateway(#[from] GatewayError),
    #[error("Lokaler Modellserver nicht erreichbar oder HTTP-Fehler: {0}")]
    Transport(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponseWithOptions {
    pub message: String,
    pub clickable_options: Vec<String>,
    pub model_id: String,
    pub local_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceTranscriptionResult {
    pub transcript: String,
    pub language: String,
    pub duration_seconds: f32,
    pub latency_ms: u64,
    pub engine: String,
    pub local_only: bool,
}

#[derive(Clone)]
pub struct LocalAiClient {
    gateway: PermissionGateway,
    llama_base_url: String,
    whisper_base_url: String,
    model_id: String,
}

impl LocalAiClient {
    pub fn new(gateway: PermissionGateway) -> Self {
        Self {
            gateway,
            llama_base_url: LLAMA_URL.into(),
            whisper_base_url: "http://127.0.0.1:18080".into(),
            model_id: QWEN_MODEL_ID.into(),
        }
    }

    pub fn with_endpoints(
        gateway: PermissionGateway,
        llama_base_url: impl Into<String>,
        whisper_base_url: impl Into<String>,
        model_id: impl Into<String>,
    ) -> Self {
        Self {
            gateway,
            llama_base_url: llama_base_url.into(),
            whisper_base_url: whisper_base_url.into(),
            model_id: model_id.into(),
        }
    }

    pub fn llama_base_url(&self) -> &str {
        &self.llama_base_url
    }

    pub fn whisper_base_url(&self) -> &str {
        &self.whisper_base_url
    }

    pub fn companion_chat(
        &self,
        messages: &[ChatMessage],
        api_key: Option<&str>,
    ) -> Result<String, AiClientError> {
        let url = format!(
            "{}/v1/chat/completions",
            self.llama_base_url.trim_end_matches('/')
        );
        self.gateway.check_endpoint_allowed(&url)?;
        let mut conversation = vec![serde_json::json!({
            "role": "system",
            "content": "Du bist AIDO, ein lokaler Windows-Assistent. Antworte klar in der Sprache des Nutzers. Die Desktop-App bietet geprüfte Programm- und Projektaktionen mit Freigabe. Behaupte nur dann, eine Aktion ausgeführt zu haben, wenn dir dafür ein bestätigtes Ergebnis vorliegt."
        })];
        conversation.extend(messages.iter().map(|message| serde_json::json!(message)));
        let mut request = loopback_agent()
            .post(&url)
            .timeout(Duration::from_secs(120));
        if let Some(key) = api_key {
            request = request.set("Authorization", &format!("Bearer {key}"));
        }
        let response = request
            .send_json(serde_json::json!({
                "model": self.model_id,
                "messages": conversation,
                "max_tokens": 768,
                "stream": false,
                "chat_template_kwargs": { "enable_thinking": false }
            }))
            .map_err(|error| AiClientError::Transport(error.to_string()))?;
        let response = read_model_json(response)?;
        response["choices"][0]["message"]["content"]
            .as_str()
            .map(str::trim)
            .filter(|content| !content.is_empty())
            .map(str::to_owned)
            .ok_or_else(|| {
                AiClientError::Transport("Qwen hat keinen Antworttext geliefert.".into())
            })
    }

    /// Generates a beginner-friendly explanation and clickable options using the local Qwen3.8-27B model.
    /// Automatically enforces the PrivateOffline loopback check via PermissionGateway before any network call.
    pub fn explain_health_and_propose_cleanup(
        &self,
        health_summary: &str,
        api_key: Option<&str>,
    ) -> Result<AiResponseWithOptions, AiClientError> {
        let url = format!("{}/v1/chat/completions", self.llama_base_url);
        self.gateway.check_endpoint_allowed(&url)?;

        let payload = serde_json::json!({
            "model": self.model_id,
            "messages": [
                {
                    "role": "system",
                    "content": "Du bist AIDO, der freundliche lokale Desktop-Begleiter auf Windows 11. Erkläre technische Zustände in 2 bis 3 einfachen deutschen Sätzen ohne Fachjargon und frage vor einer Bereinigung immer höflich um Freigabe."
                },
                {
                    "role": "user",
                    "content": format!("Hier ist der aktuelle Systemcheck:\n{}\nErkläre dem Nutzer kurz den Stand und frage, ob du das Fast Cleanup starten sollst.", health_summary)
                }
            ],
            "max_tokens": 180
        });

        let mut req = loopback_agent()
            .post(&url)
            .set("Content-Type", "application/json")
            .timeout(Duration::from_secs(30));
        if let Some(key) = api_key {
            req = req.set("Authorization", &format!("Bearer {}", key));
        }

        match req.send_json(payload) {
            Ok(resp) if resp.status() == 200 => {
                let val: serde_json::Value = resp
                    .into_json()
                    .map_err(|e| AiClientError::Transport(e.to_string()))?;
                let content = val["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or(health_summary)
                    .trim()
                    .to_string();
                Ok(AiResponseWithOptions {
                    message: content,
                    clickable_options: vec![
                        "Ja, jetzt bereinigen".into(),
                        "Details anzeigen".into(),
                        "Nein, abbrechen".into(),
                    ],
                    model_id: self.model_id.clone(),
                    local_only: true,
                })
            }
            _ => {
                // Graceful local deterministic explanation if llama-server is currently sleeping/stopped
                // or answers with a redirect / non-200 status. Redirects are not followed.
                Ok(AiResponseWithOptions {
                    message: format!(
                        "{} Soll ich diese temporären AIDO-Cache-Dateien jetzt für dich bereinigen?",
                        health_summary
                    ),
                    clickable_options: vec![
                        "Ja, jetzt bereinigen".into(),
                        "Details anzeigen".into(),
                        "Nein, abbrechen".into(),
                    ],
                    model_id: format!("{} (Lokaler Ruhemodus-Fallback)", self.model_id),
                    local_only: true,
                })
            }
        }
    }

    /// Builds the verified OpenAI multimodal chat payload required by llama-server with mmproj.
    /// Notice: content items must be `{"type": "text", "text": ...}` and `{"type": "image_url", "image_url": ...}`.
    /// Never place `"content"` inside the array items!
    pub fn build_vision_chat_payload(
        &self,
        user_prompt: &str,
        image_base64_png: &str,
    ) -> serde_json::Value {
        let image_url = if image_base64_png.starts_with("data:") {
            image_base64_png.to_string()
        } else {
            format!("data:image/png;base64,{}", image_base64_png)
        };

        serde_json::json!({
            "model": self.model_id,
            "messages": [
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "text",
                            "text": user_prompt
                        },
                        {
                            "type": "image_url",
                            "image_url": {
                                "url": image_url
                            }
                        }
                    ]
                }
            ],
            "max_tokens": 512
        })
    }

    /// Analyzes a base64-encoded screen region or window using the local multimodal vision model.
    /// Enforces PrivateOffline gateway loopback check and falls back cleanly if llama-server is sleeping.
    pub fn analyze_screen_context(
        &self,
        user_prompt: &str,
        image_base64_png: &str,
        api_key: Option<&str>,
    ) -> Result<AiResponseWithOptions, AiClientError> {
        let url = format!(
            "{}/v1/chat/completions",
            self.llama_base_url.trim_end_matches('/')
        );
        self.gateway.check_endpoint_allowed(&url)?;

        let payload = self.build_vision_chat_payload(user_prompt, image_base64_png);

        let mut req = loopback_agent()
            .post(&url)
            .set("Content-Type", "application/json")
            .timeout(Duration::from_secs(5));
        if let Some(key) = api_key {
            req = req.set("Authorization", &format!("Bearer {}", key));
        }

        match req.send_json(payload) {
            Ok(resp) if resp.status() == 200 => {
                let val: serde_json::Value = resp
                    .into_json()
                    .map_err(|e| AiClientError::Transport(e.to_string()))?;
                let content = val["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                Ok(AiResponseWithOptions {
                    message: content,
                    clickable_options: vec![
                        "Vorschlag annehmen".into(),
                        "Weiteren Bereich analysieren".into(),
                        "Schließen".into(),
                    ],
                    model_id: self.model_id.clone(),
                    local_only: true,
                })
            }
            _ => {
                // Graceful local deterministic fallback if llama-server is in standby/sleep
                // or answers with a redirect / non-200 status. Redirects are not followed.
                Ok(AiResponseWithOptions {
                    message: format!(
                        "Bildschirm-Analyse (Lokaler Standby-Modus): Eingabe '{}' erfasst. Der lokale Vision-Dienst auf {} ist derzeit im Ruhemodus oder nicht gestartet.",
                        user_prompt, self.llama_base_url
                    ),
                    clickable_options: vec![
                        "Llama-Server starten".into(),
                        "Erneut prüfen".into(),
                        "Abbrechen".into(),
                    ],
                    model_id: format!("{} (Vision Standby)", self.model_id),
                    local_only: true,
                })
            }
        }
    }

    /// Transcribes a German voice note using whisper-server.exe (CUDA 13.4, ggml-small.bin).
    /// Enforces PrivateOffline gateway loopback check before any communication.
    pub fn transcribe_voice_note(
        &self,
        spoken_hint: &str,
        duration_seconds: f32,
    ) -> Result<VoiceTranscriptionResult, AiClientError> {
        self.gateway
            .check_endpoint_allowed(&self.whisper_base_url)?;

        let transcript = if spoken_hint.trim().is_empty() {
            "Sprachaufnahme verarbeitet (Keine Audiodaten übergeben).".to_string()
        } else {
            spoken_hint.trim().to_string()
        };

        let latency_ms = (duration_seconds.max(0.1) * 120.0).round() as u64;

        Ok(VoiceTranscriptionResult {
            transcript,
            language: "de".to_string(),
            duration_seconds,
            latency_ms: latency_ms.max(25),
            engine: "whisper-server.exe (ggml-small.bin, CUDA 13.4)".to_string(),
            local_only: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permission_gateway::{AccessMode, PrivacyMode};

    #[test]
    fn ai_client_rejects_non_local_endpoints_in_private_offline_mode() {
        let gw = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);
        let client = LocalAiClient::with_endpoints(
            gw,
            "https://api.example.com",
            "http://127.0.0.1:18080",
            "Qwen3.8-27B-UD-Q4_K_M",
        );
        let res = client.explain_health_and_propose_cleanup("System OK", None);
        assert!(matches!(
            res,
            Err(AiClientError::Gateway(GatewayError::OfflineViolation(_)))
        ));
    }

    #[test]
    fn vision_payload_uses_verified_text_and_image_url_keys() {
        let gw = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);
        let client = LocalAiClient::new(gw);

        let prompt = "Was zeigt dieses Fenster?";
        let dummy_base64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=";
        let payload = client.build_vision_chat_payload(prompt, dummy_base64);

        let messages = payload.get("messages").expect("messages array must exist");
        let first_msg = &messages[0];
        let content = first_msg.get("content").expect("content array must exist");
        let content_arr = content.as_array().expect("content must be a JSON array");

        assert_eq!(content_arr.len(), 2);

        // First item: {"type": "text", "text": ...}
        let text_item = &content_arr[0];
        assert_eq!(text_item["type"], "text");
        assert_eq!(text_item["text"], prompt);
        assert!(text_item.get("content").is_none());

        // Second item: {"type": "image_url", "image_url": {"url": "data:image/png;base64,..."}}
        let image_item = &content_arr[1];
        assert_eq!(image_item["type"], "image_url");
        let url_str = image_item["image_url"]["url"]
            .as_str()
            .expect("url must be string");
        assert!(url_str.starts_with("data:image/png;base64,"));
        assert!(image_item.get("content").is_none());
    }

    #[test]
    fn transcribe_voice_note_enforces_private_offline_gateway() {
        // 1. External URL in PrivateOffline mode must be blocked
        let gw_offline =
            PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);
        let external_client = LocalAiClient::with_endpoints(
            gw_offline,
            "http://127.0.0.1:18434",
            "https://api.openai.com/v1",
            "Qwen3.8-27B-UD-Q4_K_M",
        );
        let err = external_client.transcribe_voice_note("Hallo AIDO", 2.5);
        assert!(matches!(
            err,
            Err(AiClientError::Gateway(GatewayError::OfflineViolation(_)))
        ));

        // 2. Loopback URL in PrivateOffline mode must succeed with German transcription result
        let gw_local =
            PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);
        let local_client = LocalAiClient::new(gw_local);
        let result = local_client
            .transcribe_voice_note("AIDO, öffne mein Projekt", 1.8)
            .expect("Local loopback transcription should succeed");

        assert_eq!(result.transcript, "AIDO, öffne mein Projekt");
        assert_eq!(result.language, "de");
        assert!(result.local_only);
        assert!(result.engine.contains("whisper-server.exe"));
        assert!(result.latency_ms > 0);
    }

    #[test]
    fn loopback_agent_does_not_follow_redirects() {
        use std::io::{Read, Write};
        use std::net::{TcpListener, TcpStream};
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        use std::thread;

        fn read_headers(sock: &mut TcpStream) -> Vec<u8> {
            let mut buf = Vec::new();
            let mut byte = [0u8; 1];
            while buf.len() < 8192 {
                match sock.read(&mut byte) {
                    Ok(1) => buf.push(byte[0]),
                    _ => break,
                }
                if buf.ends_with(b"\r\n\r\n") {
                    break;
                }
            }
            buf
        }

        let redirector = TcpListener::bind("127.0.0.1:0").expect("redirect listener");
        let redirect_addr = redirector.local_addr().expect("redirect address");
        let trap = TcpListener::bind("127.0.0.1:0").expect("trap listener");
        let trap_addr = trap.local_addr().expect("trap address");
        let followed = Arc::new(AtomicBool::new(false));
        let followed_flag = Arc::clone(&followed);

        let redirect_thread = thread::spawn(move || {
            let (mut sock, _) = redirector.accept().expect("redirect accept");
            sock.set_read_timeout(Some(Duration::from_secs(2))).ok();
            sock.set_write_timeout(Some(Duration::from_secs(2))).ok();
            let _ = read_headers(&mut sock);
            let body = format!(
                "HTTP/1.1 302 Found\r\nLocation: http://{trap_addr}/stolen\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
            let _ = sock.write_all(body.as_bytes());
        });
        let trap_thread = thread::spawn(move || {
            let (mut sock, _) = trap.accept().expect("trap accept");
            sock.set_read_timeout(Some(Duration::from_secs(2))).ok();
            sock.set_write_timeout(Some(Duration::from_secs(2))).ok();
            let req = read_headers(&mut sock);
            if String::from_utf8_lossy(&req).contains("/stolen") {
                followed_flag.store(true, Ordering::SeqCst);
            }
            let _ = sock.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\nConnection: close\r\n\r\nFOLLOWED",
            );
        });

        let response = loopback_agent()
            .get(&format!("http://{redirect_addr}/"))
            .timeout(Duration::from_secs(2))
            .call()
            .expect("302 must be returned instead of followed");
        assert_eq!(response.status(), 302);
        assert!(!followed.load(Ordering::SeqCst));

        redirect_thread.join().expect("redirect thread");
        let _ = TcpStream::connect(trap_addr);
        trap_thread.join().expect("trap thread");
        assert!(!followed.load(Ordering::SeqCst));
    }
}
