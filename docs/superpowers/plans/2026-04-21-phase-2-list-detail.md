# Phase 2: Liste + Detail + Status + Activity-Log – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Echtes Lead-Management-UI: Sidebar/TopBar-Layout, Firmen-Liste mit Filter+Suche, Detail-Sheet (slide from right) mit Status-Änderung, Activity-Log-Timeline, Wiedervorlage, Ansprechpartner, manuelles Hinzufügen. Plus Frontend-Logger + Error-Boundary + Crash-File-Handler aus Spec §10a (in Plan 1 bewusst zurückgestellt). Nach diesem Plan ist die App **produktiv nutzbar** für Vater.

**Architecture:** Backend bekommt drei neue Module (`db::activity`, mehr `commands.rs`-Endpunkte) – pure Erweiterung des Plan-1-Kerns ohne Refactor. Frontend ersetzt das Debug-UI durch ein dauerhaftes Layout: Sidebar (Routing per Zustand-State, kein React Router – KISS), TopBar (Theme-Toggle + Backup-Stub-Button), Hauptbereich mit Firmen-Liste links / Detail-Sheet von rechts. State-Management mit Zustand (eine Store-Datei pro Domäne). Alle UI-Strings auf Deutsch.

**Tech Stack:** Tauri 2 / React 19 / TypeScript / Tailwind v4 / shadcn/ui (neue Components: Sheet, Input, Select, Badge, Calendar+Popover, Textarea, Label, Dialog, ScrollArea, Separator) / Zustand / date-fns / Vitest + React Testing Library (neu in diesem Plan) / lucide-react (Icons – schon installiert).

**Spec-Referenz:** `docs/superpowers/specs/2026-04-21-firmensuche-design.md`
- §5 (Datenmodell – activity_log) · §6.2 (Status-Update-Flow) · §7.2/7.3 (Liste + Detail-Screens) · §10a.2 (Frontend-Log-Bridge) · §10a.4 (Crash-Handling)

**CLAUDE.md-Prinzipien**: UX zuerst (kurze Klickwege, sofortige Reaktion), nicht unnötig kompliziert (kein React Router, kein Custom-CSS), Tests + Logs für alles.

**Kein Git** – Checkpoints statt Commits.

---

## Datei-Struktur (Ziel nach diesem Plan)

```
src/
├─ App.tsx                              # nur noch Layout-Wurzel
├─ main.tsx                             # + Error-Boundary
├─ index.css
├─ lib/
│  ├─ tauri.ts                          # erweitert um neue Commands
│  ├─ logger.ts                         # NEU: Frontend-Logger via Tauri-Event
│  └─ format.ts                         # NEU: Datums-/Score-Formatter
├─ components/
│  ├─ layout/
│  │  ├─ Sidebar.tsx                    # NEU
│  │  ├─ TopBar.tsx                     # NEU
│  │  └─ AppLayout.tsx                  # NEU: Sidebar + TopBar + Slot
│  ├─ companies/
│  │  ├─ CompanyList.tsx                # NEU: virtualisierte Liste
│  │  ├─ CompanyCard.tsx                # NEU: ein Listeneintrag
│  │  ├─ FilterBar.tsx                  # NEU: Status/Branche/Score/Suche
│  │  ├─ StatusBadge.tsx                # NEU: farbige Status-Pille
│  │  ├─ ScoreBadge.tsx                 # NEU
│  │  └─ EmptyState.tsx                 # NEU
│  ├─ detail/
│  │  ├─ CompanyDetailSheet.tsx         # NEU: shadcn Sheet
│  │  ├─ StatusEditor.tsx               # NEU: Status-Dropdown
│  │  ├─ FollowupEditor.tsx             # NEU: DatePicker
│  │  ├─ ContactPersonEditor.tsx        # NEU: Inline-Input
│  │  ├─ ActivityTimeline.tsx           # NEU: vertikale Timeline
│  │  └─ AddActivityForm.tsx            # NEU: Notiz/Anruf/Mail/Besuch
│  ├─ manual/
│  │  └─ ManualAddDialog.tsx            # NEU: Dialog mit Form
│  ├─ ErrorBoundary.tsx                 # NEU
│  └─ ui/                               # shadcn-Komponenten (auto-generated)
├─ pages/
│  ├─ DebugSearchPage.tsx               # bleibt vorerst (für Suche-Trigger bis Plan 4)
│  ├─ CompaniesPage.tsx                 # NEU: Liste + Detail-Sheet kombiniert
│  └─ NotImplementedPage.tsx            # Platzhalter für Karte/Profile/Settings (Plan 3+)
├─ stores/
│  ├─ uiStore.ts                        # NEU: currentView, theme, sheet open/close
│  └─ filterStore.ts                    # NEU: Status/Branche/Score/Suchstring
└─ tests/                               # NEU: Frontend-Tests
   ├─ setup.ts
   └─ ...

src-tauri/src/
├─ db/
│  ├─ activity.rs                       # NEU: CRUD für activity_log
│  └─ companies.rs                      # MODIFIY: list/get/update_status/update_followup/update_contact_person/delete
└─ commands.rs                          # MODIFY: ~7 neue Commands
```

---

# PHASE 2 – Liste + Detail + Status + Activity-Log

## Backend-Erweiterungen

---

### Task 2.1: Activity-Log-Repository (TDD)

**Files:**
- Create: `src-tauri/src/db/activity.rs`
- Modify: `src-tauri/src/db/mod.rs` (add `pub mod activity;`)

- [ ] **Step 1: Modul + Tests + Implementation**

```rust
// src-tauri/src/db/activity.rs
use crate::error::AppResult;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewActivity {
    pub company_id: String,
    pub r#type: String, // "notiz" | "anruf" | "mail" | "besuch" | "status_änderung"
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ActivityRow {
    pub id: String,
    pub company_id: String,
    pub r#type: String,
    pub content: String,
    pub created_at: String,
}

pub async fn add(pool: &SqlitePool, a: &NewActivity) -> AppResult<ActivityRow> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO activity_log (id, company_id, type, content, created_at) VALUES (?,?,?,?,?)"
    )
    .bind(&id).bind(&a.company_id).bind(&a.r#type).bind(&a.content).bind(&now)
    .execute(pool).await?;
    Ok(ActivityRow {
        id, company_id: a.company_id.clone(), r#type: a.r#type.clone(),
        content: a.content.clone(), created_at: now,
    })
}

pub async fn list_for_company(pool: &SqlitePool, company_id: &str) -> AppResult<Vec<ActivityRow>> {
    let rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
        "SELECT id, company_id, type, content, created_at FROM activity_log WHERE company_id = ? ORDER BY created_at DESC"
    ).bind(company_id).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|(id, company_id, r#type, content, created_at)| ActivityRow {
        id, company_id, r#type, content, created_at,
    }).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{open_in_memory, companies::{insert_or_merge, NewCompany}};

    async fn seed_company(pool: &SqlitePool) -> String {
        let c = NewCompany {
            osm_id: Some("node/1".into()), name: "X".into(),
            street: None, postal_code: None, city: None, country: "DE".into(),
            lat: 0.0, lng: 0.0, phone: None, email: None, website: None,
            industry_category_id: Some(1), size_estimate: None, probability_score: 50,
            source: "osm".into(),
        };
        insert_or_merge(pool, &c).await.unwrap();
        let row: (String,) = sqlx::query_as("SELECT id FROM companies WHERE osm_id = 'node/1'")
            .fetch_one(pool).await.unwrap();
        row.0
    }

    #[tokio::test]
    async fn add_then_list_returns_entry() {
        let pool = open_in_memory().await;
        let cid = seed_company(&pool).await;
        let r = add(&pool, &NewActivity {
            company_id: cid.clone(), r#type: "notiz".into(), content: "test".into()
        }).await.unwrap();
        assert!(!r.id.is_empty());

        let list = list_for_company(&pool, &cid).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].content, "test");
    }

    #[tokio::test]
    async fn list_orders_newest_first() {
        let pool = open_in_memory().await;
        let cid = seed_company(&pool).await;
        for i in 0..3 {
            add(&pool, &NewActivity {
                company_id: cid.clone(), r#type: "notiz".into(), content: format!("{i}")
            }).await.unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        let list = list_for_company(&pool, &cid).await.unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].content, "2"); // newest first
    }

    #[tokio::test]
    async fn list_for_unknown_company_is_empty() {
        let pool = open_in_memory().await;
        let list = list_for_company(&pool, "nonexistent").await.unwrap();
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn deleting_company_cascades_activity() {
        let pool = open_in_memory().await;
        let cid = seed_company(&pool).await;
        add(&pool, &NewActivity { company_id: cid.clone(), r#type: "notiz".into(), content: "x".into() }).await.unwrap();
        sqlx::query("DELETE FROM companies WHERE id = ?").bind(&cid).execute(&pool).await.unwrap();
        let list = list_for_company(&pool, &cid).await.unwrap();
        assert!(list.is_empty(), "FK CASCADE should delete activity");
    }
}
```

- [ ] **Step 2: Modul registrieren**

`src-tauri/src/db/mod.rs`: `pub mod activity;`

- [ ] **Step 3: Tests verifizieren**

```bash
cd src-tauri && cargo test --lib db::activity 2>&1 | tail -10
```
Expected: 4 passed.

- [ ] **Step 4: Checkpoint**

> **Checkpoint 2.1:** activity_log CRUD + Cascade-Delete getestet.

---

### Task 2.2: Companies-Repo erweitern: list / get / updates (TDD)

**Files:**
- Modify: `src-tauri/src/db/companies.rs` (add functions)

- [ ] **Step 1: Tests + Implementation hinzufügen**

In `src-tauri/src/db/companies.rs` ergänzen (NICHT die existierenden Funktionen ersetzen):

