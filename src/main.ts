import { Mono3DViewer, type MonoState } from "./mono3d";

/**
 * AIDO — Dein persönlicher Desktop-Begleiter
 *
 * Minimalist, ChatGPT-clean Monochrome Companion UI Controller.
 * Strictly monochrome (#FFFFFF / #000000), zero emojis, zero technical jargon.
 */

type StageId = "agent" | "overview" | "phone";

interface ChatMessage {
  id: string;
  role: "aido" | "user";
  text: string;
  attachmentLabel?: string;
}

interface InlineApprovalProposal {
  id: string;
  title: string;
  reason: string;
  itemsSummary: string;
  onApprove: () => void;
  onReject: () => void;
}

interface ProjectOverviewItem {
  id: string;
  title: string;
  pathLabel: string;
  summary: string;
  tags: string[];
  statusText: string;
}

interface AppToolItem {
  id: string;
  name: string;
  description: string;
  category: string;
  userEnabled: boolean;
}

interface MemoryEntry {
  id: number;
  title: string;
  details: string;
  category: string;
}

interface TaskEntry {
  id: string;
  title: string;
  schedule: string;
  statusLabel: string;
}

interface PhoneUiNode {
  id: string;
  label: string;
  hint: string;
  targetScreen?: PhoneScreenId;
  actionSummary: string;
  requiresApproval?: boolean;
}

type PhoneScreenId = "home" | "settings" | "network" | "github";

// ============================================================================
// Application State
// ============================================================================

let monoViewer: Mono3DViewer | null = null;
let currentStage: StageId = "agent";
let currentTheme: "light" | "dark" = "light";
let currentCharacterState: MonoState = "idle";

let morphTimers: number[] = [];
let stateTransitionTimers: number[] = [];

let isVoiceRecording = false;
let voiceRecordStartedAt = 0;
let attachedScreenshotLabel: string | null = null;

let cleanupCompleted = false;
let pendingTempFilesCount = 5;
let pendingTempSizeLabel = "576 KB";

const projectsState: ProjectOverviewItem[] = [
  {
    id: "aido-companion",
    title: "AIDO — Desktop-Begleiter",
    pathLabel: "Arbeitsbereich · Desktop/AIDO",
    summary: "Hauptprojekt aktiv · Alle Prüfungen erfolgreich · Bereit für den nächsten Schritt.",
    tags: ["Hauptprojekt", "Aktuell geöffnet", "Bereit"],
    statusText: "Alle Komponenten sind aktuell und fehlerfrei."
  },
  {
    id: "landingpage-brand-morph",
    title: "Landingpage — Brand Morph",
    pathLabel: "Arbeitsbereich · Marken-Animation & 3D-Charakter",
    summary: "Nahtlose Übergangs-Animation und interaktive 3D-Darstellung sind synchronisiert.",
    tags: ["Design", "3D-Charakter", "Synchronisiert"],
    statusText: "Marken-Transformation und 3D-Ansicht sind aktiv eingebunden."
  }
];

const appsState: AppToolItem[] = [
  {
    id: "vscode",
    name: "VS Code",
    description: "Projekte öffnen, Dateien bearbeiten und Aufgaben begleiten",
    category: "Entwicklung",
    userEnabled: true
  },
  {
    id: "windows-terminal",
    name: "Windows Terminal",
    description: "Befehle nach Rückfrage sicher im Projektordner ausführen",
    category: "System",
    userEnabled: true
  },
  {
    id: "browser",
    name: "Browser",
    description: "Webseiten öffnen und Recherchen zusammenfassen",
    category: "Alltag",
    userEnabled: true
  },
  {
    id: "docker-desktop",
    name: "Docker Desktop",
    description: "Lokale Arbeitsumgebungen bei Bedarf starten oder anhalten",
    category: "Werkzeug",
    userEnabled: true
  },
  {
    id: "system-analyse",
    name: "System-Analyse",
    description: "Speicherplatz prüfen und temporäre Dateien sicher vorschlagen",
    category: "Wartung",
    userEnabled: true
  }
];

let memoriesState: MemoryEntry[] = [
  {
    id: 1,
    title: "Bevorzugtes Erscheinungsbild: Klares Monochrom-Design",
    details: "Ruhige Oberfläche in Schwarz und Weiß ohne ablenkende Farben oder Symbole.",
    category: "Präferenz"
  },
  {
    id: 2,
    title: "Aktiver Arbeitsbereich: AIDO Desktop-Begleiter",
    details: "Fokus auf direkte Charakter-Interaktion, klare Antworten und einfache Freigaben.",
    category: "Projekt"
  },
  {
    id: 3,
    title: "Sicherheitsregel: Immer vor Änderungen nachfragen",
    details: "Dateibereinigungen und wichtige Handy-Aktionen erfordern deine ausdrückliche Zustimmung.",
    category: "Sicherheit"
  },
  {
    id: 4,
    title: "Letzte Bereinigung: 448 KB Zwischenspeicher freigegeben",
    details: "Drei nicht mehr benötigte temporäre Dateien wurden nach Freigabe sicher entfernt.",
    category: "Verlauf"
  }
];

const tasksState: TaskEntry[] = [
  {
    id: "task-morning",
    title: "Morgen-Überblick vorbereiten",
    schedule: "Täglich beim ersten Aufwachen",
    statusLabel: "Aktiv"
  },
  {
    id: "task-cleanup-hint",
    title: "Zwischenspeicher schonend prüfen",
    schedule: "Wöchentlich mit vorheriger Rückfrage",
    statusLabel: "Bereit"
  },
  {
    id: "task-project-guard",
    title: "Projektstatus im Blick behalten",
    schedule: "Bei Änderungen im Arbeitsbereich",
    statusLabel: "Aktiv"
  }
];

let currentPhoneScreen: PhoneScreenId = "home";
let phoneScrollOffset = 0;
let phoneLastTypedText = "";
let qrTokenSeed = 482910;

const PHONE_SCREEN_META: Record<
  PhoneScreenId,
  { title: string; nodes: PhoneUiNode[] }
