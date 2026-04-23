use crate::db::{
    activity::{self, ActivityRow, NewActivity},
    companies::{self, CompanyRow, InsertResult, ListFilter, NewCompany},
};
use crate::error::{AppError, AppResult};
use crate::overpass::{
    client::OverpassClient,
    search::{self, ProgressEvent, SearchInput, SearchStats},
};
use crate::AppState;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

#[derive(Serialize)]
pub struct CategoryRow {
    pub id: i64,
    pub name_de: String,
    pub probability_weight: i64,
    pub enabled: bool,
    pub color: String,
}

#[tauri::command]
pub async fn list_categories(state: State<'_, Arc<AppState>>) -> AppResult<Vec<CategoryRow>> {
    let rows: Vec<(i64, String, i64, i64, String)> = sqlx::query_as(
        "SELECT id, name_de, probability_weight, enabled, color FROM industry_categories ORDER BY sort_order"
    ).fetch_all(&state.db).await?;
    Ok(rows
        .into_iter()
        .map(|(id, name_de, w, enabled, color)| CategoryRow {
            id,
            name_de,
            probability_weight: w,
            enabled: enabled != 0,
            color,
        })
        .collect())
}

#[derive(serde::Deserialize)]
pub struct StartSearchPayload {
    pub center_lat: f64,
    pub center_lng: f64,
    pub radius_km: u32,
    pub category_ids: Vec<i64>,
}

#[tauri::command]
pub async fn start_search(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    payload: StartSearchPayload,
) -> AppResult<SearchStats> {
    let client = OverpassClient::new(OverpassClient::default_endpoints());
    let app_handle = app.clone();
    let stats = search::run(
        &state.db,
        &client,
        SearchInput {
            center_lat: payload.center_lat,
            center_lng: payload.center_lng,
            radius_km: payload.radius_km,
            category_ids: payload.category_ids,
        },
        move |event: ProgressEvent| {
            let _ = app_handle.emit("search-progress", event);
        },
    )
    .await?;
    let _ = app.emit("search-done", &stats);
    Ok(stats)
}

#[tauri::command]
pub async fn list_companies(
    state: State<'_, Arc<crate::AppState>>,
    filter: ListFilter,
) -> AppResult<Vec<CompanyRow>> {
    companies::list(&state.db, &filter).await
}

#[tauri::command]
pub async fn get_company(
    state: State<'_, Arc<crate::AppState>>,
    id: String,
) -> AppResult<Option<CompanyRow>> {
    companies::get(&state.db, &id).await
}

#[derive(Deserialize)]
pub struct UpdateStatusPayload {
    pub id: String,
    pub new_status: String,
}

#[tauri::command]
pub async fn update_company_status(
    state: State<'_, Arc<crate::AppState>>,
    payload: UpdateStatusPayload,
) -> AppResult<()> {
    let prev = companies::update_status(&state.db, &payload.id, &payload.new_status).await?;
    activity::add(
        &state.db,
        &NewActivity {
            company_id: payload.id.clone(),
            r#type: "status_änderung".into(),
            content: format!("von {} auf {}", prev, payload.new_status),
        },
    )
    .await?;
    tracing::info!(
        company_id = %payload.id,
        prev = %prev,
        new = %payload.new_status,
        "status changed"
    );
    Ok(())
}

#[derive(Deserialize)]
pub struct UpdateFollowupPayload {
    pub id: String,
    pub when: Option<String>,
}

#[tauri::command]
pub async fn update_company_followup(
    state: State<'_, Arc<crate::AppState>>,
    payload: UpdateFollowupPayload,
) -> AppResult<()> {
    companies::update_followup(&state.db, &payload.id, payload.when.as_deref()).await
}

#[derive(Deserialize)]
pub struct UpdateContactPersonPayload {
    pub id: String,
    pub person: Option<String>,
}

#[tauri::command]
pub async fn update_company_contact_person(
    state: State<'_, Arc<crate::AppState>>,
    payload: UpdateContactPersonPayload,
) -> AppResult<()> {
    companies::update_contact_person(&state.db, &payload.id, payload.person.as_deref()).await
}

#[tauri::command]
pub async fn delete_company(state: State<'_, Arc<crate::AppState>>, id: String) -> AppResult<()> {
    companies::delete(&state.db, &id).await?;
    tracing::info!(company_id = %id, "company deleted");
    Ok(())
}

