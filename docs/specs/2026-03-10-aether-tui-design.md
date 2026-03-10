# Aether-TUI (Aether Monitor) Design Specification

## Overview
Aether-TUI is the official observability and control interface for AetherOS. It abandons the traditional "chat" paradigm in favor of a **Process Monitor** approach. Built with Rust and `ratatui`, it provides a low-latency, highly concurrent dashboard to supervise autonomous agents.

---
### 👤 Designed & Led by Joaquín Escalante
**Joaquín Escalante** ([https://github.com/joaquinescalante23](https://github.com/joaquinescalante23))

---

## 🎨 Layout & UX Design

The TUI is divided into a grid system to maximize information density without overwhelming the user:

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│ AetherOS Kernel Monitor (chronosd) | v1.1.0 | Connected: [::1]:50051        │
├───────────────┬───────────────────────────────────────────┬─────────────────┤
│ PROCESS LIST  │ 🧠 COGNITIVE STREAM (AGENT: Selected)     │ 📊 RESOURCES    │
│               │                                           │                 │
│ PID      STAT │ [SYSTEM] Booting identity 'Architect'     │ Budget: $10.00  │
│ 1a2b3c.. RUN  │ [THOUGHT] I need to create src/main.rs    │ Spent:  $0.12   │
│ 4d5e6f.. SUSP │ [TOOL] fs_write("src/main.rs", ...)       │                 │
│ 7g8h9i.. ERR  │ [RESULT] Successfully wrote file.         │ Tokens: 45k/100k│
│               │ [THOUGHT] Compiling the project now.      │                 │
│               │ [TOOL] shell_execute("cargo build")       │                 │
│               │                                           │                 │
├───────────────┴───────────────────────────────────────────┴─────────────────┤
│ ⌨️ COMMANDS: [↑↓] Navigate | [P] Pause | [R] Resume | [K] Kill | [Q] Quit   │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 🛠 Core Components

1. **Process List (Left):** Real-time list of all agents fetched via `ListAgents` gRPC call. Displays Agent ID (truncated), Name, and Status (Running, Suspended, Error).
2. **Cognitive Stream (Center):** The "mind" of the selected agent. Uses the `MonitorAgent` gRPC stream to display Thoughts, Tool Calls, and Results as they happen.
3. **Resource Dashboard (Right):** Visual indicators (progress bars) for the selected agent's budget consumption (Tokens and USD) to prevent runaway costs.
4. **Command Footer (Bottom):** Global shortcuts for lifecycle management. Pressing `P` invokes `PauseAgent` on the selected process instantly.

## 🏗 Technical Stack
- **Framework:** `ratatui` + `crossterm` for backend-agnostic terminal rendering.
- **Async Runtime:** `tokio` for handling gRPC streams without blocking the UI thread.
- **Communication:** Reuses `chronos_proto` client from the CLI.

## 🚀 Implementation Phases
1. **Scaffolding:** Create `aether-tui` crate and setup basic terminal loop.
2. **State Management:** Implement `App` state to hold agent list and selected agent data.
3. **UI Rendering:** Draw the layout blocks and populate them with mock data.
4. **gRPC Integration:** Connect the state to real data from `chronosd`.

---
*Created by Joaquín Escalante ([https://github.com/joaquinescalante23](https://github.com/joaquinescalante23))*