> = {
  home: {
    title: "Startbildschirm",
    nodes: [
      {
        id: "node-settings",
        label: "Einstellungen",
        hint: "System, WLAN & Anzeige anpassen",
        targetScreen: "settings",
        actionSummary: "App „Einstellungen“ auf dem Smartphone geöffnet."
      },
      {
        id: "node-github",
        label: "GitHub — keokukzh/AIDO",
        hint: "Projekt-Übersicht im Browser öffnen",
        targetScreen: "github",
        actionSummary: "Projektseite „keokukzh/AIDO“ auf dem Smartphone geöffnet."
      },
      {
        id: "node-network",
        label: "Netzwerk & Internet",
        hint: "WLAN-Verbindung und Kopplungsstatus",
        targetScreen: "network",
        actionSummary: "Bereich „Netzwerk & Internet“ geöffnet."
      },
      {
        id: "node-send-msg",
        label: "Nachricht senden (Freigabe erforderlich)",
        hint: "Vor dem Absenden fragt AIDO nach deiner Erlaubnis",
        actionSummary: "Nachricht wurde nach deiner Freigabe gesendet.",
        requiresApproval: true
      }
    ]
  },
  settings: {
    title: "Einstellungen",
    nodes: [
      {
        id: "node-settings-net",
        label: "Netzwerk & Internet",
        hint: "Verbunden mit Heimnetzwerk",
        targetScreen: "network",
        actionSummary: "Untermenü „Netzwerk & Internet“ geöffnet."
      },
      {
        id: "node-settings-display",
        label: "Anzeige & Helligkeit",
        hint: "Monochrom-Lesemodus aktivieren",
        actionSummary: "Anzeige-Optionen auf dem Smartphone geprüft."
      },
      {
        id: "node-settings-battery",
        label: "Akku & Gerätestatus",
        hint: "Akkustand 88 % · Normaler Betrieb",
        actionSummary: "Akkustatus (88 %) auf dem Smartphone ausgelesen."
      },
      {
        id: "node-settings-home",
        label: "Zurück zum Startbildschirm",
        hint: "Einstellungen verlassen",
        targetScreen: "home",
        actionSummary: "Zurück zum Startbildschirm gewechselt."
      }
    ]
  },
  network: {
    title: "Netzwerk & Internet",
    nodes: [
      {
        id: "node-net-wlan",
        label: "WLAN · Verbunden",
        hint: "Stabile lokale Verbindung zu AIDO",
        actionSummary: "WLAN-Verbindung geprüft: Signalstärke ausgezeichnet."
      },
      {
        id: "node-net-pair",
        label: "Kabellose Kopplung aktiv",
        hint: "Verschlüsselte Direktverbindung",
        actionSummary: "Kabellose Kopplung mit deinem Desktop bestätigt."
      },
      {
        id: "node-net-back",
        label: "Zurück zu Einstellungen",
        hint: "Vorherige Ansicht öffnen",
        targetScreen: "settings",
        actionSummary: "Zurück zu den Einstellungen gewechselt."
      }
    ]
  },
  github: {
    title: "GitHub — keokukzh/AIDO",
    nodes: [
      {
        id: "node-gh-repo",
        label: "Projektübersicht lesen",
        hint: "AIDO — Persönlicher Desktop-Begleiter",
        actionSummary: "Projektbeschreibung auf dem Smartphone gelesen."
      },
      {
        id: "node-gh-commits",
        label: "Letzte Änderungen anzeigen",
        hint: "Charakter-Zentrierte Oberfläche & 3D-Ansicht",
        actionSummary: "Liste der letzten Änderungen geöffnet."
      },
      {
        id: "node-gh-home",
        label: "Zurück zum Startbildschirm",
        hint: "Browser schließen",
        targetScreen: "home",
        actionSummary: "Zurück zum Startbildschirm gewechselt."
      }
    ]
  }
};

// ============================================================================
// Status Labels for the 7 Character States
// ============================================================================

const CHARACTER_STATE_BADGES: Record<MonoState, { short: string; full: string }> = {
  sleep: {
    short: "Schlafen",
    full: "Schlafen — Ruhemodus aktiv (Klicke zum Aufwecken)"
  },
  wake: {
    short: "Aufwachen",
    full: "Aufwachen — AIDO richtet sich aus…"
  },
  idle: {
    short: "Bereit",
    full: "Bereit — Klicke oder sprich mit AIDO"
  },
  listen: {
    short: "Zuhören",
    full: "Zuhören — Sprich jetzt…"
  },
  listening: {
    short: "Zuhören",
    full: "Zuhören — Sprich jetzt…"
  },
  think: {
    short: "Nachdenken",
    full: "Nachdenken — AIDO plant die nächsten Schritte…"
  },
  thinking: {
    short: "Nachdenken",
    full: "Nachdenken — AIDO plant die nächsten Schritte…"
  },
  work: {
    short: "Arbeiten",
    full: "Arbeiten — Aufgabe wird ausgeführt…"
  },
  speak: {
    short: "Antworten",
    full: "Antworten — AIDO spricht mit dir…"
  },
  speaking: {
    short: "Antworten",
    full: "Antworten — AIDO spricht mit dir…"
  },
  success: {
    short: "Erfolg",
    full: "Erfolg — Aufgabe abgeschlossen"
  },
  error: {
    short: "Hinweis",
    full: "Fehlermeldung — Bitte prüfen"
  }
};

// ============================================================================
// Helper Utilities
// ============================================================================

function clearStateTimers(): void {
  for (const id of stateTransitionTimers) {
    window.clearTimeout(id);
  }
  stateTransitionTimers = [];
}

function clearMorphTimers(): void {
  for (const id of morphTimers) {
    window.clearTimeout(id);
  }
  morphTimers = [];
}

function scheduleStateTransition(fn: () => void, delayMs: number): void {
  const id = window.setTimeout(fn, delayMs);
  stateTransitionTimers.push(id);
}

// ============================================================================
// 1. Character State & Brand-Morph Awakening Sequence
// ============================================================================

export function setCharacterState(state: MonoState, customStatusText?: string): void {
  currentCharacterState = state;
  monoViewer?.setState(state);

  const meta = CHARACTER_STATE_BADGES[state] || CHARACTER_STATE_BADGES.idle;

  const badgeEl = document.getElementById("mono-state-badge");
  if (badgeEl) {
    badgeEl.textContent = customStatusText || meta.full;
  }

  const topbarLabel = document.getElementById("topbar-state-label");
  if (topbarLabel) {
    topbarLabel.textContent = meta.short;
  }

  const topbarDot = document.getElementById("topbar-state-dot");
  if (topbarDot) {
    const isActivePulse =
      state === "listen" ||
      state === "listening" ||
      state === "think" ||
      state === "thinking" ||
      state === "work" ||
      state === "speak" ||
      state === "speaking";
    topbarDot.classList.toggle("pulse", isActivePulse);
  }

  const voiceRing = document.getElementById("mono-voice-ring");
  if (voiceRing) {
    const isVoiceState =
      state === "listen" ||
      state === "listening" ||
      state === "speak" ||
      state === "speaking";
    voiceRing.classList.toggle("active", isVoiceState);
  }

  // Highlight corresponding button in #mono-state-selector
  const normalizedSelectorState =
    state === "listening"
      ? "listen"
      : state === "thinking"
        ? "think"
        : state === "idle" || state === "speak" || state === "speaking"
          ? "wake"
          : state;

  const buttons = document.querySelectorAll<HTMLButtonElement>(
    "#mono-state-selector [data-mono-state]"
  );
  buttons.forEach((btn) => {
    const btnState = btn.getAttribute("data-mono-state");
    btn.classList.toggle("active", btnState === normalizedSelectorState);
  });
}

export function playBrandMorphAwakening(): void {
  clearMorphTimers();
  const morphLayer = document.getElementById("morph-awakening-layer");
  if (!morphLayer) return;

  // Restart CSS keyframe animations cleanly
  morphLayer.classList.remove("playing");
  void morphLayer.offsetWidth;
  morphLayer.classList.add("playing");

  setCharacterState("wake", "Erwachen — AGENTIC INTELLIGENT DEVELOP OPERATOR → AIDO → MONO");

  // At the MONO peak (~3.75s), crossfade smoothly into the interactive 3D character
  const t1 = window.setTimeout(() => {
    morphLayer.classList.remove("playing");
    monoViewer?.resetCamera();
    setCharacterState("wake", "Aufwachen — AIDO ist bereit für dich");
  }, 3750);

  const t2 = window.setTimeout(() => {
    setCharacterState("idle");
  }, 4950);

  morphTimers.push(t1, t2);
}

function finishMorphImmediately(): void {
  const morphLayer = document.getElementById("morph-awakening-layer");
  if (morphLayer && morphLayer.classList.contains("playing")) {
    clearMorphTimers();
    morphLayer.classList.remove("playing");
    setCharacterState("idle");
  }
}

// ============================================================================
// 2. Navigation (Agent · Overview · Handy) & Theme Toggle (W/S ⇄ S/W)
// ============================================================================