```rust
#[derive(Debug, Clone, Serialize)]
pub struct CompanyRow {
    pub id: String,
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
    pub category_name: Option<String>,
    pub category_color: Option<String>,
    pub probability_score: i64,
    pub status: String,
    pub contact_person: Option<String>,
    pub last_contact_at: Option<String>,
    pub next_followup_at: Option<String>,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct ListFilter {
    pub status: Option<String>,            // None = alle
    pub category_ids: Option<Vec<i64>>,    // None = alle
    pub min_score: Option<i64>,            // None = 0
    pub search: Option<String>,            // case-insensitive auf name/city
    pub limit: Option<i64>,                // Default 200
    pub offset: Option<i64>,               // Default 0
}

pub async fn list(pool: &SqlitePool, f: &ListFilter) -> AppResult<Vec<CompanyRow>> {
    let mut sql = String::from(
        "SELECT c.id, c.name, c.street, c.postal_code, c.city, c.country, c.lat, c.lng,
                c.phone, c.email, c.website, c.industry_category_id, ic.name_de, ic.color,
                c.probability_score, c.status, c.contact_person,
                c.last_contact_at, c.next_followup_at, c.source, c.created_at, c.updated_at
         FROM companies c LEFT JOIN industry_categories ic ON ic.id = c.industry_category_id
         WHERE 1=1"
    );
    let mut binds: Vec<String> = vec![];
    if let Some(s) = &f.status { sql.push_str(" AND c.status = ?"); binds.push(s.clone()); }
    if let Some(ids) = &f.category_ids { if !ids.is_empty() {
        let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        sql.push_str(&format!(" AND c.industry_category_id IN ({})", placeholders));
        for id in ids { binds.push(id.to_string()); }
    }}
    if let Some(min) = f.min_score { sql.push_str(" AND c.probability_score >= ?"); binds.push(min.to_string()); }
    if let Some(q) = &f.search {
        if !q.trim().is_empty() {
            sql.push_str(" AND (LOWER(c.name) LIKE ? OR LOWER(IFNULL(c.city,'')) LIKE ?)");
            let pat = format!("%{}%", q.to_lowercase());
            binds.push(pat.clone()); binds.push(pat);
        }
    }
    sql.push_str(" ORDER BY c.probability_score DESC, c.name ASC LIMIT ? OFFSET ?");
    binds.push(f.limit.unwrap_or(200).to_string());
    binds.push(f.offset.unwrap_or(0).to_string());

    let mut q = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, Option<String>, String, f64, f64,
        Option<String>, Option<String>, Option<String>, Option<i64>, Option<String>, Option<String>,
        i64, String, Option<String>, Option<String>, Option<String>, String, String, String)>(&sql);
    for b in &binds { q = q.bind(b); }
    let rows = q.fetch_all(pool).await?;
    Ok(rows.into_iter().map(|t| CompanyRow {
        id: t.0, name: t.1, street: t.2, postal_code: t.3, city: t.4, country: t.5,
        lat: t.6, lng: t.7, phone: t.8, email: t.9, website: t.10,
        industry_category_id: t.11, category_name: t.12, category_color: t.13,
        probability_score: t.14, status: t.15, contact_person: t.16,
        last_contact_at: t.17, next_followup_at: t.18, source: t.19,
        created_at: t.20, updated_at: t.21,
    }).collect())
}

pub async fn get(pool: &SqlitePool, id: &str) -> AppResult<Option<CompanyRow>> {
    let row: Option<(String, String, Option<String>, Option<String>, Option<String>, String, f64, f64,
        Option<String>, Option<String>, Option<String>, Option<i64>, Option<String>, Option<String>,
        i64, String, Option<String>, Option<String>, Option<String>, String, String, String)> =
        sqlx::query_as(
            "SELECT c.id, c.name, c.street, c.postal_code, c.city, c.country, c.lat, c.lng,
                    c.phone, c.email, c.website, c.industry_category_id, ic.name_de, ic.color,
                    c.probability_score, c.status, c.contact_person,
                    c.last_contact_at, c.next_followup_at, c.source, c.created_at, c.updated_at
             FROM companies c LEFT JOIN industry_categories ic ON ic.id = c.industry_category_id
             WHERE c.id = ?"
        ).bind(id).fetch_optional(pool).await?;
    Ok(row.map(|t| CompanyRow {
        id: t.0, name: t.1, street: t.2, postal_code: t.3, city: t.4, country: t.5,
        lat: t.6, lng: t.7, phone: t.8, email: t.9, website: t.10,
        industry_category_id: t.11, category_name: t.12, category_color: t.13,
        probability_score: t.14, status: t.15, contact_person: t.16,
        last_contact_at: t.17, next_followup_at: t.18, source: t.19,
        created_at: t.20, updated_at: t.21,
    }))
}

pub async fn update_status(pool: &SqlitePool, id: &str, new_status: &str) -> AppResult<String> {
    if !["neu","angefragt","kunde","kein_kunde"].contains(&new_status) {
        return Err(AppError::InvalidInput(format!("invalid status: {new_status}")));
    }
    let now = Utc::now().to_rfc3339();
    let prev: (String,) = sqlx::query_as("SELECT status FROM companies WHERE id = ?")
        .bind(id).fetch_one(pool).await?;
    sqlx::query(
        "UPDATE companies SET status = ?, last_contact_at = ?, updated_at = ? WHERE id = ?"
    ).bind(new_status).bind(&now).bind(&now).bind(id).execute(pool).await?;
    Ok(prev.0)
}

pub async fn update_followup(pool: &SqlitePool, id: &str, when: Option<&str>) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE companies SET next_followup_at = ?, updated_at = ? WHERE id = ?")
        .bind(when).bind(&now).bind(id).execute(pool).await?;
    Ok(())
}

pub async fn update_contact_person(pool: &SqlitePool, id: &str, person: Option<&str>) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE companies SET contact_person = ?, updated_at = ? WHERE id = ?")
        .bind(person).bind(&now).bind(id).execute(pool).await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM companies WHERE id = ?").bind(id).execute(pool).await?;
    Ok(())
}
```

Den `use crate::error::AppError;` falls noch nicht da ist, am Anfang der Datei ergänzen.

- [ ] **Step 2: Tests** (in dasselbe `mod tests` ergänzen):

```rust
#[tokio::test]
async fn list_filters_by_status() {
    let pool = open_in_memory().await;
    insert_or_merge(&pool, &sample(Some("a"), "A")).await.unwrap();
    insert_or_merge(&pool, &sample(Some("b"), "B")).await.unwrap();
    let id_b: (String,) = sqlx::query_as("SELECT id FROM companies WHERE osm_id='b'")
        .fetch_one(&pool).await.unwrap();
    update_status(&pool, &id_b.0, "kunde").await.unwrap();

    let kunden = list(&pool, &ListFilter { status: Some("kunde".into()), ..Default::default() }).await.unwrap();
    assert_eq!(kunden.len(), 1);
    assert_eq!(kunden[0].name, "B");
}

#[tokio::test]
async fn list_filters_by_search() {
    let pool = open_in_memory().await;
    let mut a = sample(Some("a"), "Müller GmbH"); a.city = Some("Hannover".into());
    let mut b = sample(Some("b"), "Schmidt AG"); b.city = Some("Bremen".into());
    insert_or_merge(&pool, &a).await.unwrap();
    insert_or_merge(&pool, &b).await.unwrap();
    let r = list(&pool, &ListFilter { search: Some("müller".into()), ..Default::default() }).await.unwrap();
    assert_eq!(r.len(), 1);
}

#[tokio::test]
async fn list_filters_by_min_score() {
    let pool = open_in_memory().await;
    let mut a = sample(Some("a"), "A"); a.probability_score = 30;
    let mut b = sample(Some("b"), "B"); b.probability_score = 90;
    insert_or_merge(&pool, &a).await.unwrap();
    insert_or_merge(&pool, &b).await.unwrap();
    let r = list(&pool, &ListFilter { min_score: Some(50), ..Default::default() }).await.unwrap();
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].name, "B");
}

#[tokio::test]
async fn update_status_sets_last_contact_at() {
    let pool = open_in_memory().await;
    insert_or_merge(&pool, &sample(Some("a"), "A")).await.unwrap();
    let id: (String,) = sqlx::query_as("SELECT id FROM companies WHERE osm_id='a'").fetch_one(&pool).await.unwrap();
    let prev = update_status(&pool, &id.0, "angefragt").await.unwrap();
    assert_eq!(prev, "neu");
    let row = get(&pool, &id.0).await.unwrap().unwrap();
    assert_eq!(row.status, "angefragt");
    assert!(row.last_contact_at.is_some());
}

#[tokio::test]
async fn update_status_rejects_invalid_value() {
    let pool = open_in_memory().await;
    insert_or_merge(&pool, &sample(Some("a"), "A")).await.unwrap();
    let id: (String,) = sqlx::query_as("SELECT id FROM companies WHERE osm_id='a'").fetch_one(&pool).await.unwrap();
    let r = update_status(&pool, &id.0, "garbage").await;
    assert!(r.is_err());
}

#[tokio::test]
async fn update_followup_sets_and_clears() {
    let pool = open_in_memory().await;
    insert_or_merge(&pool, &sample(Some("a"), "A")).await.unwrap();
    let id: (String,) = sqlx::query_as("SELECT id FROM companies WHERE osm_id='a'").fetch_one(&pool).await.unwrap();
    update_followup(&pool, &id.0, Some("2026-05-01T09:00:00+00:00")).await.unwrap();
    let row = get(&pool, &id.0).await.unwrap().unwrap();
    assert_eq!(row.next_followup_at.as_deref(), Some("2026-05-01T09:00:00+00:00"));
    update_followup(&pool, &id.0, None).await.unwrap();
    let row2 = get(&pool, &id.0).await.unwrap().unwrap();
    assert!(row2.next_followup_at.is_none());
}

#[tokio::test]
async fn delete_removes_company() {
    let pool = open_in_memory().await;
    insert_or_merge(&pool, &sample(Some("a"), "A")).await.unwrap();
    let id: (String,) = sqlx::query_as("SELECT id FROM companies WHERE osm_id='a'").fetch_one(&pool).await.unwrap();
    delete(&pool, &id.0).await.unwrap();
    assert!(get(&pool, &id.0).await.unwrap().is_none());
}
```

- [ ] **Step 3: Tests verifizieren**

```bash
cd src-tauri && cargo test --lib db::companies 2>&1 | tail -15
```
Expected: 11 passed (4 existierende + 7 neue).

- [ ] **Step 4: Checkpoint**

> **Checkpoint 2.2:** Companies-Repo komplett: list/get/update_status/update_followup/update_contact_person/delete getestet.

---

### Task 2.3: Tauri-Commands für Liste / Detail / Updates / Activity (TDD-light)

**Files:**
- Modify: `src-tauri/src/commands.rs` (add 7 new commands)
- Modify: `src-tauri/src/lib.rs` (register them)

- [ ] **Step 1: Commands ergänzen**

Im bestehenden `commands.rs` ergänzen (Imports oben aktualisieren):

