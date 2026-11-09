interface ApprovalProposal {
  requestId: string;
  title: string;
  explanation: string;
  files: string[];
  bytes: number;
  actionKind?: string;
}

interface AppCapability {
  capabilityId: string;
  titleDe: string;
  risk: "Low" | "Medium" | "High";
  requiresApproval: boolean;
}

interface AppItem {
  appId: string;
  displayName: string;
  publisher: string;
  installPath: string;
  preferredControl: string;
  userEnabled: boolean;
  capabilities: AppCapability[];
}

interface ProjectItem {
  name: string;
  canonicalPath: string;
  stack: string[];
  gitBranch: string;
  gitStatus: string;
  recommendedAction: string;
  humanSummary: string;
}

interface InstallerStep {
  stepIndex: number;
  stepKey: string;
  titleDe: string;
  status: "Pending" | "Completed" | "Paused" | "Failed" | "RolledBack";
  rollbackAction: string;
  details: string;
}

interface ModelReuseInfo {
  role: string;
  modelName: string;
  expectedPath: string;
  existsLocally: boolean;
  sizeBytes: number;
}

const installerStepsData: InstallerStep[] = [
  {
    stepIndex: 1,
    stepKey: "welcome_privacy",
    titleDe: "Willkommen & Datenschutz",
    status: "Pending",
    rollbackAction: "Datenschutzbestätigung zurücksetzen",
    details: ""
  },
  {
    stepIndex: 2,
    stepKey: "hardware_check",
    titleDe: "Hardware automatisch erkennen",
    status: "Pending",
    rollbackAction: "Hardware-Erkennungsprofil verwerfen",
    details: ""
  },
  {
    stepIndex: 3,
    stepKey: "access_profile",
    titleDe: "Berechtigungsmodus wählen",
    status: "Pending",
    rollbackAction: "Gewähltes Zugriffsprofil auf Standard zurücksetzen",
    details: ""
  },
  {
    stepIndex: 4,
    stepKey: "ai_setup",
    titleDe: "KI automatisch einrichten",
    status: "Pending",
    rollbackAction: "KI-Modell-Pfade und LLM-Slot-Registrierungen deregistrieren (Dateien bleiben auf Festplatte)",
    details: ""
  },
  {
    stepIndex: 5,
    stepKey: "app_discovery",
    titleDe: "Apps entdecken",
    status: "Pending",
    rollbackAction: "Erkannte App-Liste zurücksetzen",
    details: ""
  },
  {
    stepIndex: 6,
    stepKey: "app_integration",
    titleDe: "Apps intelligent integrieren",
    status: "Pending",
    rollbackAction: "Generierte Adapter und Tool-Registrierungen deregistrieren",
    details: ""
  },
  {
    stepIndex: 7,
    stepKey: "knowledge_index",
    titleDe: "Persönliches Wissen aufbauen",
    status: "Pending",
    rollbackAction: "Temporäre Index-Caches leeren",
    details: ""
  },
  {
    stepIndex: 8,
    stepKey: "system_validation",
    titleDe: "System validieren",
    status: "Pending",
    rollbackAction: "Validierungsergebnisse und Prüfberichte verwerfen",
    details: ""
  },
  {
    stepIndex: 9,
    stepKey: "aido_awakens",
    titleDe: "AIDO erwacht",
    status: "Pending",
    rollbackAction: "Autostart-Dienst stoppen und Bereitschaftsstatus zurücknehmen",
    details: ""
  }
];

const modelsReuseData: ModelReuseInfo[] = [
  {
    role: "Hauptmodell",
    modelName: "Qwen3.8-27B-UD-Q4_K_M.gguf",
    expectedPath: "D:\\AI_Models\\llm\\Qwen3.8-27B-UD-Q4_K_M.gguf",
    existsLocally: true,
    sizeBytes: 16464440224
  },
  {
    role: "Vision-Projektor",
    modelName: "mmproj-Qwen3.8-27B-BF16.gguf",
    expectedPath: "D:\\AI_Models\\llm\\assets\\mmproj-Qwen3.8-27B-BF16.gguf",
    existsLocally: true,
    sizeBytes: 931146432
  },
  {
    role: "Speculative-Draft",
    modelName: "Qwen3.8-27B-DFlash2-Q4_K_M.gguf",
    expectedPath: "D:\\AI_Models\\llm\\assets\\Qwen3.8-27B-DFlash2-Q4_K_M.gguf",
    existsLocally: true,
    sizeBytes: 1143006816
  },
  {
    role: "Spracherkennung (ASR)",
    modelName: "ggml-small.bin",
    expectedPath: "D:\\AI_Models\\unsloth\\whisper.cpp\\models\\ggml-small.bin",
    existsLocally: true,
    sizeBytes: 487601967
  }
];