#[tauri::command]
pub async fn list_activity(
    state: State<'_, Arc<crate::AppState>>,
    company_id: String,
) -> AppResult<Vec<ActivityRow>> {
    activity::list_for_company(&state.db, &company_id).await
}

#[tauri::command]
pub async fn add_activity(
    state: State<'_, Arc<crate::AppState>>,
    payload: NewActivity,
) -> AppResult<ActivityRow> {
    activity::add(&state.db, &payload).await
}

#[tauri::command]
pub async fn add_manual_company(
    state: State<'_, Arc<crate::AppState>>,
    payload: NewCompany,
) -> AppResult<InsertResult> {
    if payload.source != "manual" {
        return Err(AppError::InvalidInput("source must be 'manual'".into()));
    }
    if payload.name.trim().is_empty() {
        return Err(AppError::InvalidInput("name required".into()));
    }
    companies::insert_or_merge(&state.db, &payload).await
}

#[derive(Deserialize)]
pub struct FrontendLogPayload {
    pub level: String,
    pub message: String,
    pub context: Option<serde_json::Value>,
}

#[tauri::command]
pub fn frontend_log(payload: FrontendLogPayload) -> AppResult<()> {
    let ctx = payload.context.map(|c| c.to_string()).unwrap_or_default();
    match payload.level.as_str() {
        "error" => tracing::error!(source = "frontend", context = %ctx, "{}", payload.message),
        "warn" => tracing::warn!(source = "frontend", context = %ctx, "{}", payload.message),
        _ => tracing::info!(source = "frontend", context = %ctx, "{}", payload.message),
    }
    Ok(())
}

#[derive(Deserialize)]
pub struct CrashReport {
    pub message: String,
    pub stack: Option<String>,
}

#[tauri::command]
pub fn report_frontend_crash(payload: CrashReport) -> AppResult<()> {
    let body = format!(
        "{}\n\n{}",
        payload.message,
        payload.stack.unwrap_or_default()
    );
    crate::crash::write_crash("frontend", &body);
    tracing::error!(
        source = "frontend",
        "frontend crash reported: {}",
        payload.message
    );
    Ok(())
}

use crate::nominatim::{self, client::Suggestion};

#[derive(Deserialize)]
pub struct GeocodePayload {
    pub query: String,
}

#[tauri::command]
pub async fn geocode(
    state: State<'_, Arc<AppState>>,
    payload: GeocodePayload,
) -> AppResult<Vec<Suggestion>> {
    let trimmed = payload.query.trim();
    if trimmed.len() < 3 {
        return Ok(vec![]);
    }
    nominatim::query(&state.db, &state.nominatim, trimmed).await
}

// Category management commands
use crate::db::categories::{self, Category, NewCategory, UpdateCategory};

#[tauri::command]
pub async fn list_all_categories(state: State<'_, Arc<AppState>>) -> AppResult<Vec<Category>> {
    categories::list_all(&state.db).await
}