```rust
use crate::db::{
    activity::{self, ActivityRow, NewActivity},
    companies::{self, CompanyRow, ListFilter, NewCompany, InsertResult},
};
use crate::error::{AppError, AppResult};
use serde::{Deserialize};
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn list_companies(
    state: State<'_, Arc<crate::AppState>>,
    filter: ListFilter,
) -> AppResult<Vec<CompanyRow>> {
    companies::list(&state.db, &filter).await
}

#[tauri::command]
pub async fn get_company(
    state: State<'_, Arc<crate::AppState>>,
    id: String,
) -> AppResult<Option<CompanyRow>> {
    companies::get(&state.db, &id).await
}

#[derive(Deserialize)]
pub struct UpdateStatusPayload { pub id: String, pub new_status: String }

#[tauri::command]
pub async fn update_company_status(
    state: State<'_, Arc<crate::AppState>>,
    payload: UpdateStatusPayload,
) -> AppResult<()> {
    let prev = companies::update_status(&state.db, &payload.id, &payload.new_status).await?;
    activity::add(&state.db, &NewActivity {
        company_id: payload.id.clone(),
        r#type: "status_änderung".into(),
        content: format!("von {} auf {}", prev, payload.new_status),
    }).await?;
    tracing::info!(company_id = %payload.id, prev = %prev, new = %payload.new_status, "status changed");
    Ok(())
}

#[derive(Deserialize)]
pub struct UpdateFollowupPayload { pub id: String, pub when: Option<String> }

#[tauri::command]
pub async fn update_company_followup(
    state: State<'_, Arc<crate::AppState>>,
    payload: UpdateFollowupPayload,
) -> AppResult<()> {
    companies::update_followup(&state.db, &payload.id, payload.when.as_deref()).await
}

#[derive(Deserialize)]
pub struct UpdateContactPersonPayload { pub id: String, pub person: Option<String> }

#[tauri::command]
pub async fn update_company_contact_person(
    state: State<'_, Arc<crate::AppState>>,
    payload: UpdateContactPersonPayload,
) -> AppResult<()> {
    companies::update_contact_person(&state.db, &payload.id, payload.person.as_deref()).await
}

#[tauri::command]
pub async fn delete_company(
    state: State<'_, Arc<crate::AppState>>,
    id: String,
) -> AppResult<()> {
    companies::delete(&state.db, &id).await?;
    tracing::info!(company_id = %id, "company deleted");
    Ok(())
}

#[tauri::command]
pub async fn list_activity(
    state: State<'_, Arc<crate::AppState>>,
    company_id: String,
) -> AppResult<Vec<ActivityRow>> {
    activity::list_for_company(&state.db, &company_id).await
}

#[tauri::command]
pub async fn add_activity(
    state: State<'_, Arc<crate::AppState>>,
    payload: NewActivity,
) -> AppResult<ActivityRow> {
    activity::add(&state.db, &payload).await
}

#[tauri::command]
pub async fn add_manual_company(
    state: State<'_, Arc<crate::AppState>>,
    payload: NewCompany,
) -> AppResult<InsertResult> {
    if payload.source != "manual" {
        return Err(AppError::InvalidInput("source must be 'manual'".into()));
    }
    if payload.name.trim().is_empty() {
        return Err(AppError::InvalidInput("name required".into()));
    }
    companies::insert_or_merge(&state.db, &payload).await
}
```

- [ ] **Step 2: In `lib.rs::run()` registrieren**

Im `invoke_handler!` die Liste erweitern:
```rust
.invoke_handler(tauri::generate_handler![
    greet,
    commands::list_categories,
    commands::start_search,
    commands::list_companies,
    commands::get_company,
    commands::update_company_status,
    commands::update_company_followup,
    commands::update_company_contact_person,
    commands::delete_company,
    commands::list_activity,
    commands::add_activity,
    commands::add_manual_company,
])
```

- [ ] **Step 3: Build verifizieren**

```bash
cd src-tauri && cargo build 2>&1 | tail -10 && cargo test --lib 2>&1 | tail -5
```
Expected: clean build, alle Tests grün (43 Tests = 32 + 4 activity + 7 companies-list).

- [ ] **Step 4: Checkpoint**

> **Checkpoint 2.3:** Backend hat alle Commands für Liste/Detail/Status/Activity/Delete/Add-Manual.

---

## Frontend-Logger + Error-Boundary (deferred from Plan 1)

---

### Task 2.4: Frontend-Log-Bridge (Spec §10a.2)

**Files:**
- Create: `src/lib/logger.ts`
- Modify: `src-tauri/src/commands.rs` (add `frontend_log` command)
- Modify: `src-tauri/src/lib.rs` (register)

- [ ] **Step 1: Backend-Command zum Empfangen**

In `commands.rs`:
```rust
#[derive(Deserialize)]
pub struct FrontendLogPayload {
    pub level: String,        // "info" | "warn" | "error"
    pub message: String,
    pub context: Option<serde_json::Value>,
}

#[tauri::command]
pub fn frontend_log(payload: FrontendLogPayload) -> AppResult<()> {
    let ctx = payload.context.map(|c| c.to_string()).unwrap_or_default();
    match payload.level.as_str() {
        "error" => tracing::error!(source = "frontend", context = %ctx, "{}", payload.message),
        "warn"  => tracing::warn!(source = "frontend", context = %ctx, "{}", payload.message),
        _       => tracing::info!(source = "frontend", context = %ctx, "{}", payload.message),
    }
    Ok(())
}
```

In `lib.rs::invoke_handler!` ergänzen: `commands::frontend_log,`.

- [ ] **Step 2: Frontend-Wrapper**

`src/lib/logger.ts`:
```ts
import { invoke } from "@tauri-apps/api/core"

type Level = "info" | "warn" | "error"

const send = (level: Level, message: string, context?: unknown) => {
  // local console
  // eslint-disable-next-line no-console
  (level === "error" ? console.error : level === "warn" ? console.warn : console.info)(
    `[${level}] ${message}`, context ?? ""
  )
  // forward to backend
  invoke("frontend_log", { payload: { level, message, context } }).catch(() => {
    // logger MUST never throw
  })
}

export const logger = {
  info:  (msg: string, ctx?: unknown) => send("info", msg, ctx),
  warn:  (msg: string, ctx?: unknown) => send("warn", msg, ctx),
  error: (msg: string, ctx?: unknown) => send("error", msg, ctx),
}
```

- [ ] **Step 3: Smoke**

```bash
cd src-tauri && cargo build 2>&1 | tail -5
cd .. && pnpm vite build 2>&1 | tail -5
```
Expected: clean.

- [ ] **Step 4: Checkpoint**

> **Checkpoint 2.4:** Frontend-Log-Bridge live. PII-Regel beachten: NIE personenbezogene Daten loggen.

---

### Task 2.5: Error-Boundary + Crash-File-Handler (Spec §10a.4)

**Files:**
- Create: `src/components/ErrorBoundary.tsx`
- Modify: `src/main.tsx` (wrap App in ErrorBoundary)
- Create: `src-tauri/src/crash.rs`
- Modify: `src-tauri/src/lib.rs` (panic-hook + module decl)

- [ ] **Step 1: Backend-Crash-Modul**

`src-tauri/src/crash.rs`:
```rust
use std::path::PathBuf;
use std::sync::OnceLock;

static CRASH_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn init(crash_dir: PathBuf) {
    let _ = std::fs::create_dir_all(&crash_dir);
    let _ = CRASH_DIR.set(crash_dir);
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        write_crash("rust_panic", &info.to_string());
        prev(info);
    }));
}

pub fn write_crash(kind: &str, body: &str) {
    if let Some(dir) = CRASH_DIR.get() {
        let ts = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
        let path = dir.join(format!("crash-{kind}-{ts}.txt"));
        let content = format!(
            "version: {}\ntimestamp: {}\nkind: {}\n\n{}\n",
            env!("CARGO_PKG_VERSION"),
            chrono::Utc::now().to_rfc3339(),
            kind, body
        );
        let _ = std::fs::write(path, content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn write_crash_creates_file() {
        let dir = tempdir().unwrap();
        init(dir.path().to_path_buf());
        write_crash("test", "hello world");
        let entries: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
        assert_eq!(entries.len(), 1);
    }
}
```

In `lib.rs` zusätzlich:
- `pub mod crash;` oben
- in `run()` direkt nach Logger-Init: `crash::init(dirs::data_dir().expect("data dir").join("projektalpha").join("crashes"));`

Plus einen Tauri-Command, mit dem das Frontend einen Crash melden kann:

`commands.rs`:
```rust
#[derive(Deserialize)]
pub struct CrashReport { pub message: String, pub stack: Option<String> }

#[tauri::command]
pub fn report_frontend_crash(payload: CrashReport) -> AppResult<()> {
    let body = format!("{}\n\n{}", payload.message, payload.stack.unwrap_or_default());
    crate::crash::write_crash("frontend", &body);
    tracing::error!(source = "frontend", "frontend crash reported: {}", payload.message);
    Ok(())
}
```

`lib.rs::invoke_handler!`: `commands::report_frontend_crash,`

- [ ] **Step 2: Frontend-Error-Boundary**

`src/components/ErrorBoundary.tsx`:
```tsx
import { Component, type ErrorInfo, type ReactNode } from "react"
import { invoke } from "@tauri-apps/api/core"

type Props = { children: ReactNode }
type State = { hasError: boolean; error: Error | null }

export class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false, error: null }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    invoke("report_frontend_crash", {
      payload: { message: error.message, stack: `${error.stack ?? ""}\n${info.componentStack ?? ""}` }
    }).catch(() => { /* never throw from boundary */ })
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="p-8 max-w-2xl">
          <h1 className="text-2xl font-bold text-red-600">Etwas ist schiefgelaufen</h1>
          <p className="mt-2 text-sm text-gray-600">
            Die App hat einen unerwarteten Fehler erlebt. Der Fehler wurde lokal gespeichert.
          </p>
          <pre className="mt-4 p-3 bg-gray-100 dark:bg-gray-800 rounded text-xs overflow-auto">
            {this.state.error?.message}
          </pre>
          <button
            onClick={() => window.location.reload()}
            className="mt-4 px-4 py-2 bg-blue-600 text-white rounded"
          >App neu laden</button>
        </div>
      )
    }
    return this.props.children
  }
}
```

- [ ] **Step 3: In `main.tsx` einwickeln**

