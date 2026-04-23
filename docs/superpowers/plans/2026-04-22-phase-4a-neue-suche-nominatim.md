# Phase 4a: Neue-Suche-Dialog + Nominatim-Geocoding – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Der „Neue Suche"-Flow wird produktionsreif. Der User kann eine Adresse eintippen (Vorschläge per Nominatim, mit 30-Tage-DB-Cache), ODER den Mittelpunkt per Karten-Klick setzen, den Radius per Slider (1–300 km) wählen, und Branchen gezielt an-/abwählen. Derselbe Adress-Input wird ins Manual-Add-Dialog eingebaut, damit der Vater bei manueller Erfassung die Koordinaten nicht mehr aus Google Maps kopieren muss.

**Architecture:** Backend bekommt ein neues `nominatim/`-Modul (strukturell parallel zu `overpass/`): `client.rs` für HTTP mit User-Agent + 1-req/s-Rate-Limit, `mod.rs` für die Query-Funktion mit vorgelagertem DB-Cache. Die Tabelle `geocode_cache` existiert bereits seit Plan 1 — keine neue Migration. Frontend bekommt drei neue Such-Komponenten (`AddressSearchInput`, `RadiusSlider`, `CategoryPicker`) und ersetzt die provisorische `DebugSearchPage` durch eine echte `NewSearchPage`. „Als Profil speichern" ist bewusst nicht in diesem Plan — das gehört zu Plan 4b (Settings-UI) zusammen mit der Profile-Verwaltung.

**Tech Stack:** Rust (reqwest, sqlx, tokio::sync::Mutex für Rate-Limit, mockito für HTTP-Tests), React (shadcn Slider + Checkbox neu zu installieren), Debounce 500 ms im Frontend + 1 s Backend-Gate, Nominatim Endpoint `https://nominatim.openstreetmap.org/search` mit User-Agent `ProjektAlpha/0.1 (kontakt: jan-mack@web.de)`.

**Spec-Referenz:** `docs/superpowers/specs/2026-04-21-firmensuche-design.md`
- §6.3 (Adress-Suche Mittelpunkt) · §7.2 Screen 5 (Neue Suche) · §8.2 (Nominatim) · §10a (Logging)

**CLAUDE.md-Prinzipien:**
- **UX zuerst:** Typeahead zeigt Vorschläge nach 500 ms, Klick setzt Mittelpunkt + zoomt Karte hin (zoom 12). Radius-Slider aktualisiert den Karten-Kreis live. Such-Button disabled, bis Mittelpunkt UND ≥1 Branche gesetzt sind.
- **Nicht kompliziert:** Kein Autocomplete-Library, eigener State + Debounce in ~60 LOC. Rate-Limit auf Backend-Seite mit simpler `Mutex<Instant>` (kein Token-Bucket).
- **Tests + Logs:** Pure Funktionen (Cache-TTL-Check, URL-Building) sind TDD. HTTP-Client wird mit `mockito` getestet. Logs enthalten Cache-Hit/Miss, Query-Länge, Result-Count — **nicht** den Query-String selbst (Adressen sind zwar öffentlich, aber die Intention des Users ist tabu).

**Kein Git** – Checkpoints statt Commits.

---

## Datei-Struktur (Ziel nach diesem Plan)

```
src-tauri/src/
├─ nominatim/
│  ├─ mod.rs                            # NEU: query() — cache-first
│  └─ client.rs                         # NEU: HTTP mit UA, Rate-Limit, parse
├─ db/
│  └─ geocode_cache.rs                  # NEU: CRUD auf geocode_cache (TTL 30d)
├─ commands.rs                          # MODIFY: +geocode command
└─ lib.rs                               # MODIFY: register command, mount nominatim module

src/
├─ lib/
│  └─ tauri.ts                          # MODIFY: +geocode + GeocodeSuggestion type
├─ components/
│  ├─ search/                           # NEU: Ordner
│  │  ├─ AddressSearchInput.tsx         # NEU: debounced typeahead
│  │  ├─ RadiusSlider.tsx               # NEU: 1–300 km
│  │  └─ CategoryPicker.tsx             # NEU: Checkboxes mit Farbe + Score
│  ├─ manual/
│  │  └─ ManualAddDialog.tsx            # MODIFY: AddressSearchInput eingehängt
│  └─ ui/
│     ├─ slider.tsx                     # shadcn (auto-gen in Task 4a.6)
│     └─ checkbox.tsx                   # shadcn (auto-gen in Task 4a.6)
└─ pages/
   └─ NewSearchPage.tsx                 # NEU: ersetzt DebugSearchPage-Registrierung
```

Die alte `src/pages/DebugSearchPage.tsx` wird in Task 4a.11 gelöscht. Die Datenbank-Tabellen `geocode_cache` und `search_profiles` existieren seit Plan 1 — keine neue Migration nötig.

---

# PHASE 4a — Neue-Suche-Dialog + Nominatim-Geocoding

---

### Task 4a.1: `db/geocode_cache.rs` — CRUD + TTL (TDD)

**Files:**
- Create: `src-tauri/src/db/geocode_cache.rs`
- Modify: `src-tauri/src/db/mod.rs` (+ `pub mod geocode_cache;`)

Zweck: Lesen/Schreiben in die bestehende `geocode_cache`-Tabelle. Einträge älter als 30 Tage werden als „Miss" behandelt.

**Tabellen-Schema** (zur Referenz, schon in Migration 0001_initial):
```sql
CREATE TABLE geocode_cache (
    query TEXT PRIMARY KEY,
    lat REAL NOT NULL,
    lng REAL NOT NULL,
    display_name TEXT NOT NULL,
    cached_at TEXT NOT NULL
);
```

- [ ] **Step 1: Tests schreiben (am Ende des neuen Moduls)**

