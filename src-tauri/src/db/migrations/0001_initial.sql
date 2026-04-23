PRAGMA foreign_keys = ON;
PRAGMA journal_mode = WAL;

CREATE TABLE industry_categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name_de TEXT NOT NULL UNIQUE,
    osm_tags TEXT NOT NULL,
    probability_weight INTEGER NOT NULL CHECK (probability_weight BETWEEN 0 AND 100),
    enabled INTEGER NOT NULL DEFAULT 1,
    color TEXT NOT NULL DEFAULT '#3b82f6',
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE companies (
    id TEXT PRIMARY KEY,
    osm_id TEXT UNIQUE,
    name TEXT NOT NULL,
    street TEXT,
    postal_code TEXT,
    city TEXT,
    country TEXT NOT NULL DEFAULT 'DE',
    lat REAL NOT NULL,
    lng REAL NOT NULL,
    phone TEXT,
    email TEXT,
    website TEXT,
    industry_category_id INTEGER REFERENCES industry_categories(id) ON DELETE SET NULL,
    size_estimate TEXT,
    probability_score INTEGER NOT NULL DEFAULT 0 CHECK (probability_score BETWEEN 0 AND 100),
    status TEXT NOT NULL DEFAULT 'neu' CHECK (status IN ('neu','angefragt','kunde','kein_kunde')),
    contact_person TEXT,
    last_contact_at TEXT,
    next_followup_at TEXT,
    source TEXT NOT NULL CHECK (source IN ('osm','manual')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_companies_status ON companies(status);
CREATE INDEX idx_companies_followup ON companies(next_followup_at);
CREATE INDEX idx_companies_category ON companies(industry_category_id);
CREATE INDEX idx_companies_geo ON companies(lat, lng);

CREATE TABLE activity_log (
    id TEXT PRIMARY KEY,
    company_id TEXT NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    type TEXT NOT NULL CHECK (type IN ('notiz','anruf','mail','besuch','status_änderung')),
    content TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_activity_company ON activity_log(company_id);
CREATE INDEX idx_activity_created ON activity_log(created_at DESC);

CREATE TABLE search_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    center_label TEXT NOT NULL,
    center_lat REAL NOT NULL,
    center_lng REAL NOT NULL,
    radius_km INTEGER NOT NULL CHECK (radius_km BETWEEN 1 AND 300),
    enabled_category_ids TEXT NOT NULL,
    last_run_at TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE geocode_cache (
    query TEXT PRIMARY KEY,
    lat REAL NOT NULL,
    lng REAL NOT NULL,
    display_name TEXT NOT NULL,
    cached_at TEXT NOT NULL
);

CREATE TABLE app_meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
