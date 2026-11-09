# Projektstatus — AIDO (Agentic Intelligent Developing Operator)

**Stand: 9. November 2026.** Dieser Status trennt Repository-Artefakte von verifizierten Messungen am Referenz-PC.

## Repository

- `keokukzh/AIDO` (**Agentic Intelligent Developing Operator**) ist ein privates GitHub-Repository und lokal in `C:\Users\keoku\Desktop\AIDO` eingerichtet.
- Der vollständige Produkt-Masterplan v1.0 ist unter [MASTERPLAN.md](MASTERPLAN.md) hinterlegt.
- Die lokale Modell- und Runtime-Validierung (Text, Vision, Speculative Decoding, Schlafmodus/Idle-Unload, Embeddings und deutsche Spracherkennung) ist vollständig abgeschlossen und mit SHA-256-Prüfsummen im [Modellinventar](MODEL_RUNTIME_INVENTORY.md) dokumentiert.
- Der **AIDO-Kern-Prototyp (Phase 1 & Phase 2)** ist implementiert und mit **16 automatisierten Rust-Tests** (`cargo test` + `cargo clippy -- -D warnings`) sowie einem 5-stufigen End-to-End-Durchstich (`aido-cli demo`) verifiziert:
  - `permission_gateway.rs`: Zugriffskontrolle (`ReadOnly`, `StandardAccess`, `FullAccess`), `PrivateOffline` Loopback-Schutz (`127.0.0.1`) und kryptografische Einmal-Freigabetokens (`ApprovalToken`).
  - `windows_bridge.rs`: Live-Hardware-Sensorik (`nvidia-smi` RTX 5090 VRAM, RAM, Top-Prozesse) und kontrolliertes *Fast Cleanup* nach Nutzerfreigabe.
  - `app_intelligence.rs`: **App Discovery & Capability Registry** mit SQLite-Persistenz (`app_registry`), Installer-Checkbox-Steuerung und Risiko-/Freigabeklassifizierung pro Fähigkeit (*Docker Desktop*, *Cursor / VS Code*, *LM Studio*, *Google Chrome*).
  - `project_intelligence.rs`: **Project Intelligence & Kanonischer Projektkatalog** mit Stack-Erkennung, Git-Branch/Status-Prüfung, FTS5-Volltextsuche (`project_catalog_fts`) und kontrolliertem Projekt-Start-Workflow (`LaunchProject`).
  - `memory.rs`: Lokales SQLite/FTS5-Langzeitgedächtnis und lückenloses `audit_log`.
  - `index.html` & `src/main.ts`: Animierter Desktop-Charakter mit Chat, Freigabe-Karten, App-Intelligence- & Projekt-Ansicht, Push-to-Talk, Screenshot-Vorschau und Privacy/Memory Center.

## Verifizierte Messungen am Referenz-PC (RTX 5090, 32607 MiB VRAM)

- **Hauptmodell + Vision + Speculative Decoding (`llama-server.exe` v0.9.0-dev Build 12176):**
  - `D:\AI_Models\llm\Qwen3.8-27B-UD-Q4_K_M.gguf` + `D:\AI_Models\llm\assets\mmproj-Qwen3.8-27B-BF16.gguf` + `D:\AI_Models\llm\assets\Qwen3.8-27B-DFlash2-Q4_K_M.gguf` laufen im lokalen Router-Modus auf `127.0.0.1:18434`.
  - **Schlafmodus & Autoload:** Im Ruhezustand belegt der Router `0 MiB` zusätzlichen VRAM (`2.307 MiB` System-VRAM). Bei der ersten Anfrage lädt er Modell, Projektor und Draft-Modell in `9,41 s` (`25.544 MiB` VRAM bei 32k Kontext) und gibt nach Ablauf von `--sleep-idle-seconds` automatisch **`22.714 MiB` VRAM** wieder frei (Rückgang auf `2.830 MiB`; Wiederaufwachen in `8,26 s`).
  - **Geschwindigkeit & Vision:** Warme Textgenerierung mit `draft-dflash` erreichte **`151,54 Token/s`**; ein künstliches zweifarbiges Testbild wurde mit **`102,70 Token/s`** korrekt als links rot und rechts blau erkannt (inklusive Gegenprobe ohne Bild).
- **Deutsche Spracherkennung (`whisper-server.exe` v1.9.5-unsloth.18):**
  - Mit den lokalen CUDA-13-Bibliotheken aus `C:\AI\llamacpp-cuda` im Prozess-`PATH` erkennt `whisper-server.exe` die RTX 5090 (`CUDA0`, `+1.150 MiB` VRAM) und transkribiert ein 6,9 Sekunden langes deutsches Test-WAV (`ggml-small.bin`) lokal auf `127.0.0.1:18080` in **`0,341 s`**.
- **Text-Embeddings:**
  - `text-embedding-nomic-embed-text-v1.5` (768 Dimensionen) lieferte im deutschen Ähnlichkeitstest `0,8036` für verwandte Projekttexte gegenüber `0,5959` für themenfremden Text.

## Aktueller Entwicklungsfokus (Phase 2)

Implementierung und Verifikation des ersten End-to-End-AIDO-Prototyps für das Szenario **„System-Gesundheitscheck & Fast Cleanup nach Nutzerfreigabe“**:
1. **Permission Gateway & Privacy Guard (`src-tauri`):** Strikte Trennung zwischen KI-Vorschlag und Systemausführung; Erzwingung von `PrivateOffline` (nur `127.0.0.1`) und expliziten Freigabe-Tokens (`ApprovalToken`).
2. **Windows Bridge (`src-tauri`):** Lesende Systemanalyse (VRAM, RAM, Prozesse, freigegebener AIDO-Temp-Cache) und kontrolliertes *Fast Cleanup* nach Nutzerbestätigung mit Vorher/Nachher-Prüfung.
3. **AIDO Memory (`src-tauri`):** SQLite-Datenbank mit FTS5-Volltextsuche und lückenlosem Audit-Log.
4. **Desktop-Charakter & Freigabe-UI (`src`):** Animierter Charakter mit Chat, Push-to-Talk, Screenshot-Kontext, Rechtsklick-Menü und klickbarer Freigabe-Karte.
