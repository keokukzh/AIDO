# AIDO Phase 3: Smart Installer, Hardware-Klassen-Erkennung & Transaktionale Einrichtung

**Stand:** 9. November 2026  
**Ziel:** Implementierung der intelligenten Installations- und Einrichtungsarchitektur (**AIDO — Agentic Intelligent Developing Operator**) gemäß [MASTERPLAN.md](../../MASTERPLAN.md) (Kapitel 4, 6.2 & 7):
1. **Smart Installer Engine (`src-tauri/src/smart_installer.rs`)**:
   - Automatische Hardware-Klassifizierung (`HighEndGpu`, `MidRangeGpu`, `CpuFocused`, `LowSpecCloudFallback`) und Modell-Empfehlung inkl. Erkennung bereits vorhandener lokaler Modelldateien (`Qwen3.8-27B-UD-Q4_K_M`, `mmproj-BF16`, `DFlash2`, `ggml-small.bin`), damit niemals doppelte Downloads stattfinden.
   - Berechnung der **drei ehrlichen Zeitprognosen (ETA)** aus Kapitel 6 des Masterplans:
     1. `ready_to_wake_seconds` (*„AIDO einsatzbereit in“*)
     2. `personal_setup_seconds` (*„Persönliche Einrichtung abgeschlossen in“*)
     3. `extended_integration_note` (*„Erweiterte Integration / Unbekannte Adapter“* — keine künstlich erfundenen Zeiten)
   - **Transaktionale, unterbrechbare und wiederaufnehmbare 9-Phasen-Installation** in SQLite (`installer_state` & `installer_steps`) mit Rollback-Protokollierung.
2. **Core, CLI & Desktop-UI Integration (`src-tauri/src/lib.rs`, `src-tauri/src/main.rs`, `index.html`, `src/main.ts`)**:
   - Einbindung von `SmartInstallerEngine` in `AidoCore`, neuer CLI-Befehl `aido-cli installer` sowie Schritt `[6/6]` im `demo`-Durchstich.
   - Interaktiver **Smart-Installer-Wizard** in der Desktop-Oberfläche (Datenschutzmodus, Standard/Full Access, Hardware- & Modell-Wiederverwendung, App-Checkboxen, 3-stufige ETA-Anzeige, Pause/Fortsetzen).

## Global Constraints

- **Offizielle Namensdefinition:** **AIDO = Agentic Intelligent Developing Operator**.
- **Sprache & Stil:** Alle nutzerseitigen Texte, Erklärungen und Fehlermeldungen sind auf **Deutsch** und ohne Fachjargon formuliert.
- **Permission Gateway & Wiederverwendung:** Vorhandene Modelle und Programme werden niemals überschrieben oder gelöscht. Die Hardware- und Modellprüfung ist strikt lesend (`ActionKind::InspectHardwareAndModels`).
- **Pristine Tests:** `cargo test` und `cargo clippy --all-targets -- -D warnings` müssen mit 0 Fehlern und 0 Warnungen durchlaufen; `npm run build` muss fehlerfrei bündeln.

---

## Task 1: Smart Installer Engine (`src-tauri/src/smart_installer.rs`)

**Ziel:** Implementiere `src-tauri/src/smart_installer.rs` und erweitere `ActionKind` in `src-tauri/src/permission_gateway.rs`.

### Anforderungen
1. Erweitere `ActionKind` in `src-tauri/src/permission_gateway.rs` um:
   - `InspectHardwareAndModels` (read-only: `is_mutating() == false`)
   - `RunInstallerStep` (mutierend: `is_mutating() == true`)
   - `RollbackInstaller` (mutierend: `is_mutating() == true`)
