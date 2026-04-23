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
        "INSERT INTO activity_log (id, company_id, type, content, created_at) VALUES (?,?,?,?,?)",
    )
    .bind(&id)
    .bind(&a.company_id)
    .bind(&a.r#type)
    .bind(&a.content)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(ActivityRow {
        id,
        company_id: a.company_id.clone(),
        r#type: a.r#type.clone(),
        content: a.content.clone(),
        created_at: now,
    })
}

pub async fn list_for_company(pool: &SqlitePool, company_id: &str) -> AppResult<Vec<ActivityRow>> {
    let rows: Vec<(String, String, String, String, String)> = sqlx::query_as(
        "SELECT id, company_id, type, content, created_at FROM activity_log WHERE company_id = ? ORDER BY created_at DESC"
    ).bind(company_id).fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(
            |(id, company_id, r#type, content, created_at)| ActivityRow {
                id,
                company_id,
                r#type,
                content,
                created_at,
            },
        )
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{
        companies::{insert_or_merge, NewCompany},
        open_in_memory,
    };

    async fn seed_company(pool: &SqlitePool) -> String {
        let c = NewCompany {
            osm_id: Some("node/1".into()),
            name: "X".into(),
            street: None,
            postal_code: None,
            city: None,
            country: "DE".into(),
            lat: 0.0,
            lng: 0.0,
            phone: None,
            email: None,
            website: None,
            industry_category_id: Some(1),
            size_estimate: None,
            probability_score: 50,
            source: "osm".into(),
        };
        insert_or_merge(pool, &c).await.unwrap();
        let row: (String,) = sqlx::query_as("SELECT id FROM companies WHERE osm_id = 'node/1'")
            .fetch_one(pool)
            .await
            .unwrap();
        row.0
    }

    #[tokio::test]
    async fn add_then_list_returns_entry() {
        let pool = open_in_memory().await;
        let cid = seed_company(&pool).await;
        let r = add(
            &pool,
            &NewActivity {
                company_id: cid.clone(),
                r#type: "notiz".into(),
                content: "test".into(),
            },
        )
        .await
        .unwrap();
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
            add(
                &pool,
                &NewActivity {
                    company_id: cid.clone(),
                    r#type: "notiz".into(),
                    content: format!("{i}"),
                },
            )
            .await
            .unwrap();
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
        add(
            &pool,
            &NewActivity {
                company_id: cid.clone(),
                r#type: "notiz".into(),
                content: "x".into(),
            },
        )
        .await
        .unwrap();
        sqlx::query("DELETE FROM companies WHERE id = ?")
            .bind(&cid)
            .execute(&pool)
            .await
            .unwrap();
        let list = list_for_company(&pool, &cid).await.unwrap();
        assert!(list.is_empty(), "FK CASCADE should delete activity");
    }
}
