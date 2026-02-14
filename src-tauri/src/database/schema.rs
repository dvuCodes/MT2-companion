// Database schema definitions
// All table schemas defined here as constants for DRY principle

pub const CREATE_CARDS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS cards (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    clan TEXT NOT NULL,
    card_type TEXT NOT NULL,
    rarity TEXT NOT NULL,
    cost INTEGER,
    base_value INTEGER NOT NULL,
    tempo_score INTEGER NOT NULL,
    value_score INTEGER NOT NULL,
    keywords TEXT, -- JSON array
    description TEXT,
    expansion TEXT DEFAULT 'base',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_cards_clan ON cards(clan);
CREATE INDEX IF NOT EXISTS idx_cards_rarity ON cards(rarity);
CREATE INDEX IF NOT EXISTS idx_cards_expansion ON cards(expansion);
"#;

pub const CREATE_SYNERGIES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS synergies (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    card_a_id TEXT NOT NULL,
    card_b_id TEXT NOT NULL,
    synergy_type TEXT NOT NULL,
    weight REAL NOT NULL DEFAULT 1.0,
    description TEXT,
    bidirectional BOOLEAN DEFAULT 1,
    FOREIGN KEY (card_a_id) REFERENCES cards(id),
    FOREIGN KEY (card_b_id) REFERENCES cards(id),
    UNIQUE(card_a_id, card_b_id, synergy_type)
);

CREATE INDEX IF NOT EXISTS idx_synergies_card_a ON synergies(card_a_id);
CREATE INDEX IF NOT EXISTS idx_synergies_card_b ON synergies(card_b_id);
"#;

pub const CREATE_CONTEXT_MODIFIERS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS context_modifiers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    condition TEXT NOT NULL,
    card_tag TEXT NOT NULL,
    modifier INTEGER NOT NULL,
    priority TEXT NOT NULL, -- Low, Medium, High, Critical
    description TEXT,
    active BOOLEAN DEFAULT 1
);

CREATE INDEX IF NOT EXISTS idx_context_modifiers_tag ON context_modifiers(card_tag);
"#;

pub const CREATE_CHAMPION_OVERRIDES_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS champion_overrides (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    champion TEXT NOT NULL,
    path TEXT,
    card_id TEXT NOT NULL,
    value_override INTEGER NOT NULL,
    reason TEXT,
    FOREIGN KEY (card_id) REFERENCES cards(id),
    UNIQUE(champion, path, card_id)
);

CREATE INDEX IF NOT EXISTS idx_champion_overrides_champion ON champion_overrides(champion);
"#;

pub const CREATE_DECK_HISTORY_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS deck_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id TEXT NOT NULL,
    card_id TEXT NOT NULL,
    ring_number INTEGER NOT NULL,
    draft_order INTEGER NOT NULL,
    champion TEXT NOT NULL,
    covenant INTEGER NOT NULL,
    score_at_draft INTEGER,
    did_win BOOLEAN,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (card_id) REFERENCES cards(id)
);

CREATE INDEX IF NOT EXISTS idx_deck_history_run ON deck_history(run_id);
CREATE INDEX IF NOT EXISTS idx_deck_history_card ON deck_history(card_id);
"#;

pub const CREATE_EXPANSIONS_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS expansions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    release_date TEXT,
    is_active BOOLEAN DEFAULT 1,
    description TEXT
);
"#;
