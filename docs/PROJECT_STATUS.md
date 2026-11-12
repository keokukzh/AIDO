# Projektstatus — AIDO (Agentic Intelligent Developing Operator)

**Stand: 12. November 2026.** [Entwicklungsroadmap](ROADMAP.md)

## Bestätigte Abnahme: Meilenstein 4 (App-Connectors, Workflows & Whisper-Fundament) (12. November 2026)

- **Blender RPC / Scripting Adapter (`blender_connector.rs`):**
  - Erkennt installierte Blender-Versionen (`C:\Program Files\Blender Foundation\Blender 5.1\blender.exe`).
  - Führt isolierte Szenenanalysen in Headless-Hintergrund-Instanzen (`--background --python-expr`) in unter 1,8 Sekunden aus.
  - Extrahiert Engine (`BLENDER_EEVEE_NEXT` / `CYCLES`), aktive Kamera, Objektanzahl, Mesh-/Licht-/Kamera-Verteilung und Polycount.
  - Blockiert böswillige Systembefehle (`os.system`, `shutil.rmtree`) im Python-Skript und bindet alle mutierenden Render- und Skriptaktionen an Einmal-Tokens (`ActionKind::ExecuteAppAction`).
  - Neue IPC-Commands: `blender_inspect` im Desktop-Chat integriert (triggert auf «analysiere blender», «blender szene» oder `/blender`).
- **Docker Desktop & WSL2 Container Controller (`docker_controller.rs`):**
  - Automatische Erkennung der Docker CLI (`docker.exe` Version 29.9.2) und des Hintergrunddienst-Status.
  - Saubere deutsche Diagnose bei gestopptem Daemon ohne Fehlermeldungs-Absturz.
  - Bereitstellung der 3 vordefinierten Masterplan-Sandboxes (`01-fullstack-sandbox`, `02-ai-gpu-sandbox` mit RTX 5090 Passthrough, `03-automation-stack`).
  - Kontrollierte Aktionen (`Start`, `Stop`, `Restart`, `PruneUnused`) mit Input-Sanitization gegen Shell-Injections und Einmal-Token-Zwang (`ActionKind::ManageDockerSandbox`). Replay-Versuche verbrauchter Tokens werden strikt abgewiesen.
  - Im Chat integriert (triggert auf «welche container laufen», «docker status» oder `/docker`).
- **Windows Security, DPAPI Vault & System-Monitor (`windows_security.rs`):**
  - Echter hardware- und benutzergebundener Windows-DPAPI-Tresor (`WindowsDpapiVault`) über direkte Win32-FFI-Bindung an `CryptProtectData` und `CryptUnprotectData` in `crypt32.dll`. Secrets liegen niemals im Klartext auf der Festplatte.
  - `ProcessSecurityContext` prüft Windows-Integritätslevel (`Medium` Standardbenutzer vs. `High` Administrator) und stellt sicher, dass AIDO keine unprivilegierte Angriffsfläche exponiert.
  - `SystemSecurityGuard` überwacht Ports (Hermes 18434, Whisper 18080, Ollama 11434, LM Studio 1234, Web-Vorschau 8080). Hermes auf Port 18434 wird aktiv als geschützter Dienst markiert und bleibt unangetastet.
- **Whisper ASR Sprachfundament & Test-Konsolidierung (106/106 Tests grün):**
  - Die offizielle Whisper-Modelldatei `ggml-small.bin` (487.601.967 Bytes) wurde am erwarteten Pfad `D:\AI_Models\unsloth\whisper.cpp\models\ggml-small.bin` bereitgestellt.
  - `scenario_8_smart_installer_model_reuse_and_clean_uninstaller_safety` in `multi_scenario_test.rs` besteht nun fehlerfrei (`0 GB Download guarantee` & `4/4 Modelle wiederverwendet`).
  - **106 von 106 Tests** über alle Testsuiten (`lib.rs`, `desktop_actions`, `desktop_setup`, `desktop_connectors_and_workflows_test`, `desktop_memory_proactive_test`, `multi_scenario_test`, `ui_automation_test`) bestehen fehlerfrei.

## Bestätigte Abnahme: Installer 0.1.2, Meilenstein 3 (Gedächtnis & Proaktivität) und Native UI Automation (12. November 2026)

