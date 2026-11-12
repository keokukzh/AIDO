import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import awakeningMarkup from "./mono-awakening.svg?raw";
import { Mono3DViewer } from "./mono3d";

const animation = document.querySelector<HTMLElement>("#morph-awakening-layer")!;
const button = document.querySelector<HTMLButtonElement>("#open-chat")!;
const canvas = document.querySelector<HTMLCanvasElement>("#mono-canvas")!;
const fallback = document.querySelector<HTMLElement>("#mono-fallback")!;
const errorNotice = document.querySelector<HTMLElement>("#companion-error")!;
const moveGrip = document.querySelector<HTMLElement>("#move-mono")!;
const menu = document.querySelector<HTMLElement>("#mono-menu")!;

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
void viewer.load().then((loaded) => button.classList.toggle("rendered", loaded));
canvas.addEventListener("webglcontextlost", () => {
  button.classList.remove("rendered");
  viewer.destroy();
});

button.addEventListener("click", () => {
  menu.hidden = true;
  animation.classList.remove("playing");
  errorNotice.hidden = true;
  void invoke("open_chat").catch(() => showError("Der AIDO-Chat konnte nicht geöffnet werden."));
});

button.addEventListener("contextmenu", (event) => {
  event.preventDefault();
  animation.classList.remove("playing");
  menu.hidden = !menu.hidden;
  if (!menu.hidden) menu.querySelector("button")?.focus();
});

menu.addEventListener("click", (event) => {
  const target = (event.target as HTMLElement).closest<HTMLButtonElement>("button")?.dataset.target;
  if (!target) return;
  menu.hidden = true;
  void invoke("open_workspace_window", { target }).catch((error: unknown) => showError(String(error)));
});
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape") menu.hidden = true;
});

moveGrip.addEventListener("pointerdown", (event) => {
  if (event.button !== 0) return;
  event.preventDefault();
  errorNotice.hidden = true;
  void getCurrentWindow().startDragging().catch(() => showError("MONO konnte nicht verschoben werden."));
});
