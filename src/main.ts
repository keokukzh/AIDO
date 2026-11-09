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

type FocusMode = "NormalIdle" | "FullscreenOrGaming" | "VoiceRecordingOrCall";
let currentFocusMode: FocusMode = "NormalIdle";
const mutedCategories: Set<string> = new Set();
const dismissalCounts: Map<string, number> = new Map();
let deepCleanupExecuted = false;

interface DeepCleanupCandidateItem {
  fileName: string;
  category: "StaleBuildArtifacts" | "OrphanedCrashDumps" | "ObsoleteShaderAndModelTemp";
  categoryLabelDe: string;
  sizeBytes: number;
  reasonGerman: string;
}

const deepCleanupCandidatesData: DeepCleanupCandidateItem[] = [
  {
    fileName: "rust_incremental_old_target.old_build",
    category: "StaleBuildArtifacts",
    categoryLabelDe: "Veralteter Build-Cache",
    sizeBytes: 512 * 1024,
    reasonGerman: "Veralteter inkrementeller Build-Cache eines früheren Kompiliervorgangs."
  },
  {
    fileName: "orphaned_webview_crash_dump.dmp_cache",
    category: "OrphanedCrashDumps",
    categoryLabelDe: "Verwaister Crash-Dump",
    sizeBytes: 256 * 1024,
    reasonGerman: "Verwaister WebView-Crash-Dump ohne aktiven Fehlerbericht."
  },
  {
    fileName: "obsolete_cuda_compute_cache.shader_tmp",
    category: "ObsoleteShaderAndModelTemp",
    categoryLabelDe: "Veralteter Shader- & Compute-Cache",
    sizeBytes: 384 * 1024,
    reasonGerman: "Veralteter GPU-Shader- und Compute-Zwischenspeicher."
  }
];

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

  const btnProactive = document.createElement("button");
  btnProactive.textContent = "💡 Proaktive Vorschläge";
  btnProactive.dataset.action = "proactive";
  btnProactive.onclick = () => showProactiveEngineView();

  const btnDeepCleanup = document.createElement("button");
  btnDeepCleanup.textContent = "🧽 Deep Cleanup";
  btnDeepCleanup.dataset.action = "deep-cleanup";
  btnDeepCleanup.onclick = () => startDeepCleanupPreview();

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

  actionButtons.append(
    btnCheck,
    btnProactive,
    btnDeepCleanup,
    btnApps,
    btnInstaller,
    btnRuntime,
    btnPrivacy,
    btnMemory
  );
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

function startDeepCleanupPreview(): void {
  setCharacterState("Analysiere Staging (Tiefenprüfung)...", true);

  const totalBytes = deepCleanupCandidatesData.reduce((acc, c) => acc + c.sizeBytes, 0);
  const totalKb = Math.round(totalBytes / 1024);

  currentProposal = {
    requestId: crypto.randomUUID(),
    title: "AIDO Deep Cleanup (Tiefenprüfung verwaister Build- & Shader-Caches)",
    explanation: `Ich habe das Bereinigungs-Staging lesend geprüft: Es wurden 3 verwaiste Artefakte (${totalKb} KB) in 3 Kategorien gefunden. Deine persönlichen Quellcodes, Projektdateien und KI-Modelle bleiben vollständig unberührt. Darf ich die Tiefenbereinigung jetzt ausführen?`,
    files: deepCleanupCandidatesData.map((c) => `${c.fileName} (${c.sizeBytes / 1024} KB, ${c.category})`),
    bytes: totalBytes,
    actionKind: "DeepCleanupExecute"
  };

  const reqId = currentProposal.requestId.slice(0, 8);

  let html = `
    <strong>🧽 AIDO Deep Cleanup (Tiefenprüfung & Freigabe)</strong><br/>
    ${currentProposal.explanation}<br/>

    <div class="deep-cleanup-grid">
  `;

  for (const c of deepCleanupCandidatesData) {
    html += `
      <div class="deep-cleanup-item">
        <div class="deep-cleanup-category">${c.categoryLabelDe}</div>
        <div class="deep-cleanup-size">${c.sizeBytes / 1024} KB</div>
        <div class="deep-cleanup-file">${c.fileName}</div>
        <div class="deep-cleanup-desc">${c.reasonGerman}</div>
      </div>
    `;
  }

  html += `
    </div>
    <div style="font-size: 0.8rem; color: #34d399; font-weight: 600; margin-top: 6px;">
      ✓ Gesamtpotenzial: ${totalKb} KB in 3 Kategorien · Freigabe-ID: ${currentProposal.requestId}
    </div>
  `;

  setCharacterState("Warte auf deine Freigabe", false);
  aidoMessage.innerHTML = html;

  appendAudit(
    `[PermissionGateway] Freigabe-Anfrage ${reqId} erstellt (DeepCleanupExecute, 3 Kategorien, ${totalKb} KB).`
  );

  actionButtons.innerHTML = "";
  const btnApprove = document.createElement("button");
  btnApprove.className = "approve";
  btnApprove.textContent = "✅ Ja, Deep Cleanup ausführen";
  btnApprove.onclick = () => executeApprovedDeepCleanup();

  const btnReject = document.createElement("button");
  btnReject.className = "reject";
  btnReject.textContent = "❌ Nein, abbrechen";
  btnReject.onclick = () => rejectDeepCleanup();

  actionButtons.append(btnApprove, btnReject);
}

