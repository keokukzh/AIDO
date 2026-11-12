use crate::permission_gateway::ActionKind;
use crate::project_intelligence::ProjectEntry;
use crate::windows_bridge::SystemHealthReport;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserActivityState {
    NormalIdle,
    FullscreenOrGaming,
    VoiceRecordingOrCall,
    QuietHoursActive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProactiveCategory {
    DirtyGitWorkspace,
    ProjectContinuation,
    SystemMaintenance,
    MemoryConsolidation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProactiveSuggestion {
    pub id: String,
    pub category: ProactiveCategory,
    pub title: String,
    pub human_message: String,
    pub why_now: String,
    pub clickable_options: Vec<String>,
    pub priority: u8,
    pub risk_level: String,
    pub action_kind: Option<ActionKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProactiveEvaluationReport {
    pub activity_state: UserActivityState,
    pub suppressed_by_focus_guard: bool,
    pub suppression_reason: Option<String>,
    pub suggestions: Vec<ProactiveSuggestion>,
}

#[derive(Debug, Clone)]
pub struct ProactiveEngine {
    muted_categories: Arc<Mutex<HashSet<ProactiveCategory>>>,
    dismiss_counts: Arc<Mutex<HashMap<String, u32>>>,
    max_suggestions: usize,
}

impl Default for ProactiveEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ProactiveEngine {
    pub fn new() -> Self {
        Self {
            muted_categories: Arc::new(Mutex::new(HashSet::new())),
            dismiss_counts: Arc::new(Mutex::new(HashMap::new())),
            max_suggestions: 3,
        }
    }

    pub fn mute_category(&self, category: ProactiveCategory) {
        self.muted_categories.lock().unwrap().insert(category);
    }

    pub fn unmute_category(&self, category: ProactiveCategory) {
        self.muted_categories.lock().unwrap().remove(&category);
    }

    pub fn is_category_muted(&self, category: ProactiveCategory) -> bool {
        self.muted_categories.lock().unwrap().contains(&category)
    }

    /// Increments dismiss count for `suggestion_id` and returns the new count.
    /// If count >= 2, future evaluations automatically suppress that `suggestion_id`.
    pub fn record_dismissal(&self, suggestion_id: &str) -> u32 {
        let mut counts = self.dismiss_counts.lock().unwrap();
        let count = counts.entry(suggestion_id.to_string()).or_insert(0);
        *count += 1;
        *count
    }

    /// Evaluates current system, project, and memory context against active user state.
    /// Suppresses all suggestions during Fullscreen, Voice Calls, or Quiet Hours (Focus Guard).
    pub fn evaluate_context(
        &self,
        activity: UserActivityState,
        health: &SystemHealthReport,
        projects: &[ProjectEntry],
        active_episodic_count: usize,
        deep_cleanup_bytes: u64,
    ) -> ProactiveEvaluationReport {
        if activity != UserActivityState::NormalIdle {
            let reason = match activity {
                UserActivityState::FullscreenOrGaming => {
                    "Fokus-Schutz aktiv: Vollbild- oder Gaming-Modus erkannt. AIDO bleibt im Hintergrund stumm."
                }
                UserActivityState::VoiceRecordingOrCall => {
                    "Fokus-Schutz aktiv: Audioaufnahme oder Sprachanruf aktiv. AIDO unterdrückt Benachrichtigungen."
                }
                UserActivityState::QuietHoursActive => {
                    "Fokus-Schutz aktiv: Ruhezeiten aktiv. AIDO hält sich dezent im Hintergrund."
                }
                UserActivityState::NormalIdle => unreachable!(),
            };
            return ProactiveEvaluationReport {
                activity_state: activity,
                suppressed_by_focus_guard: true,
                suppression_reason: Some(reason.to_string()),
                suggestions: Vec::new(),
            };
        }

        let muted = self.muted_categories.lock().unwrap();
        let dismissals = self.dismiss_counts.lock().unwrap();

        let should_include = |cat: ProactiveCategory, id: &str| -> bool {
            if muted.contains(&cat) {
                return false;
            }
            if let Some(&count) = dismissals.get(id) {
                if count >= 2 {
                    return false;
                }
            }
            true
        };

        let mut candidates = Vec::new();

        // 1. Dirty Git workspaces (Priority 9)
        for project in projects {
            if project.git_status == "dirty" {
                let id = format!("dirty_git_{}", project.name);
                if should_include(ProactiveCategory::DirtyGitWorkspace, &id) {
                    candidates.push(ProactiveSuggestion {
                        id,
                        category: ProactiveCategory::DirtyGitWorkspace,
                        title: format!("Ungespeicherte Änderungen in {}", project.name),
                        human_message: format!(
                            "Im Projekt '{}' wurden ungespeicherte Git-Änderungen erkannt. Möchtest du den Status prüfen oder sichern?",
                            project.name
                        ),
                        why_now: "Ungespeicherte Änderungen im Git-Repository festgestellt.".to_string(),
                        clickable_options: vec![
                            "Projekt öffnen & prüfen".to_string(),
                            "Später erinnern".to_string(),
                            "Thema stummschalten".to_string(),
                        ],
                        priority: 9,
                        risk_level: "YELLOW_PROJECT".to_string(),
                        action_kind: Some(ActionKind::ScanProjects),
                    });
                }
            }
        }

        // 2. System Maintenance (Priority 8)
        if deep_cleanup_bytes > 0 || health.aido_temp_cache_bytes > 0 {
            let id = "system_cleanup_hint";
            if should_include(ProactiveCategory::SystemMaintenance, id) {
                let total_bytes = deep_cleanup_bytes + health.aido_temp_cache_bytes;
                let size_str = if total_bytes >= 1024 * 1024 {
                    format!("{:.1} MB", total_bytes as f64 / (1024.0 * 1024.0))
                } else {
                    format!("{} KB", total_bytes / 1024)
                };
                candidates.push(ProactiveSuggestion {
                    id: id.to_string(),
                    category: ProactiveCategory::SystemMaintenance,
                    title: "Systemwartung empfohlen".to_string(),
                    human_message: format!(
                        "Es können ca. {} Speicherplatz durch Fast- oder Deep-Cleanup freigegeben werden.",
                        size_str
                    ),
                    why_now: "Temporäre Caches und verwaiste Build-Artefakte belegen Speicherplatz.".to_string(),
                    clickable_options: vec![
                        "Deep Cleanup prüfen".to_string(),
                        "Fast Cleanup starten".to_string(),
                        "Später erinnern".to_string(),
                    ],
                    priority: 8,
                    risk_level: "RED_SYSTEM".to_string(),
                    action_kind: Some(ActionKind::FastCleanupExecute),
                });
            }
        }

        // 3. Memory Consolidation (Priority 7)
        if active_episodic_count >= 2 {
            let id = "memory_compaction_hint";
            if should_include(ProactiveCategory::MemoryConsolidation, id) {
                candidates.push(ProactiveSuggestion {
                    id: id.to_string(),
                    category: ProactiveCategory::MemoryConsolidation,
                    title: "Gedächtnis-Kompaktierung möglich".to_string(),
                    human_message: format!(
                        "Es liegen {} episodische Erinnerungen vor, die in kompaktes semantisches Wissen zusammengefasst werden können.",
                        active_episodic_count
                    ),
                    why_now: "Mehrere episodische Erinnerungen können konsolidiert werden.".to_string(),
                    clickable_options: vec![
                        "Jetzt kompaktieren".to_string(),
                        "Später erinnern".to_string(),
                        "Thema stummschalten".to_string(),
                    ],
                    priority: 7,
                    risk_level: "GREEN_READ".to_string(),
                    action_kind: None,
                });
            }
        }

        // 4. Project Continuation (Priority 6)
        if let Some(project) = projects.first() {
            let id = format!("continue_project_{}", project.name);
            if should_include(ProactiveCategory::ProjectContinuation, &id) {
                candidates.push(ProactiveSuggestion {
                    id,
                    category: ProactiveCategory::ProjectContinuation,
                    title: format!("Weiterarbeiten an {}", project.name),
                    human_message: format!(
                        "Möchtest du am Projekt '{}' weiterarbeiten? Empfohlene nächste Aktion: {}.",
                        project.name, project.recommended_action
                    ),
                    why_now: "Zuletzt geöffnetes Projekt steht zur Fortsetzung bereit.".to_string(),
                    clickable_options: vec![
                        "Projekt vorbereiten".to_string(),
                        "Später".to_string(),
                        "Thema stummschalten".to_string(),
                    ],
                    priority: 6,
                    risk_level: "YELLOW_PROJECT".to_string(),
                    action_kind: Some(ActionKind::LaunchProject),
                });
            }
        }

        // Sort descending by priority and truncate
        candidates.sort_by_key(|c| std::cmp::Reverse(c.priority));
        candidates.truncate(self.max_suggestions);

        ProactiveEvaluationReport {
            activity_state: activity,
            suppressed_by_focus_guard: false,
            suppression_reason: None,
            suggestions: candidates,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::windows_bridge::{GpuStatus, SystemHealthReport};

    fn dummy_health(cache_bytes: u64) -> SystemHealthReport {
        SystemHealthReport {
            timestamp: "2026-11-09T07:00:00Z".to_string(),
            gpu: GpuStatus {
                gpu_name: "RTX 5090".to_string(),
                vram_used_mib: 4000,
                vram_free_mib: 28000,
                vram_total_mib: 32000,
            },
            system_ram_free_mb: 32000,
            system_ram_total_mb: 64000,
            top_processes: Vec::new(),
            aido_temp_cache_path: "C:\\Temp".to_string(),
            aido_temp_cache_files: if cache_bytes > 0 { 2 } else { 0 },
            aido_temp_cache_bytes: cache_bytes,
            human_summary: "System OK".to_string(),
        }
    }

    fn dummy_project(name: &str, dirty: bool) -> ProjectEntry {
        ProjectEntry {
            name: name.to_string(),
            canonical_path: format!("C:\\Projects\\{}", name),
            stack: vec!["Rust".to_string(), "Tauri".to_string()],
            git_branch: "main".to_string(),
            git_status: if dirty { "dirty".to_string() } else { "clean".to_string() },
            recommended_action: "cargo check ausführen".to_string(),
            human_summary: "Ein Testprojekt".to_string(),
        }
    }

    #[test]
    fn focus_guard_suppresses_all_suggestions_during_fullscreen_or_calls() {
        let engine = ProactiveEngine::new();
        let health = dummy_health(1024 * 1024);
        let projects = vec![dummy_project("AIDO", true)];

        for state in [
            UserActivityState::FullscreenOrGaming,
            UserActivityState::VoiceRecordingOrCall,
            UserActivityState::QuietHoursActive,
        ] {
            let report = engine.evaluate_context(state, &health, &projects, 5, 512 * 1024);
            assert!(report.suppressed_by_focus_guard);
            assert!(report.suppression_reason.is_some());
            assert!(report.suggestions.is_empty());
        }
    }

    #[test]
    fn dismiss_backoff_and_category_muting_filter_suggestions() {
        let engine = ProactiveEngine::new();
        let health = dummy_health(512 * 1024);
        let projects = vec![dummy_project("AIDO", false)];

        // Initial evaluation in NormalIdle -> SystemMaintenance suggestion should appear
        let report = engine.evaluate_context(UserActivityState::NormalIdle, &health, &projects, 0, 0);
        assert!(!report.suppressed_by_focus_guard);
        assert!(report.suggestions.iter().any(|s| s.category == ProactiveCategory::SystemMaintenance));

        // Mute category -> must be filtered
        engine.mute_category(ProactiveCategory::SystemMaintenance);
        assert!(engine.is_category_muted(ProactiveCategory::SystemMaintenance));
        let report_muted = engine.evaluate_context(UserActivityState::NormalIdle, &health, &projects, 0, 0);
        assert!(!report_muted.suggestions.iter().any(|s| s.category == ProactiveCategory::SystemMaintenance));

        // Unmute category
        engine.unmute_category(ProactiveCategory::SystemMaintenance);
        assert!(!engine.is_category_muted(ProactiveCategory::SystemMaintenance));

        // Test dismiss backoff
        let count1 = engine.record_dismissal("system_cleanup_hint");
        assert_eq!(count1, 1);
        let report_dismissed_once = engine.evaluate_context(UserActivityState::NormalIdle, &health, &projects, 0, 0);
        assert!(report_dismissed_once.suggestions.iter().any(|s| s.id == "system_cleanup_hint"));

        // Second dismissal -> count >= 2 -> suppressed
        let count2 = engine.record_dismissal("system_cleanup_hint");
        assert_eq!(count2, 2);
        let report_dismissed_twice = engine.evaluate_context(UserActivityState::NormalIdle, &health, &projects, 0, 0);
        assert!(!report_dismissed_twice.suggestions.iter().any(|s| s.id == "system_cleanup_hint"));
    }

    #[test]
    fn generates_prioritized_german_suggestions_for_dirty_git_cleanup_and_memory() {
        let engine = ProactiveEngine::new();
        let health = dummy_health(256 * 1024);
        let projects = vec![
            dummy_project("AIDO-Core", true),
            dummy_project("AIDO-UI", false),
        ];

        let report = engine.evaluate_context(
            UserActivityState::NormalIdle,
            &health,
            &projects,
            4,
            512 * 1024,
        );

        assert!(!report.suppressed_by_focus_guard);
        assert!(report.suppression_reason.is_none());
        assert!(!report.suggestions.is_empty());
        assert!(report.suggestions.len() <= 3);

        // Check priority sorting descending
        for i in 0..report.suggestions.len() - 1 {
            assert!(report.suggestions[i].priority >= report.suggestions[i + 1].priority);
        }

        // Top suggestion should be DirtyGitWorkspace (priority 9)
        assert_eq!(report.suggestions[0].category, ProactiveCategory::DirtyGitWorkspace);
        assert_eq!(report.suggestions[0].id, "dirty_git_AIDO-Core");
        assert!(report.suggestions[0].human_message.contains("ungespeicherte Git-Änderungen"));

        // Second suggestion should be SystemMaintenance (priority 8)
        assert_eq!(report.suggestions[1].category, ProactiveCategory::SystemMaintenance);
        assert_eq!(report.suggestions[1].id, "system_cleanup_hint");

        // Third suggestion should be MemoryConsolidation (priority 7)
        assert_eq!(report.suggestions[2].category, ProactiveCategory::MemoryConsolidation);
        assert_eq!(report.suggestions[2].id, "memory_compaction_hint");
    }
}
