use std::path::PathBuf;
use std::sync::OnceLock;

static CRASH_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn init(crash_dir: PathBuf) {
    let _ = std::fs::create_dir_all(&crash_dir);
    let _ = CRASH_DIR.set(crash_dir);
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        write_crash("rust_panic", &info.to_string());
        prev(info);
    }));
}

pub fn write_crash(kind: &str, body: &str) {
    if let Some(dir) = CRASH_DIR.get() {
        let ts = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
        let path = dir.join(format!("crash-{kind}-{ts}.txt"));
        let content = format!(
            "version: {}\ntimestamp: {}\nkind: {}\n\n{}\n",
            env!("CARGO_PKG_VERSION"),
            chrono::Utc::now().to_rfc3339(),
            kind,
            body
        );
        let _ = std::fs::write(path, content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn write_crash_creates_file() {
        let dir = tempdir().unwrap();
        init(dir.path().to_path_buf());
        write_crash("test", "hello world");
        let entries: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
        assert_eq!(entries.len(), 1);
    }
}
