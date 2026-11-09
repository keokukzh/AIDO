import { Mono3DViewer } from "./mono3d";
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

interface SandboxCapability {
  capabilityId: string;
  titleDe: string;
  risk: "Low" | "Medium" | "High";
  status: "Verified" | "Restricted";
  requiresApproval: boolean;
}

interface SandboxAppProfile {
  appId: "obs-studio" | "blender";
  displayName: string;
  publisher: string;
  version: string;
  installPath: string;
  preferredControl: "GeneratedAdapter" | "UiAutomation";
  adapterOrigin: "AutoGenerated";
  authRequired: string;
  sandboxIsolated: boolean;
  harmlessProbePassed: boolean;
  verifiedCapabilitiesCount: number;
  restrictedCapabilitiesCount: number;
  capabilities: SandboxCapability[];
  humanSummary: string;
}

const sandboxAppsData: Record<"obs-studio" | "blender", SandboxAppProfile> = {
  "obs-studio": {
    appId: "obs-studio",
    displayName: "OBS Studio",
    publisher: "OBS Project",
    version: "1.0.0 (Synthetisiert)",
    installPath: "C:\\Program Files\\obs-studio\\bin\\64bit\\obs64.exe",
    preferredControl: "GeneratedAdapter",
    adapterOrigin: "AutoGenerated",
    authRequired: "Keine (Isolierte Sandbox)",
    sandboxIsolated: true,
    harmlessProbePassed: true,
    verifiedCapabilitiesCount: 2,
    restrictedCapabilitiesCount: 1,
    capabilities: [
      {
        capabilityId: "probe_version_status",
        titleDe: "OBS Studio Status & Version abfragen (Harmloser Sandbox-Check)",
        risk: "Low",
        status: "Verified",
        requiresApproval: false
      },
      {
        capabilityId: "open_app_window",
        titleDe: "OBS Studio im Fokus starten",
        risk: "Low",
        status: "Verified",
        requiresApproval: false
      },
      {
        capabilityId: "automate_workflow_export",
        titleDe: "OBS Studio Arbeitsablauf / Export automatisieren",
        risk: "Medium",
        status: "Restricted",
        requiresApproval: true
      }
    ],
    humanSummary: "Automatisch synthetisierter Adapter für OBS Studio. 2 verifizierte Basis-Aktionen, 1 freigabepflichtige Workflow-Aktion."
  },
  "blender": {
    appId: "blender",
    displayName: "Blender 3D",
    publisher: "Blender Foundation",
    version: "1.0.0 (Synthetisiert)",
    installPath: "C:\\Program Files\\Blender Foundation\\Blender 4.2\\blender.exe",
    preferredControl: "GeneratedAdapter",
    adapterOrigin: "AutoGenerated",
    authRequired: "Keine (Isolierte Sandbox)",
    sandboxIsolated: true,
    harmlessProbePassed: true,
    verifiedCapabilitiesCount: 2,
    restrictedCapabilitiesCount: 1,
    capabilities: [
      {
        capabilityId: "probe_version_status",
        titleDe: "Blender 3D Status & Version abfragen (Harmloser Sandbox-Check)",
        risk: "Low",
        status: "Verified",
        requiresApproval: false
      },
      {
        capabilityId: "open_app_window",
        titleDe: "Blender 3D im Fokus starten",
        risk: "Low",
        status: "Verified",
        requiresApproval: false
      },
      {
        capabilityId: "automate_workflow_export",
        titleDe: "Blender 3D Render-Job automatisieren",
        risk: "Medium",
        status: "Restricted",
        requiresApproval: true
      }
    ],
    humanSummary: "Automatisch synthetisierter Adapter für Blender 3D. 2 verifizierte Basis-Aktionen, 1 freigabepflichtige Workflow-Aktion."
  }
};

const releaseChecklistItems: string[] = [
  "Phase 1: Lokale KI-Runtime & Hardware-Erkennung (VRAM, RAM, GPU-Tiers) verifiziert",
  "Phase 2: Lokale Modell-Erkennung & Wiederverwendung ohne Redownload verifiziert",
  "Phase 3: Privater Offline-Modus & PermissionGateway (Zero-Telemetry, Loopback 127.0.0.1) verifiziert",
  "Phase 4: Windows-Health, Schnelle & Tiefen-Bereinigung mit Einmal-Freigabetoken verifiziert",
  "Phase 5: Projekt- und App-Erkennung mit risikobasierter Capability-Trennung verifiziert",
  "Phase 6: Multi-Modell Supervisor (Llama.cpp / Ollama) mit VRAM-Standby verifiziert",
  "Phase 7: Isolierte Adapter-Prüfbank für unbekannte Programme in Sandbox verifiziert",
  "Phase 8: Saubere Deinstallation mit 100 % Schutz vorhandener Modelle verifiziert"
];

let selectedSandboxAppId: "obs-studio" | "blender" = "obs-studio";
let uninstallDryRunVerified: boolean = false;

const avatarOrb = document.getElementById("avatar-orb") as HTMLDivElement | null;
const charState = document.getElementById("char-state") as HTMLSpanElement;
const aidoMessage = document.getElementById("aido-message") as HTMLDivElement;
const actionButtons = document.getElementById("action-buttons") as HTMLDivElement;
const auditList = document.getElementById("audit-list") as HTMLUListElement;
const contextMenu = document.getElementById("context-menu") as HTMLDivElement;
const privacyBadge = document.getElementById("privacy-badge") as HTMLSpanElement | null;
const btnStaticRemote = document.getElementById("btn-remote") as HTMLButtonElement | null;
btnStaticRemote?.addEventListener("click", () => showAgentRemoteView());

const monoCanvas = document.getElementById("aido-mono-3d-canvas") as HTMLCanvasElement | null;
const characterViewport = document.getElementById("character-viewport-container") as HTMLDivElement | null;
const mode3dView = document.getElementById("mode-3d-view") as HTMLDivElement | null;
const modeSvgView = document.getElementById("mode-svg-view") as HTMLDivElement | null;
const mode2dView = document.getElementById("mode-2d-view") as HTMLDivElement | null;
const btnMode3d = document.getElementById("btn-mode-3d") as HTMLButtonElement | null;
const btnModeSvg = document.getElementById("btn-mode-svg") as HTMLButtonElement | null;
const btnMode2d = document.getElementById("btn-mode-2d") as HTMLButtonElement | null;

let monoViewer: Mono3DViewer | null = null;
if (monoCanvas) {
  monoViewer = new Mono3DViewer(monoCanvas, "/assets/AIDO-MONO.glb");
  monoViewer.load().then(() => {
    appendAudit("[AIDO-MONO 3D] WebGL-Modell '/assets/AIDO-MONO.glb' erfolgreich geladen (3 artikulierte Nodes, 3.272 Vertices).");
  }).catch((err) => {
    console.error("Mono3DViewer load error:", err);
  });
}

function switchViewMode(mode: "3d" | "svg" | "2d"): void {
  if (mode3dView) mode3dView.classList.toggle("active", mode === "3d");
  if (modeSvgView) modeSvgView.classList.toggle("active", mode === "svg");
  if (mode2dView) mode2dView.classList.toggle("active", mode === "2d");

  if (btnMode3d) btnMode3d.classList.toggle("active", mode === "3d");
  if (btnModeSvg) btnModeSvg.classList.toggle("active", mode === "svg");
  if (btnMode2d) btnMode2d.classList.toggle("active", mode === "2d");

  appendAudit(
    `[AIDO-MONO] Ansichtsmodus: ${
      mode === "3d"
        ? "🧊 3D Live WebGL (AIDO-MONO.glb)"
        : mode === "svg"
        ? "✨ Morph-Animation SVG"
        : "🎯 2D Charakter-Logo"
    }`
  );
}