2. Erstelle `src-tauri/src/smart_installer.rs` und registriere `pub mod smart_installer;` in `src-tauri/src/lib.rs`.
3. Definiere folgende Datenstrukturen (`#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]`):
   - `HardwareTier`:
     - `HighEndGpu` (ab 24 GB VRAM, z. B. RTX 5090 — empfiehlt `Qwen3.8-27B-UD-Q4_K_M` + `mmproj-BF16` + `DFlash2`)
     - `MidRangeGpu` (8–23 GB VRAM — empfiehlt kompaktes 7B/8B Q4-Modell lokal)
     - `CpuFocused` (unter 8 GB VRAM, aber >= 16 GB RAM — empfiehlt 3B/4B CPU-Modell oder optionale Cloud)
     - `LowSpecCloudFallback` (empfiehlt leichtes Basismodell oder freiwillige Cloud-Anbindung)
   - `ModelReuseStatus`:
     - `role: String` (`"Hauptmodell"`, `"Vision-Projektor"`, `"Speculative-Draft"`, `"Spracherkennung (ASR)"`)
     - `model_name: String`
     - `expected_path: String`
     - `exists_locally: bool`
     - `size_bytes: u64`
   - `InstallerEtaForecast`:
     - `ready_to_wake_seconds: u64` (*AIDO einsatzbereit in*)
     - `personal_setup_seconds: u64` (*Persönliche Einrichtung abgeschlossen in*)
     - `remaining_download_bytes: u64`
     - `reused_models_count: usize`
     - `selected_apps_count: usize`
     - `extended_integration_note: String` (Hinweis auf gesonderte Adapter-Aufgaben ohne künstliche Zeitangabe)
     - `human_summary: String`
   - `InstallerStepStatus`: `Pending`, `Completed`, `Paused`, `Failed`, `RolledBack`
   - `InstallerStepRecord`:
     - `step_index: u8` (1 bis 9 gemäß Masterplan Kapitel 7: 1=Willkommen & Datenschutz, 2=Hardware erkennen, 3=Berechtigungsmodus, 4=KI einrichten, 5=Apps entdecken, 6=Apps integrieren, 7=Wissen aufbauen, 8=System validieren, 9=AIDO erwacht)
     - `step_key: String`
     - `title_de: String`
     - `status: InstallerStepStatus`
     - `rollback_action: String`
     - `details: String`
4. Implementiere `SmartInstallerEngine`:
   - `pub fn new(db_path: impl AsRef<Path>) -> Result<Self, InstallerError>`: Legt die SQLite-Tabellen `installer_steps` und `installer_profile` an und initialisiert die 9 Standardschritte, falls noch nicht vorhanden.
   - `pub fn classify_hardware(vram_total_mib: u64, ram_total_mb: u64) -> HardwareTier`
   - `pub fn inspect_existing_models(&self, gateway: &PermissionGateway, custom_paths: Option<&[(&str, &str, &Path, u64)]>) -> Result<Vec<ModelReuseStatus>, InstallerError>`:
     - Prüft `gateway.check_readonly_allowed(&ActionKind::InspectHardwareAndModels)?`.
     - Prüft standardmäßig die 4 Referenzpfade (`D:\AI_Models\llm\Qwen3.8-27B-UD-Q4_K_M.gguf`, `D:\AI_Models\llm\assets\mmproj-Qwen3.8-27B-BF16.gguf`, `D:\AI_Models\llm\assets\Qwen3.8-27B-DFlash2-Q4_K_M.gguf`, `D:\AI_Models\unsloth\whisper.cpp\models\ggml-small.bin`) oder `custom_paths` (für isolierte Unit-Tests) und erkennt, welche Dateien bereits lokal vorhanden sind (`exists_locally = true`).
   - `pub fn calculate_eta(tier: &HardwareTier, models: &[ModelReuseStatus], selected_apps_count: usize, unknown_adapters_count: usize, download_speed_mbps: u64) -> InstallerEtaForecast`:
     - Wenn Modelle bereits lokal vorhanden sind (`exists_locally == true`), entfallen deren Downloadbytes (`remaining_download_bytes` zählt nur fehlende Modelle).
     - Berechnet `ready_to_wake_seconds` (Basisinstallation + verbleibender Modell-Download + Validierung) und `personal_setup_seconds` (`ready_to_wake_seconds` + `selected_apps_count * 45`).
     - Setzt `extended_integration_note` auf Deutsch (bei `unknown_adapters_count > 0` expliziter Hinweis, dass unbekannte Adapter isoliert geprüft werden und keine erfundene Restzeit erhalten).
   - `pub fn list_steps(&self) -> Result<Vec<InstallerStepRecord>, InstallerError>`
   - `pub fn complete_step(&self, step_index: u8, details: &str) -> Result<InstallerStepRecord, InstallerError>`
   - `pub fn pause_at_step(&self, step_index: u8, reason: &str) -> Result<InstallerStepRecord, InstallerError>`
   - `pub fn next_resumable_step(&self) -> Result<Option<InstallerStepRecord>, InstallerError>`: Findet den ersten Schritt mit `Paused` oder `Pending` (Unterbrechungs- & Wiederaufnahmefähigkeit).
   - `pub fn rollback_completed_steps(&self) -> Result<usize, InstallerError>`: Setzt alle `Completed`-Schritte in umgekehrter Reihenfolge (9..=1) auf `RolledBack` und gibt die Anzahl zurückgeführter Schritte zurück.