function switchStage(stage: StageId): void {
  currentStage = stage;

  const views: Record<StageId, HTMLElement | null> = {
    agent: document.getElementById("view-agent"),
    overview: document.getElementById("view-overview"),
    phone: document.getElementById("view-phone")
  };

  (Object.keys(views) as StageId[]).forEach((key) => {
    const el = views[key];
    if (el) {
      el.hidden = key !== stage;
    }
  });

  const tabs = document.querySelectorAll<HTMLButtonElement>(".stage-nav .stage-tab");
  tabs.forEach((tab) => {
    const tabStage = tab.getAttribute("data-stage");
    tab.classList.toggle("active", tabStage === stage);
  });
}

function toggleTheme(): void {
  currentTheme = currentTheme === "light" ? "dark" : "light";
  document.documentElement.setAttribute("data-theme", currentTheme);
  monoViewer?.setTheme(currentTheme);

  const labelEl = document.getElementById("theme-toggle-label");
  if (labelEl) {
    labelEl.textContent = currentTheme === "light" ? "W/S" : "S/W";
  }
}

function triggerEmergencyStop(): void {
  clearStateTimers();
  finishMorphImmediately();

  if (isVoiceRecording) {
    isVoiceRecording = false;
    const voiceBtn = document.getElementById("btn-voice-ptt");
    voiceBtn?.classList.remove("active");
  }

  const inlineApproval = document.getElementById("inline-approval-container");
  if (inlineApproval) {
    inlineApproval.innerHTML = "";
  }

  const phoneApproval = document.getElementById("phone-approval-box");
  if (phoneApproval) {
    phoneApproval.hidden = true;
    phoneApproval.innerHTML = "";
  }

  setCharacterState("idle", "Alle Vorgänge gestoppt — Bereit");
}

// ============================================================================
// 3. Right-Click Character Context Menu & Screenshot Attachment
// ============================================================================

function openContextMenu(clientX: number, clientY: number): void {
  const menu = document.getElementById("mono-context-menu");
  if (!menu) return;

  menu.hidden = false;
  const menuWidth = 256;
  const menuHeight = 270;
  const clampedX = Math.max(12, Math.min(window.innerWidth - menuWidth - 12, clientX));
  const clampedY = Math.max(12, Math.min(window.innerHeight - menuHeight - 12, clientY));

  menu.style.left = `${clampedX}px`;
  menu.style.top = `${clampedY}px`;
}

function closeContextMenu(): void {
  const menu = document.getElementById("mono-context-menu");
  if (menu) {
    menu.hidden = true;
  }
}

function attachScreenshotContext(): void {
  attachedScreenshotLabel = "Bildschirmfoto (Aktuelles Fenster)";
  const pill = document.getElementById("composer-attachment-pill");
  const label = document.getElementById("composer-attachment-label");
  if (label) {
    label.textContent = attachedScreenshotLabel;
  }
  if (pill) {
    pill.hidden = false;
  }

  setCharacterState("success", "Bildschirmfoto als Kontext angefügt");
  scheduleStateTransition(() => {
    setCharacterState("idle");
  }, 1200);

  focusChatInput();
}

function clearScreenshotContext(): void {
  attachedScreenshotLabel = null;
  const pill = document.getElementById("composer-attachment-pill");
  if (pill) {
    pill.hidden = true;
  }
}

function focusChatInput(): void {
  if (currentStage !== "agent") {
    switchStage("agent");
  }
  const input = document.getElementById("agent-chat-input") as HTMLInputElement | null;
  if (input) {
    input.focus();
  }
}

// ============================================================================
// 4. Push-to-Talk Voice Interaction
// ============================================================================

function startVoiceRecording(): void {
  finishMorphImmediately();
  clearStateTimers();
  isVoiceRecording = true;
  voiceRecordStartedAt = performance.now();

  const voiceBtn = document.getElementById("btn-voice-ptt");
  voiceBtn?.classList.add("active");

  const input = document.getElementById("agent-chat-input") as HTMLInputElement | null;
  if (input) {
    input.placeholder = "Ich höre zu… Lass los, wenn du fertig bist.";
  }

  setCharacterState("listen", "Zuhören — Sprich jetzt mit AIDO…");
}

function stopVoiceRecording(): void {
  if (!isVoiceRecording) return;
  isVoiceRecording = false;

  const durationMs = Math.max(0, performance.now() - voiceRecordStartedAt);
  const voiceBtn = document.getElementById("btn-voice-ptt");
  voiceBtn?.classList.remove("active");

  const input = document.getElementById("agent-chat-input") as HTMLInputElement | null;
  if (input) {
    input.placeholder = "Nachricht an AIDO…";
  }

  if (durationMs < 220) {
    setCharacterState("idle");
    return;
  }

  const spokenText =
    input && input.value.trim().length > 0
      ? input.value.trim()
      : "Prüfe mein aktuelles Projekt und zeige mir, ob temporäre Dateien aufgeräumt werden können.";

  if (input) {
    input.value = "";
  }

  handleUserPrompt(spokenText);
}

// ============================================================================
// 5. ChatGPT-Clean Conversation, Reply Chips & Inline Security Approvals
// ============================================================================

function appendChatMessage(msg: ChatMessage): void {
  const feed = document.getElementById("agent-chat-feed");
  if (!feed) return;

  const wrapper = document.createElement("div");
  wrapper.className = `chat-msg ${msg.role}`;

  const roleSpan = document.createElement("span");
  roleSpan.className = "chat-msg-role";
  roleSpan.textContent = msg.role === "aido" ? "AIDO" : "Du";

  const textDiv = document.createElement("div");
  textDiv.className = "chat-msg-text";
  textDiv.textContent = msg.text;

  wrapper.appendChild(roleSpan);
  if (msg.attachmentLabel) {
    const att = document.createElement("div");
    att.style.fontSize = "0.76rem";
    att.style.opacity = "0.8";
    att.style.marginBottom = "2px";
    att.textContent = `Anhang: ${msg.attachmentLabel}`;
    wrapper.appendChild(att);
  }
  wrapper.appendChild(textDiv);

  feed.appendChild(wrapper);
}

function renderReplyChips(chips: string[]): void {
  const container = document.getElementById("agent-reply-chips");
  if (!container) return;

  container.innerHTML = "";
  for (const label of chips) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "reply-chip";
    btn.setAttribute("data-prompt", label);
    btn.textContent = label;
    btn.addEventListener("click", () => {
      handleUserPrompt(label);
    });
    container.appendChild(btn);
  }
}

function showInlineApproval(proposal: InlineApprovalProposal): void {
  const container = document.getElementById("inline-approval-container");
  if (!container) return;

  container.innerHTML = "";

  const card = document.createElement("div");
  card.className = "approval-card";

  const header = document.createElement("div");
  header.className = "approval-card-header";
  header.innerHTML = `<span>Freigabe erforderlich</span><span>Sicherheitsprüfung</span>`;

  const title = document.createElement("div");
  title.className = "approval-card-title";
  title.textContent = proposal.title;

  const reason = document.createElement("div");
  reason.className = "approval-card-reason";
  reason.textContent = `${proposal.reason} · ${proposal.itemsSummary}`;

  const actions = document.createElement("div");
  actions.className = "approval-actions";

  const approveBtn = document.createElement("button");
  approveBtn.type = "button";
  approveBtn.className = "btn-approve-solid";
  approveBtn.textContent = "Einmal erlauben";
  approveBtn.addEventListener("click", () => {
    container.innerHTML = "";
    proposal.onApprove();
  });

  const rejectBtn = document.createElement("button");
  rejectBtn.type = "button";
  rejectBtn.className = "btn-reject-outline";
  rejectBtn.textContent = "Ablehnen";
  rejectBtn.addEventListener("click", () => {
    container.innerHTML = "";
    proposal.onReject();
  });

  actions.appendChild(approveBtn);
  actions.appendChild(rejectBtn);

  card.appendChild(header);
  card.appendChild(title);
  card.appendChild(reason);
  card.appendChild(actions);

  container.appendChild(card);
}

