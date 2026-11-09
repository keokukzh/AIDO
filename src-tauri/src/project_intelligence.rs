use crate::permission_gateway::{ActionKind, ApprovalRequest, GatewayError, PermissionGateway};
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectEntry {
    pub name: String,
    pub canonical_path: String,
    pub stack: Vec<String>,
    pub git_branch: String,
    pub git_status: String,
    pub recommended_action: String,
    pub human_summary: String,
}

#[derive(Debug, Error)]
pub enum ProjectIntelError {
    #[error("SQLite-Fehler: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("JSON-Fehler: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Permission-Gateway-Fehler: {0}")]
    Gateway(#[from] GatewayError),
    #[error("Dateisystem-Fehler: {0}")]
    Io(#[from] std::io::Error),
    #[error("Projekt '{0}' wurde nicht im Katalog gefunden")]
    ProjectNotFound(String),
}

pub struct ProjectIntelligenceEngine {
    db_path: PathBuf,
}

const IGNORED_DIRS: &[&str] = &[
    "node_modules",
    "target",
    ".git",
    ".venv",
    "dist",
    ".superpowers",
];

fn is_ignored_dir(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    IGNORED_DIRS.iter().any(|&ignored| lower == ignored)
}

fn get_canonical_path(dir: &Path) -> String {
    let p = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
    let path_str = p.to_string_lossy().to_string();
    if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
        stripped.to_string()
    } else {
        path_str
    }
}

fn get_project_name(dir: &Path) -> String {
    dir.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed".to_string())
}

fn detect_stack(dir: &Path) -> Vec<String> {
    let mut has_rust = false;
    let mut has_node = false;
    let mut has_python = false;
    let mut has_docker = false;

    // Check directly in root directory
    if dir.join("Cargo.toml").is_file() {
        has_rust = true;
    }
    if dir.join("package.json").is_file() {
        has_node = true;
    }
    if dir.join("pyproject.toml").is_file() || dir.join("requirements.txt").is_file() {
        has_python = true;
    }
    if dir.join("docker-compose.yml").is_file()
        || dir.join("docker-compose.yaml").is_file()
        || dir.join("compose.yaml").is_file()
        || dir.join("compose.yml").is_file()
    {
        has_docker = true;
    }

    // Also check immediate subdirectories (e.g. src-tauri, frontend, backend, client, ui)
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if !is_ignored_dir(&name_str) {
                        let sub = entry.path();
                        if !has_rust && sub.join("Cargo.toml").is_file() {
                            has_rust = true;
                        }
                        if !has_node && sub.join("package.json").is_file() {
                            has_node = true;
                        }
                        if !has_python
                            && (sub.join("pyproject.toml").is_file()
                                || sub.join("requirements.txt").is_file())
                        {
                            has_python = true;
                        }
                        if !has_docker
                            && (sub.join("docker-compose.yml").is_file()
                                || sub.join("docker-compose.yaml").is_file()
                                || sub.join("compose.yaml").is_file()
                                || sub.join("compose.yml").is_file())
                        {
                            has_docker = true;
                        }
                    }
                }
            }
        }
    }

    let mut stack = Vec::new();
    if has_node {
        stack.push("Node.js/TypeScript".to_string());
    }
    if has_rust {
        stack.push("Rust".to_string());
    }
    if has_python {
        stack.push("Python".to_string());
    }
    if has_docker {
        stack.push("Docker Compose".to_string());
    }
    stack
}

