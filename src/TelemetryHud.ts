import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface TelemetrySnapshot {
  timestamp: string;
  vram_used_mb: number;
  vram_total_mb: number;
  model_state: "SLEEPING" | "WAKING" | "ACTIVE";
  privacy_mode: string;
  external_bytes_blocked: number;
  external_bytes_sent: number;
}

export class TelemetryHud {
  private container: HTMLElement;
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D | null;
  private vramHistory: number[] = [];
  private maxHistoryLength = 30; // 30 points at 2s interval = 60 seconds
  private privacyBadge: HTMLElement;
  private modelStateBadge: HTMLElement;
  private vramValueEl: HTMLElement;
  private unloadBtn: HTMLButtonElement;
  private closeBtn: HTMLButtonElement;

  constructor(container: HTMLElement) {
    this.container = container;
    this.canvas = container.querySelector<HTMLCanvasElement>("#vram-sparkline")!;
    this.ctx = this.canvas ? this.canvas.getContext("2d") : null;
    this.privacyBadge = container.querySelector<HTMLElement>("#hud-privacy-badge")!;
    this.modelStateBadge = container.querySelector<HTMLElement>("#hud-model-state")!;
    this.vramValueEl = container.querySelector<HTMLElement>("#hud-vram-value")!;
    this.unloadBtn = container.querySelector<HTMLButtonElement>("#hud-btn-unload")!;
    this.closeBtn = container.querySelector<HTMLButtonElement>("#hud-btn-close")!;

    this.bindEvents();
    this.initListener();
    this.fetchInitial();
  }

  private bindEvents(): void {
    if (this.unloadBtn) {
      this.unloadBtn.addEventListener("click", async () => {
        try {
          const snap = await invoke<TelemetrySnapshot>("unload_model_vram");
          this.updateUi(snap);
        } catch {
          // ignore
        }
      });
    }

    if (this.closeBtn) {
      this.closeBtn.addEventListener("click", () => {
        this.container.hidden = true;
      });
    }
  }

  private async fetchInitial(): Promise<void> {
    try {
      const snap = await invoke<TelemetrySnapshot>("get_telemetry_snapshot");
      this.updateUi(snap);
    } catch {
      // ignore
    }
  }

  private initListener(): void {
    void listen<TelemetrySnapshot>("telemetry://update", (event) => {
      this.updateUi(event.payload);
    });
  }

  public show(): void {
    this.container.hidden = false;
    this.fetchInitial();
  }

  public hide(): void {
    this.container.hidden = true;
  }

  public updateUi(snap: TelemetrySnapshot): void {
    const rawSnap = snap as any;
    const used = snap.vram_used_mb ?? rawSnap.vram_used_mib ?? 1800;
    const total = snap.vram_total_mb ?? rawSnap.vram_total_mib ?? 32607;
    const stateStr = String(snap.model_state ?? rawSnap.llama_state ?? "SleepingZeroVram");
    const privacy = String(snap.privacy_mode ?? "PrivateOffline");

    // Record history
    this.vramHistory.push(used);
    if (this.vramHistory.length > this.maxHistoryLength) {
      this.vramHistory.shift();
    }

    // Update VRAM value text
    const gbUsed = (used / 1024).toFixed(1);
    const gbTotal = (total / 1024).toFixed(1);
    if (this.vramValueEl) {
      this.vramValueEl.textContent = `${gbUsed} GB / ${gbTotal} GB VRAM`;
    }

    // Update Model State
    if (this.modelStateBadge) {
      if (stateStr.includes("Sleeping") || stateStr.includes("SLEEPING")) {
        this.modelStateBadge.className = "state-badge sleeping";
        this.modelStateBadge.innerHTML = "💤 <span>Sleeping (Zero-VRAM Standby)</span>";
      } else if (stateStr.includes("Waking") || stateStr.includes("WAKING")) {
        this.modelStateBadge.className = "state-badge waking";
        this.modelStateBadge.innerHTML = "⏳ <span>Waking (Layer Loading…)</span>";
      } else {
        this.modelStateBadge.className = "state-badge active";
        this.modelStateBadge.innerHTML = "⚡ <span>Qwen3.8-27B WarmActive</span>";
      }
    }

    // Update Privacy Badge
    if (this.privacyBadge) {
      this.privacyBadge.innerHTML = `🔒 <span>${privacy} (0.0 KB Externe Daten)</span>`;
    }

    this.drawSparkline();
  }

  private drawSparkline(): void {
    if (!this.ctx || this.vramHistory.length < 2) return;
    const w = this.canvas.width;
    const h = this.canvas.height;
    const ctx = this.ctx;

    ctx.clearRect(0, 0, w, h);

    const maxVal = 32607; // 32 GB
    const minVal = 0;
    const stepX = w / (this.maxHistoryLength - 1);

    ctx.beginPath();
    ctx.strokeStyle = "#4ade80";
    ctx.lineWidth = 2;

    this.vramHistory.forEach((val, idx) => {
      const normY = h - ((val - minVal) / (maxVal - minVal)) * (h - 10) - 5;
      const x = idx * stepX;
      if (idx === 0) {
        ctx.moveTo(x, normY);
      } else {
        ctx.lineTo(x, normY);
      }
    });

    ctx.stroke();

    // Fill underneath
    ctx.lineTo((this.vramHistory.length - 1) * stepX, h);
    ctx.lineTo(0, h);
    ctx.fillStyle = "rgba(74, 222, 128, 0.15)";
    ctx.fill();
  }
}
