import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import awakeningMarkup from "./mono-awakening.svg?raw";
import { Mono3DViewer } from "./mono3d";
import { TelemetryHud } from "./TelemetryHud";
import { CharacterGestures } from "./CharacterGestures";

const animation = document.querySelector<HTMLElement>("#morph-awakening-layer")!;
const button = document.querySelector<HTMLButtonElement>("#open-chat")!;
const canvas = document.querySelector<HTMLCanvasElement>("#mono-canvas")!;
const fallback = document.querySelector<HTMLElement>("#mono-fallback")!;
const errorNotice = document.querySelector<HTMLElement>("#companion-error")!;
const moveGrip = document.querySelector<HTMLElement>("#move-mono")!;
const menu = document.querySelector<HTMLElement>("#mono-menu")!;

const popover = document.querySelector<HTMLElement>("#zero-prompt-popover")!;
const popoverClose = document.querySelector<HTMLButtonElement>("#popover-close")!;
const btnOpenFullChat = document.querySelector<HTMLButtonElement>("#btn-open-full-chat")!;
const hudContainer = document.querySelector<HTMLElement>("#telemetry-hud")!;
const pttIndicator = document.querySelector<HTMLElement>("#ptt-indicator")!;
const screenshotContainer = document.querySelector<HTMLElement>("#screenshot-preview-container")!;

fallback.innerHTML = awakeningMarkup;
fallback.querySelector("svg")!.setAttribute("viewBox", "292 42 176 225");

function showError(message: string): void {
  errorNotice.textContent = message;
  errorNotice.hidden = false;
}

const reducedMotion = matchMedia("(prefers-reduced-motion: reduce)").matches;
const viewer = new Mono3DViewer(canvas);
viewer.setTheme("dark");
viewer.setZoom(0.86);
viewer.toggleAutoRotate(!reducedMotion);

let animationTimers: number[] = [];
function awaken(): void {
  if (reducedMotion) return;
  animationTimers.forEach((timer) => window.clearTimeout(timer));
  animation.innerHTML = awakeningMarkup;
  animation.classList.remove("playing");
  void animation.offsetWidth;
  animation.classList.add("playing");
  viewer.setState("wake");
  animationTimers = [
    window.setTimeout(() => animation.classList.remove("playing"), 3750),
    window.setTimeout(() => viewer.setState("idle"), 4950),
  ];
}
awaken();
window.addEventListener("aido-awaken", awaken);

window.addEventListener("aido-mono-state", (event: any) => {
  if (event.detail) {
    viewer.setState(event.detail);
  }
});

void viewer.load().then((loaded) => button.classList.toggle("rendered", loaded));
canvas.addEventListener("webglcontextlost", () => {
  button.classList.remove("rendered");
  viewer.destroy();
});

// Initialize Telemetry HUD
const hud = new TelemetryHud(hudContainer);

// Initialize Mouse Gestures & Zero-Prompt Decision UI
const gestures = new CharacterGestures(
  button,
  popover,
  pttIndicator,
  screenshotContainer,
  menu,
  hud
);

if (popoverClose) {
  popoverClose.addEventListener("click", () => {
    popover.hidden = true;
  });
}

if (btnOpenFullChat) {
  btnOpenFullChat.addEventListener("click", () => {
    popover.hidden = true;
    void invoke("open_chat").catch(() => showError("Der AIDO-Chat konnte nicht geöffnet werden."));
  });
}

document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") {
    menu.hidden = true;
    popover.hidden = true;
    hudContainer.hidden = true;
  }
});

moveGrip.addEventListener("pointerdown", (event) => {
  if (event.button !== 0) return;
  event.preventDefault();
  errorNotice.hidden = true;
  void getCurrentWindow().startDragging().catch(() => showError("MONO konnte nicht verschoben werden."));
});

// Proactive Suggestions Engine Integration
const bubble = document.querySelector<HTMLElement>("#proactive-bubble")!;
const bubbleText = document.querySelector<HTMLElement>("#bubble-text")!;
const bubbleClose = document.querySelector<HTMLButtonElement>("#bubble-close")!;
const bubbleView = document.querySelector<HTMLButtonElement>("#bubble-view")!;
const bubbleLater = document.querySelector<HTMLButtonElement>("#bubble-later")!;

interface ProactiveSuggestion {
  id: string;
  category: string;
  title: string;
  human_message: string;
}

let activeSuggestion: ProactiveSuggestion | null = null;

async function checkProactive(): Promise<void> {
  try {
    const report = await invoke<{ suppressed: boolean; suggestions: ProactiveSuggestion[] }>("proactive_check");
    if (!report.suppressed && report.suggestions && report.suggestions.length > 0) {
      activeSuggestion = report.suggestions[0];
      bubbleText.textContent = activeSuggestion.human_message || activeSuggestion.title;
      bubble.hidden = false;
      viewer.setState("thinking");
      window.setTimeout(() => viewer.setState("idle"), 3000);
    } else {
      bubble.hidden = true;
    }
  } catch {
    // Proactive check is non-fatal
  }
}

bubbleClose.addEventListener("click", () => {
  bubble.hidden = true;
  if (activeSuggestion) {
    void invoke("proactive_dismiss", { suggestionId: activeSuggestion.id, muteCategory: null });
  }
});

bubbleLater.addEventListener("click", () => {
  bubble.hidden = true;
  if (activeSuggestion) {
    void invoke("proactive_dismiss", { suggestionId: activeSuggestion.id, muteCategory: null });
  }
});

bubbleView.addEventListener("click", () => {
  bubble.hidden = true;
  void gestures.togglePopover();
});

window.setTimeout(() => void checkProactive(), 4000);
window.setInterval(() => void checkProactive(), 60_000);
