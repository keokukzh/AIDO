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

interface MemoryItem {
  id: number;
  createdAt: string;
  layer: "WorkingMemory" | "EpisodicMemory" | "SemanticMemory" | "SystemMemory";
  category: string;
  summary: string;
  details: string;
  source: string;
  validity: "active" | "compacted" | "forgotten_by_user";
}

const memoriesData: MemoryItem[] = [
  {
    id: 1,
    createdAt: "2026-11-09T06:00:00Z",
    layer: "SystemMemory",
    category: "Hardware",
    summary: "Predator Orion 7000 · RTX 5090 (32 GB VRAM) · 64 GB RAM",
    details: "HighEndGpu Profil mit vollständiger lokaler Modell-Wiederverwendung und 0 GB erneutem Downloadbedarf.",
    source: "AidoCore::evaluate_smart_installer_plan",
    validity: "active"
  },
  {
    id: 2,
    createdAt: "2026-11-09T06:01:00Z",
    layer: "SystemMemory",
    category: "Security",
    summary: "Permission Gateway Modus Private Offline (127.0.0.1)",
    details: "Alle Netzwerkaufrufe außerhalb der lokalen Loopback-Adresse werden technisch blockiert.",
    source: "AidoCore::new",
    validity: "active"
  },
  {
    id: 3,
    createdAt: "2026-11-09T06:02:15Z",
    layer: "EpisodicMemory",
    category: "FastCleanup",
    summary: "Fast Cleanup nach Nutzerfreigabe ausgeführt (3 Dateien, 448 KB bereinigt)",
    details: "Temporäre Staging-Caches sicher entfernt. VRAM vor Bereinigung: 2300 MiB, frei: 29888 MiB.",
    source: "AidoCore::approve_and_execute_fast_cleanup",
    validity: "active"
  },
  {
    id: 4,
    createdAt: "2026-11-09T06:03:00Z",
    layer: "EpisodicMemory",
    category: "FastCleanup",
    summary: "Zweites Fast Cleanup simuliert (2 Dateien, 128 KB)",
    details: "Automatische periodische Cache-Bereinigung vor Idle-Phase. 0 verwaiste Prozesse verblieben.",
    source: "AidoCore::demo",
    validity: "active"
  },
  {
    id: 5,
    createdAt: "2026-11-09T06:04:30Z",
    layer: "WorkingMemory",
    category: "UserSession",
    summary: "Aktuelle Entwickler-Sitzung im Workspace C:\\Users\\keoku\\Desktop\\AIDO",
    details: "Aktives Projekt AIDO, Stack: Node.js/TypeScript + Rust, Branch: main.",
    source: "AidoCore::discover_and_register_environment",
    validity: "active"
  }
];

const runtimeStatusData = {
  llamaServer: {
    serviceId: "llama_router",
    displayName: "Llama Router (Qwen3.8-27B + Vision + DFlash2)",
    endpoint: "http://127.0.0.1:18434",
    state: "SleepingZeroVram" as "StoppedStandby" | "SleepingZeroVram" | "WarmActive",
    binaryPath: "D:\\AI_Models\\unsloth\\llama.cpp\\build\\bin\\Release\\llama-server.exe",
    binaryExists: true,
    sleepIdleSeconds: 300,
    launchArgs: [
      "--host 127.0.0.1",
      "--port 18434",
      "--models-Dir D:\\AI_Models\\llm",
      "--ctx-size 32768",
      "-ngl 99",
      "--gpu-layers-draft 99",
      "--flash-attn on",
      "--sleep-idle-seconds 300"
    ],
    humanStatus: "Null-VRAM-Ruhemodus (Zero-VRAM Standby). Nach 300s Inaktivität automatisch entladen (0 MiB VRAM) und bereit für sofortigen Aufruf."
  },
  whisperServer: {
    serviceId: "whisper_asr",
    displayName: "Whisper ASR (German Speech-to-Text)",
    endpoint: "http://127.0.0.1:18080",
    state: "StoppedStandby" as "StoppedStandby" | "WarmActive",
    binaryPath: "D:\\AI_Models\\unsloth\\whisper.cpp\\build\\bin\\Release\\whisper-server.exe",
    binaryExists: true,
    cudaDllReady: true,
    launchArgs: [
      "--host 127.0.0.1",
      "--port 18080",
      "-m D:\\AI_Models\\unsloth\\whisper.cpp\\models\\ggml-small.bin",
      "-l de",
      "-fa"
    ],
    humanStatus: "Gestoppt (Bereit für On-Demand-Start auf Abruf, 0 MiB VRAM)."
  },
  cudaDllPath: "C:\\AI\\llamacpp-cuda",
  cudaDllReady: true
};

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

  const btnRuntime = document.createElement("button");
  btnRuntime.textContent = "⚡ Runtime & Memory-Kompaktierung";
  btnRuntime.onclick = () => showRuntimeAndMemoryCompaction();

  const btnPrivacy = document.createElement("button");
  btnPrivacy.textContent = "🔒 Privacy Center anzeigen";
  btnPrivacy.onclick = () => showPrivacyCenter();

  const btnMemory = document.createElement("button");
  btnMemory.textContent = "🧠 Memory & Wissen (4-Stufen)";
  btnMemory.onclick = () => showMemorySummary();

  actionButtons.append(btnCheck, btnApps, btnInstaller, btnRuntime, btnPrivacy, btnMemory);
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

