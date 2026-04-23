use crate::error::AppResult;
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DashboardKpis {
    pub customers: i64,    // status = 'kunde'
    pub requested: i64,    // status = 'angefragt'
    pub new_count: i64,    // status = 'neu'
    pub avg_score: f64,    // AVG(probability_score) WHERE status != 'kein_kunde'; 0.0 wenn leer
    pub total_active: i64, // alle außer 'kein_kunde' — für Avg-Denom-Anzeige
}

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

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RecentActivityRow {
    pub id: String,
    pub company_id: String,
    pub company_name: String,
    pub r#type: String, // 'notiz'|'anruf'|'mail'|'besuch'|'status_änderung'
    pub content: String,
    pub created_at: String, // RFC3339
}

pub async fn list_recent_activity(
    pool: &SqlitePool,
    limit: i64,
) -> AppResult<Vec<RecentActivityRow>> {
    let rows: Vec<(String, String, String, String, String, String)> = sqlx::query_as(
        "SELECT a.id, a.company_id, c.name, a.type, a.content, a.created_at
         FROM activity_log a
         JOIN companies c ON c.id = a.company_id
         ORDER BY a.created_at DESC
         LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(id, company_id, company_name, r#type, content, created_at)| RecentActivityRow {
                id,
                company_id,
                company_name,
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

    async fn seed(pool: &sqlx::SqlitePool, osm: &str, status: &str, score: i64) -> String {
        let c = NewCompany {
            osm_id: Some(osm.into()),
            name: format!("Firma {osm}"),
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
            probability_score: score,
            source: "osm".into(),
        };
        insert_or_merge(pool, &c).await.unwrap();
        let row: (String,) = sqlx::query_as("SELECT id FROM companies WHERE osm_id = ?")
            .bind(osm)
            .fetch_one(pool)
            .await
            .unwrap();
        // status direkt setzen (insert_or_merge schreibt immer 'neu')
        sqlx::query("UPDATE companies SET status = ? WHERE id = ?")
            .bind(status)
            .bind(&row.0)
            .execute(pool)
            .await
            .unwrap();
        row.0
    }

    #[tokio::test]
    async fn fetch_kpis_on_empty_db_returns_zeros() {
        let pool = open_in_memory().await;
        let k = fetch_kpis(&pool).await.unwrap();
        assert_eq!(
            k,
            DashboardKpis {
                customers: 0,
                requested: 0,
                new_count: 0,
                avg_score: 0.0,
                total_active: 0,
            }
        );
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
        seed(&pool, "node/1", "kunde", 80).await; // zählt
        seed(&pool, "node/2", "neu", 40).await; // zählt
        seed(&pool, "node/3", "kein_kunde", 10).await; // NICHT zählen
        let k = fetch_kpis(&pool).await.unwrap();
        assert_eq!(k.total_active, 2);
        assert!(
            (k.avg_score - 60.0).abs() < 0.001,
            "avg was {}",
            k.avg_score
        );
    }

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
            add(
                &pool,
                &NewActivity {
                    company_id: cid.clone(),
                    r#type: "notiz".into(),
                    content: format!("n{i}"),
                },
            )
            .await
            .unwrap();
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
        add(
            &pool,
            &NewActivity {
                company_id: cid.clone(),
                r#type: "anruf".into(),
                content: "Hallo".into(),
            },
        )
        .await
        .unwrap();
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
        add(
            &pool,
            &NewActivity {
                company_id: a,
                r#type: "notiz".into(),
                content: "A".into(),
            },
        )
        .await
        .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        add(
            &pool,
            &NewActivity {
                company_id: b,
                r#type: "mail".into(),
                content: "B".into(),
            },
        )
        .await
        .unwrap();
        let list = list_recent_activity(&pool, 10).await.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].content, "B");
        assert_eq!(list[1].content, "A");
    }
}