function executeCleanupAction(fromOverview = false): void {
  cleanupCompleted = true;
  pendingTempFilesCount = 0;
  pendingTempSizeLabel = "0 KB";

  memoriesState.unshift({
    id: Date.now(),
    title: "Temporäre Dateien bereinigt (576 KB freigegeben)",
    details: "Fünf Zwischenspeicher-Dateien wurden nach deiner Freigabe sicher entfernt.",
    category: "Wartung"
  });

  renderOverviewDashboard();

  setCharacterState("work", "Arbeiten — Temporäre Dateien werden entfernt…");
  scheduleStateTransition(() => {
    setCharacterState("success", "Erfolg — 576 KB Speicherplatz freigegeben");
    if (!fromOverview) {
      appendChatMessage({
        id: `aido-${Date.now()}`,
        role: "aido",
        text: "Erledigt. Ich habe die 5 temporären Zwischenspeicher-Dateien (576 KB) sicher entfernt. Dein System ist jetzt aufgeräumt."
      });
      renderReplyChips([
        "Projektstatus prüfen",
        "Im Overview öffnen",
        "Handy steuern"
      ]);
    }
    scheduleStateTransition(() => {
      setCharacterState("idle");
    }, 1500);
  }, 550);
}

function handleUserPrompt(rawPrompt: string): void {
  const text = rawPrompt.trim();
  if (!text) return;

  finishMorphImmediately();
  clearStateTimers();

  if ( text === "Im Overview öffnen" ) {
    switchStage("overview");
    return;
  }
  if ( text === "Handy-Ansicht öffnen" ) {
    switchStage("phone");
    return;
  }

  const attachment = attachedScreenshotLabel || undefined;
  clearScreenshotContext();

  appendChatMessage({
    id: `user-${Date.now()}`,
    role: "user",
    text,
    attachmentLabel: attachment
  });

  const lower = text.toLowerCase();

  setCharacterState("think", "Nachdenken — AIDO analysiert deine Anfrage…");

  scheduleStateTransition(() => {
    setCharacterState("work", "Arbeiten — Antwort wird vorbereitet…");

    scheduleStateTransition(() => {
      // 1. Cleanup / Aufräumen / Löschen
      if (
        lower.includes("aufräumen") ||
        lower.includes("temporär") ||
        lower.includes("bereinigen") ||
        lower.includes("speicher") ||
        lower.includes("löschen")
      ) {
        if (cleanupCompleted) {
          setCharacterState("success", "Erfolg — System ist bereits aufgeräumt");
          appendChatMessage({
            id: `aido-${Date.now()}`,
            role: "aido",
            text: "Dein System ist bereits sauber. Aktuell liegen keine unnötigen temporären Dateien vor."
          });
          renderReplyChips(["Projektstatus prüfen", "Im Overview öffnen", "Handy steuern"]);
          scheduleStateTransition(() => setCharacterState("idle"), 1400);
          return;
        }

        setCharacterState("speak", "Rückfrage — Bitte bestätige die Bereinigung");
        appendChatMessage({
          id: `aido-${Date.now()}`,
          role: "aido",
          text: "Ich habe 5 nicht mehr benötigte Zwischenspeicher-Dateien (insgesamt 576 KB) gefunden. Da dabei Dateien entfernt werden, bitte ich dich kurz um deine Freigabe."
        });

        showInlineApproval({
          id: `cleanup-${Date.now()}`,
          title: "Temporäre Zwischenspeicher-Dateien entfernen",
          reason: "Gibt 576 KB Speicherplatz frei, ohne persönliche Daten oder Projekte zu verändern",
          itemsSummary: "5 temporäre Dateien im lokalen Zwischenspeicher",
          onApprove: () => {
            executeCleanupAction(false);
          },
          onReject: () => {
            setCharacterState("error", "Abgebrochen — Keine Dateien wurden verändert");
            appendChatMessage({
              id: `aido-${Date.now()}`,
              role: "aido",
              text: "Verstanden. Ich habe keine Dateien angerührt."
            });
            renderReplyChips(["Projektstatus prüfen", "Programme verwalten", "Handy steuern"]);
            scheduleStateTransition(() => setCharacterState("idle"), 1400);
          }
        });

        renderReplyChips(["Ja, bitte ausführen", "Im Overview öffnen", "Andere Lösung"]);
        scheduleStateTransition(() => setCharacterState("idle", "Wartet auf deine Freigabe"), 1100);
        return;
      }

      if (lower === "ja, bitte ausführen") {
        if (!cleanupCompleted) {
          const inlineApproval = document.getElementById("inline-approval-container");
          if (inlineApproval) inlineApproval.innerHTML = "";
          executeCleanupAction(false);
          return;
        }
      }

      // 2. Project status / Git / Tests
      if (
        lower.includes("projekt") ||
        lower.includes("status") ||
        lower.includes("test") ||
        lower.includes("git")
      ) {
        setCharacterState("speak");
        const contextNote = attachment
          ? "Dein angefügtes Bildschirmfoto habe ich ebenfalls berücksichtigt. "
          : "";
        appendChatMessage({
          id: `aido-${Date.now()}`,
          role: "aido",
          text: `${contextNote}Dein Hauptprojekt „AIDO — Desktop-Begleiter“ ist auf dem aktuellen Stand. Die 3D-Charakter-Steuerung, die Marken-Animation und alle drei Ansichten (Agent, Overview, Handy) sind bereit. Sollen wir noch temporäre Dateien prüfen oder Details im Overview ansehen?`
        });
        renderReplyChips([
          "Temporäre Dateien aufräumen",
          "Im Overview öffnen",
          "Programme verwalten",
          "Mehr erklären"
        ]);
        scheduleStateTransition(() => {
          setCharacterState("success");
          scheduleStateTransition(() => setCharacterState("idle"), 1100);
        }, 700);
        return;
      }

      // 3. Phone / Handy
      if (
        lower.includes("handy") ||
        lower.includes("smartphone") ||
        lower.includes("android") ||
        lower.includes("einstellungen")
      ) {
        setCharacterState("speak");
        appendChatMessage({
          id: `aido-${Date.now()}`,
          role: "aido",
          text: "Dein Android-Gerät (Medium_Phone_API_37.0) ist verbunden und bereit. Ich kann direkt die Einstellungen öffnen, GitHub aufrufen oder dich zur Handy-Steuerung wechseln lassen."
        });
        renderReplyChips([
          "Handy-Ansicht öffnen",
          "Projektstatus prüfen",
          "Temporäre Dateien aufräumen"
        ]);
        scheduleStateTransition(() => {
          setCharacterState("success");
          scheduleStateTransition(() => setCharacterState("idle"), 1100);
        }, 700);
        return;
      }

      // 4. Apps / Programme
      if (lower.includes("programm") || lower.includes("werkzeug") || lower.includes("app")) {
        setCharacterState("speak");
        const enabledCount = appsState.filter((a) => a.userEnabled).length;
        appendChatMessage({
          id: `aido-${Date.now()}`,
          role: "aido",
          text: `Aktuell sind ${enabledCount} von ${appsState.length} lokalen Werkzeugen für die Zusammenarbeit freigegeben (darunter VS Code, Windows Terminal und Browser). Du kannst jedes Programm jederzeit im Overview ein- oder ausschalten.`
        });
        renderReplyChips([
          "Im Overview öffnen",
          "Projektstatus prüfen",
          "Temporäre Dateien aufräumen"
        ]);
        scheduleStateTransition(() => {
          setCharacterState("success");
          scheduleStateTransition(() => setCharacterState("idle"), 1100);
        }, 700);
        return;
      }

      // 5. General / Mehr erklären
      setCharacterState("speak");
      const attachmentAck = attachment
        ? `Ich sehe dein Bildschirmfoto (${attachment}). `
        : "";
      appendChatMessage({
        id: `aido-${Date.now()}`,
        role: "aido",
        text: `${attachmentAck}Ich begleite dich direkt auf deinem Desktop: Du kannst mich anklicken zum Schreiben, gedrückt halten zum Sprechen, mit der mittleren Maustaste ein Bildschirmfoto anfügen oder per Rechtsklick das Schnellmenü öffnen. Womit möchten wir weitermachen?`
      });
      renderReplyChips([
        "Projektstatus prüfen",
        "Temporäre Dateien aufräumen",
        "Programme verwalten",
        "Handy steuern"
      ]);
      scheduleStateTransition(() => {
        setCharacterState("success");
        scheduleStateTransition(() => setCharacterState("idle"), 1100);
      }, 700);
    }, 380);
  }, 360);
}