```rust
// src-tauri/src/db/geocode_cache.rs
use crate::error::AppResult;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CachedGeocode {
    pub lat: f64,
    pub lng: f64,
    pub display_name: String,
    pub cached_at: String,
}

const TTL_DAYS: i64 = 30;

/// Returns the cached entry if present AND younger than 30 days.
pub async fn get_fresh(pool: &SqlitePool, query: &str) -> AppResult<Option<CachedGeocode>> {
    let cutoff = (Utc::now() - Duration::days(TTL_DAYS)).to_rfc3339();
    let row: Option<(f64, f64, String, String)> = sqlx::query_as(
        "SELECT lat, lng, display_name, cached_at FROM geocode_cache
         WHERE query = ? AND cached_at > ?",
    )
    .bind(query)
    .bind(&cutoff)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(lat, lng, display_name, cached_at)| CachedGeocode {
        lat,
        lng,
        display_name,
        cached_at,
    }))
}

pub async fn upsert(
    pool: &SqlitePool,
    query: &str,
    lat: f64,
    lng: f64,
    display_name: &str,
) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO geocode_cache (query, lat, lng, display_name, cached_at)
         VALUES (?, ?, ?, ?, ?)
         ON CONFLICT(query) DO UPDATE SET
            lat = excluded.lat,
            lng = excluded.lng,
            display_name = excluded.display_name,
            cached_at = excluded.cached_at",
    )
    .bind(query)
    .bind(lat)
    .bind(lng)
    .bind(display_name)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[tokio::test]
    async fn get_returns_none_for_missing_query() {
        let pool = open_in_memory().await;
        assert!(get_fresh(&pool, "unknown").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn upsert_then_get_returns_entry() {
        let pool = open_in_memory().await;
        upsert(&pool, "Hannover", 52.37, 9.73, "Hannover, Deutschland")
            .await
            .unwrap();
        let got = get_fresh(&pool, "Hannover").await.unwrap().unwrap();
        assert_eq!(got.lat, 52.37);
        assert_eq!(got.lng, 9.73);
        assert_eq!(got.display_name, "Hannover, Deutschland");
    }

    #[tokio::test]
    async fn upsert_twice_updates_existing_row() {
        let pool = open_in_memory().await;
        upsert(&pool, "Hannover", 52.0, 9.0, "alt").await.unwrap();
        upsert(&pool, "Hannover", 52.37, 9.73, "neu").await.unwrap();
        let got = get_fresh(&pool, "Hannover").await.unwrap().unwrap();
        assert_eq!(got.display_name, "neu");
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM geocode_cache")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, 1);
    }

    #[tokio::test]
    async fn get_ignores_entries_older_than_30_days() {
        let pool = open_in_memory().await;
        // Direkt eine uralte Zeile einfügen
        let old = (Utc::now() - Duration::days(40)).to_rfc3339();
        sqlx::query(
            "INSERT INTO geocode_cache (query, lat, lng, display_name, cached_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind("Hannover")
        .bind(52.0)
        .bind(9.0)
        .bind("alt")
        .bind(&old)
        .execute(&pool)
        .await
        .unwrap();
        assert!(get_fresh(&pool, "Hannover").await.unwrap().is_none());
    }
}
```

- [ ] **Step 2: Modul registrieren**

In `src-tauri/src/db/mod.rs` neben den bestehenden Einträgen (`pub mod activity; pub mod categories; pub mod companies; pub mod migrations;`) hinzufügen:

```rust
pub mod geocode_cache;
```

- [ ] **Step 3: Tests ausführen**

```bash
cd src-tauri && cargo test --lib db::geocode_cache 2>&1 | tail -10
```

Erwartet: 4 Tests bestanden (`get_returns_none_for_missing_query`, `upsert_then_get_returns_entry`, `upsert_twice_updates_existing_row`, `get_ignores_entries_older_than_30_days`).

- [ ] **Step 4: Ganze Test-Suite grün halten**

```bash
cd src-tauri && cargo test --lib 2>&1 | tail -5
```

Erwartet: 44 + 4 = 48 Tests grün.

- [ ] **Step 5: Checkpoint**

> **Checkpoint 4a.1:** Geocode-Cache-Modul mit TTL fertig. DB-Schicht steht.

---

### Task 4a.2: `nominatim/`-Modul-Gerüst

**Files:**
- Create: `src-tauri/src/nominatim/mod.rs`
- Create: `src-tauri/src/nominatim/client.rs` (nur Stub; echtes HTTP in Task 4a.3)
- Modify: `src-tauri/src/lib.rs` (+ `pub mod nominatim;`)

Zweck: Scaffolding — leere Module plus `lib.rs`-Registrierung, damit die nachfolgenden Tasks darauf aufbauen können.

- [ ] **Step 1: `src-tauri/src/nominatim/mod.rs` anlegen**

```rust
pub mod client;
```

- [ ] **Step 2: `src-tauri/src/nominatim/client.rs` anlegen (Stub)**

```rust
use crate::error::AppResult;

/// Eine Vorschlag-Zeile aus der Nominatim-API.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Suggestion {
    pub lat: f64,
    pub lng: f64,
    pub display_name: String,
}

/// Parst eine Nominatim-JSON-Antwort (Array von Search-Objekten).
/// Reine Funktion — wird in Task 4a.3 per TDD befüllt.
pub fn parse_response(_json: &str) -> AppResult<Vec<Suggestion>> {
    unimplemented!("Task 4a.3")
}
```

- [ ] **Step 3: In `src-tauri/src/lib.rs` Modul registrieren**

Über den bestehenden `pub mod overpass;` hinzufügen:

```rust
pub mod nominatim;
```

- [ ] **Step 4: Build verifizieren**

```bash
cd src-tauri && cargo build --lib 2>&1 | tail -5
```

Erwartet: build ok (Warning wegen `unimplemented!`, aber kein Fehler).

- [ ] **Step 5: Checkpoint**

> **Checkpoint 4a.2:** Modul-Gerüst steht. Die nächsten Tasks befüllen es.

---

### Task 4a.3: `nominatim::client::parse_response` + HTTP-Query (TDD mit mockito)

**Files:**
- Modify: `src-tauri/src/nominatim/client.rs`

Zweck: (1) Parser für Nominatim-JSON und (2) HTTP-Client mit User-Agent und 1-req/s-Rate-Limit.

- [ ] **Step 1: Parser-Tests schreiben (TDD)**

