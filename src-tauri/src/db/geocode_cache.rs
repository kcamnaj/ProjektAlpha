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
    Ok(
        row.map(|(lat, lng, display_name, cached_at)| CachedGeocode {
            lat,
            lng,
            display_name,
            cached_at,
        }),
    )
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
