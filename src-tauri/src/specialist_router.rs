use crate::memory::MemoryLayer;
use crate::permission_gateway::{PermissionGateway, PrivacyMode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpecialistRole {
    AidoCore,
    SystemExpert,
    AppExpert,
    ProjectExpert,
    MemoryCurator,
    AutomationExpert,
    SecurityExpert,
    ResearchExpert,
}

impl SpecialistRole {
    pub fn title_de(&self) -> &'static str {
        match self {
            SpecialistRole::AidoCore => "AIDO Core",
            SpecialistRole::SystemExpert => "System-Experte",
            SpecialistRole::AppExpert => "Anwendungs-Experte",
            SpecialistRole::ProjectExpert => "Projekt-Experte",
            SpecialistRole::MemoryCurator => "Gedächtnis-Kurator",
            SpecialistRole::AutomationExpert => "Automatisierungs-Experte",
            SpecialistRole::SecurityExpert => "Sicherheits-Experte",
            SpecialistRole::ResearchExpert => "Recherche-Experte",
        }
    }

    pub fn description_de(&self) -> &'static str {
        match self {
            SpecialistRole::AidoCore => "Versteht Nutzerabsichten, plant und koordiniert",
            SpecialistRole::SystemExpert => "Windows, Hardware, Ressourcen und Fehleranalyse",
            SpecialistRole::AppExpert => "Anwendungen erkennen, verbinden und bedienen",
            SpecialistRole::ProjectExpert => "Projekte, Dateien, Git-Repositories und Entwicklungsumgebungen",
            SpecialistRole::MemoryCurator => "Erinnerungen aktualisieren und zusammenfassen",
            SpecialistRole::AutomationExpert => "Hintergrundaufgaben und wiederkehrende Abläufe",
            SpecialistRole::SecurityExpert => "Aktionen gegen Berechtigungsregeln prüfen",
            SpecialistRole::ResearchExpert => "Bei erlaubtem Internetzugriff Informationen recherchieren",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistDispatchPlan {
    pub user_query: String,
    pub selected_specialist: SpecialistRole,
    pub specialist_title_de: String,
    pub required_memory_layers: Vec<MemoryLayer>,
    pub allowed_in_current_privacy_mode: bool,
    pub requires_gateway_approval: bool,
    pub coordination_summary_de: String,
}

#[derive(Debug, Default)]
pub struct SpecialistRouter;

impl SpecialistRouter {
    pub fn new() -> Self {
        Self
    }

    pub fn list_all_specialists() -> Vec<(SpecialistRole, &'static str, &'static str)> {
        vec![
            (
                SpecialistRole::AidoCore,
                SpecialistRole::AidoCore.title_de(),
                SpecialistRole::AidoCore.description_de(),
            ),
            (
                SpecialistRole::SystemExpert,
                SpecialistRole::SystemExpert.title_de(),
                SpecialistRole::SystemExpert.description_de(),
            ),
            (
                SpecialistRole::AppExpert,
                SpecialistRole::AppExpert.title_de(),
                SpecialistRole::AppExpert.description_de(),
            ),
            (
                SpecialistRole::ProjectExpert,
                SpecialistRole::ProjectExpert.title_de(),
                SpecialistRole::ProjectExpert.description_de(),
            ),
            (
                SpecialistRole::MemoryCurator,
                SpecialistRole::MemoryCurator.title_de(),
                SpecialistRole::MemoryCurator.description_de(),
            ),
            (
                SpecialistRole::AutomationExpert,
                SpecialistRole::AutomationExpert.title_de(),
                SpecialistRole::AutomationExpert.description_de(),
            ),
            (
                SpecialistRole::SecurityExpert,
                SpecialistRole::SecurityExpert.title_de(),
                SpecialistRole::SecurityExpert.description_de(),
            ),
            (
                SpecialistRole::ResearchExpert,
                SpecialistRole::ResearchExpert.title_de(),
                SpecialistRole::ResearchExpert.description_de(),
            ),
        ]
    }

    pub fn route_intent(
        &self,
        gateway: &PermissionGateway,
        user_query: &str,
    ) -> SpecialistDispatchPlan {
        let q = user_query.to_lowercase();

        // Strict architectural rule (Masterplan Section 6 & 9):
        // Neither SecurityExpert nor any other specialist can self-grant execution permissions.
        // All permissions remain strictly guarded by the user-directed PermissionGateway.
        let (specialist, required_layers, requires_approval) = if q.contains("recherchiere")
            || q.contains("web")
            || q.contains("internet")
            || q.contains("suche online")
        {
            (
                SpecialistRole::ResearchExpert,
                vec![MemoryLayer::SoulMemory, MemoryLayer::SemanticMemory],
                false,
            )
        } else if q.contains("hardware")
            || q.contains("vram")
            || q.contains("cleanup")
            || q.contains("bereinigung")
        {
            (
                SpecialistRole::SystemExpert,
                vec![
                    MemoryLayer::SoulMemory,
                    MemoryLayer::SystemMemory,
                    MemoryLayer::EpisodicMemory,
                ],
                q.contains("cleanup") || q.contains("bereinigung") || q.contains("lösch"),
            )
        } else if q.contains("app")
            || q.contains("adapter")
            || q.contains("obs")
            || q.contains("blender")
        {
            (
                SpecialistRole::AppExpert,
                vec![
                    MemoryLayer::SoulMemory,
                    MemoryLayer::SystemMemory,
                    MemoryLayer::ProceduralMemory,
                ],
                false,
            )
        } else if q.contains("git")
            || q.contains("branch")
            || q.contains("cargo")
            || q.contains("repo")
            || q.contains("docker")
        {
            (
                SpecialistRole::ProjectExpert,
                vec![
                    MemoryLayer::SoulMemory,
                    MemoryLayer::ProjectMemory,
                    MemoryLayer::ProceduralMemory,
                ],
                false,
            )
        } else if q.contains("memory")
            || q.contains("erinnerung")
            || q.contains("merken")
            || q.contains("vergessen")
        {
            (
                SpecialistRole::MemoryCurator,
                vec![
                    MemoryLayer::SoulMemory,
                    MemoryLayer::EpisodicMemory,
                    MemoryLayer::SemanticMemory,
                    MemoryLayer::ProceduralMemory,
                ],
                false,
            )
        } else if q.contains("cron")
            || q.contains("jeden freitag")
            || q.contains("hintergrund")
            || q.contains("automatisierung")
        {
            (
                SpecialistRole::AutomationExpert,
                vec![
                    MemoryLayer::SoulMemory,
                    MemoryLayer::ProceduralMemory,
                    MemoryLayer::SystemMemory,
                ],
                false,
            )
        } else if q.contains("berechtigung")
            || q.contains("admin")
            || q.contains("sicherheit")
            || q.contains("privacy")
        {
            (
                SpecialistRole::SecurityExpert,
                vec![
                    MemoryLayer::SoulMemory,
                    MemoryLayer::UserMemory,
                    MemoryLayer::SystemMemory,
                ],
                true,
            )
        } else {
            (
                SpecialistRole::AidoCore,
                vec![
                    MemoryLayer::SoulMemory,
                    MemoryLayer::UserMemory,
                    MemoryLayer::EpisodicMemory,
                ],
                false,
            )
        };

        let allowed_in_current_privacy_mode = if specialist == SpecialistRole::ResearchExpert {
            gateway.privacy_mode() != PrivacyMode::PrivateOffline
        } else {
            true
        };

        let specialist_title = specialist.title_de().to_string();
        let coordination_summary = if !allowed_in_current_privacy_mode {
            format!(
                "AIDO Spezialist '{}' ist im Datenschutz-Modus {:?} blockiert: Keine ungenehmigten externen Netzwerkanfragen erlaubt.",
                specialist_title,
                gateway.privacy_mode()
            )
        } else {
            format!(
                "AIDO koordiniert intern: Anfrage an '{}' delegiert mit {} Memory-Layern.",
                specialist_title,
                required_layers.len()
            )
        };

        SpecialistDispatchPlan {
            user_query: user_query.to_string(),
            selected_specialist: specialist,
            specialist_title_de: specialist_title,
            required_memory_layers: required_layers,
            allowed_in_current_privacy_mode,
            requires_gateway_approval: requires_approval,
            coordination_summary_de: coordination_summary,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permission_gateway::{AccessMode, PermissionGateway, PrivacyMode};

    #[test]
    fn routes_queries_to_matching_internal_specialist_with_soul_and_domain_memory() {
        let gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);
        let router = SpecialistRouter::new();

        // 1. SystemExpert
        let plan_sys = router.route_intent(&gateway, "Bitte VRAM Auslastung und Cleanup Status prüfen");
        assert_eq!(plan_sys.selected_specialist, SpecialistRole::SystemExpert);
        assert!(plan_sys.required_memory_layers.contains(&MemoryLayer::SoulMemory));
        assert!(plan_sys.required_memory_layers.contains(&MemoryLayer::SystemMemory));

        // 2. ProjectExpert
        let plan_proj = router.route_intent(&gateway, "Neuen Git Branch anlegen und Cargo Build starten");
        assert_eq!(plan_proj.selected_specialist, SpecialistRole::ProjectExpert);
        assert!(plan_proj.required_memory_layers.contains(&MemoryLayer::SoulMemory));
        assert!(plan_proj.required_memory_layers.contains(&MemoryLayer::ProjectMemory));
        assert!(plan_proj.required_memory_layers.contains(&MemoryLayer::ProceduralMemory));

        // 3. MemoryCurator
        let plan_mem = router.route_intent(&gateway, "Erinnerung an das Meeting morgen merken");
        assert_eq!(plan_mem.selected_specialist, SpecialistRole::MemoryCurator);
        assert!(plan_mem.required_memory_layers.contains(&MemoryLayer::SoulMemory));
        assert!(plan_mem.required_memory_layers.contains(&MemoryLayer::EpisodicMemory));

        // 4. AppExpert
        let plan_app = router.route_intent(&gateway, "OBS Adapter aktivieren");
        assert_eq!(plan_app.selected_specialist, SpecialistRole::AppExpert);
        assert!(plan_app.required_memory_layers.contains(&MemoryLayer::SoulMemory));

        // 5. AutomationExpert
        let plan_auto = router.route_intent(&gateway, "Jeden Freitag automatische Hintergrund Datensicherung planen");
        assert_eq!(plan_auto.selected_specialist, SpecialistRole::AutomationExpert);
        assert!(plan_auto.required_memory_layers.contains(&MemoryLayer::SoulMemory));
        assert!(plan_auto.required_memory_layers.contains(&MemoryLayer::ProceduralMemory));

        // 6. SecurityExpert
        let plan_sec = router.route_intent(&gateway, "Sicherheit und Admin Berechtigung prüfen");
        assert_eq!(plan_sec.selected_specialist, SpecialistRole::SecurityExpert);
        assert!(plan_sec.required_memory_layers.contains(&MemoryLayer::SoulMemory));
        assert!(plan_sec.requires_gateway_approval);

        // 7. AidoCore (General)
        let plan_core = router.route_intent(&gateway, "Guten Morgen AIDO, wie ist der Gesamtstatus?");
        assert_eq!(plan_core.selected_specialist, SpecialistRole::AidoCore);
        assert!(plan_core.required_memory_layers.contains(&MemoryLayer::SoulMemory));
        assert!(plan_core.required_memory_layers.contains(&MemoryLayer::UserMemory));

        // Verify list_all_specialists
        let all = SpecialistRouter::list_all_specialists();
        assert_eq!(all.len(), 8);
    }

    #[test]
    fn research_expert_is_blocked_in_private_offline_mode_by_gateway_policy() {
        let gateway = PermissionGateway::new(AccessMode::StandardAccess, PrivacyMode::PrivateOffline);
        let router = SpecialistRouter::new();

        let plan_offline = router.route_intent(&gateway, "Recherchiere die neuesten AI News im Web");
        assert_eq!(plan_offline.selected_specialist, SpecialistRole::ResearchExpert);
        assert!(!plan_offline.allowed_in_current_privacy_mode);
        assert!(plan_offline.coordination_summary_de.contains("blockiert"));

        // Switch to LocalConnected or HybridAI
        gateway.set_privacy_mode(PrivacyMode::HybridAI);
        let plan_online = router.route_intent(&gateway, "Recherchiere die neuesten AI News im Web");
        assert_eq!(plan_online.selected_specialist, SpecialistRole::ResearchExpert);
        assert!(plan_online.allowed_in_current_privacy_mode);
        assert!(plan_online.coordination_summary_de.contains("Recherche-Experte"));
    }
}