// ============================================================================
// 6. Stage 2: Clean Overview Dashboard (#view-overview)
// ============================================================================

function renderOverviewDashboard(): void {
  // 1. Projects
  const projectsBadge = document.getElementById("projects-count-badge");
  if (projectsBadge) {
    projectsBadge.textContent = String(projectsState.length);
  }

  const projectsList = document.getElementById("overview-projects-list");
  if (projectsList) {
    projectsList.innerHTML = "";
    for (const proj of projectsState) {
      const row = document.createElement("div");
      row.className = "list-row";

      const main = document.createElement("div");
      main.className = "list-row-main";

      const title = document.createElement("span");
      title.className = "list-row-title";
      title.textContent = proj.title;

      const sub = document.createElement("span");
      sub.className = "list-row-sub";
      sub.textContent = `${proj.pathLabel} · ${proj.summary}`;

      const tagsRow = document.createElement("div");
      tagsRow.className = "list-row-tags";
      for (const t of proj.tags) {
        const tag = document.createElement("span");
        tag.className = "mono-tag";
        tag.textContent = t;
        tagsRow.appendChild(tag);
      }

      main.appendChild(title);
      main.appendChild(sub);
      main.appendChild(tagsRow);

      const actions = document.createElement("div");
      actions.style.display = "flex";
      actions.style.gap = "6px";
      actions.style.flexShrink = "0";

      const btnCheck = document.createElement("button");
      btnCheck.type = "button";
      btnCheck.className = "btn-pill";
      btnCheck.textContent = "Prüfen";
      btnCheck.addEventListener("click", () => {
        setCharacterState("success", `Geprüft — ${proj.title} ist fehlerfrei`);
        sub.textContent = `${proj.pathLabel} · Geprüft: ${proj.statusText}`;
        scheduleStateTransition(() => setCharacterState("idle"), 1500);
      });

      const btnDiscuss = document.createElement("button");
      btnDiscuss.type = "button";
      btnDiscuss.className = "btn-pill";
      btnDiscuss.textContent = "Im Chat besprechen";
      btnDiscuss.addEventListener("click", () => {
        switchStage("agent");
        handleUserPrompt(`Projektstatus für ${proj.title} prüfen`);
      });

      actions.appendChild(btnCheck);
      actions.appendChild(btnDiscuss);

      row.appendChild(main);
      row.appendChild(actions);
      projectsList.appendChild(row);
    }
  }

  // 2. Apps & Tools
  const appsBadge = document.getElementById("apps-count-badge");
  if (appsBadge) {
    const activeCount = appsState.filter((a) => a.userEnabled).length;
    appsBadge.textContent = `${activeCount} / ${appsState.length} aktiv`;
  }

  const appsList = document.getElementById("overview-apps-list");
  if (appsList) {
    appsList.innerHTML = "";
    for (const app of appsState) {
      const row = document.createElement("div");
      row.className = "list-row";

      const main = document.createElement("div");
      main.className = "list-row-main";

      const title = document.createElement("span");
      title.className = "list-row-title";
      title.textContent = app.name;

      const sub = document.createElement("span");
      sub.className = "list-row-sub";
      sub.textContent = `${app.category} · ${app.description}`;

      main.appendChild(title);
      main.appendChild(sub);

      const switchLabel = document.createElement("label");
      switchLabel.className = "mono-switch";

      const switchText = document.createElement("span");
      switchText.textContent = app.userEnabled ? "Aktiv" : "Inaktiv";

      const checkbox = document.createElement("input");
      checkbox.type = "checkbox";
      checkbox.checked = app.userEnabled;
      checkbox.setAttribute("aria-label", `${app.name} aktivieren`);
      checkbox.addEventListener("change", () => {
        app.userEnabled = checkbox.checked;
        renderOverviewDashboard();
        setCharacterState(
          "idle",
          `${app.name} ${app.userEnabled ? "für AIDO aktiviert" : "deaktiviert"}`
        );
      });

      switchLabel.appendChild(switchText);
      switchLabel.appendChild(checkbox);

      row.appendChild(main);
      row.appendChild(switchLabel);
      appsList.appendChild(row);
    }
  }

  // 3. System & Cleanup Summary
  const sysSummary = document.getElementById("overview-system-summary");
  if (sysSummary) {
    sysSummary.innerHTML = `
      <div class="list-row">
        <div class="list-row-main">
          <span class="list-row-title">Lokale KI im automatischen Ruhemodus</span>
          <span class="list-row-sub">0 GB Speicherverbrauch im Leerlauf · Sofort einsatzbereit bei Ansprache</span>
        </div>
        <span class="mono-tag">Sparsam</span>
      </div>
      <div class="list-row">
        <div class="list-row-main">
          <span class="list-row-title">Privatsphäre &amp; Sicherheit</span>
          <span class="list-row-sub">Alle Daten bleiben auf deinem Gerät · Wichtige Aktionen fragen immer um Erlaubnis</span>
        </div>
        <span class="mono-tag">Geschützt</span>
      </div>
    `;
  }

  const cleanupBox = document.getElementById("overview-cleanup-box");
  if (cleanupBox) {
    const titleEl = cleanupBox.querySelector(".list-row-title");
    const subEl = cleanupBox.querySelector(".list-row-sub");
    const btnEl = document.getElementById("btn-trigger-cleanup") as HTMLButtonElement | null;

    if (cleanupCompleted) {
      if (titleEl) titleEl.textContent = "Speicherplatz ist vollständig aufgeräumt";
      if (subEl) subEl.textContent = "0 temporäre Dateien offen · Zuletzt 576 KB sicher freigegeben.";
      if (btnEl) {
        btnEl.textContent = "Erneut prüfen";
      }
    } else {
      if (titleEl) titleEl.textContent = "Speicherplatz & temporäre Dateien";
      if (subEl) {
        subEl.textContent = `${pendingTempFilesCount} temporäre Zwischenspeicher-Dateien (${pendingTempSizeLabel}) können sicher entfernt werden.`;
      }
      if (btnEl) {
        btnEl.textContent = "Temporäre Dateien prüfen & aufräumen";
      }
    }
  }

  // 4. Memory & Tasks
  renderMemoryAndTasks();
}

