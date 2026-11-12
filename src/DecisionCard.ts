import { invoke } from "@tauri-apps/api/core";

export interface ProposalOption {
  id: string;
  label: string;
  is_default: boolean;
}

export interface ProactiveProposalRecord {
  id: string;
  title: string;
  human_explanation: string;
  risk_level: "GREEN_READ" | "YELLOW_PROJECT" | "RED_SYSTEM";
  action_payload_json: string;
  options: ProposalOption[];
  created_at: string;
  state: "PENDING" | "APPROVED" | "REJECTED" | "EXECUTED";
}

export class DecisionCardRenderer {
  public static renderCard(
    proposal: ProactiveProposalRecord,
    onActionCompleted: () => void
  ): HTMLElement {
    const card = document.createElement("article");
    card.className = `decision-card risk-${proposal.risk_level.toLowerCase()}`;
    card.dataset.id = proposal.id;

    // Risk badge
    const badge = document.createElement("div");
    badge.className = "risk-badge";
    if (proposal.risk_level === "GREEN_READ") {
      badge.innerHTML = "🟢 <span>Harmlos · Lesend</span>";
    } else if (proposal.risk_level === "YELLOW_PROJECT") {
      badge.innerHTML = "🟡 <span>Projekt-Aktion</span>";
    } else {
      badge.innerHTML = "🔴 <span>System-Eingriff</span>";
    }

    const title = document.createElement("h4");
    title.className = "card-title";
    title.textContent = proposal.title;

    const explanation = document.createElement("p");
    explanation.className = "card-explanation";
    explanation.textContent = proposal.human_explanation;

    const actionsContainer = document.createElement("div");
    actionsContainer.className = "card-actions";

    proposal.options.forEach((opt) => {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = `decision-btn ${opt.is_default ? "primary" : "secondary"}`;
      btn.textContent = opt.label;

      btn.addEventListener("click", async () => {
        actionsContainer.querySelectorAll("button").forEach((b) => (b.disabled = true));
        try {
          if (opt.id === "dismiss" || opt.id === "cancel") {
            await invoke("reject_proposal", { requestId: proposal.id });
          } else {
            await invoke("approve_proposal", { requestId: proposal.id, userNote: opt.id });
          }
          card.classList.add("fade-out");
          setTimeout(() => {
            card.remove();
            onActionCompleted();
          }, 300);
        } catch (err) {
          explanation.textContent = `Fehler: ${String(err)}`;
          actionsContainer.querySelectorAll("button").forEach((b) => (b.disabled = false));
        }
      });

      actionsContainer.appendChild(btn);
    });

    card.append(badge, title, explanation, actionsContainer);
    return card;
  }
}
