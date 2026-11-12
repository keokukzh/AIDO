import { invoke } from "@tauri-apps/api/core";

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
    memoryListEl.innerHTML = `<div class="empty-notice">Keine passenden Erinnerungen im aktiven Langzeitgedächtnis gefunden.</div>`;
    return;
  }

  memoryListEl.innerHTML = entries
    .map(
      (entry) => `
    <div class="memory-card" data-id="${entry.entry_id}">
      <div class="memory-content">
        <div class="memory-meta">
          <span class="memory-layer-tag">${entry.layer}</span>
          <span>•</span>
          <span>Schlüssel: ${entry.key || "-"}</span>
          <span>•</span>
          <span>${new Date(entry.created_at).toLocaleString("de-DE")}</span>
        </div>
        <div class="memory-text">${escapeHtml(entry.content)}</div>
      </div>
      <button type="button" class="forget-btn" data-action="forget" data-id="${entry.entry_id}" title="Aus dem Gedächtnis löschen">
        🗑️ Vergessen
      </button>
    </div>
  `
    )
    .join("");
}

function escapeHtml(text: string): string {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
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
    layerBadgesEl.innerHTML = Object.entries(data.layer_counts)
      .map(([layer, count]) => `<span class="layer-badge">${layer}: <b>${count}</b></span>`)
      .join("");

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
