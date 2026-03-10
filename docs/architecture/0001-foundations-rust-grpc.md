# ADR 0001: Foundations of Chronos - Rust & gRPC

## Status
Accepted

## Context
Existing agent frameworks (LangChain, CrewAI) are predominantly built in Python. While Python is excellent for research and rapid prototyping, it presents significant challenges for building long-running, high-performance system software (infrastructure):
1. **Concurrency and Performance:** Python's GIL limits efficient multi-threading, which is critical for a daemon managing hundreds of asynchronous agents.
2. **Type Safety and Memory Integrity:** In a system responsible for critical state persistence and complex state machines, Rust's borrow checker and strict type system eliminate entire classes of bugs (null pointers, data races) common in system software.
3. **Distribution and Deployment:** A compiled Rust binary (`chronosd`) is easier to distribute and deploy in production environments (Kubernetes, Edge) compared to Python environments with complex dependency graphs.

Regarding IPC (Inter-Process Communication):
Traditional agent systems communicate via raw strings or JSON over REST. This lacks strict contracts and is prone to runtime errors as agent schemas evolve.

## Decision
1. **Core Language:** Use **Rust** for all system-level components (`chronosd`, `chronos-cli`).
2. **IPC Protocol:** Use **gRPC (via Tonic)** for communication between the daemon and any external clients/agents. gRPC provides:
    - Language-agnostic typed contracts (via Protobuf).
    - Bi-directional streaming (critical for real-time agent monitoring).
    - High performance and low overhead.
3. **Internal State Representation:** Use **Protocol Buffers** for serializing agent snapshots to ensure long-term compatibility and efficient storage.

## Consequences
### Positive
- Exceptional performance and resource efficiency.
- High reliability and safety for long-running processes.
- Clear, enforced API boundaries between components.
- Modern, high-performance async runtime (Tokio).

### Negative
- Steeper learning curve for contributors compared to Python.
- Longer compilation times.
- More boilerplate required for gRPC/Protobuf definitions compared to REST/JSON.

---
*Created by Joaquín Escalante ([https://github.com/joaquinescalante23](https://github.com/joaquinescalante23))*
