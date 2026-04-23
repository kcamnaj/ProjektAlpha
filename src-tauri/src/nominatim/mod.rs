pub mod client;

use crate::db::geocode_cache;
use crate::error::AppResult;
use client::{NominatimClient, Suggestion};
use sqlx::SqlitePool;

/// Cache-first: prüft `geocode_cache` (30-Tage-TTL), bei Miss → HTTP-Call + Upsert.
/// Bei Cache-Hit wird nur der erste (beste) Vorschlag zurückgegeben. Bei Miss werden
/// bis zu 5 Vorschläge geliefert UND der beste wird gecached — das erlaubt dem Frontend
/// eine Auswahl und hält den Cache kompakt.
pub async fn query(
    pool: &SqlitePool,
    client: &NominatimClient,
    q: &str,
) -> AppResult<Vec<Suggestion>> {
    let trimmed = q.trim();
    if trimmed.is_empty() {
        return Ok(vec![]);
    }

    if let Some(cached) = geocode_cache::get_fresh(pool, trimmed).await? {
        tracing::debug!(q_len = trimmed.len(), "geocode cache hit");
        return Ok(vec![Suggestion {
            lat: cached.lat,
            lng: cached.lng,
            display_name: cached.display_name,
        }]);
    }

    tracing::debug!(q_len = trimmed.len(), "geocode cache miss");
    let suggestions = client.query(trimmed).await?;
    if let Some(best) = suggestions.first() {
        geocode_cache::upsert(pool, trimmed, best.lat, best.lng, &best.display_name).await?;
    }
    Ok(suggestions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    #[tokio::test]
    async fn empty_query_returns_empty_vec_without_http() {
        let pool = open_in_memory().await;
        // Client mit offensichtlich ungültigem Endpoint — sollte nie aufgerufen werden
        let client = NominatimClient::with_endpoint("http://127.0.0.1:1");
        let got = query(&pool, &client, "   ").await.unwrap();
        assert!(got.is_empty());
    }

    #[tokio::test]
    async fn cache_hit_skips_http_call() {
        let pool = open_in_memory().await;
        // Cache vorab füllen
        geocode_cache::upsert(&pool, "Hannover", 52.37, 9.73, "Hannover, Deutschland")
            .await
            .unwrap();
        // Client mit kaputtem Endpoint — wenn der aufgerufen würde, käme ein Fehler
        let client = NominatimClient::with_endpoint("http://127.0.0.1:1");
        let got = query(&pool, &client, "Hannover").await.unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].display_name, "Hannover, Deutschland");
    }

    #[tokio::test]
    async fn cache_miss_calls_http_and_stores_result() {
        use client::Suggestion;
        let pool = open_in_memory().await;
        let mut server = mockito::Server::new_async().await;
        // .expect(1) ist der echte Cache-Assert — wenn der 2. Call den Server
        // nochmal anfragt, failt der Test beim Drop von `_m`.
        let _m = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_body(r#"[{"lat":"52.5","lon":"13.4","display_name":"Berlin, Deutschland"}]"#)
            .expect(1)
            .create_async()
            .await;

        let client = NominatimClient::with_endpoint(server.url());
        let got = query(&pool, &client, "Berlin").await.unwrap();
        assert_eq!(
            got,
            vec![Suggestion {
                lat: 52.5,
                lng: 13.4,
                display_name: "Berlin, Deutschland".into()
            }]
        );

        // Zweiter Call → Cache-Hit (server.mock darf KEIN zweites Mal aufgerufen werden)
        let again = query(&pool, &client, "Berlin").await.unwrap();
        assert_eq!(again.len(), 1);
    }
}
