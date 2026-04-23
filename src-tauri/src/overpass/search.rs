use crate::db::{
    categories,
    companies::{insert_or_merge, NewCompany},
};
use crate::error::{AppError, AppResult};
use crate::overpass::{
    client::OverpassClient, parser::parse, query_builder::build, tile_splitter::split,
};
use serde::Serialize;
use sqlx::SqlitePool;
use std::time::Duration;

#[derive(Serialize, Clone, Debug)]
pub struct ProgressEvent {
    pub tile_idx: usize,
    pub tile_total: usize,
    pub last_count: usize,
    pub running_total_inserted: usize,
}

#[derive(Serialize, Debug)]
pub struct SearchStats {
    pub total_found: usize,
    pub neu_imported: usize,
    pub duplicates_skipped: usize,
    pub dauer_ms: u64,
}

pub struct SearchInput {
    pub center_lat: f64,
    pub center_lng: f64,
    pub radius_km: u32,
    pub category_ids: Vec<i64>,
}

pub async fn run<F>(
    pool: &SqlitePool,
    client: &OverpassClient,
    input: SearchInput,
    mut on_progress: F,
) -> AppResult<SearchStats>
where
    F: FnMut(ProgressEvent),
{
    if !(1..=300).contains(&input.radius_km) {
        return Err(AppError::InvalidInput("radius_km must be 1..=300".into()));
    }
    let cats = categories::list_by_ids(pool, &input.category_ids).await?;
    if cats.is_empty() {
        return Err(AppError::InvalidInput(
            "no enabled categories selected".into(),
        ));
    }
    let tiles = split(input.center_lat, input.center_lng, input.radius_km);
    let started = std::time::Instant::now();
    tracing::info!(
        center_lat = input.center_lat,
        center_lng = input.center_lng,
        radius_km = input.radius_km,
        n_categories = cats.len(),
        n_tiles = tiles.len(),
        "search start"
    );

    let mut total_found = 0usize;
    let mut neu_imported = 0usize;
    let mut duplicates = 0usize;

    for (idx, tile) in tiles.iter().enumerate() {
        let ql = build(&cats, tile)?;
        let body = client.run_query(&ql).await?;
        let companies: Vec<NewCompany> = parse(&body, &cats)?;
        total_found += companies.len();

        let mut tile_inserted = 0usize;
        for company in companies {
            let r = insert_or_merge(pool, &company).await?;
            if r.inserted {
                neu_imported += 1;
                tile_inserted += 1;
            } else {
                duplicates += 1;
            }
        }

        on_progress(ProgressEvent {
            tile_idx: idx + 1,
            tile_total: tiles.len(),
            last_count: tile_inserted,
            running_total_inserted: neu_imported,
        });

        // Etiquette: 1s pause between tiles
        if idx + 1 < tiles.len() {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    let stats = SearchStats {
        total_found,
        neu_imported,
        duplicates_skipped: duplicates,
        dauer_ms: started.elapsed().as_millis() as u64,
    };
    tracing::info!(
        total_found = stats.total_found,
        neu_imported = stats.neu_imported,
        duplicates = stats.duplicates_skipped,
        dauer_ms = stats.dauer_ms,
        "search done"
    );
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[tokio::test]
    async fn invalid_radius_rejected() {
        let pool = open_in_memory().await;
        let client = OverpassClient::new(vec!["http://127.0.0.1:9999".into()]);
        let r = run(
            &pool,
            &client,
            SearchInput {
                center_lat: 52.0,
                center_lng: 9.0,
                radius_km: 0,
                category_ids: vec![1],
            },
            |_| {},
        )
        .await;
        assert!(matches!(r, Err(AppError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn empty_categories_rejected() {
        let pool = open_in_memory().await;
        let client = OverpassClient::new(vec!["http://127.0.0.1:9999".into()]);
        let r = run(
            &pool,
            &client,
            SearchInput {
                center_lat: 52.0,
                center_lng: 9.0,
                radius_km: 10,
                category_ids: vec![],
            },
            |_| {},
        )
        .await;
        assert!(matches!(r, Err(AppError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn end_to_end_with_mock_overpass() {
        let pool = open_in_memory().await;
        let mut server = mockito::Server::new_async().await;
        let body = r#"{"elements":[
            {"type":"node","id":1,"lat":52.0,"lon":9.0,"tags":{"shop":"supermarket","name":"Test"}}
        ]}"#;
        let _m = server
            .mock("POST", "/")
            .with_status(200)
            .with_body(body)
            .create_async()
            .await;

        let client = OverpassClient::new(vec![server.url() + "/"]);
        let stats = run(
            &pool,
            &client,
            SearchInput {
                center_lat: 52.0,
                center_lng: 9.0,
                radius_km: 5,
                category_ids: vec![6], // Lebensmittel-Einzelhandel matches shop=supermarket
            },
            |_| {},
        )
        .await
        .unwrap();
        assert_eq!(stats.neu_imported, 1);
    }
}