fn detect_git_info(dir: &Path) -> (String, String) {
    let git_dir_path = dir.join(".git");
    if !git_dir_path.exists() {
        return ("lokal".to_string(), "lokal".to_string());
    }

    // Determine actual git directory (handles worktrees/submodules where .git is a file)
    let actual_git_dir = if git_dir_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&git_dir_path) {
            let trimmed = content.trim();
            if let Some(rel) = trimmed.strip_prefix("gitdir:") {
                let p = PathBuf::from(rel.trim());
                if p.is_absolute() {
                    p
                } else {
                    dir.join(p)
                }
            } else {
                git_dir_path.clone()
            }
        } else {
            git_dir_path.clone()
        }
    } else {
        git_dir_path.clone()
    };

    // 1. Detect branch
    let mut branch = String::new();
    let head_file = actual_git_dir.join("HEAD");
    if head_file.is_file() {
        if let Ok(head_content) = std::fs::read_to_string(&head_file) {
            let trimmed = head_content.trim();
            if let Some(b) = trimmed.strip_prefix("ref: refs/heads/") {
                branch = b.trim().to_string();
            } else if let Some(b) = trimmed.strip_prefix("ref: ") {
                branch = b.trim().to_string();
            } else if trimmed.len() >= 7 {
                branch = trimmed[..7].to_string();
            }
        }
    }

    if branch.is_empty() {
        if let Ok(output) = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(dir)
            .output()
        {
            if output.status.success() {
                let out = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !out.is_empty() && out != "HEAD" {
                    branch = out;
                }
            }
        }
    }

    if branch.is_empty() {
        branch = "main".to_string();
    }

    // 2. Detect status ("clean" or "dirty")
    let mut status = "clean".to_string();
    if let Ok(output) = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(dir)
        .output()
    {
        if output.status.success() {
            if !output.stdout.is_empty() {
                status = "dirty".to_string();
            }
        }
    }

    (branch, status)
}

fn is_project_candidate(dir: &Path) -> bool {
    dir.join(".git").exists()
        || dir.join("Cargo.toml").is_file()
        || dir.join("package.json").is_file()
        || dir.join("pyproject.toml").is_file()
        || dir.join("requirements.txt").is_file()
        || dir.join("docker-compose.yml").is_file()
        || dir.join("docker-compose.yaml").is_file()
        || dir.join("compose.yaml").is_file()
        || dir.join("compose.yml").is_file()
        || dir.join("src-tauri").join("Cargo.toml").is_file()
}

fn build_project_entry(dir: &Path) -> ProjectEntry {
    let name = get_project_name(dir);
    let canonical_path = get_canonical_path(dir);
    let stack = detect_stack(dir);
    let (git_branch, git_status) = detect_git_info(dir);

    let recommended_action = if stack.contains(&"Rust".to_string()) && !stack.contains(&"Node.js/TypeScript".to_string()) {
        "cargo test".to_string()
    } else if stack.contains(&"Node.js/TypeScript".to_string()) {
        "npm run dev".to_string()
    } else if stack.contains(&"Python".to_string()) {
        "python main.py".to_string()
    } else if stack.contains(&"Docker Compose".to_string()) {
        "docker compose up".to_string()
    } else {
        "In IDE öffnen".to_string()
    };

    let stack_str = if stack.is_empty() {
        "Kein erkannter Stack".to_string()
    } else {
        stack.join(", ")
    };

    let human_summary = format!(
        "Projekt '{}' (Stack: {}) auf Branch '{}' [{}]",
        name, stack_str, git_branch, git_status
    );

    ProjectEntry {
        name,
        canonical_path,
        stack,
        git_branch,
        git_status,
        recommended_action,
        human_summary,
    }
}

impl ProjectIntelligenceEngine {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self, ProjectIntelError> {
        let db_path = db_path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let engine = Self { db_path };
        engine.init_schema()?;
        Ok(engine)
    }

    fn connect(&self) -> Result<Connection, ProjectIntelError> {
        Ok(Connection::open(&self.db_path)?)
    }

