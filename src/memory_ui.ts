import { invoke } from "@tauri-apps/api/core";
import { el } from "./dom";

interface MemoryQuota {
  used_bytes: number;
  limit_bytes: number;
  used_percent: number;
  entry_count: number;
}

interface MemoryEntry {
  entry_id: number;
  layer: string;
  key: string;
  content: string;
  created_at: string;
  source: string;
}

interface MemoryOverviewResponse {
  quota: MemoryQuota;
  layer_counts: Record<string, number>;
  recent_entries: MemoryEntry[];
}

const statusEl = document.querySelector<HTMLElement>("#memory-status")!;
const quotaTextEl = document.querySelector<HTMLElement>("#quota-text")!;
const quotaBarEl = document.querySelector<HTMLElement>("#quota-bar-fill")!;
const layerBadgesEl = document.querySelector<HTMLElement>("#layer-badges")!;
const memoryListEl = document.querySelector<HTMLElement>("#memory-list")!;
const searchInput = document.querySelector<HTMLInputElement>("#memory-search")!;
const refreshBtn = document.querySelector<HTMLButtonElement>("#refresh")!;

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

function renderEntries(entries: MemoryEntry[]): void {
  if (!entries || entries.length === 0) {
    memoryListEl.replaceChildren(
      el("div", {
        className: "empty-notice",
        text: "Keine passenden Erinnerungen im aktiven Langzeitgedächtnis gefunden.",
      })
    );
    return;
  }

  memoryListEl.replaceChildren(
    ...entries.map((entry) => {
      const id = String(entry.entry_id);
      return el(
        "div",
        { className: "memory-card", attrs: { "data-id": id } },
        el(
          "div",
          { className: "memory-content" },
          el(
            "div",
            { className: "memory-meta" },
            el("span", { className: "memory-layer-tag", text: entry.layer }),
            el("span", { text: "•" }),
            el("span", { text: `Schlüssel: ${entry.key || "-"}` }),
            el("span", { text: "•" }),
            el("span", { text: new Date(entry.created_at).toLocaleString("de-DE") })
          ),
          el("div", { className: "memory-text", text: entry.content })
        ),
        el("button", {
          className: "forget-btn",
          text: "🗑️ Vergessen",
          attrs: {
            type: "button",
            "data-action": "forget",
            "data-id": id,
            title: "Aus dem Gedächtnis löschen",
          },
        })
      );
    })
  );
}

async function loadOverview(): Promise<void> {
  statusEl.textContent = "Lade Gedächtnis...";
  try {
    const data = await invoke<MemoryOverviewResponse>("memory_overview");
    statusEl.textContent = "";

    // Quota
    const used = formatBytes(data.quota.used_bytes);
    const limit = formatBytes(data.quota.limit_bytes);
    quotaTextEl.textContent = `${used} / ${limit} (${data.quota.used_percent.toFixed(1)} %) • ${data.quota.entry_count} Einträge`;
    quotaBarEl.style.width = `${Math.min(100, Math.max(1, data.quota.used_percent))}%`;

    // Layer Badges
    layerBadgesEl.replaceChildren(
      ...Object.entries(data.layer_counts).map(([layer, count]) =>
        el("span", { className: "layer-badge" }, `${layer}: `, el("b", { text: String(count) }))
      )
    );

    renderEntries(data.recent_entries);
  } catch (err) {
    statusEl.textContent = `Fehler beim Laden des Gedächtnisses: ${String(err)}`;
  }
}

memoryListEl.addEventListener("click", async (event) => {
  const target = event.target as HTMLElement;
  const btn = target.closest<HTMLButtonElement>("button[data-action='forget']");
  if (!btn) return;

  const id = Number(btn.dataset.id);
  if (!id) return;

  btn.disabled = true;
  btn.textContent = "Lösche...";

  try {
    const success = await invoke<boolean>("memory_forget", { entryId: id });
    if (success) {
      const card = btn.closest<HTMLElement>(".memory-card");
      if (card) {
        card.style.opacity = "0.3";
        setTimeout(() => card.remove(), 200);
      }
    } else {
      btn.disabled = false;
      btn.textContent = "Fehler";
    }
  } catch (err) {
    btn.disabled = false;
    btn.textContent = "Fehler";
    statusEl.textContent = `Fehler beim Vergessen: ${String(err)}`;
  }
});

let searchTimeout: number | undefined;
searchInput.addEventListener("input", () => {
  window.clearTimeout(searchTimeout);
  const q = searchInput.value.trim();

  if (!q) {
    void loadOverview();
    return;
  }

  searchTimeout = window.setTimeout(async () => {
    statusEl.textContent = "Suche...";
    try {
      const results = await invoke<MemoryEntry[]>("memory_search", { query: q });
      statusEl.textContent = `Suchergebnisse für "${q}": ${results.length} Treffer`;
      renderEntries(results);
    } catch (err) {
      statusEl.textContent = `Suchfehler: ${String(err)}`;
    }
  }, 250);
});

refreshBtn.addEventListener("click", () => void loadOverview());
window.addEventListener("aido-refresh-memory", () => void loadOverview());

void loadOverview();
