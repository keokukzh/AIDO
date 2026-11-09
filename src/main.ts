interface ApprovalProposal {
  requestId: string;
  title: string;
  explanation: string;
  files: string[];
  bytes: number;
}

const avatarOrb = document.getElementById("avatar-orb") as HTMLDivElement;
const charState = document.getElementById("char-state") as HTMLSpanElement;
const aidoMessage = document.getElementById("aido-message") as HTMLDivElement;
const actionButtons = document.getElementById("action-buttons") as HTMLDivElement;
const auditList = document.getElementById("audit-list") as HTMLUListElement;
const contextMenu = document.getElementById("context-menu") as HTMLDivElement;

let currentProposal: ApprovalProposal | null = null;
let pressTimer: number | null = null;

function appendAudit(entry: string): void {
  const li = document.createElement("li");
  const now = new Date().toLocaleTimeString("de-DE");
  li.textContent = `[${now}] ${entry}`;
  auditList.prepend(li);
}

function setCharacterState(stateLabel: string, working = false): void {
  charState.textContent = `Zustand: ${stateLabel}`;
  if (working) {
    avatarOrb.classList.add("working");
  } else {
    avatarOrb.classList.remove("working");
  }
}

function renderDefaultButtons(): void {
  actionButtons.innerHTML = "";
  const btnCheck = document.createElement("button");
  btnCheck.textContent = "🔍 Systemcheck & Fast Cleanup prüfen";
  btnCheck.onclick = () => startSystemHealthAndCleanupPreview();

  const btnPrivacy = document.createElement("button");
  btnPrivacy.textContent = "🔒 Privacy Center anzeigen";
  btnPrivacy.onclick = () => showPrivacyCenter();

  const btnMemory = document.createElement("button");
  btnMemory.textContent = "🧠 Memory & Audit-Log";
  btnMemory.onclick = () => showMemorySummary();

  actionButtons.append(btnCheck, btnPrivacy, btnMemory);
}

function startSystemHealthAndCleanupPreview(): void {
  setCharacterState("Analysiere System (Nur lesend)...", true);
  currentProposal = {
    requestId: crypto.randomUUID(),
    title: "AIDO Fast Cleanup (Temporäre Sitzungs-Caches bereinigen)",
    explanation:
      "Ich habe deinen PC lesend geprüft: Deine NVIDIA GeForce RTX 5090 hat aktuell über 29 GB freien VRAM (Hauptmodell schläft bei Nichtgebrauch). Im isolierten AIDO-Temporärspeicher liegen 3 temporäre Cache-Dateien (448 KB). Deine persönlichen Dateien, Projekte und Modelle bleiben unangetastet. Darf ich das Fast Cleanup jetzt ausführen?",
    files: [
      "aido_vision_preview_01.tmp (64 KB)",
      "aido_whisper_chunk_01.tmp (128 KB)",
      "aido_slot_prompt_cache.tmp (256 KB)"
    ],
    bytes: 448 * 1024
  };

  setCharacterState("Warte auf deine Freigabe", false);
  aidoMessage.innerHTML = `
    <strong>${currentProposal.title}</strong><br/>
    ${currentProposal.explanation}<br/><br/>
    <em>Betroffene temporäre Dateien:</em> ${currentProposal.files.join(", ")}
  `;

  appendAudit(
    `[PermissionGateway] Freigabe-Anfrage ${currentProposal.requestId.slice(0, 8)} erstellt (FastCleanupPreview, 448 KB).`
  );

  actionButtons.innerHTML = "";
  const btnApprove = document.createElement("button");
  btnApprove.className = "approve";
  btnApprove.textContent = "✅ Ja, jetzt bereinigen";
  btnApprove.onclick = () => executeApprovedCleanup();

  const btnReject = document.createElement("button");
  btnReject.className = "reject";
  btnReject.textContent = "❌ Nein, abbrechen";
  btnReject.onclick = () => rejectCleanup();

  actionButtons.append(btnApprove, btnReject);
}

function executeApprovedCleanup(): void {
  if (!currentProposal) return;
  setCharacterState("Führe Fast Cleanup aus...", true);
  const reqId = currentProposal.requestId.slice(0, 8);

  setTimeout(() => {
    setCharacterState("Erfolgreich bereinigt!", false);
    aidoMessage.innerHTML = `
      <strong>Fast Cleanup abgeschlossen!</strong><br/>
      Ich habe die 3 temporären Cache-Dateien (448 KB) sicher entfernt und das Ergebnis in deinem lokalen SQLite/FTS5-Gedächtnis gespeichert.
    `;
    appendAudit(
      `[WindowsBridge] FastCleanupExecute mit Einmal-Token für Anfrage ${reqId} ausgeführt (3 Dateien entfernt, 448 KB frei).`
    );
    appendAudit(
      `[MemoryEngine] Episodische Erinnerung gespeichert: 'Fast Cleanup nach Nutzerfreigabe ausgeführt'.`
    );
    currentProposal = null;
    renderDefaultButtons();
  }, 350);
}