function renderMemoryAndTasks(): void {
  const searchInput = document.getElementById("memory-search-input") as HTMLInputElement | null;
  const query = (searchInput?.value || "").trim().toLowerCase();

  const memoryList = document.getElementById("overview-memory-list");
  if (memoryList) {
    memoryList.innerHTML = "";
    const filtered = memoriesState.filter(
      (m) =>
        !query ||
        m.title.toLowerCase().includes(query) ||
        m.details.toLowerCase().includes(query) ||
        m.category.toLowerCase().includes(query)
    );

    if (filtered.length === 0) {
      const empty = document.createElement("div");
      empty.className = "list-row";
      empty.innerHTML = `<div class="list-row-main"><span class="list-row-sub">Keine passenden Einträge im Gedächtnis gefunden.</span></div>`;
      memoryList.appendChild(empty);
    } else {
      for (const mem of filtered) {
        const row = document.createElement("div");
        row.className = "list-row";

        const main = document.createElement("div");
        main.className = "list-row-main";

        const title = document.createElement("span");
        title.className = "list-row-title";
        title.textContent = mem.title;

        const sub = document.createElement("span");
        sub.className = "list-row-sub";
        sub.textContent = `${mem.category} · ${mem.details}`;

        main.appendChild(title);
        main.appendChild(sub);

        const forgetBtn = document.createElement("button");
        forgetBtn.type = "button";
        forgetBtn.className = "btn-pill";
        forgetBtn.textContent = "Vergessen";
        forgetBtn.addEventListener("click", () => {
          memoriesState = memoriesState.filter((item) => item.id !== mem.id);
          renderMemoryAndTasks();
          setCharacterState("idle", "Eintrag aus dem Gedächtnis entfernt");
        });

        row.appendChild(main);
        row.appendChild(forgetBtn);
        memoryList.appendChild(row);
      }
    }
  }

  const tasksList = document.getElementById("overview-tasks-list");
  if (tasksList) {
    tasksList.innerHTML = "";
    for (const task of tasksState) {
      const row = document.createElement("div");
      row.className = "list-row";
      row.innerHTML = `
        <div class="list-row-main">
          <span class="list-row-title">${task.title}</span>
          <span class="list-row-sub">${task.schedule}</span>
        </div>
        <span class="mono-tag">${task.statusLabel}</span>
      `;
      tasksList.appendChild(row);
    }
  }
}

// ============================================================================
// 7. Stage 3: Simplified Handy Connection & Control (#view-phone)
// ============================================================================

function renderPhoneStage(): void {
  const screenMeta = PHONE_SCREEN_META[currentPhoneScreen];

  const activeAppEl = document.getElementById("phone-active-app");
  if (activeAppEl) {
    const scrollSuffix = phoneScrollOffset > 0 ? ` · Ansicht +${phoneScrollOffset}` : "";
    const textSuffix = phoneLastTypedText ? ` · Eingabe: „${phoneLastTypedText}“` : "";
    activeAppEl.textContent = `Aktuelle Ansicht: ${screenMeta.title}${scrollSuffix}${textSuffix}`;
  }

  const nodesList = document.getElementById("phone-ui-nodes-list");
  if (nodesList) {
    nodesList.innerHTML = "";
    for (const node of screenMeta.nodes) {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = "phone-node-btn";
      btn.innerHTML = `
        <span><strong>${node.label}</strong><br/><small style="opacity:0.75">${node.hint}</small></span>
        <span>Tippen →</span>
      `;
      btn.addEventListener("click", () => {
        handlePhoneNodeTap(node);
      });
      nodesList.appendChild(btn);
    }
  }

  renderPhoneQrCode();
}

function setPhoneFeedback(message: string): void {
  const feedbackEl = document.getElementById("phone-action-feedback");
  if (feedbackEl) {
    feedbackEl.textContent = message;
  }
}

function showPhoneApproval(
  title: string,
  reason: string,
  onApprove: () => void
): void {
  const box = document.getElementById("phone-approval-box");
  if (!box) return;

  box.hidden = false;
  box.innerHTML = "";

  const header = document.createElement("div");
  header.className = "approval-card-header";
  header.innerHTML = `<span>Freigabe erforderlich</span><span>Smartphone-Schutz</span>`;

  const titleEl = document.createElement("div");
  titleEl.className = "approval-card-title";
  titleEl.textContent = title;

  const reasonEl = document.createElement("div");
  reasonEl.className = "approval-card-reason";
  reasonEl.textContent = reason;

  const actions = document.createElement("div");
  actions.className = "approval-actions";

  const allowBtn = document.createElement("button");
  allowBtn.type = "button";
  allowBtn.className = "btn-approve-solid";
  allowBtn.textContent = "Erlauben";
  allowBtn.addEventListener("click", () => {
    box.hidden = true;
    box.innerHTML = "";
    onApprove();
  });

  const denyBtn = document.createElement("button");
  denyBtn.type = "button";
  denyBtn.className = "btn-reject-outline";
  denyBtn.textContent = "Ablehnen";
  denyBtn.addEventListener("click", () => {
    box.hidden = true;
    box.innerHTML = "";
    setCharacterState("error", "Handy-Aktion abgelehnt");
    setPhoneFeedback("Aktion abgebrochen. Auf deinem Smartphone wurde nichts ausgelöst.");
    scheduleStateTransition(() => setCharacterState("idle"), 1300);
  });

  actions.appendChild(allowBtn);
  actions.appendChild(denyBtn);

  box.appendChild(header);
  box.appendChild(titleEl);
  box.appendChild(reasonEl);
  box.appendChild(actions);
}

function handlePhoneNodeTap(node: PhoneUiNode): void {
  if (node.requiresApproval) {
    setCharacterState("think", "Rückfrage — Freigabe für Smartphone-Aktion erforderlich");
    setPhoneFeedback(`Bitte bestätige: „${node.label}“`);
    showPhoneApproval(
      node.label,
      "Vor dem Absenden von Nachrichten oder sensiblen Aktionen fragt AIDO nach deiner Zustimmung.",
      () => {
        setCharacterState("work", "Arbeiten — Handy-Aktion wird ausgeführt…");
        scheduleStateTransition(() => {
          setCharacterState("success", "Erfolg — Handy-Aktion abgeschlossen");
          setPhoneFeedback(node.actionSummary);
          scheduleStateTransition(() => setCharacterState("idle"), 1300);
        }, 400);
      }
    );
    return;
  }

  setCharacterState("work", `Arbeiten — Tippe auf „${node.label}“…`);
  scheduleStateTransition(() => {
    if (node.targetScreen) {
      currentPhoneScreen = node.targetScreen;
      phoneScrollOffset = 0;
    }
    renderPhoneStage();
    setPhoneFeedback(node.actionSummary);
    setCharacterState("success", `Erfolg — ${node.label}`);
    scheduleStateTransition(() => setCharacterState("idle"), 1200);
  }, 320);
}