    fn init_schema(&self) -> Result<(), ProjectIntelError> {
        let conn = self.connect()?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS project_catalog (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                canonical_path TEXT NOT NULL UNIQUE,
                stack_json TEXT NOT NULL,
                stack_text TEXT NOT NULL,
                git_branch TEXT NOT NULL,
                git_status TEXT NOT NULL,
                recommended_action TEXT NOT NULL,
                human_summary TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS project_catalog_fts USING fts5(
                name,
                canonical_path,
                stack_text,
                content='project_catalog',
                content_rowid='id'
            );

            CREATE TRIGGER IF NOT EXISTS project_catalog_ai AFTER INSERT ON project_catalog BEGIN
                INSERT INTO project_catalog_fts(rowid, name, canonical_path, stack_text)
                VALUES (new.id, new.name, new.canonical_path, new.stack_text);
            END;

            CREATE TRIGGER IF NOT EXISTS project_catalog_ad AFTER DELETE ON project_catalog BEGIN
                INSERT INTO project_catalog_fts(project_catalog_fts, rowid, name, canonical_path, stack_text)
                VALUES ('delete', old.id, old.name, old.canonical_path, old.stack_text);
            END;

            CREATE TRIGGER IF NOT EXISTS project_catalog_au AFTER UPDATE ON project_catalog BEGIN
                INSERT INTO project_catalog_fts(project_catalog_fts, rowid, name, canonical_path, stack_text)
                VALUES ('delete', old.id, old.name, old.canonical_path, old.stack_text);
                INSERT INTO project_catalog_fts(rowid, name, canonical_path, stack_text)
                VALUES (new.id, new.name, new.canonical_path, new.stack_text);
            END;
            ",
        )?;
        Ok(())
    }

    pub fn index_roots(
        &self,
        gateway: &PermissionGateway,
        roots: &[PathBuf],
        max_depth: usize,
    ) -> Result<Vec<ProjectEntry>, ProjectIntelError> {
        gateway.check_readonly_allowed(&ActionKind::ScanProjects)?;

        let mut discovered = Vec::new();
        let mut seen_paths = HashSet::new();

        for root in roots {
            if !root.exists() {
                continue;
            }
            self.scan_recursive(root, 0, max_depth, &mut discovered, &mut seen_paths)?;
        }

        let mut conn = self.connect()?;
        let tx = conn.transaction()?;

        for entry in &discovered {
            let stack_json = serde_json::to_string(&entry.stack)?;
            let stack_text = entry.stack.join(" ");
            let now = Utc::now().to_rfc3339();

            tx.execute(
                "INSERT INTO project_catalog (
                    name, canonical_path, stack_json, stack_text, git_branch, git_status, recommended_action, human_summary, updated_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                ON CONFLICT(canonical_path) DO UPDATE SET
                    name = excluded.name,
                    stack_json = excluded.stack_json,
                    stack_text = excluded.stack_text,
                    git_branch = excluded.git_branch,
                    git_status = excluded.git_status,
                    recommended_action = excluded.recommended_action,
                    human_summary = excluded.human_summary,
                    updated_at = excluded.updated_at;",
                params![
                    &entry.name,
                    &entry.canonical_path,
                    &stack_json,
                    &stack_text,
                    &entry.git_branch,
                    &entry.git_status,
                    &entry.recommended_action,
                    &entry.human_summary,
                    &now,
                ],
            )?;
        }

        tx.commit()?;

        Ok(discovered)
    }