Am Ende von `src-tauri/src/nominatim/client.rs` einen `#[cfg(test)]`-Block hinzufügen:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RESPONSE: &str = r#"[
        {"place_id":1,"lat":"52.3756","lon":"9.7320","display_name":"Hannover, Deutschland","type":"city"},
        {"place_id":2,"lat":"48.1351","lon":"11.5820","display_name":"München, Deutschland","type":"city"}
    ]"#;

    #[test]
    fn parse_response_returns_two_suggestions() {
        let got = parse_response(SAMPLE_RESPONSE).unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].display_name, "Hannover, Deutschland");
        assert!((got[0].lat - 52.3756).abs() < 1e-6);
        assert!((got[0].lng - 9.7320).abs() < 1e-6);
    }

    #[test]
    fn parse_response_empty_array_returns_empty_vec() {
        let got = parse_response("[]").unwrap();
        assert!(got.is_empty());
    }

    #[test]
    fn parse_response_rejects_malformed_json() {
        assert!(parse_response("not json").is_err());
    }
}
```

- [ ] **Step 2: Parser implementieren (Tests GRÜN)**

Ersetze `pub fn parse_response(_json: &str) -> AppResult<Vec<Suggestion>>` mit:

```rust
pub fn parse_response(json: &str) -> AppResult<Vec<Suggestion>> {
    #[derive(serde::Deserialize)]
    struct Raw {
        lat: String,
        lon: String,
        display_name: String,
    }
    let raws: Vec<Raw> = serde_json::from_str(json)
        .map_err(|e| crate::error::AppError::InvalidInput(format!("nominatim parse: {e}")))?;
    let mut out = Vec::with_capacity(raws.len());
    for r in raws {
        let lat: f64 = r.lat.parse()
            .map_err(|e| crate::error::AppError::InvalidInput(format!("lat parse: {e}")))?;
        let lng: f64 = r.lon.parse()
            .map_err(|e| crate::error::AppError::InvalidInput(format!("lng parse: {e}")))?;
        out.push(Suggestion { lat, lng, display_name: r.display_name });
    }
    Ok(out)
}
```

- [ ] **Step 3: Parser-Tests ausführen**

```bash
cd src-tauri && cargo test --lib nominatim::client::tests 2>&1 | tail -10
```

Erwartet: 3 bestanden.

- [ ] **Step 4: HTTP-Client-Struct + Rate-Limit hinzufügen**

Am Anfang von `src-tauri/src/nominatim/client.rs`, über dem `Suggestion`-Struct. Wichtig: Wir brauchen **keine neue** `AppError`-Variante — `AppError::Network(#[from] reqwest::Error)` existiert bereits (für `?` auf reqwest-Calls), und Non-2xx-Status melden wir als `AppError::Internal(...)`. `error.rs` bleibt unverändert.

```rust
use crate::error::{AppError, AppResult};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const USER_AGENT: &str = "ProjektAlpha/0.1 (kontakt: jan-mack@web.de)";
const DEFAULT_ENDPOINT: &str = "https://nominatim.openstreetmap.org/search";
const MIN_INTERVAL: Duration = Duration::from_millis(1100); // 1 req/s mit Puffer

pub struct NominatimClient {
    http: reqwest::Client,
    endpoint: String,
    last_call: Arc<Mutex<Option<Instant>>>,
    max_results: u32,
}

impl NominatimClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent(USER_AGENT)
            .build()
            .expect("reqwest client");
        Self {
            http,
            endpoint: DEFAULT_ENDPOINT.to_string(),
            last_call: Arc::new(Mutex::new(None)),
            max_results: 5,
        }
    }

    #[cfg(test)]
    pub fn with_endpoint(endpoint: impl Into<String>) -> Self {
        let mut c = Self::new();
        c.endpoint = endpoint.into();
        c
    }

    async fn wait_rate_limit(&self) {
        let mut last = self.last_call.lock().await;
        if let Some(t) = *last {
            let elapsed = t.elapsed();
            if elapsed < MIN_INTERVAL {
                tokio::time::sleep(MIN_INTERVAL - elapsed).await;
            }
        }
        *last = Some(Instant::now());
    }

    pub async fn query(&self, q: &str) -> AppResult<Vec<Suggestion>> {
        self.wait_rate_limit().await;
        let url = format!(
            "{}?q={}&format=json&addressdetails=0&limit={}&countrycodes=de",
            self.endpoint,
            urlencoding::encode(q),
            self.max_results
        );
        let started = Instant::now();
        let response = self.http.get(&url).send().await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            tracing::warn!(status = %status, dauer_ms = started.elapsed().as_millis() as u64, "nominatim non-2xx");
            return Err(AppError::Internal(format!("nominatim {status}")));
        }
        let result = parse_response(&text);
        tracing::debug!(
            q_len = q.len(),
            dauer_ms = started.elapsed().as_millis() as u64,
            count = result.as_ref().map(|v| v.len()).unwrap_or(0),
            "nominatim query"
        );
        result
    }
}

impl Default for NominatimClient {
    fn default() -> Self {
        Self::new()
    }
}
```

- [ ] **Step 5: `urlencoding`-Crate installieren**

`urlencoding` ist aktuell keine Dep — muss hinzugefügt werden:

```bash
cd src-tauri && cargo add urlencoding
```

Erwartet: Cargo.toml bekommt `urlencoding = "2"` (oder neuere Major). Ein kurzer `cargo build --lib 2>&1 | tail -3` bestätigt, dass die Crate auflösbar ist.

**Note zu `AppError`:** Wir fügen keine neue Variante hinzu. Der `?`-Operator auf reqwest-Calls funktioniert über die bereits existierende `AppError::Network(#[from] reqwest::Error)`-Variante. Non-2xx-Status werden oben als `AppError::Internal(format!("nominatim {status}"))` zurückgegeben — das ist semantisch korrekt und hält `error.rs`-Änderungen unnötig aus Plan 4a heraus.

- [ ] **Step 6: HTTP-Integration-Test mit mockito**

Erweitere den `#[cfg(test)] mod tests`-Block:

```rust
    #[tokio::test]
    async fn client_queries_endpoint_and_parses() {
        let mut server = mockito::Server::new_async().await;
        let _m = server.mock("GET", mockito::Matcher::Regex(r"^/.*q=Hannover.*".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(SAMPLE_RESPONSE)
            .create_async().await;

        let client = NominatimClient::with_endpoint(server.url());
        let results = client.query("Hannover").await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].display_name, "Hannover, Deutschland");
    }

    #[tokio::test]
    async fn client_rate_limits_second_call() {
        let mut server = mockito::Server::new_async().await;
        let _m = server.mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_body("[]")
            .expect_at_least(2)
            .create_async().await;

        let client = NominatimClient::with_endpoint(server.url());
        let started = std::time::Instant::now();
        client.query("A").await.unwrap();
        client.query("B").await.unwrap();
        let elapsed = started.elapsed();
        assert!(elapsed >= std::time::Duration::from_millis(1000),
            "second call should be rate-limited, elapsed={:?}", elapsed);
    }

    #[tokio::test]
    async fn client_surfaces_5xx_as_error() {
        let mut server = mockito::Server::new_async().await;
        let _m = server.mock("GET", mockito::Matcher::Any)
            .with_status(502)
            .create_async().await;

        let client = NominatimClient::with_endpoint(server.url());
        assert!(client.query("X").await.is_err());
    }
```

- [ ] **Step 7: Alle Tests grün**

```bash
cd src-tauri && cargo test --lib nominatim 2>&1 | tail -15
```

Erwartet: 6 bestanden (3 parse + 3 HTTP). Der Rate-Limit-Test dauert ~1,1 s.

- [ ] **Step 8: Ganze Suite**

```bash
cd src-tauri && cargo test --lib 2>&1 | tail -5
```

Erwartet: 44 + 4 + 6 = 54 Tests grün.

- [ ] **Step 9: Checkpoint**

> **Checkpoint 4a.3:** Nominatim-HTTP-Client mit User-Agent, 1-req/s-Gate, Parser und mockito-basierten Tests steht.

---

### Task 4a.4: `nominatim::query` — Cache-First-Facade (TDD)

**Files:**
- Modify: `src-tauri/src/nominatim/mod.rs`

Zweck: Eine öffentliche `query(pool, client, q)`-Funktion, die erst den DB-Cache prüft und nur bei Miss den Client aufruft + Ergebnis cached.

- [ ] **Step 1: Code schreiben**

Inhalt von `src-tauri/src/nominatim/mod.rs` komplett ersetzen durch:

```rust
pub mod client;

use crate::db::geocode_cache;
use crate::error::AppResult;
use client::{NominatimClient, Suggestion};
use sqlx::SqlitePool;

/// Cache-first: prüft `geocode_cache` (30-Tage-TTL), bei Miss → HTTP-Call + Upsert.
/// Bei Cache-Hit wird nur der erste (beste) Vorschlag zurückgegeben. Bei Miss werden
/// bis zu 5 Vorschläge geliefert UND der beste wird gecached — das erlaubt dem Frontend
/// eine Auswahl und hält den Cache kompakt.
pub async fn query(
    pool: &SqlitePool,
    client: &NominatimClient,
    q: &str,
) -> AppResult<Vec<Suggestion>> {
    let trimmed = q.trim();
    if trimmed.is_empty() {
        return Ok(vec![]);
    }

    if let Some(cached) = geocode_cache::get_fresh(pool, trimmed).await? {
        tracing::debug!(q_len = trimmed.len(), "geocode cache hit");
        return Ok(vec![Suggestion {
            lat: cached.lat,
            lng: cached.lng,
            display_name: cached.display_name,
        }]);
    }

    tracing::debug!(q_len = trimmed.len(), "geocode cache miss");
    let suggestions = client.query(trimmed).await?;
    if let Some(best) = suggestions.first() {
        geocode_cache::upsert(pool, trimmed, best.lat, best.lng, &best.display_name).await?;
    }
    Ok(suggestions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[tokio::test]
    async fn empty_query_returns_empty_vec_without_http() {
        let pool = open_in_memory().await;
        // Client mit offensichtlich ungültigem Endpoint — sollte nie aufgerufen werden
        let client = NominatimClient::with_endpoint("http://127.0.0.1:1");
        let got = query(&pool, &client, "   ").await.unwrap();
        assert!(got.is_empty());
    }

    #[tokio::test]
    async fn cache_hit_skips_http_call() {
        let pool = open_in_memory().await;
        // Cache vorab füllen
        geocode_cache::upsert(&pool, "Hannover", 52.37, 9.73, "Hannover, Deutschland")
            .await
            .unwrap();
        // Client mit kaputtem Endpoint — wenn der aufgerufen würde, käme ein Fehler
        let client = NominatimClient::with_endpoint("http://127.0.0.1:1");
        let got = query(&pool, &client, "Hannover").await.unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].display_name, "Hannover, Deutschland");
    }

    #[tokio::test]
    async fn cache_miss_calls_http_and_stores_result() {
        use client::Suggestion;
        let pool = open_in_memory().await;
        let mut server = mockito::Server::new_async().await;
        // .expect(1) ist der echte Cache-Assert — wenn der 2. Call den Server
        // nochmal anfragt, failt der Test beim Drop von `_m`.
        let _m = server.mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_body(r#"[{"lat":"52.5","lon":"13.4","display_name":"Berlin, Deutschland"}]"#)
            .expect(1)
            .create_async().await;

        let client = NominatimClient::with_endpoint(server.url());
        let got = query(&pool, &client, "Berlin").await.unwrap();
        assert_eq!(got, vec![Suggestion { lat: 52.5, lng: 13.4, display_name: "Berlin, Deutschland".into() }]);

        // Zweiter Call → Cache-Hit (server.mock darf KEIN zweites Mal aufgerufen werden)
        let again = query(&pool, &client, "Berlin").await.unwrap();
        assert_eq!(again.len(), 1);
    }
}
```

- [ ] **Step 2: Tests ausführen**

```bash
cd src-tauri && cargo test --lib nominatim 2>&1 | tail -15
```

Erwartet: 6 (aus 4a.3) + 3 (neu) = 9 bestanden.

- [ ] **Step 3: Ganze Suite**

```bash
cd src-tauri && cargo test --lib 2>&1 | tail -5
```

Erwartet: 48 + 9 = 57 Tests grün.

- [ ] **Step 4: Checkpoint**

> **Checkpoint 4a.4:** Cache-First-Facade fertig. Das Backend-API ist bereit für den Tauri-Command.

---

### Task 4a.5: Tauri-Command `geocode`

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs` (+ handler registrieren)

Zweck: Frontend-zugänglicher Aufruf. Der Command erzeugt einen frischen `NominatimClient` pro Request — das ist ok weil reqwest intern Connection-Pooling macht und der Rate-Limiter sowieso in-memory nur für die Prozess-Lifetime lebt. (Eine Optimierung mit geteiltem Client via AppState ist YAGNI für diesen einen Use-Case.)

⚠️ **Aber:** Wenn der Rate-Limit wirklich wirken soll muss der Client persistent im `AppState` liegen (mehrere Commands in 1 s würden sonst alle gleichzeitig raus). Einfach zu lösen: Wir legen **einen** `NominatimClient` als Feld in den `AppState`. Da `AppState` bereits in einem äußeren `Arc` gewrapt ist (beim `.manage(Arc::new(AppState { ... }))`-Aufruf in `lib.rs`), teilen sich alle Commands automatisch dieselbe Client-Instanz — damit ist der interne Rate-Limit-Mutex wirksam.

- [ ] **Step 1: `AppState` erweitern (`src-tauri/src/lib.rs`)**

Aktueller Zustand:

```rust
pub struct AppState {
    pub db: SqlitePool,
}
```

Neu:

```rust
use crate::nominatim::client::NominatimClient;

pub struct AppState {
    pub db: SqlitePool,
    pub nominatim: NominatimClient,
}
```

An der Init-Stelle (wo `Arc::new(AppState { db: pool })` konstruiert wird):

```rust
.manage(Arc::new(AppState {
    db: pool,
    nominatim: NominatimClient::new(),
}))
```

- [ ] **Step 2: Command in `commands.rs` hinzufügen**

Am Ende der Datei, neben den anderen Commands:

```rust
use crate::nominatim::{self, client::Suggestion};

#[derive(Deserialize)]
pub struct GeocodePayload {
    pub query: String,
}

#[tauri::command]
pub async fn geocode(
    state: State<'_, Arc<AppState>>,
    payload: GeocodePayload,
) -> AppResult<Vec<Suggestion>> {
    let trimmed = payload.query.trim();
    if trimmed.len() < 3 {
        // Zu kurze Queries gar nicht an Nominatim schicken
        return Ok(vec![]);
    }
    nominatim::query(&state.db, &state.nominatim, trimmed).await
}
```

- [ ] **Step 3: Command im `invoke_handler` registrieren (`lib.rs`)**

Ergänze die `tauri::generate_handler![...]` Liste:

```rust
commands::geocode,
```

(Platzierung: vor `commands::frontend_log` oder sonst eine sinnvolle Stelle.)

- [ ] **Step 4: Build**

```bash
cd src-tauri && cargo build --lib 2>&1 | tail -5
```

Erwartet: ok, keine Warnings.

- [ ] **Step 5: Bestehende Tests**

```bash
cd src-tauri && cargo test --lib 2>&1 | tail -5
```

Erwartet: 57 Tests grün.

- [ ] **Step 6: Checkpoint**

> **Checkpoint 4a.5:** Backend fertig. Das Frontend kann jetzt `invoke("geocode", { payload: { query: "..." } })` aufrufen.

---

### Task 4a.6: shadcn Slider + Checkbox installieren

**Files:**
- Auto-generated: `src/components/ui/slider.tsx`
- Auto-generated: `src/components/ui/checkbox.tsx`

- [ ] **Step 1: shadcn-CLI (non-interactive)**

```bash
pnpm dlx shadcn@latest add slider checkbox --yes --overwrite 2>&1 | tail -10
```

`--yes` überspringt Prompts, `--overwrite` (harmlos falls Dateien noch nicht existieren) verhindert, dass der CLI im Tool-Umfeld hängt. Erwartet: zwei neue Dateien unter `src/components/ui/`. Falls der Befehl trotzdem hängt (shadcn-CLI-Version-Abhängigkeit), vom User in einem echten Terminal ausführen lassen.

- [ ] **Step 2: Build verifizieren**

```bash
pnpm vite build 2>&1 | tail -5
```

Erwartet: grün.

- [ ] **Step 3: Checkpoint**

> **Checkpoint 4a.6:** shadcn Slider + Checkbox installiert.

---

### Task 4a.7: Frontend-API + Typen

**Files:**
- Modify: `src/lib/tauri.ts`

Zweck: `api.geocode()` plus `GeocodeSuggestion`-Typ exportieren.

- [ ] **Step 1: Typ + API-Methode**

Am Ende des `api`-Objekts in `src/lib/tauri.ts` (vor dem schließenden `}`) hinzufügen:

```ts
export type GeocodeSuggestion = {
  lat: number
  lng: number
  display_name: string
}
```

(Den `export type`-Block oben einfügen, neben den anderen Typen.)

Und im `api`-Objekt:

```ts
  geocode: (query: string) => invoke<GeocodeSuggestion[]>("geocode", { payload: { query } }),
