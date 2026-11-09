# Projektstatus — AIDO (Agentic Intelligent Developing Operator)

**Stand: 9. November 2026.** Dieser Status trennt Repository-Artefakte von verifizierten Messungen am Referenz-PC.

## Repository

- `keokukzh/AIDO` (**Agentic Intelligent Developing Operator**) ist ein privates GitHub-Repository und lokal in `C:\Users\keoku\Desktop\AIDO` eingerichtet.
- Der vollständige Produkt-Masterplan v1.0 ist unter [MASTERPLAN.md](MASTERPLAN.md) hinterlegt.
- Die lokale Modell- und Runtime-Validierung (Text, Vision, Speculative Decoding, Schlafmodus/Idle-Unload, Embeddings und deutsche Spracherkennung) ist vollständig abgeschlossen und mit SHA-256-Prüfsummen im [Modellinventar](MODEL_RUNTIME_INVENTORY.md) dokumentiert.
- Der **AIDO-Kern-Prototyp (Phasen 1 bis 6)** ist implementiert und mit **35 automatisierten Rust-Tests** (`cargo test` + `cargo clippy -- -D warnings`, 0 Warnungen) sowie einem 8-stufigen End-to-End-Durchstich (`aido-cli demo`) verifiziert:
  - `permission_gateway.rs`: Zugriffskontrolle (`ReadOnly`, `StandardAccess`, `FullAccess`), `PrivateOffline` Loopback-Schutz (`127.0.0.1`) und kryptografische Einmal-Freigabetokens (`ApprovalToken`).
  - `windows_bridge.rs`: Live-Hardware-Sensorik (`nvidia-smi` RTX 5090 VRAM, RAM, Top-Prozesse) und zweistufiges System-Cleanup (**Fast Cleanup** für Sitzungs-Caches & **Deep Cleanup** für verwaiste Build-/Shader-/Crash-Dump-Artefakte) nach Nutzerfreigabe.
  - `app_intelligence.rs`: **App Discovery & Capability Registry** mit SQLite-Persistenz (`app_registry`), Installer-Checkbox-Steuerung und Risiko-/Freigabeklassifizierung pro Fähigkeit (*Docker Desktop*, *Cursor / VS Code*, *LM Studio*, *Google Chrome*).
  - `project_intelligence.rs`: **Project Intelligence & Kanonischer Projektkatalog** mit Stack-Erkennung, Git-Branch/Status-Prüfung, FTS5-Volltextsuche (`project_catalog_fts`) und kontrolliertem Projekt-Start-Workflow (`LaunchProject`).
  - `smart_installer.rs`: **Smart Installer, Hardware-Klassen-Erkennung & Transaktionale Einrichtung** mit 4 Hardware-Tiers (`HighEndGpu`, `MidRangeGpu`, `CpuFocused`, `LowSpecCloudFallback`), automatischer Wiederverwendung bereits vorhandener lokaler Modelle (`0 GB` Download auf dem Referenz-PC), 3-stufiger ehrlicher ETA-Prognose und 9 transaktionalen SQLite-Installationsphasen (`installer_steps`) inkl. Pause/Resume und zerstörungsfreiem Rollback.
  - `runtime_supervisor.rs` & `ai_client.rs`: **Live-Runtime-Supervisor, Vision & ASR-Orchestrierung** für `llama-server.exe` (`127.0.0.1:18434`, `--sleep-idle-seconds 300` Zero-VRAM-Standby, `mmproj` Vision, `DFlash2` Speculative Decoding) und `whisper-server.exe` (`127.0.0.1:18080`, `ggml-small.bin`, CUDA 13.4 DLL-Auflösung via `C:\AI\llamacpp-cuda`).
  - `memory.rs`: **4-Stufen-Gedächtnis & Kompaktierungs-Engine** (`WorkingMemory`, `EpisodicMemory`, `SemanticMemory`, `SystemMemory`) mit SQLite/FTS5-Volltextsuche, Hintergrund-Verdichtung (`compact_episodic_memories`), Nutzer-Kontrolle zum Vergessen einzelner Einträge (`forget_memory_entry`) und lückenlosem `audit_log`.
  - `proactive_engine.rs`: **Proaktive Vordenker-Engine (*„AIDO denkt für dich“*)** mit striktem Fokus-Schutz (`FullscreenOrGaming`, `VoiceRecordingOrCall`, `QuietHoursActive` -> 0 Unterbrechungen), 2x-Ablehnungs-Backoff, Themen-Stummschaltung und priorisierten Vorschlägen (`DirtyGitWorkspace`, `SystemMaintenance`, `MemoryConsolidation`, `ProjectContinuation`).
  - `index.html` & `src/main.ts`: Animierter Desktop-Charakter mit Chat, Freigabe-Karten, App-Intelligence-, Projekt-, Smart-Installer-, Runtime/Memory-Kompaktierungs-, Proaktive-Vordenker- und Deep-Cleanup-Ansicht, Push-to-Talk, Screenshot-Vorschau und interaktivem Privacy/Memory Center.

