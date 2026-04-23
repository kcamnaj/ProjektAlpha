# Phase 5: Dashboard – KPIs, Heute-fällig, Aktivitäts-Timeline — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Die App bekommt eine Dashboard-Startseite mit (1) vier KPI-Karten (Kunden / Angefragt / Neu / Ø Score), (2) einer prominenten „Heute fällig"-Liste für Wiedervorlagen inkl. unübersehbarer Banner-Darstellung bei offenen Fällen, und (3) einer „Letzte Aktivität"-Timeline der letzten 20 Einträge aus dem Activity-Log quer über alle Firmen. Dashboard wird zur neuen Default-View beim App-Start — der Vater sieht beim Öffnen sofort, was heute zu tun ist.

**Architecture:** Backend bekommt drei reine Read-Queries: KPI-Aggregat (4 COUNT/AVG-Werte in einer Abfrage), Due-Followups (erweitert `db/companies.rs` um `list_due_followups`, liefert bestehende `CompanyRow`), und eine cross-company Activity-Liste (neues Modul `db/dashboard.rs` mit JOIN auf companies.name). Drei neue Tauri-Commands als dünne Wrapper. Frontend bekommt eine neue `DashboardPage` mit drei Sub-Komponenten (KpiCards / DueFollowupsList / ActivityTimeline), die beim Mount parallel laden und Klicks auf eine Firma in das bestehende Detail-Sheet delegieren (`setView("companies")` + `selectCompany(id)`). Die Startup-„Toast"-Anforderung aus §6.4 der Spec wird bewusst als **prominente In-Page-Banner-Darstellung** gelöst (keine neue `sonner`-Dependency) — das passt zur UX-Regel „Erinnert statt versteckt" und bleibt YAGNI.

**Tech Stack:** Rust (sqlx mit `COUNT(CASE WHEN…)`-Aggregat, SQLite `DATE()` für Tages-Vergleich, JOIN für Cross-Company-Activity). React (shadcn Card, lucide-react Icons, bestehende `formatDateDe`/`statusLabel`-Helfer). Keine neuen NPM-Pakete, keine neuen Rust-Crates, keine neue Migration.