```tsx
import React from "react"
import ReactDOM from "react-dom/client"
import App from "./App"
import "./index.css"
import { ErrorBoundary } from "./components/ErrorBoundary"

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>,
)
```

- [ ] **Step 4: Tests + Build**

```bash
cd src-tauri && cargo test --lib crash 2>&1 | tail -5  # 1 passed
cd src-tauri && cargo build 2>&1 | tail -5            # clean
cd .. && pnpm vite build 2>&1 | tail -5               # clean
```

- [ ] **Step 5: Checkpoint**

> **Checkpoint 2.5:** Crash-Handler aktiv. Rust-Panics + Frontend-Errors landen in `<app_data>/projektalpha/crashes/crash-*.txt`.

---

## Frontend-Tests-Setup

---

### Task 2.6: Vitest + React Testing Library aufsetzen

**Files:**
- Create: `vitest.config.ts`, `src/tests/setup.ts`
- Modify: `package.json` (test scripts)
- Create: `src/lib/format.test.ts` (eine Beispiel-Suite – wird in Task 2.7 implementiert)

- [ ] **Step 1: Dependencies**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha
pnpm add -D vitest @vitest/ui jsdom @testing-library/react @testing-library/jest-dom @testing-library/user-event
```

- [ ] **Step 2: `vitest.config.ts`**

```ts
import { defineConfig } from "vitest/config"
import react from "@vitejs/plugin-react"
import path from "path"

export default defineConfig({
  plugins: [react()],
  resolve: { alias: { "@": path.resolve(__dirname, "./src") } },
  test: {
    environment: "jsdom",
    setupFiles: ["./src/tests/setup.ts"],
    css: true,
    globals: true,
  },
})
```

(`@vitejs/plugin-react` is already a dep via the Tauri scaffold; if not, `pnpm add -D @vitejs/plugin-react`.)

- [ ] **Step 3: `src/tests/setup.ts`**

```ts
import "@testing-library/jest-dom/vitest"

// stub Tauri's invoke so component tests don't hit the bridge
import { vi } from "vitest"

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async () => null),
}))

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => () => {}),
}))
```

- [ ] **Step 4: Scripts in `package.json`**

In `"scripts"` ergänzen:
```json
"test": "vitest run",
"test:watch": "vitest",
"test:ui": "vitest --ui"
```

- [ ] **Step 5: Smoke**

```bash
pnpm test 2>&1 | tail -10
```
Expected: „No test files found" — das ist OK, der Setup steht.

- [ ] **Step 6: Checkpoint**

> **Checkpoint 2.6:** Frontend-Test-Harness läuft. Tauri-Bridge gemockt.

---

## Formatters + Stores

---

### Task 2.7: format.ts mit Tests (TDD)

**Files:**
- Create: `src/lib/format.ts`, `src/lib/format.test.ts`

- [ ] **Step 1: Tests zuerst**

`src/lib/format.test.ts`:
```ts
import { describe, expect, it } from "vitest"
import { formatDateDe, formatRelativeDe, statusLabel, statusColor, scoreColor } from "./format"

describe("formatDateDe", () => {
  it("formats ISO to dd.MM.yyyy", () => {
    expect(formatDateDe("2026-04-21T14:30:00Z")).toBe("21.04.2026")
  })
  it("returns dash on null/undefined", () => {
    expect(formatDateDe(null)).toBe("—")
    expect(formatDateDe(undefined)).toBe("—")
  })
})

describe("formatRelativeDe", () => {
  it("returns 'heute' for today", () => {
    const today = new Date().toISOString()
    expect(formatRelativeDe(today)).toBe("heute")
  })
  it("returns 'vor X Tagen' for past dates", () => {
    const d = new Date(); d.setDate(d.getDate() - 5)
    expect(formatRelativeDe(d.toISOString())).toBe("vor 5 Tagen")
  })
})

describe("statusLabel", () => {
  it("maps status keys to German labels", () => {
    expect(statusLabel("neu")).toBe("Neu")
    expect(statusLabel("angefragt")).toBe("Angefragt")
    expect(statusLabel("kunde")).toBe("Kunde")
    expect(statusLabel("kein_kunde")).toBe("Kein Kunde")
  })
})

describe("statusColor", () => {
  it("returns distinct colors per status", () => {
    const colors = ["neu","angefragt","kunde","kein_kunde"].map(statusColor)
    expect(new Set(colors).size).toBe(4)
  })
})

describe("scoreColor", () => {
  it("returns red/yellow/green by tier", () => {
    expect(scoreColor(20)).toContain("red")
    expect(scoreColor(60)).toContain("yellow")
    expect(scoreColor(90)).toContain("green")
  })
})
```

- [ ] **Step 2: Test FAIL**
```bash
pnpm test 2>&1 | tail -20
```
Expected: 5 failing (no module).

- [ ] **Step 3: Implementation**

`src/lib/format.ts`:
```ts
export function formatDateDe(iso?: string | null): string {
  if (!iso) return "—"
  const d = new Date(iso)
  if (isNaN(d.getTime())) return "—"
  const dd = String(d.getDate()).padStart(2, "0")
  const mm = String(d.getMonth() + 1).padStart(2, "0")
  return `${dd}.${mm}.${d.getFullYear()}`
}

export function formatRelativeDe(iso?: string | null): string {
  if (!iso) return "—"
  const d = new Date(iso)
  const now = new Date()
  const days = Math.floor((now.getTime() - d.getTime()) / (1000 * 60 * 60 * 24))
  if (days === 0) return "heute"
  if (days === 1) return "gestern"
  if (days > 1) return `vor ${days} Tagen`
  if (days === -1) return "morgen"
  return `in ${-days} Tagen`
}

const STATUS_LABELS = { neu: "Neu", angefragt: "Angefragt", kunde: "Kunde", kein_kunde: "Kein Kunde" } as const
export type Status = keyof typeof STATUS_LABELS

export function statusLabel(s: string): string {
  return STATUS_LABELS[s as Status] ?? s
}

const STATUS_COLORS: Record<string, string> = {
  neu: "bg-blue-100 text-blue-900 dark:bg-blue-900/40 dark:text-blue-200",
  angefragt: "bg-yellow-100 text-yellow-900 dark:bg-yellow-900/40 dark:text-yellow-200",
  kunde: "bg-green-100 text-green-900 dark:bg-green-900/40 dark:text-green-200",
  kein_kunde: "bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-400",
}

export function statusColor(s: string): string {
  return STATUS_COLORS[s] ?? STATUS_COLORS.neu
}

export function scoreColor(score: number): string {
  if (score < 40) return "bg-red-100 text-red-900 dark:bg-red-900/30 dark:text-red-200"
  if (score < 75) return "bg-yellow-100 text-yellow-900 dark:bg-yellow-900/30 dark:text-yellow-200"
  return "bg-green-100 text-green-900 dark:bg-green-900/30 dark:text-green-200"
}
```

- [ ] **Step 4: Test PASS**
```bash
pnpm test 2>&1 | tail -10
```
Expected: 5 passed.

- [ ] **Step 5: Checkpoint**

> **Checkpoint 2.7:** Formatter-Modul mit Tests, deutsche Labels, Status-/Score-Farben.

---

### Task 2.8: Zustand-Stores (uiStore, filterStore)

**Files:**
- Create: `src/stores/uiStore.ts`, `src/stores/filterStore.ts`

- [ ] **Step 1: Zustand installieren**
```bash
pnpm add zustand
```

- [ ] **Step 2: `uiStore.ts`**

```ts
import { create } from "zustand"

export type View = "companies" | "search" | "map" | "profiles" | "settings"

interface UiState {
  currentView: View
  selectedCompanyId: string | null
  setView: (v: View) => void
  selectCompany: (id: string | null) => void
}

export const useUiStore = create<UiState>((set) => ({
  currentView: "companies",
  selectedCompanyId: null,
  setView: (v) => set({ currentView: v }),
  selectCompany: (id) => set({ selectedCompanyId: id }),
}))
```

- [ ] **Step 3: `filterStore.ts`**

```ts
import { create } from "zustand"

export interface CompanyFilter {
  status: string | null         // null = alle
  categoryIds: number[] | null  // null = alle
  minScore: number              // 0..100
  search: string
}

interface FilterState extends CompanyFilter {
  setStatus: (s: string | null) => void
  setCategoryIds: (ids: number[] | null) => void
  setMinScore: (n: number) => void
  setSearch: (q: string) => void
  reset: () => void
}

const initial: CompanyFilter = { status: null, categoryIds: null, minScore: 0, search: "" }

export const useFilterStore = create<FilterState>((set) => ({
  ...initial,
  setStatus: (s) => set({ status: s }),
  setCategoryIds: (ids) => set({ categoryIds: ids }),
  setMinScore: (n) => set({ minScore: n }),
  setSearch: (q) => set({ search: q }),
  reset: () => set({ ...initial }),
}))
```

- [ ] **Step 4: Smoke**
```bash
pnpm test 2>&1 | tail -5    # immer noch 5 passed
pnpm vite build 2>&1 | tail -5
```

- [ ] **Step 5: Checkpoint**

> **Checkpoint 2.8:** Stores für UI-Zustand und Filter angelegt. Zustand minimal (eine Konvention: ein Store pro Domäne).

---

## Layout

---

### Task 2.9: shadcn-Komponenten nachinstallieren

**Files:** auto-generated unter `src/components/ui/`

- [ ] **Step 1**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha
pnpm dlx shadcn@latest add sheet input select badge card scroll-area separator textarea label dialog popover calendar
```

Expected: 12 Components werden erzeugt. Falls eine Komponente nicht im Nova-Preset verfügbar ist (sollte aber alle sein), explizit dokumentieren.

- [ ] **Step 2: Build**
```bash
pnpm vite build 2>&1 | tail -5
```
Expected: clean (mehr Module, sonst nichts Auffälliges).

- [ ] **Step 3: Checkpoint**

> **Checkpoint 2.9:** Alle benötigten shadcn-Komponenten verfügbar.

---

### Task 2.10: AppLayout mit Sidebar + TopBar

**Files:**
- Create: `src/components/layout/Sidebar.tsx`, `src/components/layout/TopBar.tsx`, `src/components/layout/AppLayout.tsx`

- [ ] **Step 1: Sidebar**