btnMode3d?.addEventListener("click", () => switchViewMode("3d"));
btnModeSvg?.addEventListener("click", () => switchViewMode("svg"));
btnMode2d?.addEventListener("click", () => switchViewMode("2d"));

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
    avatarOrb?.classList.add("working");
    monoViewer?.setState("thinking");
  } else {
    avatarOrb?.classList.remove("working");
    if (stateLabel.includes("Höre zu") || stateLabel.includes("Push-to-Talk") || stateLabel.includes("Aufnahme")) {
      monoViewer?.setState("listening");
    } else if (stateLabel.includes("Schlafmodus")) {
      monoViewer?.setState("idle");
      monoViewer?.toggleAutoRotate(false);
    } else if (stateLabel.includes("spricht") || stateLabel.includes("Begrüßung") || stateLabel.includes("erwacht")) {
      monoViewer?.setState("speaking");
    } else {
      monoViewer?.setState("idle");
    }
  }
}

function renderDefaultButtons(): void {
  actionButtons.innerHTML = "";
  const btnBrandStudio = document.createElement("button");
  btnBrandStudio.textContent = "🎨 AIDO-MONO 3D & Brand Studio";
  btnBrandStudio.dataset.action = "brand-studio";
  btnBrandStudio.style.background = "rgba(255, 255, 255, 0.12)";
  btnBrandStudio.style.borderColor = "rgba(255, 255, 255, 0.3)";
  btnBrandStudio.style.color = "#f8fafc";
  btnBrandStudio.onclick = () => showBrandStudioView();

  const btnRemote = document.createElement("button");
  btnRemote.textContent = "📱 Agent Remote (QR & Handy-Cockpit)";
  btnRemote.dataset.action = "remote";
  btnRemote.style.background = "rgba(255, 255, 255, 0.09)";
  btnRemote.style.borderColor = "rgba(255, 255, 255, 0.28)";
  btnRemote.style.color = "#ffffff";
  btnRemote.onclick = () => showAgentRemoteView();

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

  const btnReleaseCheck = document.createElement("button");
  btnReleaseCheck.textContent = "🧩 Adapter-Sandbox & Release-Check";
  btnReleaseCheck.dataset.action = "release-check";
  btnReleaseCheck.onclick = () => showReleaseCheckView();

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
    btnBrandStudio,
    btnRemote,
    btnCheck,
    btnProactive,
    btnDeepCleanup,
    btnReleaseCheck,
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


function showReleaseCheckView(appId?: "obs-studio" | "blender"): void {
  if (appId) {
    selectedSandboxAppId = appId;
  }

  const app = sandboxAppsData[selectedSandboxAppId];
  setCharacterState("Adapter-Sandbox & Release-Check", false);

  let html = `
    <strong>🧩 Adapter-Sandbox (Phase 7) & Release-Check / Saubere Deinstallation (Phase 8)</strong><br/>
    Prüfbank für unbekannte Windows-Anwendungen in isolierter Sandbox sowie 8-Phasen Release-Verifikation mit Zero-Destruction Deinstallations-Garantie.<br/>

    <!-- TEIL 1: PHASE 7 ADAPTER SANDBOX -->
    <div class="intel-section">
      <div class="intel-heading">🧪 Teil 1: Isolierte Adapter-Prüfbank für unbekannte Apps (Phase 7)</div>

      <div style="display: flex; gap: 8px; margin: 8px 0;">
        <button class="filter-badge ${selectedSandboxAppId === "obs-studio" ? "active" : ""}" id="tab-obs" style="cursor: pointer;">
          🎥 OBS Studio prüfen
        </button>
        <button class="filter-badge ${selectedSandboxAppId === "blender" ? "active" : ""}" id="tab-blender" style="cursor: pointer;">
          🎨 Blender 3D prüfen
        </button>
      </div>

      <div class="sandbox-card">
        <div class="sandbox-header">
          <div>
            <span style="font-weight: 700; font-size: 1.05rem; color: #f3f4f6;">${app.displayName}</span>
            <span style="font-size: 0.78rem; color: #94a3b8; margin-left: 8px;">(${app.publisher} · ${app.version})</span>
          </div>
          <div style="display: flex; gap: 6px; flex-wrap: wrap;">
            <span class="tag branch">🔒 Sandbox isoliert</span>
            <span class="tag branch">✓ Harmloser Probe-Check: Bestanden</span>
            <span class="tag action">Steuerung: ${app.preferredControl}</span>
            <span class="tag">Herkunft: ${app.adapterOrigin}</span>
          </div>
        </div>

        <div class="app-meta">Installationspfad: <code>${app.installPath}</code> · Authentifizierung: <code>${app.authRequired}</code></div>
        <div style="font-size: 0.8rem; color: #cbd5e1; margin-top: 2px;">
          ${app.humanSummary}
        </div>

        <div class="intel-heading" style="margin-top: 10px; font-size: 0.82rem;">
          Synthetisierte Fähigkeiten (${app.verifiedCapabilitiesCount} Verifiziert / Harmlos · ${app.restrictedCapabilitiesCount} Approval-Gated):
        </div>
        <div style="display: flex; flex-direction: column; gap: 6px; margin-top: 4px;">
  `;

  for (const cap of app.capabilities) {
    const isVerified = cap.status === "Verified";
    const statusClass = isVerified ? "low" : "medium";
    const statusIcon = isVerified ? "✓" : "⚠️";
    const statusText = isVerified ? "Verifiziert · Ohne Freigabe ausführbar" : "Beschränkt · Erfordert Einmal-Freigabe";

    html += `
          <div style="background: rgba(0,0,0,0.3); border: 1px solid rgba(255,255,255,0.08); border-radius: 8px; padding: 8px 12px; display: flex; justify-content: space-between; align-items: center;">
            <div>
              <div style="font-size: 0.84rem; font-weight: 600; color: #f3f4f6;">
                <code>${cap.capabilityId}</code>: ${cap.titleDe}
              </div>
              <div style="font-size: 0.74rem; color: #94a3b8;">
                Risikostufe: ${cap.risk} · Freigabepflichtig: ${cap.requiresApproval ? "Ja (PermissionGateway geschützt)" : "Nein (harmlos)"}
              </div>
            </div>
            <div>
              <span class="cap-badge ${statusClass}">
                ${statusIcon} ${statusText}
              </span>
            </div>
          </div>
    `;
  }

  html += `
        </div>
      </div>
    </div>

    <!-- TEIL 2: PHASE 8 RELEASE READINESS & UNINSTALLER -->
    <div class="intel-section" style="margin-top: 14px;">
      <div class="intel-heading">🛡️ Teil 2: 8-Phasen Release-Checkliste & Zero-Destruction Clean Uninstaller (Phase 8)</div>

      <div class="release-checklist-grid">
  `;

  for (const item of releaseChecklistItems) {
    html += `
        <div class="release-item passed">
          <span class="release-check-icon">✅</span>
          <span>${item}</span>
        </div>
    `;
  }

  html += `
      </div>

      <div class="intel-heading" style="margin-top: 6px; font-size: 0.85rem;">
        Sicherheitsbereich der Deinstallation (Gegenüberstellung AIDO-Dateien vs. Externe Modelle):
      </div>

      <div class="uninstall-comparison-grid">
        <div class="uninstall-box danger-scope">
          <div class="uninstall-box-title">
            <span>🗑️ Zur Bereinigung vorgemerkt (AIDO-only Caches & DB)</span>
          </div>
          <div style="font-size: 0.76rem; color: #cbd5e1;">
            Ausschließlich isolierte AIDO-Laufzeitdateien werden entfernt:
          </div>
          <div class="uninstall-path-list">
            <div class="uninstall-path-item">%LOCALAPPDATA%\\AIDO\\cache\\fast_cleanup_staging</div>
            <div class="uninstall-path-item">%LOCALAPPDATA%\\AIDO\\data\\aido_memory.db</div>
          </div>
        </div>

        <div class="uninstall-box safe-scope">
          <div class="uninstall-box-title">
            <span>🛡️ 100 % Geschützt & Unberührt (Lokale KI-Modelle & Quellcodes)</span>
          </div>
          <div style="font-size: 0.76rem; color: #cbd5e1;">
            Bestehende Modelle und Quellcode-Repositories bleiben unangetastet:
          </div>
          <div class="uninstall-path-list">
            <div class="uninstall-path-item">D:\\AI_Models\\llm\\Qwen3.8-27B-UD-Q4_K_M.gguf (15.7 GB)</div>
            <div class="uninstall-path-item">D:\\AI_Models\\llm\\assets\\mmproj-Qwen3.8-27B-BF16.gguf (888 MB)</div>
            <div class="uninstall-path-item">D:\\AI_Models\\llm\\assets\\Qwen3.8-27B-DFlash2-Q4_K_M.gguf (1.09 GB)</div>
            <div class="uninstall-path-item">D:\\AI_Models\\unsloth\\whisper.cpp\\models\\ggml-small.bin (465 MB)</div>
            <div class="uninstall-path-item">C:\\Users\\keoku\\Desktop\\AIDO (Entwickler-Workspaces)</div>
          </div>
        </div>
      </div>
  `;

  if (uninstallDryRunVerified) {
    html += `
      <div class="zero-destruct-banner">
        <span style="font-size: 1.3rem;">🛡️</span>
        <div>
          <strong>Zero-Destruction Deinstallation erfolgreich verifiziert!</strong><br/>
          Einmal-Sicherheits-Token validiert. Alle 4/4 externen Modelle auf <code>D:\\AI_Models</code> und sämtliche Entwickler-Workspaces bleiben zu 100 % erhalten. Es werden ausschließlich die 2 AIDO-eigenen Cache/DB-Pfade bereinigt.
        </div>
      </div>
    `;
  }

  html += `</div>`;

  aidoMessage.innerHTML = html;

  // Tab listeners
  const tabObs = document.getElementById("tab-obs");
  if (tabObs) {
    tabObs.onclick = () => {
      runSandboxProbe("obs-studio");
    };
  }
  const tabBlender = document.getElementById("tab-blender");
  if (tabBlender) {
    tabBlender.onclick = () => {
      runSandboxProbe("blender");
    };
  }

  // Action buttons
  actionButtons.innerHTML = "";

  const btnProbeObs = document.createElement("button");
  btnProbeObs.textContent = "🧪 OBS Studio in Sandbox prüfen";
  btnProbeObs.onclick = () => runSandboxProbe("obs-studio");

  const btnProbeBlender = document.createElement("button");
  btnProbeBlender.textContent = "🧪 Blender 3D in Sandbox prüfen";
  btnProbeBlender.onclick = () => runSandboxProbe("blender");

  const btnDryRun = document.createElement("button");
  btnDryRun.className = "approve";
  btnDryRun.textContent = "🛡️ Trockenlauf: Saubere Deinstallation prüfen (Zero-Destruction)";
  btnDryRun.onclick = () => runCleanUninstallDryRun();

  const btnBack = document.createElement("button");
  btnBack.textContent = "🔙 Zurück zur Übersicht";
  btnBack.onclick = () => {
    setCharacterState("Bereit (Lokal)", false);
    aidoMessage.innerHTML =
      "Hallo! Ich bin <b>AIDO (Agentic Intelligent Developing Operator)</b>. Ich laufe lokal auf deiner RTX 5090 im Modus <b>Private Offline</b> — keine Daten verlassen deinen PC. Möchtest du einen System-Check durchführen, deine erkannte App- & Projektumgebung prüfen oder temporäre Caches bereinigen?";
    renderDefaultButtons();
  };

  actionButtons.append(btnProbeObs, btnProbeBlender, btnDryRun, btnBack);

  appendAudit(
    `[ReleaseCheck] Ansicht geöffnet: App '${app.displayName}' in Sandbox (${app.verifiedCapabilitiesCount} verifiziert, ${app.restrictedCapabilitiesCount} freigabepflichtig), Release 0.1.0-mvp bereit, Zero-Destruction aktiv.`
  );
}

function runSandboxProbe(appId: "obs-studio" | "blender"): void {
  const app = sandboxAppsData[appId];
  selectedSandboxAppId = appId;
  setCharacterState(`Prüfe ${app.displayName} in Sandbox...`, true);

  setTimeout(() => {
    setCharacterState("Adapter synthetisiert & geprüft", false);
    appendAudit(
      `[AdapterSandbox] Unbekannte App '${app.displayName}' in isolierter Sandbox geprüft: Harmloser Probe-Check bestanden, ${app.verifiedCapabilitiesCount} verifizierte Basis-Aktionen, ${app.restrictedCapabilitiesCount} freigabepflichtige Workflow-Aktion.`
    );
    showReleaseCheckView(appId);
  }, 200);
}

function runCleanUninstallDryRun(): void {
  setCharacterState("Deinstallations-Trockenlauf...", true);

  setTimeout(() => {
    uninstallDryRunVerified = true;
    setCharacterState("Zero-Destruction verifiziert!", false);

    appendAudit(
      "[PermissionGateway] Freigabe-Anfrage 'ExecuteCleanUninstall' im Trockenlauf geprüft: Einmal-Token validiert."
    );
    appendAudit(
      "[SmartInstaller] Saubere Deinstallation (Trockenlauf): 2 AIDO-Pfade markiert, 4 externe Modelle auf D:\\AI_Models zu 100 % geschützt."
    );

    showReleaseCheckView();
  }, 250);
}



// --- AIDO-MONO 3D & Brand Identity View ---
function showBrandStudioView(): void {
  const svgWrapper = document.querySelector("#mode-svg-view .svg-morph-wrapper");
  const svgMarkup = svgWrapper ? svgWrapper.innerHTML : "";
  setCharacterState("AIDO-MONO 3D & Brand Studio", false);
  monoViewer?.setState("idle");

  const stats = monoViewer?.getNodeStats() || [
    { name: "MONO · O head", nodeIndex: 0, vertices: 1832, indices: 9348 },
    { name: "MONO · D chassis and A legs", nodeIndex: 1, vertices: 1372, indices: 6912 },
    { name: "MONO · I core", nodeIndex: 2, vertices: 68, indices: 276 },
  ];
  const totalVerts = stats.reduce((acc, s) => acc + s.vertices, 0);
  const totalIndices = stats.reduce((acc, s) => acc + s.indices, 0);

  let html = `
    <strong>🎨 AIDO-MONO Brand Identity & 3D GLB Character Studio</strong><br/>
    Offizielles AIDO Designsystem, 3D-Modellarchitektur (` + "<code>/assets/AIDO-MONO.glb</code>" + `) und stufenlose Marken-Transformation.<br/>

    <!-- SEKTION 1: OFFIZIELLE MARKEN-GRAFIKEN -->
    <div class="intel-section">
      <div class="intel-heading">🎯 1. Offizielle AIDO-Markengrafiken (Dark/Light Vektor-Assets)</div>
      <div class="brand-grid-3col">
        <div class="brand-card-item">
          <div class="brand-card-img-wrap">
            <img src="/assets/AIDO-Wortmarke.png" alt="AIDO Wortmarke" />
          </div>
          <div class="brand-card-title">AIDO Wortmarke</div>
          <div class="brand-card-desc">
            Geometrische Wortmarke mit typografischem Schnitt für Header, App-Icons und Badges. (1.774 × 887 px)
          </div>
          <span class="tag branch">✓ <code>/assets/AIDO-Wortmarke.png</code></span>
        </div>

        <div class="brand-card-item">
          <div class="brand-card-img-wrap">
            <img src="/assets/AIDO-MONO-Charakter.png" alt="AIDO-MONO Charakter" />
          </div>
          <div class="brand-card-title">AIDO-MONO Charakter</div>
          <div class="brand-card-desc">
            Offizielles 2D-Charakter-Icon und Desktop-Favicon. Geformt aus den 4 Buchstaben A-I-D-O. (1.254 × 1.254 px)
          </div>
          <span class="tag branch">✓ <code>/assets/AIDO-MONO-Charakter.png</code></span>
        </div>

        <div class="brand-card-item">
          <div class="brand-card-img-wrap">
            <img src="/assets/AIDO-Ausgeschriebener-Name.png" alt="AIDO Ausgeschriebener Name" />
          </div>
          <div class="brand-card-title">Ausgeschriebener Markenname</div>
          <div class="brand-card-desc">
            Vollständiger Untertitel „AGENTIC INTELLIGENT DEVELOPING OPERATOR“ in architektonischer Typografie. (2.172 × 724 px)
          </div>
          <span class="tag branch">✓ <code>/assets/AIDO-Ausgeschriebener-Name.png</code></span>
        </div>
      </div>
    </div>

    <!-- SEKTION 2: 3D GLB CHARACTER ARCHITEKTUR & NODE INSPECTOR -->
    <div class="intel-section" style="margin-top: 14px;">
      <div class="intel-heading">🧊 2. 3D GLB Character Architecture (<code>/assets/AIDO-MONO.glb</code>)</div>
      <div style="font-size: 0.82rem; color: #cbd5e1; line-height: 1.5; margin-bottom: 8px;">
        Das offizielle AIDO-MONO 3D-Modell wurde in Blender modelliert und als offline-fähiges <b>glTF 2.0 Binary (GLB)</b> exportiert. Es nutzt <b>3 unabhängig artikulierte Nodes</b> und das PBR-Material <code>MONO · satin graphite</code> (Metallic: 0.08, Roughness: 0.42).
      </div>

      <table class="node-inspector-table">
        <thead>
          <tr>
            <th>Node-Index</th>
            <th>Node-Name im GLB</th>
            <th>Vertices</th>
            <th>Indices</th>
            <th>Artikulation & Shader-Verhalten</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td><code>Node 0</code></td>
            <td><b>MONO · O head</b></td>
            <td>1.832</td>
            <td>9.348</td>
            <td>Sanfte Schwebung (Hover), neigt sich aufmerksam bei <i>Listening</i>, nickt bei <i>Speaking</i>, scannt bei <i>Thinking</i>.</td>
          </tr>
          <tr>
            <td><code>Node 1</code></td>
            <td><b>MONO · D chassis and A legs</b></td>
            <td>1.372</td>
            <td>6.912</td>
            <td>Satin Graphite Chassis & Beine mit 3-Punkt Studio-Beleuchtung (Key, Fill, Rim-Light).</td>
          </tr>
          <tr>
            <td><code>Node 2</code></td>
            <td><b>MONO · I core</b></td>
            <td>68</td>
            <td>276</td>
            <td>State-reaktiver Pulsier-Kern mit dynamischer Emissive-Farbe (Smaragdgrün <code>#10B981</code>, Cyan <code>#38BDF8</code>).</td>
          </tr>
        </tbody>
        <tfoot>
          <tr style="font-weight: 700; color: #f8fafc; border-top: 2px solid rgba(255,255,255,0.15);">
            <td colspan="2">Gesamt-Geometrie (AIDO-MONO.glb)</td>
            <td>${totalVerts.toLocaleString("de-DE")} Vertices</td>
            <td>${totalIndices.toLocaleString("de-DE")} Indices</td>
            <td>PBR Metallic-Roughness (Offline WebGL)</td>
          </tr>
        </tfoot>
      </table>

      <!-- Interaktive Animations-Tests -->
      <div style="margin-top: 10px;">
        <div style="font-size: 0.82rem; font-weight: 700; color: #f8fafc; margin-bottom: 6px;">
          🎮 Interaktiver 3D-Node Animationstest (Status-Reaktion):
        </div>
        <div class="state-btn-group">
          <button class="btn-state-chip active" id="btn-state-idle">💤 Idle (Schwebung & Ruhepuls)</button>
          <button class="btn-state-chip emerald" id="btn-state-listen">👂 Listening (Kopf-Neigung & Smaragd #10B981)</button>
          <button class="btn-state-chip cyan" id="btn-state-think">🧠 Thinking (Orbit-Scan & Cyan #38BDF8)</button>
          <button class="btn-state-chip" id="btn-state-speak">🗣️ Speaking (Rhythmisches Nicken & Sprachpuls)</button>
        </div>
      </div>

      <!-- Kamera-Steuerung -->
      <div style="margin-top: 6px;">
        <div style="font-size: 0.82rem; font-weight: 700; color: #f8fafc; margin-bottom: 6px;">
          🎥 3D-Kamera & Ansichts-Steuerung:
        </div>
        <div class="state-btn-group">
          <button class="btn-state-chip" id="btn-cam-reset">🔄 Kamera zentrieren (Reset)</button>
          <button class="btn-state-chip" id="btn-cam-rotate">🌐 360° Auto-Rotation umschalten</button>
          <button class="btn-state-chip" id="btn-cam-zoomin">🔍 Zoom + (Näher)</button>
          <button class="btn-state-chip" id="btn-cam-zoomout">🔎 Zoom - (Weiter)</button>
        </div>
      </div>
    </div>

    <!-- SEKTION 3: INFINITE BRAND TRANSFORMATION SVG -->
    <div class="intel-section" style="margin-top: 14px;">
      <div class="intel-heading">✨ 3. Infinite Brand Transformation (Morph-Animation)</div>
      <div style="font-size: 0.82rem; color: #cbd5e1; line-height: 1.5; margin-bottom: 8px;">
        Der voll ausgeschriebene Leitspruch <i>„AGENTIC INTELLIGENT DEVELOP OPERATOR“</i> morpht in einer nahtlosen 10,4-Sekunden-CSS-Schleife in das Akronym <b>„AIDO“</b> und schließlich in den Charakter <b>„MONO“</b>.
      </div>
      <div style="background: rgba(10, 13, 18, 0.95); border: 1px solid rgba(255,255,255,0.12); border-radius: 12px; padding: 18px; display: flex; justify-content: center; align-items: center; min-height: 140px;">
        <div class="svg-morph-wrapper" style="max-width: 580px; min-height: 120px;">
          ${svgMarkup}
        </div>
      </div>
    </div>
  `;

  aidoMessage.innerHTML = html;

  // Bind animation test buttons
  const btnIdle = document.getElementById("btn-state-idle");
  const btnListen = document.getElementById("btn-state-listen");
  const btnThink = document.getElementById("btn-state-think");
  const btnSpeak = document.getElementById("btn-state-speak");

  const clearActiveChips = () => {
    [btnIdle, btnListen, btnThink, btnSpeak].forEach((b) => b?.classList.remove("active"));
  };

  btnIdle?.addEventListener("click", () => {
    clearActiveChips();
    btnIdle.classList.add("active");
    setCharacterState("Bereit (Lokal)", false);
    monoViewer?.setState("idle");
    appendAudit("[BrandStudio] Animationsstatus 'idle' aktiviert: Sanfte Schwebung von O head und Ruhepuls.");
  });

  btnListen?.addEventListener("click", () => {
    clearActiveChips();
    btnListen.classList.add("active");
    setCharacterState("Höre zu (Whisper Push-to-Talk)...", false);
    monoViewer?.setState("listening");
    appendAudit("[BrandStudio] Animationsstatus 'listening' aktiviert: O head neigt sich, I core leuchtet Smaragdgrün (#10B981).");
  });

  btnThink?.addEventListener("click", () => {
    clearActiveChips();
    btnThink.classList.add("active");
    setCharacterState("Analysiere & Überlege...", true);
    monoViewer?.setState("thinking");
    appendAudit("[BrandStudio] Animationsstatus 'thinking' aktiviert: O head scannt von links nach rechts, I core pulsiert Cyan (#38BDF8).");
  });

  btnSpeak?.addEventListener("click", () => {
    clearActiveChips();
    btnSpeak.classList.add("active");
    setCharacterState("AIDO spricht...", false);
    monoViewer?.setState("speaking");
    appendAudit("[BrandStudio] Animationsstatus 'speaking' aktiviert: O head nickt rhythmisch zur Sprachausgabe.");
  });

  // Bind camera buttons
  document.getElementById("btn-cam-reset")?.addEventListener("click", () => {
    monoViewer?.resetCamera();
    appendAudit("[BrandStudio] 3D-Kamera auf Standardperspektive zurückgesetzt.");
  });
  document.getElementById("btn-cam-rotate")?.addEventListener("click", () => {
    const isNow = monoViewer?.toggleAutoRotate();
    appendAudit(`[BrandStudio] 360° Auto-Rotation ${isNow ? "aktiviert" : "pausiert"}.`);
  });
  document.getElementById("btn-cam-zoomin")?.addEventListener("click", () => {
    monoViewer?.setZoom(1.35);
  });
  document.getElementById("btn-cam-zoomout")?.addEventListener("click", () => {
    monoViewer?.setZoom(0.85);
  });

  // Action buttons
  actionButtons.innerHTML = "";

  const btnView3D = document.createElement("button");
  btnView3D.textContent = "🧊 Zu 3D GLB Live wechseln";
  btnView3D.onclick = () => switchViewMode("3d");

  const btnViewSvg = document.createElement("button");
  btnViewSvg.textContent = "✨ Zu Morph-Animation wechseln";
  btnViewSvg.onclick = () => switchViewMode("svg");

  const btnView2D = document.createElement("button");
  btnView2D.textContent = "🎯 Zu 2D Logo wechseln";
  btnView2D.onclick = () => switchViewMode("2d");

  const btnBack = document.createElement("button");
  btnBack.textContent = "🔙 Zurück zur Übersicht";
  btnBack.onclick = () => {
    setCharacterState("Bereit (Lokal)", false);
    aidoMessage.innerHTML =
      "Hallo! Ich bin <b>AIDO (Agentic Intelligent Developing Operator)</b>. Ich laufe lokal auf deiner RTX 5090 im Modus <b>Private Offline</b> — keine Daten verlassen deinen PC. Möchtest du einen System-Check durchführen, deine erkannte App- & Projektumgebung prüfen oder temporäre Caches bereinigen?";
    renderDefaultButtons();
  };

  actionButtons.append(btnView3D, btnViewSvg, btnView2D, btnBack);
  appendAudit("[BrandStudio] AIDO-MONO Brand Identity & 3D GLB Character Studio geöffnet.");
}



// ============================================================================
// Phase 9: AIDO Agent Remote — Minimalist Monochrome UI Implementation
// Strictly Black/White (#050507 / #ffffff) or White/Black (#fafafa / #09090b)
// ============================================================================

interface RemoteProposal {
  id: string;
  title: string;
  desc: string;
  bytes: number;
  actionKind: string;
  approved: boolean;
}

let remoteMonochromeTheme: "bw" | "wb" = "bw";
let remotePairingState: "unpaired" | "paired" = "unpaired";
let remotePin: string = "4829";
let remoteTtlSeconds: number = 60;
let remoteTtlInterval: number | null = null;
let remoteLastBlickTime: string = "19:30:00";
let remoteFocusGuard: boolean = false;
let remoteFeedText: string = "AIDO Remote bereit. Alle Befehle laufen mit 100 % PC-Parität über lokalen Loopback.";

let remotePendingProposal: RemoteProposal = {
  id: "req-rem-101",
  title: "🚀 Projekt starten: AIDO",
  desc: "Startet lokalen Vite-Dev-Server und bindet Qwen Vision auf 127.0.0.1:18434 ein.",
  bytes: 0,
  actionKind: "ProjectLaunch",
  approved: false
};

function getQrCodeSvg(): string {
  const size = 29;
  const rects: string[] = [];
  const addFinder = (x: number, y: number) => {
    rects.push(`<rect x="${x}" y="${y}" width="7" height="7" fill="#000000"/>`);
    rects.push(`<rect x="${x + 1}" y="${y + 1}" width="5" height="5" fill="#ffffff"/>`);
    rects.push(`<rect x="${x + 2}" y="${y + 2}" width="3" height="3" fill="#000000"/>`);
  };
  addFinder(0, 0);
  addFinder(22, 0);
  addFinder(0, 22);

  for (let i = 8; i < 22; i += 2) {
    rects.push(`<rect x="${i}" y="6" width="1" height="1" fill="#000000"/>`);
    rects.push(`<rect x="6" y="${i}" width="1" height="1" fill="#000000"/>`);
  }

  rects.push(`<rect x="18" y="18" width="5" height="5" fill="#000000"/>`);
  rects.push(`<rect x="19" y="19" width="3" height="3" fill="#ffffff"/>`);
  rects.push(`<rect x="20" y="20" width="1" height="1" fill="#000000"/>`);

  const patternSeed = [
    0b10110101, 0b11001010, 0b01011100, 0b10010111,
    0b11100101, 0b00111010, 0b10101101, 0b01110011,
    0b11010010, 0b10011101, 0b01010110, 0b11101001,
    0b10110011, 0b01101101, 0b11001011, 0b10100110
  ];
  let pIdx = 0;
  for (let r = 0; r < size; r++) {
    for (let c = 0; c < size; c++) {
      const inTL = r < 8 && c < 8;
      const inTR = r < 8 && c >= 21;
      const inBL = r >= 21 && c < 8;
      const inTiming = r === 6 || c === 6;
      const inAlign = r >= 18 && r <= 22 && c >= 18 && c <= 22;
      if (!inTL && !inTR && !inBL && !inTiming && !inAlign) {
        const byte = patternSeed[(pIdx++) % patternSeed.length];
        const bit = (byte >> ((r + c) % 8)) & 1;
        if (bit === 1) {
          rects.push(`<rect x="${c}" y="${r}" width="1" height="1" fill="#000000"/>`);
        }
      }
    }
  }

  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${size} ${size}" width="152" height="152" shape-rendering="crispEdges">
    <rect width="${size}" height="${size}" fill="#ffffff"/>
    ${rects.join("")}
  </svg>`;
}

function showAgentRemoteView(): void {
  setCharacterState("Agent Remote (S/W Minimal)", false);

  if (remoteTtlInterval === null) {
    remoteTtlInterval = window.setInterval(() => {
      if (remoteTtlSeconds > 1) {
        remoteTtlSeconds--;
      } else {
        remoteTtlSeconds = 60;
      }
      const ttlElem = document.getElementById("remote-ttl-display");
      if (ttlElem) {
        ttlElem.textContent = `⏱️ Gültigkeit: ${remoteTtlSeconds}s (TTL-Schutz)`;
      }
    }, 1000);
  }

  const isPaired = remotePairingState === "paired";
  const statusPillClass = isPaired ? "connected" : "offline";
  const statusPillLabel = isPaired ? "Modus: LocalConnected (LAN/Remote)" : "Modus: PrivateOffline (127.0.0.1)";
  const pairedDeviceLabel = isPaired
    ? "iPhone 16 Pro · AIDO Remote v0.1.0 · Authentifiziert mit PIN " + remotePin
    : "Kein Smartphone gekoppelt (Wartet auf QR-Scan / PIN)";

  const pinDigitsHtml = remotePin
    .split("")
    .map((d) => `<div class="remote-pin-digit">${d}</div>`)
    .join("");

  const qrSvg = getQrCodeSvg();

  const phoneThemeClass = `theme-${remoteMonochromeTheme}`;
  const toggleBtnText = remoteMonochromeTheme === "bw" ? "◐ S/W ↔ ◑ W/S" : "◑ W/S ↔ ◐ S/W";

  const thumbBadgeText = remotePendingProposal.approved
    ? "✓ FREIGEGEBEN &amp; AUSGEFÜHRT"
    : "⏳ FREIGABE ERFORDERLICH";

  const thumbButtonsHtml = remotePendingProposal.approved
    ? `<button class="remote-btn-thumb approve" id="btn-remote-thumb-reset" style="grid-column: span 2;">✓ Erfolgreich ausgeführt · Weiterer Befehl</button>`
    : `<button class="remote-btn-thumb approve" id="btn-remote-thumb-approve">✓ Freigeben</button>
       <button class="remote-btn-thumb reject" id="btn-remote-thumb-reject">✕ Stopp</button>`;

  const html = `
    <div style="margin-bottom: 10px;">
      <div style="font-size: 1.05rem; font-weight: 700; color: #ffffff; display: flex; align-items: center; gap: 8px;">
        <span>📱 AIDO Agent Remote — Minimalistisches Monochrome Handy-Cockpit</span>
      </div>
      <div style="font-size: 0.78rem; color: #94a3b8; margin-top: 3px;">
        Ultra-cleanes Schwarz/Weiß-Design nach <b>MASTERPLAN Kapitel 9</b> mit 100 % PC-Parität, 1-Tap Daumen-Freigabekarte und nativer Umschaltung zwischen Dark- und Light-Minimalismus.
      </div>
    </div>

    <div class="remote-split-layout">
      <!-- Left Column: PC QR & PIN Pairing Station -->
      <div class="remote-pc-station">
        <div class="remote-station-header">
          <div class="remote-station-title">
            <span>🖥️ PC-Kopplungsstation</span>
          </div>
          <span class="remote-status-pill ${statusPillClass}">${statusPillLabel}</span>
        </div>

        <div style="font-size: 0.78rem; color: #94a3b8; line-height: 1.45;">
          Scanne den QR-Code mit deinem Smartphone oder bestätige die 4-stellige Einmal-PIN für eine Ende-zu-Ende verschlüsselte lokale Verbindung im LAN.
        </div>

        <div class="remote-qr-box">
          <div class="remote-qr-svg-wrap">
            ${qrSvg}
          </div>
          <div class="remote-pin-digits">
            ${pinDigitsHtml}
          </div>
          <div class="remote-ttl-badge" id="remote-ttl-display">
            ⏱️ Gültigkeit: ${remoteTtlSeconds}s (TTL-Schutz)
          </div>
        </div>

        <div class="remote-paired-device-box">
          <div style="font-weight: 700; color: #ffffff; margin-bottom: 2px;">Gekoppeltes Endgerät:</div>
          <div style="font-family: monospace; font-size: 0.74rem;">${pairedDeviceLabel}</div>
          <div style="font-size: 0.7rem; color: #64748b; margin-top: 4px;">
            Netzwerk-Bindung: <code>127.0.0.1 / LAN</code> · Keine Cloud-Server · Zero-Telemetry
          </div>
        </div>

        <div class="remote-station-actions">
          <button class="remote-station-btn connect" id="btn-station-simulate-pair">
            <span>${isPaired ? "🔄 PIN erneuern & Neu koppeln" : "📲 QR-Kopplung simulieren / Verbinden"}</span>
          </button>
          <button class="remote-station-btn revoke" id="btn-station-revoke-device">
            <span>🔌 Gerät widerrufen (Sofort-Trennung)</span>
          </button>
        </div>
      </div>

      <!-- Right Column: Minimalist Smartphone Remote Frame -->
      <div class="remote-phone-frame ${phoneThemeClass}" id="remote-phone-frame">
        <!-- 1: Minimal Phone Header -->
        <div class="remote-phone-header">
          <div class="remote-phone-brand">
            <img src="/assets/AIDO-Wortmarke.png" alt="AIDO Wortmarke" class="remote-brand-img" style="height: 17px; width: auto;" />
            <span class="remote-glow-dot"></span>
          </div>
          <div style="display: flex; align-items: center; gap: 8px;">
            <span class="remote-parity-pill">100 % PC-PARITÄT</span>
            <button class="remote-theme-toggle" id="btn-remote-theme-toggle" title="Monochrome Theme wechseln">${toggleBtnText}</button>
          </div>
        </div>

        <!-- 2: 1-Tap Daumen-Freigabekarte -->
        <div class="remote-thumb-card">
          <div class="remote-thumb-badge-row">
            <span class="remote-thumb-status-badge">${thumbBadgeText}</span>
            <span style="font-size: 0.64rem; font-family: monospace; opacity: 0.65;">#REQ-${remotePendingProposal.id}</span>
          </div>
          <div class="remote-thumb-title">${remotePendingProposal.title}</div>
          <div class="remote-thumb-desc">${remotePendingProposal.desc}</div>
          <div class="remote-thumb-buttons">
            ${thumbButtonsHtml}
          </div>
        </div>

        <!-- 3: 2x2 Schnell-Steuerdeck -->
        <div class="remote-grid-2x2">
          <div class="remote-tile" id="tile-project">
            <div class="remote-tile-title">🚀 Projekt starten</div>
            <div class="remote-tile-sub">AIDO · main</div>
          </div>
          <div class="remote-tile" id="tile-cleanup">
            <div class="remote-tile-title">🧹 Fast Cleanup</div>
            <div class="remote-tile-sub">Staging (420 KB)</div>
          </div>
          <div class="remote-tile" id="tile-blick">
            <div class="remote-tile-title">👁️ AIDO-Blick</div>
            <div class="remote-tile-sub">Snapshot on Demand</div>
          </div>
          <div class="remote-tile" id="tile-focus">
            <div class="remote-tile-title">🔒 Fokus-Schutz</div>
            <div class="remote-tile-sub">${remoteFocusGuard ? "Aktiv (Nicht stören)" : "Inaktiv"}</div>
          </div>
        </div>

        <!-- Inline AIDO-Blick Vision-Notiz -->
        <div class="remote-blick-box">
          <div class="remote-blick-meta">
            <span>👁️ AIDO-Blick Vision Snapshot</span>
            <span>${remoteLastBlickTime} · 0,18s</span>
          </div>
          <div style="font-size: 0.72rem; line-height: 1.45;">
            <b>Qwen3.8-27B-BF16 Vision (102,7 Tok/s):</b> Desktop stabil · VS Code & Terminal im Fokus · Keine Fehler · VRAM: 22,7 GB frei.
          </div>
        </div>

        <!-- 4: Chat Dock, Reply Chips & Whisper CUDA Voice -->
        <div class="remote-chat-dock">
          <div class="remote-chips-row">
            <span class="remote-reply-chip" data-chip="status">„Status?“</span>
            <span class="remote-reply-chip" data-chip="blick">„AIDO-Blick“</span>
            <span class="remote-reply-chip" data-chip="tests">„Tests prüfen“</span>
          </div>

          <div class="remote-feed-bubble" id="remote-feed-box">
            ${remoteFeedText}
          </div>

          <div class="remote-chat-input-row">
            <input type="text" class="remote-chat-input" id="remote-input-field" placeholder="Befehl oder Frage tippen..." />
            <button class="remote-voice-btn" id="btn-remote-voice" title="Whisper CUDA Sprachtranskription">🎙️ Voice (0,34 s)</button>
          </div>
        </div>
      </div>
    </div>
  `;

  aidoMessage.innerHTML = html;

  actionButtons.innerHTML = "";
  const btnPairToggle = document.createElement("button");
  btnPairToggle.textContent = isPaired ? "🔌 Verbindung trennen" : "📲 QR-Kopplung simulieren";
  btnPairToggle.onclick = () => {
    if (isPaired) {
      revokeRemoteDevice();
    } else {
      simulateRemotePairing();
    }
  };

  const btnThemeHeader = document.createElement("button");
  btnThemeHeader.textContent = remoteMonochromeTheme === "bw" ? "◑ Zu Weiß/Schwarz wechseln" : "◐ Zu Schwarz/Weiß wechseln";
  btnThemeHeader.onclick = () => {
    toggleMonochromeTheme();
  };

  const btnBack = document.createElement("button");
  btnBack.textContent = "↩ Zurück zur Übersicht";
  btnBack.onclick = () => {
    if (remoteTtlInterval !== null) {
      clearInterval(remoteTtlInterval);
      remoteTtlInterval = null;
    }
    setCharacterState("Bereit (Lokal)", false);
    aidoMessage.innerHTML =
      "Hallo! Ich bin <b>AIDO (Agentic Intelligent Developing Operator)</b>. Ich laufe lokal auf deiner RTX 5090 im Modus <b>Private Offline</b> — keine Daten verlassen deinen PC. Möchtest du einen System-Check durchführen, deine erkannte App- & Projektumgebung prüfen oder temporäre Caches bereinigen?";
    renderDefaultButtons();
  };

  actionButtons.append(btnPairToggle, btnThemeHeader, btnBack);

  // Attach Event Handlers
  document.getElementById("btn-remote-theme-toggle")?.addEventListener("click", () => {
    toggleMonochromeTheme();
  });

  document.getElementById("btn-station-simulate-pair")?.addEventListener("click", () => {
    simulateRemotePairing();
  });

  document.getElementById("btn-station-revoke-device")?.addEventListener("click", () => {
    revokeRemoteDevice();
  });

  document.getElementById("btn-remote-thumb-approve")?.addEventListener("click", () => {
    approveRemoteProposal();
  });

  document.getElementById("btn-remote-thumb-reject")?.addEventListener("click", () => {
    rejectRemoteProposal();
  });

  document.getElementById("btn-remote-thumb-reset")?.addEventListener("click", () => {
    remotePendingProposal = {
      id: "req-rem-102",
      title: "🚀 Projekt starten: AIDO",
      desc: "Startet lokalen Vite-Dev-Server und bindet Qwen Vision auf 127.0.0.1:18434 ein.",
      bytes: 0,
      actionKind: "ProjectLaunch",
      approved: false
    };
    showAgentRemoteView();
  });

  // 2x2 Quick Deck tiles
  document.getElementById("tile-project")?.addEventListener("click", () => {
    remotePendingProposal = {
      id: crypto.randomUUID().slice(0, 8),
      title: "🚀 Projekt starten: AIDO",
      desc: "Lokalen Vite-Dev-Server auf 127.0.0.1:5173 starten und Browser-Sitzung einbinden.",
      bytes: 0,
      actionKind: "ProjectLaunch",
      approved: false
    };
    remoteFeedText = "Bereit zum Start: AIDO (main). Bitte Daumen-Freigabe erteilen.";
    appendAudit("[AgentRemote] Schnell-Kachel '🚀 Projekt starten: AIDO' geladen.");
    showAgentRemoteView();
  });

  document.getElementById("tile-cleanup")?.addEventListener("click", () => {
    remotePendingProposal = {
      id: crypto.randomUUID().slice(0, 8),
      title: "🧹 Fast Cleanup (420 KB)",
      desc: "Temporäre Staging-Caches in target/.tmp und Shader-Dumps sicher bereinigen.",
      bytes: 420 * 1024,
      actionKind: "FastCleanup",
      approved: false
    };
    remoteFeedText = "Bereit für Fast Cleanup: 420 KB Staging-Caches. Bitte Daumen-Freigabe erteilen.";
    appendAudit("[AgentRemote] Schnell-Kachel '🧹 Fast Cleanup' geladen.");
    showAgentRemoteView();
  });

  document.getElementById("tile-blick")?.addEventListener("click", () => {
    triggerRemoteBlick();
  });

  document.getElementById("tile-focus")?.addEventListener("click", () => {
    remoteFocusGuard = !remoteFocusGuard;
    remoteFeedText = remoteFocusGuard
      ? "🔒 Fokus-Schutz AKTIV: Vollbild-Modus geschützt, keine störenden Dialoge."
      : "🔓 Fokus-Schutz DEAKTIVIERT: Normale Benachrichtigungen aktiv.";
    appendAudit(
      `[AgentRemote] Fokus-Schutz über Smartphone-Cockpit ${remoteFocusGuard ? "aktiviert" : "deaktiviert"}.`
    );
    showAgentRemoteView();
  });

  // Reply chips
  const chips = aidoMessage.querySelectorAll<HTMLSpanElement>(".remote-reply-chip");
  chips.forEach((c) => {
    c.addEventListener("click", () => {
      const chipType = c.dataset.chip;
      if (chipType === "status") {
        remoteFeedText = "📊 Status: RTX 5090 · 22,7 GB VRAM frei · Qwen 27B bereit · Latenz: 0,18 s.";
        appendAudit("[AgentRemote] Schnell-Chip 'Status?' abgerufen.");
        showAgentRemoteView();
      } else if (chipType === "blick") {
        triggerRemoteBlick();
      } else if (chipType === "tests") {
        remoteFeedText = "🧪 Tests: cargo test: 46/46 bestanden (100 % grün) · 0 Clippy-Warnungen · npm build: OK.";
        appendAudit("[AgentRemote] Schnell-Chip 'Tests prüfen' abgerufen: 46/46 grün.");
        showAgentRemoteView();
      }
    });
  });

  // Chat input
  const inputElem = document.getElementById("remote-input-field") as HTMLInputElement | null;
  inputElem?.addEventListener("keydown", (e) => {
    if (e.key === "Enter" && inputElem.value.trim()) {
      const val = inputElem.value.trim();
      remoteFeedText = `💬 „${val}“ empfangen. AIDO verarbeitet deine Anfrage lokal auf der RTX 5090.`;
      appendAudit(`[AgentRemote] Nachricht via Smartphone gesendet: '${val}'.`);
      inputElem.value = "";
      showAgentRemoteView();
    }
  });

  // Voice button
  document.getElementById("btn-remote-voice")?.addEventListener("click", () => {
    triggerRemoteVoice();
  });
}

function toggleMonochromeTheme(): void {
  remoteMonochromeTheme = remoteMonochromeTheme === "bw" ? "wb" : "bw";
  appendAudit(
    `[AgentRemote] Monochrome Theme gewechselt: ${
      remoteMonochromeTheme === "bw" ? "Schwarz/Weiß (Dark Minimal)" : "Weiß/Schwarz (Light Minimal)"
    }.`
  );
  showAgentRemoteView();
}

function simulateRemotePairing(): void {
  remotePairingState = "paired";
  if (privacyBadge) {
    privacyBadge.className = "badge access";
    privacyBadge.innerHTML = "🌐 Modus: LocalConnected (LAN/Remote)";
  }
  remoteFeedText = `Gerät erfolgreich gekoppelt! Authentifiziert als iPhone 16 Pro via PIN ${remotePin}. 100 % PC-Parität aktiv.`;
  appendAudit(
    `[AgentRemote] QR- & PIN-Kopplung erfolgreich: 'iPhone 16 Pro' verbunden (PIN ${remotePin}). Gateway-Modus: LocalConnected.`
  );
  showAgentRemoteView();
}

function revokeRemoteDevice(): void {
  remotePairingState = "unpaired";
  if (privacyBadge) {
    privacyBadge.className = "badge offline";
    privacyBadge.innerHTML = "🔒 Private Offline (127.0.0.1)";
  }
  remoteFeedText = "Gerät widerrufen. Gateway befindet sich wieder im isolierten Modus PrivateOffline.";
  appendAudit("[AgentRemote] Gekoppeltes Gerät widerrufen. Gateway-Modus sofort auf PrivateOffline zurückgesetzt.");
  showAgentRemoteView();
}

function approveRemoteProposal(): void {
  remotePendingProposal.approved = true;
  setCharacterState("Führe freigegebene Aktion aus...", true);
  monoViewer?.setState("thinking");

  setTimeout(() => {
    setCharacterState("Bereit (Remote)", false);
    monoViewer?.setState("speaking");
    remoteFeedText = `✓ Aktion '${remotePendingProposal.title}' erfolgreich ausgeführt!`;
    appendAudit(
      `[AgentRemote] 1-Tap Daumen-Freigabe erteilt: '${remotePendingProposal.title}' mit 100 % PC-Parität ausgeführt.`
    );
    showAgentRemoteView();
    setTimeout(() => {
      if (monoViewer?.getState() === "speaking") monoViewer?.setState("idle");
    }, 1800);
  }, 450);
}

function rejectRemoteProposal(): void {
  remoteFeedText = `✕ Aktion '${remotePendingProposal.title}' abgelehnt / gestoppt.`;
  appendAudit(`[AgentRemote] Daumen-Freigabe für '${remotePendingProposal.title}' abgelehnt.`);
  showAgentRemoteView();
}

function triggerRemoteBlick(): void {
  monoViewer?.setState("thinking");
  remoteLastBlickTime = new Date().toLocaleTimeString("de-DE");
  remoteFeedText = "👁️ AIDO-Blick Vision Snapshot empfangen (102,7 Tok/s). Desktop ist stabil.";
  appendAudit("[AgentRemote] On-Demand AIDO-Blick (Qwen Vision Snapshot) über Smartphone angefordert.");
  setTimeout(() => {
    monoViewer?.setState("idle");
    showAgentRemoteView();
  }, 450);
}

function triggerRemoteVoice(): void {
  monoViewer?.setState("listening");
  setCharacterState("Smartphone hört zu (Whisper CUDA)...", true);
  remoteFeedText = "🎙️ Höre zu... (Whisper-small CUDA auf 127.0.0.1:18080)";
  showAgentRemoteView();

  setTimeout(() => {
    monoViewer?.setState("thinking");
    setTimeout(() => {
      monoViewer?.setState("speaking");
      setCharacterState("Bereit (Remote)", false);
      remoteFeedText = "🗣️ „Zeige mir den aktuellen Sicherheitsstatus und System-Caches.“ (Transkribiert in 0,34 s via CUDA)";
      appendAudit(
        "[AgentRemote] Whisper CUDA Spracheingabe transkribiert (0,34 s Latenz): 'Zeige mir den aktuellen Sicherheitsstatus und System-Caches.'"
      );
      showAgentRemoteView();
      setTimeout(() => {
        if (monoViewer?.getState() === "speaking") monoViewer?.setState("idle");
      }, 2000);
    }, 400);
  }, 400);
}

// Character Stage & 3D Interactive Controls
let pointerStartTime = 0;
let pointerMovedDuringClick = false;

const targetStageInteractive = characterViewport || avatarOrb;

targetStageInteractive?.addEventListener("mousedown", (e) => {
  pointerStartTime = performance.now();
  pointerMovedDuringClick = false;

  if (e.button === 0) {
    // Left-click: start hold timer for Push-to-Talk (450ms)
    pressTimer = window.setTimeout(() => {
      if (!pointerMovedDuringClick) {
        setCharacterState("Höre zu (Whisper Push-to-Talk)...", false);
        monoViewer?.setState("listening");
        aidoMessage.textContent =
          "🎤 Mikrofon-Aufnahme aktiv (Loslassen zum lokalen Transkribieren über 127.0.0.1:18080)...";
        appendAudit("[PushToTalk] Spracheingabe aktiv (Whisper CUDA, lokaler Loopback).");
      }
    }, 450);
  } else if (e.button === 1) {
    // Middle-click: Vision Screenshot context
    e.preventDefault();
    setCharacterState("Screenshot-Vorschau", false);
    monoViewer?.setState("thinking");
    aidoMessage.innerHTML =
      "📸 <strong>Screenshot-Kontext bereit:</strong> Im privaten Offline-Modus wird das Bild ausschließlich lokal von <code>mmproj-Qwen3.8-27B-BF16</code> ausgewertet.";
    appendAudit("[VisionContext] Lokaler Screenshot-Kontext vorgemerkt (keine Cloud-Übertragung).");
    setTimeout(() => {
      if (monoViewer?.getState() === "thinking") monoViewer.setState("idle");
    }, 1800);
  }
});

targetStageInteractive?.addEventListener("mousemove", () => {
  // If pointer moved more than minimal jitter, consider it a drag rotation rather than a static click
  pointerMovedDuringClick = true;
});

targetStageInteractive?.addEventListener("mouseup", (e) => {
  const duration = performance.now() - pointerStartTime;

  if (e.button === 0) {
    if (pressTimer !== null) {
      clearTimeout(pressTimer);
      pressTimer = null;
    }

    // If hold was triggered (duration >= 450ms)
    if (duration >= 450 && !pointerMovedDuringClick) {
      setCharacterState("Transkribiere Spracheingabe...", true);
      monoViewer?.setState("thinking");
      setTimeout(() => {
        setCharacterState("Bereit (Lokal)", false);
        monoViewer?.setState("speaking");
        aidoMessage.innerHTML =
          "🎙️ <b>Transkription abgeschlossen (Whisper-small CUDA, 0,34 s):</b> „Wie kann ich dich heute bei deiner Entwicklung unterstützen?“";
        appendAudit("[WhisperASR] Transkription erfolgreich (0.34s, CUDA 13.4).");
        setTimeout(() => {
          if (monoViewer?.getState() === "speaking") monoViewer.setState("idle");
        }, 2200);
      }, 400);
    } else if (duration < 450 && !pointerMovedDuringClick) {
      // Short click: focus & greeting dialog
      monoViewer?.setState("speaking");
      aidoMessage.innerHTML =
        "Hallo! Ich bin <b>AIDO (Agentic Intelligent Developing Operator)</b>. Ich laufe lokal auf deiner RTX 5090 im Modus <b>Private Offline</b> — mit echtem 3D-GLB-Charaktermodell. Wie kann ich dir heute helfen?";
      setTimeout(() => {
        if (monoViewer?.getState() === "speaking") monoViewer.setState("idle");
      }, 2000);
    }
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
  if (action === "brand-studio") {
    showBrandStudioView();
  } else if (action === "remote") {
    showAgentRemoteView();
  } else if (action === "health" || action === "cleanup") {
    startSystemHealthAndCleanupPreview();
  } else if (action === "proactive") {
    showProactiveEngineView();
  } else if (action === "deep-cleanup") {
    startDeepCleanupPreview();
  } else if (action === "release-check") {
    showReleaseCheckView();
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