- **Meilenstein 3: 7-Stufen-Gedächtnis & Proaktive MONO-Engine:**
  - **Privacy & Memory Center:** Neues Fenster `memory.html` (`src/memory_ui.ts` & `src/memory.css`), erreichbar über das Rechtsklick-Menü von MONO (`🧠 Gedächtnis & Wissen`), das Tray-Menü im Infobereich und einen Button in der Chat-Kopfzeile. Bietet Quotenübersicht (SSD/SQLite), Anzeige aller aktiven Einträge über alle 7 Ebenen (`SoulMemory`, `UserMemory`, `SystemMemory`, `ProjectMemory`, `EpisodicMemory`, `SemanticMemory`, `ProceduralMemory`) sowie sofortige Löschung (`🗑️ Vergessen`).
  - **Chat-Gedächtnisanreicherung & Auto-Learning:** Vor jeder Chat-Inferenz werden relevante Fakten via FTS5 aus SQLite geholt und dem Modell übergeben. Chatbefehle wie `Vergiss X` markieren Einträge direkt als vergessen (`validity = 'forgotten_by_user'`). Nutzerpräferenzen ("Mein Lieblings...", "Ich bevorzuge...", "Merke dir...") werden nach der Antwort automatisch im `UserMemory` abgelegt.
  - **Proaktive Sprechblase über MONO:** Der Desktop-Companion prüft periodisch im Hintergrund Git-Zustände registrierter Projekte und System-Caches via `ProactiveEngine`. Wenn ungesicherte Änderungen vorliegen, erscheint eine sanfte Sprechblase mit den Optionen *Ansehen*, *Später* und *Ausblenden* – mit striktem Fokus-Schutz bei Vollbild oder Gaming.
  - **Verifikation:** Neuer Integrationstest `tests/desktop_memory_proactive_test.rs` besteht fehlerfrei (7 Ebenen, FTS5-Filterung aktiver vs. vergessener Einträge, Fokus-Guard-Unterdrückung bei Gaming, Kategorien-Stummschaltung).
