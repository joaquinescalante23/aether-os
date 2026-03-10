# ADR 0002: Mission-Based Multi-Agent Orchestration

## Status
Accepted

## Context
Traditional agent systems often use simple graphs or sequential chains to coordinate agents. This approach is rigid and fails when a task requires dynamic collaboration, shared context, or role-based specialization. To build a true Agent Operating System, we need a way to group autonomous processes under a single objective.

## Decision
We implement a **Mission-based Orchestration** model.
1. **The Mission Entity:** A high-level domain object that groups multiple agent processes. It maintains a `shared_context_id` to enable future inter-agent memory synchronization.
2. **Orchestrator Service:** An application-layer service responsible for spawning specialized agents from `Identity` templates (Architects, Workers, Auditors) and coordinating their lifecycles.
3. **Managed Workgroups:** Agents in a mission are managed as a unit by the Kernel, allowing for aggregated budget tracking and collective mission status reporting.

## Consequences
### Positive
- Enables solving complex, multi-step tasks (e.g., full-stack development) that a single agent cannot handle.
- Role-based specialization improves result quality and reduces prompt confusion.
- Provides a clear path for implementing Inter-Process Communication (IPC) between agents.

### Negative
- Higher complexity in state management (managing multiple related processes).
- Increased token consumption as multiple agents collaborate.