## Verifizierte Messungen am Referenz-PC (RTX 5090, 32607 MiB VRAM)

- **Hauptmodell + Vision + Speculative Decoding (`llama-server.exe` v0.9.0-dev Build 12176):**
  - `D:\AI_Models\llm\Qwen3.8-27B-UD-Q4_K_M.gguf` + `D:\AI_Models\llm\assets\mmproj-Qwen3.8-27B-BF16.gguf` + `D:\AI_Models\llm\assets\Qwen3.8-27B-DFlash2-Q4_K_M.gguf` laufen im lokalen Router-Modus auf `127.0.0.1:18434`.
  - **Schlafmodus & Autoload:** Im Ruhezustand belegt der Router `0 MiB` zusätzlichen VRAM (`2.307 MiB` System-VRAM). Bei der ersten Anfrage lädt er Modell, Projektor und Draft-Modell in `9,41 s` (`25.544 MiB` VRAM bei 32k Kontext) und gibt nach Ablauf von `--sleep-idle-seconds` automatisch **`22.714 MiB` VRAM** wieder frei (Rückgang auf `2.830 MiB`; Wiederaufwachen in `8,26 s`).
  - **Geschwindigkeit & Vision:** Warme Textgenerierung mit `draft-dflash` erreichte **`151,54 Token/s`**; ein künstliches zweifarbiges Testbild wurde mit **`102,70 Token/s`** korrekt als links rot und rechts blau erkannt (inklusive Gegenprobe ohne Bild).
- **Deutsche Spracherkennung (`whisper-server.exe` v1.9.5-unsloth.18):**
  - Mit den lokalen CUDA-13-Bibliotheken aus `C:\AI\llamacpp-cuda` im Prozess-`PATH` erkennt `whisper-server.exe` die RTX 5090 (`CUDA0`, `+1.150 MiB` VRAM) und transkribiert ein 6,9 Sekunden langes deutsches Test-WAV (`ggml-small.bin`) lokal auf `127.0.0.1:18080` in **`0,341 s`**.
- **Text-Embeddings:**
  - `text-embedding-nomic-embed-text-v1.5` (768 Dimensionen) lieferte im deutschen Ähnlichkeitstest `0,8036` für verwandte Projekttexte gegenüber `0,5959` für themenfremden Text.

## Abgeschlossene Phasen (Phasen 0–6) & Nächster Meilenstein (Phase 7/8)

- **Phasen 0–6 abgeschlossen:** Runtime-Validierung, Permission Gateway, Windows Bridge (Fast & Deep Cleanup), SQLite/FTS5-Memory (4-Stufen-Kompaktierung & Vergessen), App Discovery & Capability Registry, Kanonischer Projektkatalog, 9-stufiger Smart Installer, Live-Runtime-Supervisor sowie die Proaktive Vordenker-Engine mit Fokus-Schutz sind vollständig implementiert und getestet (35/35 Tests grün).
- **Nächster Meilenstein (Phase 7 & 8):** Isolierte Adapter-Sandbox (Unbekannte Apps sicher prüfen & verifizieren) sowie Packaging, Saubere Deinstallation und Release-Readiness.
