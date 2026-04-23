import { invoke } from "@tauri-apps/api/core"
import { listen, type UnlistenFn } from "@tauri-apps/api/event"

export type CategoryRow = {
  id: number
  name_de: string
  probability_weight: number
  enabled: boolean
  color: string
}

export type CategoryFull = {
  id: number
  name_de: string
  osm_tags: string
  probability_weight: number
  enabled: boolean
  color: string
}

export type NewCategoryInput = {
  name_de: string
  osm_tags: string
  probability_weight: number
  color: string
}

export type UpdateCategoryInput = NewCategoryInput & { id: number }

export type SearchStats = {
  total_found: number
  neu_imported: number
  duplicates_skipped: number
  dauer_ms: number
}

export type ProgressEvent = {
  tile_idx: number
  tile_total: number
  last_count: number
  running_total_inserted: number
}

export type CompanyRow = {
  id: string
  name: string
  street: string | null
  postal_code: string | null
  city: string | null
  country: string
  lat: number
  lng: number
  phone: string | null
  email: string | null
  website: string | null
  industry_category_id: number | null
  category_name: string | null
  category_color: string | null
  probability_score: number
  status: string
  contact_person: string | null
  last_contact_at: string | null
  next_followup_at: string | null
  source: string
  created_at: string
  updated_at: string
}

export type ListFilter = {
  status?: string | null
  category_ids?: number[] | null
  min_score?: number | null
  search?: string | null
  limit?: number | null
  offset?: number | null
}

export type ActivityRow = {
  id: string
  company_id: string
  type: string
  content: string
  created_at: string
}

export type GeocodeSuggestion = {
  lat: number
  lng: number
  display_name: string
}

export type DashboardKpis = {
  customers: number
  requested: number
  new_count: number
  avg_score: number
  total_active: number
}

export type RecentActivityRow = {
  id: string
  company_id: string
  company_name: string
  type: string         // 'notiz'|'anruf'|'mail'|'besuch'|'status_änderung'
  content: string
  created_at: string   // RFC3339
}

export type SearchProfile = {
  id: number
  name: string
  center_label: string
  center_lat: number
  center_lng: number
  radius_km: number
  enabled_category_ids: string // JSON-Array von Zahlen
  last_run_at: string | null
  created_at: string
}

export type NewSearchProfileInput = {
  name: string
  center_label: string
  center_lat: number
  center_lng: number
  radius_km: number
  enabled_category_ids: string
}

export const api = {
  listCategories: () => invoke<CategoryRow[]>("list_categories"),
  listAllCategories: () => invoke<CategoryFull[]>("list_all_categories"),
  createCategory: (payload: NewCategoryInput) => invoke<number>("create_category", { payload }),
  updateCategory: (payload: UpdateCategoryInput) => invoke<void>("update_category", { payload }),
  setCategoryEnabled: (id: number, enabled: boolean) =>
    invoke<void>("set_category_enabled", { payload: { id, enabled } }),
  deleteCategory: (id: number) => invoke<void>("delete_category", { id }),
  startSearch: (payload: { center_lat: number; center_lng: number; radius_km: number; category_ids: number[] }) =>
    invoke<SearchStats>("start_search", { payload }),
  onSearchProgress: (cb: (e: ProgressEvent) => void): Promise<UnlistenFn> =>
    listen<ProgressEvent>("search-progress", (e) => cb(e.payload)),
  onSearchDone: (cb: (s: SearchStats) => void): Promise<UnlistenFn> =>
    listen<SearchStats>("search-done", (e) => cb(e.payload)),
  listCompanies: (filter: ListFilter) => invoke<CompanyRow[]>("list_companies", { filter }),
  getCompany: (id: string) => invoke<CompanyRow | null>("get_company", { id }),
  updateCompanyStatus: (id: string, new_status: string) =>
    invoke<void>("update_company_status", { payload: { id, new_status } }),
  updateCompanyFollowup: (id: string, when: string | null) =>
    invoke<void>("update_company_followup", { payload: { id, when } }),
  updateCompanyContactPerson: (id: string, person: string | null) =>
    invoke<void>("update_company_contact_person", { payload: { id, person } }),
  deleteCompany: (id: string) => invoke<void>("delete_company", { id }),
  listActivity: (company_id: string) => invoke<ActivityRow[]>("list_activity", { companyId: company_id }),
  addActivity: (payload: { company_id: string; type: string; content: string }) =>
    invoke<ActivityRow>("add_activity", { payload }),
  addManualCompany: (payload: {
    osm_id: string | null
    name: string
    street: string | null
    postal_code: string | null
    city: string | null
    country: string
    lat: number; lng: number
    phone: string | null; email: string | null; website: string | null
    industry_category_id: number | null
    size_estimate: string | null
    probability_score: number
    source: "manual"
  }) => invoke<{ inserted: boolean; updated_fields: string[] }>("add_manual_company", { payload }),
  geocode: (query: string) => invoke<GeocodeSuggestion[]>("geocode", { payload: { query } }),
  listSearchProfiles: () => invoke<SearchProfile[]>("list_search_profiles"),
  createSearchProfile: (payload: NewSearchProfileInput) =>
    invoke<number>("create_search_profile", { payload }),
  renameSearchProfile: (id: number, new_name: string) =>
    invoke<void>("rename_search_profile", { payload: { id, new_name } }),
  deleteSearchProfile: (id: number) => invoke<void>("delete_search_profile", { id }),
  markSearchProfileRun: (id: number) => invoke<void>("mark_search_profile_run", { id }),
  backupDb: () => invoke<string | null>("backup_db"),
  restoreDb: () => invoke<boolean>("restore_db"),
  openDataDir: () => invoke<void>("open_data_dir"),
  appVersion: () => invoke<string>("app_version"),
  dashboardKpis: () => invoke<DashboardKpis>("dashboard_kpis"),
  listDueFollowups: () => invoke<CompanyRow[]>("list_due_followups"),
  listRecentActivity: (limit?: number) =>
    invoke<RecentActivityRow[]>("list_recent_activity", { limit: limit ?? 20 }),
}