const appsData: AppItem[] = [
  {
    appId: "docker-desktop",
    displayName: "Docker Desktop",
    publisher: "Docker Inc.",
    installPath: "C:\\Program Files\\Docker\\Docker\\Docker Desktop.exe",
    preferredControl: "Cli",
    userEnabled: true,
    capabilities: [
      { capabilityId: "open_app", titleDe: "Docker Desktop öffnen", risk: "Low", requiresApproval: false },
      { capabilityId: "list_containers", titleDe: "Laufende Container auflisten", risk: "Low", requiresApproval: false },
      { capabilityId: "start_stop_container", titleDe: "Container starten / stoppen", risk: "Medium", requiresApproval: true },
      { capabilityId: "prune_volumes", titleDe: "Ungenutzte Volumes bereinigen", risk: "High", requiresApproval: true }
    ]
  },
  {
    appId: "cursor-ide",
    displayName: "Cursor / VS Code",
    publisher: "Anysphere / Microsoft",
    installPath: "C:\\Users\\keoku\\AppData\\Local\\Programs\\cursor\\Cursor.exe",
    preferredControl: "Cli",
    userEnabled: true,
    capabilities: [
      { capabilityId: "open_app", titleDe: "Editor öffnen", risk: "Low", requiresApproval: false },
      { capabilityId: "open_project", titleDe: "Projektordner öffnen", risk: "Low", requiresApproval: true }
    ]
  },
  {
    appId: "lm-studio",
    displayName: "LM Studio",
    publisher: "Element Labs, Inc.",
    installPath: "C:\\Program Files\\LM Studio\\LM Studio.exe",
    preferredControl: "NativeApi",
    userEnabled: true,
    capabilities: [
      { capabilityId: "query_models", titleDe: "Verfügbare Modelle abfragen", risk: "Low", requiresApproval: false },
      { capabilityId: "load_unload_model", titleDe: "Modell laden / entladen", risk: "Medium", requiresApproval: true }
    ]
  },
  {
    appId: "google-chrome",
    displayName: "Google Chrome",
    publisher: "Google LLC",
    installPath: "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
    preferredControl: "UiAutomation",
    userEnabled: true,
    capabilities: [
      { capabilityId: "open_url", titleDe: "URL im Browser öffnen", risk: "Low", requiresApproval: true }
    ]
  }
];

const projectsData: ProjectItem[] = [
  {
    name: "AIDO",
    canonicalPath: "C:\\Users\\keoku\\Desktop\\AIDO",
    stack: ["Node.js/TypeScript", "Rust"],
    gitBranch: "main",
    gitStatus: "dirty",
    recommendedAction: "npm run dev",
    humanSummary: "Projekt 'AIDO' (Stack: Node.js/TypeScript, Rust) auf Branch 'main' [dirty]"
  }
];

const avatarOrb = document.getElementById("avatar-orb") as HTMLDivElement;
const charState = document.getElementById("char-state") as HTMLSpanElement;
const aidoMessage = document.getElementById("aido-message") as HTMLDivElement;
const actionButtons = document.getElementById("action-buttons") as HTMLDivElement;
const auditList = document.getElementById("audit-list") as HTMLUListElement;
const contextMenu = document.getElementById("context-menu") as HTMLDivElement;

let currentProposal: ApprovalProposal | null = null;
let pressTimer: number | null = null;

function appendAudit(entry: string): void {
  const li = document.createElement("li");
  const now = new Date().toLocaleTimeString("de-DE");
  li.textContent = `[${now}] ${entry}`;
  auditList.prepend(li);
}

function setCharacterState(stateLabel: string, working = false): void {
  charState.textContent = `Zustand: ${stateLabel}`;
  if (working) {
    avatarOrb.classList.add("working");
  } else {
    avatarOrb.classList.remove("working");
  }
}

