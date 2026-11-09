# AIDO Phase 5 & 6: Proaktive Vordenker-Engine („AIDO denkt für dich“), Fokus-Schutz & Zweistufiges System-Cleanup (Fast & Deep Cleanup)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` to implement this plan task-by-task using `Model: "flash"` for subagents.

**Goal:** Implement AIDO's signature proactive intelligence (`src-tauri/src/proactive_engine.rs`) with strict Focus/Fullscreen/Quiet-Hours throttling rules and topic muting, plus the second stage of system maintenance (**Deep Cleanup** in `src-tauri/src/windows_bridge.rs`) guarded by one-time `ApprovalToken`s in `PermissionGateway`, and integrate both into `AidoCore`, `aido-cli`, and the Desktop UI.

**Architecture:**
1. **`src-tauri/src/proactive_engine.rs` (New Module):**
   - Evaluates system health, indexed projects (including dirty Git worktrees), and episodic memory density to generate up to 3 high-value, non-intrusive German suggestions with clickable options.
   - Enforces Focus Guard (`FullscreenOrGaming`, `VoiceRecordingOrCall`, `QuietHoursActive` -> 0 interruptions) and automatic backoff when a topic is dismissed $\ge 2$ times or muted by the user.
2. **`src-tauri/src/windows_bridge.rs` & `src-tauri/src/permission_gateway.rs` (Extended):**
   - Adds `ActionKind::DeepCleanupScan` (read-only) and `ActionKind::DeepCleanupExecute` (mutating, requires one-time `ApprovalToken`).
   - Implements `seed_demo_deep_cleanup_files`, `scan_deep_cleanup`, `prepare_deep_cleanup`, and `execute_deep_cleanup` in `WindowsBridge` for categorized deep cleanup (`.old_build`, `.dmp_cache`, `.shader_tmp`) inside AIDO's isolated staging boundary.
3. **`src-tauri/src/lib.rs`, `src-tauri/src/main.rs`, `index.html`, `src/main.ts`:**
   - Wires `ProactiveEngine` and Deep Cleanup into `AidoCore`, adds `aido-cli proactive` and Step `[8/8]` to `aido-cli demo`, and adds the interactive **„💡 Proaktive Vorschläge“** and **„🧽 Deep Cleanup“** workflows to the Desktop UI and right-click context menu.

---

### Task 1: Proactive Initiative Engine (`proactive_engine.rs`) & Deep Cleanup (`windows_bridge.rs`)

**Files:**
- Modify: `src-tauri/src/permission_gateway.rs`
- Modify: `src-tauri/src/windows_bridge.rs`
- Create: `src-tauri/src/proactive_engine.rs`
- Modify: `src-tauri/src/lib.rs` (export `pub mod proactive_engine;`)

---

### Task 2: `AidoCore`, CLI (`aido-cli`) & Desktop UI Integration for Proactive Engine & Deep Cleanup

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `index.html`
- Modify: `src/main.ts`