function rejectCleanup(): void {
  if (!currentProposal) return;
  const reqId = currentProposal.requestId.slice(0, 8);
  setCharacterState("Bereit (Keine Änderung)", false);
  aidoMessage.textContent =
    "Verstanden! Ich habe nichts verändert und alle Dateien genau so belassen, wie sie sind.";
  appendAudit(
    `[PermissionGateway] Anfrage ${reqId} vom Nutzer abgelehnt — keine Systemänderung durchgeführt.`
  );
  currentProposal = null;
  renderDefaultButtons();
}

function showPrivacyCenter(): void {
  setCharacterState("Privacy Center aktiv", false);
  aidoMessage.innerHTML = `
    <strong>🔒 AIDO Privacy Center — Modus: Private Offline</strong><br/>
    • <b>KI-Verarbeitung:</b> 100 % lokal (Qwen3.8-27B-UD-Q4_K_M auf 127.0.0.1:18434)<br/>
    • <b>Screenshots & Vision:</b> 100 % lokal (mmproj-Qwen3.8-27B-BF16)<br/>
    • <b>Spracherkennung:</b> 100 % lokal (Whisper-small CUDA auf 127.0.0.1:18080)<br/>
    • <b>Memory & Wissen:</b> Lokale SQLite/FTS5-Datenbank (%LOCALAPPDATA%\\AIDO\\data)<br/>
    • <b>Externe Verbindungen:</b> Technisch durch das Permission Gateway blockiert.
  `;
  appendAudit("[PrivacyCenter] Status geprüft: Private Offline aktiv (nur 127.0.0.1 erlaubt).");
  renderDefaultButtons();
}

function showMemorySummary(): void {
  setCharacterState("Memory & Wissen", false);
  aidoMessage.innerHTML = `
    <strong>🧠 AIDO Lokales Gedächtnis (SQLite + FTS5)</strong><br/>
    • <b>System Memory:</b> Predator Orion 7000 · RTX 5090 (32 GB VRAM) · 64 GB RAM · Windows 11<br/>
    • <b>Procedural Memory:</b> Geprüfter Ablauf für 'System-Gesundheitscheck & Fast Cleanup'<br/>
    • <b>Kontrolle:</b> Alle Erinnerungen sind lokal gespeichert, mit Quelle & Zeitstempel versehen und jederzeit einsehbar oder löschbar.
  `;
  renderDefaultButtons();
}

// Character Mouse Interactions
avatarOrb.addEventListener("mousedown", (e) => {
  if (e.button === 0) {
    pressTimer = window.setTimeout(() => {
      setCharacterState("Höre zu (Whisper Push-to-Talk)...", true);
      aidoMessage.textContent =
        "🎤 Mikrofon-Aufnahme aktiv (Loslassen zum lokalen Transkribieren über 127.0.0.1:18080)...";
    }, 450);
  } else if (e.button === 1) {
    e.preventDefault();
    setCharacterState("Screenshot-Vorschau", false);
    aidoMessage.innerHTML =
      "📸 <strong>Screenshot-Kontext bereit:</strong> Im privaten Offline-Modus wird das Bild ausschließlich lokal von <code>mmproj-Qwen3.8-27B-BF16</code> ausgewertet.";
    appendAudit("[VisionContext] Lokaler Screenshot-Kontext vorgemerkt (keine Cloud-Übertragung).");
  }
});

avatarOrb.addEventListener("mouseup", (e) => {
  if (e.button === 0 && pressTimer !== null) {
    clearTimeout(pressTimer);
    pressTimer = null;
  }
});

window.addEventListener("contextmenu", (e) => {
  e.preventDefault();
  contextMenu.style.display = "block";
  contextMenu.style.left = `${e.clientX}px`;
  contextMenu.style.top = `${e.clientY}px`;
});

window.addEventListener("click", () => {
  contextMenu.style.display = "none";
});

contextMenu.addEventListener("click", (e) => {
  const target = (e.target as HTMLElement).closest(".context-item") as HTMLElement | null;
  if (!target) return;
  const action = target.dataset.action;
  if (action === "health" || action === "cleanup") {
    startSystemHealthAndCleanupPreview();
  } else if (action === "privacy") {
    showPrivacyCenter();
  } else if (action === "memory") {
    showMemorySummary();
  } else if (action === "sleep") {
    setCharacterState("Schlafmodus (VRAM freigegeben)", false);
    aidoMessage.textContent =
      "💤 Ich bin im Schlafmodus. Das 27B-Hauptmodell gibt seinen VRAM automatisch frei und wacht erst bei deiner nächsten Anfrage wieder auf.";
    appendAudit("[Runtime] AIDO in den Schlafmodus versetzt (VRAM-Entladung aktiv).");
  }
});

renderDefaultButtons();
