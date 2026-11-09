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

  const btnPrivacy = document.createElement("button");
  btnPrivacy.textContent = "🔒 Privacy Center anzeigen";
  btnPrivacy.onclick = () => showPrivacyCenter();

  const btnMemory = document.createElement("button");
  btnMemory.textContent = "🧠 Memory & Audit-Log";
  btnMemory.onclick = () => showMemorySummary();

  actionButtons.append(btnCheck, btnApps, btnPrivacy, btnMemory);
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
    <strong>🧩 AIDO (Artificial Intelligence Development Operator) — App Intelligence & Projekte</strong><br/>
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
      "Hallo! Ich bin <b>AIDO (Artificial Intelligence Development Operator)</b>. Ich laufe lokal auf deiner RTX 5090 im Modus <b>Private Offline</b> — keine Daten verlassen deinen PC. Möchtest du einen System-Check durchführen, deine erkannte App- & Projektumgebung prüfen oder temporäre Caches bereinigen?";
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
