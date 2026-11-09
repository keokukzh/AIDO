# AIDO — Agentic Intelligent Developing Operator

**AIDO** (**Agentic Intelligent Developing Operator**) ist ein persönlicher, animierter Desktop-Begleiter für Windows 11, der Menschen ohne technische Vorkenntnisse bei der Bedienung, Organisation, Entwicklung und Pflege ihres Computers unterstützt.

> **„AIDO denkt für dich. Nicht du für AIDO.“**  
> **„Dein PC. Dein AIDO. Deine Daten.“**

## Projektstand (9. November 2026)

1. **Lokale Modell- & Runtime-Kette verifiziert (Phase 0 & 1 abgeschlossen):**
   - **Hauptmodell & Vision:** `Qwen3.8-27B-UD-Q4_K_M.gguf` + `mmproj-Qwen3.8-27B-BF16.gguf` + `Qwen3.8-27B-DFlash2-Q4_K_M.gguf` auf `llama-server.exe` (`v0.9.0-dev` Build 12176, CUDA 13.4) mit **151,54 Token/s** (Text warm), **102,70 Token/s** (Vision) und automatischer VRAM-Freigabe im Schlafmodus (`--sleep-idle-seconds`, **22,7 GB VRAM** werden im Leerlauf automatisch freigegeben).
   - **Deutsche Spracherkennung (ASR):** `whisper-server.exe` (`v1.9.5-unsloth.18`) + `ggml-small.bin` auf CUDA 13.4 (**0,341 s** Latenz für 6,9 s deutsches Audio).
   - **Lokale Text-Embeddings:** `text-embedding-nomic-embed-text-v1.5` (768 Dimensionen).
2. **Ausführbarer AIDO-Kern-Prototyp (`src-tauri/` & `src/`, Phasen 1–3 abgeschlossen, 22/22 Tests grün):**
   - **Permission Gateway (`src-tauri/src/permission_gateway.rs`):** Erzwingt die Trennung zwischen KI-Vorschlag und Ausführung (`ReadOnly`, `StandardAccess`, `FullAccess`), prüft Einmal-Freigabetokens (`ApprovalToken`) und blockiert im Modus `PrivateOffline` technisch jede Nicht-Loopback-Verbindung (`!= 127.0.0.1`).
   - **Windows Bridge (`src-tauri/src/windows_bridge.rs`):** Liest GPU-/VRAM-Status (`nvidia-smi`), Systemspeicher (RAM), Top-Prozesse und führt nach expliziter Nutzerfreigabe ein kontrolliertes *Fast Cleanup* im isolierten AIDO-Cache-Bereich aus.
   - **App Discovery & Capability Registry (`src-tauri/src/app_intelligence.rs`):** Erkennt installierte Programme (*Docker Desktop*, *Cursor / VS Code*, *LM Studio*, *Google Chrome*), speichert deren Fähigkeiten in SQLite (`app_registry`) und trennt lesende von bestätigungspflichtigen Aktionen.
   - **Project Intelligence & Kanonischer Projektkatalog (`src-tauri/src/project_intelligence.rs`):** Indiziert Projekte (`AIDO`, `aidsec` usw.) mit Stack-Erkennung, Git-Branch/Status und FTS5-Volltextsuche (`project_catalog_fts`).
   - **Smart Installer & ETA Engine (`src-tauri/src/smart_installer.rs`):** Klassifiziert die Hardware (`HighEndGpu` bis `LowSpecCloudFallback`), erkennt vorhandene lokale GGUF/Whisper-Modelle zur sofortigen Wiederverwendung (`0 GB` Download auf dem Referenz-PC), berechnet die 3-stufige ehrliche ETA-Prognose und verwaltet 9 transaktionale Installationsphasen in SQLite (`installer_steps`) inkl. Pause/Resume und zerstörungsfreiem Rollback.
   - **Memory Engine (`src-tauri/src/memory.rs`):** Lokale SQLite-Datenbank mit **FTS5-Volltextsuche** und lückenlosem `audit_log`.
   - **Desktop-Oberfläche (`index.html`, `src/main.ts`):** Animierter Charakter mit Chat, Freigabe-Karte, App-Intelligence-, Projekt- und Smart-Installer-Wizard-Ansicht, Push-to-Talk, Screenshot-Kontext, Rechtsklick-Menü und Privacy/Memory Center.

## Schnellstart (Kern-Prototyp & Tests)

```powershell
# Rust Unit- & Integrationstests ausführen (22 Tests: Gateway, Bridge, Memory, Apps, Projekte, Smart Installer)
cd src-tauri
cargo test

# 6-stufige End-to-End-Demo (Systemcheck -> Fast Cleanup -> Memory -> Apps -> Projekte -> Smart Installer)
cargo run --bin aido-cli -- demo

# Smart-Installer-Plan als JSON ausgeben
cargo run --bin aido-cli -- installer
```

## Dokumentation

| Dokument | Inhalt |
| --- | --- |
| [Masterplan](docs/MASTERPLAN.md) | Vollständiger Produkt-Masterplan v1.0 |
| [Vision](docs/VISION.md) | Produktidee, Zielgruppe und Grenzen |
| [Ziele](docs/GOALS.md) | Ziele und messbare Erfolgskriterien |
| [Anforderungen](docs/REQUIREMENTS.md) | Geplante Funktionen und Schutzanforderungen |
| [Architektur](docs/ARCHITECTURE.md) | Komponenten und technische Kandidaten |
| [Projektstatus](docs/PROJECT_STATUS.md) | Verifizierter Stand und Testergebnisse |
| [Entwicklungsumgebung](docs/DEVELOPMENT_SETUP.md) | Früher Docker-Plan, Ist-Check und Übernahme für AIDO |
| [Modellinventar](docs/MODEL_RUNTIME_INVENTORY.md) | Verifizierte Laufzeiten, SHA-256-Prüfsummen und Benchmarks |
| [Roadmap](docs/ROADMAP.md) | Entwicklungsreihenfolge und Abnahme |
| [Entscheidungen](docs/DECISIONS.md) | Beschlüsse aus den AIDO-Interviews |