```

- [ ] **Step 2: Build**

```bash
pnpm vite build 2>&1 | tail -5
pnpm test 2>&1 | tail -5
```

Erwartet: Build + 20 Tests grün.

- [ ] **Step 3: Checkpoint**

> **Checkpoint 4a.7:** Frontend-API ist bereit.

---

### Task 4a.8: `AddressSearchInput`-Komponente

**Files:**
- Create: `src/components/search/AddressSearchInput.tsx`
- Create: `src/components/search/AddressSearchInput.test.tsx`

Zweck: Eingabefeld mit 500 ms Debounce, zeigt bei ≥ 3 Zeichen die Nominatim-Vorschläge als Dropdown; Klick setzt den Wert in der Parent-State.

**Wichtig:** Tests werden für die **debounce-Logik** geschrieben (pure + mit Vitest Fake Timers). Die volle Render-Logik ist im Smoke-Test abgedeckt.

- [ ] **Step 1: Komponente schreiben**

```tsx
// src/components/search/AddressSearchInput.tsx
import { useEffect, useRef, useState } from "react"
import { Input } from "@/components/ui/input"
import { api, type GeocodeSuggestion } from "@/lib/tauri"
import { logger } from "@/lib/logger"

interface AddressSearchInputProps {
  onPick: (s: GeocodeSuggestion) => void
  placeholder?: string
  /** In ms. Default 500. Für Tests über-ridable. */
  debounceMs?: number
}