function showMemorySummary(filterLayer = "Alle"): void {
  setCharacterState("Memory & Privacy Center (4-Stufen)", false);

  const totalCount = memoriesData.length;
  const workingCount = memoriesData.filter((m) => m.layer === "WorkingMemory" && m.validity === "active").length;
  const episodicCount = memoriesData.filter((m) => m.layer === "EpisodicMemory" && m.validity === "active").length;
  const semanticCount = memoriesData.filter((m) => m.layer === "SemanticMemory" && m.validity === "active").length;
  const systemCount = memoriesData.filter((m) => m.layer === "SystemMemory" && m.validity === "active").length;

  const filteredMemories = filterLayer === "Alle"
    ? memoriesData
    : memoriesData.filter((m) => m.layer === filterLayer);

  let html = `
    <strong>🧠 AIDO 4-Stufen-Gedächtnis & Privacy Center (SQLite + FTS5)</strong><br/>
    Vollständig lokales, 4-stufiges Gedächtnis nach <b>MASTERPLAN Kapitel 5.2</b>. Alle Erinnerungen sind transparent nachvollziehbar, lokal gespeichert und können vom Nutzer mit einem Klick vergessen werden.<br/>

    <!-- Layer Filter Badges -->
    <div class="memory-layers-bar">
      <span class="filter-badge ${filterLayer === "Alle" ? "active" : ""}" data-filter="Alle">Alle (${totalCount})</span>
      <span class="filter-badge ${filterLayer === "WorkingMemory" ? "active" : ""}" data-filter="WorkingMemory">Stufe 1: Working (${workingCount})</span>
      <span class="filter-badge ${filterLayer === "EpisodicMemory" ? "active" : ""}" data-filter="EpisodicMemory">Stufe 2: Episodic (${episodicCount})</span>
      <span class="filter-badge ${filterLayer === "SemanticMemory" ? "active" : ""}" data-filter="SemanticMemory">Stufe 3: Semantic (${semanticCount})</span>
      <span class="filter-badge ${filterLayer === "SystemMemory" ? "active" : ""}" data-filter="SystemMemory">Stufe 4: System (${systemCount})</span>
    </div>

    ${episodicCount > 0 ? `
    <button class="btn-compact" id="btn-compact-memory">
      🧹 Episodisches Gedächtnis jetzt kompaktieren (${episodicCount} aktive Einträge → Stufe 3 Semantic)
    </button>
    ` : ""}

    <div class="memory-list">
  `;

  for (const m of [...filteredMemories].reverse()) {
    let layerClass = "working";
    if (m.layer === "EpisodicMemory") layerClass = "episodic";
    else if (m.layer === "SemanticMemory") layerClass = "semantic";
    else if (m.layer === "SystemMemory") layerClass = "system";

    let validityBadge = '<span class="tag branch">✓ Aktiv</span>';
    let cardClass = "";
    if (m.validity === "compacted") {
      validityBadge = '<span class="tag" style="background: rgba(148, 163, 184, 0.2); color: #cbd5e1;">📦 Kompaktiert (Semantisch verdichtet)</span>';
      cardClass = "compacted";
    } else if (m.validity === "forgotten_by_user") {
      validityBadge = '<span class="tag dirty" style="color: #f87171;">🗑️ Vom Nutzer vergessen</span>';
      cardClass = "forgotten";
    }

    const showForgetBtn = m.validity !== "forgotten_by_user";

    html += `
      <div class="memory-card ${cardClass}">
        <div class="memory-card-header">
          <div style="display: flex; align-items: center; gap: 8px;">
            <span class="layer-tag ${layerClass}">${m.layer}</span>
            <span style="font-weight: 700; color: #f3f4f6; font-size: 0.88rem;">${m.category}</span>
            <span style="font-size: 0.72rem; color: #94a3b8;">#${m.id} · ${m.createdAt.slice(0, 19).replace("T", " ")}</span>
          </div>
          <div style="display: flex; align-items: center; gap: 8px;">
            ${validityBadge}
            ${showForgetBtn ? `<button class="btn-forget" data-forget-id="${m.id}" title="Diese Erinnerung dauerhaft ungültig machen">🗑️ Vergessen</button>` : ""}
          </div>
        </div>
        <div style="font-weight: 600; font-size: 0.88rem; color: #e0e7ff;">${m.summary}</div>
        <div style="font-size: 0.78rem; color: #cbd5e1; white-space: pre-line;">${m.details}</div>
        <div style="font-size: 0.7rem; color: #94a3b8; font-family: monospace;">Quelle: ${m.source}</div>
      </div>
    `;
  }

  html += `</div>`;

  aidoMessage.innerHTML = html;

  // Add event listeners for layer filter badges
  const badges = aidoMessage.querySelectorAll<HTMLSpanElement>(".filter-badge");
  badges.forEach((b) => {
    b.addEventListener("click", () => {
      const filter = b.dataset.filter || "Alle";
      showMemorySummary(filter);
    });
  });

  // Add event listener for compaction button
  const compactBtn = aidoMessage.querySelector<HTMLButtonElement>("#btn-compact-memory");
  if (compactBtn) {
    compactBtn.addEventListener("click", () => {
      performMemoryCompaction();
    });
  }

  // Add event listeners for forget buttons
  const forgetButtons = aidoMessage.querySelectorAll<HTMLButtonElement>(".btn-forget");
  forgetButtons.forEach((btn) => {
    btn.addEventListener("click", (e) => {
      e.stopPropagation();
      const id = Number(btn.dataset.forgetId);
      if (id) {
        forgetMemoryEntryItem(id, filterLayer);
      }
    });
  });

  appendAudit(
    `[MemoryEngine] 4-Stufen-Gedächtnis aufgerufen: Filter '${filterLayer}', ${filteredMemories.length} Erinnerung(en) angezeigt.`
  );

  renderDefaultButtons();
}

