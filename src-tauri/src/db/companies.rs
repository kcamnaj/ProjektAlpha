use crate::error::{AppError, AppResult};
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
pub struct InsertResult {
    pub inserted: bool,
    pub updated_fields: Vec<&'static str>,
}

pub async fn insert_or_merge(pool: &SqlitePool, c: &NewCompany) -> AppResult<InsertResult> {
    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    if let Some(osm_id) = &c.osm_id {
        let existing: Option<(String, String)> =
            sqlx::query_as("SELECT id, source FROM companies WHERE osm_id = ?")
                .bind(osm_id)
                .fetch_optional(pool)
                .await?;

        if let Some((existing_id, source)) = existing {
            if source == "manual" {
                // Manual entries are sacrosanct: never updated by OSM re-imports
                return Ok(InsertResult {
                    inserted: false,
                    updated_fields: vec![],
                });
            }
            let mut updated: Vec<&'static str> = vec![];
            macro_rules! maybe_update {
                ($field:literal, $val:expr) => {
                    if let Some(v) = &$val {
                        let was: (Option<String>,) = sqlx::query_as(&format!(
                            "SELECT {} FROM companies WHERE id = ?",
                            $field
                        ))
                        .bind(&existing_id)
                        .fetch_one(pool)
                        .await?;
                        if was.0.is_none() {
                            sqlx::query(&format!(
                                "UPDATE companies SET {} = ?, updated_at = ? WHERE id = ?",
                                $field
                            ))
                            .bind(v)
                            .bind(&now)
                            .bind(&existing_id)
                            .execute(pool)
                            .await?;
                            updated.push($field);
                        }
                    }
                };
            }
            maybe_update!("phone", c.phone);
            maybe_update!("email", c.email);
            maybe_update!("website", c.website);
            return Ok(InsertResult {
                inserted: false,
                updated_fields: updated,
            });
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

    Ok(InsertResult {
        inserted: true,
        updated_fields: vec![],
    })
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
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
    pub status: Option<String>,
    pub category_ids: Option<Vec<i64>>,
    pub min_score: Option<i64>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

const COMPANY_SELECT: &str =
    "SELECT c.id, c.name, c.street, c.postal_code, c.city, c.country, c.lat, c.lng,
            c.phone, c.email, c.website, c.industry_category_id,
            ic.name_de AS category_name, ic.color AS category_color,
            c.probability_score, c.status, c.contact_person,
            c.last_contact_at, c.next_followup_at, c.source, c.created_at, c.updated_at
     FROM companies c LEFT JOIN industry_categories ic ON ic.id = c.industry_category_id";

pub async fn list(pool: &SqlitePool, f: &ListFilter) -> AppResult<Vec<CompanyRow>> {
    let mut sql = format!("{COMPANY_SELECT} WHERE 1=1");
    let mut binds: Vec<String> = vec![];
    if let Some(s) = &f.status {
        sql.push_str(" AND c.status = ?");
        binds.push(s.clone());
    }
    if let Some(ids) = &f.category_ids {
        if !ids.is_empty() {
            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            sql.push_str(&format!(
                " AND c.industry_category_id IN ({})",
                placeholders
            ));
            for id in ids {
                binds.push(id.to_string());
            }
        }
    }
    if let Some(min) = f.min_score {
        sql.push_str(" AND c.probability_score >= ?");
        binds.push(min.to_string());
    }
    if let Some(q) = &f.search {
        if !q.trim().is_empty() {
            sql.push_str(" AND (LOWER(c.name) LIKE ? OR LOWER(IFNULL(c.city,'')) LIKE ?)");
            let pat = format!("%{}%", q.to_lowercase());
            binds.push(pat.clone());
            binds.push(pat);
        }
    }
    sql.push_str(" ORDER BY c.probability_score DESC, c.name ASC LIMIT ? OFFSET ?");
    binds.push(f.limit.unwrap_or(200).to_string());
    binds.push(f.offset.unwrap_or(0).to_string());

    let mut q = sqlx::query_as::<_, CompanyRow>(&sql);
    for b in &binds {
        q = q.bind(b);
    }
    let rows = q.fetch_all(pool).await?;
    Ok(rows)
}

pub async fn get(pool: &SqlitePool, id: &str) -> AppResult<Option<CompanyRow>> {
    let sql = format!("{COMPANY_SELECT} WHERE c.id = ?");
    let row = sqlx::query_as::<_, CompanyRow>(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn update_status(pool: &SqlitePool, id: &str, new_status: &str) -> AppResult<String> {
    if !["neu", "angefragt", "kunde", "kein_kunde"].contains(&new_status) {
        return Err(AppError::InvalidInput(format!(
            "invalid status: {new_status}"
        )));
    }
    let now = Utc::now().to_rfc3339();
    let prev: (String,) = sqlx::query_as("SELECT status FROM companies WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;
    sqlx::query(
        "UPDATE companies SET status = ?, last_contact_at = ?, updated_at = ? WHERE id = ?",
    )
    .bind(new_status)
    .bind(&now)
    .bind(&now)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(prev.0)
}

pub async fn update_followup(pool: &SqlitePool, id: &str, when: Option<&str>) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE companies SET next_followup_at = ?, updated_at = ? WHERE id = ?")
        .bind(when)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_contact_person(
    pool: &SqlitePool,
    id: &str,
    person: Option<&str>,
) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE companies SET contact_person = ?, updated_at = ? WHERE id = ?")
        .bind(person)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM companies WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

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
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    fn sample(osm_id: Option<&str>, name: &str) -> NewCompany {
        NewCompany {
            osm_id: osm_id.map(String::from),
            name: name.to_string(),
            street: None,
            postal_code: None,
            city: Some("Hannover".into()),
            country: "DE".into(),
            lat: 52.37,
            lng: 9.73,
            phone: None,
            email: None,
            website: None,
            industry_category_id: Some(1),
            size_estimate: None,
            probability_score: 95,
            source: "osm".into(),
        }
    }

    #[tokio::test]
    async fn first_insert_succeeds() {
        let pool = open_in_memory().await;
        let r = insert_or_merge(&pool, &sample(Some("node/123"), "Müller GmbH"))
            .await
            .unwrap();
        assert!(r.inserted);
    }

    #[tokio::test]
    async fn duplicate_osm_id_skipped() {
        let pool = open_in_memory().await;
        insert_or_merge(&pool, &sample(Some("node/1"), "A"))
            .await
            .unwrap();
        let r = insert_or_merge(&pool, &sample(Some("node/1"), "A"))
            .await
            .unwrap();
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

        let phone: (String,) =
            sqlx::query_as("SELECT phone FROM companies WHERE osm_id = 'node/9'")
                .fetch_one(&pool)
                .await
                .unwrap();
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

        // 3rd run with another phone must NOT overwrite
        let mut third = sample(Some("node/2"), "X");
        third.phone = Some("999".into());
        let r3 = insert_or_merge(&pool, &third).await.unwrap();
        assert!(r3.updated_fields.is_empty());
    }

    #[tokio::test]
    async fn list_filters_by_status() {
        let pool = open_in_memory().await;
        insert_or_merge(&pool, &sample(Some("a"), "A"))
            .await
            .unwrap();
        insert_or_merge(&pool, &sample(Some("b"), "B"))
            .await
            .unwrap();
        let id_b: (String,) = sqlx::query_as("SELECT id FROM companies WHERE osm_id='b'")
            .fetch_one(&pool)
            .await
            .unwrap();
        update_status(&pool, &id_b.0, "kunde").await.unwrap();

        let kunden = list(
            &pool,
            &ListFilter {
                status: Some("kunde".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(kunden.len(), 1);
        assert_eq!(kunden[0].name, "B");
    }

    #[tokio::test]
    async fn list_filters_by_search() {
        let pool = open_in_memory().await;
        let mut a = sample(Some("a"), "Müller GmbH");
        a.city = Some("Hannover".into());
        let mut b = sample(Some("b"), "Schmidt AG");
        b.city = Some("Bremen".into());
        insert_or_merge(&pool, &a).await.unwrap();
        insert_or_merge(&pool, &b).await.unwrap();
        let r = list(
            &pool,
            &ListFilter {
                search: Some("müller".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(r.len(), 1);
    }

    #[tokio::test]
    async fn list_filters_by_min_score() {
        let pool = open_in_memory().await;
        let mut a = sample(Some("a"), "A");
        a.probability_score = 30;
        let mut b = sample(Some("b"), "B");
        b.probability_score = 90;
        insert_or_merge(&pool, &a).await.unwrap();
        insert_or_merge(&pool, &b).await.unwrap();
        let r = list(
            &pool,
            &ListFilter {
                min_score: Some(50),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].name, "B");
    }

    #[tokio::test]
    async fn update_status_sets_last_contact_at() {
        let pool = open_in_memory().await;
        insert_or_merge(&pool, &sample(Some("a"), "A"))
            .await
            .unwrap();
        let id: (String,) = sqlx::query_as("SELECT id FROM companies WHERE osm_id='a'")
            .fetch_one(&pool)
            .await
            .unwrap();
        let prev = update_status(&pool, &id.0, "angefragt").await.unwrap();
        assert_eq!(prev, "neu");
        let row = get(&pool, &id.0).await.unwrap().unwrap();
        assert_eq!(row.status, "angefragt");
        assert!(row.last_contact_at.is_some());
    }

    #[tokio::test]
    async fn update_status_rejects_invalid_value() {
        let pool = open_in_memory().await;
        insert_or_merge(&pool, &sample(Some("a"), "A"))
            .await
            .unwrap();
        let id: (String,) = sqlx::query_as("SELECT id FROM companies WHERE osm_id='a'")
            .fetch_one(&pool)
            .await
            .unwrap();
        let r = update_status(&pool, &id.0, "garbage").await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn update_followup_sets_and_clears() {
        let pool = open_in_memory().await;
        insert_or_merge(&pool, &sample(Some("a"), "A"))
            .await
            .unwrap();
        let id: (String,) = sqlx::query_as("SELECT id FROM companies WHERE osm_id='a'")
            .fetch_one(&pool)
            .await
            .unwrap();
        update_followup(&pool, &id.0, Some("2026-05-01T09:00:00+00:00"))
            .await
            .unwrap();
        let row = get(&pool, &id.0).await.unwrap().unwrap();
        assert_eq!(
            row.next_followup_at.as_deref(),
            Some("2026-05-01T09:00:00+00:00")
        );
        update_followup(&pool, &id.0, None).await.unwrap();
        let row2 = get(&pool, &id.0).await.unwrap().unwrap();
        assert!(row2.next_followup_at.is_none());
    }

    #[tokio::test]
    async fn delete_removes_company() {
        let pool = open_in_memory().await;
        insert_or_merge(&pool, &sample(Some("a"), "A"))
            .await
            .unwrap();
        let id: (String,) = sqlx::query_as("SELECT id FROM companies WHERE osm_id='a'")
            .fetch_one(&pool)
            .await
            .unwrap();
        delete(&pool, &id.0).await.unwrap();
        assert!(get(&pool, &id.0).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn list_due_followups_returns_today_and_overdue() {
        let pool = open_in_memory().await;
        insert_or_merge(&pool, &sample(Some("node/1"), "A"))
            .await
            .unwrap();
        insert_or_merge(&pool, &sample(Some("node/2"), "B"))
            .await
            .unwrap();
        insert_or_merge(&pool, &sample(Some("node/3"), "C"))
            .await
            .unwrap();

        let ids: Vec<String> =
            sqlx::query_as::<_, (String,)>("SELECT id FROM companies ORDER BY name")
                .fetch_all(&pool)
                .await
                .unwrap()
                .into_iter()
                .map(|(s,)| s)
                .collect();

        // A: heute fällig, B: überfällig (gestern), C: morgen (noch nicht)
        let today = chrono::Utc::now().format("%Y-%m-%dT12:00:00Z").to_string();
        let yesterday = (chrono::Utc::now() - chrono::Duration::days(1))
            .format("%Y-%m-%dT12:00:00Z")
            .to_string();
        let tomorrow = (chrono::Utc::now() + chrono::Duration::days(1))
            .format("%Y-%m-%dT12:00:00Z")
            .to_string();
        update_followup(&pool, &ids[0], Some(&today)).await.unwrap();
        update_followup(&pool, &ids[1], Some(&yesterday))
            .await
            .unwrap();
        update_followup(&pool, &ids[2], Some(&tomorrow))
            .await
            .unwrap();

        let due = list_due_followups(&pool).await.unwrap();
        let due_names: Vec<&str> = due.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(due_names, vec!["B", "A"], "überfällig zuerst, dann heute");
        assert_eq!(due.len(), 2);
    }

    #[tokio::test]
    async fn list_due_followups_excludes_kein_kunde() {
        let pool = open_in_memory().await;
        insert_or_merge(&pool, &sample(Some("node/1"), "X"))
            .await
            .unwrap();
        let id: (String,) = sqlx::query_as("SELECT id FROM companies WHERE osm_id = 'node/1'")
            .fetch_one(&pool)
            .await
            .unwrap();
        update_status(&pool, &id.0, "kein_kunde").await.unwrap();
        let today = chrono::Utc::now().format("%Y-%m-%dT12:00:00Z").to_string();
        update_followup(&pool, &id.0, Some(&today)).await.unwrap();

        let due = list_due_followups(&pool).await.unwrap();
        assert!(due.is_empty(), "kein_kunde soll ausgeschlossen sein");
    }

    #[tokio::test]
    async fn list_due_followups_excludes_null_followup() {
        let pool = open_in_memory().await;
        insert_or_merge(&pool, &sample(Some("node/1"), "X"))
            .await
            .unwrap();
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
}