```tsx
// src/components/layout/Sidebar.tsx
import { useUiStore, type View } from "@/stores/uiStore"
import { Building2, Map, FolderOpen, Settings, Search } from "lucide-react"
import { cn } from "@/lib/utils"

const items: { key: View; label: string; Icon: typeof Building2 }[] = [
  { key: "companies", label: "Firmen",       Icon: Building2 },
  { key: "search",    label: "Neue Suche",   Icon: Search },
  { key: "map",       label: "Karte",        Icon: Map },
  { key: "profiles",  label: "Profile",      Icon: FolderOpen },
  { key: "settings",  label: "Einstellungen", Icon: Settings },
]

export function Sidebar() {
  const { currentView, setView } = useUiStore()
  return (
    <nav className="w-56 border-r bg-sidebar text-sidebar-foreground flex flex-col py-4">
      <div className="px-4 mb-6">
        <h1 className="font-semibold tracking-tight">ProjektAlpha</h1>
      </div>
      <ul className="space-y-1 px-2">
        {items.map(({ key, label, Icon }) => (
          <li key={key}>
            <button
              onClick={() => setView(key)}
              className={cn(
                "w-full flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-colors",
                "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
                currentView === key && "bg-sidebar-accent text-sidebar-accent-foreground font-medium"
              )}
            >
              <Icon className="size-4" />
              {label}
            </button>
          </li>
        ))}
      </ul>
    </nav>
  )
}
```

- [ ] **Step 2: TopBar**

```tsx
// src/components/layout/TopBar.tsx
import { Button } from "@/components/ui/button"
import { Moon, Sun, Download } from "lucide-react"
import { useEffect, useState } from "react"

export function TopBar() {
  const [dark, setDark] = useState(() =>
    document.documentElement.classList.contains("dark")
  )
  useEffect(() => {
    document.documentElement.classList.toggle("dark", dark)
  }, [dark])

  return (
    <header className="h-12 border-b flex items-center justify-end gap-2 px-4">
      <Button variant="ghost" size="icon" onClick={() => alert("Backup-Funktion kommt in Plan 4")}>
        <Download className="size-4" />
        <span className="sr-only">Backup</span>
      </Button>
      <Button variant="ghost" size="icon" onClick={() => setDark(d => !d)}>
        {dark ? <Sun className="size-4" /> : <Moon className="size-4" />}
        <span className="sr-only">Theme umschalten</span>
      </Button>
    </header>
  )
}
```

- [ ] **Step 3: AppLayout**

```tsx
// src/components/layout/AppLayout.tsx
import type { ReactNode } from "react"
import { Sidebar } from "./Sidebar"
import { TopBar } from "./TopBar"

export function AppLayout({ children }: { children: ReactNode }) {
  return (
    <div className="h-screen flex">
      <Sidebar />
      <div className="flex-1 flex flex-col">
        <TopBar />
        <main className="flex-1 overflow-hidden">{children}</main>
      </div>
    </div>
  )
}
```

- [ ] **Step 4: App.tsx neu schreiben (vorerst noch mit Debug-Page)**

```tsx
// src/App.tsx
import { AppLayout } from "@/components/layout/AppLayout"
import { useUiStore } from "@/stores/uiStore"
import { CompaniesPage } from "@/pages/CompaniesPage"
import { DebugSearchPage } from "@/pages/DebugSearchPage"
import { NotImplementedPage } from "@/pages/NotImplementedPage"

function App() {
  const view = useUiStore(s => s.currentView)
  return (
    <AppLayout>
      {view === "companies" && <CompaniesPage />}
      {view === "search" && <DebugSearchPage />}
      {(view === "map" || view === "profiles" || view === "settings") && <NotImplementedPage view={view} />}
    </AppLayout>
  )
}
export default App
```

- [ ] **Step 5: Stub-Pages erstellen, damit der Build durchläuft**

`src/pages/NotImplementedPage.tsx`:
```tsx
export function NotImplementedPage({ view }: { view: string }) {
  return (
    <div className="p-8">
      <h2 className="text-xl font-semibold mb-2">{view}</h2>
      <p className="text-sm text-muted-foreground">Diese Ansicht kommt in einem späteren Plan.</p>
    </div>
  )
}
```

`src/pages/CompaniesPage.tsx` (vorerst leer, wird in 2.11 gefüllt):
```tsx
export function CompaniesPage() {
  return <div className="p-8">Lade...</div>
}
```

`src/pages/DebugSearchPage.tsx` (das alte App.tsx-Inhalt verschieben):
```tsx
// Den kompletten alten App.tsx-Body (Test-Suche-Button + Branchen-Liste) hier reinkopieren.
// Komponenten-Definition exportieren als `DebugSearchPage`.
import { useEffect, useState } from "react"
import { Button } from "@/components/ui/button"
import { api, type CategoryRow, type SearchStats, type ProgressEvent } from "@/lib/tauri"

export function DebugSearchPage() {
  const [cats, setCats] = useState<CategoryRow[]>([])
  const [progress, setProgress] = useState<ProgressEvent | null>(null)
  const [stats, setStats] = useState<SearchStats | null>(null)
  const [busy, setBusy] = useState(false)
  const [err, setErr] = useState<string | null>(null)

  useEffect(() => {
    api.listCategories().then(setCats).catch(e => setErr(String(e)))
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
      <h2 className="text-xl font-semibold">Debug-Suche (provisorisch)</h2>
      <Button onClick={runSearch} disabled={busy}>
        {busy ? "Suche läuft..." : "Test-Suche Hannover 5 km"}
      </Button>
      {progress && <div className="text-sm">Tile {progress.tile_idx}/{progress.tile_total} · +{progress.last_count} (gesamt {progress.running_total_inserted})</div>}
      {stats && <pre className="bg-gray-100 dark:bg-gray-800 p-3 rounded text-xs">{JSON.stringify(stats, null, 2)}</pre>}
      {err && <pre className="text-red-600 text-sm">{err}</pre>}
    </div>
  )
}
```

- [ ] **Step 6: Build + smoke**
```bash
pnpm vite build 2>&1 | tail -5
```
Expected: clean.

- [ ] **Step 7: Checkpoint**

> **Checkpoint 2.10:** Layout steht. Sidebar wechselt Views via Zustand-Store. Theme-Toggle funktioniert.

---

## Companies-Liste + Filter

---

### Task 2.11: CompanyList + CompanyCard + StatusBadge + ScoreBadge

**Files:**
- Create: `src/components/companies/{StatusBadge,ScoreBadge,CompanyCard,CompanyList,EmptyState}.tsx`
- Modify: `src/lib/tauri.ts` (TS types + new wrappers)
- Modify: `src/pages/CompaniesPage.tsx`

- [ ] **Step 1: Erweitere `tauri.ts`**

Ergänze in `src/lib/tauri.ts`:
```ts
export type CompanyRow = {
  id: string
  name: string
  street: string | null
  postal_code: string | null
  city: string | null
  country: string
  lat: number
  lng: number
  phone: string | null
  email: string | null
  website: string | null
  industry_category_id: number | null
  category_name: string | null
  category_color: string | null
  probability_score: number
  status: string
  contact_person: string | null
  last_contact_at: string | null
  next_followup_at: string | null
  source: string
  created_at: string
  updated_at: string
}

export type ListFilter = {
  status?: string | null
  category_ids?: number[] | null
  min_score?: number | null
  search?: string | null
  limit?: number | null
  offset?: number | null
}

export type ActivityRow = {
  id: string
  company_id: string
  type: string
  content: string
  created_at: string
}

// in api object ergänzen:
listCompanies: (filter: ListFilter) => invoke<CompanyRow[]>("list_companies", { filter }),
getCompany: (id: string) => invoke<CompanyRow | null>("get_company", { id }),
updateCompanyStatus: (id: string, new_status: string) =>
  invoke<void>("update_company_status", { payload: { id, new_status } }),
updateCompanyFollowup: (id: string, when: string | null) =>
  invoke<void>("update_company_followup", { payload: { id, when } }),
updateCompanyContactPerson: (id: string, person: string | null) =>
  invoke<void>("update_company_contact_person", { payload: { id, person } }),
deleteCompany: (id: string) => invoke<void>("delete_company", { id }),
listActivity: (company_id: string) => invoke<ActivityRow[]>("list_activity", { companyId: company_id }),
addActivity: (payload: { company_id: string; type: string; content: string }) =>
  invoke<ActivityRow>("add_activity", { payload }),
```

(Achtung: Tauri serialisiert Argument-Namen camelCase ↔ snake_case automatisch. `companyId` → `company_id`. Zur Sicherheit entweder testen oder beide Schreibweisen probieren.)

- [ ] **Step 2: StatusBadge + ScoreBadge**

```tsx
// src/components/companies/StatusBadge.tsx
import { statusLabel, statusColor } from "@/lib/format"
import { cn } from "@/lib/utils"

export function StatusBadge({ status }: { status: string }) {
  return (
    <span className={cn("inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium", statusColor(status))}>
      {statusLabel(status)}
    </span>
  )
}
```

```tsx
// src/components/companies/ScoreBadge.tsx
import { scoreColor } from "@/lib/format"
import { cn } from "@/lib/utils"

export function ScoreBadge({ score }: { score: number }) {
  return (
    <span className={cn("inline-flex items-center px-2 py-0.5 rounded text-xs font-medium tabular-nums", scoreColor(score))}>
      {score}%
    </span>
  )
}
```

- [ ] **Step 3: CompanyCard**

```tsx
// src/components/companies/CompanyCard.tsx
import type { CompanyRow } from "@/lib/tauri"
import { StatusBadge } from "./StatusBadge"
import { ScoreBadge } from "./ScoreBadge"
import { formatRelativeDe } from "@/lib/format"
import { cn } from "@/lib/utils"

export function CompanyCard({
  company, selected, onClick,
}: { company: CompanyRow; selected: boolean; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "w-full text-left p-3 border-b hover:bg-accent/50 transition-colors",
        selected && "bg-accent"
      )}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="min-w-0 flex-1">
          <div className="font-medium truncate">{company.name}</div>
          <div className="text-xs text-muted-foreground truncate">
            {company.city ?? "—"} · {company.category_name ?? "Sonstige"}
          </div>
        </div>
        <ScoreBadge score={company.probability_score} />
      </div>
      <div className="mt-2 flex items-center justify-between gap-2">
        <StatusBadge status={company.status} />
        <span className="text-xs text-muted-foreground">{formatRelativeDe(company.last_contact_at)}</span>
      </div>
    </button>
  )
}
```

- [ ] **Step 4: EmptyState**