function executeApprovedDeepCleanup(): void {
  if (!currentProposal) return;
  setCharacterState("Führe Deep Cleanup aus...", true);
  const reqId = currentProposal.requestId.slice(0, 8);
  const freedKb = Math.round(currentProposal.bytes / 1024);

  setTimeout(() => {
    deepCleanupExecuted = true;
    setCharacterState("Tiefenbereinigung erfolgreich!", false);
    aidoMessage.innerHTML = `
      <strong>🧽 Deep Cleanup erfolgreich abgeschlossen!</strong><br/>
      Ich habe die 3 verwaisten Build-, Shader- und Crash-Dateien (<b>${freedKb} KB</b>) sicher aus dem Staging entfernt.<br/><br/>
      <em>Bereinigte Kategorien:</em><br/>
      • StaleBuildArtifacts (rust_incremental_old_target.old_build, 512 KB)<br/>
      • OrphanedCrashDumps (orphaned_webview_crash_dump.dmp_cache, 256 KB)<br/>
      • ObsoleteShaderAndModelTemp (obsolete_cuda_compute_cache.shader_tmp, 384 KB)<br/><br/>
      Deine persönlichen Daten und Quellcodes blieben unberührt. Das Ergebnis wurde im lokalen 4-Stufen-Gedächtnis gespeichert.
    `;

    appendAudit(
      `[WindowsBridge] DeepCleanupExecute mit Einmal-Token für Anfrage ${reqId} ausgeführt (3 veraltete Build-/Shader-Dateien entfernt, ${freedKb} KB frei).`
    );

    const nextId = Math.max(...memoriesData.map((m) => m.id), 0) + 1;
    memoriesData.push({
      id: nextId,
      createdAt: new Date().toISOString(),
      layer: "EpisodicMemory",
      category: "DeepCleanup",
      summary: `Deep Cleanup nach Nutzerfreigabe ausgeführt (3 Dateien, ${freedKb} KB bereinigt)`,
      details: `Deep Cleanup erfolgreich abgeschlossen: 3 veraltete Build- und Shader-Dateien (${freedKb} KB) wurden bereinigt. VRAM-Status: 29910 MiB frei von 32607 MiB.`,
      source: "AidoCore::approve_and_execute_deep_cleanup",
      validity: "active"
    });

    appendAudit(
      `[MemoryEngine] Episodische Erinnerung #${nextId} gespeichert: 'Deep Cleanup nach Nutzerfreigabe ausgeführt'.`
    );

    currentProposal = null;
    renderDefaultButtons();
  }, 350);
}

function rejectDeepCleanup(): void {
  if (!currentProposal) return;
  const reqId = currentProposal.requestId.slice(0, 8);
  setCharacterState("Bereit (Keine Änderung)", false);
  aidoMessage.textContent =
    "Tiefenbereinigung abgebrochen. Alle 3 Cache-Dateien (1.152 KB) wurden unverändert belassen.";
  appendAudit(
    `[PermissionGateway] Anfrage ${reqId} für Deep Cleanup vom Nutzer abgelehnt — keine Dateien gelöscht.`
  );
  currentProposal = null;
  renderDefaultButtons();
}

