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
    Ok(rows
        .into_iter()
        .map(|(id, name_de, osm_tags, w, e, color)| Category {
            id,
            name_de,
            osm_tags,
            probability_weight: w,
            enabled: e != 0,
            color,
        })
        .collect())
}

pub async fn list_by_ids(pool: &SqlitePool, ids: &[i64]) -> AppResult<Vec<Category>> {
    if ids.is_empty() {
        return Ok(vec![]);
    }
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let q = format!("SELECT id, name_de, osm_tags, probability_weight, enabled, color FROM industry_categories WHERE id IN ({}) ORDER BY sort_order", placeholders);
    let mut query = sqlx::query_as::<_, (i64, String, String, i64, i64, String)>(&q);
    for id in ids {
        query = query.bind(id);
    }
    let rows = query.fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|(id, name_de, osm_tags, w, e, color)| Category {
            id,
            name_de,
            osm_tags,
            probability_weight: w,
            enabled: e != 0,
            color,
        })
        .collect())
}

pub async fn list_all(pool: &SqlitePool) -> AppResult<Vec<Category>> {
    let rows: Vec<(i64, String, String, i64, i64, String)> = sqlx::query_as(
        "SELECT id, name_de, osm_tags, probability_weight, enabled, color FROM industry_categories ORDER BY sort_order, id"
    ).fetch_all(pool).await?;
    Ok(rows
        .into_iter()
        .map(|(id, name_de, osm_tags, w, e, color)| Category {
            id,
            name_de,
            osm_tags,
            probability_weight: w,
            enabled: e != 0,
            color,
        })
        .collect())
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewCategory {
    pub name_de: String,
    pub osm_tags: String,
    pub probability_weight: i64,
    pub color: String,
}

pub async fn create(pool: &SqlitePool, c: &NewCategory) -> AppResult<i64> {
    let next_sort: (i64,) =
        sqlx::query_as("SELECT COALESCE(MAX(sort_order), 0) + 10 FROM industry_categories")
            .fetch_one(pool)
            .await?;
    let result = sqlx::query(
        "INSERT INTO industry_categories (name_de, osm_tags, probability_weight, enabled, color, sort_order)
         VALUES (?, ?, ?, 1, ?, ?)"
    )
    .bind(&c.name_de)
    .bind(&c.osm_tags)
    .bind(c.probability_weight)
    .bind(&c.color)
    .bind(next_sort.0)
    .execute(pool).await?;
    Ok(result.last_insert_rowid())
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCategory {
    pub id: i64,
    pub name_de: String,
    pub osm_tags: String,
    pub probability_weight: i64,
    pub color: String,
}

pub async fn update(pool: &SqlitePool, c: &UpdateCategory) -> AppResult<()> {
    sqlx::query(
        "UPDATE industry_categories
         SET name_de = ?, osm_tags = ?, probability_weight = ?, color = ?
         WHERE id = ?",
    )
    .bind(&c.name_de)
    .bind(&c.osm_tags)
    .bind(c.probability_weight)
    .bind(&c.color)
    .bind(c.id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_enabled(pool: &SqlitePool, id: i64, enabled: bool) -> AppResult<()> {
    sqlx::query("UPDATE industry_categories SET enabled = ? WHERE id = ?")
        .bind(if enabled { 1 } else { 0 })
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM industry_categories WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[tokio::test]
    async fn list_enabled_excludes_disabled() {
        let pool = open_in_memory().await;
        let cats = list_enabled(&pool).await.unwrap();
        assert_eq!(cats.len(), 10); // 11 seeds, 1 disabled (Bürogebäude)
        assert!(cats.iter().all(|c| c.enabled));
    }

    #[tokio::test]
    async fn list_by_ids_returns_ordered() {
        let pool = open_in_memory().await;
        let cats = list_by_ids(&pool, &[2, 1]).await.unwrap();
        assert_eq!(cats.len(), 2);
        assert_eq!(cats[0].id, 1); // sort_order 10 < 20
    }

    #[tokio::test]
    async fn list_all_includes_disabled() {
        let pool = open_in_memory().await;
        let all = list_all(&pool).await.unwrap();
        assert_eq!(all.len(), 11);
        assert!(all.iter().any(|c| !c.enabled));
    }

    #[tokio::test]
    async fn create_inserts_with_next_sort_order() {
        let pool = open_in_memory().await;
        let new_id = create(
            &pool,
            &NewCategory {
                name_de: "TestBranche".into(),
                osm_tags: r#"[{"shop":"computer"}]"#.into(),
                probability_weight: 50,
                color: "#123456".into(),
            },
        )
        .await
        .unwrap();
        assert!(new_id > 11);
        let after = list_all(&pool).await.unwrap();
        assert_eq!(after.len(), 12);
        let inserted = after.iter().find(|c| c.id == new_id).unwrap();
        assert_eq!(inserted.name_de, "TestBranche");
        assert_eq!(inserted.probability_weight, 50);
        assert!(inserted.enabled);
    }

    #[tokio::test]
    async fn update_changes_fields() {
        let pool = open_in_memory().await;
        update(
            &pool,
            &UpdateCategory {
                id: 1,
                name_de: "Umbenannt".into(),
                osm_tags: r#"[{"industrial":"warehouse"}]"#.into(),
                probability_weight: 42,
                color: "#abcdef".into(),
            },
        )
        .await
        .unwrap();
        let all = list_all(&pool).await.unwrap();
        let got = all.iter().find(|c| c.id == 1).unwrap();
        assert_eq!(got.name_de, "Umbenannt");
        assert_eq!(got.probability_weight, 42);
        assert_eq!(got.color, "#abcdef");
    }

    #[tokio::test]
    async fn update_enabled_toggles_flag() {
        let pool = open_in_memory().await;
        update_enabled(&pool, 1, false).await.unwrap();
        let all = list_all(&pool).await.unwrap();
        assert!(!all.iter().find(|c| c.id == 1).unwrap().enabled);
        update_enabled(&pool, 1, true).await.unwrap();
        let all = list_all(&pool).await.unwrap();
        assert!(all.iter().find(|c| c.id == 1).unwrap().enabled);
    }

    #[tokio::test]
    async fn delete_removes_row() {
        let pool = open_in_memory().await;
        delete(&pool, 1).await.unwrap();
        let all = list_all(&pool).await.unwrap();
        assert_eq!(all.len(), 10);
        assert!(all.iter().all(|c| c.id != 1));
    }
}