```tsx
// src/components/companies/EmptyState.tsx
import { Building2 } from "lucide-react"

export function EmptyState({ message }: { message: string }) {
  return (
    <div className="flex-1 flex flex-col items-center justify-center text-muted-foreground p-8">
      <Building2 className="size-12 mb-3 opacity-30" />
      <p className="text-sm">{message}</p>
    </div>
  )
}
```

- [ ] **Step 5: CompanyList**

```tsx
// src/components/companies/CompanyList.tsx
import type { CompanyRow } from "@/lib/tauri"
import { ScrollArea } from "@/components/ui/scroll-area"
import { CompanyCard } from "./CompanyCard"
import { EmptyState } from "./EmptyState"

export function CompanyList({
  companies, selectedId, onSelect,
}: { companies: CompanyRow[]; selectedId: string | null; onSelect: (id: string) => void }) {
  if (companies.length === 0) {
    return <EmptyState message="Keine Firmen passen zum Filter. Suche starten oder Filter anpassen." />
  }
  return (
    <ScrollArea className="h-full">
      <div className="border-r">
        {companies.map(c => (
          <CompanyCard key={c.id} company={c} selected={selectedId === c.id} onClick={() => onSelect(c.id)} />
        ))}
      </div>
    </ScrollArea>
  )
}
```

- [ ] **Step 6: CompaniesPage füllen** (Detail-Sheet kommt in 2.13, hier nur Liste-Layout)

```tsx
// src/pages/CompaniesPage.tsx
import { useEffect, useState } from "react"
import { api, type CompanyRow } from "@/lib/tauri"
import { useUiStore } from "@/stores/uiStore"
import { useFilterStore } from "@/stores/filterStore"
import { CompanyList } from "@/components/companies/CompanyList"
import { logger } from "@/lib/logger"

export function CompaniesPage() {
  const [companies, setCompanies] = useState<CompanyRow[]>([])
  const [loading, setLoading] = useState(true)
  const { selectedCompanyId, selectCompany } = useUiStore()
  const filter = useFilterStore()

  useEffect(() => {
    setLoading(true)
    api.listCompanies({
      status: filter.status,
      category_ids: filter.categoryIds,
      min_score: filter.minScore,
      search: filter.search,
    })
      .then(setCompanies)
      .catch(e => logger.error("listCompanies failed", { e: String(e) }))
      .finally(() => setLoading(false))
  }, [filter.status, filter.categoryIds, filter.minScore, filter.search])

  return (
    <div className="h-full flex">
      <div className="w-96 flex flex-col">
        {loading
          ? <div className="p-4 text-sm text-muted-foreground">Lade…</div>
          : <CompanyList companies={companies} selectedId={selectedCompanyId} onSelect={selectCompany} />}
      </div>
      <div className="flex-1 p-8 text-muted-foreground">
        {/* Detail-Sheet kommt in 2.13 */}
        {selectedCompanyId ? `Firma ${selectedCompanyId} ausgewählt` : "Firma aus der Liste auswählen"}
      </div>
    </div>
  )
}
```

- [ ] **Step 7: Build + smoke**
```bash
pnpm vite build 2>&1 | tail -5
```

- [ ] **Step 8: Checkpoint**

> **Checkpoint 2.11:** Liste lädt echte Firmen, Auswahl per Klick, Empty-/Loading-States vorhanden.

---

### Task 2.12: FilterBar mit Tests

**Files:**
- Create: `src/components/companies/FilterBar.tsx`, `src/components/companies/FilterBar.test.tsx`
- Modify: `src/pages/CompaniesPage.tsx` (FilterBar einhängen)

- [ ] **Step 1: Component**

```tsx
// src/components/companies/FilterBar.tsx
import { useFilterStore } from "@/stores/filterStore"
import { Input } from "@/components/ui/input"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Button } from "@/components/ui/button"
import { Search, X } from "lucide-react"

export function FilterBar() {
  const { status, search, minScore, setStatus, setSearch, setMinScore, reset } = useFilterStore()

  return (
    <div className="border-b p-3 flex flex-wrap items-center gap-2">
      <div className="relative flex-1 min-w-48">
        <Search className="absolute left-2.5 top-2.5 size-4 text-muted-foreground" />
        <Input
          placeholder="Firma oder Stadt suchen…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="pl-9"
        />
      </div>
      <Select value={status ?? "all"} onValueChange={(v) => setStatus(v === "all" ? null : v)}>
        <SelectTrigger className="w-40"><SelectValue /></SelectTrigger>
        <SelectContent>
          <SelectItem value="all">Alle Status</SelectItem>
          <SelectItem value="neu">Neu</SelectItem>
          <SelectItem value="angefragt">Angefragt</SelectItem>
          <SelectItem value="kunde">Kunde</SelectItem>
          <SelectItem value="kein_kunde">Kein Kunde</SelectItem>
        </SelectContent>
      </Select>
      <div className="flex items-center gap-2 text-sm">
        <label htmlFor="min-score">Min. Score</label>
        <input
          id="min-score"
          type="range" min={0} max={100} step={5}
          value={minScore} onChange={(e) => setMinScore(Number(e.target.value))}
          className="w-24"
        />
        <span className="tabular-nums w-8">{minScore}</span>
      </div>
      <Button variant="ghost" size="sm" onClick={reset}>
        <X className="size-3 mr-1" /> Reset
      </Button>
    </div>
  )
}
```

- [ ] **Step 2: Test**

```tsx
// src/components/companies/FilterBar.test.tsx
import { describe, expect, it } from "vitest"
import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { FilterBar } from "./FilterBar"
import { useFilterStore } from "@/stores/filterStore"

describe("FilterBar", () => {
  it("typing in search updates the store", async () => {
    useFilterStore.getState().reset()
    render(<FilterBar />)
    const input = screen.getByPlaceholderText(/Firma oder Stadt/i)
    await userEvent.type(input, "müller")
    expect(useFilterStore.getState().search).toBe("müller")
  })

  it("reset clears all filters", async () => {
    useFilterStore.getState().setSearch("foo")
    useFilterStore.getState().setMinScore(50)
    render(<FilterBar />)
    await userEvent.click(screen.getByRole("button", { name: /Reset/i }))
    const s = useFilterStore.getState()
    expect(s.search).toBe("")
    expect(s.minScore).toBe(0)
  })
})
```

- [ ] **Step 3: FilterBar einhängen**

In `CompaniesPage.tsx` zwischen Liste und Detail einfügen:
```tsx
import { FilterBar } from "@/components/companies/FilterBar"
// oben in der Komponente:
<div className="w-96 flex flex-col">
  <FilterBar />
  ... (CompanyList wie gehabt)
</div>
```

- [ ] **Step 4: Tests + Build**
```bash
pnpm test 2>&1 | tail -10  # 7 passed (5 format + 2 filterbar)
pnpm vite build 2>&1 | tail -5
```

- [ ] **Step 5: Checkpoint**

> **Checkpoint 2.12:** FilterBar steuert Liste live. Search debounced über useEffect-Trigger der Liste.

---

## Detail-Sheet

---

### Task 2.13: CompanyDetailSheet (Container) mit Status-Editor

**Files:**
- Create: `src/components/detail/CompanyDetailSheet.tsx`, `src/components/detail/StatusEditor.tsx`
- Modify: `src/pages/CompaniesPage.tsx` (Sheet einhängen)

- [ ] **Step 1: StatusEditor**

```tsx
// src/components/detail/StatusEditor.tsx
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { statusLabel } from "@/lib/format"

export function StatusEditor({
  value, onChange,
}: { value: string; onChange: (next: string) => void }) {
  return (
    <Select value={value} onValueChange={onChange}>
      <SelectTrigger className="w-48"><SelectValue /></SelectTrigger>
      <SelectContent>
        {(["neu","angefragt","kunde","kein_kunde"] as const).map(s => (
          <SelectItem key={s} value={s}>{statusLabel(s)}</SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}
```

- [ ] **Step 2: CompanyDetailSheet (Header + Status only, andere Editoren in 2.14/2.15)**

```tsx
// src/components/detail/CompanyDetailSheet.tsx
import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetDescription } from "@/components/ui/sheet"
import { Button } from "@/components/ui/button"
import { Phone, Mail, Globe, Trash2 } from "lucide-react"
import { useEffect, useState } from "react"
import { api, type CompanyRow } from "@/lib/tauri"
import { ScoreBadge } from "@/components/companies/ScoreBadge"
import { StatusEditor } from "./StatusEditor"
import { logger } from "@/lib/logger"

export function CompanyDetailSheet({
  companyId, open, onOpenChange, onChanged,
}: {
  companyId: string | null
  open: boolean
  onOpenChange: (open: boolean) => void
  onChanged: () => void   // refetch parent list
}) {
  const [company, setCompany] = useState<CompanyRow | null>(null)
  const [busy, setBusy] = useState(false)

  useEffect(() => {
    if (!companyId) { setCompany(null); return }
    api.getCompany(companyId)
      .then(setCompany)
      .catch(e => logger.error("getCompany failed", { id: companyId }))
  }, [companyId])

  const updateStatus = async (next: string) => {
    if (!company) return
    setBusy(true)
    try {
      await api.updateCompanyStatus(company.id, next)
      const refreshed = await api.getCompany(company.id)
      setCompany(refreshed)
      onChanged()
    } catch (e) { logger.error("updateCompanyStatus failed", { id: company.id }) }
    finally { setBusy(false) }
  }

  const onDelete = async () => {
    if (!company) return
    if (!confirm(`Firma "${company.name}" wirklich löschen?`)) return
    try {
      await api.deleteCompany(company.id)
      onOpenChange(false)
      onChanged()
    } catch (e) { logger.error("deleteCompany failed", { id: company.id }) }
  }

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent className="w-[480px] sm:max-w-none overflow-y-auto">
        {company ? (
          <>
            <SheetHeader>
              <div className="flex items-start gap-2">
                <SheetTitle className="text-lg">{company.name}</SheetTitle>
                <ScoreBadge score={company.probability_score} />
              </div>
              <SheetDescription>
                {[company.street, company.postal_code, company.city].filter(Boolean).join(", ") || "Adresse unbekannt"}
              </SheetDescription>
            </SheetHeader>

            <div className="mt-4 flex flex-wrap gap-2">
              {company.phone && (
                <Button variant="outline" size="sm" asChild>
                  <a href={`tel:${company.phone}`}><Phone className="size-3 mr-1" />{company.phone}</a>
                </Button>
              )}
              {company.email && (
                <Button variant="outline" size="sm" asChild>
                  <a href={`mailto:${company.email}`}><Mail className="size-3 mr-1" />Mail</a>
                </Button>
              )}
              {company.website && (
                <Button variant="outline" size="sm" asChild>
                  <a href={company.website} target="_blank" rel="noopener noreferrer">
                    <Globe className="size-3 mr-1" />Website
                  </a>
                </Button>
              )}
            </div>

            <div className="mt-6 space-y-4">
              <div>
                <div className="text-sm font-medium mb-1">Status</div>
                <StatusEditor value={company.status} onChange={updateStatus} />
              </div>
              {/* Followup, ContactPerson, ActivityLog kommen in 2.14/2.15 */}
            </div>

            <div className="mt-8 pt-4 border-t">
              <Button variant="destructive" size="sm" onClick={onDelete} disabled={busy}>
                <Trash2 className="size-3 mr-1" /> Firma löschen
              </Button>
            </div>
          </>
        ) : (
          <div className="text-sm text-muted-foreground">Lade…</div>
        )}
      </SheetContent>
    </Sheet>
  )
}
```

