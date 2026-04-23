pub mod backup;
pub mod commands;
pub mod crash;
pub mod db;
pub mod error;
pub mod logging;
pub mod nominatim;
pub mod overpass;

use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;

pub struct AppState {
    pub db: SqlitePool,
    pub nominatim: crate::nominatim::client::NominatimClient,
    pub data_dir: PathBuf,
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let data_dir: PathBuf = dirs::data_dir()
        .map(|p| p.join("projektalpha"))
        .unwrap_or_else(|| PathBuf::from("./projektalpha"));
    let log_dir = data_dir.join("logs");

    let _guard = logging::init(logging::LogConfig {
        log_dir,
        default_level: "info".to_string(),
    })
    .expect("logger init failed");

    let crash_dir = data_dir.join("crashes");
    crash::init(crash_dir);

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "app starting");

    let db_path = data_dir.join("data.db");
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    let pool = runtime
        .block_on(db::open(&db_path))
        .expect("db open failed");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Arc::new(AppState {
            db: pool,
            nominatim: crate::nominatim::client::NominatimClient::new(),
            data_dir: data_dir.clone(),
        }))
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::list_categories,
            commands::start_search,
            commands::list_companies,
            commands::get_company,
            commands::update_company_status,
            commands::update_company_followup,
            commands::update_company_contact_person,
            commands::delete_company,
            commands::list_activity,
            commands::add_activity,
            commands::add_manual_company,
            commands::geocode,
            commands::frontend_log,
            commands::report_frontend_crash,
            commands::list_all_categories,
            commands::create_category,
            commands::update_category,
            commands::set_category_enabled,
            commands::delete_category,
            commands::list_search_profiles,
            commands::create_search_profile,
            commands::rename_search_profile,
            commands::delete_search_profile,
            commands::mark_search_profile_run,
            commands::backup_db,
            commands::restore_db,
            commands::open_data_dir,
            commands::app_version,
            commands::dashboard_kpis,
            commands::list_due_followups,
            commands::list_recent_activity,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    drop(runtime);
}
