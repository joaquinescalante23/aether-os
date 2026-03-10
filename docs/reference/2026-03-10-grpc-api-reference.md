# Chronos gRPC API Reference

## Overview
The Chronos API is the primary interface for managing the Agent Operating System. It uses gRPC for high-performance, bi-directional streaming and strictly typed communication.

---
### 👤 Created & Led by Joaquín Escalante
**Joaquín Escalante** ([https://github.com/joaquinescalante23](https://github.com/joaquinescalante23))

---

## Service: `ChronosKernel`

### 1. `SpawnAgent`
Initializes and starts a new agent process in the kernel.
- **Request:** `SpawnAgentRequest` (Name, Model, Initial Prompt, Budget)
- **Response:** `SpawnAgentResponse` (Agent ID, Initial State)

### 2. `StopAgent`
Gracefully terminates an agent process.
- **Request:** `StopAgentRequest` (Agent ID)
- **Response:** `StopAgentResponse` (Final Stats)

### 3. `PauseAgent`
Triggers an immediate **Cognitive Checkpoint** and suspends execution.
- **Request:** `PauseAgentRequest` (Agent ID)
- **Response:** `PauseAgentResponse` (Checkpoint ID, Snapshot Summary)

### 4. `ResumeAgent`
Restores an agent from its last known checkpoint.
- **Request:** `ResumeAgentRequest` (Agent ID or Checkpoint ID)
- **Response:** `ResumeAgentResponse` (Resumed State)

### 5. `ListAgents`
Lists all agent processes currently managed by the kernel.
- **Request:** `Empty`
- **Response:** `ListAgentsResponse` (List of Agent Summaries)

### 6. `InspectAgent`
Provides a deep-dive into the agent's current "heap" (memory) and "context window".
- **Request:** `InspectAgentRequest` (Agent ID)
- **Response:** `InspectAgentResponse` (Full Cognitive State)

### 7. `MonitorAgent` (Streaming)
Provides a real-time stream of the agent's thoughts, tool calls, and state changes.
- **Request:** `MonitorAgentRequest` (Agent ID)
- **Response:** `Stream of AgentEvent`

## Data Structures

### `AgentState` (Enum)
- `PENDING`: Initializing.
- `RUNNING`: Actively thinking or calling tools.
- `SUSPENDED`: Paused via `PauseAgent`.
- `TERMINATED`: Stopped.
- `ERROR`: Crashed or exceeded budget.

### `AgentEvent` (Message)
- `Thought`: Raw LLM output.
- `ToolCall`: Request to execute a tool.
- `ToolResult`: Output from a tool.
- `Checkpoint`: Notification of a successful state save.

---
*Created by Joaquín Escalante ([https://github.com/joaquinescalante23](https://github.com/joaquinescalante23))*
