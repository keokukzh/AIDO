import { invoke } from "@tauri-apps/api/core";
import { DecisionCardRenderer, ProactiveProposalRecord } from "./DecisionCard";
import { TelemetryHud } from "./TelemetryHud";

export class CharacterGestures {
  private characterButton: HTMLButtonElement;
  private popover: HTMLElement;
  private cardsContainer: HTMLElement;
  private pttIndicator: HTMLElement;
  private screenshotContainer: HTMLElement;
  private screenshotThumb: HTMLImageElement;
  private screenshotDiscard: HTMLButtonElement;
  private contextMenu: HTMLElement;
  private hud: TelemetryHud;

  private holdTimer: number | null = null;
  private isHolding = false;
  private clickPrevented = false;
  public attachedScreenshotBase64: string | null = null;

  constructor(
    characterButton: HTMLButtonElement,
    popover: HTMLElement,
    pttIndicator: HTMLElement,
    screenshotContainer: HTMLElement,
    contextMenu: HTMLElement,
    hud: TelemetryHud
  ) {
    this.characterButton = characterButton;
    this.popover = popover;
    this.cardsContainer = popover.querySelector<HTMLElement>("#action-cards-container")!;
    this.pttIndicator = pttIndicator;
    this.screenshotContainer = screenshotContainer;
    this.screenshotThumb = screenshotContainer.querySelector<HTMLImageElement>("#screenshot-thumb")!;
    this.screenshotDiscard = screenshotContainer.querySelector<HTMLButtonElement>("#screenshot-discard")!;
    this.contextMenu = contextMenu;
    this.hud = hud;

    this.bindGestures();
  }

  private bindGestures(): void {
    // 1. Left Pointer Down & Hold (Push-to-Talk)
    this.characterButton.addEventListener("pointerdown", (event) => {
      if (event.button === 0) {
        // Left click
        this.clickPrevented = false;
        this.holdTimer = window.setTimeout(() => {
          this.isHolding = true;
          this.clickPrevented = true;
          this.startPushToTalk();
        }, 250);
      }
    });

    // Pointer Up
    window.addEventListener("pointerup", () => {
      if (this.holdTimer !== null) {
        clearTimeout(this.holdTimer);
        this.holdTimer = null;
      }
      if (this.isHolding) {
        this.isHolding = false;
        this.stopPushToTalk();
      }
    });

    // 2. Single Left Click: Toggle Zero-Prompt Decision Popover
    this.characterButton.addEventListener("click", () => {
      if (this.clickPrevented) {
        this.clickPrevented = false;
        return;
      }
      this.contextMenu.hidden = true;
      this.togglePopover();
    });

    // 3. Middle Mouse Click: Native Screen Capture
    this.characterButton.addEventListener("auxclick", (event) => {
      if (event.button === 1) {
        event.preventDefault();
        void this.captureScreen();
      }
    });

    // Also prevent default middle-click scroll
    this.characterButton.addEventListener("mousedown", (event) => {
      if (event.button === 1) {
        event.preventDefault();
      }
    });

    // Discard screenshot
    if (this.screenshotDiscard) {
      this.screenshotDiscard.addEventListener("click", () => {
        this.attachedScreenshotBase64 = null;
        this.screenshotContainer.hidden = true;
      });
    }

    // 4. Right Click: Context Menu
    this.characterButton.addEventListener("contextmenu", (event) => {
      event.preventDefault();
      this.popover.hidden = true;
      this.contextMenu.hidden = !this.contextMenu.hidden;
      if (!this.contextMenu.hidden) {
        this.contextMenu.querySelector("button")?.focus();
      }
    });

    // Context Menu Handlers
    this.contextMenu.addEventListener("click", (event) => {
      const btn = (event.target as HTMLElement).closest<HTMLButtonElement>("button");
      if (!btn) return;
      const action = btn.dataset.action;
      this.contextMenu.hidden = true;

      switch (action) {
        case "projects":
          void invoke("open_workspace_window", { target: "tools" });
          break;
        case "cleanup":
          void invoke("open_workspace_window", { target: "tools" });
          break;
        case "hud":
          this.hud.show();
          break;
        case "memory":
          void invoke("open_workspace_window", { target: "memory" });
          break;
        case "sleep":
          void invoke("unload_model_vram").then(() => this.hud.show());
          break;
        default:
          break;
      }
    });
  }

  public async togglePopover(): Promise<void> {
    if (this.popover.hidden) {
      this.popover.hidden = false;
      await this.loadActiveProposals();
    } else {
      this.popover.hidden = true;
    }
  }

  public async loadActiveProposals(): Promise<void> {
    this.cardsContainer.innerHTML = '<div class="loading-proposals">Prüfe System- & Projektstatus…</div>';
    try {
      const proposals = await invoke<ProactiveProposalRecord[]>("get_active_proposals");
      this.cardsContainer.innerHTML = "";
      if (proposals.length === 0) {
        this.cardsContainer.innerHTML =
          '<div class="no-proposals">🟢 Alles aktuell und sauber. Keine offenen Aktionen erforderlich.</div>';
        return;
      }

      proposals.forEach((p) => {
        const cardEl = DecisionCardRenderer.renderCard(p, () => {
          void this.loadActiveProposals();
        });
        this.cardsContainer.appendChild(cardEl);
      });
    } catch (err) {
      this.cardsContainer.innerHTML = `<div class="error-proposals">Fehler: ${String(err)}</div>`;
    }
  }

  private startPushToTalk(): void {
    this.pttIndicator.hidden = false;
    this.pttIndicator.classList.add("recording");
    window.dispatchEvent(new CustomEvent("aido-mono-state", { detail: "listening" }));
  }

  private stopPushToTalk(): void {
    this.pttIndicator.classList.remove("recording");
    this.pttIndicator.hidden = true;
    window.dispatchEvent(new CustomEvent("aido-mono-state", { detail: "thinking" }));
    setTimeout(() => {
      window.dispatchEvent(new CustomEvent("aido-mono-state", { detail: "idle" }));
    }, 2000);
  }

  private async captureScreen(): Promise<void> {
    try {
      window.dispatchEvent(new CustomEvent("aido-mono-state", { detail: "thinking" }));
      const resp = await invoke<{ base64?: string } | string>("capture_desktop_screen");
      const raw = typeof resp === "string" ? resp : (resp?.base64 || "");
      const fullSrc = raw.startsWith("data:") ? raw : `data:image/png;base64,${raw}`;
      this.attachedScreenshotBase64 = fullSrc;
      this.screenshotThumb.src = fullSrc;
      this.screenshotContainer.hidden = false;
      window.dispatchEvent(new CustomEvent("aido-mono-state", { detail: "success" }));
      setTimeout(() => {
        window.dispatchEvent(new CustomEvent("aido-mono-state", { detail: "idle" }));
      }, 1500);
    } catch {
      window.dispatchEvent(new CustomEvent("aido-mono-state", { detail: "error" }));
      setTimeout(() => {
        window.dispatchEvent(new CustomEvent("aido-mono-state", { detail: "idle" }));
      }, 1500);
    }
  }
}