function renderDefaultButtons(): void {
  actionButtons.innerHTML = "";
  const btnCheck = document.createElement("button");
  btnCheck.textContent = "🔍 Systemcheck & Fast Cleanup prüfen";
  btnCheck.onclick = () => startSystemHealthAndCleanupPreview();

  const btnApps = document.createElement("button");
  btnApps.textContent = "🧩 App Intelligence & Projekte";
  btnApps.onclick = () => showAppIntelligenceAndProjects();

  const btnInstaller = document.createElement("button");
  btnInstaller.textContent = "🚀 Smart Installer (9-Phasen-Setup)";
  btnInstaller.onclick = () => showSmartInstallerWizard();

  const btnPrivacy = document.createElement("button");
  btnPrivacy.textContent = "🔒 Privacy Center anzeigen";
  btnPrivacy.onclick = () => showPrivacyCenter();

  const btnMemory = document.createElement("button");
  btnMemory.textContent = "🧠 Memory & Audit-Log";
  btnMemory.onclick = () => showMemorySummary();

  actionButtons.append(btnCheck, btnApps, btnInstaller, btnPrivacy, btnMemory);
}

function startSystemHealthAndCleanupPreview(): void {
  setCharacterState("Analysiere System (Nur lesend)...", true);
  currentProposal = {
    requestId: crypto.randomUUID(),
    title: "AIDO Fast Cleanup (Temporäre Sitzungs-Caches bereinigen)",
    explanation:
      "Ich habe deinen PC lesend geprüft: Deine NVIDIA GeForce RTX 5090 hat aktuell über 29 GB freien VRAM (Hauptmodell schläft bei Nichtgebrauch). Im isolierten AIDO-Temporärspeicher liegen 3 temporäre Cache-Dateien (448 KB). Deine persönlichen Dateien, Projekte und Modelle bleiben unangetastet. Darf ich das Fast Cleanup jetzt ausführen?",
    files: [
      "aido_vision_preview_01.tmp (64 KB)",
      "aido_whisper_chunk_01.tmp (128 KB)",
      "aido_slot_prompt_cache.tmp (256 KB)"
    ],
    bytes: 448 * 1024,
    actionKind: "FastCleanup"
  };

  setCharacterState("Warte auf deine Freigabe", false);
  aidoMessage.innerHTML = `
    <strong>${currentProposal.title}</strong><br/>
    ${currentProposal.explanation}<br/><br/>
    <em>Betroffene temporäre Dateien:</em> ${currentProposal.files.join(", ")}
  `;

  appendAudit(
    `[PermissionGateway] Freigabe-Anfrage ${currentProposal.requestId.slice(0, 8)} erstellt (FastCleanupPreview, 448 KB).`
  );

  actionButtons.innerHTML = "";
  const btnApprove = document.createElement("button");
  btnApprove.className = "approve";
  btnApprove.textContent = "✅ Ja, jetzt bereinigen";
  btnApprove.onclick = () => executeApprovedCleanup();

  const btnReject = document.createElement("button");
  btnReject.className = "reject";
  btnReject.textContent = "❌ Nein, abbrechen";
  btnReject.onclick = () => rejectCleanup();

  actionButtons.append(btnApprove, btnReject);
}

function executeApprovedCleanup(): void {
  if (!currentProposal) return;
  setCharacterState("Führe Fast Cleanup aus...", true);
  const reqId = currentProposal.requestId.slice(0, 8);

  setTimeout(() => {
    setCharacterState("Erfolgreich bereinigt!", false);
    aidoMessage.innerHTML = `
      <strong>Fast Cleanup abgeschlossen!</strong><br/>
      Ich habe die 3 temporären Cache-Dateien (448 KB) sicher entfernt und das Ergebnis in deinem lokalen SQLite/FTS5-Gedächtnis gespeichert.
    `;
    appendAudit(
      `[WindowsBridge] FastCleanupExecute mit Einmal-Token für Anfrage ${reqId} ausgeführt (3 Dateien entfernt, 448 KB frei).`
    );
    appendAudit(
      `[MemoryEngine] Episodische Erinnerung gespeichert: 'Fast Cleanup nach Nutzerfreigabe ausgeführt'.`
    );
    currentProposal = null;
    renderDefaultButtons();
  }, 350);
}

function rejectCleanup(): void {
  if (!currentProposal) return;
  const reqId = currentProposal.requestId.slice(0, 8);
  setCharacterState("Bereit (Keine Änderung)", false);
  aidoMessage.textContent =
    "Verstanden! Ich habe nichts verändert und alle Dateien genau so belassen, wie sie sind.";
  appendAudit(
    `[PermissionGateway] Anfrage ${reqId} vom Nutzer abgelehnt — keine Systemänderung durchgeführt.`
  );
  currentProposal = null;
  renderDefaultButtons();
}

