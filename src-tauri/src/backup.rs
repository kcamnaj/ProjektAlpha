use chrono::Utc;
use std::path::{Path, PathBuf};

/// Gibt den Ordner zurück, in dem Pre-Restore-Snapshots abgelegt werden.
pub fn snapshot_dir(app_data: &Path) -> PathBuf {
    app_data.join("backups")
}

/// Erzeugt einen zeitgestempelten Dateinamen für einen Snapshot.
/// Format: `pre-restore-YYYY-MM-DD-HHMMSS.db`
pub fn snapshot_filename_now() -> String {
    format!("pre-restore-{}.db", Utc::now().format("%Y-%m-%d-%H%M%S"))
}

/// Erzeugt einen Vorschlag für den Backup-Zielnamen.
/// Format: `projektalpha-backup-YYYY-MM-DD.db`
pub fn backup_suggested_filename_now() -> String {
    format!("projektalpha-backup-{}.db", Utc::now().format("%Y-%m-%d"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_dir_is_under_app_data() {
        let got = snapshot_dir(Path::new("/app/data"));
        assert_eq!(got, PathBuf::from("/app/data/backups"));
    }

    #[test]
    fn snapshot_filename_has_correct_prefix_and_extension() {
        let name = snapshot_filename_now();
        assert!(name.starts_with("pre-restore-"), "got: {name}");
        assert!(name.ends_with(".db"), "got: {name}");
    }

    #[test]
    fn backup_suggested_filename_has_correct_prefix_and_extension() {
        let name = backup_suggested_filename_now();
        assert!(name.starts_with("projektalpha-backup-"), "got: {name}");
        assert!(name.ends_with(".db"), "got: {name}");
    }
}
