use std::path::PathBuf;

pub struct LogConfig {
    pub log_dir: PathBuf,
    pub default_level: String,
}

pub fn init(config: LogConfig) -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    std::fs::create_dir_all(&config.log_dir)?;
    let file_appender = tracing_appender::rolling::daily(&config.log_dir, "projektalpha.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.default_level));

    let file_layer = fmt::layer()
        .json()
        .with_writer(non_blocking)
        .with_current_span(false);

    let console_layer = fmt::layer().with_target(false).with_ansi(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(console_layer)
        .try_init()
        .map_err(|e| anyhow::anyhow!("logger init failed: {e}"))?;

    Ok(guard)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn init_creates_log_directory() {
        let dir = tempdir().unwrap();
        let log_dir = dir.path().join("logs");
        let config = LogConfig {
            log_dir: log_dir.clone(),
            default_level: "info".to_string(),
        };
        let _guard = init(config);
        assert!(log_dir.exists(), "log dir should be created");
    }
}
