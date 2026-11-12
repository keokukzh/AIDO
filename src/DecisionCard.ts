import { invoke } from "@tauri-apps/api/core";

export interface ProposalOption {
  id: string;
  label: string;
  is_default: boolean;
}

export interface BlastRadius {
  summary_de: string;
  leaves_device: boolean;
  privacy_sensitive: boolean;
  affects_other_programs: boolean;
}

export interface IssuedApproval {
  status: string;
  token_id: string;
  action: string;
  token: {
    token_id: string;
    request_id: string;
    action: string;
    issued_at: string;
    user_note?: string | null;
  };
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
  phase?: string;
  preview_de?: string;
  blast_summary_de?: string;
  blast_radius?: BlastRadius;
  execute_command?: string;
  execute_args?: Record<string, unknown>;
}

export class DecisionCardRenderer {
  public static renderCard(
    proposal: ProactiveProposalRecord,
    onActionCompleted: () => void,
    onExecuted?: (result: unknown) => void
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

    const blastText = proposal.blast_summary_de ?? proposal.blast_radius?.summary_de;
    const blast = document.createElement("p");
    blast.className = "card-blast";
    if (blastText) {
      blast.textContent = blastText;
    } else {
      blast.hidden = true;
    }

    const actionsContainer = document.createElement("div");
    actionsContainer.className = "card-actions";

    proposal.options.forEach((opt) => {
      const btn = document.createElement("button");
      btn.type = "button";
      btn.className = `decision-btn ${opt.is_default ? "primary" : "secondary"}`;
      btn.textContent = opt.label;

      btn.addEventListener("click", async () => {
        actionsContainer.querySelectorAll("button").forEach((b) => (b.disabled = true));
        let approved = false;
        try {
          if (opt.id === "dismiss" || opt.id === "cancel") {
            await invoke("reject_proposal", { requestId: proposal.id });
          } else {
            const approval = await invoke<IssuedApproval>("approve_proposal", {
              requestId: proposal.id,
              userNote: opt.id,
            });
            approved = true;
            if (proposal.execute_command) {
              if (!approval.token?.token_id) {
                throw new Error("Die Freigabe hat kein Token geliefert.");
              }
              const args =
                proposal.execute_args && typeof proposal.execute_args === "object"
                  ? proposal.execute_args
                  : {};
              const result = await invoke(proposal.execute_command, {
                ...args,
                token: approval.token,
              });
              onExecuted?.(result);
            }
          }
          card.classList.add("fade-out");
          setTimeout(() => {
            card.remove();
            onActionCompleted();
          }, 300);
        } catch (err) {
          explanation.textContent = `Fehler: ${String(err)}`;
          if (!approved) {
            actionsContainer.querySelectorAll("button").forEach((b) => (b.disabled = false));
          }
        }
      });

      actionsContainer.appendChild(btn);
    });

    card.append(badge, title, explanation, blast, actionsContainer);
    return card;
  }
}