export function AddressSearchInput({ onPick, placeholder, debounceMs = 500 }: AddressSearchInputProps) {
  const [value, setValue] = useState("")
  const [suggestions, setSuggestions] = useState<GeocodeSuggestion[]>([])
  const [loading, setLoading] = useState(false)
  const [open, setOpen] = useState(false)
  const timerRef = useRef<number | null>(null)

  useEffect(() => {
    if (timerRef.current) window.clearTimeout(timerRef.current)
    if (value.trim().length < 3) {
      setSuggestions([])
      setOpen(false)
      return
    }
    timerRef.current = window.setTimeout(() => {
      setLoading(true)
      api.geocode(value)
        .then(results => {
          setSuggestions(results)
          setOpen(results.length > 0)
          logger.info("geocode done", { q_len: value.length, count: results.length })
        })
        .catch(e => logger.error("geocode failed", { e: String(e) }))
        .finally(() => setLoading(false))
    }, debounceMs)
    return () => {
      if (timerRef.current) window.clearTimeout(timerRef.current)
    }
  }, [value, debounceMs])

  const pick = (s: GeocodeSuggestion) => {
    onPick(s)
    setValue(s.display_name)
    setOpen(false)
  }

  return (
    <div className="relative">
      <Input
        value={value}
        onChange={(e) => setValue(e.target.value)}
        placeholder={placeholder ?? "Adresse, Stadt oder PLZ…"}
        onFocus={() => { if (suggestions.length > 0) setOpen(true) }}
        onBlur={() => setTimeout(() => setOpen(false), 200)}
      />
      {loading && (
        <div className="absolute right-3 top-2.5 text-xs text-muted-foreground">…</div>
      )}
      {open && suggestions.length > 0 && (
        <ul className="absolute left-0 right-0 top-full mt-1 z-50 bg-popover text-popover-foreground border rounded-md shadow max-h-60 overflow-y-auto">
          {suggestions.map((s, i) => (
            <li key={i}>
              <button
                type="button"
                className="w-full text-left px-3 py-2 text-sm hover:bg-accent truncate"
                onMouseDown={(e) => e.preventDefault()} // verhindert Blur-Race
                onClick={() => pick(s)}
              >
                {s.display_name}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
```

- [ ] **Step 2: Test-Datei**

```tsx
// src/components/search/AddressSearchInput.test.tsx
import { describe, it, expect, vi, beforeEach } from "vitest"
import { render, screen, fireEvent } from "@testing-library/react"
import { AddressSearchInput } from "./AddressSearchInput"

// invoke-Mock ist bereits in src/tests/setup.ts aktiv — wir überschreiben das return per-Test
import { invoke } from "@tauri-apps/api/core"

// Kleine Helper: echte Zeit warten, statt Fake-Timers (die mit findByText/waitFor
// und der async invoke-Promise-Chain in der Komponente kollidieren).
// 50 ms Puffer reicht, da debounceMs als Prop kontrolliert wird.
const waitRealMs = (ms: number) => new Promise(r => setTimeout(r, ms))

describe("AddressSearchInput", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset()
  })

  it("renders the input with placeholder", () => {
    render(<AddressSearchInput onPick={vi.fn()} placeholder="Test" />)
    expect(screen.getByPlaceholderText("Test")).toBeInTheDocument()
  })

  it("does NOT call geocode when query is shorter than 3 chars", async () => {
    render(<AddressSearchInput onPick={vi.fn()} debounceMs={30} />)
    fireEvent.change(screen.getByRole("textbox"), { target: { value: "Ha" } })
    await waitRealMs(80)
    expect(invoke).not.toHaveBeenCalled()
  })

  it("debounces the geocode call until after the debounce window", async () => {
    vi.mocked(invoke).mockResolvedValue([])
    render(<AddressSearchInput onPick={vi.fn()} debounceMs={30} />)
    const input = screen.getByRole("textbox")
    // Erste Eingabe
    fireEvent.change(input, { target: { value: "Han" } })
    await waitRealMs(10)
    expect(invoke).not.toHaveBeenCalled()
    // Zweite Eingabe kurz danach resettet den Timer
    fireEvent.change(input, { target: { value: "Hann" } })
    await waitRealMs(10)
    expect(invoke).not.toHaveBeenCalled()
    // Jetzt abwarten, bis die Debounce abgelaufen ist
    await waitRealMs(60)
    expect(invoke).toHaveBeenCalledTimes(1)
  })

  it("calls onPick with the chosen suggestion", async () => {
    vi.mocked(invoke).mockResolvedValue([
      { lat: 52.37, lng: 9.73, display_name: "Hannover, Deutschland" },
    ])
    const onPick = vi.fn()
    render(<AddressSearchInput onPick={onPick} debounceMs={30} />)
    fireEvent.change(screen.getByRole("textbox"), { target: { value: "Hannover" } })
    const suggestion = await screen.findByText("Hannover, Deutschland")
    fireEvent.click(suggestion)
    expect(onPick).toHaveBeenCalledWith({ lat: 52.37, lng: 9.73, display_name: "Hannover, Deutschland" })
  })
})
```

> **Warum keine Vitest Fake-Timers:** `vi.useFakeTimers()` ersetzt `setTimeout`, aber `findByText`/`waitFor` aus React Testing Library pollen auf echten Timern — das führt zum Deadlock. Kompromiss: Wir reduzieren die Debounce per `debounceMs={30}`-Prop und warten 60 ms mit echten Timern. Jeder Test dauert < 200 ms — keine messbare CI-Kosten.

- [ ] **Step 3: Tests ausführen**

```bash
pnpm test src/components/search/AddressSearchInput.test.tsx 2>&1 | tail -15
```

Erwartet: 4 bestanden.

- [ ] **Step 4: Ganze Frontend-Suite**

```bash
pnpm test 2>&1 | tail -8
```

Erwartet: 20 + 4 = 24 Tests grün.

- [ ] **Step 5: Checkpoint**

> **Checkpoint 4a.8:** AddressSearchInput fertig und getestet.

---

### Task 4a.9: `RadiusSlider`-Komponente

**Files:**
- Create: `src/components/search/RadiusSlider.tsx`

Zweck: Kleiner Wrapper um shadcn Slider mit Label + Anzeige des aktuellen Werts. 1–300 km, Step 1.

- [ ] **Step 1: Komponente schreiben**

```tsx
// src/components/search/RadiusSlider.tsx
import { Slider } from "@/components/ui/slider"
import { Label } from "@/components/ui/label"

interface RadiusSliderProps {
  value: number
  onChange: (km: number) => void
  min?: number
  max?: number
}

export function RadiusSlider({ value, onChange, min = 1, max = 300 }: RadiusSliderProps) {
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <Label>Radius</Label>
        <span className="text-sm font-medium tabular-nums">{value} km</span>
      </div>
      <Slider
        value={[value]}
        min={min}
        max={max}
        step={1}
        onValueChange={(v) => onChange(v[0])}
      />
      <div className="flex justify-between text-xs text-muted-foreground">
        <span>{min} km</span>
        <span>{max} km</span>
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Build**

```bash
pnpm vite build 2>&1 | tail -5
```

- [ ] **Step 3: Checkpoint**

> **Checkpoint 4a.9:** RadiusSlider fertig (kein Unit-Test — dünner Wrapper, durch Smoke abgedeckt).

---

### Task 4a.10: `CategoryPicker`-Komponente

**Files:**
- Create: `src/components/search/CategoryPicker.tsx`

Zweck: Liste aller Branchen mit Checkbox, Farb-Swatch, Name und Score. Zwei Helper: „Alle aktiv" / „Nichts aktiv".

- [ ] **Step 1: Komponente schreiben**

```tsx
// src/components/search/CategoryPicker.tsx
import { Checkbox } from "@/components/ui/checkbox"
import { Label } from "@/components/ui/label"
import { Button } from "@/components/ui/button"
import type { CategoryRow } from "@/lib/tauri"

interface CategoryPickerProps {
  categories: CategoryRow[]
  selected: Set<number>
  onChange: (next: Set<number>) => void
}

export function CategoryPicker({ categories, selected, onChange }: CategoryPickerProps) {
  const toggle = (id: number, checked: boolean) => {
    const next = new Set(selected)
    if (checked) next.add(id)
    else next.delete(id)
    onChange(next)
  }
  const setAll = (val: boolean) => {
    onChange(val ? new Set(categories.map(c => c.id)) : new Set())
  }

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <Label>Branchen ({selected.size}/{categories.length})</Label>
        <div className="flex gap-2">
          <Button type="button" variant="ghost" size="sm" onClick={() => setAll(true)}>Alle</Button>
          <Button type="button" variant="ghost" size="sm" onClick={() => setAll(false)}>Keine</Button>
        </div>
      </div>
      <div className="border rounded-md max-h-64 overflow-y-auto divide-y">
        {categories.map(c => {
          const id = `cat-${c.id}`
          const isOn = selected.has(c.id)
          return (
            <label key={c.id} htmlFor={id} className="flex items-center gap-3 px-3 py-2 cursor-pointer hover:bg-accent/40">
              <Checkbox
                id={id}
                checked={isOn}
                onCheckedChange={(v) => toggle(c.id, v === true)}
              />
              <span className="inline-block size-3 rounded-sm shrink-0" style={{ background: c.color }} />
              <span className="flex-1 text-sm truncate">{c.name_de}</span>
              <span className="text-xs text-muted-foreground tabular-nums">{c.probability_weight}%</span>
            </label>
          )
        })}
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Build + Tests**

```bash
pnpm vite build 2>&1 | tail -5
pnpm test 2>&1 | tail -5
```

Erwartet: beide grün.

- [ ] **Step 3: Checkpoint**

> **Checkpoint 4a.10:** CategoryPicker fertig.

---

### Task 4a.11: `NewSearchPage` — echter Neue-Suche-Flow

**Files:**
- Create: `src/pages/NewSearchPage.tsx`
- Delete: `src/pages/DebugSearchPage.tsx`
- Modify: `src/App.tsx` (Import + Routing-Eintrag umstellen)

Zweck: Seite, die alles zusammenbindet — Adress-Input, Karten-Picker, Radius-Slider, Branchen-Picker, Start-Button, Progress-Anzeige. Initial-Categories-Auswahl: **alle aktivierten** (basierend auf `enabled` in DB).

- [ ] **Step 1: Datei schreiben**

```tsx
// src/pages/NewSearchPage.tsx
import { useEffect, useMemo, useState } from "react"
import { Button } from "@/components/ui/button"
import { api, type CategoryRow, type SearchStats, type ProgressEvent, type GeocodeSuggestion } from "@/lib/tauri"
import { CenterPickerMap, type Center } from "@/components/map/CenterPickerMap"
import { AddressSearchInput } from "@/components/search/AddressSearchInput"
import { RadiusSlider } from "@/components/search/RadiusSlider"
import { CategoryPicker } from "@/components/search/CategoryPicker"
import { logger } from "@/lib/logger"

export function NewSearchPage() {
  const [cats, setCats] = useState<CategoryRow[]>([])
  const [selectedCats, setSelectedCats] = useState<Set<number>>(new Set())
  const [center, setCenter] = useState<Center | null>(null)
  const [centerLabel, setCenterLabel] = useState<string | null>(null)
  const [radiusKm, setRadiusKm] = useState(25)
  const [progress, setProgress] = useState<ProgressEvent | null>(null)
  const [stats, setStats] = useState<SearchStats | null>(null)
  const [busy, setBusy] = useState(false)
  const [err, setErr] = useState<string | null>(null)

  useEffect(() => {
    api.listCategories()
      .then(all => {
        setCats(all)
        setSelectedCats(new Set(all.filter(c => c.enabled).map(c => c.id)))
      })
      .catch(e => logger.error("listCategories failed", { e: String(e) }))
    const unp = api.onSearchProgress(setProgress)
    const und = api.onSearchDone(setStats)
    return () => { unp.then(f => f()); und.then(f => f()) }
  }, [])

  const canStart = useMemo(
    () => !!center && selectedCats.size > 0 && !busy,
    [center, selectedCats, busy]
  )

  const onAddressPick = (s: GeocodeSuggestion) => {
    setCenter({ lat: s.lat, lng: s.lng })
    setCenterLabel(s.display_name)
    logger.info("address picked", { display_len: s.display_name.length })
  }

  const onMapClick = (c: Center) => {
    setCenter(c)
    setCenterLabel(null) // Karten-Klick hat kein menschliches Label
  }

  const runSearch = async () => {
    if (!center) return
    setBusy(true); setErr(null); setProgress(null); setStats(null)
    logger.info("search start", {
      lat: center.lat.toFixed(4),
      lng: center.lng.toFixed(4),
      radius_km: radiusKm,
      cats: selectedCats.size,
    })
    try {
      await api.startSearch({
        center_lat: center.lat,
        center_lng: center.lng,
        radius_km: radiusKm,
        category_ids: Array.from(selectedCats),
      })
    } catch (e) {
      setErr(String(e))
      logger.error("search failed", { e: String(e) })
    } finally {
      setBusy(false)
    }
  }

  return (
    <div className="h-full flex">
      {/* Linke Spalte: Formular */}
      <div className="w-96 border-r flex flex-col">
        <div className="p-4 border-b">
          <h2 className="text-lg font-semibold">Neue Suche</h2>
        </div>
        <div className="flex-1 overflow-y-auto p-4 space-y-5">
          <div className="space-y-2">
            <AddressSearchInput onPick={onAddressPick} placeholder="Adresse, Stadt oder PLZ…" />
            <p className="text-xs text-muted-foreground">
              {centerLabel ?? (center
                ? `Mittelpunkt: ${center.lat.toFixed(3)}, ${center.lng.toFixed(3)} (per Karten-Klick)`
                : "Oder klick rechts auf die Karte.")}
            </p>
          </div>

          <RadiusSlider value={radiusKm} onChange={setRadiusKm} />

          <CategoryPicker
            categories={cats}
            selected={selectedCats}
            onChange={setSelectedCats}
          />

          <Button onClick={runSearch} disabled={!canStart} className="w-full">
            {busy ? "Suche läuft…" : "Suche starten"}
          </Button>

          {(progress || stats || err) && (
            <div className="text-sm space-y-1 pt-2 border-t">
              {progress && (
                <div>Tile {progress.tile_idx}/{progress.tile_total} · +{progress.last_count} (gesamt {progress.running_total_inserted})</div>
              )}
              {stats && (
                <div className="text-green-700 dark:text-green-400">
                  Fertig: {stats.neu_imported} neu / {stats.duplicates_skipped} Duplikate in {Math.round(stats.dauer_ms / 100) / 10}s
                </div>
              )}
              {err && <div className="text-red-600">{err}</div>}
            </div>
          )}
        </div>
      </div>

      {/* Rechte Spalte: Karte */}
      <div className="flex-1 relative">
        <CenterPickerMap
          center={center}
          radiusKm={radiusKm}
          onCenterChange={onMapClick}
        />
      </div>
    </div>
  )
}
```

⚠️ **Bekannte Einschränkung:** `CenterPickerMap` liest `initialView` nur beim Mount — wenn der User eine Adresse wählt, wird zwar Pin + Kreis aktualisiert, aber die Karte zoomt nicht automatisch hin. Für MVP akzeptabel; falls im späteren Smoke-Test unbequem, in Plan 4b eine `flyTo`-Prop nachrüsten (trivial, ~5 Zeilen).

- [ ] **Step 2: `App.tsx` aktualisieren**

```tsx
// src/App.tsx — ersetze nur die Import-Zeile + die Routing-Zeile
import { NewSearchPage } from "@/pages/NewSearchPage"

// ... und die Zeile
{view === "search" && <DebugSearchPage />}
// ersetze durch
{view === "search" && <NewSearchPage />}
```

Alte Zeile mit `DebugSearchPage` entfernen.

- [ ] **Step 3: Alte Datei löschen**

```bash
rm /Users/jan/Dev/Projects/ProjectAlpha/src/pages/DebugSearchPage.tsx
```

- [ ] **Step 4: Build + Tests**

```bash
pnpm vite build 2>&1 | tail -5
pnpm test 2>&1 | tail -5
```

Erwartet: beide grün.

- [ ] **Step 5: Checkpoint**

> **Checkpoint 4a.11:** Echte Neue-Suche-Seite ersetzt das Debug-UI.

---

### Task 4a.12: AddressSearchInput in ManualAddDialog integrieren

**Files:**
- Modify: `src/components/manual/ManualAddDialog.tsx`

Zweck: Der Vater soll beim manuellen Anlegen einer Firma die Adresse tippen können, statt Koordinaten zu kopieren. Beim Vorschlags-Klick werden `lat`, `lng`, ggf. `street`, `city`, `postal_code` geparst.

Nominatim liefert `display_name` als Komma-getrennte Kette, z. B. `"Bahnhofstraße 1, 30159 Hannover, Niedersachsen, Deutschland"`. Echte Adress-Details kämen über `addressdetails=1` (nicht aktiviert, YAGNI). Für MVP gilt: **nur `lat`/`lng` werden aus dem Pick gesetzt** — der User korrigiert Straße/PLZ/Stadt danach manuell. Der gefundene `display_name` wird als Placeholder unter dem Input angezeigt, damit der User abschreiben kann.

- [ ] **Step 1: Erst die Datei lesen**

Bevor du editierst, lies den aktuellen Inhalt von `src/components/manual/ManualAddDialog.tsx` (er wurde in Plan 2 angelegt und hat evtl. kleinere Abweichungen zum hier gezeigten Snippet). Die Edits unten gehen von einer bekannten Form-Shape aus (`{ name, street, postal_code, city, phone, email, website, industry_category_id, lat, lng }`). Falls der aktuelle State anders heißt, passe die Patches entsprechend an.

- [ ] **Step 2: Datei anpassen**

Im Header oben neue Imports hinzufügen:

```tsx
import { AddressSearchInput } from "@/components/search/AddressSearchInput"
import type { GeocodeSuggestion } from "@/lib/tauri"
```

`form`-State erweitern, um den angezeigten Adress-Display zu halten:

```tsx
  const [form, setForm] = useState({
    name: "", street: "", postal_code: "", city: "",
    phone: "", email: "", website: "",
    industry_category_id: "1", lat: "", lng: "",
  })
  const [addressHint, setAddressHint] = useState<string | null>(null)
```

Der Alert bei fehlenden Koordinaten wird geändert — statt „Plan 4 wird Adress-Suche bekommen" jetzt eine freundlichere Nachricht. Den Block

```tsx
    if (!form.lat || !form.lng) {
      alert("Bitte Koordinaten eingeben (z.B. von Google Maps Rechtsklick → erste Zahl). Plan 4 wird Adress-Suche bekommen.")
      return
    }
```

ersetzen durch:

```tsx
    if (!form.lat || !form.lng) {
      alert("Bitte eine Adresse suchen (oder Koordinaten per Hand eintragen).")
      return
    }
```

Und im JSX-Abschnitt, **ganz oben** innerhalb `<div className="space-y-3">`, vor dem `([["name", ...]]).map(...)`, einfügen:

```tsx
          <div className="space-y-1">
            <Label>Adresse suchen</Label>
            <AddressSearchInput
              onPick={(s: GeocodeSuggestion) => {
                setForm(f => ({ ...f, lat: String(s.lat), lng: String(s.lng) }))
                setAddressHint(s.display_name)
              }}
              placeholder="z.B. Bahnhofstr. 1, Hannover"
            />
            {addressHint && (
              <p className="text-xs text-muted-foreground truncate">Gefunden: {addressHint}</p>
            )}
          </div>
```

- [ ] **Step 3: Build + Tests**

```bash
pnpm vite build 2>&1 | tail -5
pnpm test 2>&1 | tail -5
```

Erwartet: beide grün. Keine Regression in bestehenden Tests (der Dialog ist nicht unit-getestet).

- [ ] **Step 4: Checkpoint**

> **Checkpoint 4a.12:** ManualAddDialog hat jetzt Adress-Suche.

---

### Task 4a.13: Logging- und PII-Review

**Files:** keine (nur Verifikation)

Zweck: Sicherstellen, dass die neuen Logs keine Suchbegriffe (Adressen / Such-Intentionen) oder PII enthalten.

- [ ] **Step 1: Neue Log-Punkte durchgehen**

```bash
grep -rn "logger\." src/components/search src/pages/NewSearchPage.tsx 2>&1
grep -rn "tracing::" src-tauri/src/nominatim src-tauri/src/commands.rs src-tauri/src/db/geocode_cache.rs 2>&1
```

Erlaubt:
- `logger.info("geocode done", { q_len, count })` — Länge + Anzahl, kein Query-String
- `logger.info("address picked", { display_len })` — Länge des gewählten Display-Names
- `logger.info("search start", { lat, lng, radius_km, cats })` — Koordinaten + Zahlen
- `tracing::debug!(q_len, dauer_ms, count, "nominatim query")` — OK
- `tracing::debug!(q_len, "geocode cache hit/miss")` — OK

**Nicht erlaubt** (muss entfernt werden, falls vorhanden):
- Rohe Query-Strings (`q`, `query`, `value`, `display_name`) in Logs
- Telefon-, Mail-, Ansprechpartner-Daten
- Notiz-Inhalte

Falls irgendwo ein Query-String im Log landet → entfernen und Checkpoint neu.

- [ ] **Step 2: Checkpoint**

> **Checkpoint 4a.13:** PII-Audit clean.

---

### Task 4a.14: Live-Smoke-Test (aufgeschoben)

**Files:** keine

Der User testet am Ende der gesamten Phase-4-Arbeiten (nach Plan 4b) manuell. Hier nur die Acceptance-Kriterien dokumentieren, damit beim späteren Durchklicken nichts vergessen wird:

**Acceptance-Kriterien (später durchzuklicken):**

1. Sidebar → „Neue Suche": neue Seite mit Formular links, Karte rechts → ✓
2. Ins Adress-Feld „Hannover" tippen → nach ~500 ms erscheinen Vorschläge → ✓
3. Vorschlag „Hannover, Niedersachsen, Deutschland" anklicken → Marker + Radius-Kreis erscheinen auf der Karte → ✓
4. Radius-Slider auf 50 bewegen → Kreis wächst live → ✓
5. In „Branchen" alle abwählen, dann nur „Logistik / Spedition" aktivieren → Such-Button bleibt aktiv → ✓
6. „Suche starten" → Progress-Events → Erfolgsmeldung → neue Firmen in Liste/Karte → ✓
7. Nochmal „Hannover" eingeben → Vorschlag kommt **sofort** (Cache-Hit, kein 500-ms-Warten) → ✓
8. Sidebar → Firmen → „Manuell"-Button → im Dialog das neue Adress-Feld nutzen → Koordinaten werden eingesetzt → Firma anlegen → ✓
9. Logs prüfen:
   ```bash
   tail -100 ~/Library/Application\ Support/projektalpha/logs/projektalpha.log.*
   ```
   Erwartet: `nominatim query`, `geocode cache hit/miss`, `search start`, `address picked` — **keine** Query-Strings, **keine** PII.
10. App neu starten → alle Ansichten (Firmen, Neue Suche, Karte) rendern ohne Fehler → ✓

- [ ] **Step 1: Checkpoint markieren, User informieren**

> **Checkpoint 4a.14 = Plan 4a fertig** (Implementierung). Smoke-Test folgt am Ende der Phase-4-Arbeiten.

---

## Was am Ende dieses Plans funktioniert

- ✅ Nominatim-Geocoding mit 30-Tage-DB-Cache, 1-req/s-Rate-Limit, User-Agent gemäß OSM-Policy
- ✅ Drei neue Such-Komponenten: AddressSearchInput (debounced typeahead), RadiusSlider (1–300 km), CategoryPicker (Checkboxes mit Farbe + Score)
- ✅ Neue-Suche-Seite ersetzt das Debug-UI vollständig
- ✅ Adress-Suche auch im Manual-Add-Dialog — der Vater tippt statt Koordinaten zu kopieren
- ✅ Rust-Test-Suite gewachsen: 44 → 57 (4 geocode_cache + 9 nominatim)
- ✅ Frontend-Test-Suite gewachsen: 20 → 24 (4 neue für AddressSearchInput debounce-Logik)
- ✅ Strukturierte Logs ohne Query-Strings / PII
- ✅ DebugSearchPage gelöscht

## Was bewusst NICHT in diesem Plan ist

- **Als Profil speichern** — braucht Profile-Liste + Edit/Duplicate/Delete-UI → Plan 4b
- **Branchen-Editor** (Name, OSM-Tags JSON, Score-Slider, Pin-Farbe, enable/disable) → Plan 4b
- **Settings-UI** (Backup/Restore, DB-Pfad öffnen, Über-Tab) → Plan 4b
- **`addressdetails=1`** für strukturierte Adress-Auto-Fill im Manual-Add-Dialog → Plan 5 Polish (YAGNI — User kann Felder nach Adress-Pick manuell ergänzen)
- **flyTo-Re-Center auf CenterPickerMap** bei Adress-Pick → Plan 4b wenn im Smoke-Test als Pain-Point → ansonsten bleiben
- **Stadt-Autocomplete mit Fuzzy-Matching** → YAGNI, Nominatim übernimmt das serverseitig
- **Dashboard mit KPIs + Today-Liste** → Plan 5
- **Auto-Updater + GitHub Actions Cross-Build** → Plan 6

---

## Nächster Plan

**Plan 4b — Settings-UI + Such-Profile-Verwaltung** (Branchen-Editor, Profile-CRUD inkl. „Als Profil speichern"-Flow aus 4a, Backup/Restore-UI, DB-Pfad-Öffnen, Über-Tab).
