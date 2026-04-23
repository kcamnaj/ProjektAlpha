use sqlx::SqlitePool;

const MIGRATIONS: &[(&str, &str)] = &[
    ("0001_initial", include_str!("migrations/0001_initial.sql")),
    (
        "0002_seed_categories",
        include_str!("migrations/0002_seed_categories.sql"),
    ),
];

pub async fn run(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT PRIMARY KEY,
            applied_at TEXT NOT NULL
        )",
    )
    .execute(pool)
    .await?;

    for (version, sql) in MIGRATIONS {
        let already: Option<(String,)> =
            sqlx::query_as("SELECT version FROM schema_migrations WHERE version = ?")
                .bind(version)
                .fetch_optional(pool)
                .await?;

        if already.is_some() {
            tracing::debug!(version, "migration already applied");
            continue;
        }

        let started = std::time::Instant::now();
        let mut tx = pool.begin().await?;
        for stmt in sql.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            sqlx::query(stmt).execute(&mut *tx).await?;
        }
        sqlx::query(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?, datetime('now'))",
        )
        .bind(version)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        tracing::info!(
            version,
            dauer_ms = started.elapsed().as_millis() as u64,
            "migration applied"
        );
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
        let tables: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
                .fetch_all(&pool)
                .await
                .unwrap();
        let names: Vec<&str> = tables.iter().map(|(n,)| n.as_str()).collect();
        for expected in [
            "activity_log",
            "app_meta",
            "companies",
            "geocode_cache",
            "industry_categories",
            "schema_migrations",
            "search_profiles",
        ] {
            assert!(names.contains(&expected), "missing table: {expected}");
        }
    }

    #[tokio::test]
    async fn migrations_idempotent() {
        let pool = open_in_memory().await;
        run(&pool).await.unwrap();
        run(&pool).await.unwrap();
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM schema_migrations")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count.0, MIGRATIONS.len() as i64);
    }

    #[tokio::test]
    async fn seed_inserts_eleven_categories() {
        let pool = open_in_memory().await;
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM industry_categories")
            .fetch_one(&pool)
            .await
            .unwrap();
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
}