- **Native Windows UI Automation (`agent-desktop` Integration):**
  - Vercel Labs `agent-desktop` (v1.2.0, nativer Rust-Treiber für Windows UIA) wurde in `tooling/agent-desktop/agent-desktop.exe` eingebunden und als App-Ressource in den NSIS-Installer integriert.
  - Modul [ui_automation.rs](file:///C:/Users/keoku/Desktop/AIDO/src-tauri/src/ui_automation.rs) implementiert die semantische Desktop-Bedienung: `list_windows`, `take_snapshot` (Accessibility Tree mit `@ref`-IDs), `Click`, `TypeText`, `SetValue`, `PressKey` und `FocusWindow`.
  - **Sicherheits-Enforcement:** `ReadOnly` erlaubt nur UI-Inspektion (`ActionKind::UiObserve`). Mutierende Aktionen (`ActionKind::UiAct`) erfordern in `StandardAccess` ein kryptografisches Einmal-Token (`ApprovalToken`), während `FullAccess` freigegebene Routine-Aktionen erlaubt. Jede Interaktion wird in SQLite `audit_log` dokumentiert.
  - **Verifikation:** Neuer Integrationstest `tests/ui_automation_test.rs` besteht fehlerfrei (Snapshot & Fokus-Steuerung auf echtem Notepad-Fenster).
- **Installer-Upgrade und Setup-Erhalt:** Der NSIS-Installer `src-tauri\target\release\bundle\nsis\AIDO_0.1.2_x64-setup.exe` (72,42 MiB) wurde gebaut und auf dem Referenz-PC installiert. `memory.html` und `agent-desktop.exe` werden mitgeliefert. AIDO startet eigenständig ohne Vite auf dem Windows-Desktop, MONO erwacht animiert und die bestehende SQLite-Einrichtung (`%LOCALAPPDATA%\AIDO\data\aido_memory.db`) blieb zu 100 % erhalten.
- **Desktop-Handle-Robustheit (Win32):** In Headless-/Agent-Subprozess-Kontexten verhinderte ein abweichender Desktop (`exebox-...`) das Binden von WebView2 (`0x80070578 Ungültiges Fensterhandle`). Dies wurde in `src-tauri/src/desktop/main.rs` durch automatische Thread-Desktop-Umschaltung auf `WinSta0\Default` via `OpenDesktopW` und `SetThreadDesktop` behoben (`Cargo.toml` um `Win32_System_StationsAndDesktops` ergänzt).
- **WSL2-Start («Loading model») behoben:** Die Ursache für das Blockieren von `llama-server` in WSL2 bei «Loading model» war das Kernel-`mmap` bei der 16,46 GB grossen Modelldatei über das Windows-Laufwerk `/mnt/d/` (Plan-9-/drvfs-Treiber) bei 11,9 GB WSL2-RAM-Limit (Disk Sleep `p9_client_rpc`). Behoben durch `--load-mode none` in `src-tauri/src/companion_runtime.rs`. Der VmRSS-Speicherbedarf sank von 11,2 GB auf 1,08 GB (8,8 % RAM), alle Layer laden direkt in die RTX 5090 (17.053 MiB VRAM) und der Multi-Turn-Chattest mit «Was ist 17 mal 23?» und «Addiere 1...» antwortete in 45,12 s mit `391` und `392`. Nach dem Shutdown fällt der VRAM sofort wieder auf 1.867 MiB ab.
- **Hermes-Koexistenz:** Während des gesamten WSL2- und Desktop-Tests lief der Hermes-Agent auf PID 22532 (Port 18434) ununterbrochen und ungestört weiter. AIDO wählt bei belegten Ports dynamisch freie Loopback-Ports.
- **REA 6.7.0 & Notepad-PE-Analyse:** Das korrigierte Windows-Pfadformat beim Aufruf der gebündelten REA-Laufzeit (`tooling/rea/rea.js`) wurde erfolgreich verifiziert. Die lesende PE-Ressourcenanalyse von `C:\Windows\System32\notepad.exe` extrahiert Architektur, Header und Importe ohne Seiteneffekte.
- **Antigravity-Projektstart & Blender-Regression:**
  - Der Projektpfad «Öffne mein Projekt AIDO» normalisiert kanonisch exakt auf `C:\Users\keoku\Desktop\AIDO`.
  - **Ablehnen:** Lässt das Projekt geschlossen und verwirft das Token.
  - **Ändern:** Invalidiert das bisherige Freigabe-Token und erzwingt eine Neuanforderung.
  - **Öffnen:** Startet Antigravity gezielt mit `--new-window <pfad>`.
  - Einmal-Token-Sicherheit, Replay-Schutz und lückenlose Protokollierung im SQLite-`audit_log` wurden automatisiert (`tests/desktop_actions.rs`) und manuell bestätigt. «Starte Blender» wurde als Regressionstest erfolgreich abgenommen.
- **Build- & Regressionstests:**
  - Frontend-Build (`npm run build`): Erfolgreich in 175 ms.
  - Browser-Vorschau: Läuft separat via `vite preview` (Status 200).
  - CLI: `aido-cli health` meldet den System- und VRAM-Zustand (1.804 MiB belegt von 32.607 MiB, 30.384 MiB frei).
  - Rust-Tests: 79/79 Unit-Tests in `src/lib.rs`, `desktop_actions`, `desktop_setup` und `companion_chat` bestanden. `scenario_1` bestätigt volle VRAM-Reserve (>30 GB frei). `scenario_8` scheitert planmässig, da `D:\AI_Models\unsloth\whisper.cpp\models\ggml-small.bin` am Referenzpfad noch nicht abgelegt ist.

## Nativer Desktop-Companion und lokaler Textchat vom 11. November 2026

- **Build und Start:** `npm run tauri -- build --no-bundle -- --locked` baut `src-tauri\target\release\AIDO.exe`. Die EXE startete auf dem Windows-11-Referenz-PC ohne Vite. MONO renderte animiert, öffnete per Klick den separaten Chat und blieb sichtbar, nachdem das Chatfenster geschlossen wurde.
- **Qwen-Chat:** Ein absichtlich fehlender GGUF-Pfad führte zu einer Fehlermeldung mit «Erneut versuchen». Nachdem der Modellpfad wieder erreichbar war, startete AIDO den lokalen Server auf `127.0.0.1:18434` und zeigte eine echte Qwen-Antwort. Dieser Durchlauf lief auf der CPU. Der Desktop-Start übernimmt jetzt den vorhandenen CUDA-Bibliothekspfad. Die Runtime erkannte mit diesem Pfad die RTX 5090 bei der separaten Geräteprüfung.
- **Paralleler Betrieb mit Hermes:** Mit aktivem, authentifiziertem Hermes-Dienst auf Port 18434 startete AIDO einen eigenen Qwen-Dienst auf einem freien Loopback-Port. AIDO verwendete die RTX 5090 und beantwortete die Rechenfrage und die anschliessende Rückfrage mit `391` und `392`. Hermes lief auf seinem bisherigen Port weiter.
- **Regression:** Der Frontend-Build, die Browser-Aufwachanimation und der separate CLI-Start wurden geprüft. 87 von 88 Rust-Tests bestanden. `scenario_8_smart_installer_model_reuse_and_clean_uninstaller_safety` scheitert weiterhin, weil `D:\AI_Models\unsloth\whisper.cpp\models\ggml-small.bin` am Referenzpfad fehlt.
- **Offene Abnahme:** Die Bedienung im Windows-Infobereich und die Prozessbereinigung beim Beenden bleiben offen. Mikrofon, Screenshot-Kontext, Android-Steuerung, Systemaktionen und Windows-Autostart gehören zu späteren Desktop-Integrationen.

## Historischer Stand vom 9. November 2026

### Repository und Meilensteine der Phasen 0–8

- `keokukzh/AIDO` (**Agentic Intelligent Developing Operator**) ist ein privates GitHub-Repository und lokal in `C:\Users\keoku\Desktop\AIDO` eingerichtet.
- Der vollständige Produkt-Masterplan v1.0 ist unter [MASTERPLAN.md](MASTERPLAN.md) hinterlegt.
- Die lokale Modell- und Runtime-Validierung (Text, Vision, Speculative Decoding, Schlafmodus/Idle-Unload, Embeddings und deutsche Spracherkennung) ist vollständig abgeschlossen und mit SHA-256-Prüfsummen im [Modellinventar](MODEL_RUNTIME_INVENTORY.md) dokumentiert.
- **Alle 8 Entwicklungsphasen (Phasen 0 bis 8)** sind implementiert und mit **40 automatisierten Rust-Tests** (`cargo test` + `cargo clippy -- -D warnings`, 0 Warnungen) sowie einem 9-stufigen End-to-End-Durchstich (`aido-cli demo`) verifiziert:
  - `permission_gateway.rs`: Zugriffskontrolle (`ReadOnly`, `StandardAccess`, `FullAccess`), `PrivateOffline` Loopback-Schutz (`127.0.0.1`) und kryptografische Einmal-Freigabetokens (`ApprovalToken`).
  - `windows_bridge.rs`: Live-Hardware-Sensorik (`nvidia-smi` RTX 5090 VRAM, RAM, Top-Prozesse) und zweistufiges System-Cleanup (**Fast Cleanup** für Sitzungs-Caches & **Deep Cleanup** für verwaiste Build-/Shader-/Crash-Dump-Artefakte) nach Nutzerfreigabe.
  - `app_intelligence.rs`: **App Discovery, Capability Registry & Isolierte Adapter-Prüfbank (Phase 7)** mit SQLite-Persistenz (`app_registry`), Installer-Checkbox-Steuerung, Risiko-/Freigabeklassifizierung pro Fähigkeit (*Docker Desktop*, *Cursor / VS Code*, *LM Studio*, *Google Chrome*) sowie isolierter Sandbox-Synthese (`synthesize_and_verify_unknown_app_adapter`) für unbekannte Programme (*OBS Studio*, *Blender 3D*).
  - `project_intelligence.rs`: **Project Intelligence & Kanonischer Projektkatalog** mit Stack-Erkennung, Git-Branch/Status-Prüfung, FTS5-Volltextsuche (`project_catalog_fts`) und kontrolliertem Projekt-Start-Workflow (`LaunchProject`).
  - `smart_installer.rs`: **Smart Installer, ETA-Engine, Zero-Destruction-Deinstallation & Release-Readiness (Phasen 3 & 8)** mit 4 Hardware-Tiers (`HighEndGpu`, `MidRangeGpu`, `CpuFocused`, `LowSpecCloudFallback`), automatischer Wiederverwendung bereits vorhandener lokaler Modelle (`0 GB` Download auf dem Referenz-PC), 3-stufiger ehrlicher ETA-Prognose, 9 transaktionalen SQLite-Installationsphasen (`installer_steps`), **Sauberer Deinstallation (`prepare_clean_uninstall` / `execute_clean_uninstall` — 100 % Schutz vorhandener KI-Modelle & Nutzerprojekte)** und 8-Phasen-Release-Prüfung (`evaluate_release_readiness`).
  - `runtime_supervisor.rs` & `ai_client.rs`: **Live-Runtime-Supervisor, Vision & ASR-Orchestrierung (Phase 4)** für `llama-server.exe` (`127.0.0.1:18434`, `--sleep-idle-seconds 300` Zero-VRAM-Standby, `mmproj` Vision, `DFlash2` Speculative Decoding) und `whisper-server.exe` (`127.0.0.1:18080`, `ggml-small.bin`, CUDA 13.4 DLL-Auflösung via `C:\AI\llamacpp-cuda`).
  - `memory.rs`: **4-Stufen-Gedächtnis & Kompaktierungs-Engine (Phase 4)** (`WorkingMemory`, `EpisodicMemory`, `SemanticMemory`, `SystemMemory`) mit SQLite/FTS5-Volltextsuche, Hintergrund-Verdichtung (`compact_episodic_memories`), Nutzer-Kontrolle zum Vergessen einzelner Einträge (`forget_memory_entry`) und lückenlosem `audit_log`.
  - `proactive_engine.rs`: **Proaktive Vordenker-Engine (*„AIDO denkt für dich“*, Phasen 5 & 6)** mit striktem Fokus-Schutz (`FullscreenOrGaming`, `VoiceRecordingOrCall`, `QuietHoursActive` -> 0 Unterbrechungen), 2x-Ablehnungs-Backoff, Themen-Stummschaltung und priorisierten Vorschlägen (`DirtyGitWorkspace`, `SystemMaintenance`, `MemoryConsolidation`, `ProjectContinuation`).
  - `index.html` & `src/main.ts`: Animierter Desktop-Charakter mit Chat, Freigabe-Karten, App-Intelligence-, Projekt-, Smart-Installer-, Runtime/Memory-Kompaktierungs-, Proaktive-Vordenker-, Deep-Cleanup- und Adapter-Sandbox/Release-Check-Ansicht, Push-to-Talk, Screenshot-Vorschau und interaktivem Privacy/Memory Center.

## Verifizierte Messungen am Referenz-PC (RTX 5090, 32607 MiB VRAM)

- **Hauptmodell + Vision + Speculative Decoding (`llama-server.exe` v0.9.0-dev Build 12176):**
  - `D:\AI_Models\llm\Qwen3.8-27B-UD-Q4_K_M.gguf` + `D:\AI_Models\llm\assets\mmproj-Qwen3.8-27B-BF16.gguf` + `D:\AI_Models\llm\assets\Qwen3.8-27B-DFlash2-Q4_K_M.gguf` laufen im lokalen Router-Modus auf `127.0.0.1:18434`.
  - **Schlafmodus & Autoload:** Im Ruhezustand belegt der Router `0 MiB` zusätzlichen VRAM (`2.307 MiB` System-VRAM). Bei der ersten Anfrage lädt er Modell, Projektor und Draft-Modell in `9,41 s` (`25.544 MiB` VRAM bei 32k Kontext) und gibt nach Ablauf von `--sleep-idle-seconds` automatisch **`22.714 MiB` VRAM** wieder frei (Rückgang auf `2.830 MiB`; Wiederaufwachen in `8,26 s`).
  - **Geschwindigkeit & Vision:** Warme Textgenerierung mit `draft-dflash` erreichte **`151,54 Token/s`**; ein künstliches zweifarbiges Testbild wurde mit **`102,70 Token/s`** korrekt als links rot und rechts blau erkannt (inklusive Gegenprobe ohne Bild).
- **Deutsche Spracherkennung (`whisper-server.exe` v1.9.5-unsloth.18):**
  - Mit den lokalen CUDA-13-Bibliotheken aus `C:\AI\llamacpp-cuda` im Prozess-`PATH` erkennt `whisper-server.exe` die RTX 5090 (`CUDA0`, `+1.150 MiB` VRAM) und transkribiert ein 6,9 Sekunden langes deutsches Test-WAV (`ggml-small.bin`) lokal auf `127.0.0.1:18080` in **`0,341 s`**.
- **Text-Embeddings:**
  - `text-embedding-nomic-embed-text-v1.5` (768 Dimensionen) lieferte im deutschen Ähnlichkeitstest `0,8036` für verwandte Projekttexte gegenüber `0,5959` für themenfremden Text.