5. Schreibe mindestens **4 Unit-Tests** in `smart_installer.rs`:
   - `test_hardware_tier_classification` (prüft RTX 5090 32 GB -> `HighEndGpu`, 12 GB -> `MidRangeGpu`, 0 VRAM / 32 GB RAM -> `CpuFocused`, 0 VRAM / 8 GB RAM -> `LowSpecCloudFallback`).
   - `test_model_reuse_and_eta_calculation` (prüft, dass vorhandene lokale Modelle 0 Download-Bytes verursachen und unbekannte Adapter ehrlich ohne künstliche Zeit ausgewiesen werden).
   - `test_transactional_installer_pause_and_resume` (prüft Fortschritt über Schritte 1..4, Pause bei Schritt 5, Wiederaufnahme genau bei Schritt 5 über SQLite).
   - `test_installer_rollback_preserves_existing_models` (prüft `rollback_completed_steps`).

---

## Task 2: Integration in `AidoCore`, CLI (`aido-cli`) & Interaktiver Installer-Wizard in der Desktop-UI (`index.html`, `src/main.ts`)

**Ziel:** Verbinde `SmartInstallerEngine` mit `AidoCore` (`src-tauri/src/lib.rs`), ergänze den CLI-Befehl `aido-cli installer` sowie Schritt `[6/6]` in `aido-cli demo` (`src-tauri/src/main.rs`) und baue den interaktiven Smart-Installer-Wizard in die Desktop-UI (`index.html`, `src/main.ts`) ein.

### Anforderungen
1. **`src-tauri/src/lib.rs` (`AidoCore`)**:
   - Füge `pub installer: smart_installer::SmartInstallerEngine` zu `AidoCore` hinzu und initialisiere es in `AidoCore::new` mit `db_path`.
   - Implementiere `pub fn evaluate_smart_installer_plan(&self, selected_apps_count: usize, unknown_adapters_count: usize) -> Result<(smart_installer::HardwareTier, Vec<smart_installer::ModelReuseStatus>, smart_installer::InstallerEtaForecast), Box<dyn std::error::Error>>`:
     - Liest GPU/RAM über `self.bridge.run_system_health_check()?`, bestimmt `HardwareTier`, prüft `self.installer.inspect_existing_models(&self.gateway, None)?` und berechnet `InstallerEtaForecast`.
     - Schreibt einen Audit-Log-Eintrag (`SmartInstallerPlan`) und speichert das Ergebnis in `SystemMemory`.
   - Ergänze einen Integrationstest `aido_core_smart_installer_plan_reuses_models_and_supports_resume` in `src-tauri/src/lib.rs`.
2. **`src-tauri/src/main.rs` (`aido-cli`)**:
   - Neuer Unterbefehl `installer`: Führt `core.evaluate_smart_installer_plan(4, 1)?` aus und gibt Hardware-Tier, Modell-Wiederverwendungsstatus, ETA-Prognose und die 9 Installationsschritte als formatiertes JSON aus.
   - Erweitere `demo` von 5 auf **6 Schritte** (`[1/6]` bis `[6/6]`):
     - Schritt `[6/6] Smart Installer & Modell-Wiederverwendung`: Zeigt erkannte Hardware-Klasse (`HighEndGpu`), Anzahl wiederverwendeter lokaler Modelle (z. B. 4/4 vorhanden -> `0 MB` erneuter Download!), die 3-stufige ETA-Prognose (*AIDO einsatzbereit in*, *Persönliche Einrichtung abgeschlossen in*, *Erweiterte Integration*) und demonstriert das Abschließen der ersten Installationsphasen samt Wiederaufnahme-Prüfung.
3. **Desktop-UI (`index.html` & `src/main.ts`)**:
   - Füge in der Aktionsleiste und im Rechtsklick-Kontextmenü den Punkt `🚀 Smart Installer (9-Phasen-Setup)` (`data-action="installer"`) hinzu.
   - Rendere bei Klick auf `🚀 Smart Installer (9-Phasen-Setup)` die interaktive **Smart-Installer-Ansicht**:
     - Hardware-Erkennung & Modell-Wiederverwendung (zeigt grün an: `Qwen3.8-27B-UD-Q4_K_M`, `mmproj-BF16`, `DFlash2`, `ggml-small.bin` bereits lokal vorhanden — `0 GB Download nötig`).
     - Auswahl von Datenschutzmodus (`Private Offline`, `Local Connected`, `Hybrid AI`) und Zugriffsprofil (`Standard Access`, `Full Access`).
     - Drei getrennte ETA-Anzeigen (*AIDO einsatzbereit in*, *Persönliche Einrichtung abgeschlossen in*, *Erweiterte Integration*).
     - Interaktive Steuerung der 9 Einrichtungsphasen mit Buttons für **„Nächste Phase ausführen“**, **„Installation pausieren / später fortsetzen“** und **„Transaktionales Rollback testen“**.