function executePhoneGoal(rawGoal: string): void {
  const goal = rawGoal.trim();
  if (!goal) return;

  const lower = goal.toLowerCase();

  // Sensitive action check
  if (
    lower.includes("bank") ||
    lower.includes("paypal") ||
    lower.includes("überweisung") ||
    lower.includes("kaufen") ||
    lower.includes("löschen") ||
    lower.includes("nachricht senden") ||
    lower.includes("whatsapp")
  ) {
    setCharacterState("think", "Freigabe erforderlich — Sensible Handy-Anweisung");
    setPhoneFeedback(`Sicherheitsprüfung für Anweisung: „${goal}“`);
    showPhoneApproval(
      `Anweisung ausführen: „${goal}“`,
      "Diese Aktion betrifft Nachrichten, Zahlungen oder persönliche Daten auf deinem Smartphone.",
      () => {
        setCharacterState("work", "Arbeiten — Anweisung wird auf dem Smartphone ausgeführt…");
        scheduleStateTransition(() => {
          setCharacterState("success", "Erfolg — Handy-Anweisung ausgeführt");
          setPhoneFeedback(`Nach deiner Freigabe ausgeführt: „${goal}“`);
          scheduleStateTransition(() => setCharacterState("idle"), 1300);
        }, 450);
      }
    );
    return;
  }

  setCharacterState("work", `Arbeiten — Sende Anweisung an Smartphone…`);

  scheduleStateTransition(() => {
    if (lower.includes("einstellung")) {
      currentPhoneScreen = "settings";
      phoneScrollOffset = 0;
      setPhoneFeedback("Einstellungen wurden auf dem Smartphone geöffnet.");
    } else if (lower.includes("netzwerk") || lower.includes("wlan")) {
      currentPhoneScreen = "network";
      phoneScrollOffset = 0;
      setPhoneFeedback("Netzwerk & Internet auf dem Smartphone geöffnet.");
    } else if (lower.includes("github") || lower.includes("browser")) {
      currentPhoneScreen = "github";
      phoneScrollOffset = 0;
      setPhoneFeedback("GitHub-Projektseite keokukzh/AIDO auf dem Smartphone geöffnet.");
    } else if (lower.includes("wische") || lower.includes("scroll")) {
      phoneScrollOffset += 1;
      setPhoneFeedback("Auf dem Smartphone-Bildschirm nach oben gewischt.");
    } else if (lower.startsWith("schreibe ") || lower.includes("hallo")) {
      const typed = goal.replace(/^schreibe\s+/i, "").trim() || "Hallo AIDO";
      phoneLastTypedText = typed;
      setPhoneFeedback(`Text „${typed}“ auf dem Smartphone eingegeben.`);
    } else if (lower.includes("startbildschirm") || lower.includes("home")) {
      currentPhoneScreen = "home";
      phoneScrollOffset = 0;
      phoneLastTypedText = "";
      setPhoneFeedback("Zurück zum Startbildschirm gewechselt.");
    } else {
      setPhoneFeedback(`Anweisung ausgeführt: „${goal}“`);
    }

    renderPhoneStage();
    setCharacterState("success", "Erfolg — Smartphone hat die Anweisung ausgeführt");
    scheduleStateTransition(() => setCharacterState("idle"), 1300);
  }, 380);
}

function renderPhoneQrCode(): void {
  const qrBox = document.getElementById("phone-qr-box");
  const urlEl = document.getElementById("phone-remote-url");
  if (!qrBox) return;

  const url = `http://192.168.178.42:5173/?remote=1&code=${qrTokenSeed}`;
  if (urlEl) {
    urlEl.textContent = url;
  }

  // Generate crisp 21x21 monochrome SVG QR matrix
  const size = 21;
  const cells: string[] = [];

  const isFinderPattern = (r: number, c: number): boolean => {
    const inTopLeft = r < 7 && c < 7;
    const inTopRight = r < 7 && c >= size - 7;
    const inBottomLeft = r >= size - 7 && c < 7;
    if (!inTopLeft && !inTopRight && !inBottomLeft) return false;

    const lr = inBottomLeft ? r - (size - 7) : r;
    const lc = inTopRight ? c - (size - 7) : c;

    if (lr === 0 || lr === 6 || lc === 0 || lc === 6) return true;
    if (lr >= 2 && lr <= 4 && lc >= 2 && lc <= 4) return true;
    return false;
  };

  const isFinderSeparator = (r: number, c: number): boolean => {
    if (r < 8 && c < 8) return true;
    if (r < 8 && c >= size - 8) return true;
    if (r >= size - 8 && c < 8) return true;
    return false;
  };

  for (let r = 0; r < size; r++) {
    for (let c = 0; c < size; c++) {
      let dark = false;
      if (isFinderPattern(r, c)) {
        dark = true;
      } else if (isFinderSeparator(r, c)) {
        dark = false;
      } else if (r === 6 || c === 6) {
        dark = (r + c) % 2 === 0;
      } else {
        const hash = ((r * 31 + c * 17 + qrTokenSeed) * 2654435761) >>> 0;
        dark = (hash & 3) === 0 || (hash & 5) === 1;
      }

      if (dark) {
        cells.push(`<rect x="${c}" y="${r}" width="1" height="1" fill="#000000" />`);
      }
    }
  }

  qrBox.innerHTML = `
    <svg viewBox="0 0 ${size} ${size}" shape-rendering="crispEdges" role="img" aria-label="QR-Code für Handy-Kopplung">
      <rect width="${size}" height="${size}" fill="#ffffff" />
      ${cells.join("")}
    </svg>
  `;
}

// ============================================================================
// 8. Initialization & Event Wiring
// ============================================================================

