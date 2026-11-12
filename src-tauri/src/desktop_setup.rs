use crate::permission_gateway::{AccessMode, PrivacyMode};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelBackend {
    #[default]
    Windows,
    Wsl,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ModelProfile {
    pub backend: ModelBackend,
    pub executable: String,
    pub model: String,
    pub distro: String,
    pub user: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SetupSettings {
    pub step: u8,
    pub completed: bool,
    pub access_mode: AccessMode,
    pub privacy_mode: PrivacyMode,
    pub project_roots: Vec<PathBuf>,
    pub recent_metadata: bool,
    pub model_profile: ModelProfile,
    pub rea_enabled: bool,
    pub antigravity_enabled: bool,
    pub blender_enabled: bool,
}

impl Default for SetupSettings {
    fn default() -> Self {
        Self {
            step: 0,
            completed: false,
            access_mode: AccessMode::StandardAccess,
            privacy_mode: PrivacyMode::PrivateOffline,
            project_roots: Vec::new(),
            recent_metadata: true,
            model_profile: ModelProfile::default(),
            rea_enabled: true,
            antigravity_enabled: false,
            blender_enabled: true,
        }
    }
}

pub struct SetupStore {
    path: PathBuf,
}

impl SetupStore {
    pub fn new(path: &Path) -> Result<Self, String> {
        let parent = path.parent().ok_or("Der AIDO-Datenordner fehlt.")?;
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let store = Self { path: path.into() };
        store
            .connect()?
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS desktop_setup (
                id INTEGER PRIMARY KEY CHECK (id IN (1, 2)),
                settings TEXT NOT NULL
            );",
            )
            .map_err(|error| error.to_string())?;
        Ok(store)
    }

    fn connect(&self) -> Result<Connection, String> {
        let connection = Connection::open(&self.path).map_err(|error| error.to_string())?;
        connection
            .busy_timeout(std::time::Duration::from_secs(3))
            .map_err(|error| error.to_string())?;
        Ok(connection)
    }

    pub fn load(&self) -> Result<SetupSettings, String> {
        Ok(self.read(1)?.unwrap_or_default())
    }

    pub fn active(&self) -> Result<Option<SetupSettings>, String> {
        self.read(2)
    }

    fn read(&self, id: i64) -> Result<Option<SetupSettings>, String> {
        let row: Option<String> = self
            .connect()?
            .query_row(
                "SELECT settings FROM desktop_setup WHERE id = ?1",
                [id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        row.map(|json| serde_json::from_str(&json).map_err(|error| error.to_string()))
            .transpose()
    }

    pub fn save(&self, settings: &SetupSettings) -> Result<(), String> {
        let json = serde_json::to_string(settings).map_err(|error| error.to_string())?;
        self.connect()?
            .execute(
                "INSERT INTO desktop_setup (id, settings) VALUES (1, ?1)
             ON CONFLICT(id) DO UPDATE SET settings = excluded.settings",
                params![json],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn activate(&self, settings: &SetupSettings) -> Result<(), String> {
        let json = serde_json::to_string(settings).map_err(|error| error.to_string())?;
        self.connect()?
            .execute(
                "INSERT INTO desktop_setup (id, settings) VALUES (1, ?1), (2, ?1)
             ON CONFLICT(id) DO UPDATE SET settings = excluded.settings",
                params![json],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }
}

pub fn canonical_directory(path: &Path) -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        use std::path::{Component, Prefix};
        if !matches!(path.components().next(), Some(Component::Prefix(prefix))
            if matches!(prefix.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)))
        {
            return Err("Wähle einen lokalen Windows-Ordner.".into());
        }
    }
    if !path.is_absolute() || !path.is_dir() {
        return Err(format!(
            "Projektordner nicht erreichbar: {}",
            path.display()
        ));
    }
    let canonical = path.canonicalize().map_err(|error| error.to_string())?;
    let text = canonical.to_string_lossy();
    if let Some(local) = text.strip_prefix(r"\\?\") {
        if local.starts_with("UNC\\") || local.as_bytes().get(1) != Some(&b':') {
            return Err("Wähle einen lokalen Windows-Ordner.".into());
        }
        return Ok(PathBuf::from(local));
    }
    Ok(canonical)
}
