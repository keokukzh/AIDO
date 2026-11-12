import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

type Backend = "Windows" | "Wsl";
interface ModelProfile { backend: Backend; executable: string; model: string; distro: string; user: string }
interface Draft {
  step: number;
  completed: boolean;
  access_mode: string;
  privacy_mode: string;
  project_roots: string[];
  recent_metadata: boolean;
  model_profile: ModelProfile;
  rea_enabled: boolean;
  antigravity_enabled: boolean;
  blender_enabled: boolean;
}
interface SetupState { draft: Draft; suggested_project: string | null; antigravity: boolean; blender: boolean }
interface FinishResult { completed: boolean; projects: number; apps: number; model: string; gpu: string | null; rea: string | null }

const progress = document.querySelector<HTMLElement>("#step-progress")!;
const content = document.querySelector<HTMLElement>("#setup-content")!;
const status = document.querySelector<HTMLElement>("#setup-status")!;
const actions = document.querySelector<HTMLElement>("#setup-actions")!;
const back = document.querySelector<HTMLButtonElement>("#back")!;
const next = document.querySelector<HTMLButtonElement>("#next")!;
let state: SetupState;
let busy = false;
const profiles: Partial<Record<Backend, ModelProfile>> = {};

function el<K extends keyof HTMLElementTagNameMap>(tag: K, text = "", className = ""): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  node.textContent = text;
  if (className) node.className = className;
  return node;
}

function select(label: string, value: string, options: [string, string][], set: (value: string) => void): HTMLLabelElement {
  const wrapper = el("label", label, "field");
  const field = el("select");
  for (const [id, name] of options) { const option = el("option", name); option.value = id; field.append(option); }
  field.value = value;
  field.addEventListener("change", () => set(field.value));
  wrapper.append(field);
  return wrapper;
}

function checkbox(label: string, checked: boolean, set: (value: boolean) => void): HTMLLabelElement {
  const wrapper = el("label", "", "option");
  const field = el("input");
  field.type = "checkbox";
  field.checked = checked;
  field.addEventListener("change", () => set(field.checked));
  wrapper.append(field, el("span", label));
  return wrapper;
}

function button(label: string, action: () => void): HTMLButtonElement {
  const control = el("button", label, "secondary");
  control.type = "button";
  control.addEventListener("click", action);
  return control;
}

function textField(label: string, value: string, set: (value: string) => void, picker?: string): HTMLLabelElement {
  const wrapper = el("label", label, "field");
  const row = el("span", "", "row");
  const field = el("input");
  field.type = "text";
  field.value = value;
  field.addEventListener("input", () => set(field.value.trim()));
  row.append(field);
  if (picker) row.append(button("Auswählen", () => void run(async () => {
    const path = await invoke<string | null>("setup_pick", { kind: picker });
    if (path) { set(path); field.value = path; await saveDraft(); }
  })));
  wrapper.append(row);
  return wrapper;
}

function setBusy(value: boolean): void {
  busy = value;
  next.disabled = value;
  back.disabled = value || state?.draft.step === 0;
  content.querySelectorAll<HTMLButtonElement>("button").forEach((control) => { control.disabled = value; });
}

async function run(action: () => Promise<void>): Promise<void> {
  if (busy) return;
  setBusy(true);
  try { await action(); } catch (error) { status.textContent = String(error); }
  finally { setBusy(false); }
}

async function saveDraft(): Promise<void> {
  await invoke("save_setup", { draft: state.draft, finish: false });
}

function modelStep(draft: Draft): void {
  const profile = draft.model_profile;
  content.append(el("h2", "Lokales Modell"), el("p", "Wähle die Laufzeit, die AIDO für das vorhandene GGUF verwenden soll."));
  content.append(select("Laufzeit", profile.backend, [["Windows", "Windows"], ["Wsl", "Ubuntu in WSL2"]], (value) => {
    const backend = value as Backend;
    profiles[profile.backend] = profile;
    draft.model_profile = profiles[backend] ?? { backend, distro: backend === "Wsl" ? "Ubuntu" : "", user: "", executable: "", model: "" };
    render();
  }));
  if (profile.backend === "Wsl") {
    content.append(button("WSL2-Profil erkennen", () => void run(async () => {
      status.textContent = "Prüfe das lokale Ubuntu-Profil…";
      draft.model_profile = await invoke<ModelProfile>("suggest_wsl_profile");
      await saveDraft();
      render();
      status.textContent = "WSL-Pfade vorgeschlagen. Prüfe die Laufzeit vor dem Abschluss.";
    })));
    content.append(textField("Distribution", profile.distro, (value) => { profile.distro = value; }));
    content.append(textField("Linux-Benutzer", profile.user, (value) => { profile.user = value; }));
  }
  const native = profile.backend === "Windows";
  content.append(textField(native ? "llama-server.exe" : "Linux-Pfad zu llama-server", profile.executable,
    (value) => { profile.executable = value; }, native ? "runtime" : undefined));
  content.append(textField(native ? "GGUF-Modelldatei" : "Linux-Pfad zum GGUF", profile.model,
    (value) => { profile.model = value; }, native ? "model" : undefined));
  content.append(button("Laufzeit prüfen", () => void run(async () => {
    status.textContent = "Prüfe Modell und Laufzeit…";
    status.textContent = await invoke<string>("setup_model_check", { profile });
    await saveDraft();
  })));
}

