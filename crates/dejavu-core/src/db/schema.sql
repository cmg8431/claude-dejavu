-- claude-dejavu SQLite schema

CREATE TABLE IF NOT EXISTS sessions (
    id          TEXT PRIMARY KEY,
    project_path TEXT NOT NULL,
    started_at  TEXT NOT NULL DEFAULT (datetime('now')),
    ended_at    TEXT,
    message_count INTEGER DEFAULT 0,
    tool_call_count INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS patterns (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    detector_type TEXT NOT NULL,  -- 'revert_cycle' | 'repeated_error' | 'silent_fix' | 'user_correction' | 'tool_event'
    project_path TEXT NOT NULL,
    session_id  TEXT NOT NULL,
    detected_at TEXT NOT NULL DEFAULT (datetime('now')),
    evidence_json TEXT NOT NULL,  -- JSON blob with detector-specific data
    cluster_id  TEXT              -- groups related patterns
);

CREATE TABLE IF NOT EXISTS rules (
    id          TEXT PRIMARY KEY,  -- r-001 format
    project_path TEXT NOT NULL,
    scope       TEXT NOT NULL DEFAULT 'project',  -- 'global' | 'project' | 'file' | 'personal'
    scope_target TEXT,            -- file path for file-scoped rules
    text        TEXT NOT NULL,    -- the actual rule text
    confidence  REAL NOT NULL DEFAULT 0.0,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    last_fired  TEXT,
    fire_count  INTEGER DEFAULT 0,
    source_patterns TEXT NOT NULL, -- JSON array of pattern IDs
    status      TEXT NOT NULL DEFAULT 'proposed'  -- 'proposed' | 'active' | 'dead' | 'rejected'
);

CREATE TABLE IF NOT EXISTS rule_fires (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_id     TEXT NOT NULL,
    session_id  TEXT NOT NULL,
    fired_at    TEXT NOT NULL DEFAULT (datetime('now')),
    prevented   INTEGER DEFAULT 0   -- 1 if the mistake was prevented
);

-- Correction queue: real-time user corrections captured via UserPromptSubmit hook
CREATE TABLE IF NOT EXISTS corrections (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id  TEXT NOT NULL,
    project_path TEXT NOT NULL,
    prompt_text TEXT NOT NULL,
    correction_type TEXT NOT NULL, -- 'explicit' | 'redirect' | 'negative' | 'preference'
    confidence  REAL NOT NULL DEFAULT 0.0,
    matched_text TEXT,
    captured_text TEXT,
    processed   INTEGER DEFAULT 0, -- 1 when consumed by detector
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Cross-project rule promotion ledger
CREATE TABLE IF NOT EXISTS promotions (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_text_hash TEXT NOT NULL, -- normalized hash of rule text
    source_project TEXT NOT NULL,
    target_scope TEXT NOT NULL,   -- 'global' when promoted
    promoted_at TEXT NOT NULL DEFAULT (datetime('now')),
    original_rule_id TEXT,
    UNIQUE(rule_text_hash, source_project)
);

CREATE INDEX IF NOT EXISTS idx_patterns_project ON patterns(project_path);
CREATE INDEX IF NOT EXISTS idx_patterns_detector ON patterns(detector_type);
CREATE INDEX IF NOT EXISTS idx_patterns_cluster ON patterns(cluster_id);
CREATE INDEX IF NOT EXISTS idx_rules_project ON rules(project_path);
CREATE INDEX IF NOT EXISTS idx_rules_status ON rules(status);
CREATE INDEX IF NOT EXISTS idx_rule_fires_rule ON rule_fires(rule_id);
CREATE INDEX IF NOT EXISTS idx_corrections_project ON corrections(project_path);
CREATE INDEX IF NOT EXISTS idx_corrections_processed ON corrections(processed);
CREATE INDEX IF NOT EXISTS idx_promotions_hash ON promotions(rule_text_hash);