function showAppIntelligenceAndProjects(): void {
  setCharacterState("App- & Projekt-Intelligenz aktiv", false);

  let html = `
    <strong>🧩 AIDO (Agentic Intelligent Developing Operator) — App Intelligence & Projekte</strong><br/>
    Hier ist die Übersicht deiner erkannten Desktop-Apps und deines kanonischen Projektkatalogs auf diesem PC. Alle Aktionen erfordern eine explizite Nutzer-Freigabe über das Permission Gateway.
    
    <div class="intel-section">
      <div class="intel-heading">📱 Erkannte Windows-Programme & Geprüfte Fähigkeiten</div>
      <div class="apps-grid">
  `;

  for (const app of appsData) {
    const capsHtml = app.capabilities
      .map((c) => {
        const riskClass = c.risk.toLowerCase();
        const approvalBadge = c.requiresApproval ? "🛡️ Freigabepflicht" : "⚡ Direkt";
        return `<span class="cap-badge ${riskClass}" title="${c.titleDe}">• ${c.capabilityId} (${approvalBadge})</span>`;
      })
      .join("");

    html += `
      <div class="app-card">
        <div class="app-header">
          <div class="app-title-group">
            <span class="app-name">${app.displayName}</span>
            <span class="tag">${app.preferredControl}</span>
          </div>
          <label class="app-toggle">
            <input type="checkbox" data-app-id="${app.appId}" ${app.userEnabled ? "checked" : ""} />
            <span>${app.userEnabled ? "Aktiv" : "Aus"}</span>
          </label>
        </div>
        <div class="app-meta">Herausgeber: ${app.publisher} · Pfad: <code>${app.installPath}</code></div>
        <div class="cap-list">
          ${capsHtml}
        </div>
      </div>
    `;
  }

  html += `
      </div>
      <div class="intel-heading" style="margin-top: 10px;">📁 Kanonischer Projektkatalog</div>
  `;

  for (const proj of projectsData) {
    const stackTags = proj.stack.map((s) => `<span class="tag">${s}</span>`).join(" ");
    html += `
      <div class="project-card">
        <div class="project-header">
          <span class="project-title">${proj.name}</span>
          <div class="tags-row">
            <span class="tag branch">Branch: ${proj.gitBranch}</span>
            <span class="tag dirty">Status: ${proj.gitStatus}</span>
          </div>
        </div>
        <div class="project-meta">Kanonischer Pfad: <code>${proj.canonicalPath}</code></div>
        <div class="tags-row">
          <span class="tag" style="color: #cbd5e1;">Stack:</span> ${stackTags}
          <span class="tag action">Empfohlene Aktion: ${proj.recommendedAction}</span>
        </div>
        <button class="btn-launch" id="btn-launch-aido">🚀 Projekt-Start vorbereiten (Freigabe anfordern)</button>
      </div>
    `;
  }

  html += `</div>`;

  aidoMessage.innerHTML = html;

  // Add event listeners to app checkboxes
  const checkboxes = aidoMessage.querySelectorAll<HTMLInputElement>("input[type='checkbox']");
  checkboxes.forEach((cb) => {
    cb.addEventListener("change", (e) => {
      const target = e.target as HTMLInputElement;
      const appId = target.dataset.appId;
      const app = appsData.find((a) => a.appId === appId);
      if (app) {
        app.userEnabled = target.checked;
        const labelSpan = target.parentElement?.querySelector("span");
        if (labelSpan) {
          labelSpan.textContent = app.userEnabled ? "Aktiv" : "Aus";
        }
        appendAudit(
          `[AppIntelligence] App '${app.displayName}' ${app.userEnabled ? "aktiviert" : "deaktiviert"} (Nutzer-Einstellung).`
        );
      }
    });
  });

  // Add event listener to project launch button
  const launchBtn = aidoMessage.querySelector<HTMLButtonElement>("#btn-launch-aido");
  if (launchBtn) {
    launchBtn.addEventListener("click", () => {
      prepareProjectLaunchPreview("AIDO");
    });
  }

  appendAudit(
    `[AppIntelligence] Umgebungserkennung geladen: ${appsData.length} bekannte Apps & ${projectsData.length} Projekt(e) im Katalog.`
  );

  // Render contextual action buttons
  actionButtons.innerHTML = "";
  const btnLaunchQuick = document.createElement("button");
  btnLaunchQuick.textContent = "🚀 AIDO Projekt-Start anfordern";
  btnLaunchQuick.onclick = () => prepareProjectLaunchPreview("AIDO");

  const btnBack = document.createElement("button");
  btnBack.textContent = "🔙 Zurück zur Übersicht";
  btnBack.onclick = () => {
    setCharacterState("Bereit (Lokal)", false);
    aidoMessage.innerHTML =
      "Hallo! Ich bin <b>AIDO (Agentic Intelligent Developing Operator)</b>. Ich laufe lokal auf deiner RTX 5090 im Modus <b>Private Offline</b> — keine Daten verlassen deinen PC. Möchtest du einen System-Check durchführen, deine erkannte App- & Projektumgebung prüfen oder temporäre Caches bereinigen?";
    renderDefaultButtons();
  };

  actionButtons.append(btnLaunchQuick, btnBack);
}

