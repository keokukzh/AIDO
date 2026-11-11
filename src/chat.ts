import { invoke } from "@tauri-apps/api/core";

interface ChatMessage {
  role: "user" | "assistant";
  content: string;
}

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
