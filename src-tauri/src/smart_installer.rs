use crate::permission_gateway::{ActionKind, GatewayError, PermissionGateway};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InstallerError {
    #[error("SQLite-Fehler: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Dateisystem-Fehler: {0}")]
    Io(#[from] std::io::Error),
    #[error("Gateway-Fehler: {0}")]
    Gateway(#[from] GatewayError),
    #[error("Ungültiger Installationsschritt: {0}")]
    InvalidStep(u8),
    #[error("Installationsfehler: {0}")]
    Other(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareTier {
    HighEndGpu,
    MidRangeGpu,
    CpuFocused,
    LowSpecCloudFallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelReuseStatus {
    pub role: String,
    pub model_name: String,
    pub expected_path: String,
    pub exists_locally: bool,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallerEtaForecast {
    pub ready_to_wake_seconds: u64,
    pub personal_setup_seconds: u64,
    pub remaining_download_bytes: u64,
    pub reused_models_count: usize,
    pub selected_apps_count: usize,
    pub extended_integration_note: String,
    pub human_summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallerStepStatus {
    Pending,
    Completed,
    Paused,
    Failed,
    RolledBack,
}

impl InstallerStepStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            InstallerStepStatus::Pending => "Pending",
            InstallerStepStatus::Completed => "Completed",
            InstallerStepStatus::Paused => "Paused",
            InstallerStepStatus::Failed => "Failed",
            InstallerStepStatus::RolledBack => "RolledBack",
        }
    }
}

impl std::str::FromStr for InstallerStepStatus {
    type Err = InstallerError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(InstallerStepStatus::Pending),
            "Completed" => Ok(InstallerStepStatus::Completed),
            "Paused" => Ok(InstallerStepStatus::Paused),
            "Failed" => Ok(InstallerStepStatus::Failed),
            "RolledBack" => Ok(InstallerStepStatus::RolledBack),
            other => Err(InstallerError::Other(format!("Unbekannter InstallerStepStatus: {}", other))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallerStepRecord {
    pub step_index: u8,
    pub step_key: String,
    pub title_de: String,
    pub status: InstallerStepStatus,
    pub rollback_action: String,
    pub details: String,
}

pub struct SmartInstallerEngine {
    db_path: PathBuf,
}

impl SmartInstallerEngine {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self, InstallerError> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let engine = Self { db_path };
        engine.init_schema()?;
        Ok(engine)
    }

    fn connect(&self) -> Result<Connection, InstallerError> {
        Ok(Connection::open(&self.db_path)?)
    }

    fn init_schema(&self) -> Result<(), InstallerError> {
        let conn = self.connect()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS installer_steps (
                step_index INTEGER PRIMARY KEY,
                step_key TEXT NOT NULL UNIQUE,
                title_de TEXT NOT NULL,
                status TEXT NOT NULL,
                rollback_action TEXT NOT NULL,
                details TEXT NOT NULL
            );",
        )?;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM installer_steps",
            [],
            |row| row.get(0),
        )?;

        if count == 0 {
            let default_steps = [
                (
                    1u8,
                    "welcome_privacy",
                    "Willkommen & Datenschutz",
                    "Datenschutzbestätigung zurücksetzen",
                ),
                (
                    2u8,
                    "hardware_check",
                    "Hardware automatisch erkennen",
                    "Hardware-Erkennungsprofil verwerfen",
                ),
                (
                    3u8,
                    "access_profile",
                    "Berechtigungsmodus wählen",
                    "Gewähltes Zugriffsprofil auf Standard zurücksetzen",
                ),
                (
                    4u8,
                    "ai_setup",
                    "KI automatisch einrichten",
                    "KI-Modell-Pfade und LLM-Slot-Registrierungen deregistrieren (Modell-Dateien auf Festplatte unberührt lassen)",
                ),
                (
                    5u8,
                    "app_discovery",
                    "Apps entdecken",
                    "Erkannte App-Liste zurücksetzen",
                ),
                (
                    6u8,
                    "app_integration",
                    "Apps intelligent integrieren",
                    "Generierte Adapter und Tool-Registrierungen deregistrieren",
                ),
                (
                    7u8,
                    "knowledge_index",
                    "Persönliches Wissen aufbauen",
                    "Temporäre Index-Caches leeren",
                ),
                (
                    8u8,
                    "system_validation",
                    "System validieren",
                    "Validierungsergebnisse und Prüfberichte verwerfen",
                ),
                (
                    9u8,
                    "aido_awakens",
                    "AIDO erwacht",
                    "Autostart-Dienst stoppen und Bereitschaftsstatus zurücknehmen",
                ),
            ];

            let mut stmt = conn.prepare(
                "INSERT INTO installer_steps (step_index, step_key, title_de, status, rollback_action, details)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;

            for (idx, key, title, rollback) in default_steps {
                stmt.execute(params![
                    idx as i64,
                    key,
                    title,
                    InstallerStepStatus::Pending.as_str(),
                    rollback,
                    "",
                ])?;
            }
        }

        Ok(())
    }

    pub fn classify_hardware(vram_total_mib: u64, ram_total_mb: u64) -> HardwareTier {
        if vram_total_mib >= 24_000 {
            HardwareTier::HighEndGpu
        } else if vram_total_mib >= 8_000 {
            HardwareTier::MidRangeGpu
        } else if ram_total_mb >= 16_000 {
            HardwareTier::CpuFocused
        } else {
            HardwareTier::LowSpecCloudFallback
        }
    }

    pub fn inspect_existing_models(
        &self,
        gateway: &PermissionGateway,
        custom_specs: Option<&[(&str, &str, &Path, u64)]>,
    ) -> Result<Vec<ModelReuseStatus>, InstallerError> {
        gateway.check_readonly_allowed(&ActionKind::InspectHardwareAndModels)?;

        let default_specs = [
            (
                "Hauptmodell",
                "Qwen3.8-27B-UD-Q4_K_M.gguf",
                Path::new(r"D:\AI_Models\llm\Qwen3.8-27B-UD-Q4_K_M.gguf"),
                16_464_440_224u64,
            ),
            (
                "Vision-Projektor",
                "mmproj-Qwen3.8-27B-BF16.gguf",
                Path::new(r"D:\AI_Models\llm\assets\mmproj-Qwen3.8-27B-BF16.gguf"),
                931_146_432u64,
            ),
            (
                "Speculative-Draft",
                "Qwen3.8-27B-DFlash2-Q4_K_M.gguf",
                Path::new(r"D:\AI_Models\llm\assets\Qwen3.8-27B-DFlash2-Q4_K_M.gguf"),
                1_143_006_816u64,
            ),
            (
                "Spracherkennung (ASR)",
                "ggml-small.bin",
                Path::new(r"D:\AI_Models\unsloth\whisper.cpp\models\ggml-small.bin"),
                487_601_967u64,
            ),
        ];

        let specs: Vec<(&str, &str, &Path, u64)> = match custom_specs {
            Some(specs) => specs.to_vec(),
            None => default_specs.to_vec(),
        };

        let mut results = Vec::with_capacity(specs.len());
        for (role, model_name, path, expected_size) in specs {
            let exists = path.is_file();
            let size = if exists {
                path.metadata().map(|m| m.len()).unwrap_or(expected_size)
            } else {
                expected_size
            };

            results.push(ModelReuseStatus {
                role: role.to_string(),
                model_name: model_name.to_string(),
                expected_path: path.to_string_lossy().to_string(),
                exists_locally: exists,
                size_bytes: size,
            });
        }

        Ok(results)
    }

    pub fn calculate_eta(
        tier: &HardwareTier,
        models: &[ModelReuseStatus],
        selected_apps_count: usize,
        unknown_adapters_count: usize,
        download_speed_mbps: u64,
    ) -> InstallerEtaForecast {
        let remaining_download_bytes: u64 = models
            .iter()
            .filter(|m| !m.exists_locally)
            .map(|m| m.size_bytes)
            .sum();

        let reused_models_count = models.iter().filter(|m| m.exists_locally).count();

        let bytes_per_sec = download_speed_mbps.max(1) * 125_000;
        let download_seconds = if remaining_download_bytes == 0 {
            0
        } else {
            (remaining_download_bytes as f64 / bytes_per_sec as f64).ceil() as u64
        };

        let ready_to_wake_seconds = 45 + download_seconds;
        let personal_setup_seconds = ready_to_wake_seconds + (selected_apps_count as u64 * 30);

        let extended_integration_note = if unknown_adapters_count > 0 {
            format!(
                "{} App(s) erfordern unbekannte Adapter. Diese werden isoliert geprüft; hierfür wird keine künstlich genaue Restzeit garantiert.",
                unknown_adapters_count
            )
        } else {
            "Alle ausgewählten Apps nutzen bekannte Standard-Adapter.".to_string()
        };

        let human_summary = format!(
            "Hardware-Klasse: {:?}. Wiederverwendete lokale Modelle: {}/{}. AIDO einsatzbereit in ca. {}s (persönliche Einrichtung ca. {}s).",
            tier, reused_models_count, models.len(), ready_to_wake_seconds, personal_setup_seconds
        );

        InstallerEtaForecast {
            ready_to_wake_seconds,
            personal_setup_seconds,
            remaining_download_bytes,
            reused_models_count,
            selected_apps_count,
            extended_integration_note,
            human_summary,
        }
    }

    fn row_to_step_record(row: &rusqlite::Row) -> rusqlite::Result<InstallerStepRecord> {
        let status_str: String = row.get(3)?;
        let status = InstallerStepStatus::from_str(&status_str)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e)))?;
        Ok(InstallerStepRecord {
            step_index: row.get::<_, i64>(0)? as u8,
            step_key: row.get(1)?,
            title_de: row.get(2)?,
            status,
            rollback_action: row.get(4)?,
            details: row.get(5)?,
        })
    }

    fn get_step_by_index(&self, conn: &Connection, step_index: u8) -> Result<InstallerStepRecord, InstallerError> {
        conn.query_row(
            "SELECT step_index, step_key, title_de, status, rollback_action, details FROM installer_steps WHERE step_index = ?1",
            params![step_index as i64],
            Self::row_to_step_record,
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => InstallerError::InvalidStep(step_index),
            other => InstallerError::Sqlite(other),
        })
    }

    pub fn list_steps(&self) -> Result<Vec<InstallerStepRecord>, InstallerError> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT step_index, step_key, title_de, status, rollback_action, details FROM installer_steps ORDER BY step_index ASC",
        )?;
        let rows = stmt.query_map([], Self::row_to_step_record)?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn complete_step(&self, step_index: u8, details: &str) -> Result<InstallerStepRecord, InstallerError> {
        let conn = self.connect()?;
        let affected = conn.execute(
            "UPDATE installer_steps SET status = ?1, details = ?2 WHERE step_index = ?3",
            params![InstallerStepStatus::Completed.as_str(), details, step_index as i64],
        )?;
        if affected == 0 {
            return Err(InstallerError::InvalidStep(step_index));
        }
        self.get_step_by_index(&conn, step_index)
    }

    pub fn pause_at_step(&self, step_index: u8, reason: &str) -> Result<InstallerStepRecord, InstallerError> {
        let conn = self.connect()?;
        let affected = conn.execute(
            "UPDATE installer_steps SET status = ?1, details = ?2 WHERE step_index = ?3",
            params![InstallerStepStatus::Paused.as_str(), reason, step_index as i64],
        )?;
        if affected == 0 {
            return Err(InstallerError::InvalidStep(step_index));
        }
        self.get_step_by_index(&conn, step_index)
    }

    pub fn next_resumable_step(&self) -> Result<Option<InstallerStepRecord>, InstallerError> {
        let steps = self.list_steps()?;
        Ok(steps
            .into_iter()
            .find(|s| s.status == InstallerStepStatus::Paused || s.status == InstallerStepStatus::Pending))
    }

    pub fn rollback_completed_steps(&self) -> Result<usize, InstallerError> {
        let conn = self.connect()?;
        let count = conn.execute(
            "UPDATE installer_steps SET status = ?1 WHERE status IN (?2, ?3)",
            params![
                InstallerStepStatus::RolledBack.as_str(),
                InstallerStepStatus::Completed.as_str(),
                InstallerStepStatus::Paused.as_str(),
            ],
        )?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permission_gateway::{AccessMode, PrivacyMode};
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_hardware_tier_classification() {
        assert_eq!(
            SmartInstallerEngine::classify_hardware(24_000, 32_000),
            HardwareTier::HighEndGpu
        );
        assert_eq!(
            SmartInstallerEngine::classify_hardware(48_000, 64_000),
            HardwareTier::HighEndGpu
        );
        assert_eq!(
            SmartInstallerEngine::classify_hardware(8_000, 16_000),
            HardwareTier::MidRangeGpu
        );
        assert_eq!(
            SmartInstallerEngine::classify_hardware(23_999, 16_000),
            HardwareTier::MidRangeGpu
        );
        assert_eq!(
            SmartInstallerEngine::classify_hardware(4_000, 16_000),
            HardwareTier::CpuFocused
        );
        assert_eq!(
            SmartInstallerEngine::classify_hardware(0, 32_000),
            HardwareTier::CpuFocused
        );
        assert_eq!(
            SmartInstallerEngine::classify_hardware(4_000, 8_000),
            HardwareTier::LowSpecCloudFallback
        );
        assert_eq!(
            SmartInstallerEngine::classify_hardware(0, 4_000),
            HardwareTier::LowSpecCloudFallback
        );
    }

    #[test]
    fn test_model_reuse_and_eta_calculation() {
        let temp_dir = std::env::temp_dir().join(format!("aido_test_installer_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let model_path = temp_dir.join("local_model.gguf");
        {
            let mut f = File::create(&model_path).unwrap();
            f.write_all(&vec![0u8; 1_000_000]).unwrap();
        }
        let missing_path = temp_dir.join("missing_model.gguf");

        let db_path = temp_dir.join("installer.db");
        let engine = SmartInstallerEngine::new(&db_path).unwrap();

        let gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

        let custom_specs = [
            ("Hauptmodell", "local_model.gguf", model_path.as_path(), 1_000_000u64),
            ("Zusatzmodell", "missing_model.gguf", missing_path.as_path(), 2_500_000u64),
        ];

        let models = engine
            .inspect_existing_models(&gateway, Some(&custom_specs))
            .unwrap();

        assert_eq!(models.len(), 2);
        assert!(models[0].exists_locally);
        assert_eq!(models[0].size_bytes, 1_000_000);
        assert!(!models[1].exists_locally);
        assert_eq!(models[1].size_bytes, 2_500_000);

        // Download speed: 10 Mbps = 1_250_000 bytes/sec
        // Missing bytes: 2_500_000 bytes => ceil(2_500_000 / 1_250_000) = 2s download
        // Ready to wake: 45 + 2 = 47s
        // Selected apps: 2 => personal setup = 47 + 2 * 30 = 107s
        let forecast = SmartInstallerEngine::calculate_eta(
            &HardwareTier::HighEndGpu,
            &models,
            2,
            1,
            10,
        );

        assert_eq!(forecast.remaining_download_bytes, 2_500_000);
        assert_eq!(forecast.reused_models_count, 1);
        assert_eq!(forecast.selected_apps_count, 2);
        assert_eq!(forecast.ready_to_wake_seconds, 47);
        assert_eq!(forecast.personal_setup_seconds, 107);
        assert!(forecast.extended_integration_note.contains("1 App(s) erfordern unbekannte Adapter"));
        assert!(forecast.human_summary.contains("HighEndGpu"));

        // When all models exist locally, download_seconds = 0, ready_to_wake = 45s
        let all_local = [
            ModelReuseStatus {
                role: "Haupt".into(),
                model_name: "m1".into(),
                expected_path: "/path".into(),
                exists_locally: true,
                size_bytes: 100,
            }
        ];
        let forecast_instant = SmartInstallerEngine::calculate_eta(
            &HardwareTier::MidRangeGpu,
            &all_local,
            0,
            0,
            100,
        );
        assert_eq!(forecast_instant.remaining_download_bytes, 0);
        assert_eq!(forecast_instant.ready_to_wake_seconds, 45);
        assert_eq!(forecast_instant.personal_setup_seconds, 45);
        assert_eq!(forecast_instant.extended_integration_note, "Alle ausgewählten Apps nutzen bekannte Standard-Adapter.");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_transactional_installer_pause_and_resume() {
        let temp_dir = std::env::temp_dir().join(format!("aido_test_installer_{}", uuid::Uuid::new_v4()));
        let db_path = temp_dir.join("installer.db");
        let engine = SmartInstallerEngine::new(&db_path).unwrap();

        let steps = engine.list_steps().unwrap();
        assert_eq!(steps.len(), 9);
        for s in &steps {
            assert_eq!(s.status, InstallerStepStatus::Pending);
        }

        let first = engine.next_resumable_step().unwrap().unwrap();
        assert_eq!(first.step_index, 1);
        assert_eq!(first.step_key, "welcome_privacy");

        let completed_1 = engine.complete_step(1, "Nutzer hat Datenschutz bestätigt").unwrap();
        assert_eq!(completed_1.status, InstallerStepStatus::Completed);
        assert_eq!(completed_1.details, "Nutzer hat Datenschutz bestätigt");

        let completed_2 = engine.complete_step(2, "Hardware erkannt: HighEndGpu").unwrap();
        assert_eq!(completed_2.status, InstallerStepStatus::Completed);

        let paused_3 = engine.pause_at_step(3, "Nutzer pausiert für spätere Konfiguration").unwrap();
        assert_eq!(paused_3.status, InstallerStepStatus::Paused);
        assert_eq!(paused_3.details, "Nutzer pausiert für spätere Konfiguration");

        let next = engine.next_resumable_step().unwrap().unwrap();
        assert_eq!(next.step_index, 3);
        assert_eq!(next.status, InstallerStepStatus::Paused);

        let completed_3 = engine.complete_step(3, "Zugriffsprofil StandardAccess gewählt").unwrap();
        assert_eq!(completed_3.status, InstallerStepStatus::Completed);

        let next_after_3 = engine.next_resumable_step().unwrap().unwrap();
        assert_eq!(next_after_3.step_index, 4);
        assert_eq!(next_after_3.status, InstallerStepStatus::Pending);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_installer_rollback_preserves_existing_models() {
        let temp_dir = std::env::temp_dir().join(format!("aido_test_installer_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let existing_model_path = temp_dir.join("existing_model.gguf");
        {
            let mut f = File::create(&existing_model_path).unwrap();
            f.write_all(b"UNTOUCHABLE_EXISTING_MODEL_DATA").unwrap();
        }

        let db_path = temp_dir.join("installer.db");
        let engine = SmartInstallerEngine::new(&db_path).unwrap();

        engine.complete_step(1, "Schritt 1 abgeschlossen").unwrap();
        engine.complete_step(2, "Schritt 2 abgeschlossen").unwrap();
        engine.complete_step(3, "Schritt 3 abgeschlossen").unwrap();
        engine.pause_at_step(4, "Schritt 4 pausiert").unwrap();

        let rolled_back_count = engine.rollback_completed_steps().unwrap();
        assert_eq!(rolled_back_count, 4);

        let steps = engine.list_steps().unwrap();
        for step in &steps {
            if step.step_index <= 4 {
                assert_eq!(step.status, InstallerStepStatus::RolledBack);
            } else {
                assert_eq!(step.status, InstallerStepStatus::Pending);
            }
        }

        // Verify that existing foreign files/models were preserved untouched
        assert!(existing_model_path.is_file());
        let file_contents = std::fs::read(&existing_model_path).unwrap();
        assert_eq!(file_contents, b"UNTOUCHABLE_EXISTING_MODEL_DATA");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_readonly_gateway_allows_model_inspection() {
        let temp_dir = std::env::temp_dir().join(format!("aido_test_installer_{}", uuid::Uuid::new_v4()));
        let db_path = temp_dir.join("installer.db");
        let engine = SmartInstallerEngine::new(&db_path).unwrap();

        let readonly_gateway = PermissionGateway::new(AccessMode::ReadOnly, PrivacyMode::PrivateOffline);
        let models = engine.inspect_existing_models(&readonly_gateway, None).unwrap();
        assert_eq!(models.len(), 4);

        // Invalid step returns InvalidStep error
        let err = engine.complete_step(42, "Ungültiger Schritt").unwrap_err();
        assert!(matches!(err, InstallerError::InvalidStep(42)));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