function prepareProjectLaunchPreview(projectName: string): void {
  const project = projectsData.find((p) => p.name === projectName) || projectsData[0];
  setCharacterState("Warte auf deine Freigabe", false);

  currentProposal = {
    requestId: crypto.randomUUID(),
    title: `Projekt-Start: ${project.name}`,
    explanation: `Ich habe dein Projekt '${project.name}' gefunden (Stack: ${project.stack.join(", ")}). Ich würde die Arbeitsumgebung prüfen und den Startbefehl '${project.recommendedAction}' im Verzeichnis '${project.canonicalPath}' vorbereiten. Soll ich das für dich ausführen?`,
    files: [project.canonicalPath, project.recommendedAction],
    bytes: 0,
    actionKind: "LaunchProject"
  };

  const reqId = currentProposal.requestId.slice(0, 8);

  aidoMessage.innerHTML = `
    <strong>${currentProposal.title} (Permission Gateway Freigabe)</strong><br/>
    ${currentProposal.explanation}<br/><br/>
    <div class="tags-row" style="margin-top: 6px;">
      <span class="tag">Ziel: ${project.canonicalPath}</span>
      <span class="tag action">Befehl: ${project.recommendedAction}</span>
      <span class="tag branch">Branch: ${project.gitBranch} [${project.gitStatus}]</span>
    </div>
  `;

  appendAudit(
    `[PermissionGateway] Freigabe-Anfrage ${reqId} erstellt (LaunchProject, Projekt: '${project.name}').`
  );

  actionButtons.innerHTML = "";
  const btnApprove = document.createElement("button");
  btnApprove.className = "approve";
  btnApprove.textContent = "✅ Ja, Projekt starten";
  btnApprove.onclick = () => executeApprovedLaunch(project);

  const btnReject = document.createElement("button");
  btnReject.className = "reject";
  btnReject.textContent = "❌ Nein, abbrechen";
  btnReject.onclick = () => rejectLaunch();

  actionButtons.append(btnApprove, btnReject);
}

function executeApprovedLaunch(project: ProjectItem): void {
  if (!currentProposal) return;
  const reqId = currentProposal.requestId.slice(0, 8);
  setCharacterState("Starte Projektumgebung...", true);

  setTimeout(() => {
    setCharacterState("Projektumgebung aktiv", false);
    aidoMessage.innerHTML = `
      <strong>🚀 Projekt '${project.name}' erfolgreich gestartet!</strong><br/>
      AIDO hat die Arbeitsumgebung für <strong>${project.name}</strong> vorbereitet und die Entwicklungsdienste initialisiert.<br/><br/>
      <em>Verzeichnis:</em> <code>${project.canonicalPath}</code><br/>
      <em>Ausgeführter Startbefehl:</em> <code>${project.recommendedAction}</code><br/>
      <em>Status:</em> Entwicklungs-Server aktiv, Fenster bereit.
    `;
    appendAudit(
      `[WindowsBridge] LaunchProject mit Einmal-Token für Anfrage ${reqId} ausgeführt (${project.recommendedAction} in ${project.canonicalPath}).`
    );
    appendAudit(
      `[MemoryEngine] SystemMemory gespeichert: 'Projekt-Start ${project.name} freigegeben & ausgeführt'.`
    );
    currentProposal = null;
    renderDefaultButtons();
  }, 350);
}

