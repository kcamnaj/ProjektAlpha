use crate::error::{AppError, AppResult};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

const USER_AGENT: &str = "ProjektAlpha/0.1 (kontakt: jan-mack@web.de)";
const DEFAULT_ENDPOINT: &str = "https://nominatim.openstreetmap.org/search";
const MIN_INTERVAL: Duration = Duration::from_millis(1100); // 1 req/s mit Puffer

pub struct NominatimClient {
    http: reqwest::Client,
    endpoint: String,
    last_call: Arc<Mutex<Option<Instant>>>,
    max_results: u32,
}

impl NominatimClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent(USER_AGENT)
            .build()
            .expect("reqwest client");
        Self {
            http,
            endpoint: DEFAULT_ENDPOINT.to_string(),
            last_call: Arc::new(Mutex::new(None)),
            max_results: 5,
        }
    }

    #[cfg(test)]
    pub fn with_endpoint(endpoint: impl Into<String>) -> Self {
        let mut c = Self::new();
        c.endpoint = endpoint.into();
        c
    }

    async fn wait_rate_limit(&self) {
        let mut last = self.last_call.lock().await;
        if let Some(t) = *last {
            let elapsed = t.elapsed();
            if elapsed < MIN_INTERVAL {
                tokio::time::sleep(MIN_INTERVAL - elapsed).await;
            }
        }
        *last = Some(Instant::now());
    }

    pub async fn query(&self, q: &str) -> AppResult<Vec<Suggestion>> {
        self.wait_rate_limit().await;
        let url = format!(
            "{}?q={}&format=json&addressdetails=0&limit={}&countrycodes=de",
            self.endpoint,
            urlencoding::encode(q),
            self.max_results
        );
        let started = Instant::now();
        let response = self.http.get(&url).send().await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            tracing::warn!(
                status = %status,
                dauer_ms = started.elapsed().as_millis() as u64,
                "nominatim non-2xx"
            );
            return Err(AppError::Internal(format!("nominatim {status}")));
        }
        let result = parse_response(&text);
        tracing::debug!(
            q_len = q.len(),
            dauer_ms = started.elapsed().as_millis() as u64,
            count = result.as_ref().map(|v| v.len()).unwrap_or(0),
            "nominatim query"
        );
        result
    }
}

impl Default for NominatimClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Eine Vorschlag-Zeile aus der Nominatim-API.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Suggestion {
    pub lat: f64,
    pub lng: f64,
    pub display_name: String,
}

pub fn parse_response(json: &str) -> AppResult<Vec<Suggestion>> {
    #[derive(serde::Deserialize)]
    struct Raw {
        lat: String,
        lon: String,
        display_name: String,
    }
    let raws: Vec<Raw> = serde_json::from_str(json)
        .map_err(|e| AppError::InvalidInput(format!("nominatim parse: {e}")))?;
    let mut out = Vec::with_capacity(raws.len());
    for r in raws {
        let lat: f64 = r
            .lat
            .parse()
            .map_err(|e| AppError::InvalidInput(format!("lat parse: {e}")))?;
        let lng: f64 = r
            .lon
            .parse()
            .map_err(|e| AppError::InvalidInput(format!("lng parse: {e}")))?;
        out.push(Suggestion {
            lat,
            lng,
            display_name: r.display_name,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RESPONSE: &str = r#"[
        {"place_id":1,"lat":"52.3756","lon":"9.7320","display_name":"Hannover, Deutschland","type":"city"},
        {"place_id":2,"lat":"48.1351","lon":"11.5820","display_name":"München, Deutschland","type":"city"}
    ]"#;

    #[test]
    fn parse_response_returns_two_suggestions() {
        let got = parse_response(SAMPLE_RESPONSE).unwrap();
        assert_eq!(got.len(), 2);
        assert_eq!(got[0].display_name, "Hannover, Deutschland");
        assert!((got[0].lat - 52.3756).abs() < 1e-6);
        assert!((got[0].lng - 9.7320).abs() < 1e-6);
    }

    #[test]
    fn parse_response_empty_array_returns_empty_vec() {
        let got = parse_response("[]").unwrap();
        assert!(got.is_empty());
    }

    #[test]
    fn parse_response_rejects_malformed_json() {
        assert!(parse_response("not json").is_err());
    }

    #[tokio::test]
    async fn client_queries_endpoint_and_parses() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", mockito::Matcher::Regex(r"^/.*q=Hannover.*".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(SAMPLE_RESPONSE)
            .create_async()
            .await;

        let client = NominatimClient::with_endpoint(server.url());
        let results = client.query("Hannover").await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].display_name, "Hannover, Deutschland");
    }

    #[tokio::test]
    async fn client_rate_limits_second_call() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_body("[]")
            .expect_at_least(2)
            .create_async()
            .await;

        let client = NominatimClient::with_endpoint(server.url());
        let started = std::time::Instant::now();
        client.query("A").await.unwrap();
        client.query("B").await.unwrap();
        let elapsed = started.elapsed();
        assert!(
            elapsed >= std::time::Duration::from_millis(1000),
            "second call should be rate-limited, elapsed={:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn client_surfaces_5xx_as_error() {
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(502)
            .create_async()
            .await;

        let client = NominatimClient::with_endpoint(server.url());
        assert!(client.query("X").await.is_err());
    }
}