function render(): void {
  const { draft, suggested_project, antigravity, blender } = state;
  content.replaceChildren();
  status.textContent = "";
  actions.hidden = false;
  progress.textContent = `Schritt ${draft.step + 1} von 4`;
  back.disabled = draft.step === 0;
  next.textContent = draft.step === 3 ? "AIDO starten" : "Weiter";
  if (draft.step === 0) {
    content.append(el("h2", "Datenschutz und Zugriffe"));
    content.append(select("Betriebsmodus", draft.privacy_mode, [
      ["PrivateOffline", "Private Offline"], ["LocalConnected", "Lokal verbunden"], ["HybridAI", "Hybrid AI"],
    ], (value) => { draft.privacy_mode = value; }));
    content.append(el("p", "Der Chat verwendet die lokale Laufzeit. Externe Dienste werden separat verbunden.", "detail"));
    content.append(select("Windows-Zugriff", draft.access_mode, [
      ["ReadOnly", "Nur lesen"], ["StandardAccess", "Standard Access"], ["FullAccess", "Full Access"],
    ], (value) => { draft.access_mode = value; }));
    content.append(el("p", "AIDO zeigt vor einer Aktion eine Vorschau und fragt dich um Freigabe. Full Access startet AIDO ohne dauerhafte Administratorrechte.", "detail"));
  } else if (draft.step === 1) {
    content.append(el("h2", "Projektordner"), el("p", "Wähle die Ordner, in denen AIDO nach Projekten suchen darf."));
    const list = el("ul");
    for (const path of draft.project_roots) {
      const item = el("li", path + " ");
      item.append(button("Entfernen", () => void run(async () => {
        draft.project_roots = draft.project_roots.filter((entry) => entry !== path);
        await saveDraft();
        render();
      })));
      list.append(item);
    }
    content.append(list);
    const add = (path: string) => { if (!draft.project_roots.includes(path)) draft.project_roots.push(path); };
    content.append(button("Ordner auswählen", () => void run(async () => {
      const path = await invoke<string | null>("setup_pick", { kind: "folder" });
      if (path) { add(path); await saveDraft(); render(); }
    })));
    if (suggested_project && !draft.project_roots.includes(suggested_project)) {
      content.append(button("AIDO-Projekt hinzufügen", () => void run(async () => {
        add(suggested_project); await saveDraft(); render();
      })));
    }
    content.append(el("p", antigravity ? "Antigravity wurde gefunden." : "Antigravity wurde noch nicht gefunden.", "detail"));
    content.append(checkbox("Ordner mit Änderungen in den letzten 30 Tagen anzeigen", draft.recent_metadata, (value) => { draft.recent_metadata = value; }));
  } else if (draft.step === 2) {
    modelStep(draft);
  } else {
    content.append(el("h2", "Werkzeuge und Abschluss"));
    content.append(checkbox("Mitgeliefertes REA für lesende Programmanalysen aktivieren", draft.rea_enabled, (value) => { draft.rea_enabled = value; }));
    if (antigravity) content.append(checkbox("Antigravity · freigegebene Projekte öffnen", draft.antigravity_enabled,
      (value) => { draft.antigravity_enabled = value; }));
    if (blender) content.append(checkbox("Blender · Programm nach Freigabe öffnen", draft.blender_enabled,
      (value) => { draft.blender_enabled = value; }));
    content.append(el("p", "AIDO prüft Hardware, Modell, bekannte Apps und die ausgewählten Ordner. Danach erscheint MONO auf dem Desktop.", "detail"));
  }
}

back.addEventListener("click", () => void run(async () => {
  state.draft.step--;
  await saveDraft();
  render();
}));
next.addEventListener("click", () => void run(async () => {
  if (state.draft.step < 3) {
    if (state.draft.step === 2) {
      status.textContent = "Prüfe Modell und Laufzeit…";
      await invoke("setup_model_check", { profile: state.draft.model_profile });
    }
    state.draft.step++;
    await saveDraft();
    render();
    return;
  }
  status.textContent = "AIDO prüft deine Einrichtung…";
  const result = await invoke<FinishResult>("save_setup", { draft: state.draft, finish: true });
  actions.hidden = true;
  progress.textContent = "";
  status.textContent = "";
  content.replaceChildren(el("h2", "MONO ist bereit"), el("p", `${result.projects} Projekte und ${result.apps} Programme erkannt.`),
    el("p", result.model), ...(result.gpu ? [el("p", result.gpu)] : []),
    ...(result.rea ? [el("p", result.rea)] : []),
    button("Einrichtung schliessen", () => void getCurrentWindow().close()));
}));

async function load(): Promise<void> {
  state = await invoke<SetupState>("workspace_status");
  render();
}

window.addEventListener("aido-refresh-setup", () => void run(load));
void run(load);