function rejectLaunch(): void {
  if (!currentProposal) return;
  const reqId = currentProposal.requestId.slice(0, 8);
  setCharacterState("Bereit (Keine Änderung)", false);
  aidoMessage.textContent =
    "Projekt-Start abgebrochen. Keine Prozesse gestartet und keine Dateien verändert.";
  appendAudit(
    `[PermissionGateway] Anfrage ${reqId} für Projekt-Start vom Nutzer abgelehnt — keine Ausführung.`
  );
  currentProposal = null;
  renderDefaultButtons();
}

function showPrivacyCenter(): void {
  setCharacterState("Privacy Center aktiv", false);
  aidoMessage.innerHTML = `
    <strong>🔒 AIDO Privacy Center — Modus: Private Offline</strong><br/>
    • <b>KI-Verarbeitung:</b> 100 % lokal (Qwen3.8-27B-UD-Q4_K_M auf 127.0.0.1:18434)<br/>
    • <b>Screenshots & Vision:</b> 100 % lokal (mmproj-Qwen3.8-27B-BF16)<br/>
    • <b>Spracherkennung:</b> 100 % lokal (Whisper-small CUDA auf 127.0.0.1:18080)<br/>
    • <b>Memory & Wissen:</b> Lokale SQLite/FTS5-Datenbank (%LOCALAPPDATA%\\AIDO\\data)<br/>
    • <b>Externe Verbindungen:</b> Technisch durch das Permission Gateway blockiert.
  `;
  appendAudit("[PrivacyCenter] Status geprüft: Private Offline aktiv (nur 127.0.0.1 erlaubt).");
  renderDefaultButtons();
}

function showMemorySummary(): void {
  setCharacterState("Memory & Wissen", false);
  aidoMessage.innerHTML = `
    <strong>🧠 AIDO Lokales Gedächtnis (SQLite + FTS5)</strong><br/>
    • <b>System Memory:</b> Predator Orion 7000 · RTX 5090 (32 GB VRAM) · 64 GB RAM · Windows 11<br/>
    • <b>Procedural Memory:</b> Geprüfter Ablauf für 'System-Gesundheitscheck & Fast Cleanup' und 'App/Project Launch'<br/>
    • <b>Kontrolle:</b> Alle Erinnerungen sind lokal gespeichert, mit Quelle & Zeitstempel versehen und jederzeit einsehbar oder löschbar.
  `;
  renderDefaultButtons();
}

function getStatusBadge(status: InstallerStep["status"]): { label: string; className: string } {
  switch (status) {
    case "Completed":
      return { label: "✓ Abgeschlossen", className: "completed" };
    case "Paused":
      return { label: "⏸️ Pausiert", className: "paused" };
    case "RolledBack":
      return { label: "⏪ Zurückgesetzt", className: "rolledback" };
    default:
      return { label: "⏳ Ausstehend", className: "pending" };
  }
}

