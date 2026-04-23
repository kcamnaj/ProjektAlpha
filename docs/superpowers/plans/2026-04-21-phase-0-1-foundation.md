# Phase 0 + 1: Foundation & Core-Suche – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Eine lauffähige Tauri-App-Hülle mit migrierter SQLite-DB, Branchen-Seeds und vollständig getesteter Overpass-Integration, die per Tauri-Command „start_search" eine Umkreis-Suche ausführt und Firmen in die DB importiert (UI noch leer – CLI/Test-getrieben validierbar).

**Architecture:** Tauri-Workspace mit Rust-Core und React-Frontend-Shell. Der Rust-Core ist der Star dieser Phase: modulare Struktur (`db/`, `overpass/`, `nominatim/`, `commands.rs`), strict TDD, strukturierte Logs ab Tag 1. Frontend zeigt nur eine Debug-Seite mit Suche-Trigger und Liste der gefundenen Firmen – Polish kommt in Phase 2.

**Tech Stack:** Tauri 2, Rust (sqlx, reqwest, tracing, tokio, serde, anyhow), React 18 + TypeScript + Vite, Tailwind CSS + shadcn/ui (initialisiert, noch nicht prominent genutzt), SQLite mit WAL.

**Spec-Referenz:** [`docs/superpowers/specs/2026-04-21-firmensuche-design.md`](../specs/2026-04-21-firmensuche-design.md)
**CLAUDE.md:** Drei Prinzipien einhalten – UX zuerst (für später), nicht unnötig kompliziert, Tests + Logs für alles.

**Wichtig – kein Git:** Statt `git commit` nach jedem Schritt ein **Checkpoint**: kurze Notiz was jetzt funktionieren sollte. User entscheidet selbst, wann er sichert.

---

## Datei-Struktur (Ziel nach diesem Plan)

```
projektalpha/
├─ src/                                      # React-Frontend
│  ├─ App.tsx                                # Debug-UI: Such-Trigger + Result-Anzeige
│  ├─ main.tsx
│  ├─ index.css                              # Tailwind-Direktiven
│  └─ lib/
│     └─ tauri.ts                            # invoke-Wrapper mit Typen
├─ src-tauri/
│  ├─ src/
│  │  ├─ main.rs                             # Tauri-Bootstrapping, Logger-Setup
│  │  ├─ commands.rs                         # Tauri-Commands (start_search, etc.)
│  │  ├─ error.rs                            # Zentrale Error-Typen (thiserror)
│  │  ├─ db/
│  │  │  ├─ mod.rs                           # DB-Pool + open()
│  │  │  ├─ migrations.rs                    # Migration-Runner
│  │  │  ├─ companies.rs                     # CRUD Firmen
│  │  │  ├─ activity.rs                      # Activity-Log
│  │  │  ├─ categories.rs                    # Branchen
│  │  │  ├─ profiles.rs                      # Such-Profile (Stub)
│  │  │  └─ migrations/
│  │  │     ├─ 0001_initial.sql
│  │  │     └─ 0002_seed_categories.sql
│  │  ├─ overpass/
│  │  │  ├─ mod.rs
│  │  │  ├─ query_builder.rs                 # Overpass-QL aus Categories
│  │  │  ├─ tile_splitter.rs                 # Radius → Tiles
│  │  │  ├─ client.rs                        # HTTP + Endpoint-Rotation + Retry
│  │  │  ├─ parser.rs                        # JSON → Company-Structs
│  │  │  └─ scoring.rs                       # Tag → Category Match + Score
│  │  ├─ nominatim/
│  │  │  ├─ mod.rs
│  │  │  ├─ client.rs
│  │  │  └─ cache.rs
│  │  └─ logging.rs                          # tracing-Setup
│  ├─ tests/                                 # Integration-Tests (out-of-tree)
│  │  └─ fixtures/                           # Overpass-JSON-Fixtures
│  ├─ Cargo.toml
│  ├─ tauri.conf.json
│  └─ build.rs
├─ tests/                                    # Frontend-E2E (Phase 2+)
├─ docs/superpowers/...                      # bereits vorhanden
├─ CLAUDE.md
├─ package.json
├─ vite.config.ts
├─ tsconfig.json
├─ tailwind.config.ts
├─ postcss.config.js
├─ components.json                           # shadcn config
└─ .gitignore                                # nur als Doku, kein Git aktiv
```

---

## Vorbedingungen (einmalig prüfen, nicht im Plan-Loop)

**Auf der Mac-Dev-Maschine installiert haben:**
- Node.js ≥ 20, pnpm ≥ 9
- Rust stable ≥ 1.75 (`rustup default stable`)
- `cargo install create-tauri-app` (für Vorlage, optional)

Wenn etwas fehlt: installieren bevor Phase 0 startet.

---

# PHASE 0 – Setup

Ziel der Phase: lauffähige App-Hülle (`pnpm tauri dev` öffnet Fenster), SQLite verbunden + migriert, Logging aktiv, Tests laufen.

---

### Task 0.1: Tauri-Projekt initialisieren

**Files:**
- Create: `package.json`, `vite.config.ts`, `tsconfig.json`, `index.html`, `src/main.tsx`, `src/App.tsx`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`, `src-tauri/build.rs`

- [ ] **Step 1: Tauri-Vorlage scaffolden**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha
pnpm create tauri-app@latest . --template react-ts --manager pnpm --identifier de.projektalpha.app --app-name "ProjektAlpha"
```

Bei Prompts: aktuelle Working-Dir nutzen, `.` als App-Name akzeptieren falls Frage kommt. Falls Tool sich beschwert dass Verzeichnis nicht leer (wegen `docs/`, `CLAUDE.md`): `--force` oder das Tool händisch starten und „existing files überschreiben" nicht wählen — manuelle Anlage notfalls über `cargo install create-tauri-app` und dann gleiche Befehle in einem Temp-Ordner, dann `src/`, `src-tauri/`, Config-Files herüberkopieren.

- [ ] **Step 2: Dependencies installieren**

```bash
pnpm install
```

Erwartete Output-Zeile: `dependencies installed in <X>s`.

- [ ] **Step 3: Smoke-Test der Hülle**

```bash
pnpm tauri dev
```

