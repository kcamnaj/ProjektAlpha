use crate::error::AppResult;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchProfile {
    pub id: i64,
    pub name: String,
    pub center_label: String,
    pub center_lat: f64,
    pub center_lng: f64,
    pub radius_km: i64,
    pub enabled_category_ids: String, // JSON-Array: "[1,2,3]"
    pub last_run_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewSearchProfile {
    pub name: String,
    pub center_label: String,
    pub center_lat: f64,
    pub center_lng: f64,
    pub radius_km: i64,
    pub enabled_category_ids: String,
}

pub async fn list_all(pool: &SqlitePool) -> AppResult<Vec<SearchProfile>> {
    let rows: Vec<(
        i64,
        String,
        String,
        f64,
        f64,
        i64,
        String,
        Option<String>,
        String,
    )> = sqlx::query_as(
        "SELECT id, name, center_label, center_lat, center_lng, radius_km,
                    enabled_category_ids, last_run_at, created_at
             FROM search_profiles ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, name, cl, lat, lng, rk, ids, lr, ca)| SearchProfile {
            id,
            name,
            center_label: cl,
            center_lat: lat,
            center_lng: lng,
            radius_km: rk,
            enabled_category_ids: ids,
            last_run_at: lr,
            created_at: ca,
        })
        .collect())
}

pub async fn get(pool: &SqlitePool, id: i64) -> AppResult<Option<SearchProfile>> {
    let row: Option<(
        i64,
        String,
        String,
        f64,
        f64,
        i64,
        String,
        Option<String>,
        String,
    )> = sqlx::query_as(
        "SELECT id, name, center_label, center_lat, center_lng, radius_km,
                    enabled_category_ids, last_run_at, created_at
             FROM search_profiles WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(
        row.map(|(id, name, cl, lat, lng, rk, ids, lr, ca)| SearchProfile {
            id,
            name,
            center_label: cl,
            center_lat: lat,
            center_lng: lng,
            radius_km: rk,
            enabled_category_ids: ids,
            last_run_at: lr,
            created_at: ca,
        }),
    )
}

pub async fn create(pool: &SqlitePool, p: &NewSearchProfile) -> AppResult<i64> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "INSERT INTO search_profiles
         (name, center_label, center_lat, center_lng, radius_km, enabled_category_ids, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&p.name)
    .bind(&p.center_label)
    .bind(p.center_lat)
    .bind(p.center_lng)
    .bind(p.radius_km)
    .bind(&p.enabled_category_ids)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn rename(pool: &SqlitePool, id: i64, new_name: &str) -> AppResult<()> {
    sqlx::query("UPDATE search_profiles SET name = ? WHERE id = ?")
        .bind(new_name)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM search_profiles WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn mark_run(pool: &SqlitePool, id: i64) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE search_profiles SET last_run_at = ? WHERE id = ?")
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    fn sample() -> NewSearchProfile {
        NewSearchProfile {
            name: "Hannover 25km".into(),
            center_label: "Hannover, Deutschland".into(),
            center_lat: 52.37,
            center_lng: 9.73,
            radius_km: 25,
            enabled_category_ids: "[1,2,3]".into(),
        }
    }

    #[tokio::test]
    async fn create_then_list_returns_one() {
        let pool = open_in_memory().await;
        let id = create(&pool, &sample()).await.unwrap();
        assert!(id > 0);
        let all = list_all(&pool).await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "Hannover 25km");
        assert_eq!(all[0].radius_km, 25);
    }

    #[tokio::test]
    async fn get_returns_specific_row() {
        let pool = open_in_memory().await;
        let id = create(&pool, &sample()).await.unwrap();
        let got = get(&pool, id).await.unwrap().unwrap();
        assert_eq!(got.enabled_category_ids, "[1,2,3]");
        assert!((got.center_lat - 52.37).abs() < 1e-6);
    }

    #[tokio::test]
    async fn get_missing_returns_none() {
        let pool = open_in_memory().await;
        assert!(get(&pool, 999).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn rename_changes_name_only() {
        let pool = open_in_memory().await;
        let id = create(&pool, &sample()).await.unwrap();
        rename(&pool, id, "Hannover neu").await.unwrap();
        let got = get(&pool, id).await.unwrap().unwrap();
        assert_eq!(got.name, "Hannover neu");
        assert_eq!(got.radius_km, 25);
    }

    #[tokio::test]
    async fn delete_removes_row() {
        let pool = open_in_memory().await;
        let id = create(&pool, &sample()).await.unwrap();
        delete(&pool, id).await.unwrap();
        assert!(list_all(&pool).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn mark_run_sets_last_run_at() {
        let pool = open_in_memory().await;
        let id = create(&pool, &sample()).await.unwrap();
        let before = get(&pool, id).await.unwrap().unwrap();
        assert!(before.last_run_at.is_none());
        mark_run(&pool, id).await.unwrap();
        let after = get(&pool, id).await.unwrap().unwrap();
        assert!(after.last_run_at.is_some());
    }
}