function showSmartInstallerWizard(): void {
  setCharacterState("Smart Installer aktiv", false);

  const nextResumable = installerStepsData.find(
    (s) => s.status === "Paused" || s.status === "Pending"
  );
  const completedCount = installerStepsData.filter((s) => s.status === "Completed").length;

  let html = `
    <strong>🚀 AIDO (Agentic Intelligent Developing Operator) — Smart Installer (9-Phasen-Setup)</strong><br/>
    Transaktionales, fehlertolerantes Setup mit automatischer Hardware-Klassenerkennung und lokaler Modell-Wiederverwendung.<br/>

    <div class="installer-section">
      <!-- 3 getrennte ETA-Boxen -->
      <div class="eta-forecast-grid">
        <div class="eta-forecast-box highlight">
          <div class="eta-title">⚡ AIDO einsatzbereit in</div>
          <div class="eta-main-val">45 Sek.</div>
          <div class="eta-desc">0 MB Download nötig · Vorhandene lokale Modelle werden direkt eingebunden</div>
        </div>
        <div class="eta-forecast-box">
          <div class="eta-title">👤 Persönliche Einrichtung in</div>
          <div class="eta-main-val">~2,5 Min.</div>
          <div class="eta-desc">App-Erkennung, Berechtigungsprofil & lokaler Wissens-Index</div>
        </div>
        <div class="eta-forecast-box warning">
          <div class="eta-title">🧩 Erweiterte Integration</div>
          <div class="eta-main-val">Isoliert</div>
          <div class="eta-desc">Isolierte Adapterprüfung ohne künstliches Zeitlimit (1 individuelle App)</div>
        </div>
      </div>

      <!-- Hardware-Klasse & Modell-Wiederverwendung -->
      <div class="models-reuse-box">
        <div class="models-reuse-header">
          <span>🖥️ Hardware: HighEndGpu (RTX 5090 · 32 GB VRAM · 64 GB RAM)</span>
          <span class="tag branch">4/4 Modelle lokal erkannt — 0 GB erneuter Download</span>
        </div>
        <div style="display: flex; flex-direction: column; gap: 6px;">
  `;

  for (const m of modelsReuseData) {
    const sizeMb = Math.round(m.sizeBytes / (1024 * 1024));
    html += `
          <div class="model-item">
            <div>
              <span style="color: #6ee7b7; font-weight: 600;">✓ Gefunden:</span>
              <span style="color: #f3f4f6; margin-left: 6px; font-weight: 600;">${m.modelName}</span>
              <span style="color: #94a3b8; font-size: 0.74rem; margin-left: 6px;">(${m.role} · ${sizeMb} MB)</span>
            </div>
            <div class="model-path">${m.expectedPath}</div>
          </div>
    `;
  }

  html += `
        </div>
      </div>

      <!-- 9 Phasen-Liste -->
      <div class="intel-heading">📋 9 Transaktionale Installationsphasen (${completedCount}/9 abgeschlossen)</div>
      <div class="phases-list">
  `;

  for (const step of installerStepsData) {
    const badge = getStatusBadge(step.status);
    const isActive = nextResumable && nextResumable.stepIndex === step.stepIndex;
    const activeClass = isActive ? "active" : step.status.toLowerCase();

    html += `
        <div class="phase-row ${activeClass}">
          <div class="phase-info">
            <div class="phase-title">
              <span>Phase ${step.stepIndex}/9: ${step.titleDe}</span>
              ${isActive ? '<span class="tag action" style="font-size: 0.68rem;">Nächster Schritt</span>' : ""}
            </div>
            <div class="phase-rollback">
              <span>🛡️ Rollback-Sicherheit: ${step.rollbackAction}</span>
              ${step.details ? ` · <span style="color: #a5b4fc;">${step.details}</span>` : ""}
            </div>
          </div>
          <div>
            <span class="phase-badge ${badge.className}">${badge.label}</span>
          </div>
        </div>
    `;
  }

  html += `
      </div>
    </div>
  `;

  aidoMessage.innerHTML = html;

  // Build interactive action buttons
  actionButtons.innerHTML = "";

  const btnNext = document.createElement("button");
  btnNext.className = "approve";
  if (nextResumable) {
    btnNext.textContent = `▶️ Nächste Phase abschließen (Phase ${nextResumable.stepIndex})`;
    btnNext.onclick = () => completeNextInstallerPhase();
  } else {
    btnNext.textContent = "✅ Alle 9 Phasen abgeschlossen!";
    btnNext.disabled = true;
  }

  const btnPause = document.createElement("button");
  btnPause.textContent = "⏸️ Installation pausieren (Wiederaufnahme-Test)";
  btnPause.onclick = () => pauseInstaller();

  const btnRollback = document.createElement("button");
  btnRollback.className = "reject";
  btnRollback.textContent = "⏪ Transaktionales Rollback";
  btnRollback.onclick = () => rollbackInstaller();

  const btnBack = document.createElement("button");
  btnBack.textContent = "🔙 Zurück zur Übersicht";
  btnBack.onclick = () => {
    setCharacterState("Bereit (Lokal)", false);
    aidoMessage.innerHTML =
      "Hallo! Ich bin <b>AIDO (Agentic Intelligent Developing Operator)</b>. Ich laufe lokal auf deiner RTX 5090 im Modus <b>Private Offline</b> — keine Daten verlassen deinen PC. Möchtest du einen System-Check durchführen, deine erkannte App- & Projektumgebung prüfen oder temporäre Caches bereinigen?";
    renderDefaultButtons();
  };

  actionButtons.append(btnNext, btnPause, btnRollback, btnBack);

  appendAudit(
    `[SmartInstaller] Wizard geöffnet: HighEndGpu erkannt, 4/4 Modelle lokal wiederverwendet, ${completedCount}/9 Phasen abgeschlossen.`
  );
}