- [ ] **Step 3: In CompaniesPage einhängen**

```tsx
// In src/pages/CompaniesPage.tsx neben CompanyList:
import { CompanyDetailSheet } from "@/components/detail/CompanyDetailSheet"

const [refreshTick, setRefreshTick] = useState(0)
// useEffect dependency um refreshTick erweitern:
}, [filter.status, filter.categoryIds, filter.minScore, filter.search, refreshTick])

return (
  <div className="h-full flex">
    {/* sidebar mit Liste */}
    ...
    <CompanyDetailSheet
      companyId={selectedCompanyId}
      open={!!selectedCompanyId}
      onOpenChange={(o) => { if (!o) selectCompany(null) }}
      onChanged={() => setRefreshTick(t => t + 1)}
    />
  </div>
)
```

(Die rechte Spalte mit „Firma X ausgewählt" kann jetzt entfernt werden — das Sheet schiebt sich rein.)

- [ ] **Step 4: Build + smoke**
```bash
pnpm vite build 2>&1 | tail -5
```

- [ ] **Step 5: Checkpoint**

> **Checkpoint 2.13:** Detail-Sheet öffnet, zeigt Adresse + Aktionsbuttons + Status-Dropdown. Status-Änderung schreibt in DB + Activity-Log + refresht Liste.

---

### Task 2.14: Followup-Editor + ContactPerson-Editor

**Files:**
- Create: `src/components/detail/FollowupEditor.tsx`, `src/components/detail/ContactPersonEditor.tsx`
- Modify: `CompanyDetailSheet.tsx` (Editoren einhängen)

- [ ] **Step 1: FollowupEditor**

```tsx
// src/components/detail/FollowupEditor.tsx
import { Calendar } from "@/components/ui/calendar"
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover"
import { Button } from "@/components/ui/button"
import { CalendarDays, X } from "lucide-react"
import { formatDateDe } from "@/lib/format"

export function FollowupEditor({
  value, onChange,
}: { value: string | null; onChange: (next: string | null) => void }) {
  const date = value ? new Date(value) : undefined
  return (
    <div className="flex items-center gap-2">
      <Popover>
        <PopoverTrigger asChild>
          <Button variant="outline" size="sm" className="w-44 justify-start">
            <CalendarDays className="size-3 mr-2" />
            {value ? formatDateDe(value) : "Wiedervorlage setzen"}
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-auto p-0" align="start">
          <Calendar
            mode="single"
            selected={date}
            onSelect={(d) => onChange(d ? d.toISOString() : null)}
            initialFocus
          />
        </PopoverContent>
      </Popover>
      {value && (
        <Button variant="ghost" size="icon" onClick={() => onChange(null)}>
          <X className="size-3" />
        </Button>
      )}
    </div>
  )
}
```

- [ ] **Step 2: ContactPersonEditor (inline)**

```tsx
// src/components/detail/ContactPersonEditor.tsx
import { Input } from "@/components/ui/input"
import { useState, useEffect } from "react"

export function ContactPersonEditor({
  value, onCommit,
}: { value: string | null; onCommit: (next: string | null) => void }) {
  const [v, setV] = useState(value ?? "")
  useEffect(() => { setV(value ?? "") }, [value])
  return (
    <Input
      placeholder="z.B. Frau Müller"
      value={v}
      onChange={(e) => setV(e.target.value)}
      onBlur={() => {
        const next = v.trim() === "" ? null : v.trim()
        if (next !== value) onCommit(next)
      }}
    />
  )
}
```

- [ ] **Step 3: Im Sheet einhängen**

In `CompanyDetailSheet.tsx` unter dem Status-Block ergänzen:

```tsx
import { FollowupEditor } from "./FollowupEditor"
import { ContactPersonEditor } from "./ContactPersonEditor"

const updateFollowup = async (when: string | null) => {
  if (!company) return
  await api.updateCompanyFollowup(company.id, when)
  setCompany(await api.getCompany(company.id))
  onChanged()
}
const updateContactPerson = async (person: string | null) => {
  if (!company) return
  await api.updateCompanyContactPerson(company.id, person)
  setCompany(await api.getCompany(company.id))
  onChanged()
}

// JSX:
<div>
  <div className="text-sm font-medium mb-1">Wiedervorlage</div>
  <FollowupEditor value={company.next_followup_at} onChange={updateFollowup} />
</div>
<div>
  <div className="text-sm font-medium mb-1">Ansprechpartner</div>
  <ContactPersonEditor value={company.contact_person} onCommit={updateContactPerson} />
</div>
```

- [ ] **Step 4: Build**
```bash
pnpm vite build 2>&1 | tail -5
```

- [ ] **Step 5: Checkpoint**

> **Checkpoint 2.14:** Wiedervorlage-DatePicker und Ansprechpartner-Inline-Edit funktional.

---

### Task 2.15: Activity-Timeline + AddActivityForm

**Files:**
- Create: `src/components/detail/ActivityTimeline.tsx`, `src/components/detail/AddActivityForm.tsx`
- Modify: `CompanyDetailSheet.tsx` (einhängen)

- [ ] **Step 1: AddActivityForm**

```tsx
// src/components/detail/AddActivityForm.tsx
import { useState } from "react"
import { Textarea } from "@/components/ui/textarea"
import { Button } from "@/components/ui/button"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Plus } from "lucide-react"

const TYPES = [
  { v: "notiz", l: "Notiz" },
  { v: "anruf", l: "Anruf" },
  { v: "mail",  l: "Mail" },
  { v: "besuch",l: "Besuch" },
] as const

export function AddActivityForm({ onSubmit }: { onSubmit: (type: string, content: string) => Promise<void> }) {
  const [type, setType] = useState<string>("notiz")
  const [text, setText] = useState("")
  const [busy, setBusy] = useState(false)

  const submit = async () => {
    if (text.trim() === "") return
    setBusy(true)
    try {
      await onSubmit(type, text.trim())
      setText("")
    } finally { setBusy(false) }
  }

  return (
    <div className="border rounded-md p-3 space-y-2">
      <div className="flex items-center gap-2">
        <Select value={type} onValueChange={setType}>
          <SelectTrigger className="w-32"><SelectValue /></SelectTrigger>
          <SelectContent>
            {TYPES.map(t => <SelectItem key={t.v} value={t.v}>{t.l}</SelectItem>)}
          </SelectContent>
        </Select>
        <Button size="sm" onClick={submit} disabled={busy || !text.trim()}>
          <Plus className="size-3 mr-1" /> Erfassen
        </Button>
      </div>
      <Textarea
        placeholder="Was ist passiert? (z.B. Frau Müller bittet um Rückruf KW 18)"
        value={text}
        onChange={(e) => setText(e.target.value)}
        rows={3}
      />
    </div>
  )
}
```

- [ ] **Step 2: ActivityTimeline**

```tsx
// src/components/detail/ActivityTimeline.tsx
import type { ActivityRow } from "@/lib/tauri"
import { formatDateDe } from "@/lib/format"
import { Phone, Mail, NotebookPen, Footprints, Activity } from "lucide-react"

const ICONS: Record<string, typeof Phone> = {
  notiz: NotebookPen, anruf: Phone, mail: Mail, besuch: Footprints, status_änderung: Activity,
}
const LABELS: Record<string, string> = {
  notiz: "Notiz", anruf: "Anruf", mail: "Mail", besuch: "Besuch", status_änderung: "Status",
}

export function ActivityTimeline({ entries }: { entries: ActivityRow[] }) {
  if (entries.length === 0) {
    return <div className="text-sm text-muted-foreground">Noch keine Einträge.</div>
  }
  return (
    <ol className="space-y-3">
      {entries.map(e => {
        const Icon = ICONS[e.type] ?? NotebookPen
        return (
          <li key={e.id} className="flex gap-3">
            <div className="mt-0.5">
              <Icon className="size-4 text-muted-foreground" />
            </div>
            <div className="flex-1 min-w-0">
              <div className="text-xs text-muted-foreground">
                {LABELS[e.type] ?? e.type} · {formatDateDe(e.created_at)}
              </div>
              <div className="text-sm whitespace-pre-wrap">{e.content}</div>
            </div>
          </li>
        )
      })}
    </ol>
  )
}
```

- [ ] **Step 3: Im Sheet einhängen**

```tsx
import { ActivityTimeline } from "./ActivityTimeline"
import { AddActivityForm } from "./AddActivityForm"
import type { ActivityRow } from "@/lib/tauri"

const [activity, setActivity] = useState<ActivityRow[]>([])

const loadActivity = async (id: string) => {
  setActivity(await api.listActivity(id))
}

// im useEffect, nach company laden:
if (companyId) loadActivity(companyId)

const addActivity = async (type: string, content: string) => {
  if (!company) return
  await api.addActivity({ company_id: company.id, type, content })
  await loadActivity(company.id)
}

// JSX nach den Editoren:
<div>
  <div className="text-sm font-medium mb-2">Verlauf</div>
  <ActivityTimeline entries={activity} />
</div>
<AddActivityForm onSubmit={addActivity} />
```

- [ ] **Step 4: Build**
```bash
pnpm vite build 2>&1 | tail -5
```

- [ ] **Step 5: Checkpoint**

> **Checkpoint 2.15:** Activity-Timeline lädt Verlauf, neue Notizen/Anrufe werden gespeichert + sofort sichtbar.

---

## Manuelles Hinzufügen

---

### Task 2.16: ManualAddDialog

**Files:**
- Create: `src/components/manual/ManualAddDialog.tsx`
- Modify: `src/components/companies/FilterBar.tsx` (Plus-Button hinzufügen)

- [ ] **Step 1: Dialog**

```tsx
// src/components/manual/ManualAddDialog.tsx
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Button } from "@/components/ui/button"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { useEffect, useState } from "react"
import { api, type CategoryRow } from "@/lib/tauri"
import { Plus } from "lucide-react"

export function ManualAddDialog({ onAdded }: { onAdded: () => void }) {
  const [open, setOpen] = useState(false)
  const [cats, setCats] = useState<CategoryRow[]>([])
  const [busy, setBusy] = useState(false)
  const [form, setForm] = useState({
    name: "", street: "", postal_code: "", city: "",
    phone: "", email: "", website: "",
    industry_category_id: "1", lat: "", lng: "",
  })

  useEffect(() => { if (open) api.listCategories().then(setCats) }, [open])

  const submit = async () => {
    if (!form.name.trim()) return
    if (!form.lat || !form.lng) {
      alert("Bitte Koordinaten eingeben (z.B. von Google Maps Rechtsklick → erste Zahl). Plan 4 wird Adress-Suche bekommen.")
      return
    }
    setBusy(true)
    try {
      const cat = cats.find(c => c.id === Number(form.industry_category_id))
      await api.addManualCompany({
        osm_id: null,
        name: form.name.trim(),
        street: form.street || null,
        postal_code: form.postal_code || null,
        city: form.city || null,
        country: "DE",
        lat: Number(form.lat), lng: Number(form.lng),
        phone: form.phone || null, email: form.email || null, website: form.website || null,
        industry_category_id: Number(form.industry_category_id),
        size_estimate: null,
        probability_score: cat?.probability_weight ?? 50,
        source: "manual",
      })
      onAdded()
      setOpen(false)
      setForm({ name: "", street: "", postal_code: "", city: "", phone: "", email: "", website: "",
                industry_category_id: "1", lat: "", lng: "" })
    } finally { setBusy(false) }
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button size="sm" variant="outline"><Plus className="size-3 mr-1" />Manuell</Button>
      </DialogTrigger>
      <DialogContent className="max-w-md">
        <DialogHeader><DialogTitle>Firma manuell hinzufügen</DialogTitle></DialogHeader>
        <div className="space-y-3">
          {[
            ["name", "Firmenname *"],
            ["street", "Straße + Nr."],
            ["postal_code", "PLZ"],
            ["city", "Stadt"],
            ["lat", "Breitengrad (z.B. 52.3756)"],
            ["lng", "Längengrad (z.B. 9.7320)"],
            ["phone", "Telefon"],
            ["email", "E-Mail"],
            ["website", "Website (https://…)"],
          ].map(([k, label]) => (
            <div key={k}>
              <Label htmlFor={k}>{label}</Label>
              <Input id={k} value={(form as Record<string,string>)[k]}
                onChange={(e) => setForm({ ...form, [k]: e.target.value })} />
            </div>
          ))}
          <div>
            <Label>Branche</Label>
            <Select value={form.industry_category_id} onValueChange={(v) => setForm({ ...form, industry_category_id: v })}>
              <SelectTrigger><SelectValue /></SelectTrigger>
              <SelectContent>
                {cats.map(c => <SelectItem key={c.id} value={String(c.id)}>{c.name_de}</SelectItem>)}
              </SelectContent>
            </Select>
          </div>
        </div>
        <Button onClick={submit} disabled={busy || !form.name.trim()}>Anlegen</Button>
      </DialogContent>
    </Dialog>
  )
}
```

- [ ] **Step 2: tauri.ts erweitern**

```ts
addManualCompany: (payload: {
  osm_id: string | null
  name: string
  street: string | null
  postal_code: string | null
  city: string | null
  country: string
  lat: number; lng: number
  phone: string | null; email: string | null; website: string | null
  industry_category_id: number | null
  size_estimate: string | null
  probability_score: number
  source: "manual"
}) => invoke<{ inserted: boolean; updated_fields: string[] }>("add_manual_company", { payload }),
```

- [ ] **Step 3: In FilterBar einhängen**

```tsx
// FilterBar.tsx ergänzen:
import { ManualAddDialog } from "@/components/manual/ManualAddDialog"
// als Prop aufnehmen:
export function FilterBar({ onAdded }: { onAdded: () => void }) {
  // ... bestehende UI ...
  <ManualAddDialog onAdded={onAdded} />
}
```

In `CompaniesPage.tsx` `<FilterBar onAdded={() => setRefreshTick(t => t+1)} />` rendern.

- [ ] **Step 4: Build**
```bash
pnpm vite build 2>&1 | tail -5
```

- [ ] **Step 5: Checkpoint**

> **Checkpoint 2.16:** Manuelles Hinzufügen funktioniert. Adress-Geocoding wird in Plan 4 nachgereicht.

---

## Polish + Live-Smoke

---

### Task 2.17: Search-Done-Toast → Liste auto-refreshen

**Files:**
- Modify: `src/pages/CompaniesPage.tsx` (Listen-Refresh nach Suche)

- [ ] **Step 1: Listener für `search-done`**

```tsx
// In CompaniesPage.tsx useEffect:
import { api } from "@/lib/tauri"
useEffect(() => {
  const un = api.onSearchDone(() => setRefreshTick(t => t + 1))
  return () => { un.then(f => f()) }
}, [])
```

- [ ] **Step 2: Build + smoke**
```bash
pnpm vite build 2>&1 | tail -5
```

- [ ] **Step 3: Checkpoint**

> **Checkpoint 2.17:** Nach jedem `search-done` aktualisiert sich die Liste automatisch.

---

### Task 2.18: Live-Smoke-Test (manuell, kritisch wichtig)

**Files:** keine

- [ ] **Step 1**: Tauri starten
```bash
pnpm tauri dev
```

- [ ] **Step 2: Manuell durchklicken** (sich Notizen machen — diese Punkte sind die Acceptance-Kriterien für Phase 2):
  1. Sidebar zeigt 5 Items, „Firmen" ist initial aktiv → ✓
  2. Liste zeigt Firmen aus Plan 1 (Hannover-Suche) → ✓
  3. Filter „Status: Kunde" zeigt 0 (keine Kunden bisher) → ✓
  4. Suche „edeka" filtert → ✓
  5. Klick auf eine Firma → Sheet schiebt von rechts rein → ✓
  6. Status auf „angefragt" setzen → Badge wechselt + Activity-Log bekommt Eintrag → ✓
  7. Wiedervorlage auf morgen setzen → erscheint als Datum → X-Button leert sie → ✓
  8. Ansprechpartner „Frau Müller" eintragen + Tab → wird gespeichert → ✓
  9. Notiz erfassen „Test" → Timeline zeigt sie sofort → ✓
  10. Manuelle Firma anlegen (mit beliebigen Koordinaten) → erscheint in Liste → ✓
  11. Sidebar → „Neue Suche" → Test-Suche-Button funktioniert weiter → Liste refresht nach Done → ✓
  12. Theme-Toggle → Light/Dark wechselt → ✓
  13. „Karte"/„Profile"/„Einstellungen" → NotImplemented-Stub sichtbar → ✓
  14. Firma über „Löschen" → Bestätigung → wird entfernt → Activity verschwindet (CASCADE) → ✓

Falls irgendein Punkt scheitert: dokumentieren, im nächsten Iterations-Loop fixen.

- [ ] **Step 2: Logs prüfen**

Log-Pfad finden (dirs::data_dir() liefert auf Mac `~/Library/Application Support/`, dann `projektalpha/logs/projektalpha.log.<datum>`):
```bash
ls ~/Library/Application\ Support/projektalpha/logs/
tail -50 ~/Library/Application\ Support/projektalpha/logs/projektalpha.log.*
```
Erwartet: JSON-Zeilen mit `status changed`, `company deleted`, evtl. `frontend_log`-Einträge. KEINE PII (keine Telefonnummern, keine Mail-Adressen, keine Notizinhalte) in den Logs.

- [ ] **Step 3: Checkpoint**

> **Checkpoint 2.18 = Phase 2 fertig.** Die App ist ab jetzt **produktiv für deinen Vater nutzbar**: Liste, Filter, Detail, Status, Notizen, Wiedervorlage, manuelles Hinzufügen — alles über Klick erreichbar, Daten persistent in lokaler SQLite.

---

## Was am Ende dieses Plans funktioniert

- ✅ Layout: Sidebar (5 Views), TopBar (Theme + Backup-Stub), Hauptbereich
- ✅ Firmen-Liste mit Cards, gefiltert nach Status/Branche/Score/Suche, sortiert nach Score+Name
- ✅ Detail-Sheet mit Adresse, Anrufen/Mail/Website-Buttons, Status-Dropdown, Wiedervorlage, Ansprechpartner, Activity-Timeline, Lösch-Button
- ✅ Status-Änderungen schreiben automatisch ins Activity-Log + setzen `last_contact_at`
- ✅ Manuelles Hinzufügen einer Firma (Koordinaten manuell)
- ✅ Frontend-Logger forwardet ins Rust-Log
- ✅ Error-Boundary fängt React-Crashes + schreibt Crash-File
- ✅ Tauri-Panic-Handler schreibt Crash-File
- ✅ Vitest + React Testing Library aufgesetzt mit ersten Tests
- ✅ Backend-Test-Suite gewachsen auf ~43 Tests
- ✅ Frontend-Test-Suite mit 7 Tests (5 format + 2 FilterBar)

## Was bewusst NICHT in diesem Plan ist

- **Karte/MapLibre** → Plan 3
- **Adress-Geocoding mit Nominatim** → Plan 4 (UI für „Neue Suche" mit Adress-Eingabe)
- **Such-Profile + Settings-UI + Branchen-Editor + Backup/Restore-UI** → Plan 4
- **Dashboard mit KPIs + Today-Liste** → Plan 5
- **Toast-Notifications** (alerts/confirms reichen vorerst) → später Polish
- **Auto-Updater + GitHub Actions Cross-Build (Windows .msi)** → Plan 6
- **Code-Signing** → bewusst weggelassen (Spec §12.3)

---

## Nächster Plan

**Plan 3 — Phase 3 (Karte mit MapLibre + Pin-Sync mit Liste + Karten-Mittelpunkt-Picker für Suche).**
