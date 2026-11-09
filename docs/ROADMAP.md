# Entwicklungsroadmap

Die Phasen sind in Abhängigkeitsreihenfolge geplant. Abgeschlossene Nachweise sind in [PROJECT_STATUS.md](PROJECT_STATUS.md) und [MODEL_RUNTIME_INVENTORY.md](MODEL_RUNTIME_INVENTORY.md) dokumentiert.

| Phase | Ziel | Status / Nachweis |
| --- | --- | --- |
| 0 | Referenz-PC, vorhandene Modelllaufzeiten und Modelle gezielt inventarisieren; historischen Docker-Plan einordnen. | **Abgeschlossen.** Runtimes, SHA-256-Prüfsummen, Vision, DFlash2, Idle-Sleep und CUDA-Whisper verifiziert. |
| 1 | Windows Bridge, Permission Gateway, lokales SQLite/FTS5-Memory und erster Desktop-Prototyp (System-Gesundheitscheck & Fast Cleanup). | **Abgeschlossen.** Erlaubte, abgelehnte und bestätigungspflichtige Aktionen reproduzierbar getrennt und getestet. |
| 2 | App Discovery, Capability Registry und Project Intelligence (Kanonischer Projektkatalog). | **Abgeschlossen.** Apps und Projekte werden erkannt, in SQLite/FTS5 gespeichert und mit Rechten, Risiko und Freigabestatus verwaltet (16/16 Tests grün). |
| 3 | Smart Installer und Datenmodi. | **Nächster Meilenstein:** Geführter Installer-Flow (Hardwareprüfung, Modellwahl, App-Auswahl-Wizard, dynamische ETA-Berechnung). |
| 4 | Vollständige Integration von Hauptmodell, Vision, Whisper-ASR und Embeddings in die Live-Oberfläche. | **Runtime-Kette verifiziert;** UI-/Backend-Anbindung Teil des Prototyps. |
| 5 | Ausbau 3D-Charakter, verständliches Onboarding und Privacy/Memory Center. | Basis-Charakter & Privacy/Memory-Ansicht im MVP-Prototyp enthalten. |
| 6 | Proaktive Vorschläge und geplante Hintergrundaufgaben. | Geplant nach Abschluss des MVP-Durchstichs. |
| 7 | Breitere App-Abdeckung und isoliert geprüfte Adaptererzeugung. | Geplant. |
| 8 | Verpackung, Updates, Deinstallation und Releasevorbereitung. | Geplant. |

## Unmittelbare Schritte

1. **Abgeschlossen:** Lokale Modell- und Runtime-Validierung (`Qwen3.8-27B-UD-Q4_K_M` + `mmproj-BF16` + `DFlash2` + `--sleep-idle-seconds` auf `llama-server.exe` Build 12176 sowie `whisper-server.exe` + `ggml-small.bin` auf CUDA 13.4).
2. **Aktuell:** Aufbau und automatisierter Test des AIDO-Kern-Prototyps (`src-tauri` + `src`) für den End-to-End-Ablauf:
   - *System-Gesundheitscheck (lesend)* → *verständliche Erklärung & Vorschau* → *explizite Nutzerfreigabe im Permission Gateway* → *kontrolliertes Fast Cleanup* → *Speicherung im lokalen SQLite/FTS5-Memory & Audit-Log*.
3. **Nächster Meilenstein nach MVP-Durchstich:** App Discovery & Capability Registry (Phase 2) für installierte Windows-Anwendungen und bestehende Projekte aus dem Projektindex.