function showProactiveEngineView(): void {
  setCharacterState("Vordenker-Modus aktiv", false);

  const activeEpisodicCount = memoriesData.filter(
    (m) => m.layer === "EpisodicMemory" && m.validity === "active"
  ).length;

  let bannerHtml = "";
  if (currentFocusMode === "FullscreenOrGaming") {
    bannerHtml = `
      <div class="focus-guard-banner">
        <span style="font-size: 1.6rem;">🎮</span>
        <div>
          <strong>Fokus-Schutz aktiv: Vollbild- oder Gaming-Modus erkannt</strong><br/>
          AIDO bleibt im Hintergrund stumm und unterdrückt alle proaktiven Vorschläge und Popups, um deine Konzentration und Framerate nicht zu stören.
        </div>
      </div>
    `;
  } else if (currentFocusMode === "VoiceRecordingOrCall") {
    bannerHtml = `
      <div class="focus-guard-banner call">
        <span style="font-size: 1.6rem;">🎙️</span>
        <div>
          <strong>Fokus-Schutz aktiv: Audioaufnahme oder Sprachanruf aktiv</strong><br/>
          AIDO pausiert alle Benachrichtigungen und Töne, um laufende Audioaufnahmen und Gespräche nicht zu unterbrechen.
        </div>
      </div>
    `;
  }

  // Define candidate suggestions
  const allSuggestions = [
    {
      id: "dirty_git_AIDO",
      category: "DirtyGitWorkspace",
      categoryLabel: "Dirty Git Workspace",
      priority: 9,
      title: "Ungespeicherte Änderungen in AIDO",
      humanMessage: "Im Projekt 'AIDO' wurden ungespeicherte Git-Änderungen erkannt (Branch: main, Status: dirty). Möchtest du den Status prüfen oder sichern?",
      whyNow: "Ungespeicherte Änderungen im Git-Repository festgestellt.",
      primaryActionLabel: "🚀 Projekt öffnen & prüfen",
      onPrimaryAction: () => prepareProjectLaunchPreview("AIDO")
    },
    {
      id: "system_cleanup_hint",
      category: "SystemMaintenance",
      categoryLabel: "Systemwartung & Deep Cleanup",
      priority: 8,
      title: deepCleanupExecuted
        ? "Systemwartung: Bereinigt"
        : "Tiefenbereinigung 1.152 KB verfügbar",
      humanMessage: deepCleanupExecuted
        ? "Die 3 verwaisten Build- und Shader-Artefakte wurden bereits erfolgreich bereinigt."
        : "Im Deep-Cleanup-Staging liegen 3 verwaiste Build- und Shader-Artefakte (1.152 KB), die gefahrlos freigegeben werden können.",
      whyNow: deepCleanupExecuted
        ? "Wartung kürzlich abgeschlossen."
        : "Veralteter Build- und Shader-Cache belegt Speicherplatz.",
      primaryActionLabel: deepCleanupExecuted ? "🧹 Fast Cleanup prüfen" : "🧽 Deep Cleanup öffnen",
      onPrimaryAction: () => {
        if (deepCleanupExecuted) {
          startSystemHealthAndCleanupPreview();
        } else {
          startDeepCleanupPreview();
        }
      }
    },
    {
      id: "memory_compaction_hint",
      category: "MemoryConsolidation",
      categoryLabel: "Gedächtniskompaktierung",
      priority: 7,
      title: activeEpisodicCount >= 2
        ? `Episodisches Gedächtnis kompaktieren (${activeEpisodicCount} aktiv)`
        : "Gedächtnis konsolidiert",
      humanMessage: activeEpisodicCount >= 2
        ? `${activeEpisodicCount} episodische Cleanup-Aktivitäten können zu dauerhaftem semantischen Wissen (Stufe 3) konsolidiert werden.`
        : "Alle episodischen Erinnerungen wurden bereits semantisch verdichtet.",
      whyNow: activeEpisodicCount >= 2
        ? "Schwellenwert für 4-Stufen-Kompaktierung erreicht."
        : "Gedächtnisstruktur ist optimal organisiert.",
      primaryActionLabel: "🧹 Jetzt kompaktieren",
      onPrimaryAction: () => performMemoryCompaction()
    },
    {
      id: "continue_project_AIDO",
      category: "ProjectContinuation",
      categoryLabel: "Projektfortsetzung",
      priority: 6,
      title: "Weiterarbeiten an AIDO",
      humanMessage: "Möchtest du am Projekt 'AIDO' weiterarbeiten? Empfohlene nächste Aktion: npm run dev.",
      whyNow: "Zuletzt geöffnetes Projekt steht zur Fortsetzung bereit.",
      primaryActionLabel: "🚀 Entwicklungsserver starten",
      onPrimaryAction: () => prepareProjectLaunchPreview("AIDO")
    }
  ];

  // Filter out suggestions dismissed >= 2 times or muted
  const visibleSuggestions = (currentFocusMode === "NormalIdle")
    ? allSuggestions.filter((s) => {
        if (mutedCategories.has(s.category)) return false;
        const dismissCount = dismissalCounts.get(s.id) || 0;
        return dismissCount < 2;
      })
    : [];

  let html = `
    <strong>💡 AIDO Proaktive Vordenker-Engine & Fokus-Schutz</strong><br/>
    AIDO agiert als vorausschauender Partner: Erkennung ungespeicherter Git-Änderungen, anstehender System- & Tiefenbereinigungen und Gedächtniskompaktierung nach <b>MASTERPLAN Kapitel 6.8</b>.<br/>

    <!-- Focus-Guard Mode Switcher -->
    <div style="margin-top: 10px; font-size: 0.82rem; font-weight: 700; color: #c7d2fe;">🛡️ Fokus-Schutz Aktivitätsmodus:</div>
    <div class="focus-mode-bar">
      <span class="focus-pill ${currentFocusMode === "NormalIdle" ? "active" : ""}" data-focus="NormalIdle">
        🟢 Normaler Desktop
      </span>
      <span class="focus-pill gaming ${currentFocusMode === "FullscreenOrGaming" ? "active" : ""}" data-focus="FullscreenOrGaming">
        🎮 Vollbild / Gaming (Stumm)
      </span>
      <span class="focus-pill call ${currentFocusMode === "VoiceRecordingOrCall" ? "active" : ""}" data-focus="VoiceRecordingOrCall">
        🎙️ Sprachaufnahme / Call (Stumm)
      </span>
    </div>

    ${bannerHtml}
  `;

  if (currentFocusMode === "NormalIdle") {
    html += `
      <div class="intel-heading">⚡ Priorisierte proaktive Vorschläge (${visibleSuggestions.length} aktiv)</div>
      <div class="proactive-grid">
    `;

    if (visibleSuggestions.length === 0) {
      html += `
        <div style="padding: 16px; background: rgba(0,0,0,0.25); border-radius: 12px; font-size: 0.85rem; color: #94a3b8;">
          Aktuell liegen keine weiteren proaktiven Vorschläge vor (alle Vorschläge wurden bearbeitet, zurückgestellt oder deren Themen stummgeschaltet).
        </div>
      `;
    }

    for (const s of visibleSuggestions) {
      const dismissCount = dismissalCounts.get(s.id) || 0;
      const backoffBadge = dismissCount === 1
        ? '<span class="tag dirty">⏰ Zurückgestellt (1x verworfen · bei 2x wird pausiert)</span>'
        : "";

      html += `
        <div class="proactive-card priority-${s.priority}">
          <div class="proactive-header">
            <div class="proactive-title">
              <span>${s.title}</span>
              <span class="tag action">Prio ${s.priority}</span>
              <span class="tag branch">${s.categoryLabel}</span>
              ${backoffBadge}
            </div>
          </div>
          <div style="font-size: 0.86rem; color: #e0e7ff; line-height: 1.45;">${s.humanMessage}</div>
          <div class="proactive-why">Warum jetzt: ${s.whyNow}</div>
          <div class="proactive-actions">
            <button class="approve btn-primary-proactive" data-sugg-id="${s.id}">
              ${s.primaryActionLabel}
            </button>
            <button class="btn-subtle btn-dismiss-proactive" data-sugg-id="${s.id}">
              ⏳ Später erinnern
            </button>
            <button class="btn-subtle btn-mute-proactive" data-cat="${s.category}" data-cat-label="${s.categoryLabel}">
              🔕 Thema stummschalten
            </button>
          </div>
        </div>
      `;
    }

    html += `</div>`;

    if (mutedCategories.size > 0) {
      html += `
        <div style="margin-top: 14px; padding: 10px 14px; background: rgba(0,0,0,0.25); border-radius: 10px; display: flex; justify-content: space-between; align-items: center; font-size: 0.8rem; color: #94a3b8;">
          <span>Stummgeschaltete Themen: ${Array.from(mutedCategories).join(", ")}</span>
          <button class="btn-subtle" id="btn-unmute-all">🔔 Alle Themen wieder aktivieren</button>
        </div>
      `;
    }
  }

  aidoMessage.innerHTML = html;

  // Add event listeners for focus pills
  const focusPills = aidoMessage.querySelectorAll<HTMLSpanElement>(".focus-pill");
  focusPills.forEach((pill) => {
    pill.addEventListener("click", () => {
      const mode = pill.dataset.focus as FocusMode;
      if (mode && mode !== currentFocusMode) {
        currentFocusMode = mode;
        if (mode === "NormalIdle") {
          appendAudit("[FocusGuard] Modus gewechselt zu: Normaler Desktop (Proaktive Vorschläge aktiv).");
        } else if (mode === "FullscreenOrGaming") {
          appendAudit("[FocusGuard] Fokus-Schutz aktiviert: Vollbild / Gaming erkannt — alle Vorschläge stummgeschaltet.");
        } else if (mode === "VoiceRecordingOrCall") {
          appendAudit("[FocusGuard] Fokus-Schutz aktiviert: Audio / Call erkannt — Benachrichtigungen unterdrückt.");
        }
        showProactiveEngineView();
      }
    });
  });

  // Add event listeners for primary proactive actions
  const primaryButtons = aidoMessage.querySelectorAll<HTMLButtonElement>(".btn-primary-proactive");
  primaryButtons.forEach((btn) => {
    btn.addEventListener("click", () => {
      const id = btn.dataset.suggId;
      const sugg = allSuggestions.find((s) => s.id === id);
      if (sugg) {
        sugg.onPrimaryAction();
      }
    });
  });

  // Add event listeners for dismissal backoff
  const dismissButtons = aidoMessage.querySelectorAll<HTMLButtonElement>(".btn-dismiss-proactive");
  dismissButtons.forEach((btn) => {
    btn.addEventListener("click", () => {
      const id = btn.dataset.suggId;
      if (!id) return;
      const current = dismissalCounts.get(id) || 0;
      const updated = current + 1;
      dismissalCounts.set(id, updated);
      const sugg = allSuggestions.find((s) => s.id === id);
      const title = sugg ? sugg.title : id;
      if (updated >= 2) {
        appendAudit(`[ProactiveEngine] Vorschlag '${title}' 2x verworfen — 2x Backoff aktiv (Vorschlag für diese Sitzung pausiert).`);
      } else {
        appendAudit(`[ProactiveEngine] Vorschlag '${title}' zurückgestellt (${updated}/2 Ablehnungen).`);
      }
      showProactiveEngineView();
    });
  });

  // Add event listeners for muting category
  const muteButtons = aidoMessage.querySelectorAll<HTMLButtonElement>(".btn-mute-proactive");
  muteButtons.forEach((btn) => {
    btn.addEventListener("click", () => {
      const cat = btn.dataset.cat;
      const label = btn.dataset.catLabel || cat;
      if (!cat) return;
      mutedCategories.add(cat);
      appendAudit(`[ProactiveEngine] Thema '${label}' stummgeschaltet.`);
      showProactiveEngineView();
    });
  });

  // Add event listener for unmute all
  const unmuteBtn = aidoMessage.querySelector<HTMLButtonElement>("#btn-unmute-all");
  if (unmuteBtn) {
    unmuteBtn.addEventListener("click", () => {
      mutedCategories.clear();
      appendAudit("[ProactiveEngine] Alle Themen wurden wieder aktiviert.");
      showProactiveEngineView();
    });
  }

  appendAudit(
    `[ProactiveEngine] Vordenker-Ansicht geladen (Aktivitätszustand: ${currentFocusMode}, ${visibleSuggestions.length} aktive Vorschläge).`
  );

  // Render contextual buttons
  actionButtons.innerHTML = "";
  const btnDeepCleanupQuick = document.createElement("button");
  btnDeepCleanupQuick.textContent = "🧽 Deep Cleanup prüfen";
  btnDeepCleanupQuick.onclick = () => startDeepCleanupPreview();

  const btnMemoryQuick = document.createElement("button");
  btnMemoryQuick.textContent = "🧠 Memory & Wissen";
  btnMemoryQuick.onclick = () => showMemorySummary();

  const btnBack = document.createElement("button");
  btnBack.textContent = "🔙 Zurück zur Übersicht";
  btnBack.onclick = () => {
    setCharacterState("Bereit (Lokal)", false);
    aidoMessage.innerHTML =
      "Hallo! Ich bin <b>AIDO (Agentic Intelligent Developing Operator)</b>. Ich laufe lokal auf deiner RTX 5090 im Modus <b>Private Offline</b> — keine Daten verlassen deinen PC. Möchtest du einen System-Check durchführen, deine erkannte App- & Projektumgebung prüfen oder temporäre Caches bereinigen?";
    renderDefaultButtons();
  };

  actionButtons.append(btnDeepCleanupQuick, btnMemoryQuick, btnBack);
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
  } else if (action === "proactive") {
    showProactiveEngineView();
  } else if (action === "deep-cleanup") {
    startDeepCleanupPreview();
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
