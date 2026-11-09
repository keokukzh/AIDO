# Entwicklungsroadmap

Die Phasen sind in Abhängigkeitsreihenfolge geplant. Abgeschlossene Nachweise sind in [PROJECT_STATUS.md](PROJECT_STATUS.md) und [MODEL_RUNTIME_INVENTORY.md](MODEL_RUNTIME_INVENTORY.md) dokumentiert.

| Phase | Ziel | Status / Nachweis |
| --- | --- | --- |
| 0 | Referenz-PC, vorhandene Modelllaufzeiten und Modelle gezielt inventarisieren; historischen Docker-Plan einordnen. | **Abgeschlossen.** Runtimes, SHA-256-Prüfsummen, Vision, DFlash2, Idle-Sleep und CUDA-Whisper verifiziert. |
| 1 | Windows Bridge, Permission Gateway, lokales SQLite/FTS5-Memory und erster Desktop-Prototyp (System-Gesundheitscheck & Fast Cleanup). | **Abgeschlossen.** Erlaubte, abgelehnte und bestätigungspflichtige Aktionen reproduzierbar getrennt und getestet. |
| 2 | App Discovery, Capability Registry und Project Intelligence (Kanonischer Projektkatalog). | **Abgeschlossen.** Apps und Projekte werden erkannt, in SQLite/FTS5 gespeichert und mit Rechten, Risiko und Freigabestatus verwaltet. |
| 3 | Smart Installer, Hardware-Klassen-Erkennung, Modell-Wiederverwendung und transaktionale 9-Phasen-Einrichtung. | **Abgeschlossen.** 4 Hardware-Tiers, 0-GB-Modell-Wiederverwendung, 3-stufige ETA-Prognose, SQLite-Resume & Rollback. |
| 4 | Vollständige Live-Orchestrierung von Hauptmodell, Vision, Whisper-ASR und 4-stufiger Memory-Kompaktierung. | **Abgeschlossen.** `RuntimeSupervisor` (`--sleep-idle-seconds 300`, CUDA 13.4 DLLs), multimodale Vision/ASR-Pipeline und 4-Stufen-Gedächtniskompaktierung inkl. Nutzer-Vergessen. |
| 5 | Ausbau Charakter-Interaktion, verständliches Onboarding und Privacy/Memory Center. | **Abgeschlossen.** 4-Stufen-Layer-Filter, Kompaktierungs-Trigger, Vergessen-Steuerung und Fokus-Schutz integriert. |
| 6 | Proaktive Vorschläge (Vordenker-Engine) und zweistufiges System-Cleanup (*Fast Cleanup* & *Deep Cleanup*). | **Abgeschlossen.** `ProactiveEngine` mit Vollbild-/Call-Fokussperre, 2x-Ablehnungs-Backoff, Themen-Stummschaltung und *Deep Cleanup* mit Freigabetoken. |
| 7 | Breitere App-Abdeckung und isoliert geprüfte Adaptererzeugung. | **Abgeschlossen.** Isolierte Adapter-Sandbox (`synthesize_and_verify_unknown_app_adapter`) prüft unbekannte Apps (z. B. OBS Studio / Blender 3D) und trennt harmlose von bestätigungspflichtigen Fähigkeiten. |
| 8 | Verpackung, Updates, saubere Deinstallation und Releasevorbereitung. | **Abgeschlossen.** Zero-Destruction Clean Uninstaller (`prepare_clean_uninstall` / `execute_clean_uninstall` schützt externe KI-Modelle & Projekte zu 100 %) und 8-Phasen-Release-Readiness (`40/40` Tests grün). |
| 9 | Agent Remote: Minimalistisches QR- & Smartphone-Daumen-Cockpit mit 100 % PC-Parität (Ende-zu-Ende verschlüsselte lokale LAN-Kopplung mit 60s TTL / 4-stelliger PIN, automatische Umschaltung von `PrivateOffline` zu `LocalConnected`, 1-Tap Daumen-Freigabekarte, 2×2 Schnell-Steuerdeck mit Inline AIDO-Blick Vision Snapshot, monochromer S/W- & W/S-Theme-Toggle und Whisper CUDA Push-to-Talk). | **✅ Abgeschlossen.** Alle 46/46 Rust-Tests grün, vollständige Frontend-Integration und Zero-Cloud-Sicherheit verifiziert. |

## Zusammenfassung (Alle 9 Phasen abgeschlossen)

- **46/46 automatisierte Rust-Tests** (`cargo test --manifest-path src-tauri/Cargo.toml`) und **0 Clippy-Warnungen** (`cargo clippy --all-targets -- -D warnings`).
- **10-stufiger End-to-End-CLI-Durchstich** (`cargo run --bin aido-cli -- demo`) verifiziert alle Kernmodule inklusive Agent Remote live auf dem Referenz-PC.
- **Produktionsreifer Frontend-Build** (`npm run build`) für die interaktive Desktop-Oberfläche mit minimalistischem S/W- & W/S-Remote-Cockpit.
