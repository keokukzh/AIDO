# AIDO Phase 4: Live-Runtime-Supervisor, Vision/ASR-Pipeline & 4-Stufen-Memory-Kompaktierung

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` to implement this plan task-by-task using `Model: "flash"` for subagents.

**Goal:** Implement the local Runtime Supervisor for `llama-server.exe` (with `--sleep-idle-seconds` zero-VRAM standby, DFlash2 speculative decoding, and multimodal `mmproj` Vision) and `whisper-server.exe` (CUDA 13.4 German ASR), plus the 4-stage Memory Compaction Engine (`WorkingMemory` -> `EpisodicMemory` -> `SemanticMemory` / `SystemMemory`) and interactive Privacy/Memory Center controls.

**Architecture:**
1. **`src-tauri/src/runtime_supervisor.rs` (New Module):**
   - Encapsulates runtime paths, CUDA 13 DLL environment (`C:\AI\llamacpp-cuda`), CLI launch flags (`--sleep-idle-seconds 300`, `--ctx-size 32768`, `--flash-attn on`, `-ngl 99`, `--gpu-layers-draft 99`), and endpoint health/standby probing (`127.0.0.1:18434` and `127.0.0.1:18080`) guarded by `PermissionGateway`.
2. **`src-tauri/src/ai_client.rs` (Extended):**
   - Adds `analyze_screen_context` (using the verified OpenAI multimodal JSON format `{"type": "text", "text": ...}` + `{"type": "image_url", "image_url": {"url": "data:image/png;base64,..."}}`) and `transcribe_voice_note` (local German ASR pipeline via `127.0.0.1:18080` with deterministic offline fallback).
3. **`src-tauri/src/memory.rs` (Extended):**
   - Implements the 4-stage memory lifecycle (`WorkingMemory`, `EpisodicMemory`, `SemanticMemory`, `SystemMemory`), background compaction (`compact_episodic_memories` consolidating active `EpisodicMemory` entries into a `SemanticMemory` digest and marking source entries `compacted`), layer filtering (`list_memories_by_layer`), and user-controlled forgetting (`forget_memory_entry`).
4. **`src-tauri/src/lib.rs`, `src-tauri/src/main.rs`, `index.html`, `src/main.ts`:**
   - Integrates `RuntimeSupervisor` and memory compaction into `AidoCore`, adds `aido-cli runtime` and Step `[7/7]` to `aido-cli demo`, and upgrades the Desktop UI with live Runtime-Sleep status, 4-stage Memory Compaction trigger, and per-entry "Vergessen" buttons.

---

### Task 1: Runtime Supervisor (`runtime_supervisor.rs`) & Multimodal Vision/ASR Client (`ai_client.rs`)

**Files:**
- Create: `src-tauri/src/runtime_supervisor.rs`
- Modify: `src-tauri/src/ai_client.rs`
- Modify: `src-tauri/src/permission_gateway.rs`
- Modify: `src-tauri/src/lib.rs` (register `pub mod runtime_supervisor;`)

---

### Task 2: 4-Stage Memory Compaction (`memory.rs`), `AidoCore` & CLI/UI Integration

**Files:**
- Modify: `src-tauri/src/memory.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/main.rs`
- Modify: `index.html`
- Modify: `src/main.ts`
