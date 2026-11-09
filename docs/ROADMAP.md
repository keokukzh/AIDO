# Entwicklungsroadmap

Die Phasen sind in Abhängigkeitsreihenfolge geplant. Abgeschlossene Nachweise sind in [PROJECT_STATUS.md](PROJECT_STATUS.md) und [MODEL_RUNTIME_INVENTORY.md](MODEL_RUNTIME_INVENTORY.md) dokumentiert.

| Phase | Ziel | Status / Nachweis |
| --- | --- | --- |
| 0 | Referenz-PC, vorhandene Modelllaufzeiten und Modelle gezielt inventarisieren; historischen Docker-Plan einordnen. | **Abgeschlossen.** Runtimes, SHA-256-Prüfsummen, Vision, DFlash2, Idle-Sleep und CUDA-Whisper verifiziert. |
| 1 | Windows Bridge, Permission Gateway, lokales SQLite/FTS5-Memory und erster Desktop-Prototyp (System-Gesundheitscheck & Fast Cleanup). | **Abgeschlossen.** Erlaubte, abgelehnte und bestätigungspflichtige Aktionen reproduzierbar getrennt und getestet. |
| 2 | App Discovery, Capability Registry und Project Intelligence (Kanonischer Projektkatalog). | **Abgeschlossen.** Apps und Projekte werden erkannt, in SQLite/FTS5 gespeichert und mit Rechten, Risiko und Freigabestatus verwaltet. |
| 3 | Smart Installer, Hardware-Klassen-Erkennung, Modell-Wiederverwendung und transaktionale 9-Phasen-Einrichtung. | **Abgeschlossen.** 4 Hardware-Tiers, 0-GB-Modell-Wiederverwendung, 3-stufige ETA-Prognose, SQLite-Resume & Rollback (22/22 Tests grün). |
| 4 | Vollständige Live-Orchestrierung von Hauptmodell, Vision, Whisper-ASR und mehrstufiger Memory-Kompaktierung. | **Nächster Meilenstein:** Automatisches Prozess-Management für `llama-server.exe` & `whisper-server.exe`, Live-Vision-Capture und 4-Stufen-Memory-Verdichtung. |
| 5 | Ausbau 3D-Charakter, verständliches Onboarding und Privacy/Memory Center. | Basis-Charakter & Privacy/Memory-Ansicht im Prototyp enthalten. |
| 6 | Proaktive Vorschläge (Vordenker-Engine) und geplante Hintergrundaufgaben. | Geplant nach Phase 4/5. |
| 7 | Breitere App-Abdeckung und isoliert geprüfte Adaptererzeugung. | Geplant. |
| 8 | Verpackung, Updates, Deinstallation und Releasevorbereitung. | Geplant. |

## Unmittelbare Schritte

1. **Abgeschlossen (Phasen 0–3):**
   - Lokale Modell- und Runtime-Validierung (`Qwen3.8-27B-UD-Q4_K_M` + `mmproj-BF16` + `DFlash2` + `--sleep-idle-seconds` sowie `whisper-server.exe` + `ggml-small.bin`).
   - Permission Gateway, Windows Bridge, SQLite/FTS5-Memory, App Discovery & Capability Registry, Project Intelligence und der 9-stufige transaktionale Smart Installer (`22/22` Tests grün, 6-Schritt-CLI-Demo).
2. **Nächster Meilenstein (Phase 4):**
   - Live-Runtime-Supervisor für `llama-server.exe` (`127.0.0.1:18434`) und `whisper-server.exe` (`127.0.0.1:18080`) inkl. On-Demand-Start/Schlafmodus-Überwachung und mehrstufiger Hintergrund-Kompaktierung im Memory-System.
