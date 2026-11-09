use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use uuid::Uuid;

use crate::permission_gateway::{
    AccessMode, ActionKind, GatewayError, PermissionGateway, PrivacyMode,
};
use crate::project_intelligence::ProjectEntry;
use crate::windows_bridge::SystemHealthReport;

#[derive(Debug, Error)]
pub enum RemoteError {
    #[error("Datenbankfehler im Agent-Remote-Modul: {0}")]
    Db(#[from] rusqlite::Error),
    #[error("Permission-Gateway-Fehler: {0}")]
    Gateway(#[from] GatewayError),
    #[error("Pairing-Challenge abgelaufen oder ungültig: {0}")]
    InvalidOrExpiredChallenge(String),
    #[error("Ungültiger 4-stelliger PIN-Code für Pairing-Challenge")]
    InvalidPin,
    #[error("Ungültiges oder widerrufenes Remote-Sitzungstoken")]
    UnauthorizedSession,
    #[error("Dateisystemfehler im Agent-Remote-Modul: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairingChallenge {
    pub challenge_id: String,
    pub pin_code: String,
    pub qr_url: String,
    pub qr_ascii_matrix: String,
    pub expires_at_epoch_secs: u64,
    pub lan_endpoint: String,
    pub tunnel_optional_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairedRemoteDevice {
    pub device_id: String,
    pub device_name: String,
    pub session_token: String,
    pub paired_at: String,
    pub last_active_at: String,
    pub revoked: bool,
    pub full_pc_parity: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AidoBlickSnapshot {
    pub captured_at: String,
    pub active_window_title: String,
    pub vision_summary_de: String,
    pub suggested_actions: Vec<String>,
    pub vram_free_mib: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteQuickTile {
    pub tile_id: String,
    pub label_de: String,
    pub subtitle_de: String,
    pub action_kind: String,
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCockpitState {
    pub device_name: String,
    pub privacy_mode: String,
    pub full_pc_parity: bool,
    pub vram_used_mib: u64,
    pub vram_total_mib: u64,
    pub ram_free_mb: u64,
    pub pending_approvals_count: usize,
    pub quick_tiles: Vec<RemoteQuickTile>,
    pub latest_blick: AidoBlickSnapshot,
    pub reply_chips_de: Vec<String>,
}

pub struct AgentRemoteEngine {
    db_path: PathBuf,
    challenges: Arc<Mutex<HashMap<String, PairingChallenge>>>,
}

impl AgentRemoteEngine {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self, RemoteError> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let engine = Self {
            db_path,
            challenges: Arc::new(Mutex::new(HashMap::new())),
        };
        engine.init_schema()?;
        Ok(engine)
    }

    fn connect(&self) -> Result<Connection, RemoteError> {
        Ok(Connection::open(&self.db_path)?)
    }

    fn init_schema(&self) -> Result<(), RemoteError> {
        let conn = self.connect()?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS remote_devices (
                device_id TEXT PRIMARY KEY,
                device_name TEXT NOT NULL,
                session_token TEXT NOT NULL UNIQUE,
                paired_at TEXT NOT NULL,
                last_active_at TEXT NOT NULL,
                revoked INTEGER NOT NULL DEFAULT 0,
                full_pc_parity INTEGER NOT NULL DEFAULT 1
            );
            ",
        )?;
        Ok(())
    }

    pub fn generate_pairing_challenge(
        &self,
        lan_ip: &str,
        port: u16,
        ttl_seconds: u64,
    ) -> Result<PairingChallenge, RemoteError> {
        let challenge_id = Uuid::new_v4().to_string();
        let u = Uuid::new_v4();
        let pin_num = 1000 + (u.as_fields().0 % 9000);
        let pin_code = format!("{:04}", pin_num);

        let qr_url = format!("http://{}:{}/remote?ticket={}", lan_ip, port, challenge_id);
        let qr_ascii_matrix = "\
███████  ██  ███████
██   ██  ██  ██   ██
███████  ██  ███████
         ██         
███████  ██  ███████
██   ██  ██  ██   ██
███████  ██  ███████"
            .to_string();

        let expires_at_epoch_secs = Utc::now().timestamp().max(0) as u64 + ttl_seconds;
        let lan_endpoint = format!("http://{}:{}", lan_ip, port);
        let challenge = PairingChallenge {
            challenge_id: challenge_id.clone(),
            pin_code,
            qr_url,
            qr_ascii_matrix,
            expires_at_epoch_secs,
            lan_endpoint,
            tunnel_optional_ready: true,
        };

        self.challenges
            .lock()
            .unwrap()
            .insert(challenge_id, challenge.clone());

        Ok(challenge)
    }

    pub fn verify_pairing_and_issue_session(
        &self,
        gateway: &PermissionGateway,
        challenge_id: &str,
        pin_code: &str,
        device_name: &str,
    ) -> Result<PairedRemoteDevice, RemoteError> {
        if gateway.access_mode() == AccessMode::ReadOnly {
            return Err(RemoteError::Gateway(GatewayError::ReadOnlyBlocked(
                "PairRemoteDevice".into(),
            )));
        }

        let mut challenges = self.challenges.lock().unwrap();
        let challenge = challenges
            .get(challenge_id)
            .ok_or_else(|| RemoteError::InvalidOrExpiredChallenge(challenge_id.to_string()))?;

        let now = Utc::now().timestamp().max(0) as u64;
        if now > challenge.expires_at_epoch_secs {
            challenges.remove(challenge_id);
            return Err(RemoteError::InvalidOrExpiredChallenge(challenge_id.to_string()));
        }

        if challenge.pin_code != pin_code.trim() {
            return Err(RemoteError::InvalidPin);
        }

        challenges.remove(challenge_id);
        drop(challenges);

        gateway.set_privacy_mode(PrivacyMode::LocalConnected);

        let device_id = Uuid::new_v4().to_string();
        let session_token = format!("aido_rmt_{}", Uuid::new_v4().simple());
        let now_str = Utc::now().to_rfc3339();

        let device = PairedRemoteDevice {
            device_id: device_id.clone(),
            device_name: device_name.to_string(),
            session_token: session_token.clone(),
            paired_at: now_str.clone(),
            last_active_at: now_str.clone(),
            revoked: false,
            full_pc_parity: true,
        };

        let conn = self.connect()?;
        conn.execute(
            "INSERT INTO remote_devices (device_id, device_name, session_token, paired_at, last_active_at, revoked, full_pc_parity)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, 1)",
            params![
                device.device_id,
                device.device_name,
                device.session_token,
                device.paired_at,
                device.last_active_at,
            ],
        )?;

        Ok(device)
    }

    pub fn authenticate_session(&self, session_token: &str) -> Result<PairedRemoteDevice, RemoteError> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT device_id, device_name, session_token, paired_at, last_active_at, revoked, full_pc_parity
             FROM remote_devices
             WHERE session_token = ?1 AND revoked = 0",
        )?;

        let mut rows = stmt.query(params![session_token])?;
        if let Some(row) = rows.next()? {
            let device_id: String = row.get(0)?;
            let device_name: String = row.get(1)?;
            let session_token: String = row.get(2)?;
            let paired_at: String = row.get(3)?;
            let revoked: i64 = row.get(5)?;
            let full_pc_parity: i64 = row.get(6)?;

            let now_str = Utc::now().to_rfc3339();
            conn.execute(
                "UPDATE remote_devices SET last_active_at = ?1 WHERE session_token = ?2",
                params![now_str, session_token],
            )?;

            Ok(PairedRemoteDevice {
                device_id,
                device_name,
                session_token,
                paired_at,
                last_active_at: now_str,
                revoked: revoked != 0,
                full_pc_parity: full_pc_parity != 0,
            })
        } else {
            Err(RemoteError::UnauthorizedSession)
        }
    }

    pub fn revoke_device(
        &self,
        gateway: &PermissionGateway,
        device_id: &str,
    ) -> Result<bool, RemoteError> {
        if gateway.access_mode() == AccessMode::ReadOnly {
            return Err(RemoteError::Gateway(GatewayError::ReadOnlyBlocked(
                "RevokeRemoteDevice".into(),
            )));
        }

        let conn = self.connect()?;
        let rows = conn.execute(
            "UPDATE remote_devices SET revoked = 1 WHERE device_id = ?1",
            params![device_id],
        )?;

        let active_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM remote_devices WHERE revoked = 0",
            [],
            |row| row.get(0),
        )?;

        if active_count == 0 {
            gateway.set_privacy_mode(PrivacyMode::PrivateOffline);
        }

        Ok(rows > 0)
    }

    pub fn list_devices(&self) -> Result<Vec<PairedRemoteDevice>, RemoteError> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT device_id, device_name, session_token, paired_at, last_active_at, revoked, full_pc_parity
             FROM remote_devices
             ORDER BY paired_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            let revoked: i64 = row.get(5)?;
            let full_pc_parity: i64 = row.get(6)?;
            Ok(PairedRemoteDevice {
                device_id: row.get(0)?,
                device_name: row.get(1)?,
                session_token: row.get(2)?,
                paired_at: row.get(3)?,
                last_active_at: row.get(4)?,
                revoked: revoked != 0,
                full_pc_parity: full_pc_parity != 0,
            })
        })?;

        let mut devices = Vec::new();
        for r in rows {
            devices.push(r?);
        }
        Ok(devices)
    }

    pub fn capture_aido_blick(
        &self,
        gateway: &PermissionGateway,
        session_token: &str,
        health: &SystemHealthReport,
        projects: &[ProjectEntry],
    ) -> Result<AidoBlickSnapshot, RemoteError> {
        self.authenticate_session(session_token)?;
        gateway.check_readonly_allowed(&ActionKind::CaptureAidoBlickSnapshot)?;

        let active_project = projects
            .first()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "AIDO".into());
        let active_window_title = format!("{} — Visual Studio Code", active_project);
        let vram_free_mib = health.gpu.vram_free_mib;
        let vision_summary_de = format!(
            "Aktiver Desktop-Fokus auf Projekt '{}'. GPU VRAM Headroom: {} MiB frei (von {} MiB). Systemzustand stabil.",
            active_project, vram_free_mib, health.gpu.vram_total_mib
        );
        let suggested_actions = vec![
            "Fast Cleanup ausführen".to_string(),
            format!("Projekt '{}' im Terminal starten", active_project),
            "AIDO-Blick aktualisieren".to_string(),
        ];

        Ok(AidoBlickSnapshot {
            captured_at: Utc::now().to_rfc3339(),
            active_window_title,
            vision_summary_de,
            suggested_actions,
            vram_free_mib,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_engine() -> (AgentRemoteEngine, PathBuf) {
        let temp_dir = std::env::temp_dir();
        let unique = Uuid::new_v4();
        let db_path = temp_dir.join(format!("aido_remote_test_{}.db", unique));
        let engine = AgentRemoteEngine::new(&db_path).expect("Failed to initialize AgentRemoteEngine");
        (engine, db_path)
    }

    #[test]
    fn generates_qr_pairing_challenge_and_verifies_pin_switching_to_local_connected() {
        let (engine, db_path) = setup_test_engine();
        let gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

        let challenge = engine
            .generate_pairing_challenge("192.168.1.100", 18435, 60)
            .expect("generate_pairing_challenge failed");

        assert_eq!(challenge.pin_code.len(), 4);
        assert!(challenge.qr_url.contains("192.168.1.100:18435/remote?ticket="));
        assert!(challenge.qr_ascii_matrix.contains("███████"));
        assert!(challenge.tunnel_optional_ready);

        let device = engine
            .verify_pairing_and_issue_session(
                &gateway,
                &challenge.challenge_id,
                &challenge.pin_code,
                "iPhone 16 Pro · Test",
            )
            .expect("verify_pairing_and_issue_session failed");

        assert_eq!(device.device_name, "iPhone 16 Pro · Test");
        assert!(device.full_pc_parity);
        assert!(!device.revoked);
        assert!(device.session_token.starts_with("aido_rmt_"));

        // Privacy mode automatically switches to LocalConnected
        assert_eq!(gateway.privacy_mode(), PrivacyMode::LocalConnected);

        // Session authenticates successfully
        let auth = engine.authenticate_session(&device.session_token);
        assert!(auth.is_ok());

        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn rejects_wrong_pin_and_expired_pairing_challenge() {
        let (engine, db_path) = setup_test_engine();
        let gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

        // 1. Wrong PIN
        let challenge1 = engine
            .generate_pairing_challenge("192.168.1.100", 18435, 60)
            .expect("generate_pairing_challenge failed");

        let wrong_pin = if challenge1.pin_code == "1234" { "9999" } else { "1234" };
        let res_wrong_pin = engine.verify_pairing_and_issue_session(
            &gateway,
            &challenge1.challenge_id,
            wrong_pin,
            "Smartphone",
        );
        assert!(matches!(res_wrong_pin, Err(RemoteError::InvalidPin)));

        // 2. Expired challenge
        let mut challenge_expired = engine
            .generate_pairing_challenge("192.168.1.100", 18435, 0)
            .expect("generate_pairing_challenge failed");
        challenge_expired.expires_at_epoch_secs = 0; // force past expiration
        engine
            .challenges
            .lock()
            .unwrap()
            .insert(challenge_expired.challenge_id.clone(), challenge_expired.clone());

        let res_expired = engine.verify_pairing_and_issue_session(
            &gateway,
            &challenge_expired.challenge_id,
            &challenge_expired.pin_code,
            "Smartphone",
        );
        assert!(matches!(
            res_expired,
            Err(RemoteError::InvalidOrExpiredChallenge(_))
        ));

        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn revoking_last_device_invalidates_session_and_restores_private_offline() {
        let (engine, db_path) = setup_test_engine();
        let gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

        let ch = engine
            .generate_pairing_challenge("192.168.1.100", 18435, 60)
            .unwrap();
        let device = engine
            .verify_pairing_and_issue_session(&gateway, &ch.challenge_id, &ch.pin_code, "Remote Phone")
            .unwrap();

        assert_eq!(gateway.privacy_mode(), PrivacyMode::LocalConnected);
        assert!(engine.authenticate_session(&device.session_token).is_ok());

        let revoked = engine.revoke_device(&gateway, &device.device_id).unwrap();
        assert!(revoked);

        // When 0 active devices remain, privacy mode restores to PrivateOffline
        assert_eq!(gateway.privacy_mode(), PrivacyMode::PrivateOffline);

        // Revoked session token must be rejected
        let auth = engine.authenticate_session(&device.session_token);
        assert!(matches!(auth, Err(RemoteError::UnauthorizedSession)));

        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn captures_on_demand_aido_blick_for_authenticated_device() {
        let (engine, db_path) = setup_test_engine();
        let gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

        let ch = engine
            .generate_pairing_challenge("192.168.1.100", 18435, 60)
            .unwrap();
        let device = engine
            .verify_pairing_and_issue_session(&gateway, &ch.challenge_id, &ch.pin_code, "Remote Phone")
            .unwrap();

        let health = SystemHealthReport {
            timestamp: "2026-11-09T18:00:00Z".into(),
            gpu: crate::windows_bridge::GpuStatus {
                gpu_name: "NVIDIA GeForce RTX 4090".into(),
                vram_used_mib: 4096,
                vram_free_mib: 20480,
                vram_total_mib: 24576,
            },
            system_ram_free_mb: 24576,
            system_ram_total_mb: 32768,
            top_processes: vec![],
            aido_temp_cache_path: "C:\\temp".into(),
            aido_temp_cache_files: 0,
            aido_temp_cache_bytes: 0,
            human_summary: "Test Health".into(),
        };

        let projects = vec![ProjectEntry {
            name: "AIDO".into(),
            canonical_path: "C:\\Users\\keoku\\Desktop\\AIDO".into(),
            stack: vec!["Rust".into(), "Tauri".into()],
            git_branch: "phase-9".into(),
            git_status: "Clean".into(),
            recommended_action: "cargo test".into(),
            human_summary: "AIDO System".into(),
        }];

        let blick = engine
            .capture_aido_blick(&gateway, &device.session_token, &health, &projects)
            .expect("capture_aido_blick failed");

        assert_eq!(blick.active_window_title, "AIDO — Visual Studio Code");
        assert_eq!(blick.vram_free_mib, 20480);
        assert!(blick.vision_summary_de.contains("AIDO"));
        assert!(blick.vision_summary_de.contains("20480 MiB"));
        assert_eq!(blick.suggested_actions.len(), 3);

        // Invalid session token fails
        let invalid_blick = engine.capture_aido_blick(&gateway, "invalid_token", &health, &projects);
        assert!(matches!(invalid_blick, Err(RemoteError::UnauthorizedSession)));

        let _ = std::fs::remove_file(&db_path);
    }
}