function completeNextInstallerPhase(): void {
  const next = installerStepsData.find((s) => s.status === "Paused" || s.status === "Pending");
  if (!next) return;

  setCharacterState(`Schließe Phase ${next.stepIndex} ab...`, true);

  setTimeout(() => {
    next.status = "Completed";
    const now = new Date().toLocaleTimeString("de-DE");
    next.details = `Erfolgreich abgeschlossen um ${now}`;

    appendAudit(
      `[SmartInstaller] Phase ${next.stepIndex}/9 ('${next.titleDe}') erfolgreich abgeschlossen.`
    );

    showSmartInstallerWizard();
  }, 250);
}

function pauseInstaller(): void {
  const target = installerStepsData.find((s) => s.status === "Pending" || s.status === "Paused");
  if (!target) return;

  target.status = "Paused";
  const now = new Date().toLocaleTimeString("de-DE");
  target.details = `Pausiert durch Nutzer um ${now} (Zustand in SQLite gesichert)`;

  appendAudit(
    `[SmartInstaller] Installation bei Phase ${target.stepIndex}/9 ('${target.titleDe}') pausiert. Wiederaufnahme jederzeit möglich.`
  );

  showSmartInstallerWizard();
}

function rollbackInstaller(): void {
  let rolledBackCount = 0;
  for (const step of installerStepsData) {
    if (step.status === "Completed" || step.status === "Paused") {
      step.status = "RolledBack";
      step.details = `Rollback ausgeführt: ${step.rollbackAction} (Modell-Dateien auf Festplatte unberührt)`;
      rolledBackCount++;
    }
  }

  appendAudit(
    `[SmartInstaller] Transaktionales Rollback für ${rolledBackCount} Phase(n) ausgeführt. Modell-Dateien blieben unangetastet.`
  );

  showSmartInstallerWizard();
}

// Character Mouse Interactions
avatarOrb.addEventListener("mousedown", (e) => {
  if (e.button === 0) {
    pressTimer = window.setTimeout(() => {
      setCharacterState("Höre zu (Whisper Push-to-Talk)...", true);
      aidoMessage.textContent =
        "🎤 Mikrofon-Aufnahme aktiv (Loslassen zum lokalen Transkribieren über 127.0.0.1:18080)...";
    }, 450);
  } else if (e.button === 1) {
    e.preventDefault();
    setCharacterState("Screenshot-Vorschau", false);
    aidoMessage.innerHTML =
      "📸 <strong>Screenshot-Kontext bereit:</strong> Im privaten Offline-Modus wird das Bild ausschließlich lokal von <code>mmproj-Qwen3.8-27B-BF16</code> ausgewertet.";
    appendAudit("[VisionContext] Lokaler Screenshot-Kontext vorgemerkt (keine Cloud-Übertragung).");
  }
});

avatarOrb.addEventListener("mouseup", (e) => {
  if (e.button === 0 && pressTimer !== null) {
    clearTimeout(pressTimer);
    pressTimer = null;
  }
});

window.addEventListener("contextmenu", (e) => {
  e.preventDefault();
  contextMenu.style.display = "block";
  contextMenu.style.left = `${e.clientX}px`;
  contextMenu.style.top = `${e.clientY}px`;
});

window.addEventListener("click", () => {
  contextMenu.style.display = "none";
});

contextMenu.addEventListener("click", (e) => {
  const target = (e.target as HTMLElement).closest(".context-item") as HTMLElement | null;
  if (!target) return;
  const action = target.dataset.action;
  if (action === "health" || action === "cleanup") {
    startSystemHealthAndCleanupPreview();
  } else if (action === "apps") {
    showAppIntelligenceAndProjects();
  } else if (action === "installer") {
    showSmartInstallerWizard();
  } else if (action === "privacy") {
    showPrivacyCenter();
  } else if (action === "memory") {
    showMemorySummary();
  } else if (action === "sleep") {
    setCharacterState("Schlafmodus (VRAM freigegeben)", false);
    aidoMessage.textContent =
      "💤 Ich bin im Schlafmodus. Das 27B-Hauptmodell gibt seinen VRAM automatisch frei und wacht erst bei deiner nächsten Anfrage wieder auf.";
    appendAudit("[Runtime] AIDO in den Schlafmodus versetzt (VRAM-Entladung aktiv).");
  }
});

renderDefaultButtons();
