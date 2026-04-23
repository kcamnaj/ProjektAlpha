use crate::error::{AppError, AppResult};
use std::time::Duration;

pub struct OverpassClient {
    endpoints: Vec<String>,
    http: reqwest::Client,
    pub max_retries: u32,
}

impl OverpassClient {
    pub fn new(endpoints: Vec<String>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(60))
            .user_agent("ProjektAlpha/0.1")
            .build()
            .expect("reqwest client");
        Self {
            endpoints,
            http,
            max_retries: 3,
        }
    }

    pub fn default_endpoints() -> Vec<String> {
        vec![
            "https://overpass-api.de/api/interpreter".into(),
            "https://overpass.kumi.systems/api/interpreter".into(),
            "https://overpass.private.coffee/api/interpreter".into(),
        ]
    }

    pub async fn run_query(&self, ql: &str) -> AppResult<String> {
        let mut last_err: Option<AppError> = None;
        for endpoint in &self.endpoints {
            for attempt in 0..self.max_retries {
                let started = std::time::Instant::now();
                let res = self.http.post(endpoint).body(ql.to_string()).send().await;
                let dauer_ms = started.elapsed().as_millis() as u64;
                match res {
                    Ok(r) if r.status().is_success() => {
                        let text = r.text().await?;
                        tracing::debug!(
                            endpoint,
                            attempt,
                            dauer_ms,
                            bytes = text.len(),
                            "overpass success"
                        );
                        return Ok(text);
                    }
                    Ok(r) => {
                        let status = r.status();
                        tracing::warn!(
                            endpoint,
                            attempt,
                            dauer_ms,
                            http_status = status.as_u16(),
                            "overpass non-2xx"
                        );
                        last_err = Some(AppError::Internal(format!("http {}", status)));
                        // retry on 5xx and 429; for other 4xx, break out of retry loop and rotate
                        if !(status.is_server_error() || status.as_u16() == 429) {
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::warn!(endpoint, attempt, dauer_ms, fehler = %e, "overpass network error");
                        last_err = Some(e.into());
                    }
                }
                let backoff_ms = 500 * 2_u64.pow(attempt);
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }
            tracing::warn!(endpoint, "endpoint exhausted, rotating");
        }
        Err(last_err.unwrap_or_else(|| AppError::Internal("all endpoints exhausted".into())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn returns_body_on_first_success() {
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("POST", "/")
            .with_status(200)
            .with_body("{\"elements\":[]}")
            .create_async()
            .await;

        let client = OverpassClient::new(vec![server.url() + "/"]);
        let body = client.run_query("test").await.unwrap();
        assert!(body.contains("elements"));
        m.assert_async().await;
    }

    #[tokio::test]
    async fn retries_on_500_then_succeeds() {
        let mut server = mockito::Server::new_async().await;
        let _m1 = server
            .mock("POST", "/")
            .with_status(500)
            .expect(1)
            .create_async()
            .await;
        let _m2 = server
            .mock("POST", "/")
            .with_status(200)
            .with_body("ok")
            .create_async()
            .await;

        let mut client = OverpassClient::new(vec![server.url() + "/"]);
        client.max_retries = 2;
        let body = client.run_query("q").await.unwrap();
        assert_eq!(body, "ok");
    }

    #[tokio::test]
    async fn rotates_endpoint_when_all_retries_fail() {
        let mut server_a = mockito::Server::new_async().await;
        let _ma = server_a
            .mock("POST", "/")
            .with_status(503)
            .expect(2)
            .create_async()
            .await;
        let mut server_b = mockito::Server::new_async().await;
        let _mb = server_b
            .mock("POST", "/")
            .with_status(200)
            .with_body("from-b")
            .create_async()
            .await;

        let mut client = OverpassClient::new(vec![server_a.url() + "/", server_b.url() + "/"]);
        client.max_retries = 2;
        let body = client.run_query("q").await.unwrap();
        assert_eq!(body, "from-b");
    }
}