Erwartet: Tauri-Fenster öffnet sich mit React-Default-Page („Welcome to Tauri"). Schließen mit Ctrl+C.

- [ ] **Step 4: Checkpoint**

> **Checkpoint 0.1:** Tauri-Hülle läuft. Dev-Server startet, Fenster öffnet sich, React-Default ist sichtbar.

---

### Task 0.2: Tailwind + shadcn/ui einrichten

**Files:**
- Create: `tailwind.config.ts`, `postcss.config.js`, `components.json`
- Modify: `src/index.css` (Tailwind-Direktiven), `vite.config.ts` (path alias), `tsconfig.json` (path alias)

- [ ] **Step 1: Tailwind installieren**

```bash
pnpm add -D tailwindcss@latest postcss autoprefixer
pnpm dlx tailwindcss init -p
```

Erzeugt `tailwind.config.js` und `postcss.config.js`. Falls JS-Variante: in `tailwind.config.ts` umbenennen und auf TS-Syntax umstellen.

- [ ] **Step 2: tailwind.config.ts editieren**

```ts
import type { Config } from "tailwindcss"

export default {
  darkMode: ["class"],
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: { extend: {} },
  plugins: [],
} satisfies Config
```

- [ ] **Step 3: src/index.css mit Tailwind-Direktiven**

```css
@tailwind base;
@tailwind components;
@tailwind utilities;
```

(Falls noch andere Inhalte da sind: oben drüber setzen.)

- [ ] **Step 4: Path-Alias für shadcn**

`vite.config.ts` ergänzen:
```ts
import path from "path"
// im config.resolve:
resolve: { alias: { "@": path.resolve(__dirname, "./src") } }
```

`tsconfig.json` `compilerOptions` ergänzen:
```json
"baseUrl": ".",
"paths": { "@/*": ["./src/*"] }
```

- [ ] **Step 5: shadcn/ui initialisieren**

```bash
pnpm dlx shadcn@latest init
```

Bei Prompts: Default-Style, Slate-Farbe, CSS-Variablen ja, `src/components` als Komponenten-Pfad, `src/lib/utils` als Utils-Pfad.

- [ ] **Step 6: Eine Test-Komponente einbauen**

```bash
pnpm dlx shadcn@latest add button
```

`src/App.tsx` minimal anpassen:
```tsx
import { Button } from "@/components/ui/button"

function App() {
  return (
    <div className="p-8">
      <h1 className="text-3xl font-bold">ProjektAlpha</h1>
      <Button className="mt-4">Test</Button>
    </div>
  )
}
export default App
```

- [ ] **Step 7: Smoke-Test**

```bash
pnpm tauri dev
```

Erwartet: Tauri-Fenster zeigt „ProjektAlpha" + shadcn-Button (sauber gestylt).

- [ ] **Step 8: Checkpoint**

> **Checkpoint 0.2:** Tailwind + shadcn aktiv. Dark-Mode-Klassen verfügbar (testen wir später).

---

### Task 0.3: Rust-Dependencies aufnehmen

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Cargo.toml erweitern**

In `[dependencies]` Block hinzufügen (oder ersetzen, je nachdem was schon da ist):

```toml
[dependencies]
tauri = { version = "2", features = ["protocol-asset"] }
tauri-plugin-fs = "2"
tauri-plugin-dialog = "2"
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
dirs = "5"
serde_json = "1"
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite", "macros", "chrono", "uuid", "migrate"] }
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json", "fmt"] }
tracing-appender = "0.2"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
thiserror = "2"
anyhow = "1"
once_cell = "1"
regex = "1"

[dev-dependencies]
tokio-test = "0.4"
mockito = "1"
tempfile = "3"
```

- [ ] **Step 2: Build prüfen**

```bash
cd src-tauri && cargo check
```

Erwartet: `Finished` ohne Fehler. Wenn Konflikte mit existierenden Tauri-Plugins aus dem Scaffold: Versionen abgleichen.

- [ ] **Step 3: Checkpoint**

> **Checkpoint 0.3:** Alle Rust-Deps kompilieren. Bereit für Logging + DB.

---

### Task 0.4: Logging-Modul (TDD)

**Files:**
- Create: `src-tauri/src/logging.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Failing Test schreiben**

`src-tauri/src/logging.rs`:
```rust
use std::path::PathBuf;

pub struct LogConfig {
    pub log_dir: PathBuf,
    pub default_level: String,
}

pub fn init(config: LogConfig) -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    std::fs::create_dir_all(&config.log_dir)?;
    let file_appender = tracing_appender::rolling::daily(&config.log_dir, "projektalpha.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.default_level));

    let file_layer = fmt::layer()
        .json()
        .with_writer(non_blocking)
        .with_current_span(false);

    let console_layer = fmt::layer()
        .with_target(false)
        .with_ansi(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(console_layer)
        .try_init()
        .map_err(|e| anyhow::anyhow!("logger init failed: {e}"))?;

    Ok(guard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn init_creates_log_directory() {
        let dir = tempdir().unwrap();
        let log_dir = dir.path().join("logs");
        let config = LogConfig {
            log_dir: log_dir.clone(),
            default_level: "info".to_string(),
        };
        let _guard = init(config);
        assert!(log_dir.exists(), "log dir should be created");
    }
}
```

- [ ] **Step 2: Test laufen lassen (FAIL erwartet)**

```bash
cd src-tauri && cargo test --lib logging
```

Erwartet: Compile-Error bzw. Modul nicht eingehängt.

- [ ] **Step 3: main.rs einhängen**

`src-tauri/src/main.rs` ergänzen:
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod logging;
```

- [ ] **Step 4: Test laufen lassen (PASS erwartet)**

```bash
cargo test --lib logging
```

Erwartet: 1 passed.

- [ ] **Step 5: In main() einbinden**

```rust
fn main() {
    let log_dir = dirs::data_dir()
        .map(|p| p.join("projektalpha").join("logs"))
        .unwrap_or_else(|| std::path::PathBuf::from("./logs"));

    let _guard = logging::init(logging::LogConfig {
        log_dir,
        default_level: "info".to_string(),
    }).expect("logger init failed");

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "app starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

(Die `dirs`-Crate wurde bereits in Task 0.3 mit aufgenommen.)

- [ ] **Step 6: Smoke-Test**

```bash
pnpm tauri dev
```

Beim Start: Log-Zeile in Konsole, Datei `<app_data>/projektalpha/logs/projektalpha.log.YYYY-MM-DD` existiert mit JSON-Zeile `app starting`.

- [ ] **Step 7: Checkpoint**

> **Checkpoint 0.4:** Logging läuft. JSON-File + Konsole. Geprüft im App-Data-Ordner.

---

### Task 0.5: Error-Modul (TDD)

**Files:**
- Create: `src-tauri/src/error.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Failing Test schreiben**

`src-tauri/src/error.rs`:
```rust
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("json parse error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("internal: {0}")]
    Internal(String),
}

#[derive(Serialize)]
pub struct SerializableError {
    pub kind: String,
    pub message: String,
}

impl AppError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Database(_) => "database",
            Self::Network(_) => "network",
            Self::Json(_) => "json",
            Self::Io(_) => "io",
            Self::InvalidInput(_) => "invalid_input",
            Self::NotFound(_) => "not_found",
            Self::Internal(_) => "internal",
        }
    }
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        SerializableError { kind: self.kind().to_string(), message: self.to_string() }
            .serialize(s)
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_input_serializes_to_kind_and_message() {
        let err = AppError::InvalidInput("radius must be > 0".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"kind\":\"invalid_input\""));
        assert!(json.contains("radius must be > 0"));
    }
}
```

- [ ] **Step 2: main.rs einhängen, testen**

```rust
mod error;
```

```bash
cargo test --lib error
```

Erwartet: 1 passed.

- [ ] **Step 3: Checkpoint**

> **Checkpoint 0.5:** Zentrale Error-Typen verfügbar; serialisieren sich für IPC sauber.

---

### Task 0.6: SQLite-Setup + Migrations-Runner (TDD)

**Files:**
- Create: `src-tauri/src/db/mod.rs`, `src-tauri/src/db/migrations.rs`, `src-tauri/src/db/migrations/0001_initial.sql`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Initial-Schema (0001_initial.sql)**

`src-tauri/src/db/migrations/0001_initial.sql`:
```sql
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
```

- [ ] **Step 2: db/mod.rs schreiben**

`src-tauri/src/db/mod.rs`:
```rust
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::Path;
use std::str::FromStr;

pub mod migrations;

pub async fn open(db_path: &Path) -> anyhow::Result<SqlitePool> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new().max_connections(5).connect_with(options).await?;
    migrations::run(&pool).await?;
    Ok(pool)
}

#[cfg(test)]
pub async fn open_in_memory() -> SqlitePool {
    let options = SqliteConnectOptions::from_str("sqlite::memory:")
        .unwrap()
        .foreign_keys(true);
    let pool = SqlitePoolOptions::new().max_connections(1).connect_with(options).await.unwrap();
    migrations::run(&pool).await.unwrap();
    pool
}
```

- [ ] **Step 3: db/migrations.rs (TDD: erst Test)**

`src-tauri/src/db/migrations.rs`:
```rust
use sqlx::SqlitePool;

const MIGRATIONS: &[(&str, &str)] = &[
    ("0001_initial", include_str!("migrations/0001_initial.sql")),
];

pub async fn run(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL
        )"
    ).execute(pool).await?;

    for (version, sql) in MIGRATIONS {
        let already: Option<(String,)> = sqlx::query_as(
            "SELECT version FROM schema_migrations WHERE version = ?"
        ).bind(version).fetch_optional(pool).await?;

        if already.is_some() {
            tracing::debug!(version, "migration already applied");
            continue;
        }

        let started = std::time::Instant::now();
        let mut tx = pool.begin().await?;
        // sqlx kann mehrere Statements pro execute_many; wir nutzen den raw query
        for stmt in sql.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            sqlx::query(stmt).execute(&mut *tx).await?;
        }
        sqlx::query("INSERT INTO schema_migrations (version, applied_at) VALUES (?, datetime('now'))")
            .bind(version)
            .execute(&mut *tx).await?;
        tx.commit().await?;

        tracing::info!(version, dauer_ms = started.elapsed().as_millis() as u64, "migration applied");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[tokio::test]
    async fn migrations_create_all_tables() {
        let pool = open_in_memory().await;
        let tables: Vec<(String,)> = sqlx::query_as(
            "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
        ).fetch_all(&pool).await.unwrap();
        let names: Vec<&str> = tables.iter().map(|(n,)| n.as_str()).collect();
        for expected in ["activity_log","app_meta","companies","geocode_cache","industry_categories","schema_migrations","search_profiles"] {
            assert!(names.contains(&expected), "missing table: {expected}");
        }
    }

    #[tokio::test]
    async fn migrations_idempotent() {
        let pool = open_in_memory().await;
        run(&pool).await.unwrap();
        run(&pool).await.unwrap();
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM schema_migrations").fetch_one(&pool).await.unwrap();
        assert_eq!(count.0, 1);
    }
}
```

- [ ] **Step 4: main.rs einhängen, Tests**

`src-tauri/src/main.rs`:
```rust
mod db;
```

```bash
cargo test --lib db::migrations
```

Erwartet: 2 passed.

- [ ] **Step 5: Pool in main() öffnen + Tauri-State**

```rust
use std::sync::Arc;
use sqlx::SqlitePool;

pub struct AppState {
    pub db: SqlitePool,
}

fn main() {
    // ... logging init ...

    let app_data = dirs::data_dir()
        .expect("no data dir")
        .join("projektalpha");
    let db_path = app_data.join("data.db");

    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let pool = runtime.block_on(db::open(&db_path)).expect("db open");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(Arc::new(AppState { db: pool }))
        .run(tauri::generate_context!())
        .expect("run");
}
```

- [ ] **Step 6: Smoke-Test**

```bash
pnpm tauri dev
```

Erwartet: App startet, Logfile zeigt `migration applied version=0001_initial dauer_ms=...`. SQLite-Datei existiert in `~/Library/Application Support/projektalpha/data.db` (Mac).

- [ ] **Step 7: Checkpoint**

> **Checkpoint 0.6:** Schema migriert idempotent. DB persistent in App-Data.

---

### Task 0.7: Branchen-Seed (TDD)

**Files:**
- Create: `src-tauri/src/db/migrations/0002_seed_categories.sql`
- Modify: `src-tauri/src/db/migrations.rs` (Liste erweitern), Test ergänzen

- [ ] **Step 1: Failing Test ergänzen**

In `src-tauri/src/db/migrations.rs` `tests`-Modul:
```rust
#[tokio::test]
async fn seed_inserts_eleven_categories() {
    let pool = open_in_memory().await;
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM industry_categories")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(count.0, 11, "expected 11 seeded categories");
}

#[tokio::test]
async fn seed_logistik_has_weight_95() {
    let pool = open_in_memory().await;
    let weight: (i64,) = sqlx::query_as(
        "SELECT probability_weight FROM industry_categories WHERE name_de = 'Logistik / Spedition'"
    ).fetch_one(&pool).await.unwrap();
    assert_eq!(weight.0, 95);
}
```

- [ ] **Step 2: Test laufen (FAIL)**

```bash
cargo test --lib seed
```

Erwartet: panic „expected 11" (count = 0).

- [ ] **Step 3: Seed-SQL anlegen**

`src-tauri/src/db/migrations/0002_seed_categories.sql`:
```sql
INSERT INTO industry_categories (name_de, osm_tags, probability_weight, enabled, color, sort_order) VALUES
('Logistik / Spedition',          '[{"office":"logistics"},{"shop":"wholesale"}]',                 95, 1, '#ef4444', 10),
('Lebensmittel-Großhandel',       '[{"shop":"wholesale","wholesale":"food"}]',                     90, 1, '#f97316', 20),
('Lagerhalle / Warehouse',        '[{"industrial":"warehouse"},{"building":"warehouse"}]',         85, 1, '#f59e0b', 30),
('Industrielle Produktion',       '[{"building":"industrial"},{"landuse":"industrial"}]',          80, 1, '#eab308', 40),
('Baumarkt / DIY',                '[{"shop":"doityourself"},{"shop":"hardware"}]',                 80, 1, '#84cc16', 45),
('Lebensmittel-Einzelhandel',     '[{"shop":"supermarket"},{"shop":"convenience"}]',               75, 1, '#22c55e', 50),
('Möbel-/Bauhandel',              '[{"shop":"furniture"},{"shop":"trade"}]',                       70, 1, '#10b981', 60),
('Pharma / Kosmetik',             '[{"industrial":"chemical"}]',                                   65, 1, '#06b6d4', 70),
('Bäckerei (industriell)',        '[{"craft":"bakery"},{"shop":"bakery"}]',                        60, 1, '#3b82f6', 80),
('Autohaus',                      '[{"shop":"car"}]',                                              40, 1, '#8b5cf6', 90),
('Bürogebäude',                   '[{"building":"office"},{"office":"company"}]',                   5, 0, '#a855f7', 100);
```

- [ ] **Step 4: Migration in Liste registrieren**

`migrations.rs`:
```rust
const MIGRATIONS: &[(&str, &str)] = &[
    ("0001_initial", include_str!("migrations/0001_initial.sql")),
    ("0002_seed_categories", include_str!("migrations/0002_seed_categories.sql")),
];
```

- [ ] **Step 5: Tests laufen (PASS)**

```bash
cargo test --lib
```

Erwartet: alle inkl. seed-Tests grün.

- [ ] **Step 6: Checkpoint**

> **Checkpoint 0.7:** 11 Branchen-Seeds in DB. Logistik 95, Bürogebäude 5 (disabled).

---

### Task 0.8: Frontend ↔ Backend IPC-Smoke-Test

**Files:**
- Create: `src/lib/tauri.ts`
- Modify: `src-tauri/src/commands.rs` (neu), `src-tauri/src/main.rs`, `src/App.tsx`

- [ ] **Step 1: Rust-Command anlegen**

`src-tauri/src/commands.rs`:
```rust
use crate::error::AppResult;
use crate::AppState;
use serde::Serialize;
use std::sync::Arc;
use tauri::State;

#[derive(Serialize)]
pub struct CategoryRow {
    pub id: i64,
    pub name_de: String,
    pub probability_weight: i64,
    pub enabled: bool,
    pub color: String,
}

#[tauri::command]
pub async fn list_categories(state: State<'_, Arc<AppState>>) -> AppResult<Vec<CategoryRow>> {
    let rows: Vec<(i64, String, i64, i64, String)> = sqlx::query_as(
        "SELECT id, name_de, probability_weight, enabled, color FROM industry_categories ORDER BY sort_order"
    ).fetch_all(&state.db).await?;
    Ok(rows.into_iter().map(|(id, name_de, w, enabled, color)| CategoryRow {
        id, name_de, probability_weight: w, enabled: enabled != 0, color
    }).collect())
}
```

- [ ] **Step 2: In main.rs registrieren**

```rust
mod commands;
// im Builder:
.invoke_handler(tauri::generate_handler![commands::list_categories])
```

- [ ] **Step 3: TS-Wrapper**

`src/lib/tauri.ts`:
```ts
import { invoke } from "@tauri-apps/api/core"

export type CategoryRow = {
  id: number
  name_de: string
  probability_weight: number
  enabled: boolean
  color: string
}

export const api = {
  listCategories: () => invoke<CategoryRow[]>("list_categories"),
}
```

- [ ] **Step 4: App.tsx einfache Anzeige**

```tsx
import { useEffect, useState } from "react"
import { Button } from "@/components/ui/button"
import { api, type CategoryRow } from "@/lib/tauri"

function App() {
  const [cats, setCats] = useState<CategoryRow[]>([])
  const [err, setErr] = useState<string | null>(null)

  const load = async () => {
    try { setCats(await api.listCategories()); setErr(null) }
    catch (e) { setErr(String(e)) }
  }

  useEffect(() => { load() }, [])

  return (
    <div className="p-8 space-y-4">
      <h1 className="text-3xl font-bold">ProjektAlpha</h1>
      <Button onClick={load}>Branchen neu laden</Button>
      {err && <pre className="text-red-600">{err}</pre>}
      <ul className="space-y-1">
        {cats.map(c => (
          <li key={c.id} className="flex gap-2 items-center">
            <span className="inline-block w-3 h-3 rounded" style={{ background: c.color }} />
            <span className={c.enabled ? "" : "opacity-50 line-through"}>{c.name_de}</span>
            <span className="text-sm text-gray-500">({c.probability_weight}%)</span>
          </li>
        ))}
      </ul>
    </div>
  )
}
export default App
```

- [ ] **Step 5: Smoke-Test**

```bash
pnpm tauri dev
```

Erwartet: Fenster zeigt 11 Branchen-Zeilen mit Farbpunkt + Score + „Bürogebäude" durchgestrichen.

- [ ] **Step 6: Checkpoint**

> **Checkpoint 0.8: Phase 0 fertig.** UI lädt echte DB-Daten via Tauri-Command. Logging + Error-Path live. **Bereit für Overpass-Integration.**

---

# PHASE 1 – Core-Suche

Ziel: Tauri-Command `start_search` ausführbar mit `{center, radius_km, enabled_category_ids}`, läuft Overpass-Queries (mit Tile-Splitting, Endpoint-Rotation, Retry, Logging), parst Ergebnisse, importiert in DB. Validiert per Tests + Live-Smoke gegen kleinen Radius.

---

### Task 1.1: Companies-Repository (TDD)

**Files:**
- Create: `src-tauri/src/db/companies.rs`
- Modify: `src-tauri/src/db/mod.rs`

- [ ] **Step 1: Struct + Failing Test**

`src-tauri/src/db/companies.rs`:
```rust
use crate::error::AppResult;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCompany {
    pub osm_id: Option<String>,
    pub name: String,
    pub street: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub country: String,
    pub lat: f64,
    pub lng: f64,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub industry_category_id: Option<i64>,
    pub size_estimate: Option<String>,
    pub probability_score: i64,
    pub source: String, // "osm" | "manual"
}

#[derive(Debug, Serialize)]
pub struct InsertResult { pub inserted: bool, pub updated_fields: Vec<&'static str> }

pub async fn insert_or_merge(pool: &SqlitePool, c: &NewCompany) -> AppResult<InsertResult> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    if let Some(osm_id) = &c.osm_id {
        // existing?
        let existing: Option<(String, String)> = sqlx::query_as(
            "SELECT id, source FROM companies WHERE osm_id = ?"
        ).bind(osm_id).fetch_optional(pool).await?;

        if let Some((existing_id, source)) = existing {
            // manuelle Einträge sind tabu
            if source == "manual" {
                return Ok(InsertResult { inserted: false, updated_fields: vec![] });
            }
            // sonst: leere Felder mit OSM-Daten füllen
            let mut updated = vec![];
            macro_rules! maybe_update {
                ($field:literal, $val:expr) => {
                    if let Some(v) = &$val {
                        let was: (Option<String>,) = sqlx::query_as(
                            &format!("SELECT {} FROM companies WHERE id = ?", $field)
                        ).bind(&existing_id).fetch_one(pool).await?;
                        if was.0.is_none() {
                            sqlx::query(&format!("UPDATE companies SET {} = ?, updated_at = ? WHERE id = ?", $field))
                                .bind(v).bind(&now).bind(&existing_id).execute(pool).await?;
                            updated.push($field);
                        }
                    }
                };
            }
            maybe_update!("phone", c.phone);
            maybe_update!("email", c.email);
            maybe_update!("website", c.website);
            return Ok(InsertResult { inserted: false, updated_fields: updated });
        }
    }

    sqlx::query(
        "INSERT INTO companies (id, osm_id, name, street, postal_code, city, country, lat, lng, phone, email, website, industry_category_id, size_estimate, probability_score, status, source, created_at, updated_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'neu', ?, ?, ?)"
    )
    .bind(&id).bind(&c.osm_id).bind(&c.name)
    .bind(&c.street).bind(&c.postal_code).bind(&c.city).bind(&c.country)
    .bind(c.lat).bind(c.lng)
    .bind(&c.phone).bind(&c.email).bind(&c.website)
    .bind(c.industry_category_id).bind(&c.size_estimate).bind(c.probability_score)
    .bind(&c.source).bind(&now).bind(&now)
    .execute(pool).await?;

    Ok(InsertResult { inserted: true, updated_fields: vec![] })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    fn sample(osm_id: Option<&str>, name: &str) -> NewCompany {
        NewCompany {
            osm_id: osm_id.map(String::from),
            name: name.to_string(),
            street: None, postal_code: None, city: Some("Hannover".into()), country: "DE".into(),
            lat: 52.37, lng: 9.73,
            phone: None, email: None, website: None,
            industry_category_id: Some(1),
            size_estimate: None, probability_score: 95,
            source: "osm".into(),
        }
    }

    #[tokio::test]
    async fn first_insert_succeeds() {
        let pool = open_in_memory().await;
        let r = insert_or_merge(&pool, &sample(Some("node/123"), "Müller GmbH")).await.unwrap();
        assert!(r.inserted);
    }

    #[tokio::test]
    async fn duplicate_osm_id_skipped() {
        let pool = open_in_memory().await;
        insert_or_merge(&pool, &sample(Some("node/1"), "A")).await.unwrap();
        let r = insert_or_merge(&pool, &sample(Some("node/1"), "A")).await.unwrap();
        assert!(!r.inserted);
    }

    #[tokio::test]
    async fn manual_entries_never_overwritten() {
        let pool = open_in_memory().await;
        let mut manual = sample(Some("node/9"), "Manual Co");
        manual.source = "manual".into();
        manual.phone = Some("0511-1".into());
        insert_or_merge(&pool, &manual).await.unwrap();

        let mut osm = sample(Some("node/9"), "OSM Co");
        osm.phone = Some("OVERRIDE".into());
        let r = insert_or_merge(&pool, &osm).await.unwrap();

        assert!(!r.inserted);
        assert!(r.updated_fields.is_empty());

        let phone: (String,) = sqlx::query_as("SELECT phone FROM companies WHERE osm_id = 'node/9'")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(phone.0, "0511-1");
    }

    #[tokio::test]
    async fn osm_fills_empty_fields_only() {
        let pool = open_in_memory().await;
        let first = sample(Some("node/2"), "X"); // no phone
        insert_or_merge(&pool, &first).await.unwrap();

        let mut second = sample(Some("node/2"), "X");
        second.phone = Some("123".into());
        let r = insert_or_merge(&pool, &second).await.unwrap();
        assert!(r.updated_fields.contains(&"phone"));

        // dritter Lauf mit anderem Phone darf nicht überschreiben
        let mut third = sample(Some("node/2"), "X");
        third.phone = Some("999".into());
        let r3 = insert_or_merge(&pool, &third).await.unwrap();
        assert!(r3.updated_fields.is_empty());
    }
}
```

- [ ] **Step 2: db/mod.rs ergänzen**

```rust
pub mod companies;
```

- [ ] **Step 3: Tests laufen**

```bash
cargo test --lib db::companies
```

Erwartet: 4 passed.

- [ ] **Step 4: Checkpoint**

> **Checkpoint 1.1:** Companies-Insert mit Duplicate- und Merge-Logik abgesichert.

---

### Task 1.2: Categories-Lookup (TDD, klein)

**Files:**
- Create: `src-tauri/src/db/categories.rs`
- Modify: `src-tauri/src/db/mod.rs`

- [ ] **Step 1: Modul + Test**

```rust
use crate::error::AppResult;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name_de: String,
    pub osm_tags: String, // raw JSON
    pub probability_weight: i64,
    pub enabled: bool,
    pub color: String,
}

pub async fn list_enabled(pool: &SqlitePool) -> AppResult<Vec<Category>> {
    let rows: Vec<(i64, String, String, i64, i64, String)> = sqlx::query_as(
        "SELECT id, name_de, osm_tags, probability_weight, enabled, color FROM industry_categories WHERE enabled = 1 ORDER BY sort_order"
    ).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|(id,name_de,osm_tags,w,e,color)| Category{
        id, name_de, osm_tags, probability_weight: w, enabled: e!=0, color
    }).collect())
}

pub async fn list_by_ids(pool: &SqlitePool, ids: &[i64]) -> AppResult<Vec<Category>> {
    if ids.is_empty() { return Ok(vec![]); }
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let q = format!("SELECT id, name_de, osm_tags, probability_weight, enabled, color FROM industry_categories WHERE id IN ({}) ORDER BY sort_order", placeholders);
    let mut query = sqlx::query_as::<_, (i64, String, String, i64, i64, String)>(&q);
    for id in ids { query = query.bind(id); }
    let rows = query.fetch_all(pool).await?;
    Ok(rows.into_iter().map(|(id,name_de,osm_tags,w,e,color)| Category{
        id, name_de, osm_tags, probability_weight: w, enabled: e!=0, color
    }).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[tokio::test]
    async fn list_enabled_excludes_disabled() {
        let pool = open_in_memory().await;
        let cats = list_enabled(&pool).await.unwrap();
        assert_eq!(cats.len(), 10); // 11 seeds, 1 disabled
        assert!(cats.iter().all(|c| c.enabled));
    }

    #[tokio::test]
    async fn list_by_ids_returns_ordered() {
        let pool = open_in_memory().await;
        let cats = list_by_ids(&pool, &[2, 1]).await.unwrap();
        assert_eq!(cats.len(), 2);
        assert_eq!(cats[0].id, 1); // sort_order 10 < 20
    }
}
```

- [ ] **Step 2: Hängen + Tests**

`db/mod.rs`: `pub mod categories;`

```bash
cargo test --lib db::categories
```

Erwartet: 2 passed.

- [ ] **Step 3: Checkpoint**

> **Checkpoint 1.2:** Categories-Lookups grün.

---

### Task 1.3: Tile-Splitter (TDD)

**Files:**
- Create: `src-tauri/src/overpass/mod.rs`, `src-tauri/src/overpass/tile_splitter.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Tests zuerst**

`src-tauri/src/overpass/tile_splitter.rs`:
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tile {
    pub center_lat: f64,
    pub center_lng: f64,
    pub radius_m: u32,
}

pub fn split(center_lat: f64, center_lng: f64, radius_km: u32) -> Vec<Tile> {
    if radius_km <= 50 {
        return vec![Tile { center_lat, center_lng, radius_m: radius_km * 1000 }];
    }
    if radius_km <= 150 {
        let half = radius_km as f64 / 2.0;
        let lat_off = km_to_lat_deg(half);
        let lng_off = km_to_lng_deg(half, center_lat);
        let r_m = (half * 1000.0) as u32;
        return vec![
            Tile { center_lat: center_lat + lat_off, center_lng: center_lng + lng_off, radius_m: r_m },
            Tile { center_lat: center_lat + lat_off, center_lng: center_lng - lng_off, radius_m: r_m },
            Tile { center_lat: center_lat - lat_off, center_lng: center_lng + lng_off, radius_m: r_m },
            Tile { center_lat: center_lat - lat_off, center_lng: center_lng - lng_off, radius_m: r_m },
        ];
    }
    // > 150 km: Grid mit 50 km Kacheln, Circle-Clip
    let cell_km = 50.0_f64;
    let n = (2.0 * radius_km as f64 / cell_km).ceil() as i32;
    let half_n = n / 2;
    let lat_step = km_to_lat_deg(cell_km);

    let mut tiles = vec![];
    for i in -half_n..=half_n {
        for j in -half_n..=half_n {
            let lat = center_lat + (i as f64) * lat_step;
            let lng_step = km_to_lng_deg(cell_km, lat);
            let lng = center_lng + (j as f64) * lng_step;
            let dist_km = haversine_km(center_lat, center_lng, lat, lng);
            if dist_km <= radius_km as f64 + cell_km / 2.0 {
                tiles.push(Tile { center_lat: lat, center_lng: lng, radius_m: 25_000 });
            }
        }
    }
    tiles
}

fn km_to_lat_deg(km: f64) -> f64 { km / 111.0 }
fn km_to_lng_deg(km: f64, at_lat: f64) -> f64 { km / (111.0 * at_lat.to_radians().cos()) }

fn haversine_km(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> f64 {
    let r = 6371.0_f64;
    let dlat = (lat2 - lat1).to_radians();
    let dlng = (lng2 - lng1).to_radians();
    let a = (dlat/2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlng/2.0).sin().powi(2);
    2.0 * r * a.sqrt().asin()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_radius_returns_single_tile() {
        let tiles = split(52.37, 9.73, 50);
        assert_eq!(tiles.len(), 1);
        assert_eq!(tiles[0].radius_m, 50_000);
    }

    #[test]
    fn medium_radius_returns_four_quadrants() {
        let tiles = split(52.37, 9.73, 100);
        assert_eq!(tiles.len(), 4);
        assert!(tiles.iter().all(|t| t.radius_m == 50_000));
    }

    #[test]
    fn large_radius_returns_grid_within_circle() {
        let tiles = split(52.37, 9.73, 300);
        assert!(tiles.len() > 4, "got {}", tiles.len());
        // Alle Tile-Mittelpunkte sollen innerhalb radius+padding liegen
        for t in &tiles {
            let d = haversine_km(52.37, 9.73, t.center_lat, t.center_lng);
            assert!(d <= 300.0 + 25.0, "tile {:?} too far ({} km)", t, d);
        }
    }

    #[test]
    fn large_radius_covers_center_and_edges() {
        let tiles = split(52.37, 9.73, 200);
        // Center-Tile vorhanden
        let any_near_center = tiles.iter().any(|t| {
            haversine_km(52.37, 9.73, t.center_lat, t.center_lng) < 30.0
        });
        assert!(any_near_center, "no tile near center");
        // Mind. ein Tile am äußeren Rand (>150km)
        let any_far = tiles.iter().any(|t| {
            haversine_km(52.37, 9.73, t.center_lat, t.center_lng) > 150.0
        });
        assert!(any_far, "no outer-ring tile");
    }
}
```

- [ ] **Step 2: overpass/mod.rs + main.rs einhängen**

`src-tauri/src/overpass/mod.rs`:
```rust
pub mod tile_splitter;
```

`main.rs`: `mod overpass;`

- [ ] **Step 3: Tests**

```bash
cargo test --lib overpass::tile_splitter
```

Erwartet: 4 passed.

- [ ] **Step 4: Checkpoint**

> **Checkpoint 1.3:** Tile-Splitter deterministisch & getestet für 50/100/200/300 km.

---

### Task 1.4: Query-Builder (TDD)

**Files:**
- Create: `src-tauri/src/overpass/query_builder.rs`
- Modify: `src-tauri/src/overpass/mod.rs`

- [ ] **Step 1: Test zuerst**

`src-tauri/src/overpass/query_builder.rs`:
```rust
use crate::db::categories::Category;
use crate::error::{AppError, AppResult};
use crate::overpass::tile_splitter::Tile;
use serde_json::Value;

/// osm_tags JSON-Format:
/// - Outer Array = Liste OR-verknüpfter Regeln
/// - Jedes Object = Tag-Bedingungen (UND-verknüpft)
/// Beispiel: `[{"shop":"wholesale","wholesale":"food"}, {"shop":"supermarket"}]`
/// → matched: (shop=wholesale AND wholesale=food) ODER (shop=supermarket)
pub fn build(categories: &[Category], tile: &Tile) -> AppResult<String> {
    if categories.is_empty() {
        return Err(AppError::InvalidInput("no categories".into()));
    }
    let mut out = String::from("[out:json][timeout:25];\n(\n");
    for cat in categories {
        let rules: Value = serde_json::from_str(&cat.osm_tags)
            .map_err(|e| AppError::Internal(format!("bad osm_tags for cat {}: {}", cat.id, e)))?;
        let arr = rules.as_array().ok_or_else(|| AppError::Internal("osm_tags not an array".into()))?;
        for rule in arr {
            let obj = rule.as_object().ok_or_else(|| AppError::Internal("rule not an object".into()))?;
            let mut conds = String::new();
            for (k, v) in obj {
                let vs = v.as_str().ok_or_else(|| AppError::Internal("tag value not string".into()))?;
                conds.push_str(&format!("[\"{}\"=\"{}\"]", k, vs));
            }
            out.push_str(&format!(
                "  nwr{}(around:{},{},{});\n",
                conds, tile.radius_m, tile.center_lat, tile.center_lng
            ));
        }
    }
    out.push_str(");\nout center tags;\n");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cat(id: i64, tags: &str, w: i64) -> Category {
        Category { id, name_de: format!("c{id}"), osm_tags: tags.into(), probability_weight: w, enabled: true, color: "#000".into() }
    }

    #[test]
    fn single_or_rule() {
        let c = cat(1, r#"[{"shop":"supermarket"}]"#, 75);
        let t = Tile { center_lat: 52.0, center_lng: 9.0, radius_m: 50_000 };
        let q = build(&[c], &t).unwrap();
        assert!(q.contains("[\"shop\"=\"supermarket\"]"));
        assert!(q.contains("(around:50000,52,9)"));
        assert!(q.starts_with("[out:json]"));
        assert!(q.contains("out center tags"));
    }

    #[test]
    fn multi_tag_and_within_one_rule() {
        let c = cat(1, r#"[{"shop":"wholesale","wholesale":"food"}]"#, 90);
        let t = Tile { center_lat: 50.0, center_lng: 8.0, radius_m: 25_000 };
        let q = build(&[c], &t).unwrap();
        assert!(q.contains("[\"shop\"=\"wholesale\"][\"wholesale\"=\"food\"]") ||
                q.contains("[\"wholesale\"=\"food\"][\"shop\"=\"wholesale\"]"));
    }

    #[test]
    fn empty_categories_errors() {
        let t = Tile { center_lat: 0.0, center_lng: 0.0, radius_m: 1000 };
        assert!(build(&[], &t).is_err());
    }

    #[test]
    fn malformed_tags_errors() {
        let c = cat(1, r#"not json"#, 50);
        let t = Tile { center_lat: 0.0, center_lng: 0.0, radius_m: 1000 };
        assert!(build(&[c], &t).is_err());
    }
}
```

- [ ] **Step 2: mod.rs**

```rust
pub mod query_builder;
```

- [ ] **Step 3: Tests**

```bash
cargo test --lib overpass::query_builder
```

Erwartet: 4 passed.

- [ ] **Step 4: Checkpoint**

> **Checkpoint 1.4:** QL-Generator korrekt und robust gegen kaputte Seeds.

---

### Task 1.5: Scoring & Tag-Matching (TDD)

**Files:**
- Create: `src-tauri/src/overpass/scoring.rs`
- Modify: `src-tauri/src/overpass/mod.rs`

- [ ] **Step 1: Test zuerst**

`src-tauri/src/overpass/scoring.rs`:
```rust
use crate::db::categories::Category;
use std::collections::HashMap;

/// Findet erste Kategorie, deren osm_tags-Regel für die gegebenen tags zutrifft.
pub fn match_category<'a>(tags: &HashMap<String, String>, categories: &'a [Category]) -> Option<&'a Category> {
    for cat in categories {
        let rules: serde_json::Value = match serde_json::from_str(&cat.osm_tags) {
            Ok(v) => v, Err(_) => continue,
        };
        let arr = match rules.as_array() { Some(a) => a, None => continue };
        for rule in arr {
            let obj = match rule.as_object() { Some(o) => o, None => continue };
            if obj.iter().all(|(k, v)| {
                tags.get(k).map(|tv| Some(tv.as_str()) == v.as_str()).unwrap_or(false)
            }) {
                return Some(cat);
            }
        }
    }
    None
}

pub fn score_for_category(cat: &Category) -> i64 {
    cat.probability_weight
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cat(id: i64, tags: &str, w: i64) -> Category {
        Category { id, name_de: format!("c{id}"), osm_tags: tags.into(), probability_weight: w, enabled: true, color: "#000".into() }
    }

    fn tagmap(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs.iter().map(|(k,v)| (k.to_string(), v.to_string())).collect()
    }

    #[test]
    fn matches_single_tag() {
        let cs = vec![cat(1, r#"[{"shop":"supermarket"}]"#, 75)];
        let m = match_category(&tagmap(&[("shop","supermarket")]), &cs);
        assert!(m.is_some());
    }

    #[test]
    fn requires_all_tags_in_rule() {
        let cs = vec![cat(1, r#"[{"shop":"wholesale","wholesale":"food"}]"#, 90)];
        // teilweise: kein Match
        assert!(match_category(&tagmap(&[("shop","wholesale")]), &cs).is_none());
        // vollständig: Match
        assert!(match_category(&tagmap(&[("shop","wholesale"),("wholesale","food")]), &cs).is_some());
    }

    #[test]
    fn or_alternative_rules() {
        let cs = vec![cat(1, r#"[{"shop":"a"},{"shop":"b"}]"#, 50)];
        assert!(match_category(&tagmap(&[("shop","a")]), &cs).is_some());
        assert!(match_category(&tagmap(&[("shop","b")]), &cs).is_some());
        assert!(match_category(&tagmap(&[("shop","c")]), &cs).is_none());
    }

    #[test]
    fn first_matching_category_wins() {
        let cs = vec![
            cat(1, r#"[{"shop":"supermarket"}]"#, 75),
            cat(2, r#"[{"shop":"supermarket"}]"#, 99),
        ];
        let m = match_category(&tagmap(&[("shop","supermarket")]), &cs).unwrap();
        assert_eq!(m.id, 1);
    }
}
```

- [ ] **Step 2: mod + Tests**

`overpass/mod.rs`: `pub mod scoring;`

```bash
cargo test --lib overpass::scoring
```

Erwartet: 4 passed.

- [ ] **Step 3: Checkpoint**

> **Checkpoint 1.5:** Scoring + Tag-Match korrekt (UND innerhalb Regel, ODER zwischen Regeln).

---

### Task 1.6: Overpass-JSON-Parser (TDD mit Fixtures)

**Files:**
- Create: `src-tauri/src/overpass/parser.rs`, `src-tauri/tests/fixtures/overpass_simple.json`, `src-tauri/tests/fixtures/overpass_with_polygons.json`
- Modify: `src-tauri/src/overpass/mod.rs`

- [ ] **Step 1: Fixture: einfacher Treffer**

`src-tauri/tests/fixtures/overpass_simple.json`:
```json
{
  "version": 0.6,
  "generator": "Overpass API",
  "elements": [
    {
      "type": "node",
      "id": 12345,
      "lat": 52.3756,
      "lon": 9.7320,
      "tags": {
        "name": "Müller Logistik GmbH",
        "shop": "wholesale",
        "wholesale": "food",
        "addr:street": "Industriestr.",
        "addr:housenumber": "12",
        "addr:postcode": "30659",
        "addr:city": "Hannover",
        "phone": "+49 511 1234567",
        "email": "info@mueller-test.example",
        "website": "https://mueller-test.example"
      }
    }
  ]
}
```

- [ ] **Step 2: Fixture: Polygon (way mit center)**

`src-tauri/tests/fixtures/overpass_with_polygons.json`:
```json
{
  "elements": [
    {
      "type": "way",
      "id": 99999,
      "center": { "lat": 53.0, "lon": 10.0 },
      "tags": {
        "industrial": "warehouse",
        "name": "Test Warehouse"
      }
    },
    {
      "type": "node",
      "id": 88888,
      "lat": 0.0,
      "lon": 0.0,
      "tags": { "shop": "fancy", "name": "No Match" }
    }
  ]
}
```

- [ ] **Step 3: Parser-Modul + Tests**

`src-tauri/src/overpass/parser.rs`:
```rust
use crate::db::categories::Category;
use crate::db::companies::NewCompany;
use crate::error::{AppError, AppResult};
use crate::overpass::scoring::{match_category, score_for_category};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
struct Response { elements: Vec<Element> }

#[derive(Deserialize)]
struct Element {
    #[serde(rename = "type")] kind: String,
    id: i64,
    lat: Option<f64>,
    lon: Option<f64>,
    center: Option<Center>,
    #[serde(default)]
    tags: HashMap<String, String>,
}

#[derive(Deserialize)]
struct Center { lat: f64, lon: f64 }

pub fn parse(json: &str, categories: &[Category]) -> AppResult<Vec<NewCompany>> {
    let resp: Response = serde_json::from_str(json)?;
    let mut out = Vec::new();
    for el in resp.elements {
        let (lat, lng) = match (el.lat, el.lon, &el.center) {
            (Some(la), Some(lo), _) => (la, lo),
            (_, _, Some(c)) => (c.lat, c.lon),
            _ => continue,
        };

        let cat = match match_category(&el.tags, categories) {
            Some(c) => c,
            None => continue, // keine bekannte Branche → skip
        };

        let osm_id = format!("{}/{}", el.kind, el.id);
        let name = el.tags.get("name").cloned()
            .unwrap_or_else(|| {
                let plz = el.tags.get("addr:postcode").map(String::as_str).unwrap_or("?");
                let city = el.tags.get("addr:city").map(String::as_str).unwrap_or("");
                format!("Unbenannt ({} {})", plz, city).trim().to_string()
            });

        let street = el.tags.get("addr:street").map(|s| {
            let nr = el.tags.get("addr:housenumber").map(String::as_str).unwrap_or("");
            format!("{} {}", s, nr).trim().to_string()
        });

        out.push(NewCompany {
            osm_id: Some(osm_id),
            name,
            street,
            postal_code: el.tags.get("addr:postcode").cloned(),
            city: el.tags.get("addr:city").cloned(),
            country: el.tags.get("addr:country").cloned().unwrap_or_else(|| "DE".into()),
            lat, lng,
            phone: el.tags.get("phone").cloned().or_else(|| el.tags.get("contact:phone").cloned()),
            email: el.tags.get("email").cloned().or_else(|| el.tags.get("contact:email").cloned()),
            website: el.tags.get("website").cloned().or_else(|| el.tags.get("contact:website").cloned()),
            industry_category_id: Some(cat.id),
            size_estimate: None,
            probability_score: score_for_category(cat),
            source: "osm".into(),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeds() -> Vec<Category> {
        vec![
            Category { id: 1, name_de: "LM-GH".into(), osm_tags: r#"[{"shop":"wholesale","wholesale":"food"}]"#.into(), probability_weight: 90, enabled: true, color: "#000".into() },
            Category { id: 2, name_de: "Lager".into(), osm_tags: r#"[{"industrial":"warehouse"}]"#.into(), probability_weight: 85, enabled: true, color: "#000".into() },
        ]
    }

    #[test]
    fn parses_node_with_full_address() {
        let json = include_str!("../../tests/fixtures/overpass_simple.json");
        let r = parse(json, &seeds()).unwrap();
        assert_eq!(r.len(), 1);
        let c = &r[0];
        assert_eq!(c.name, "Müller Logistik GmbH");
        assert_eq!(c.osm_id.as_deref(), Some("node/12345"));
        assert_eq!(c.street.as_deref(), Some("Industriestr. 12"));
        assert_eq!(c.postal_code.as_deref(), Some("30659"));
        assert_eq!(c.industry_category_id, Some(1));
        assert_eq!(c.probability_score, 90);
    }

    #[test]
    fn parses_way_via_center_and_skips_unknown() {
        let json = include_str!("../../tests/fixtures/overpass_with_polygons.json");
        let r = parse(json, &seeds()).unwrap();
        assert_eq!(r.len(), 1, "polygon match wins, unknown shop skipped");
        assert_eq!(r[0].osm_id.as_deref(), Some("way/99999"));
        assert_eq!(r[0].lat, 53.0);
        assert_eq!(r[0].industry_category_id, Some(2));
    }
}
```

- [ ] **Step 4: mod + Tests**

`overpass/mod.rs`: `pub mod parser;`

```bash
cargo test --lib overpass::parser
```

Erwartet: 2 passed.

- [ ] **Step 5: Checkpoint**

> **Checkpoint 1.6:** Parser konsumiert echte Overpass-Strukturen, fällt nicht über fehlende Felder.

---

### Task 1.7: Overpass-Client mit Endpoint-Rotation (TDD via mockito)

**Files:**
- Create: `src-tauri/src/overpass/client.rs`
- Modify: `src-tauri/src/overpass/mod.rs`

- [ ] **Step 1: Tests zuerst (mit mockito)**

`src-tauri/src/overpass/client.rs`:
```rust
use crate::error::{AppError, AppResult};
use std::time::Duration;

pub struct OverpassClient {
    endpoints: Vec<String>,
    http: reqwest::Client,
    max_retries: u32,
}

impl OverpassClient {
    pub fn new(endpoints: Vec<String>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .user_agent("ProjektAlpha/0.1")
            .build()
            .expect("reqwest client");
        Self { endpoints, http, max_retries: 3 }
    }

    pub fn default_endpoints() -> Vec<String> {
        vec![
            "https://overpass-api.de/api/interpreter".into(),
            "https://overpass.kumi.systems/api/interpreter".into(),
            "https://overpass.private.coffee/api/interpreter".into(),
        ]
    }

    pub async fn run_query(&self, ql: &str) -> AppResult<String> {
        let mut last_err: Option<AppError> = None;
        for endpoint in &self.endpoints {
            for attempt in 0..self.max_retries {
                let started = std::time::Instant::now();
                let res = self.http.post(endpoint).body(ql.to_string()).send().await;
                let dauer_ms = started.elapsed().as_millis() as u64;
                match res {
                    Ok(r) if r.status().is_success() => {
                        let text = r.text().await?;
                        tracing::debug!(endpoint, attempt, dauer_ms, bytes = text.len(), "overpass success");
                        return Ok(text);
                    }
                    Ok(r) => {
                        let status = r.status();
                        tracing::warn!(endpoint, attempt, dauer_ms, http_status = status.as_u16(), "overpass non-2xx");
                        last_err = Some(AppError::Internal(format!("http {}", status)));
                        // bei 5xx oder 429 retry; bei 4xx anderem direkt brechen
                        if !(status.is_server_error() || status.as_u16() == 429) { break; }
                    }
                    Err(e) => {
                        tracing::warn!(endpoint, attempt, dauer_ms, fehler = %e, "overpass network error");
                        last_err = Some(e.into());
                    }
                }
                let backoff_ms = 500 * 2_u64.pow(attempt);
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }
            tracing::warn!(endpoint, "endpoint exhausted, rotating");
        }
        Err(last_err.unwrap_or_else(|| AppError::Internal("all endpoints exhausted".into())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn returns_body_on_first_success() {
        let mut server = mockito::Server::new_async().await;
        let m = server.mock("POST", "/").with_status(200).with_body("{\"elements\":[]}").create_async().await;

        let client = OverpassClient::new(vec![server.url() + "/"]);
        let body = client.run_query("test").await.unwrap();
        assert!(body.contains("elements"));
        m.assert_async().await;
    }

    #[tokio::test]
    async fn retries_on_500_then_succeeds() {
        let mut server = mockito::Server::new_async().await;
        let _m1 = server.mock("POST", "/").with_status(500).expect(1).create_async().await;
        let _m2 = server.mock("POST", "/").with_status(200).with_body("ok").create_async().await;

        let mut client = OverpassClient::new(vec![server.url() + "/"]);
        client.max_retries = 2;
        let body = client.run_query("q").await.unwrap();
        assert_eq!(body, "ok");
    }

    #[tokio::test]
    async fn rotates_endpoint_when_all_retries_fail() {
        let mut server_a = mockito::Server::new_async().await;
        let _ma = server_a.mock("POST", "/").with_status(503).expect(2).create_async().await;
        let mut server_b = mockito::Server::new_async().await;
        let _mb = server_b.mock("POST", "/").with_status(200).with_body("from-b").create_async().await;

        let mut client = OverpassClient::new(vec![server_a.url() + "/", server_b.url() + "/"]);
        client.max_retries = 2;
        let body = client.run_query("q").await.unwrap();
        assert_eq!(body, "from-b");
    }
}
```

- [ ] **Step 2: mod + Tests**

`overpass/mod.rs`: `pub mod client;`

```bash
cargo test --lib overpass::client
```

Erwartet: 3 passed (Tests dauern wegen Backoff einige Sekunden).

- [ ] **Step 3: Checkpoint**

> **Checkpoint 1.7:** Client retry'd, rotiert, loggt – ohne PII.

---

### Task 1.8: Search-Orchestrator (TDD)

**Files:**
- Create: `src-tauri/src/overpass/search.rs`
- Modify: `src-tauri/src/overpass/mod.rs`

- [ ] **Step 1: Modul + Test**

`src-tauri/src/overpass/search.rs`:
```rust
use crate::db::{categories, companies::{insert_or_merge, NewCompany}};
use crate::error::{AppError, AppResult};
use crate::overpass::{client::OverpassClient, parser::parse, query_builder::build, tile_splitter::split};
use serde::Serialize;
use sqlx::SqlitePool;
use std::time::Duration;

#[derive(Serialize, Clone, Debug)]
pub struct ProgressEvent {
    pub tile_idx: usize,
    pub tile_total: usize,
    pub last_count: usize,
    pub running_total_inserted: usize,
}

#[derive(Serialize, Debug)]
pub struct SearchStats {
    pub total_found: usize,
    pub neu_imported: usize,
    pub duplicates_skipped: usize,
    pub dauer_ms: u64,
}

pub struct SearchInput {
    pub center_lat: f64,
    pub center_lng: f64,
    pub radius_km: u32,
    pub category_ids: Vec<i64>,
}

pub async fn run<F>(pool: &SqlitePool, client: &OverpassClient, input: SearchInput, mut on_progress: F) -> AppResult<SearchStats>
where F: FnMut(ProgressEvent) {
    if !(1..=300).contains(&input.radius_km) {
        return Err(AppError::InvalidInput("radius_km must be 1..=300".into()));
    }
    let cats = categories::list_by_ids(pool, &input.category_ids).await?;
    if cats.is_empty() {
        return Err(AppError::InvalidInput("no enabled categories selected".into()));
    }
    let tiles = split(input.center_lat, input.center_lng, input.radius_km);
    let started = std::time::Instant::now();
    tracing::info!(
        center_lat = input.center_lat, center_lng = input.center_lng,
        radius_km = input.radius_km, n_categories = cats.len(), n_tiles = tiles.len(),
        "search start"
    );

    let mut total_found = 0usize;
    let mut neu_imported = 0usize;
    let mut duplicates = 0usize;

    for (idx, tile) in tiles.iter().enumerate() {
        let ql = build(&cats, tile)?;
        let body = client.run_query(&ql).await?;
        let companies = parse(&body, &cats)?;
        total_found += companies.len();

        let mut tile_inserted = 0usize;
        for company in companies {
            let r = insert_or_merge(pool, &company).await?;
            if r.inserted { neu_imported += 1; tile_inserted += 1; } else { duplicates += 1; }
        }

        on_progress(ProgressEvent {
            tile_idx: idx + 1,
            tile_total: tiles.len(),
            last_count: tile_inserted,
            running_total_inserted: neu_imported,
        });

        // Etiquette: 1s Pause zwischen Tiles
        if idx + 1 < tiles.len() {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    let stats = SearchStats {
        total_found, neu_imported, duplicates_skipped: duplicates,
        dauer_ms: started.elapsed().as_millis() as u64,
    };
    tracing::info!(
        total_found = stats.total_found, neu_imported = stats.neu_imported,
        duplicates = stats.duplicates_skipped, dauer_ms = stats.dauer_ms,
        "search done"
    );
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;
    use mockito;

    #[tokio::test]
    async fn invalid_radius_rejected() {
        let pool = open_in_memory().await;
        let client = OverpassClient::new(vec!["http://127.0.0.1:9999".into()]);
        let r = run(&pool, &client, SearchInput {
            center_lat: 52.0, center_lng: 9.0, radius_km: 0, category_ids: vec![1],
        }, |_| {}).await;
        assert!(matches!(r, Err(AppError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn empty_categories_rejected() {
        let pool = open_in_memory().await;
        let client = OverpassClient::new(vec!["http://127.0.0.1:9999".into()]);
        let r = run(&pool, &client, SearchInput {
            center_lat: 52.0, center_lng: 9.0, radius_km: 10, category_ids: vec![],
        }, |_| {}).await;
        assert!(matches!(r, Err(AppError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn end_to_end_with_mock_overpass() {
        let pool = open_in_memory().await;
        let mut server = mockito::Server::new_async().await;
        let body = r#"{"elements":[
            {"type":"node","id":1,"lat":52.0,"lon":9.0,"tags":{"shop":"supermarket","name":"Test"}}
        ]}"#;
        let _m = server.mock("POST", "/").with_status(200).with_body(body).create_async().await;

        let client = OverpassClient::new(vec![server.url() + "/"]);
        let stats = run(&pool, &client, SearchInput {
            center_lat: 52.0, center_lng: 9.0, radius_km: 5,
            category_ids: vec![6], // Lebensmittel-Einzelhandel matched shop=supermarket
        }, |_| {}).await.unwrap();
        assert_eq!(stats.neu_imported, 1);
    }
}
```

- [ ] **Step 2: mod + Tests**

`overpass/mod.rs`: `pub mod search;`

```bash
cargo test --lib overpass::search
```

Erwartet: 3 passed.

- [ ] **Step 3: Checkpoint**

> **Checkpoint 1.8:** End-to-End Search-Pipeline grün – noch ohne UI.

---

### Task 1.9: Tauri-Command `start_search` mit Progress-Events

**Files:**
- Modify: `src-tauri/src/commands.rs`, `src-tauri/src/main.rs`

- [ ] **Step 1: Command schreiben**

`src-tauri/src/commands.rs` ergänzen (oben Imports anpassen):
```rust
use crate::overpass::{client::OverpassClient, search::{self, SearchInput, ProgressEvent, SearchStats}};
use tauri::{Emitter, AppHandle};

#[derive(serde::Deserialize)]
pub struct StartSearchPayload {
    pub center_lat: f64,
    pub center_lng: f64,
    pub radius_km: u32,
    pub category_ids: Vec<i64>,
}

#[tauri::command]
pub async fn start_search(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    payload: StartSearchPayload,
) -> AppResult<SearchStats> {
    let client = OverpassClient::new(OverpassClient::default_endpoints());
    let app_handle = app.clone();
    let stats = search::run(&state.db, &client, SearchInput {
        center_lat: payload.center_lat,
        center_lng: payload.center_lng,
        radius_km: payload.radius_km,
        category_ids: payload.category_ids,
    }, move |event: ProgressEvent| {
        let _ = app_handle.emit("search-progress", event);
    }).await?;
    let _ = app.emit("search-done", &stats);
    Ok(stats)
}
```

- [ ] **Step 2: Registrieren**

`main.rs`:
```rust
.invoke_handler(tauri::generate_handler![
    commands::list_categories,
    commands::start_search,
])
```

- [ ] **Step 3: Frontend-Wrapper**

`src/lib/tauri.ts` ergänzen:
```ts
import { listen, type UnlistenFn } from "@tauri-apps/api/event"

export type SearchStats = {
  total_found: number; neu_imported: number; duplicates_skipped: number; dauer_ms: number
}
export type ProgressEvent = {
  tile_idx: number; tile_total: number; last_count: number; running_total_inserted: number
}

export const api = {
  // ... bestehend
  startSearch: (payload: { center_lat: number; center_lng: number; radius_km: number; category_ids: number[] }) =>
    invoke<SearchStats>("start_search", { payload }),
  onSearchProgress: (cb: (e: ProgressEvent) => void): Promise<UnlistenFn> =>
    listen<ProgressEvent>("search-progress", (e) => cb(e.payload)),
  onSearchDone: (cb: (s: SearchStats) => void): Promise<UnlistenFn> =>
    listen<SearchStats>("search-done", (e) => cb(e.payload)),
}
```

- [ ] **Step 4: Debug-UI in App.tsx**

```tsx
import { useEffect, useState } from "react"
import { Button } from "@/components/ui/button"
import { api, type CategoryRow, type SearchStats, type ProgressEvent } from "@/lib/tauri"

function App() {
  const [cats, setCats] = useState<CategoryRow[]>([])
  const [progress, setProgress] = useState<ProgressEvent | null>(null)
  const [stats, setStats] = useState<SearchStats | null>(null)
  const [busy, setBusy] = useState(false)
  const [err, setErr] = useState<string | null>(null)

  useEffect(() => {
    api.listCategories().then(setCats)
    const unp = api.onSearchProgress(setProgress)
    const und = api.onSearchDone(setStats)
    return () => { unp.then(f => f()); und.then(f => f()) }
  }, [])

  const runSearch = async () => {
    setBusy(true); setErr(null); setProgress(null); setStats(null)
    try {
      await api.startSearch({
        center_lat: 52.3756, center_lng: 9.7320, radius_km: 5,
        category_ids: cats.filter(c => c.enabled).map(c => c.id),
      })
    } catch (e) { setErr(String(e)) }
    finally { setBusy(false) }
  }

  return (
    <div className="p-8 space-y-4">
      <h1 className="text-3xl font-bold">ProjektAlpha · Debug</h1>
      <Button onClick={runSearch} disabled={busy}>
        {busy ? "Suche läuft..." : "Test-Suche Hannover 5 km"}
      </Button>
      {progress && <div>Tile {progress.tile_idx}/{progress.tile_total} · +{progress.last_count} (gesamt {progress.running_total_inserted})</div>}
      {stats && <pre className="bg-gray-100 p-2 rounded">{JSON.stringify(stats, null, 2)}</pre>}
      {err && <pre className="text-red-600">{err}</pre>}
    </div>
  )
}
export default App
```

- [ ] **Step 5: Live-Smoke-Test**

```bash
pnpm tauri dev
```

Klick auf „Test-Suche Hannover 5 km". Erwartet:
- Innerhalb < 30 s erscheint `stats` mit `neu_imported > 0` (Hannover hat Supermärkte etc.)
- Log-Datei zeigt `search start`, `overpass success`, `search done`
- Erneuter Klick: `neu_imported = 0`, `duplicates_skipped > 0`

Bei Fehler: Log-Datei prüfen → vermutlich Overpass timeout → erneut versuchen oder Kategorien-IDs prüfen.

- [ ] **Step 6: Checkpoint**

> **Checkpoint 1.9 = Phase 1 fertig.** Tauri-Command `start_search` ist End-to-End funktional, importiert echte OSM-Firmen, vermeidet Duplikate, loggt strukturiert. Phase 2 (Liste + Detail) hat alle Daten, mit denen sie arbeiten kann.

---

## Was am Ende dieses Plans funktioniert

- ✅ App startet sauber auf Mac (Windows-Build kommt in eigenem Plan später)
- ✅ Logging schreibt JSON nach App-Data-Logs, Crash-Pfad vorbereitet
- ✅ SQLite-DB migriert idempotent, 11 Branchen geseedet
- ✅ Overpass-Suche mit Tile-Splitting für 1–300 km, Endpoint-Rotation, Retry
- ✅ Duplikat-Strategie korrekt (manuelle Daten unantastbar, OSM-Refresh nur in leere Felder)
- ✅ Test-Coverage hoch in den kritischen Modulen (Tile, Builder, Scoring, Parser, Client, Search)
- ✅ Debug-UI zeigt Branchen + triggert Suche → Phase 2 baut darauf das echte Lead-UI

## Was bewusst NICHT in diesem Plan ist

- Karten-Ansicht (Phase 3)
- Liste/Filter/Detail-Sheet/Status-Updates → **Plan 2**
- Such-Profile, Settings-UI, Branchen-Editor → **Plan 4**
- Backup/Restore-UI (Logik kann hier schon angefangen werden, ist aber bewusst Phase 4)
- Auto-Updater, GitHub-Actions-Cross-Build (Phase 6)
- **Frontend-Log-Bridge** (Spec §10a.2): React-Errors via Tauri-Event ans Backend → kommt mit Plan 2, sobald die echte UI Error-Boundaries braucht
- **Crash-File-Handler** (Spec §10a.4): `last_crash.txt` schreiben + beim Start anzeigen → kommt mit Plan 2 (Error-Boundary-Setup) bzw. Plan 6 (Polish)
- **Nominatim-Geocoding** (Spec §6.3, §8.2): erst nötig, wenn die UI in Plan 4 die Adress-Suche bekommt

---

## Nächster Plan

Nach Abschluss dieser Phase: **Plan 2 — Phase 2 (Liste + Detail + Status + Activity-Log)**, der die Debug-UI durch echtes Lead-Management ersetzt und ab dann produktiv nutzbar macht.
