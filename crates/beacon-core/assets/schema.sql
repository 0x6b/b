-- Enable foreign key constraints (must be done per connection in SQLite)
PRAGMA foreign_keys = ON;

-- Migration: Add status column to existing plans table if it doesn't exist
-- SQLite doesn't support ALTER TABLE ADD COLUMN IF NOT EXISTS, so we handle this in application code

-- Plans table: stores task plans with metadata
CREATE TABLE IF NOT EXISTS plans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL, -- Title of the plan
    description TEXT, -- Detailed multi-line description of the plan
    status TEXT NOT NULL DEFAULT 'active' CHECK(status IN ('active', 'archived')),
    directory TEXT, -- Working directory for the plan (defaults to CWD)
    created_at TEXT NOT NULL, -- ISO 8601 format (e.g., "2024-01-15T10:30:00Z")
    updated_at TEXT NOT NULL  -- ISO 8601 format
);

-- Steps table: stores individual steps within plans
CREATE TABLE IF NOT EXISTS steps (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    plan_id INTEGER NOT NULL,
    title TEXT NOT NULL, -- Brief title/summary of the step
    description TEXT, -- Detailed multi-line description of the step
    acceptance_criteria TEXT, -- Clear completion criteria for the step
    step_references TEXT, -- Comma-separated list of references (URLs, file paths)
    status TEXT NOT NULL DEFAULT 'todo' CHECK(status IN ('todo', 'inprogress', 'done')),
    result TEXT, -- Description of what was accomplished (required when status = 'done')
    step_order INTEGER NOT NULL, -- 'order' is a SQL reserved keyword
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (plan_id) REFERENCES plans(id) ON DELETE CASCADE
);

-- Indexes for query performance
CREATE INDEX IF NOT EXISTS idx_steps_plan_id ON steps(plan_id);
CREATE INDEX IF NOT EXISTS idx_steps_status ON steps(status);
CREATE INDEX IF NOT EXISTS idx_steps_plan_id_order ON steps(plan_id, step_order);
CREATE INDEX IF NOT EXISTS idx_plans_created_at ON plans(created_at);
CREATE INDEX IF NOT EXISTS idx_plans_title ON plans(title COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_plans_status ON plans(status);

-- View for active plans with step counts (useful for summary queries)
CREATE VIEW IF NOT EXISTS plan_summaries AS
SELECT 
    p.id,
    p.title,
    p.description,
    p.status,
    p.directory,
    p.created_at,
    p.updated_at,
    COUNT(s.id) as total_steps,
    SUM(CASE WHEN s.status = 'done' THEN 1 ELSE 0 END) as completed_steps,
    SUM(CASE WHEN s.status = 'todo' THEN 1 ELSE 0 END) as pending_steps,
    SUM(CASE WHEN s.status = 'inprogress' THEN 1 ELSE 0 END) as in_progress_steps
FROM plans p
LEFT JOIN steps s ON p.id = s.plan_id
WHERE p.status = 'active'
GROUP BY p.id;

-- View for all plans including archived ones
CREATE VIEW IF NOT EXISTS all_plan_summaries AS
SELECT 
    p.id,
    p.title,
    p.description,
    p.status,
    p.directory,
    p.created_at,
    p.updated_at,
    COUNT(s.id) as total_steps,
    SUM(CASE WHEN s.status = 'done' THEN 1 ELSE 0 END) as completed_steps,
    SUM(CASE WHEN s.status = 'todo' THEN 1 ELSE 0 END) as pending_steps,
    SUM(CASE WHEN s.status = 'inprogress' THEN 1 ELSE 0 END) as in_progress_steps
FROM plans p
LEFT JOIN steps s ON p.id = s.plan_id
GROUP BY p.id;