**Spec-Referenz:** `docs/superpowers/specs/2026-04-21-firmensuche-design.md`
- §6.4 (Wiedervorlage-Prüfung beim App-Start) · §7.2 Screen 1 (Dashboard) · §10a (Logging ohne PII) · §14 (Phase 5: „Dashboard + Wiedervorlage")

**CLAUDE.md-Prinzipien:**
- **UX zuerst:** Dashboard ist neue Default-Landing-Page. Fälligkeits-Banner erscheint rot/auffällig oben, wenn `count > 0`, sonst komplett ausgeblendet — keine leere „0 Fälligkeiten"-Optik. Klick auf Firma in Heute-fällig-Liste oder Timeline öffnet sofort das bestehende Detail-Sheet (ein Klick = Detail).
- **Nicht kompliziert:** Kein Date-Picker, keine Filter, keine KPI-Zeiträume — Spec will Momentaufnahme, nicht Vergleich. Keine Toaster-Lib; Banner reicht. Eine Aggregate-Query für alle 4 KPIs (nicht vier einzelne Roundtrips).
- **Tests + Logs:** Alle neuen DB-Funktionen sind TDD (in-memory SQLite, pro Funktion 3-5 Tests für Happy Path + Edge Cases). Frontend: pure Helper-Funktion `formatRelativeFollowup` bekommt Unit-Tests; Komponenten-Render-Test für KpiCards. Dashboard-Queries loggen nur `count`- und `limit`-Integer — **niemals** Firmennamen, Content-Snippets oder IDs in INFO-Logs (nur `debug!`-Level für IDs, falls nötig).

**Kein Git** – Checkpoints statt Commits.

---

## Datei-Struktur (Ziel nach diesem Plan)

```
src-tauri/src/
├─ db/
│  ├─ companies.rs                       # MODIFY: +list_due_followups + Tests
│  ├─ dashboard.rs                       # NEU: DashboardKpis + fetch_kpis + RecentActivityRow + list_recent_activity
│  └─ mod.rs                             # MODIFY: pub mod dashboard;
├─ commands.rs                           # MODIFY: +3 neue Commands (dashboard_kpis, list_due_followups, list_recent_activity)
└─ lib.rs                                # MODIFY: 3 Commands registrieren

src/
├─ lib/
│  ├─ tauri.ts                           # MODIFY: +3 API-Funktionen + Types (DashboardKpis, RecentActivityRow)
│  ├─ format.ts                          # MODIFY: +formatRelativeFollowup
│  └─ format.test.ts                     # MODIFY: +4 Tests für formatRelativeFollowup
├─ stores/
│  └─ uiStore.ts                         # MODIFY: View += "dashboard", currentView default = "dashboard"
├─ components/
│  ├─ layout/
│  │  └─ Sidebar.tsx                     # MODIFY: +Dashboard-Item ganz oben
│  └─ dashboard/
│     ├─ KpiCards.tsx                    # NEU: 4 shadcn-Karten (Kunden/Angefragt/Neu/Ø Score)
│     ├─ KpiCards.test.tsx               # NEU: Render-Test
│     ├─ DueFollowupsList.tsx            # NEU: Banner + Liste mit Klick → Detail
│     └─ ActivityTimeline.tsx            # NEU: Gruppe-nach-Tag + Icon pro Typ
├─ hooks/
│  └─ useDashboardData.ts                # NEU: Parallel-Load (kpis, followups, activity)
├─ pages/
│  └─ DashboardPage.tsx                  # NEU: Layout-Shell mit drei Sektionen
└─ App.tsx                               # MODIFY: view === "dashboard" → <DashboardPage />
```

Keine neue Migration — alle benötigten Tabellen (`companies`, `activity_log`) existieren seit Plan 1.

---

# PHASE 5 — Dashboard

## Backend

---

### Task 5.1: `db/dashboard.rs` — KPI-Aggregat

**Files:**
- Create: `src-tauri/src/db/dashboard.rs`
- Modify: `src-tauri/src/db/mod.rs`

- [ ] **Step 1: Modul registrieren**

```rust
// src-tauri/src/db/mod.rs — neue Zeile in alphabetischer Reihenfolge
pub mod dashboard;
```

Erwartet nach dem Hinzufügen: Build bricht (`dashboard.rs` existiert noch nicht). Das ist gewollt — wir legen die Datei in Step 2 an.

- [ ] **Step 2: `fetch_kpis` + Struct anlegen (leeres Stub, damit Tests compilen)**

```rust
// src-tauri/src/db/dashboard.rs
use crate::error::AppResult;
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DashboardKpis {
    pub customers: i64,         // status = 'kunde'
    pub requested: i64,         // status = 'angefragt'
    pub new_count: i64,         // status = 'neu'
    pub avg_score: f64,         // AVG(probability_score) WHERE status != 'kein_kunde'; 0.0 wenn leer
    pub total_active: i64,      // alle außer 'kein_kunde' — für Avg-Denom-Anzeige
}

pub async fn fetch_kpis(_pool: &SqlitePool) -> AppResult<DashboardKpis> {
    todo!("implement in Step 4")
}
```

- [ ] **Step 3: Failing Tests schreiben**

**Hinweis:** `NewCompany` hat kein `status`-Feld (`insert_or_merge` schreibt immer `'neu'`). Der `seed()`-Helfer setzt den Status daher per `UPDATE` nach dem Insert.

```rust
// am Ende von src-tauri/src/db/dashboard.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{companies::{insert_or_merge, NewCompany}, open_in_memory};

    async fn seed(pool: &sqlx::SqlitePool, osm: &str, status: &str, score: i64) -> String {
        let c = NewCompany {
            osm_id: Some(osm.into()),
            name: format!("Firma {osm}"),
            street: None, postal_code: None, city: None, country: "DE".into(),
            lat: 0.0, lng: 0.0,
            phone: None, email: None, website: None,
            industry_category_id: Some(1),
            size_estimate: None,
            probability_score: score,
            source: "osm".into(),
        };
        insert_or_merge(pool, &c).await.unwrap();
        let row: (String,) = sqlx::query_as("SELECT id FROM companies WHERE osm_id = ?")
            .bind(osm).fetch_one(pool).await.unwrap();
        // status direkt setzen (insert_or_merge schreibt immer 'neu')
        sqlx::query("UPDATE companies SET status = ? WHERE id = ?")
            .bind(status).bind(&row.0).execute(pool).await.unwrap();
        row.0
    }

    #[tokio::test]
    async fn fetch_kpis_on_empty_db_returns_zeros() {
        let pool = open_in_memory().await;
        let k = fetch_kpis(&pool).await.unwrap();
        assert_eq!(k, DashboardKpis {
            customers: 0, requested: 0, new_count: 0, avg_score: 0.0, total_active: 0,
        });
    }

    #[tokio::test]
    async fn fetch_kpis_counts_by_status() {
        let pool = open_in_memory().await;
        seed(&pool, "node/1", "kunde", 80).await;
        seed(&pool, "node/2", "kunde", 90).await;
        seed(&pool, "node/3", "angefragt", 60).await;
        seed(&pool, "node/4", "neu", 40).await;
        seed(&pool, "node/5", "neu", 50).await;
        seed(&pool, "node/6", "neu", 70).await;
        let k = fetch_kpis(&pool).await.unwrap();
        assert_eq!(k.customers, 2);
        assert_eq!(k.requested, 1);
        assert_eq!(k.new_count, 3);
    }

    #[tokio::test]
    async fn fetch_kpis_avg_excludes_kein_kunde() {
        let pool = open_in_memory().await;
        seed(&pool, "node/1", "kunde", 80).await;        // zählt
        seed(&pool, "node/2", "neu", 40).await;          // zählt
        seed(&pool, "node/3", "kein_kunde", 10).await;   // NICHT zählen
        let k = fetch_kpis(&pool).await.unwrap();
        assert_eq!(k.total_active, 2);
        assert!((k.avg_score - 60.0).abs() < 0.001, "avg was {}", k.avg_score);
    }
}
```

- [ ] **Step 4: Tests ausführen, FAIL erwarten**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib dashboard::tests 2>&1 | tail -30
```

Erwartet: `panicked at 'not yet implemented'` aus dem `todo!()`.

- [ ] **Step 5: `fetch_kpis` implementieren**

```rust
pub async fn fetch_kpis(pool: &SqlitePool) -> AppResult<DashboardKpis> {
    // Eine Aggregat-Query, alle KPIs auf einen Schlag.
    // COALESCE(AVG(...), 0.0) fängt leeren Fall → NULL → wir wollen 0.0
    let row: (i64, i64, i64, f64, i64) = sqlx::query_as(
        "SELECT
            COALESCE(SUM(CASE WHEN status = 'kunde' THEN 1 ELSE 0 END), 0) AS customers,
            COALESCE(SUM(CASE WHEN status = 'angefragt' THEN 1 ELSE 0 END), 0) AS requested,
            COALESCE(SUM(CASE WHEN status = 'neu' THEN 1 ELSE 0 END), 0) AS new_count,
            COALESCE(AVG(CASE WHEN status != 'kein_kunde' THEN probability_score END), 0.0) AS avg_score,
            COALESCE(SUM(CASE WHEN status != 'kein_kunde' THEN 1 ELSE 0 END), 0) AS total_active
        FROM companies"
    ).fetch_one(pool).await?;
    Ok(DashboardKpis {
        customers: row.0,
        requested: row.1,
        new_count: row.2,
        avg_score: row.3,
        total_active: row.4,
    })
}
```

- [ ] **Step 6: Tests ausführen, GREEN erwarten**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib dashboard::tests 2>&1 | tail -15
```

Erwartet: `test result: ok. 3 passed`.

- [ ] **Step 7: `cargo fmt` + Full-Build**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo fmt && cargo build --lib 2>&1 | tail -5
```

Erwartet: `Finished …` ohne Errors.

- [ ] **Step 8: Checkpoint 5.1**

---

### Task 5.2: `list_due_followups` in `db/companies.rs`

**Files:**
- Modify: `src-tauri/src/db/companies.rs` (neue Funktion + Tests direkt unter bestehenden)

- [ ] **Step 1: Failing Tests schreiben**

Vor die `#[cfg(test)] mod tests`-Klammer den ersten Fall einfügen (oder unter den bestehenden `sample()`-Helfer). Alle vier Tests werden in demselben `mod tests`-Block ergänzt:

```rust
// in src-tauri/src/db/companies.rs, INNERHALB #[cfg(test)] mod tests { ... }

#[tokio::test]
async fn list_due_followups_returns_today_and_overdue() {
    let pool = open_in_memory().await;
    insert_or_merge(&pool, &sample(Some("node/1"), "A")).await.unwrap();
    insert_or_merge(&pool, &sample(Some("node/2"), "B")).await.unwrap();
    insert_or_merge(&pool, &sample(Some("node/3"), "C")).await.unwrap();

    let ids: Vec<String> = sqlx::query_as::<_, (String,)>("SELECT id FROM companies ORDER BY name")
        .fetch_all(&pool).await.unwrap().into_iter().map(|(s,)| s).collect();

    // A: heute fällig, B: überfällig (gestern), C: morgen (noch nicht)
    let today = chrono::Utc::now().format("%Y-%m-%dT12:00:00Z").to_string();
    let yesterday = (chrono::Utc::now() - chrono::Duration::days(1)).format("%Y-%m-%dT12:00:00Z").to_string();
    let tomorrow = (chrono::Utc::now() + chrono::Duration::days(1)).format("%Y-%m-%dT12:00:00Z").to_string();
    update_followup(&pool, &ids[0], Some(&today)).await.unwrap();
    update_followup(&pool, &ids[1], Some(&yesterday)).await.unwrap();
    update_followup(&pool, &ids[2], Some(&tomorrow)).await.unwrap();

    let due = list_due_followups(&pool).await.unwrap();
    let due_names: Vec<&str> = due.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(due_names, vec!["B", "A"], "überfällig zuerst, dann heute");
    assert_eq!(due.len(), 2);
}

#[tokio::test]
async fn list_due_followups_excludes_kein_kunde() {
    let pool = open_in_memory().await;
    insert_or_merge(&pool, &sample(Some("node/1"), "X")).await.unwrap();
    let id: (String,) = sqlx::query_as("SELECT id FROM companies WHERE osm_id = 'node/1'")
        .fetch_one(&pool).await.unwrap();
    update_status(&pool, &id.0, "kein_kunde").await.unwrap();
    let today = chrono::Utc::now().format("%Y-%m-%dT12:00:00Z").to_string();
    update_followup(&pool, &id.0, Some(&today)).await.unwrap();

    let due = list_due_followups(&pool).await.unwrap();
    assert!(due.is_empty(), "kein_kunde soll ausgeschlossen sein");
}

#[tokio::test]
async fn list_due_followups_excludes_null_followup() {
    let pool = open_in_memory().await;
    insert_or_merge(&pool, &sample(Some("node/1"), "X")).await.unwrap();
    // kein update_followup → next_followup_at ist NULL
    let due = list_due_followups(&pool).await.unwrap();
    assert!(due.is_empty());
}

#[tokio::test]
async fn list_due_followups_empty_db() {
    let pool = open_in_memory().await;
    let due = list_due_followups(&pool).await.unwrap();
    assert!(due.is_empty());
}
```

- [ ] **Step 2: Tests ausführen, FAIL erwarten**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib companies::tests::list_due 2>&1 | tail -20
```

Erwartet: Compile-Error `cannot find function list_due_followups in this scope`.

- [ ] **Step 3: Implementierung hinzufügen**

Direkt unter `pub async fn delete(…)` in `src-tauri/src/db/companies.rs`:

```rust
/// Fällige Wiedervorlagen: `next_followup_at IS NOT NULL` UND `DATE(next_followup_at) <= DATE('now')`
/// UND Status ist nicht `kein_kunde` (abgeschlossene Nicht-Kunden werden übersprungen).
/// Sortiert aufsteigend nach `next_followup_at` — überfällige zuerst, dann heutige.
pub async fn list_due_followups(pool: &SqlitePool) -> AppResult<Vec<CompanyRow>> {
    let sql = format!(
        "{COMPANY_SELECT}
         WHERE c.next_followup_at IS NOT NULL
           AND DATE(c.next_followup_at) <= DATE('now')
           AND c.status != 'kein_kunde'
         ORDER BY c.next_followup_at ASC"
    );
    let rows = sqlx::query_as::<_, CompanyRow>(&sql)
        .fetch_all(pool).await?;
    Ok(rows)
}
```

Hinweis: `COMPANY_SELECT` ist der bestehende SELECT-Prefix mit JOINs (existiert schon in der Datei um Zeile ~120-180).

- [ ] **Step 4: Tests ausführen, GREEN erwarten**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib companies::tests::list_due 2>&1 | tail -20
```

Erwartet: `4 passed`.

- [ ] **Step 5: Komplette companies-Test-Suite läuft**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib companies 2>&1 | tail -10
```

Erwartet: alle bisherigen companies-Tests + 4 neue → green.

- [ ] **Step 6: Checkpoint 5.2**

---

### Task 5.3: `list_recent_activity` in `db/dashboard.rs`

**Files:**
- Modify: `src-tauri/src/db/dashboard.rs`

- [ ] **Step 1: Struct + Stub hinzufügen**

Unter `fetch_kpis` in `dashboard.rs`:

```rust
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RecentActivityRow {
    pub id: String,
    pub company_id: String,
    pub company_name: String,
    pub r#type: String,      // 'notiz'|'anruf'|'mail'|'besuch'|'status_änderung'
    pub content: String,
    pub created_at: String,  // RFC3339
}

pub async fn list_recent_activity(
    _pool: &SqlitePool,
    _limit: i64,
) -> AppResult<Vec<RecentActivityRow>> {
    todo!("implement in Step 3")
}
```

- [ ] **Step 2: Failing Tests schreiben**

In den bestehenden `mod tests`-Block in `dashboard.rs`:

```rust
#[tokio::test]
async fn list_recent_activity_empty_returns_empty() {
    let pool = open_in_memory().await;
    let list = list_recent_activity(&pool, 20).await.unwrap();
    assert!(list.is_empty());
}

#[tokio::test]
async fn list_recent_activity_orders_newest_first_and_respects_limit() {
    use crate::db::activity::{add, NewActivity};
    let pool = open_in_memory().await;
    let cid = seed(&pool, "node/1", "neu", 50).await;
    for i in 0..5 {
        add(&pool, &NewActivity {
            company_id: cid.clone(),
            r#type: "notiz".into(),
            content: format!("n{i}"),
        }).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    let list = list_recent_activity(&pool, 3).await.unwrap();
    assert_eq!(list.len(), 3);
    assert_eq!(list[0].content, "n4", "newest first");
    assert_eq!(list[2].content, "n2");
}

#[tokio::test]
async fn list_recent_activity_includes_company_name() {
    use crate::db::activity::{add, NewActivity};
    let pool = open_in_memory().await;
    let cid = seed(&pool, "node/1", "neu", 50).await;
    add(&pool, &NewActivity {
        company_id: cid.clone(), r#type: "anruf".into(), content: "Hallo".into(),
    }).await.unwrap();
    let list = list_recent_activity(&pool, 10).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].company_name, "Firma node/1");
    assert_eq!(list[0].r#type, "anruf");
}

#[tokio::test]
async fn list_recent_activity_spans_multiple_companies() {
    use crate::db::activity::{add, NewActivity};
    let pool = open_in_memory().await;
    let a = seed(&pool, "node/1", "neu", 50).await;
    let b = seed(&pool, "node/2", "neu", 50).await;
    add(&pool, &NewActivity { company_id: a, r#type: "notiz".into(), content: "A".into() }).await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    add(&pool, &NewActivity { company_id: b, r#type: "mail".into(), content: "B".into() }).await.unwrap();
    let list = list_recent_activity(&pool, 10).await.unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].content, "B");
    assert_eq!(list[1].content, "A");
}
```

- [ ] **Step 3: Tests ausführen, FAIL erwarten**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib dashboard::tests::list_recent 2>&1 | tail -20
```

Erwartet: `panicked at 'implement in Step 3'`.

- [ ] **Step 4: Implementierung**

```rust
pub async fn list_recent_activity(
    pool: &SqlitePool,
    limit: i64,
) -> AppResult<Vec<RecentActivityRow>> {
    let rows: Vec<(String, String, String, String, String, String)> = sqlx::query_as(
        "SELECT a.id, a.company_id, c.name, a.type, a.content, a.created_at
         FROM activity_log a
         JOIN companies c ON c.id = a.company_id
         ORDER BY a.created_at DESC
         LIMIT ?"
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, company_id, company_name, r#type, content, created_at)| RecentActivityRow {
            id, company_id, company_name, r#type, content, created_at,
        })
        .collect())
}
```

- [ ] **Step 5: Tests GREEN**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib dashboard 2>&1 | tail -15
```

Erwartet: `7 passed` (3 KPI + 4 recent_activity).

- [ ] **Step 6: Full-Suite läuft**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib 2>&1 | tail -5
```

Erwartet: 71 bestehend + 4 companies + 7 dashboard = **82 passed**.

- [ ] **Step 7: Checkpoint 5.3**

---

### Task 5.4: Drei Tauri-Commands + Registrierung

**Files:**
- Modify: `src-tauri/src/commands.rs` (drei neue `#[tauri::command]`-Blöcke am Ende anfügen)
- Modify: `src-tauri/src/lib.rs` (drei Einträge im `invoke_handler![...]` anfügen)

- [ ] **Step 1: Commands anfügen**

Am Ende von `src-tauri/src/commands.rs`:

```rust
// Dashboard commands
use crate::db::dashboard::{self, DashboardKpis, RecentActivityRow};
use crate::db::companies as companies_db;

#[tauri::command]
pub async fn dashboard_kpis(state: State<'_, Arc<AppState>>) -> AppResult<DashboardKpis> {
    let k = dashboard::fetch_kpis(&state.db).await?;
    tracing::info!(
        customers = k.customers,
        requested = k.requested,
        new_count = k.new_count,
        total_active = k.total_active,
        "dashboard kpis fetched"
    );
    Ok(k)
}

#[tauri::command]
pub async fn list_due_followups(
    state: State<'_, Arc<AppState>>,
) -> AppResult<Vec<companies_db::CompanyRow>> {
    let list = companies_db::list_due_followups(&state.db).await?;
    tracing::info!(count = list.len(), "due followups listed");
    Ok(list)
}

#[tauri::command]
pub async fn list_recent_activity(
    state: State<'_, Arc<AppState>>,
    limit: Option<i64>,
) -> AppResult<Vec<RecentActivityRow>> {
    let lim = limit.unwrap_or(20).clamp(1, 100);
    let list = dashboard::list_recent_activity(&state.db, lim).await?;
    tracing::info!(limit = lim, count = list.len(), "recent activity listed");
    Ok(list)
}
```

Wichtig — **PII-Check**: Wir loggen nur Counts / Limits, **keine** Firmennamen, **keine** Activity-Contents, **keine** IDs. Folgt §10a der Spec.

- [ ] **Step 2: Commands in `lib.rs` registrieren**

In `src-tauri/src/lib.rs` im `invoke_handler![...]`-Block (aktuell endet bei `commands::app_version`), vor dem schließenden `]` anfügen:

```rust
            commands::app_version,
            commands::dashboard_kpis,
            commands::list_due_followups,
            commands::list_recent_activity,
        ])
```

- [ ] **Step 3: Build**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo build --lib 2>&1 | tail -5
```

Erwartet: `Finished` ohne Errors.

- [ ] **Step 4: Full-Test-Suite**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib 2>&1 | tail -5
```

Erwartet: **82 passed** (keine Regression).

- [ ] **Step 5: Checkpoint 5.4**

---

## Frontend

---

### Task 5.5: API + Types in `src/lib/tauri.ts`

**Files:**
- Modify: `src/lib/tauri.ts`

- [ ] **Step 1: Typen oberhalb von `export const api`**

```typescript
// src/lib/tauri.ts — direkt nach CompanyRow / ListFilter
export type DashboardKpis = {
  customers: number
  requested: number
  new_count: number
  avg_score: number
  total_active: number
}

export type RecentActivityRow = {
  id: string
  company_id: string
  company_name: string
  type: string         // 'notiz'|'anruf'|'mail'|'besuch'|'status_änderung'
  content: string
  created_at: string   // RFC3339
}
```

- [ ] **Step 2: API-Methoden ans Ende von `api` anfügen**

Im `export const api = { … }`, hinter `appVersion: …`, anfügen:

```typescript
  dashboardKpis: () => invoke<DashboardKpis>("dashboard_kpis"),
  listDueFollowups: () => invoke<CompanyRow[]>("list_due_followups"),
  listRecentActivity: (limit?: number) =>
    invoke<RecentActivityRow[]>("list_recent_activity", { limit: limit ?? 20 }),
```

- [ ] **Step 3: TypeScript-Check**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm tsc --noEmit 2>&1 | tail -10
```

Erwartet: keine Errors.

- [ ] **Step 4: Checkpoint 5.5**

---

### Task 5.6: Routing — View-Type, Sidebar, App-Switch

**Files:**
- Modify: `src/stores/uiStore.ts`
- Modify: `src/components/layout/Sidebar.tsx`
- Modify: `src/App.tsx`

- [ ] **Step 1: `View`-Typ erweitern + Default umstellen**

`src/stores/uiStore.ts`:

```typescript
// Ersetze die View-Definition und den create-Aufruf
export type View = "dashboard" | "companies" | "search" | "map" | "profiles" | "settings"

interface UiState {
  currentView: View
  selectedCompanyId: string | null
  setView: (v: View) => void
  selectCompany: (id: string | null) => void
}

export const useUiStore = create<UiState>((set) => ({
  currentView: "dashboard",   // <-- NEU: Dashboard ist Landing
  selectedCompanyId: null,
  setView: (v) => set({ currentView: v }),
  selectCompany: (id) => set({ selectedCompanyId: id }),
}))
```

- [ ] **Step 2: Sidebar-Item ganz oben**

`src/components/layout/Sidebar.tsx`, `items`-Array aktualisieren:

```tsx
import { useUiStore, type View } from "@/stores/uiStore"
import { Building2, Map, FolderOpen, Settings, Search, LayoutDashboard } from "lucide-react"
import { cn } from "@/lib/utils"

const items: { key: View; label: string; Icon: typeof Building2 }[] = [
  { key: "dashboard", label: "Dashboard",    Icon: LayoutDashboard },  // <-- NEU, ganz oben
  { key: "companies", label: "Firmen",       Icon: Building2 },
  { key: "search",    label: "Neue Suche",   Icon: Search },
  { key: "map",       label: "Karte",        Icon: Map },
  { key: "profiles",  label: "Profile",      Icon: FolderOpen },
  { key: "settings",  label: "Einstellungen", Icon: Settings },
]
// Rest bleibt unverändert
```

- [ ] **Step 3: App.tsx-Branch**

`src/App.tsx`:

```tsx
import { AppLayout } from "@/components/layout/AppLayout"
import { useUiStore } from "@/stores/uiStore"
import { DashboardPage } from "@/pages/DashboardPage"
import { CompaniesPage } from "@/pages/CompaniesPage"
import { NewSearchPage } from "@/pages/NewSearchPage"
import { MapPage } from "@/pages/MapPage"
import { NotImplementedPage } from "@/pages/NotImplementedPage"
import { SettingsPage } from "@/pages/SettingsPage"

function App() {
  const view = useUiStore(s => s.currentView)
  return (
    <AppLayout>
      {view === "dashboard" && <DashboardPage />}
      {view === "companies" && <CompaniesPage />}
      {view === "search" && <NewSearchPage />}
      {view === "map" && <MapPage />}
      {view === "profiles" && <NotImplementedPage view={view} />}
      {view === "settings" && <SettingsPage />}
    </AppLayout>
  )
}
export default App
```

- [ ] **Step 4: Stub-DashboardPage anlegen, damit Build durchgeht**

`src/pages/DashboardPage.tsx`:

```tsx
export function DashboardPage() {
  return <div className="p-6 text-sm text-muted-foreground">Dashboard (wird in Task 5.11 gefüllt)</div>
}
```

- [ ] **Step 5: Build + TS-Check**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm tsc --noEmit 2>&1 | tail -5
```

Erwartet: keine Errors.

- [ ] **Step 6: Checkpoint 5.6**

---

### Task 5.7: Helper `formatRelativeFollowup` + Tests

**Files:**
- Modify: `src/lib/format.ts`
- Modify: `src/lib/format.test.ts`

- [ ] **Step 1: Failing Tests schreiben**

In `src/lib/format.test.ts` ganz ans Ende anfügen:

```typescript
import { formatRelativeFollowup } from "./format"

describe("formatRelativeFollowup", () => {
  it("returns 'heute fällig' for today", () => {
    const today = new Date(); today.setHours(12, 0, 0, 0)
    expect(formatRelativeFollowup(today.toISOString())).toBe("heute fällig")
  })
  it("returns 'überfällig (vor X Tagen)' for past dates", () => {
    const d = new Date(); d.setDate(d.getDate() - 3); d.setHours(12, 0, 0, 0)
    expect(formatRelativeFollowup(d.toISOString())).toBe("überfällig (vor 3 Tagen)")
  })
  it("returns 'überfällig (seit gestern)' for yesterday", () => {
    const d = new Date(); d.setDate(d.getDate() - 1); d.setHours(12, 0, 0, 0)
    expect(formatRelativeFollowup(d.toISOString())).toBe("überfällig (seit gestern)")
  })
  it("returns 'morgen' for future dates", () => {
    const d = new Date(); d.setDate(d.getDate() + 1); d.setHours(12, 0, 0, 0)
    expect(formatRelativeFollowup(d.toISOString())).toBe("in 1 Tag")
  })
  it("returns '—' for null", () => {
    expect(formatRelativeFollowup(null)).toBe("—")
  })
})
```

- [ ] **Step 2: Tests ausführen, FAIL erwarten**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test -- format 2>&1 | tail -15
```

Erwartet: 5 neue Tests failen (Import existiert nicht).

- [ ] **Step 3: `formatRelativeFollowup` in `format.ts` hinzufügen**

Ans Ende von `src/lib/format.ts` anfügen:

```typescript
/**
 * Erzeugt ein Label für next_followup_at bezogen auf heute.
 * Tag-basiert (nicht stundengenau) — DATE-Grenzen zählen.
 */
export function formatRelativeFollowup(iso?: string | null): string {
  if (!iso) return "—"
  const target = new Date(iso)
  if (isNaN(target.getTime())) return "—"
  // Tag-Grenzen vergleichen, keine Stunden
  const t = new Date(target.getFullYear(), target.getMonth(), target.getDate())
  const now = new Date()
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  const days = Math.round((t.getTime() - today.getTime()) / (1000 * 60 * 60 * 24))
  if (days === 0) return "heute fällig"
  if (days === -1) return "überfällig (seit gestern)"
  if (days < -1) return `überfällig (vor ${-days} Tagen)`
  if (days === 1) return "in 1 Tag"
  return `in ${days} Tagen`
}
```

- [ ] **Step 4: Tests ausführen, GREEN erwarten**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test -- format 2>&1 | tail -15
```

Erwartet: alle (bestehend + 5 neu) green.

- [ ] **Step 5: Checkpoint 5.7**

---

### Task 5.8: `KpiCards`-Komponente + Render-Test

**Files:**
- Create: `src/components/dashboard/KpiCards.tsx`
- Create: `src/components/dashboard/KpiCards.test.tsx`

- [ ] **Step 1: Failing Test schreiben**

`src/components/dashboard/KpiCards.test.tsx`:

```tsx
import { describe, expect, it } from "vitest"
import { render, screen } from "@testing-library/react"
import { KpiCards } from "./KpiCards"

describe("KpiCards", () => {
  it("renders all four KPI tiles with German labels", () => {
    render(<KpiCards kpis={{ customers: 7, requested: 12, new_count: 33, avg_score: 57.4, total_active: 52 }} />)
    expect(screen.getByText("Kunden")).toBeInTheDocument()
    expect(screen.getByText("Angefragt")).toBeInTheDocument()
    expect(screen.getByText("Neu")).toBeInTheDocument()
    expect(screen.getByText("Ø Score")).toBeInTheDocument()
    expect(screen.getByText("7")).toBeInTheDocument()
    expect(screen.getByText("12")).toBeInTheDocument()
    expect(screen.getByText("33")).toBeInTheDocument()
    expect(screen.getByText("57,4")).toBeInTheDocument() // deutsche Komma-Formatierung
  })

  it("renders '—' for avg_score when total_active is 0", () => {
    render(<KpiCards kpis={{ customers: 0, requested: 0, new_count: 0, avg_score: 0, total_active: 0 }} />)
    expect(screen.getByText("—")).toBeInTheDocument()
  })
})
```

- [ ] **Step 2: Test läuft rot**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test -- KpiCards 2>&1 | tail -15
```

Erwartet: Import-Fehler.

- [ ] **Step 3: Komponente schreiben**

`src/components/dashboard/KpiCards.tsx`:

```tsx
import { Card, CardContent, CardDescription, CardTitle } from "@/components/ui/card"
import { Users, PhoneCall, Sparkles, Gauge } from "lucide-react"
import type { DashboardKpis } from "@/lib/tauri"

interface KpiCardsProps {
  kpis: DashboardKpis
}

export function KpiCards({ kpis }: KpiCardsProps) {
  const avgDisplay =
    kpis.total_active === 0
      ? "—"
      : kpis.avg_score.toFixed(1).replace(".", ",")

  return (
    <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
      <KpiTile icon={<Users className="size-5" />} label="Kunden" value={String(kpis.customers)} />
      <KpiTile icon={<PhoneCall className="size-5" />} label="Angefragt" value={String(kpis.requested)} />
      <KpiTile icon={<Sparkles className="size-5" />} label="Neu" value={String(kpis.new_count)} />
      <KpiTile icon={<Gauge className="size-5" />} label="Ø Score" value={avgDisplay} hint={kpis.total_active > 0 ? `über ${kpis.total_active} Firmen` : "keine Daten"} />
    </div>
  )
}

interface KpiTileProps {
  icon: React.ReactNode
  label: string
  value: string
  hint?: string
}

function KpiTile({ icon, label, value, hint }: KpiTileProps) {
  return (
    <Card size="sm">
      <CardContent className="flex flex-col gap-1">
        <div className="flex items-center gap-2 text-muted-foreground">
          {icon}
          <CardTitle className="text-sm">{label}</CardTitle>
        </div>
        <div className="text-3xl font-semibold tracking-tight">{value}</div>
        {hint && <CardDescription className="text-xs">{hint}</CardDescription>}
      </CardContent>
    </Card>
  )
}
```

- [ ] **Step 4: Tests GREEN**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test -- KpiCards 2>&1 | tail -15
```

Erwartet: 2 passed.

- [ ] **Step 5: Vollständige Frontend-Tests**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test 2>&1 | tail -10
```

Erwartet: 24 bestehend + 5 `formatRelativeFollowup` + 2 KpiCards = **31 passed**.

- [ ] **Step 6: Checkpoint 5.8**

---

### Task 5.9: `DueFollowupsList`-Komponente

**Files:**
- Create: `src/components/dashboard/DueFollowupsList.tsx`

- [ ] **Step 1: Komponente schreiben**

```tsx
// src/components/dashboard/DueFollowupsList.tsx
import type { CompanyRow } from "@/lib/tauri"
import { useUiStore } from "@/stores/uiStore"
import { Card, CardContent } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { AlertTriangle, Phone, ArrowRight } from "lucide-react"
import { formatRelativeFollowup, statusLabel, statusColor } from "@/lib/format"
import { cn } from "@/lib/utils"

interface DueFollowupsListProps {
  rows: CompanyRow[]
}

export function DueFollowupsList({ rows }: DueFollowupsListProps) {
  const setView = useUiStore(s => s.setView)
  const selectCompany = useUiStore(s => s.selectCompany)

  const openCompany = (id: string) => {
    selectCompany(id)
    setView("companies")
  }

  if (rows.length === 0) {
    return (
      <Card size="sm">
        <CardContent>
          <div className="text-sm text-muted-foreground">Keine Wiedervorlagen heute fällig. 🎉</div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card className="ring-2 ring-destructive/60" size="sm">
      <CardContent className="flex flex-col gap-3">
        <div className="flex items-center gap-2 text-destructive">
          <AlertTriangle className="size-5" />
          <span className="font-semibold">
            {rows.length} {rows.length === 1 ? "Wiedervorlage" : "Wiedervorlagen"} fällig
          </span>
        </div>
        <ul className="flex flex-col divide-y">
          {rows.map((c) => (
            <li key={c.id}>
              <button
                onClick={() => openCompany(c.id)}
                className="w-full py-2 flex items-center gap-3 hover:bg-accent/50 rounded-md px-2 text-left"
              >
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="font-medium truncate">{c.name}</span>
                    <span className={cn("text-[10px] px-1.5 py-0.5 rounded", statusColor(c.status))}>
                      {statusLabel(c.status)}
                    </span>
                  </div>
                  <div className="text-xs text-muted-foreground flex items-center gap-2">
                    <span>{formatRelativeFollowup(c.next_followup_at)}</span>
                    {c.city && <span>· {c.city}</span>}
                    {c.phone && <span className="inline-flex items-center gap-1"><Phone className="size-3" />{c.phone}</span>}
                  </div>
                </div>
                <ArrowRight className="size-4 text-muted-foreground shrink-0" />
              </button>
            </li>
          ))}
        </ul>
      </CardContent>
    </Card>
  )
}
```

- [ ] **Step 2: TS-Check**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm tsc --noEmit 2>&1 | tail -5
```

Erwartet: keine Errors.

- [ ] **Step 3: Checkpoint 5.9**

---

### Task 5.10: `ActivityTimeline`-Komponente

**Files:**
- Create: `src/components/dashboard/ActivityTimeline.tsx`

- [ ] **Step 1: Komponente schreiben**

```tsx
// src/components/dashboard/ActivityTimeline.tsx
import type { RecentActivityRow } from "@/lib/tauri"
import { useUiStore } from "@/stores/uiStore"
import { Card, CardContent, CardTitle } from "@/components/ui/card"
import { StickyNote, Phone, Mail, Footprints, ArrowRightLeft } from "lucide-react"
import { formatDateDe } from "@/lib/format"

const ICON: Record<string, React.ReactNode> = {
  notiz: <StickyNote className="size-4" />,
  anruf: <Phone className="size-4" />,
  mail: <Mail className="size-4" />,
  besuch: <Footprints className="size-4" />,
  "status_änderung": <ArrowRightLeft className="size-4" />,
}

const TYPE_LABEL: Record<string, string> = {
  notiz: "Notiz",
  anruf: "Anruf",
  mail: "Mail",
  besuch: "Besuch",
  "status_änderung": "Status",
}

interface ActivityTimelineProps {
  rows: RecentActivityRow[]
}

export function ActivityTimeline({ rows }: ActivityTimelineProps) {
  const setView = useUiStore(s => s.setView)
  const selectCompany = useUiStore(s => s.selectCompany)

  const openCompany = (id: string) => {
    selectCompany(id)
    setView("companies")
  }

  // Gruppierung nach Tag (yyyy-MM-dd)
  const groups: Record<string, RecentActivityRow[]> = {}
  for (const r of rows) {
    const day = r.created_at.slice(0, 10)
    groups[day] ??= []
    groups[day].push(r)
  }
  const orderedDays = Object.keys(groups).sort().reverse()

  return (
    <Card size="sm">
      <CardContent className="flex flex-col gap-4">
        <CardTitle className="text-sm">Letzte Aktivität</CardTitle>
        {rows.length === 0 && (
          <div className="text-sm text-muted-foreground">Noch keine Aktivität erfasst.</div>
        )}
        {orderedDays.map((day) => (
          <div key={day} className="flex flex-col gap-1">
            <div className="text-xs font-medium text-muted-foreground">{formatDateDe(day)}</div>
            <ul className="flex flex-col">
              {groups[day].map((r) => (
                <li key={r.id}>
                  <button
                    onClick={() => openCompany(r.company_id)}
                    className="w-full py-1.5 flex items-start gap-2 hover:bg-accent/50 rounded-md px-2 text-left"
                  >
                    <div className="text-muted-foreground pt-0.5">{ICON[r.type] ?? <StickyNote className="size-4" />}</div>
                    <div className="flex-1 min-w-0">
                      <div className="text-sm">
                        <span className="font-medium">{r.company_name}</span>
                        <span className="text-muted-foreground"> · {TYPE_LABEL[r.type] ?? r.type}</span>
                      </div>
                      <div className="text-xs text-muted-foreground truncate">{r.content}</div>
                    </div>
                  </button>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </CardContent>
    </Card>
  )
}
```

- [ ] **Step 2: TS-Check**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm tsc --noEmit 2>&1 | tail -5
```

Erwartet: keine Errors.

- [ ] **Step 3: Checkpoint 5.10**

---

### Task 5.11: `useDashboardData`-Hook + `DashboardPage`-Assembly

**Files:**
- Create: `src/hooks/useDashboardData.ts`
- Modify: `src/pages/DashboardPage.tsx` (ersetzt den Stub aus 5.6)

- [ ] **Step 1: Hook schreiben**

`src/hooks/useDashboardData.ts`:

```typescript
import { useCallback, useEffect, useState } from "react"
import { api, type CompanyRow, type DashboardKpis, type RecentActivityRow } from "@/lib/tauri"
import { logger } from "@/lib/logger"

export interface DashboardData {
  kpis: DashboardKpis | null
  followups: CompanyRow[]
  activity: RecentActivityRow[]
  loading: boolean
  error: string | null
  refresh: () => Promise<void>
}

export function useDashboardData(): DashboardData {
  const [kpis, setKpis] = useState<DashboardKpis | null>(null)
  const [followups, setFollowups] = useState<CompanyRow[]>([])
  const [activity, setActivity] = useState<RecentActivityRow[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const refresh = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const [k, f, a] = await Promise.all([
        api.dashboardKpis(),
        api.listDueFollowups(),
        api.listRecentActivity(20),
      ])
      setKpis(k)
      setFollowups(f)
      setActivity(a)
      logger.info("dashboard loaded", {
        customers: k.customers, requested: k.requested, new_count: k.new_count,
        followup_count: f.length, activity_count: a.length,
      })
    } catch (e) {
      const msg = String(e)
      setError(msg)
      logger.error("dashboard load failed", { e: msg })
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => { refresh() }, [refresh])

  return { kpis, followups, activity, loading, error, refresh }
}
```

- [ ] **Step 2: `DashboardPage` assemblieren** (ersetzt Stub aus 5.6)

`src/pages/DashboardPage.tsx`:

```tsx
import { KpiCards } from "@/components/dashboard/KpiCards"
import { DueFollowupsList } from "@/components/dashboard/DueFollowupsList"
import { ActivityTimeline } from "@/components/dashboard/ActivityTimeline"
import { useDashboardData } from "@/hooks/useDashboardData"
import { Button } from "@/components/ui/button"
import { RefreshCw } from "lucide-react"

export function DashboardPage() {
  const { kpis, followups, activity, loading, error, refresh } = useDashboardData()

  return (
    <div className="h-full overflow-y-auto">
      <div className="max-w-5xl mx-auto p-6 flex flex-col gap-5">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-semibold">Dashboard</h2>
          <Button variant="ghost" size="sm" onClick={refresh} disabled={loading}>
            <RefreshCw className={loading ? "size-4 animate-spin" : "size-4"} />
            Aktualisieren
          </Button>
        </div>

        {error && (
          <div className="text-sm text-destructive bg-destructive/10 rounded-md p-3">
            Fehler beim Laden: {error}
          </div>
        )}

        {/* Banner-Slot: Heute fällig wird zuerst gerendert, damit fällige Fälle unübersehbar sind */}
        {kpis !== null && <DueFollowupsList rows={followups} />}

        {kpis !== null && <KpiCards kpis={kpis} />}

        {kpis !== null && <ActivityTimeline rows={activity} />}

        {loading && kpis === null && (
          <div className="text-sm text-muted-foreground">Lade…</div>
        )}
      </div>
    </div>
  )
}
```

- [ ] **Step 3: Build + TS-Check**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm tsc --noEmit 2>&1 | tail -5
```

Erwartet: keine Errors.

- [ ] **Step 4: Vite-Build**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm vite build 2>&1 | tail -10
```

Erwartet: `✓ built in …` ohne Errors.

- [ ] **Step 5: Komplette Test-Suite läuft**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test 2>&1 | tail -5
```

Erwartet: **31 passed** (unverändert seit Task 5.8).

- [ ] **Step 6: Checkpoint 5.11**

---

### Task 5.12: Acceptance / Smoke-Test-Liste

**Files:** keine — nur Verifikation.

- [ ] **Step 1: App starten**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm tauri dev
```

Parallel Log-Stream:

```bash
tail -F ~/Library/Application\ Support/projektalpha/logs/projektalpha.log.*
```

- [ ] **Step 2: Dashboard-Acceptance durchklicken**

**Sidebar / Routing (3 Punkte):**
1. App startet direkt auf Dashboard-View (Sidebar-Eintrag „Dashboard" aktiv) → ✓
2. Sidebar zeigt „Dashboard" ganz oben mit LayoutDashboard-Icon → ✓
3. Klick auf „Firmen" → zurück zu „Dashboard" → Zustand bleibt gleich, Daten werden nicht doppelt geladen (nur beim Mount) → ✓

**KPI-Karten (4 Punkte):**
4. Auf leerer DB: Kunden 0, Angefragt 0, Neu 0, Ø Score zeigt „—" mit „keine Daten"-Hinweis → ✓
5. Nach einer Suche mit Firmen-Import: „Neu"-Count stimmt mit `Firmen`-Liste überein → ✓
6. Status manuell auf „Kunde" setzen → „Aktualisieren" klicken → Kunden-Zahl +1, Neu-Zahl −1 → ✓
7. Ø-Score wird mit deutschem Komma formatiert (z. B. „57,4"), Hinweis zeigt „über X Firmen" → ✓

**Heute-fällig-Liste (6 Punkte):**
8. Ohne fällige Einträge: freundliche Kachel „Keine Wiedervorlagen heute fällig." → ✓
9. Im Detail-Sheet einer Firma Datum „heute" als Wiedervorlage setzen → Dashboard „Aktualisieren" → Firma erscheint in der Banner-Liste mit roter Outline → ✓
10. Label zeigt „heute fällig" → ✓
11. Wiedervorlage vor 2 Tagen setzen → Label „überfällig (vor 2 Tagen)", sortiert oberhalb von „heute"-Einträgen → ✓
12. Klick auf Firmen-Zeile → View wechselt zu „Firmen" → Detail-Sheet ist bereits offen mit dieser Firma → ✓
13. Status auf „Kein Kunde" setzen → Dashboard „Aktualisieren" → Firma verschwindet aus Fällig-Liste → ✓

**Aktivitäts-Timeline (5 Punkte):**
14. Initial leer: „Noch keine Aktivität erfasst." → ✓
15. In Firma A Notiz hinzufügen, in Firma B Anruf protokollieren, Status von C ändern → „Aktualisieren" → drei Einträge in Timeline → ✓
16. Gruppiert unter heutigem Datum (z. B. „23.04.2026"), neuester Eintrag oben → ✓
17. Icon pro Typ unterscheidbar (Notiz / Anruf / Mail / Besuch / Status) → ✓
18. Klick auf Eintrag → navigiert zu Firma im Detail-Sheet → ✓

**Logs (2 Punkte):**
19. Log enthält `dashboard kpis fetched customers=… requested=… new_count=… total_active=…`, `due followups listed count=…`, `recent activity listed limit=20 count=…` → ✓
20. Log enthält **keine** Firmennamen, **keine** Content-Snippets, **keine** IDs aus Dashboard-Pfad → ✓

**Regressions-Smoke (2 Punkte):**
21. „Firmen"-Ansicht öffnen → Liste + Karte + FilterBar funktionieren wie zuvor → ✓
22. „Neue Suche" + „Karte" + „Einstellungen" (alle 4 Tabs) funktionieren unverändert → ✓

- [ ] **Step 3: Checkpoint 5.12 = Plan 5 fertig**

---

## Was am Ende dieses Plans funktioniert

- ✅ Dashboard-View mit drei Sektionen (KPI-Karten, Heute-fällig-Liste, Aktivitäts-Timeline) als Landing-Page beim App-Start
- ✅ Vier KPIs aus einer einzigen SQL-Aggregat-Query (O(1) Roundtrips)
- ✅ Unübersehbare „Heute fällig"-Darstellung durch farbigen Ring + Banner-Kopfzeile (ersetzt Spec-§6.4-Toast durch eine In-Page-Lösung)
- ✅ Ein-Klick-Navigation von Dashboard-Zeile zu Firmen-Detail-Sheet
- ✅ Aktivitäts-Timeline gruppiert nach Tag, mit Typ-Icons
- ✅ „Aktualisieren"-Button für manuellen Re-Load
- ✅ Rust-Test-Suite gewachsen: 71 → 82 (3 KPI + 4 followup + 4 recent_activity)
- ✅ Frontend-Test-Suite gewachsen: 24 → 31 (5 `formatRelativeFollowup` + 2 KpiCards)
- ✅ Keine PII in Logs (nur Counts / Limits)
- ✅ Keine neue Migration, keine neuen NPM- oder Rust-Dependencies

## Was bewusst NICHT in diesem Plan ist

- **Sonner/Toaster-Lib + Push-Benachrichtigung beim App-Start** → ersetzt durch In-Page-Banner. Sollte ein echter OS-Toast später gewünscht werden, ist das ein kleiner Zusatz-Task in Plan 6 (Polish) — die Datenquelle (`listDueFollowups`) bleibt identisch.
- **Dashboard-Auto-Refresh / Polling** → YAGNI für Single-User-Desktop; manuelles „Aktualisieren" reicht.
- **KPI-Zeiträume / Trends** (Woche, Monat, Vergleich) → Spec will Snapshot.
- **Eigene Tauri-Event-Emission `followups-due`** (Spec §6.4 Punkt 3) → nicht nötig, weil Dashboard beim Mount lädt und Banner ausreichend auffällig ist. Falls später eine Notification gebraucht wird, kann der Event im `app_start`-Pfad emittiert werden — zusätzlicher Plan-6-Punkt.
- **Klick-auf-KPI-Kachel = gefilterte Firmen-Liste** → Nice-to-have; erfordert Anpassung am `filterStore`. Kann in Plan 6 oder später kommen.
- **E2E-Playwright-Test für Dashboard** → Plan-Übergreifender Smoke-Test bleibt Plan-6-Thema (gemeinsame Playwright-Suite).
- **Filter für Timeline (z. B. nur Anrufe)** → keine Spec-Anforderung, YAGNI.

---

## Nächster Plan

**Plan 6 — Release Polish**: Auto-Updater, GitHub Actions Cross-Build (Mac DMG + Windows MSI), `README.md` für End-User, offene Tech-Debt-Punkte aus der bisher gesammelten Liste (CenterPickerMap flyTo, native alert/confirm durch shadcn AlertDialog, `data_dir()`-Duplizierung, Sidebar-Item „Profile" konsolidieren), plus optional ein echter OS-Toast für Wiedervorlagen und E2E-Playwright-Suite.