function showRuntimeAndMemoryCompaction(): void {
  setCharacterState("Runtime & Memory-Kompaktierung", false);

  const activeEpisodicCount = memoriesData.filter(
    (m) => m.layer === "EpisodicMemory" && m.validity === "active"
  ).length;
  const semanticCount = memoriesData.filter(
    (m) => m.layer === "SemanticMemory" && m.validity === "active"
  ).length;
  const workingCount = memoriesData.filter(
    (m) => m.layer === "WorkingMemory" && m.validity === "active"
  ).length;
  const systemCount = memoriesData.filter(
    (m) => m.layer === "SystemMemory" && m.validity === "active"
  ).length;

  let html = `
    <strong>⚡ AIDO Live-Runtime-Supervisor & 4-Stufen-Memory-Kompaktierung</strong><br/>
    Überwachung der lokalen Inferenz-Dienste mit automatischem Null-VRAM-Schlafmodus und Verdichtung episodischer Aktivitäten in dauerhaftes semantisches Wissen.<br/>

    <div class="intel-section">
      <div class="intel-heading">🖥️ Lokale KI-Inferenz-Runtimes & CUDA 13.4 Status</div>
      <div class="runtime-grid">
        <!-- Llama Server Card -->
        <div class="runtime-card standby">
          <div class="runtime-header">
            <span class="runtime-title">${runtimeStatusData.llamaServer.displayName}</span>
            <span class="tag branch">💤 Zero-VRAM Standby</span>
          </div>
          <div class="app-meta">Endpunkt: <code>${runtimeStatusData.llamaServer.endpoint}</code> · Executable: <code>${runtimeStatusData.llamaServer.binaryPath}</code></div>
          <div style="font-size: 0.8rem; color: #cbd5e1; margin-top: 4px;">
            ${runtimeStatusData.llamaServer.humanStatus}
          </div>
          <div class="tags-row" style="margin-top: 4px;">
            <span class="tag action">--sleep-idle-seconds ${runtimeStatusData.llamaServer.sleepIdleSeconds}</span>
            <span class="tag">Qwen3.8-27B-UD-Q4_K_M</span>
            <span class="tag">mmproj-BF16</span>
            <span class="tag">DFlash2 (151.5 Tok/s)</span>
          </div>
        </div>

        <!-- Whisper Server Card -->
        <div class="runtime-card standby">
          <div class="runtime-header">
            <span class="runtime-title">${runtimeStatusData.whisperServer.displayName}</span>
            <span class="tag branch">⚡ CUDA 13.4 On-Demand</span>
          </div>
          <div class="app-meta">Endpunkt: <code>${runtimeStatusData.whisperServer.endpoint}</code> · Modell: <code>ggml-small.bin (de)</code></div>
          <div style="font-size: 0.8rem; color: #cbd5e1; margin-top: 4px;">
            ${runtimeStatusData.whisperServer.humanStatus}
          </div>
          <div class="tags-row" style="margin-top: 4px;">
            <span class="tag action">CUDA DLL: ${runtimeStatusData.cudaDllPath} (✓ Bereit)</span>
            <span class="tag">Sprache: Deutsch</span>
            <span class="tag">Latenz: ~0.34s</span>
          </div>
        </div>
      </div>

      <div class="intel-heading" style="margin-top: 10px;">🧠 4-Stufen-Memory-Kompaktierung (Stufe 2 Episodic → Stufe 3 Semantic)</div>
      <div class="eta-forecast-grid">
        <div class="eta-forecast-box">
          <div class="eta-title">Stufe 1 · Working Memory</div>
          <div class="eta-main-val">${workingCount} aktiv</div>
          <div class="eta-desc">Flüchtiger Sitzungskontext & aktuelle Instruktionen</div>
        </div>
        <div class="eta-forecast-box ${activeEpisodicCount >= 2 ? "highlight" : ""}">
          <div class="eta-title">Stufe 2 · Episodic Memory</div>
          <div class="eta-main-val">${activeEpisodicCount} aktiv</div>
          <div class="eta-desc">Einzelaktionen, Bereinigungs-Logs & System-Ereignisse</div>
        </div>
        <div class="eta-forecast-box">
          <div class="eta-title">Stufe 3 · Semantic Memory</div>
          <div class="eta-main-val">${semanticCount} verdichtet</div>
          <div class="eta-desc">Dauerhaft konsolidiertes Wissen & Verhaltensmuster</div>
        </div>
        <div class="eta-forecast-box">
          <div class="eta-title">Stufe 4 · System Memory</div>
          <div class="eta-main-val">${systemCount} aktiv</div>
          <div class="eta-desc">Hardware, Berechtigungsprofil & Sicherheitsrichtlinien</div>
        </div>
      </div>

      <div style="margin-top: 8px;">
        <button class="btn-compact" id="btn-compact-from-runtime">
          🧹 Episodisches Gedächtnis jetzt kompaktieren (Stufe 2 → Stufe 3 Semantic)
        </button>
      </div>
    </div>
  `;

  aidoMessage.innerHTML = html;

  const btnCompact = document.getElementById("btn-compact-from-runtime");
  if (btnCompact) {
    btnCompact.onclick = () => performMemoryCompaction();
  }

  appendAudit(
    `[RuntimeSupervisor] Status abgefragt: Llama (Zero-VRAM Standby, --sleep-idle-seconds 300), Whisper (CUDA 13.4), ${activeEpisodicCount} aktive episodische Erinnerungen.`
  );

  actionButtons.innerHTML = "";
  const btnDoCompact = document.createElement("button");
  btnDoCompact.className = "approve";
  btnDoCompact.textContent = "🧹 Jetzt kompaktieren";
  btnDoCompact.onclick = () => performMemoryCompaction();

  const btnOpenMemory = document.createElement("button");
  btnOpenMemory.textContent = "🧠 4-Stufen-Gedächtnis im Detail";
  btnOpenMemory.onclick = () => showMemorySummary();

  const btnBack = document.createElement("button");
  btnBack.textContent = "🔙 Zurück zur Übersicht";
  btnBack.onclick = () => {
    setCharacterState("Bereit (Lokal)", false);
    aidoMessage.innerHTML =
      "Hallo! Ich bin <b>AIDO (Agentic Intelligent Developing Operator)</b>. Ich laufe lokal auf deiner RTX 5090 im Modus <b>Private Offline</b> — keine Daten verlassen deinen PC. Möchtest du einen System-Check durchführen, deine erkannte App- & Projektumgebung prüfen oder temporäre Caches bereinigen?";
    renderDefaultButtons();
  };

  actionButtons.append(btnDoCompact, btnOpenMemory, btnBack);
}

