import { invoke } from "@tauri-apps/api/core";
import { DecisionCardRenderer, ProactiveProposalRecord } from "./DecisionCard";

interface Inventory {
  rea: { ready: boolean; message: string; checked: string | null };
  antigravity: boolean;
  antigravity_enabled: boolean;
  blender: boolean;
  blender_enabled: boolean;
  apps: { name: string; installed: boolean; enabled: boolean; origin: string; capabilities: { title: string; status: string }[] }[];
  skills: { title: string; version: number; status: string }[];
  recent_projects: { name: string; path: string }[];
}

interface ReaReport {
  evidence_id: string;
  subject: { name: string; architecture?: string; digest: { sha256: string } };
  normalized_result?: { coverage?: { resources: number } };
  limitations?: string[];
}

const content = document.querySelector<HTMLElement>("#tools-content")!;
const status = document.querySelector<HTMLElement>("#tools-status")!;
const refresh = document.querySelector<HTMLButtonElement>("#refresh")!;

function element<K extends keyof HTMLElementTagNameMap>(tag: K, text: string): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  node.textContent = text;
  return node;
}

function panel(title: string, ...children: HTMLElement[]): HTMLElement {
  const node = document.createElement("section");
  node.className = "panel";
  node.append(element("h2", title), ...children);
  return node;
}

function list(items: string[], empty: string): HTMLElement {
  if (!items.length) return element("p", empty);
  const node = document.createElement("ul");
  for (const item of items) node.append(element("li", item));
  return node;
}

async function load(): Promise<void> {
  refresh.disabled = true;
  status.textContent = "Prüfe lokale Werkzeuge…";
  try {
    const inventory = await invoke<Inventory>("tools_inventory");
    const choose = element("button", "Programmdatei prüfen");
    choose.type = "button";
    choose.disabled = !inventory.rea.ready;
    const report = document.createElement("div");
    choose.addEventListener("click", async () => {
      choose.disabled = true;
      status.textContent = "REA prüft die gewählte Datei…";
      report.replaceChildren();
      try {
        const result = await invoke<ReaReport | null>("inspect_rea");
        if (result) {
          report.append(element("p", `${result.subject.name} · ${result.subject.architecture ?? "Architektur unbekannt"}`),
            element("p", `Ressourcen: ${result.normalized_result?.coverage?.resources ?? "unbekannt"}`),
            element("p", `SHA-256 ${result.subject.digest.sha256}`),
            element("p", `Beleg ${result.evidence_id}`),
            list(result.limitations ?? [], "Lesende Analyse der ausgewählten Datei."));
        }
        status.textContent = "";
      } catch (error) { status.textContent = String(error); }
      finally { choose.disabled = false; }
    });
    const rea = panel("Reverse Engineer Anything", element("p", inventory.rea.message),
      element("p", inventory.rea.checked ? `Letzter lokaler Versionscheck ${new Date(inventory.rea.checked).toLocaleString("de-CH")}` : "Lokaler Versionscheck ausstehend."),
      choose, report);
    const apps = panel("Apps und Adapter", element("p", "Registereinträge zeigen vorhandene Bausteine. AIDO weist ausführbare Desktop-Funktionen separat aus."),
      element("p", `Antigravity · ${inventory.antigravity ? "erkannt" : "fehlt"} · Projektöffnung ${inventory.antigravity_enabled ? "aktiviert" : "deaktiviert"}`),
      element("p", `Blender · ${inventory.blender ? "erkannt" : "fehlt"} · Start ${inventory.blender_enabled ? "nach Freigabe" : "deaktiviert"}`));
    for (const app of inventory.apps) {
      const details = document.createElement("details");
      details.append(element("summary", `${app.name} · ${app.installed ? "vorhanden" : "am erfassten Pfad fehlt"} · ${app.origin === "Official" ? "mitgelieferter Registereintrag" : "generierter Registereintrag"}`),
        list(app.capabilities.map((capability) => `${capability.title} · Registerstatus: ${capability.status}`), "Keine Fähigkeiten erfasst."));
      apps.append(details);
    }
    const projects = panel(
      "Projektordner · letzte 30 Tage",
      element("p", "Ein Start öffnet nur das genannte lokale Projekt und braucht eine einmalige Freigabe."),
      list(inventory.recent_projects.map((project) => `${project.name} · ${project.path}`), "Keine kürzlich geänderten freigegebenen Ordner erfasst.")
    );
    const gateHost = document.createElement("div");
    gateHost.className = "gate-host";
    for (const project of inventory.recent_projects) {
      if (!project.name.trim()) continue;
      const start = element("button", `«${project.name}» nach Freigabe starten`);
      start.type = "button";
      start.className = "secondary";
      start.addEventListener("click", () => {
        void requestProjectLaunch(project.name, gateHost, status);
      });
      projects.append(start);
    }
    projects.append(gateHost);
    content.replaceChildren(rea, apps,
      panel("Gespeicherte Skills", element("p", "Diese Abläufe sind im lokalen Gedächtnis gespeichert. Der Desktop führt sie derzeit nicht automatisch aus."),
        list(inventory.skills.map((skill) => `${skill.title} · v${skill.version} · ${skill.status === "active" ? "aktiv im Register" : skill.status}`), "Keine Skills gespeichert.")),
      projects);
    status.textContent = "";
  } catch (error) { status.textContent = String(error); }
  finally { refresh.disabled = false; }
}

async function requestProjectLaunch(
  projectName: string,
  host: HTMLElement,
  statusLine: HTMLElement
): Promise<void> {
  host.replaceChildren();
  statusLine.textContent = "Vorschau für den Projektstart wird erstellt…";
  try {
    const proposal = await invoke<ProactiveProposalRecord>("launch_project_orchestrated", {
      projectName,
      mode: "antigravity",
    });
    if (proposal.phase !== "proposal" || !proposal.id) {
      throw new Error("Die Vorschau ist unvollständig.");
    }
    const card = DecisionCardRenderer.renderCard(
      proposal,
      () => host.replaceChildren(),
      (result) => {
        const executed = (result ?? {}) as { message?: string };
        statusLine.textContent = executed.message ?? "Projektstart wurde ausgelöst.";
      }
    );
    host.replaceChildren(card);
    statusLine.textContent = "Bitte prüfe die Vorschau und entscheide dann.";
  } catch (error) {
    statusLine.textContent = String(error);
  }
}

refresh.addEventListener("click", () => void load());
window.addEventListener("aido-refresh-tools", () => void load());
void load();
