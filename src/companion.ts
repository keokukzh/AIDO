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

fallback.innerHTML = awakeningMarkup;
fallback.querySelector("svg")!.setAttribute("viewBox", "292 42 176 225");

function showError(message: string): void {
  errorNotice.textContent = message;
  errorNotice.hidden = false;
}

const reducedMotion = matchMedia("(prefers-reduced-motion: reduce)").matches;
if (!reducedMotion) {
  animation.innerHTML = awakeningMarkup;
  animation.classList.add("playing");
}
const viewer = new Mono3DViewer(canvas);
viewer.setTheme("dark");
viewer.setZoom(0.86);
viewer.toggleAutoRotate(!reducedMotion);
if (!reducedMotion) {
  viewer.setState("wake");
  window.setTimeout(() => animation.classList.remove("playing"), 3750);
  window.setTimeout(() => viewer.setState("idle"), 4950);
}
void viewer.load().then((loaded) => button.classList.toggle("rendered", loaded));
canvas.addEventListener("webglcontextlost", () => {
  button.classList.remove("rendered");
  viewer.destroy();
});

button.addEventListener("click", () => {
  animation.classList.remove("playing");
  errorNotice.hidden = true;
  void invoke("open_chat").catch(() => showError("Der AIDO-Chat konnte nicht geöffnet werden."));
});

moveGrip.addEventListener("pointerdown", (event) => {
  if (event.button !== 0) return;
  event.preventDefault();
  errorNotice.hidden = true;
  void getCurrentWindow().startDragging().catch(() => showError("MONO konnte nicht verschoben werden."));
});