function performMemoryCompaction(): void {
  const activeEpisodics = memoriesData.filter(
    (m) => m.layer === "EpisodicMemory" && m.validity === "active"
  );

  if (activeEpisodics.length === 0) {
    appendAudit("[MemoryEngine] Kompaktierung übersprungen: Keine aktiven episodischen Erinnerungen vorhanden.");
    showMemorySummary("EpisodicMemory");
    return;
  }

  setCharacterState("Kompaktiere episodisches Gedächtnis...", true);

  setTimeout(() => {
    const count = activeEpisodics.length;
    const nextId = Math.max(...memoriesData.map((m) => m.id), 0) + 1;
    const now = new Date().toISOString();

    // Mark active episodic entries as compacted
    for (const m of activeEpisodics) {
      m.validity = "compacted";
    }

    // Create consolidated semantic entry
    const newSemantic: MemoryItem = {
      id: nextId,
      createdAt: now,
      layer: "SemanticMemory",
      category: "FastCleanup",
      summary: `4-Stufen-Kompaktierung: ${count} episodische Aktivitäten konsolidiert in semantisches Wissen (FastCleanup)`,
      details: `Semantisch verdichtetes Wissen aus ${count} episodischen Einzelereignissen:\n` +
        activeEpisodics.map((e) => `• [${e.createdAt.slice(0, 19).replace("T", " ")}] ID #${e.id}: ${e.summary}`).join("\n"),
      source: "MemoryEngine::compact_episodic_memories",
      validity: "active"
    };

    memoriesData.push(newSemantic);

    appendAudit(
      `[MemoryEngine] 4-Stufen-Kompaktierung erfolgreich: ${count} episodische Einträge verdichtet zu SemanticMemory #${nextId}.`
    );

    setCharacterState("Kompaktierung abgeschlossen", false);
    showMemorySummary("SemanticMemory");
  }, 350);
}

function forgetMemoryEntryItem(id: number, currentFilter?: string): void {
  const item = memoriesData.find((m) => m.id === id);
  if (!item) return;

  item.validity = "forgotten_by_user";
  appendAudit(
    `[MemoryEngine] Nutzer hat Erinnerung #${id} vergessen ('${item.summary.slice(0, 35)}...').`
  );
  showMemorySummary(currentFilter);
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
  } else if (action === "runtime") {
    showRuntimeAndMemoryCompaction();
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
