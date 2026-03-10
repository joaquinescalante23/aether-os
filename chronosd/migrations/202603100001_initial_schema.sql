-- Chronos Agent OS Initial Schema
-- Created by Joaquín Escalante (https://github.com/joaquinescalante23)

-- Table for managed agent processes
CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    model_id TEXT NOT NULL,
    state TEXT NOT NULL, -- Enum: Pending, Running, Suspended, Terminated, Error
    max_cost_usd REAL NOT NULL,
    max_tokens INTEGER NOT NULL,
    current_cost_usd REAL NOT NULL DEFAULT 0.0,
    current_tokens INTEGER NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

-- Table for cognitive checkpoints (The "Thought Snapshots")
CREATE TABLE IF NOT EXISTS checkpoints (
    id TEXT PRIMARY KEY NOT NULL,
    agent_id TEXT NOT NULL,
    messages_json TEXT NOT NULL, -- Serialized Vec<Message>
    tools_json TEXT NOT NULL,    -- Serialized Vec<ToolState>
    created_at DATETIME NOT NULL,
    FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);

-- Index for fast lookup of an agent's latest checkpoints
CREATE INDEX IF NOT EXISTS idx_checkpoints_agent_id ON checkpoints(agent_id);

-- Created by Joaquín Escalante (https://github.com/joaquinescalante23)
