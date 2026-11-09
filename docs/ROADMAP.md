# Entwicklungsroadmap

Die Phasen sind in Abhängigkeitsreihenfolge geplant. Abgeschlossene Nachweise sind in [PROJECT_STATUS.md](PROJECT_STATUS.md) und [MODEL_RUNTIME_INVENTORY.md](MODEL_RUNTIME_INVENTORY.md) dokumentiert.

| Phase | Ziel | Status / Nachweis |
| --- | --- | --- |
| 0 | Referenz-PC, vorhandene Modelllaufzeiten und Modelle gezielt inventarisieren; historischen Docker-Plan einordnen. | **Abgeschlossen.** Runtimes, SHA-256-Prüfsummen, Vision, DFlash2, Idle-Sleep und CUDA-Whisper verifiziert. |
| 1 | Windows Bridge, Permission Gateway, lokales SQLite/FTS5-Memory und erster Desktop-Prototyp (System-Gesundheitscheck & Fast Cleanup). | **Abgeschlossen.** Erlaubte, abgelehnte und bestätigungspflichtige Aktionen reproduzierbar getrennt und getestet. |
| 2 | App Discovery, Capability Registry und Project Intelligence (Kanonischer Projektkatalog). | **Abgeschlossen.** Apps und Projekte werden erkannt, in SQLite/FTS5 gespeichert und mit Rechten, Risiko und Freigabestatus verwaltet. |
| 3 | Smart Installer, Hardware-Klassen-Erkennung, Modell-Wiederverwendung und transaktionale 9-Phasen-Einrichtung. | **Abgeschlossen.** 4 Hardware-Tiers, 0-GB-Modell-Wiederverwendung, 3-stufige ETA-Prognose, SQLite-Resume & Rollback. |
| 4 | Vollständige Live-Orchestrierung von Hauptmodell, Vision, Whisper-ASR und 4-stufiger Memory-Kompaktierung. | **Abgeschlossen.** `RuntimeSupervisor` (`--sleep-idle-seconds 300`, CUDA 13.4 DLLs), multimodale Vision/ASR-Pipeline und 4-Stufen-Gedächtniskompaktierung inkl. Nutzer-Vergessen (30/30 Tests grün). |
| 5 | Ausbau 3D-Charakter, verständliches Onboarding und Privacy/Memory Center. | **Basis & erweitertes Privacy/Memory Center abgeschlossen** (4-Stufen-Layer-Filter, Kompaktierung, Vergessen-Steuerung). |
| 6 | Proaktive Vorschläge (Vordenker-Engine), Deep Cleanup und geplante Hintergrundaufgaben. | **Nächster Meilenstein:** Proaktive Initiative-Engine (Kontextvorschläge ohne Unterbrechung, Deep Cleanup mit Vorher/Nachher-Freigabe). |
| 7 | Breitere App-Abdeckung und isoliert geprüfte Adaptererzeugung. | Geplant. |
| 8 | Verpackung, Updates, Deinstallation und Releasevorbereitung. | Geplant. |

## Unmittelbare Schritte

1. **Abgeschlossen (Phasen 0–4):**
   - Lokale Modell- und Runtime-Validierung (`Qwen3.8-27B-UD-Q4_K_M` + `mmproj-BF16` + `DFlash2` + `--sleep-idle-seconds` sowie `whisper-server.exe` + `ggml-small.bin`).
   - Permission Gateway, Windows Bridge, App Discovery & Capability Registry, Project Intelligence, 9-stufiger transaktionaler Smart Installer sowie `RuntimeSupervisor` und 4-stufige Memory-Kompaktierung (`30/30` Tests grün, 7-Schritt-CLI-Demo).
2. **Nächster Meilenstein (Phase 6 — Proaktive Vordenker-Engine & Deep Cleanup):**
   - Proaktive Initiative-Logik (*„AIDO denkt für dich“*), die auf Basis von Systemzustand, offenen Projekten und Erinnerungen hilfreiche Vorschläge mit Ruhezeiten/Drosselregeln generiert, sowie zweistufiges System-Cleanup (*Fast Cleanup* & *Deep Cleanup*).