async function initApp(): Promise<void> {
  // 1. Initialize 3D Character on #mono-3d-canvas
  const canvas = document.getElementById("mono-3d-canvas") as HTMLCanvasElement | null;
  if (canvas) {
    monoViewer = new Mono3DViewer(canvas, "/assets/AIDO-MONO.glb");
    monoViewer.setTheme(currentTheme);

    // Wire all 5 Direct Character Mouse Gestures (Masterplan Chapter 10.2)
    monoViewer.setGestureCallbacks({
      onShortClick: () => {
        closeContextMenu();
        finishMorphImmediately();
        if (currentCharacterState === "sleep") {
          setCharacterState("wake", "Aufwachen — Hallo! Wie kann ich dir helfen?");
          scheduleStateTransition(() => setCharacterState("idle"), 1100);
        } else {
          setCharacterState("idle", "Bereit — Schreibe oder sprich direkt mit mir");
        }
        focusChatInput();
      },
      onHoldStart: () => {
        closeContextMenu();
        startVoiceRecording();
      },
      onHoldEnd: () => {
        stopVoiceRecording();
      },
      onMiddleClick: () => {
        closeContextMenu();
        attachScreenshotContext();
      },
      onRightClick: (e) => {
        openContextMenu(e.clientX, e.clientY);
      }
    });

    await monoViewer.load();
  }

  // Allow clicking the awakening SVG to transition immediately if desired
  const morphLayer = document.getElementById("morph-awakening-layer");
  morphLayer?.addEventListener("click", () => {
    finishMorphImmediately();
  });

  // 2. Set welcoming German message in #agent-chat-feed
  const feed = document.getElementById("agent-chat-feed");
  if (feed) {
    feed.innerHTML = "";
    appendChatMessage({
      id: "welcome-aido",
      role: "aido",
      text: "Hallo. Ich bin AIDO, dein persönlicher Desktop-Begleiter. Klicke mich direkt an, halte mich gedrückt zum Sprechen oder schreibe mir unten, wobei ich dir helfen soll."
    });
  }

  renderReplyChips([
    "Projektstatus prüfen",
    "Temporäre Dateien aufräumen",
    "Programme verwalten",
    "Handy steuern"
  ]);

  // 3. Bind 7-State Selector Pills & Replay Morph
  const stateButtons = document.querySelectorAll<HTMLButtonElement>(
    "#mono-state-selector [data-mono-state]"
  );
  stateButtons.forEach((btn) => {
    btn.addEventListener("click", () => {
      finishMorphImmediately();
      clearStateTimers();
      const targetState = (btn.getAttribute("data-mono-state") || "idle") as MonoState;
      setCharacterState(targetState);
    });
  });

  const replayMorphBtn = document.getElementById("btn-replay-morph");
  replayMorphBtn?.addEventListener("click", () => {
    playBrandMorphAwakening();
  });

  // 4. Bind Top Navigation Bar (Agent · Overview · Handy), Stop & Theme Toggle
  document.getElementById("btn-brand-home")?.addEventListener("click", () => {
    switchStage("agent");
  });
  document.getElementById("nav-stage-agent")?.addEventListener("click", () => {
    switchStage("agent");
  });
  document.getElementById("nav-stage-overview")?.addEventListener("click", () => {
    switchStage("overview");
  });
  document.getElementById("nav-stage-phone")?.addEventListener("click", () => {
    switchStage("phone");
  });

  document.getElementById("btn-theme-toggle")?.addEventListener("click", () => {
    toggleTheme();
  });

  document.getElementById("btn-emergency-stop")?.addEventListener("click", () => {
    triggerEmergencyStop();
  });

  // 5. Bind Chat Composer Controls
  const chatInput = document.getElementById("agent-chat-input") as HTMLInputElement | null;
  const sendBtn = document.getElementById("btn-send-chat");

  const submitCurrentChat = () => {
    if (!chatInput) return;
    const val = chatInput.value.trim();
    if (!val && !attachedScreenshotLabel) {
      chatInput.focus();
      return;
    }
    chatInput.value = "";
    handleUserPrompt(val || "Bitte analysiere dieses Bildschirmfoto.");
  };

  sendBtn?.addEventListener("click", submitCurrentChat);
  chatInput?.addEventListener("keydown", (e) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      submitCurrentChat();
    }
  });

  document.getElementById("btn-capture-screenshot")?.addEventListener("click", () => {
    attachScreenshotContext();
  });

  document.getElementById("btn-remove-attachment")?.addEventListener("click", () => {
    clearScreenshotContext();
  });

  const voicePttBtn = document.getElementById("btn-voice-ptt");
  voicePttBtn?.addEventListener("pointerdown", (e) => {
    e.preventDefault();
    if (!isVoiceRecording) {
      startVoiceRecording();
    } else {
      stopVoiceRecording();
    }
  });
  voicePttBtn?.addEventListener("pointerup", () => {
    if (isVoiceRecording && performance.now() - voiceRecordStartedAt >= 300) {
      stopVoiceRecording();
    }
  });

  // 6. Bind Right-Click Character Context Menu
  const ctxItems = document.querySelectorAll<HTMLButtonElement>(
    "#mono-context-menu [data-ctx-action]"
  );
  ctxItems.forEach((item) => {
    item.addEventListener("click", () => {
      const action = item.getAttribute("data-ctx-action");
      closeContextMenu();
      switch (action) {
        case "focus-chat":
          focusChatInput();
          break;
        case "voice-ptt":
          switchStage("agent");
          startVoiceRecording();
          window.setTimeout(() => {
            if (isVoiceRecording) stopVoiceRecording();
          }, 1800);
          break;
        case "screenshot":
          switchStage("agent");
          attachScreenshotContext();
          break;
        case "replay-morph":
          switchStage("agent");
          playBrandMorphAwakening();
          break;
        case "open-overview":
          switchStage("overview");
          break;
        case "open-phone":
          switchStage("phone");
          break;
        case "toggle-theme":
          toggleTheme();
          break;
      }
    });
  });

  window.addEventListener("click", (e) => {
    const menu = document.getElementById("mono-context-menu");
    if (menu && !menu.hidden && !menu.contains(e.target as Node)) {
      closeContextMenu();
    }
  });

  window.addEventListener("keydown", (e) => {
    if (e.key === "Escape") {
      closeContextMenu();
    }
  });

  // 7. Bind Stage 2 (Overview) Controls
  renderOverviewDashboard();

  document.getElementById("btn-scan-projects")?.addEventListener("click", () => {
    setCharacterState("work", "Projekte werden aktualisiert…");
    scheduleStateTransition(() => {
      renderOverviewDashboard();
      setCharacterState("success", "Alle Projekte sind aktuell");
      scheduleStateTransition(() => setCharacterState("idle"), 1200);
    }, 350);
  });

  document.getElementById("btn-trigger-cleanup")?.addEventListener("click", () => {
    if (cleanupCompleted) {
      setCharacterState("success", "Speicherplatz ist bereits sauber");
      scheduleStateTransition(() => setCharacterState("idle"), 1200);
      return;
    }
    executeCleanupAction(true);
  });

  document.getElementById("memory-search-input")?.addEventListener("input", () => {
    renderMemoryAndTasks();
  });

  // 8. Bind Stage 3 (Phone) Controls
  renderPhoneStage();

  const phoneGoalInput = document.getElementById("phone-goal-input") as HTMLInputElement | null;
  const runPhoneGoal = () => {
    if (!phoneGoalInput) return;
    const val = phoneGoalInput.value.trim();
    if (!val) {
      phoneGoalInput.focus();
      return;
    }
    phoneGoalInput.value = "";
    executePhoneGoal(val);
  };

  document.getElementById("btn-phone-run")?.addEventListener("click", runPhoneGoal);
  phoneGoalInput?.addEventListener("keydown", (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      runPhoneGoal();
    }
  });

  const phoneQuickChips = document.querySelectorAll<HTMLButtonElement>(
    "#phone-quick-chips [data-phone-goal]"
  );
  phoneQuickChips.forEach((chip) => {
    chip.addEventListener("click", () => {
      const goal = chip.getAttribute("data-phone-goal") || "";
      executePhoneGoal(goal);
    });
  });

  document.getElementById("btn-phone-observe")?.addEventListener("click", () => {
    setCharacterState("work", "Bildschirm des Smartphones wird gelesen…");
    scheduleStateTransition(() => {
      renderPhoneStage();
      const meta = PHONE_SCREEN_META[currentPhoneScreen];
      setPhoneFeedback(
        `Bildschirm aktualisiert: ${meta.title} (${meta.nodes.length} interaktive Elemente erkannt).`
      );
      setCharacterState("success", "Smartphone-Bildschirm gelesen");
      scheduleStateTransition(() => setCharacterState("idle"), 1100);
    }, 300);
  });

  document.getElementById("btn-phone-back")?.addEventListener("click", () => {
    currentPhoneScreen = currentPhoneScreen === "network" ? "settings" : "home";
    phoneScrollOffset = 0;
    renderPhoneStage();
    setPhoneFeedback("Taste „Zurück“ auf dem Smartphone ausgelöst.");
  });

  document.getElementById("btn-phone-home")?.addEventListener("click", () => {
    currentPhoneScreen = "home";
    phoneScrollOffset = 0;
    phoneLastTypedText = "";
    renderPhoneStage();
    setPhoneFeedback("Zum Startbildschirm des Smartphones gewechselt.");
  });

  document.getElementById("btn-regenerate-qr")?.addEventListener("click", () => {
    qrTokenSeed = Math.floor(100000 + Math.random() * 900000);
    renderPhoneQrCode();
    setPhoneFeedback("Neuer Verbindungs-Code für dein Smartphone wurde erstellt.");
  });

  document.getElementById("btn-phone-pair")?.addEventListener("click", () => {
    const ipInput = document.getElementById("phone-pair-ip") as HTMLInputElement | null;
    const codeInput = document.getElementById("phone-pair-code") as HTMLInputElement | null;
    const ip = ipInput?.value.trim() || "192.168.178.55:39123";
    const code = codeInput?.value.trim() || "842910";

    const badge = document.getElementById("phone-connection-badge");
    if (badge) {
      badge.textContent = "Verbunden (WLAN)";
    }
    setCharacterState("success", "Smartphone erfolgreich per WLAN gekoppelt");
    setPhoneFeedback(`Kabellose Verbindung mit ${ip} (Code ${code}) erfolgreich hergestellt.`);
    scheduleStateTransition(() => setCharacterState("idle"), 1400);
  });

  // 9. Play signature Brand-Morph Awakening sequence on startup
  playBrandMorphAwakening();
}

window.addEventListener("DOMContentLoaded", () => {
  void initApp();
});