fn validate_category_input(
    name_de: &str,
    weight: i64,
    color: &str,
    osm_tags: &str,
) -> AppResult<()> {
    if name_de.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "Branchen-Name darf nicht leer sein".into(),
        ));
    }
    if !(0..=100).contains(&weight) {
        return Err(AppError::InvalidInput(
            "Gewichtung muss zwischen 0 und 100 liegen".into(),
        ));
    }
    let c = color.trim_start_matches('#');
    let is_hex = c.len() == 3 || c.len() == 6;
    let all_hex = c.chars().all(|ch| ch.is_ascii_hexdigit());
    if !color.starts_with('#') || !is_hex || !all_hex {
        return Err(AppError::InvalidInput(
            "Farbe muss Hex-Format #RGB oder #RRGGBB haben".into(),
        ));
    }
    let parsed: serde_json::Value = serde_json::from_str(osm_tags)
        .map_err(|e| AppError::InvalidInput(format!("osm_tags: {e}")))?;
    let arr = parsed
        .as_array()
        .ok_or_else(|| AppError::InvalidInput("osm_tags muss ein JSON-Array sein".into()))?;
    for item in arr {
        if !item.is_object() {
            return Err(AppError::InvalidInput(
                "osm_tags: jedes Element muss ein Objekt sein".into(),
            ));
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn create_category(
    state: State<'_, Arc<AppState>>,
    payload: NewCategory,
) -> AppResult<i64> {
    validate_category_input(
        &payload.name_de,
        payload.probability_weight,
        &payload.color,
        &payload.osm_tags,
    )?;
    let id = categories::create(&state.db, &payload).await?;
    tracing::info!(category_id = id, "category created");
    Ok(id)
}

#[tauri::command]
pub async fn update_category(
    state: State<'_, Arc<AppState>>,
    payload: UpdateCategory,
) -> AppResult<()> {
    validate_category_input(
        &payload.name_de,
        payload.probability_weight,
        &payload.color,
        &payload.osm_tags,
    )?;
    categories::update(&state.db, &payload).await?;
    tracing::info!(category_id = payload.id, "category updated");
    Ok(())
}

#[derive(Deserialize)]
pub struct SetCategoryEnabledPayload {
    pub id: i64,
    pub enabled: bool,
}

#[tauri::command]
pub async fn set_category_enabled(
    state: State<'_, Arc<AppState>>,
    payload: SetCategoryEnabledPayload,
) -> AppResult<()> {
    categories::update_enabled(&state.db, payload.id, payload.enabled).await?;
    tracing::info!(
        category_id = payload.id,
        enabled = payload.enabled,
        "category enabled toggled"
    );
    Ok(())
}

#[tauri::command]
pub async fn delete_category(state: State<'_, Arc<AppState>>, id: i64) -> AppResult<()> {
    categories::delete(&state.db, id).await?;
    tracing::info!(category_id = id, "category deleted");
    Ok(())
}

// Search profile commands
use crate::db::search_profiles::{self, NewSearchProfile, SearchProfile};

#[tauri::command]
pub async fn list_search_profiles(
    state: State<'_, Arc<AppState>>,
) -> AppResult<Vec<SearchProfile>> {
    search_profiles::list_all(&state.db).await
}

#[tauri::command]
pub async fn create_search_profile(
    state: State<'_, Arc<AppState>>,
    payload: NewSearchProfile,
) -> AppResult<i64> {
    if payload.name.trim().is_empty() {
        return Err(AppError::InvalidInput(
            "Profil-Name darf nicht leer sein".into(),
        ));
    }
    if !(1..=300).contains(&payload.radius_km) {
        return Err(AppError::InvalidInput(
            "Radius muss zwischen 1 und 300 km liegen".into(),
        ));
    }
    let parsed: serde_json::Value = serde_json::from_str(&payload.enabled_category_ids)
        .map_err(|e| AppError::InvalidInput(format!("enabled_category_ids: {e}")))?;
    if !parsed
        .as_array()
        .map(|a| a.iter().all(|v| v.is_i64() || v.is_u64()))
        .unwrap_or(false)
    {
        return Err(AppError::InvalidInput(
            "enabled_category_ids muss JSON-Array von Integers sein".into(),
        ));
    }
    let id = search_profiles::create(&state.db, &payload).await?;
    tracing::info!(profile_id = id, "search profile created");
    Ok(id)
}

#[derive(Deserialize)]
pub struct RenameProfilePayload {
    pub id: i64,
    pub new_name: String,
}

#[tauri::command]
pub async fn rename_search_profile(
    state: State<'_, Arc<AppState>>,
    payload: RenameProfilePayload,
) -> AppResult<()> {
    if payload.new_name.trim().is_empty() {
        return Err(AppError::InvalidInput("Name darf nicht leer sein".into()));
    }
    search_profiles::rename(&state.db, payload.id, payload.new_name.trim()).await
}

#[tauri::command]
pub async fn delete_search_profile(state: State<'_, Arc<AppState>>, id: i64) -> AppResult<()> {
    search_profiles::delete(&state.db, id).await?;
    tracing::info!(profile_id = id, "search profile deleted");
    Ok(())
}

#[tauri::command]
pub async fn mark_search_profile_run(state: State<'_, Arc<AppState>>, id: i64) -> AppResult<()> {
    search_profiles::mark_run(&state.db, id).await
}

// Backup / Restore / Utility commands
use crate::backup;
use std::path::PathBuf;
use tauri_plugin_dialog::DialogExt;

// Backup/Restore sind synchron: sie blockieren auf den File-Dialog und machen
// danach `std::fs::copy`. `#[tauri::command] pub fn` läuft auf Tauri's dediziertem
// Thread-Pool, damit blockieren wir KEINEN tokio-Worker. Für async commands würde
// `blocking_save_file` einen tokio-Worker parken — Anti-Pattern.
#[tauri::command]
pub fn backup_db(app: AppHandle, state: State<'_, Arc<AppState>>) -> AppResult<Option<String>> {
    let src = state.data_dir.join("data.db");
    if !src.exists() {
        return Err(AppError::Internal("data.db existiert nicht".into()));
    }
    let suggested = backup::backup_suggested_filename_now();
    let picked = app
        .dialog()
        .file()
        .set_file_name(&suggested)
        .add_filter("SQLite-Datenbank", &["db"])
        .blocking_save_file();
    let Some(target) = picked else {
        return Ok(None);
    };
    let target_path: PathBuf = target
        .into_path()
        .map_err(|e| AppError::Internal(format!("dialog path: {e}")))?;
    std::fs::copy(&src, &target_path)?;
    let s = target_path.to_string_lossy().to_string();
    tracing::info!(target_len = s.len(), "backup written");
    Ok(Some(s))
}

#[tauri::command]
pub fn restore_db(app: AppHandle, state: State<'_, Arc<AppState>>) -> AppResult<bool> {
    let picked = app
        .dialog()
        .file()
        .add_filter("SQLite-Datenbank", &["db"])
        .blocking_pick_file();
    let Some(source) = picked else {
        return Ok(false);
    };
    let source_path: PathBuf = source
        .into_path()
        .map_err(|e| AppError::Internal(format!("dialog path: {e}")))?;

    let dst = state.data_dir.join("data.db");
    let snap_dir = backup::snapshot_dir(&state.data_dir);
    std::fs::create_dir_all(&snap_dir)?;
    let snap_path = snap_dir.join(backup::snapshot_filename_now());

    if dst.exists() {
        std::fs::copy(&dst, &snap_path)?;
        tracing::info!(
            snap_len = snap_path.to_string_lossy().len(),
            "pre-restore snapshot written"
        );
    }
    std::fs::copy(&source_path, &dst)?;
    tracing::info!("restore complete — restarting app");

    // app.restart() has return type `-> !` (diverging, Tauri 2.10+). No Ok(...) needed
    // after — the `!` type coerces to any return type.
    app.restart();
}

#[tauri::command]
pub fn open_data_dir(state: State<'_, Arc<AppState>>) -> AppResult<()> {
    tauri_plugin_opener::open_path(&state.data_dir, None::<&str>)
        .map_err(|e| AppError::Internal(format!("open_path: {e}")))?;
    tracing::info!("data dir opened");
    Ok(())
}

#[tauri::command]
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// Dashboard commands
use crate::db::companies as companies_db;
use crate::db::dashboard::{self, DashboardKpis, RecentActivityRow};

#[tauri::command]
pub async fn dashboard_kpis(state: State<'_, Arc<AppState>>) -> AppResult<DashboardKpis> {
    let k = dashboard::fetch_kpis(&state.db).await?;
    tracing::info!(
        customers = k.customers,
        requested = k.requested,
        new_count = k.new_count,
        total_active = k.total_active,
        "dashboard kpis fetched"
    );
    Ok(k)
}

#[tauri::command]
pub async fn list_due_followups(
    state: State<'_, Arc<AppState>>,
) -> AppResult<Vec<companies_db::CompanyRow>> {
    let list = companies_db::list_due_followups(&state.db).await?;
    tracing::info!(count = list.len(), "due followups listed");
    Ok(list)
}

#[tauri::command]
pub async fn list_recent_activity(
    state: State<'_, Arc<AppState>>,
    limit: Option<i64>,
) -> AppResult<Vec<RecentActivityRow>> {
    let lim = limit.unwrap_or(20).clamp(1, 100);
    let list = dashboard::list_recent_activity(&state.db, lim).await?;
    tracing::info!(limit = lim, count = list.len(), "recent activity listed");
    Ok(list)
}
