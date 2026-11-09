use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("SQLite-Fehler: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Dateisystem-Fehler: {0}")]
    Io(#[from] std::io::Error),
    #[error("Aufgabe nicht gefunden: {0}")]
    TaskNotFound(String),
    #[error("Ungültiger Status oder Parameter: {0}")]
    InvalidData(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionChannel {
    ApiOrCliBackground,
    InteractiveUiSession,
}

impl ExecutionChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionChannel::ApiOrCliBackground => "ApiOrCliBackground",
            ExecutionChannel::InteractiveUiSession => "InteractiveUiSession",
        }
    }

    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "ApiOrCliBackground" => Some(ExecutionChannel::ApiOrCliBackground),
            "InteractiveUiSession" => Some(ExecutionChannel::InteractiveUiSession),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Queued,
    Running,
    PausedDesktopLocked,
    Completed,
    Failed,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Queued => "Queued",
            TaskStatus::Running => "Running",
            TaskStatus::PausedDesktopLocked => "PausedDesktopLocked",
            TaskStatus::Completed => "Completed",
            TaskStatus::Failed => "Failed",
        }
    }

    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "Queued" => Some(TaskStatus::Queued),
            "Running" => Some(TaskStatus::Running),
            "PausedDesktopLocked" => Some(TaskStatus::PausedDesktopLocked),
            "Completed" => Some(TaskStatus::Completed),
            "Failed" => Some(TaskStatus::Failed),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentTask {
    pub task_id: String,
    pub title_de: String,
    pub channel: ExecutionChannel,
    pub status: TaskStatus,
    pub schedule_note: Option<String>,
    pub requires_approval: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopLockTransitionReport {
    pub desktop_locked: bool,
    pub continued_background_tasks: usize,
    pub paused_ui_tasks: usize,
    pub resumed_ui_tasks: usize,
    pub human_summary_de: String,
}

pub struct TaskEngine {
    db_path: PathBuf,
}

impl TaskEngine {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self, TaskError> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let engine = Self { db_path };
        engine.init_schema()?;
        Ok(engine)
    }

    fn connect(&self) -> Result<Connection, TaskError> {
        Ok(Connection::open(&self.db_path)?)
    }

    fn init_schema(&self) -> Result<(), TaskError> {
        let conn = self.connect()?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS persistent_tasks (
                task_id TEXT PRIMARY KEY,
                title_de TEXT NOT NULL,
                channel TEXT NOT NULL,
                status TEXT NOT NULL,
                schedule_note TEXT,
                requires_approval INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    pub fn enqueue_task(
        &self,
        title_de: &str,
        channel: ExecutionChannel,
        schedule_note: Option<&str>,
        requires_approval: bool,
    ) -> Result<PersistentTask, TaskError> {
        let conn = self.connect()?;
        let task_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        let status = TaskStatus::Queued;

        conn.execute(
            "INSERT INTO persistent_tasks (task_id, title_de, channel, status, schedule_note, requires_approval, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
            params![
                task_id,
                title_de,
                channel.as_str(),
                status.as_str(),
                schedule_note,
                if requires_approval { 1 } else { 0 },
                now
            ],
        )?;

        Ok(PersistentTask {
            task_id,
            title_de: title_de.to_string(),
            channel,
            status,
            schedule_note: schedule_note.map(|s| s.to_string()),
            requires_approval,
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub fn start_task(&self, task_id: &str) -> Result<PersistentTask, TaskError> {
        let conn = self.connect()?;
        let now = Utc::now().to_rfc3339();
        let affected = conn.execute(
            "UPDATE persistent_tasks SET status = ?1, updated_at = ?2 WHERE task_id = ?3",
            params![TaskStatus::Running.as_str(), now, task_id],
        )?;
        if affected == 0 {
            return Err(TaskError::TaskNotFound(task_id.to_string()));
        }
        self.get_task(task_id)
    }

    pub fn complete_task(&self, task_id: &str) -> Result<PersistentTask, TaskError> {
        let conn = self.connect()?;
        let now = Utc::now().to_rfc3339();
        let affected = conn.execute(
            "UPDATE persistent_tasks SET status = ?1, updated_at = ?2 WHERE task_id = ?3",
            params![TaskStatus::Completed.as_str(), now, task_id],
        )?;
        if affected == 0 {
            return Err(TaskError::TaskNotFound(task_id.to_string()));
        }
        self.get_task(task_id)
    }

    pub fn fail_task(&self, task_id: &str) -> Result<PersistentTask, TaskError> {
        let conn = self.connect()?;
        let now = Utc::now().to_rfc3339();
        let affected = conn.execute(
            "UPDATE persistent_tasks SET status = ?1, updated_at = ?2 WHERE task_id = ?3",
            params![TaskStatus::Failed.as_str(), now, task_id],
        )?;
        if affected == 0 {
            return Err(TaskError::TaskNotFound(task_id.to_string()));
        }
        self.get_task(task_id)
    }

    pub fn get_task(&self, task_id: &str) -> Result<PersistentTask, TaskError> {
        let conn = self.connect()?;
        let task = conn.query_row(
            "SELECT task_id, title_de, channel, status, schedule_note, requires_approval, created_at, updated_at
             FROM persistent_tasks WHERE task_id = ?1",
            params![task_id],
            |row| {
                let channel_str: String = row.get(2)?;
                let status_str: String = row.get(3)?;
                let req_appr_int: i64 = row.get(5)?;
                Ok(PersistentTask {
                    task_id: row.get(0)?,
                    title_de: row.get(1)?,
                    channel: ExecutionChannel::from_str_opt(&channel_str)
                        .unwrap_or(ExecutionChannel::ApiOrCliBackground),
                    status: TaskStatus::from_str_opt(&status_str)
                        .unwrap_or(TaskStatus::Queued),
                    schedule_note: row.get(4)?,
                    requires_approval: req_appr_int != 0,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        ).optional()?;

        task.ok_or_else(|| TaskError::TaskNotFound(task_id.to_string()))
    }

    pub fn evaluate_desktop_lock_state(&self, desktop_locked: bool) -> Result<DesktopLockTransitionReport, TaskError> {
        let conn = self.connect()?;
        let now = Utc::now().to_rfc3339();

        if desktop_locked {
            // Pause all running InteractiveUiSession tasks
            let paused_ui_tasks = conn.execute(
                "UPDATE persistent_tasks
                 SET status = ?1, updated_at = ?2
                 WHERE status = ?3 AND channel = ?4",
                params![
                    TaskStatus::PausedDesktopLocked.as_str(),
                    now,
                    TaskStatus::Running.as_str(),
                    ExecutionChannel::InteractiveUiSession.as_str(),
                ],
            )?;

            // Count continuing background tasks
            let continued_background_tasks: i64 = conn.query_row(
                "SELECT COUNT(*) FROM persistent_tasks WHERE status = ?1 AND channel = ?2",
                params![
                    TaskStatus::Running.as_str(),
                    ExecutionChannel::ApiOrCliBackground.as_str(),
                ],
                |r| r.get(0),
            )?;
            let continued_background_tasks = continued_background_tasks as usize;
            let resumed_ui_tasks = 0;

            let human_summary_de = format!(
                "Desktop gesperrt: {} Hintergrund-Aufgaben laufen ungestört weiter, {} UI-Aufgaben pausiert.",
                continued_background_tasks, paused_ui_tasks
            );

            Ok(DesktopLockTransitionReport {
                desktop_locked: true,
                continued_background_tasks,
                paused_ui_tasks,
                resumed_ui_tasks,
                human_summary_de,
            })
        } else {
            // Resume all PausedDesktopLocked tasks to Running
            let resumed_ui_tasks = conn.execute(
                "UPDATE persistent_tasks
                 SET status = ?1, updated_at = ?2
                 WHERE status = ?3",
                params![
                    TaskStatus::Running.as_str(),
                    now,
                    TaskStatus::PausedDesktopLocked.as_str(),
                ],
            )?;

            // Count running background tasks
            let continued_background_tasks: i64 = conn.query_row(
                "SELECT COUNT(*) FROM persistent_tasks WHERE status = ?1 AND channel = ?2",
                params![
                    TaskStatus::Running.as_str(),
                    ExecutionChannel::ApiOrCliBackground.as_str(),
                ],
                |r| r.get(0),
            )?;
            let continued_background_tasks = continued_background_tasks as usize;
            let paused_ui_tasks = 0;

            let human_summary_de = format!(
                "Desktop aktiv: {} pausierte UI-Aufgaben fortgesetzt, {} Hintergrund-Aufgaben aktiv.",
                resumed_ui_tasks, continued_background_tasks
            );

            Ok(DesktopLockTransitionReport {
                desktop_locked: false,
                continued_background_tasks,
                paused_ui_tasks,
                resumed_ui_tasks,
                human_summary_de,
            })
        }
    }

    pub fn list_tasks(&self) -> Result<Vec<PersistentTask>, TaskError> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT task_id, title_de, channel, status, schedule_note, requires_approval, created_at, updated_at
             FROM persistent_tasks
             ORDER BY rowid ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            let channel_str: String = row.get(2)?;
            let status_str: String = row.get(3)?;
            let req_appr_int: i64 = row.get(5)?;
            Ok(PersistentTask {
                task_id: row.get(0)?,
                title_de: row.get(1)?,
                channel: ExecutionChannel::from_str_opt(&channel_str)
                    .unwrap_or(ExecutionChannel::ApiOrCliBackground),
                status: TaskStatus::from_str_opt(&status_str)
                    .unwrap_or(TaskStatus::Queued),
                schedule_note: row.get(4)?,
                requires_approval: req_appr_int != 0,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })?;

        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enqueues_and_persists_tasks_across_engine_restarts() {
        let temp_dir = std::env::temp_dir().join(format!("aido_task_test_{}", Uuid::new_v4()));
        let db_file = temp_dir.join("aido_tasks.db");

        let task_id = {
            let engine = TaskEngine::new(&db_file).unwrap();
            let task = engine
                .enqueue_task(
                    "Tägliche Datensicherung",
                    ExecutionChannel::ApiOrCliBackground,
                    Some("0 2 * * *"),
                    false,
                )
                .unwrap();
            assert_eq!(task.status, TaskStatus::Queued);
            assert_eq!(task.channel, ExecutionChannel::ApiOrCliBackground);
            assert_eq!(task.schedule_note.as_deref(), Some("0 2 * * *"));

            let running_task = engine.start_task(&task.task_id).unwrap();
            assert_eq!(running_task.status, TaskStatus::Running);
            task.task_id
        };

        // Reopen DB with fresh engine instance
        {
            let engine2 = TaskEngine::new(&db_file).unwrap();
            let tasks = engine2.list_tasks().unwrap();
            assert_eq!(tasks.len(), 1);
            assert_eq!(tasks[0].task_id, task_id);
            assert_eq!(tasks[0].title_de, "Tägliche Datensicherung");
            assert_eq!(tasks[0].status, TaskStatus::Running);
            assert_eq!(tasks[0].channel, ExecutionChannel::ApiOrCliBackground);

            let completed = engine2.complete_task(&task_id).unwrap();
            assert_eq!(completed.status, TaskStatus::Completed);

            let rechecked = engine2.get_task(&task_id).unwrap();
            assert_eq!(rechecked.status, TaskStatus::Completed);
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn desktop_lock_pauses_only_interactive_ui_tasks_while_background_cli_continues() {
        let temp_dir = std::env::temp_dir().join(format!("aido_task_lock_{}", Uuid::new_v4()));
        let db_file = temp_dir.join("aido_tasks.db");
        let engine = TaskEngine::new(&db_file).unwrap();

        let cli_task = engine
            .enqueue_task(
                "Cargo Test Hintergrund-Lauf",
                ExecutionChannel::ApiOrCliBackground,
                None,
                false,
            )
            .unwrap();

        let ui_task = engine
            .enqueue_task(
                "Browser UI Formular ausfüllen",
                ExecutionChannel::InteractiveUiSession,
                None,
                true,
            )
            .unwrap();

        // Start both tasks
        engine.start_task(&cli_task.task_id).unwrap();
        engine.start_task(&ui_task.task_id).unwrap();

        // Simulate Windows desktop lock
        let report_locked = engine.evaluate_desktop_lock_state(true).unwrap();
        assert!(report_locked.desktop_locked);
        assert_eq!(report_locked.continued_background_tasks, 1);
        assert_eq!(report_locked.paused_ui_tasks, 1);
        assert_eq!(report_locked.resumed_ui_tasks, 0);
        assert!(report_locked.human_summary_de.contains("Desktop gesperrt"));

        // CLI task must still be Running
        let cli_recheck = engine.get_task(&cli_task.task_id).unwrap();
        assert_eq!(cli_recheck.status, TaskStatus::Running);

        // UI task must be PausedDesktopLocked
        let ui_recheck = engine.get_task(&ui_task.task_id).unwrap();
        assert_eq!(ui_recheck.status, TaskStatus::PausedDesktopLocked);

        // Simulate Windows unlock
        let report_unlocked = engine.evaluate_desktop_lock_state(false).unwrap();
        assert!(!report_unlocked.desktop_locked);
        assert_eq!(report_unlocked.resumed_ui_tasks, 1);
        assert_eq!(report_unlocked.continued_background_tasks, 1);
        assert_eq!(report_unlocked.paused_ui_tasks, 0);
        assert!(report_unlocked.human_summary_de.contains("Desktop aktiv"));

        // UI task must now be resumed to Running
        let ui_resumed = engine.get_task(&ui_task.task_id).unwrap();
        assert_eq!(ui_resumed.status, TaskStatus::Running);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
