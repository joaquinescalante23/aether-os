# AetherOS v1.2: Somnium & Deep-Heal Architecture Design

## 👤 Architect & Project Lead
**Joaquín Escalante** ([https://github.com/joaquinescalante23](https://github.com/joaquinescalante23))

## Overview
AetherOS v1.2 introduces two revolutionary sub-systems to the Cognitive Kernel: **Somnium** (Adaptive Context Management) and **Deep-Heal** (Automated Self-Correction via Multi-Model Bursting). These features aim to achieve zero-intervention autonomy for long-running agent missions.

---

## 1. Somnium: Cognitive Memory Compression
The Context Window is a finite resource. Current frameworks rely on simple RAG or sliding windows, which leads to loss of critical intent.

### Mechanism:
- **Trigger:** When an agent's memory exceeds 80% of its context limit.
- **The "Dream" State:** The Kernel spawns a transient `SomniumProcess`. 
- **Action:** This process analyzes the full trace, identifies "stale thoughts" vs "terminal state directives," and generates a **Compressed Kernel Snapshot (CKS)**.
- **Output:** The agent is resumed with a clean slate, where its first message is the CKS injected by the Kernel.

---

## 2. Deep-Heal: Automated Self-Correction
AetherOS acts as a quality gate between the agent's work and the persistent Datahive.

### The Feedback Loop:
1. **Execution (Worker):** Agent performs task.
2. **Shadow Audit:** Kernel invokes a `ReviewerIdentity` process to validate the result (Compilation check, Logic check).
3. **Rewind & Retry:** If validation fails, the Kernel uses **Time-Travel Checkpointing** to reset the Worker to its previous state.
4. **Cognitive Burst (The Elite Factor):**
   - If the Worker fails the audit 3 times, the Kernel triggers a `ModelBurst`.
   - The task is re-routed to a higher-reasoning model (e.g., GPT-4o -> o1 / Claude 3.5 Opus).
   - Once resolved, the solution is fed back to the original model, and the mission continues.

---

## 3. Technical Implementation (Rust)

### New Domain Entities:
- `AuditReport`: Struct containing validation results and error logs.
- `MemorySnapshot`: Enhanced checkpoint with compression metadata.

### Application Services:
- `ShadowService`: Orchestrates the Auditor and the Bursting logic.
- `CompressionService`: Manages the Somnium dreaming cycles.

### gRPC Updates:
- `AuditStatus` added to `AgentEvent` stream.
- `BurstTriggered` notification for TUI.

---

## Success Criteria
- [ ] Agents can run for >1,000 steps without context degradation via Somnium.
- [ ] Automated recovery from compilation errors without user input.
- [ ] Zero-loss state migration after a Model Burst event.

---
*Created by Joaquín Escalante ([https://github.com/joaquinescalante23](https://github.com/joaquinescalante23))*
