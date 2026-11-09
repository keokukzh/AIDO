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
| 6 | Proaktive Vorschläge (Vordenker-Engine) und zweistufiges System-Cleanup (*Fast Cleanup* & *Deep Cleanup*). | **Abgeschlossen.** `ProactiveEngine` mit Vollbild-/Call-Fokussperre, 2x-Ablehnungs-Backoff, Themen-Stummschaltung und *Deep Cleanup* mit Freigabetoken (35/35 Tests grün). |
| 7 | Breitere App-Abdeckung und isoliert geprüfte Adaptererzeugung. | **Nächster Meilenstein:** Isolierte Adapter-Prüfbank für unbekannte Programme. |
| 8 | Verpackung, Updates, saubere Deinstallation und Releasevorbereitung. | Geplant nach Phase 7. |

## Unmittelbare Schritte

1. **Abgeschlossen (Phasen 0–6):**
   - Lokale Modell- und Runtime-Validierung (`Qwen3.8-27B-UD-Q4_K_M` + `mmproj-BF16` + `DFlash2` + `--sleep-idle-seconds` sowie `whisper-server.exe` + `ggml-small.bin`).
   - Permission Gateway, Windows Bridge (Fast & Deep Cleanup), App Discovery & Capability Registry, Project Intelligence, 9-stufiger Smart Installer, `RuntimeSupervisor`, 4-stufige Memory-Kompaktierung und `ProactiveEngine` (`35/35` Tests grün, 8-Schritt-CLI-Demo).
2. **Nächster Meilenstein (Phase 7 & 8):**
   - Isolierte Adapter-Generierung & -Verifikation für unbekannte Programme sowie sauberes Release-/Uninstaller-Profil (Zero-Destruction für bestehende Nutzerdaten & Modelle).
