use crate::permission_gateway::{GatewayError, PermissionGateway};
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
            llama_base_url: "http://127.0.0.1:18434".into(),
            whisper_base_url: "http://127.0.0.1:18080".into(),
            model_id: "Qwen3.8-27B-UD-Q4_K_M".into(),
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

        let mut req = ureq::post(&url)
            .set("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(30));
        if let Some(key) = api_key {
            req = req.set("Authorization", &format!("Bearer {}", key));
        }

        match req.send_json(payload) {
            Ok(resp) => {
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
            Err(_) => {
                // Graceful local deterministic explanation if llama-server is currently sleeping/stopped
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
}
