import { invoke } from "@tauri-apps/api/core";

interface ChatMessage {
  role: "user" | "assistant";
  content: string;
}

interface Project { name: string; canonical_path: string }
interface LaunchProposal { id: string; project: string; path: string }

const history: ChatMessage[] = [];
const encoder = new TextEncoder();
const feed = document.querySelector<HTMLElement>("#conversation")!;
const empty = document.querySelector<HTMLElement>("#empty-chat")!;
const composer = document.querySelector<HTMLFormElement>("#composer")!;
const input = document.querySelector<HTMLTextAreaElement>("#prompt")!;
const send = document.querySelector<HTMLButtonElement>("#send")!;
const retry = document.querySelector<HTMLButtonElement>("#retry")!;
const errorBox = document.querySelector<HTMLElement>("#chat-error")!;
const errorMessage = document.querySelector<HTMLElement>("#error-message")!;
const status = document.querySelector<HTMLElement>("#chat-status")!;

let pending: { message: ChatMessage; element: HTMLElement } | undefined;
let busy = false;
let actionCard: HTMLElement | undefined;

function appendMessage(message: ChatMessage): HTMLElement {
  empty.hidden = true;
  const row = document.createElement("article");
  row.classList.add("message", message.role);
  const author = document.createElement("strong");
  author.textContent = message.role === "user" ? "Du" : "AIDO";
  const content = document.createElement("p");
  content.textContent = message.content;
  row.append(author, content);
  feed.append(row);
  feed.scrollTop = feed.scrollHeight;
  return row;
}

function contextFor(message: ChatMessage): ChatMessage[] {
  let bytes = encoder.encode(message.content).length;
  let start = history.length;
  while (start >= 2 && history.length - start < 14) {
    bytes += encoder.encode(history[start - 2].content).length;
    bytes += encoder.encode(history[start - 1].content).length;
    if (bytes > 6000) break;
    start -= 2;
  }
  return [...history.slice(start), message];
}

function projectQuery(content: string): string | undefined {
  return content.match(/^(?:bitte\s+)?(?:öffne|starte)\s+(?:(?:mein|das)\s+)?projekt\s+(.+?)[.!]?$/iu)?.[1]?.trim();
}

function appQuery(content: string): string | undefined {
  return content.match(/^(?:bitte\s+)?(?:starte|öffne)\s+(?:das\s+programm\s+)?([\p{L}][\p{L}\d ._-]{0,49}?)(?:\s+bitte)?[.!]?$/iu)?.[1]?.trim();
}

function actionButton(label: string, action: () => Promise<void>): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.textContent = label;
  button.addEventListener("click", async () => {
    const card = actionCard;
    if (!card) return;
    const buttons = card.querySelectorAll<HTMLButtonElement>("button");
    buttons.forEach((control) => { control.disabled = true; });
    try { await action(); }
    catch (error) {
      card.remove();
      appendMessage({ role: "assistant", content: String(error) });
    } finally {
      buttons.forEach((control) => { control.disabled = false; });
    }
  });
  return button;
}

function showAction(title: string, buttons: HTMLButtonElement[]): void {
  actionCard?.remove();
  const card = appendMessage({ role: "assistant", content: title });
  const controls = document.createElement("div");
  controls.className = "action-controls";
  controls.append(...buttons);
  card.append(controls);
  actionCard = card;
  feed.scrollTop = feed.scrollHeight;
}

async function showProposal(project: Project): Promise<void> {
  const proposal = await invoke<LaunchProposal>("prepare_project", { path: project.canonical_path });
  showAction(`Ich öffne «${proposal.project}» in Antigravity.\nOrdner: ${proposal.path}\nDarf ich das ausführen?`, [
    actionButton("Öffnen", () => decide(proposal, true)),
    actionButton("Ablehnen", () => decide(proposal, false)),
    actionButton("Ändern", async () => {
      await invoke("decide_project", { id: proposal.id, approve: false });
      await chooseProject();
    }),
  ]);
}

async function decide(proposal: LaunchProposal, approve: boolean): Promise<void> {
  const result = await invoke<string>("decide_project", { id: proposal.id, approve });
  actionCard?.remove();
  actionCard = undefined;
  appendMessage({ role: "assistant", content: result });
}

async function chooseProject(query?: string): Promise<void> {
  const projects = await invoke<Project[]>("workspace_projects");
  const matches = query ? projects.filter((project) => project.name.toLocaleLowerCase("de") === query.toLocaleLowerCase("de")) : projects;
  if (query && matches.length === 1) { await showProposal(matches[0]); return; }
  if (!matches.length) throw new Error(query ? `Kein freigegebenes Projekt namens «${query}» gefunden.` : "Keine freigegebenen Projekte gefunden.");
  showAction(query ? `Mehrere Projekte heissen «${query}». Welchen Ordner meinst du?` : "Welchen freigegebenen Projektordner soll ich öffnen?",
    matches.map((project) => actionButton(project.canonical_path, () => showProposal(project))));
}

async function ask(message: ChatMessage): Promise<void> {
  if (busy) return;
  if (pending?.message !== message) {
    pending?.element.remove();
    pending = { message, element: appendMessage(message) };
  }
  busy = true;
  send.disabled = true;
  errorBox.hidden = true;
  retry.hidden = true;
  status.textContent = "Warte auf das lokale Modell…";
  try {
    const project = projectQuery(message.content);
    if (project) {
      status.textContent = "Suche das Projekt im freigegebenen Katalog…";
      await chooseProject(project);
      pending = undefined;
      return;
    }
    const app = appQuery(message.content);
    if (app) {
      status.textContent = "Prüfe den lokalen Programmstart…";
      const proposal = await invoke<LaunchProposal>("prepare_app", { name: app });
      showAction(`Ich starte ${proposal.project}.\nProgramm: ${proposal.path}\nDarf ich das ausführen?`, [
        actionButton("Starten", () => decide(proposal, true)),
        actionButton("Ablehnen", () => decide(proposal, false)),
      ]);
      pending = undefined;
      return;
    }
    const content = await invoke<string>("chat", { messages: contextFor(message) });
    const answer: ChatMessage = { role: "assistant", content };
    history.push(message, answer);
    appendMessage(answer);
    pending = undefined;
  } catch (error) {
    errorMessage.textContent = typeof error === "string" ? error : "Der lokale Chat ist gerade nicht verfügbar.";
    errorBox.hidden = false;
    retry.hidden = false;
  } finally {
    busy = false;
    send.disabled = false;
    status.textContent = "";
    input.focus();
  }
}

composer.addEventListener("submit", (event) => {
  event.preventDefault();
  const content = input.value.trim();
  if (busy || !content) return;
  if (encoder.encode(content).length > 4000) {
    errorMessage.textContent = "Die Nachricht ist zu lang. Kürze sie und sende sie erneut.";
    errorBox.hidden = false;
    retry.hidden = true;
    return;
  }
  input.value = "";
  void ask({ role: "user", content });
});

input.addEventListener("keydown", (event) => {
  if (event.key === "Enter" && !event.shiftKey && !event.isComposing) {
    event.preventDefault();
    composer.requestSubmit();
  }
});

retry.addEventListener("click", () => {
  if (pending) void ask(pending.message);
});