    fn scan_recursive(
        &self,
        dir: &Path,
        current_depth: usize,
        max_depth: usize,
        discovered: &mut Vec<ProjectEntry>,
        seen_paths: &mut HashSet<String>,
    ) -> Result<(), ProjectIntelError> {
        if !dir.is_dir() {
            return Ok(());
        }

        if is_project_candidate(dir) {
            let entry = build_project_entry(dir);
            if seen_paths.insert(entry.canonical_path.clone()) {
                discovered.push(entry);
            }
            return Ok(());
        }

        if current_depth < max_depth {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let file_name = entry.file_name();
                        let name_str = file_name.to_string_lossy();
                        if !is_ignored_dir(&name_str) {
                            self.scan_recursive(
                                &path,
                                current_depth + 1,
                                max_depth,
                                discovered,
                                seen_paths,
                            )?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn list_projects(&self) -> Result<Vec<ProjectEntry>, ProjectIntelError> {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(
            "SELECT name, canonical_path, stack_json, git_branch, git_status, recommended_action, human_summary
             FROM project_catalog
             ORDER BY name ASC;",
        )?;

        let rows = stmt.query_map([], |row| {
            let name: String = row.get(0)?;
            let canonical_path: String = row.get(1)?;
            let stack_json: String = row.get(2)?;
            let git_branch: String = row.get(3)?;
            let git_status: String = row.get(4)?;
            let recommended_action: String = row.get(5)?;
            let human_summary: String = row.get(6)?;

            let stack: Vec<String> = serde_json::from_str(&stack_json).unwrap_or_default();

            Ok(ProjectEntry {
                name,
                canonical_path,
                stack,
                git_branch,
                git_status,
                recommended_action,
                human_summary,
            })
        })?;

        let mut entries = Vec::new();
        for r in rows {
            entries.push(r?);
        }
        Ok(entries)
    }

    pub fn search_projects(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ProjectEntry>, ProjectIntelError> {
        let query_trimmed = query.trim();
        if query_trimmed.is_empty() {
            return self.list_projects();
        }

        let conn = self.connect()?;

        // Attempt 0: Exact (case-insensitive) project name match
        let mut exact_matches = Vec::new();
        {
            let mut stmt = conn.prepare(
                "SELECT name, canonical_path, stack_json, git_branch, git_status, recommended_action, human_summary
                 FROM project_catalog
                 WHERE LOWER(name) = LOWER(?1)
                 LIMIT ?2;",
            )?;
            let rows = stmt.query_map(params![query_trimmed, limit as i64], |row| {
                let name: String = row.get(0)?;
                let canonical_path: String = row.get(1)?;
                let stack_json: String = row.get(2)?;
                let git_branch: String = row.get(3)?;
                let git_status: String = row.get(4)?;
                let recommended_action: String = row.get(5)?;
                let human_summary: String = row.get(6)?;

                let stack: Vec<String> = serde_json::from_str(&stack_json).unwrap_or_default();

                Ok(ProjectEntry {
                    name,
                    canonical_path,
                    stack,
                    git_branch,
                    git_status,
                    recommended_action,
                    human_summary,
                })
            })?;
            for r in rows {
                exact_matches.push(r?);
            }
        }
        if !exact_matches.is_empty() {
            return Ok(exact_matches);
        }

        // Attempt 1: FTS5 MATCH
        if let Ok(entries) = self.search_fts5(&conn, query_trimmed, limit) {
            if !entries.is_empty() {
                return Ok(entries);
            }
        }

        // Attempt 2: Fallback LIKE search
        self.search_like(&conn, query_trimmed, limit)
    }

    fn search_fts5(
        &self,
        conn: &Connection,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ProjectEntry>, rusqlite::Error> {
        let sanitized = query
            .split_whitespace()
            .map(|token| {
                let clean = token.replace('"', "");
                format!("\"{}\"", clean)
            })
            .collect::<Vec<_>>()
            .join(" ");

        let mut stmt = conn.prepare(
            "SELECT p.name, p.canonical_path, p.stack_json, p.git_branch, p.git_status, p.recommended_action, p.human_summary
             FROM project_catalog_fts f
             JOIN project_catalog p ON p.id = f.rowid
             WHERE project_catalog_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2;",
        )?;

        let rows = stmt.query_map(params![sanitized, limit as i64], |row| {
            let name: String = row.get(0)?;
            let canonical_path: String = row.get(1)?;
            let stack_json: String = row.get(2)?;
            let git_branch: String = row.get(3)?;
            let git_status: String = row.get(4)?;
            let recommended_action: String = row.get(5)?;
            let human_summary: String = row.get(6)?;

            let stack: Vec<String> = serde_json::from_str(&stack_json).unwrap_or_default();

            Ok(ProjectEntry {
                name,
                canonical_path,
                stack,
                git_branch,
                git_status,
                recommended_action,
                human_summary,
            })
        })?;

        let mut entries = Vec::new();
        for r in rows {
            entries.push(r?);
        }
        Ok(entries)
    }

    fn search_like(
        &self,
        conn: &Connection,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ProjectEntry>, ProjectIntelError> {
        let pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT name, canonical_path, stack_json, git_branch, git_status, recommended_action, human_summary
             FROM project_catalog
             WHERE name LIKE ?1 OR canonical_path LIKE ?1 OR stack_text LIKE ?1
             ORDER BY name ASC
             LIMIT ?2;",
        )?;

        let rows = stmt.query_map(params![pattern, limit as i64], |row| {
            let name: String = row.get(0)?;
            let canonical_path: String = row.get(1)?;
            let stack_json: String = row.get(2)?;
            let git_branch: String = row.get(3)?;
            let git_status: String = row.get(4)?;
            let recommended_action: String = row.get(5)?;
            let human_summary: String = row.get(6)?;

            let stack: Vec<String> = serde_json::from_str(&stack_json).unwrap_or_default();

            Ok(ProjectEntry {
                name,
                canonical_path,
                stack,
                git_branch,
                git_status,
                recommended_action,
                human_summary,
            })
        })?;

        let mut entries = Vec::new();
        for r in rows {
            entries.push(r?);
        }
        Ok(entries)
    }

    pub fn prepare_project_launch(
        &self,
        gateway: &PermissionGateway,
        project_name_or_query: &str,
    ) -> Result<(ProjectEntry, ApprovalRequest), ProjectIntelError> {
        let results = self.search_projects(project_name_or_query, 1)?;
        let project = results
            .into_iter()
            .next()
            .ok_or_else(|| ProjectIntelError::ProjectNotFound(project_name_or_query.to_string()))?;

        let stack_str = if project.stack.is_empty() {
            "Kein erkannter Stack".to_string()
        } else {
            project.stack.join(", ")
        };

        let human_title = format!("Projekt-Start: {}", project.name);
        let human_explanation = format!(
            "Ich habe dein Projekt '{}' gefunden (Stack: {}). Ich würde die Arbeitsumgebung prüfen und das Projekt vorbereiten. Soll ich das ausführen?",
            project.name, stack_str
        );

        let target_summary = vec![
            project.name.clone(),
            project.canonical_path.clone(),
            project.recommended_action.clone(),
        ];

        let approval_request = gateway.request_approval(
            ActionKind::LaunchProject,
            human_title,
            human_explanation,
            target_summary,
            0,
            0,
        )?;

        Ok((project, approval_request))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permission_gateway::{AccessMode, PrivacyMode};
    use uuid::Uuid;

    fn setup_test_engine() -> (ProjectIntelligenceEngine, PathBuf) {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("aido_project_intel_test_{}.db", Uuid::new_v4()));
        let engine = ProjectIntelligenceEngine::new(&db_path).expect("Engine initialization failed");
        (engine, db_path)
    }

    fn cleanup_db(path: &Path) {
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_scan_tree_stack_detection_and_fts5_search() {
        let (engine, db_path) = setup_test_engine();
        let gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

        let temp_base = std::env::temp_dir().join(format!("proj_scan_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&temp_base).expect("Failed to create temp scan dir");

        // 1. Rust project with mock git
        let rust_proj = temp_base.join("rust_service");
        std::fs::create_dir_all(&rust_proj).unwrap();
        std::fs::write(rust_proj.join("Cargo.toml"), "[package]\nname = \"rust_service\"\n").unwrap();
        let git_dir = rust_proj.join(".git");
        std::fs::create_dir_all(&git_dir).unwrap();
        std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/develop\n").unwrap();

        // 2. Node.js project (local, no git)
        let web_proj = temp_base.join("web_dashboard");
        std::fs::create_dir_all(&web_proj).unwrap();
        std::fs::write(web_proj.join("package.json"), "{\"name\": \"web_dashboard\"}").unwrap();

        // 3. Polyglot project (Rust + Node.js like Tauri / AIDO)
        let poly_proj = temp_base.join("aido_desktop");
        std::fs::create_dir_all(&poly_proj).unwrap();
        std::fs::write(poly_proj.join("package.json"), "{\"name\": \"aido_desktop\"}").unwrap();
        let tauri_sub = poly_proj.join("src-tauri");
        std::fs::create_dir_all(&tauri_sub).unwrap();
        std::fs::write(tauri_sub.join("Cargo.toml"), "[package]\nname = \"aido_core\"\n").unwrap();

        // 4. Python project
        let py_proj = temp_base.join("ai_worker");
        std::fs::create_dir_all(&py_proj).unwrap();
        std::fs::write(py_proj.join("pyproject.toml"), "[tool.poetry]\nname = \"ai_worker\"\n").unwrap();

        // 5. Ignored directory with nested project (must NOT be indexed)
        let ignored = temp_base.join("node_modules").join("dummy_pkg");
        std::fs::create_dir_all(&ignored).unwrap();
        std::fs::write(ignored.join("package.json"), "{}").unwrap();

        // Index the roots
        let discovered = engine
            .index_roots(&gateway, &[temp_base.clone()], 3)
            .expect("index_roots failed");

        assert_eq!(discovered.len(), 4, "Should find exactly 4 projects, ignoring node_modules");

        let names: Vec<String> = discovered.iter().map(|p| p.name.clone()).collect();
        assert!(names.contains(&"rust_service".to_string()));
        assert!(names.contains(&"web_dashboard".to_string()));
        assert!(names.contains(&"aido_desktop".to_string()));
        assert!(names.contains(&"ai_worker".to_string()));

        // Check rust_service
        let rust_entry = discovered.iter().find(|p| p.name == "rust_service").unwrap();
        assert_eq!(rust_entry.stack, vec!["Rust"]);
        assert_eq!(rust_entry.git_branch, "develop");
        assert_eq!(rust_entry.recommended_action, "cargo test");

        // Check web_dashboard
        let web_entry = discovered.iter().find(|p| p.name == "web_dashboard").unwrap();
        assert_eq!(web_entry.stack, vec!["Node.js/TypeScript"]);
        assert_eq!(web_entry.git_branch, "lokal");
        assert_eq!(web_entry.git_status, "lokal");
        assert_eq!(web_entry.recommended_action, "npm run dev");

        // Check aido_desktop (Polyglot)
        let poly_entry = discovered.iter().find(|p| p.name == "aido_desktop").unwrap();
        assert_eq!(poly_entry.stack, vec!["Node.js/TypeScript", "Rust"]);
        assert_eq!(poly_entry.recommended_action, "npm run dev");

        // Check ai_worker
        let py_entry = discovered.iter().find(|p| p.name == "ai_worker").unwrap();
        assert_eq!(py_entry.stack, vec!["Python"]);
        assert_eq!(py_entry.recommended_action, "python main.py");

        // Verify persistence & list_projects
        let all_projects = engine.list_projects().expect("list_projects failed");
        assert_eq!(all_projects.len(), 4);

        // Verify FTS5 searches
        let rust_search = engine.search_projects("Rust", 10).expect("search failed");
        assert_eq!(rust_search.len(), 2); // rust_service & aido_desktop

        let aido_search = engine.search_projects("aido", 5).expect("search failed");
        assert_eq!(aido_search.len(), 1);
        assert_eq!(aido_search[0].name, "aido_desktop");

        let web_search = engine.search_projects("web_dashboard", 5).expect("search failed");
        assert_eq!(web_search.len(), 1);
        assert_eq!(web_search[0].name, "web_dashboard");

        let _ = std::fs::remove_dir_all(&temp_base);
        cleanup_db(&db_path);
    }

    #[test]
    fn test_prepare_project_launch_generates_approval_request() {
        let (engine, db_path) = setup_test_engine();
        let gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

        let temp_base = std::env::temp_dir().join(format!("aido_launch_test_{}", Uuid::new_v4()));
        let proj_dir = temp_base.join("AIDO");
        std::fs::create_dir_all(&proj_dir).unwrap();
        std::fs::write(proj_dir.join("package.json"), "{\"name\":\"AIDO\"}").unwrap();
        let tauri_dir = proj_dir.join("src-tauri");
        std::fs::create_dir_all(&tauri_dir).unwrap();
        std::fs::write(tauri_dir.join("Cargo.toml"), "[package]\nname=\"aido_core\"\n").unwrap();

        engine.index_roots(&gateway, &[temp_base.clone()], 2).unwrap();

        let (project, approval_req) = engine
            .prepare_project_launch(&gateway, "AIDO")
            .expect("prepare_project_launch failed");

        assert_eq!(project.name, "AIDO");
        assert_eq!(approval_req.action, ActionKind::LaunchProject);
        assert!(approval_req.human_title.contains("AIDO"));
        assert!(approval_req
            .human_explanation
            .contains("Ich habe dein Projekt 'AIDO' gefunden (Stack: Node.js/TypeScript, Rust)"));
        assert!(approval_req.target_summary.contains(&project.name));
        assert!(approval_req.target_summary.contains(&project.canonical_path));

        // Test gateway approval and token issuance
        let token = gateway
            .approve_request(approval_req.request_id, Some("Start freigegeben".into()))
            .expect("Approval failed");
        assert_eq!(token.action, ActionKind::LaunchProject);

        // One-time token consumption
        assert!(gateway
            .consume_approval_token(&token, &ActionKind::LaunchProject)
            .is_ok());

        let _ = std::fs::remove_dir_all(&temp_base);
        cleanup_db(&db_path);
    }

    #[test]
    fn test_prepare_project_launch_blocked_in_readonly_mode() {
        let (engine, db_path) = setup_test_engine();
        let standard_gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

        let temp_base = std::env::temp_dir().join(format!("aido_readonly_test_{}", Uuid::new_v4()));
        let proj_dir = temp_base.join("ReadOnlyApp");
        std::fs::create_dir_all(&proj_dir).unwrap();
        std::fs::write(proj_dir.join("Cargo.toml"), "[package]\nname=\"ro_app\"\n").unwrap();

        // Indexing is allowed in read-only mode because ActionKind::ScanProjects is read-only
        let readonly_gateway = PermissionGateway::new(AccessMode::ReadOnly, PrivacyMode::PrivateOffline);
        let scan_res = engine.index_roots(&readonly_gateway, &[temp_base.clone()], 2);
        assert!(scan_res.is_ok(), "ReadOnly mode should permit scanning");

        // prepare_project_launch creates an ApprovalRequest for ActionKind::LaunchProject, which is mutating
        let launch_res = engine.prepare_project_launch(&readonly_gateway, "ReadOnlyApp");
        assert!(
            launch_res.is_err(),
            "prepare_project_launch must fail in ReadOnly mode"
        );
        match launch_res.unwrap_err() {
            ProjectIntelError::Gateway(GatewayError::ReadOnlyBlocked(action)) => {
                assert!(action.contains("LaunchProject"));
            }
            other => panic!("Expected ReadOnlyBlocked, got {:?}", other),
        }

        // With StandardAccess, it succeeds
        let standard_launch = engine.prepare_project_launch(&standard_gateway, "ReadOnlyApp");
        assert!(standard_launch.is_ok());

        let _ = std::fs::remove_dir_all(&temp_base);
        cleanup_db(&db_path);
    }

    #[test]
    fn test_prepare_project_launch_project_not_found() {
        let (engine, db_path) = setup_test_engine();
        let gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);

        let err = engine.prepare_project_launch(&gateway, "NonExistentProject");
        assert!(matches!(err, Err(ProjectIntelError::ProjectNotFound(_))));

        cleanup_db(&db_path);
    }
}
