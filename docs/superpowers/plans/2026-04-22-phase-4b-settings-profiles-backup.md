# Phase 4b: Settings-UI + Such-Profile + Backup/Restore – Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Die App bekommt eine echte Settings-Ansicht mit vier Tabs: (1) Branchen-Editor zum Bearbeiten der Industrie-Kategorien inkl. OSM-Tags-JSON + Pin-Farbe, (2) Such-Profile zum Speichern wiederkehrender Suchen (inkl. „Als Profil speichern"-Integration in NewSearchPage), (3) Daten-Tab mit Backup/Restore und „Datenordner öffnen", (4) Über-Tab mit Version. Nach diesem Plan ist Phase 4 komplett — der Vater kann Branchen nach Bedarf anpassen, Standardsuchen ein-klick-starten und seine Daten sichern.

**Architecture:** Backend bekommt CRUD-Module für `industry_categories` (Erweiterung von `db/categories.rs`) und `search_profiles` (neues Modul). Backup/Restore nutzt die bereits installierten Tauri-Plugins `dialog` (File-Picker) und `fs` (Copy) auf Rust-Seite — keine neue Frontend-Dep. Restore ruft `app.restart()` nach dem DB-Austausch, damit der sqlx-Pool keinen Stale-State hat. Frontend bekommt eine neue `SettingsPage` mit shadcn Tabs-Komponente plus vier Tab-Komponenten. `NewSearchPage` lernt, Profile zu laden und das aktuelle Formular als Profil zu speichern.

**Tech Stack:** Rust (sqlx, tauri::AppHandle::restart (`-> !`), tauri_plugin_dialog::DialogExt für File-Picker, `std::fs::copy` für Backup/Restore), React (shadcn Tabs + Dialog + Select, bestehende Input/Label/Button). Farbpicker: natives `<input type="color">` (kein extra Paket). OSM-Tags-Editor: simples `<Textarea>` mit JSON.parse-Validierung im `onBlur` — strukturierter Per-Tag-Editor wäre YAGNI für den Single-User-Nutzer.

**Spec-Referenz:** `docs/superpowers/specs/2026-04-21-firmensuche-design.md`
- §5 (Datenmodell `industry_categories`, `search_profiles`, `app_meta`) · §7.2 Screen 6 (Einstellungen-Tabs) · §10a (Logging ohne PII)

**CLAUDE.md-Prinzipien:**
- **UX zuerst:** Branchen-Liste ist inline editierbar (enable-Toggle + Quick-Edit-Dialog für Details). Backup zeigt per Toast den Ziel-Pfad an. Restore fragt **zweimal** nach (Picker + nochmaliger Confirm-Dialog), weil es die Daten aller anderen Tabs ersetzt. Profile-„Laden" füllt das NewSearchPage-Formular mit einem Klick.
- **Nicht kompliziert:** OSM-Tags-Editor ist eine Textarea mit Live-Validation, kein Struktur-Editor. Backup ist ein rohes SQLite-File-Copy — keine SQL-Dump-Formate, kein Versionshandshake.
- **Tests + Logs:** Alle neuen DB-Funktionen sind TDD (in-memory SQLite). Backup/Restore-Pfadmathematik ist eine pure Funktion und testbar. Logs enthalten Tabellennamen, Counts, Pfad-Längen — **nicht** Profil-Namen, OSM-Tag-Inhalte oder vollständige Pfade (letzteres kann User-Namen enthalten = keine PII, aber halten wir kurz).

**Kein Git** – Checkpoints statt Commits.

---

## Datei-Struktur (Ziel nach diesem Plan)

```
src-tauri/src/
├─ db/
│  ├─ categories.rs                     # MODIFY: +list_all, +create, +update, +delete, +update_enabled
│  └─ search_profiles.rs                # NEU: CRUD
├─ backup.rs                            # NEU: backup_to + restore_from + snapshot_dir
├─ commands.rs                          # MODIFY: +8 neue Commands
└─ lib.rs                               # MODIFY: plugin(dialog) + plugin(fs) init, +Commands registrieren

src/
├─ lib/
│  └─ tauri.ts                          # MODIFY: +API + Types für Categories, Profiles, Backup
├─ components/
│  ├─ ui/
│  │  └─ tabs.tsx                       # shadcn (auto-gen in Task 4b.1)
│  └─ settings/
│     ├─ BranchenTab.tsx                # NEU: Liste + Add-Button
│     ├─ BranchenEditDialog.tsx         # NEU: Name + osm_tags + weight + color + enabled
│     ├─ ProfileTab.tsx                 # NEU: Liste mit Load/Rename/Duplicate/Delete
│     ├─ DatenTab.tsx                   # NEU: Backup/Restore/DB-Pfad öffnen
│     └─ UeberTab.tsx                   # NEU: Version + Kurzbeschreibung
├─ pages/
│  ├─ SettingsPage.tsx                  # NEU: Tabs-Shell
│  └─ NewSearchPage.tsx                 # MODIFY: Profil-Load-Dropdown + „Als Profil speichern"-Button
└─ App.tsx                              # MODIFY: view === "settings" → <SettingsPage />
```

Keine neue Migration — `industry_categories`, `search_profiles`, `app_meta` existieren seit Plan 1.

---

# PHASE 4b — Settings + Profile + Backup

## Settings-Shell

---

### Task 4b.1: shadcn Tabs installieren + `SettingsPage`-Shell + Routing

**Files:**
- Auto-generated: `src/components/ui/tabs.tsx`
- Create: `src/pages/SettingsPage.tsx`
- Modify: `src/App.tsx`

- [ ] **Step 1: shadcn Tabs**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm dlx shadcn@latest add tabs --yes --overwrite 2>&1 | tail -5
```

Erwartet: `src/components/ui/tabs.tsx` erstellt.

- [ ] **Step 2: Settings-Shell mit vier leeren Tabs**

```tsx
// src/pages/SettingsPage.tsx
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { BranchenTab } from "@/components/settings/BranchenTab"
import { ProfileTab } from "@/components/settings/ProfileTab"
import { DatenTab } from "@/components/settings/DatenTab"
import { UeberTab } from "@/components/settings/UeberTab"

export function SettingsPage() {
  return (
    <div className="h-full flex flex-col">
      <div className="p-4 border-b">
        <h2 className="text-lg font-semibold">Einstellungen</h2>
      </div>
      <Tabs defaultValue="branchen" className="flex-1 flex flex-col">
        <TabsList className="mx-4 mt-3 self-start">
          <TabsTrigger value="branchen">Branchen</TabsTrigger>
          <TabsTrigger value="profile">Profile</TabsTrigger>
          <TabsTrigger value="daten">Daten</TabsTrigger>
          <TabsTrigger value="ueber">Über</TabsTrigger>
        </TabsList>
        <TabsContent value="branchen" className="flex-1 overflow-y-auto p-4"><BranchenTab /></TabsContent>
        <TabsContent value="profile" className="flex-1 overflow-y-auto p-4"><ProfileTab /></TabsContent>
        <TabsContent value="daten" className="flex-1 overflow-y-auto p-4"><DatenTab /></TabsContent>
        <TabsContent value="ueber" className="flex-1 overflow-y-auto p-4"><UeberTab /></TabsContent>
      </Tabs>
    </div>
  )
}
```

- [ ] **Step 3: Stub-Tabs anlegen, damit der Import kompiliert**

Jeweils ein minimaler Platzhalter — die echten Inhalte kommen in den Tasks 4b.5 / 4b.8 / 4b.12 / 4b.13:

```tsx
// src/components/settings/BranchenTab.tsx
export function BranchenTab() { return <div className="text-sm text-muted-foreground">Wird in Task 4b.5 gefüllt.</div> }
```

```tsx
// src/components/settings/ProfileTab.tsx
export function ProfileTab() { return <div className="text-sm text-muted-foreground">Wird in Task 4b.8 gefüllt.</div> }
```

```tsx
// src/components/settings/DatenTab.tsx
export function DatenTab() { return <div className="text-sm text-muted-foreground">Wird in Task 4b.12 gefüllt.</div> }
```

```tsx
// src/components/settings/UeberTab.tsx
export function UeberTab() { return <div className="text-sm text-muted-foreground">Wird in Task 4b.13 gefüllt.</div> }
```

- [ ] **Step 4: App-Routing**

In `src/App.tsx`:
- Import hinzufügen: `import { SettingsPage } from "@/pages/SettingsPage"`
- Die Zeile `{(view === "profiles" || view === "settings") && <NotImplementedPage view={view} />}` ersetzen durch:

```tsx
{view === "profiles" && <NotImplementedPage view={view} />}
{view === "settings" && <SettingsPage />}
```

(„Profile" als Sidebar-Item bleibt vorerst NotImplemented — die Profile-Verwaltung leben in Settings/Profile-Tab. Das Sidebar-Item könnte man später umbenennen oder entfernen.)

- [ ] **Step 5: Build + Tests**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm vite build 2>&1 | tail -5
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test 2>&1 | tail -5
```

Erwartet: grün, 24 Frontend-Tests unverändert.

- [ ] **Step 6: Checkpoint**

> **Checkpoint 4b.1:** Sidebar → „Einstellungen" zeigt 4 leere Tabs. Die UI-Shell steht.

---

## Branchen-Editor

---

### Task 4b.2: `db/categories.rs` — CRUD-Erweiterung (TDD)

**Files:**
- Modify: `src-tauri/src/db/categories.rs` (add `list_all`, `create`, `update`, `delete`, `update_enabled`)

Zweck: `categories.rs` hat aktuell nur `list_enabled` + `list_by_ids`. Wir ergänzen die Mutations-Funktionen. Die Validierung (Name nicht leer, Weight 0–100) wird im Command gemacht — die DB-Funktionen vertrauen ihren Inputs (CHECK-Constraints in SQL fangen den Rest ab).

- [ ] **Step 1: Neue Funktionen + Tests**

Am Ende von `src-tauri/src/db/categories.rs`, **vor** dem `#[cfg(test)] mod tests`-Block, einfügen:

```rust
pub async fn list_all(pool: &SqlitePool) -> AppResult<Vec<Category>> {
    let rows: Vec<(i64, String, String, i64, i64, String)> = sqlx::query_as(
        "SELECT id, name_de, osm_tags, probability_weight, enabled, color FROM industry_categories ORDER BY sort_order, id"
    ).fetch_all(pool).await?;
    Ok(rows.into_iter().map(|(id,name_de,osm_tags,w,e,color)| Category{
        id, name_de, osm_tags, probability_weight: w, enabled: e!=0, color
    }).collect())
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewCategory {
    pub name_de: String,
    pub osm_tags: String,
    pub probability_weight: i64,
    pub color: String,
}

pub async fn create(pool: &SqlitePool, c: &NewCategory) -> AppResult<i64> {
    let next_sort: (i64,) = sqlx::query_as(
        "SELECT COALESCE(MAX(sort_order), 0) + 10 FROM industry_categories"
    ).fetch_one(pool).await?;
    let result = sqlx::query(
        "INSERT INTO industry_categories (name_de, osm_tags, probability_weight, enabled, color, sort_order)
         VALUES (?, ?, ?, 1, ?, ?)"
    )
    .bind(&c.name_de)
    .bind(&c.osm_tags)
    .bind(c.probability_weight)
    .bind(&c.color)
    .bind(next_sort.0)
    .execute(pool).await?;
    Ok(result.last_insert_rowid())
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateCategory {
    pub id: i64,
    pub name_de: String,
    pub osm_tags: String,
    pub probability_weight: i64,
    pub color: String,
}

pub async fn update(pool: &SqlitePool, c: &UpdateCategory) -> AppResult<()> {
    sqlx::query(
        "UPDATE industry_categories
         SET name_de = ?, osm_tags = ?, probability_weight = ?, color = ?
         WHERE id = ?"
    )
    .bind(&c.name_de)
    .bind(&c.osm_tags)
    .bind(c.probability_weight)
    .bind(&c.color)
    .bind(c.id)
    .execute(pool).await?;
    Ok(())
}

pub async fn update_enabled(pool: &SqlitePool, id: i64, enabled: bool) -> AppResult<()> {
    sqlx::query("UPDATE industry_categories SET enabled = ? WHERE id = ?")
        .bind(if enabled { 1 } else { 0 })
        .bind(id)
        .execute(pool).await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM industry_categories WHERE id = ?")
        .bind(id)
        .execute(pool).await?;
    Ok(())
}
```

- [ ] **Step 2: Tests erweitern**

Im bestehenden `#[cfg(test)] mod tests`-Block unten weitere Tests anhängen:

```rust
    #[tokio::test]
    async fn list_all_includes_disabled() {
        let pool = open_in_memory().await;
        let all = list_all(&pool).await.unwrap();
        assert_eq!(all.len(), 11); // alle Seeds
        assert!(all.iter().any(|c| !c.enabled)); // "Bürogebäude" ist disabled
    }

    #[tokio::test]
    async fn create_inserts_with_next_sort_order() {
        let pool = open_in_memory().await;
        let before = list_all(&pool).await.unwrap();
        let max_sort_before = before.iter().filter_map(|_| None::<i64>).max().unwrap_or(0);
        let _ = max_sort_before;
        let new_id = create(&pool, &NewCategory {
            name_de: "TestBranche".into(),
            osm_tags: r#"[{"shop":"computer"}]"#.into(),
            probability_weight: 50,
            color: "#123456".into(),
        }).await.unwrap();
        assert!(new_id > 11);
        let after = list_all(&pool).await.unwrap();
        assert_eq!(after.len(), 12);
        let inserted = after.iter().find(|c| c.id == new_id).unwrap();
        assert_eq!(inserted.name_de, "TestBranche");
        assert_eq!(inserted.probability_weight, 50);
        assert!(inserted.enabled);
    }

    #[tokio::test]
    async fn update_changes_fields() {
        let pool = open_in_memory().await;
        update(&pool, &UpdateCategory {
            id: 1,
            name_de: "Umbenannt".into(),
            osm_tags: r#"[{"industrial":"warehouse"}]"#.into(),
            probability_weight: 42,
            color: "#abcdef".into(),
        }).await.unwrap();
        let all = list_all(&pool).await.unwrap();
        let got = all.iter().find(|c| c.id == 1).unwrap();
        assert_eq!(got.name_de, "Umbenannt");
        assert_eq!(got.probability_weight, 42);
        assert_eq!(got.color, "#abcdef");
    }

    #[tokio::test]
    async fn update_enabled_toggles_flag() {
        let pool = open_in_memory().await;
        update_enabled(&pool, 1, false).await.unwrap();
        let all = list_all(&pool).await.unwrap();
        assert!(!all.iter().find(|c| c.id == 1).unwrap().enabled);
        update_enabled(&pool, 1, true).await.unwrap();
        let all = list_all(&pool).await.unwrap();
        assert!(all.iter().find(|c| c.id == 1).unwrap().enabled);
    }

    #[tokio::test]
    async fn delete_removes_row() {
        let pool = open_in_memory().await;
        delete(&pool, 1).await.unwrap();
        let all = list_all(&pool).await.unwrap();
        assert_eq!(all.len(), 10);
        assert!(all.iter().all(|c| c.id != 1));
    }
```

- [ ] **Step 3: Tests ausführen**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib db::categories 2>&1 | tail -10
```

Erwartet: 2 alte + 5 neue = **7 bestanden**.

- [ ] **Step 4: Ganze Suite**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib 2>&1 | tail -3
```

Erwartet: 57 + 5 = **62 Tests grün**.

- [ ] **Step 5: Checkpoint**

> **Checkpoint 4b.2:** Branchen-CRUD im Backend steht.

---

### Task 4b.3: Tauri-Commands für Branchen

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs` (Handler-Liste)

Zweck: 5 neue Commands (`list_all_categories`, `create_category`, `update_category`, `delete_category`, `set_category_enabled`). Input-Validierung erfolgt hier — DB-Schicht bleibt dumm.

- [ ] **Step 1: Commands hinzufügen**

Am Ende von `src-tauri/src/commands.rs`:

```rust
use crate::db::categories::{self, Category, NewCategory, UpdateCategory};

#[tauri::command]
pub async fn list_all_categories(state: State<'_, Arc<AppState>>) -> AppResult<Vec<Category>> {
    categories::list_all(&state.db).await
}

fn validate_category_input(name_de: &str, weight: i64, color: &str, osm_tags: &str) -> AppResult<()> {
    if name_de.trim().is_empty() {
        return Err(AppError::InvalidInput("Branchen-Name darf nicht leer sein".into()));
    }
    if !(0..=100).contains(&weight) {
        return Err(AppError::InvalidInput("Gewichtung muss zwischen 0 und 100 liegen".into()));
    }
    // Hex color: #RGB or #RRGGBB
    let c = color.trim_start_matches('#');
    let is_hex = c.len() == 3 || c.len() == 6;
    let all_hex = c.chars().all(|ch| ch.is_ascii_hexdigit());
    if !color.starts_with('#') || !is_hex || !all_hex {
        return Err(AppError::InvalidInput("Farbe muss Hex-Format #RGB oder #RRGGBB haben".into()));
    }
    // osm_tags: must parse as JSON array of objects
    let parsed: serde_json::Value = serde_json::from_str(osm_tags)
        .map_err(|e| AppError::InvalidInput(format!("osm_tags: {e}")))?;
    let arr = parsed.as_array()
        .ok_or_else(|| AppError::InvalidInput("osm_tags muss ein JSON-Array sein".into()))?;
    for item in arr {
        if !item.is_object() {
            return Err(AppError::InvalidInput("osm_tags: jedes Element muss ein Objekt sein".into()));
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn create_category(
    state: State<'_, Arc<AppState>>,
    payload: NewCategory,
) -> AppResult<i64> {
    validate_category_input(&payload.name_de, payload.probability_weight, &payload.color, &payload.osm_tags)?;
    let id = categories::create(&state.db, &payload).await?;
    tracing::info!(category_id = id, "category created");
    Ok(id)
}

#[tauri::command]
pub async fn update_category(
    state: State<'_, Arc<AppState>>,
    payload: UpdateCategory,
) -> AppResult<()> {
    validate_category_input(&payload.name_de, payload.probability_weight, &payload.color, &payload.osm_tags)?;
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
    tracing::info!(category_id = payload.id, enabled = payload.enabled, "category enabled toggled");
    Ok(())
}

#[tauri::command]
pub async fn delete_category(
    state: State<'_, Arc<AppState>>,
    id: i64,
) -> AppResult<()> {
    categories::delete(&state.db, id).await?;
    tracing::info!(category_id = id, "category deleted");
    Ok(())
}
```

- [ ] **Step 2: Handler registrieren (`lib.rs`)**

In der `tauri::generate_handler![...]`-Liste ergänzen (unabhängige Positionierung, nah zu `list_categories`):

```rust
commands::list_all_categories,
commands::create_category,
commands::update_category,
commands::set_category_enabled,
commands::delete_category,
```

- [ ] **Step 3: Build + Tests**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo build --lib 2>&1 | tail -5
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib 2>&1 | tail -3
```

Erwartet: build grün, 62 Tests weiterhin grün (Commands werden nicht separat getestet — die DB-Layer-Tests decken die Logik ab).

- [ ] **Step 4: Checkpoint**

> **Checkpoint 4b.3:** Branchen-Commands sind registriert und aufrufbar.

---

### Task 4b.4: Frontend-API + Typen (Branchen)

**Files:**
- Modify: `src/lib/tauri.ts`

Zweck: Neue Methoden und Typen exportieren.

- [ ] **Step 1: Typen erweitern**

Am Anfang, nah bei `CategoryRow`, hinzufügen:

```ts
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
```

- [ ] **Step 2: API-Methoden**

Im `api`-Objekt:

```ts
  listAllCategories: () => invoke<CategoryFull[]>("list_all_categories"),
  createCategory: (payload: NewCategoryInput) => invoke<number>("create_category", { payload }),
  updateCategory: (payload: UpdateCategoryInput) => invoke<void>("update_category", { payload }),
  setCategoryEnabled: (id: number, enabled: boolean) =>
    invoke<void>("set_category_enabled", { payload: { id, enabled } }),
  deleteCategory: (id: number) => invoke<void>("delete_category", { id }),
```

- [ ] **Step 3: Build + Tests**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm vite build 2>&1 | tail -5
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test 2>&1 | tail -5
```

Erwartet: grün, 24 Tests unverändert.

- [ ] **Step 4: Checkpoint**

> **Checkpoint 4b.4:** Frontend-API für Branchen bereit.

---

### Task 4b.5: Branchen-Tab + Edit-Dialog

**Files:**
- Modify: `src/components/settings/BranchenTab.tsx`
- Create: `src/components/settings/BranchenEditDialog.tsx`

Zweck: Liste aller Branchen inkl. disabled. Zeilen zeigen Farb-Swatch, Name, Gewichtung, Enabled-Toggle, Edit- und Lösch-Buttons. Ein „+ Neue Branche"-Button öffnet den Dialog im New-Modus. Edit-Dialog hat Name, OSM-Tags-Textarea (mit JSON-Live-Check), Weight-Slider, Color-Picker.

- [ ] **Step 1: `BranchenEditDialog` schreiben**

```tsx
// src/components/settings/BranchenEditDialog.tsx
import { useEffect, useState } from "react"
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Button } from "@/components/ui/button"
import { Textarea } from "@/components/ui/textarea"
import { Slider } from "@/components/ui/slider"
import { api, type CategoryFull } from "@/lib/tauri"
import { logger } from "@/lib/logger"

interface BranchenEditDialogProps {
  open: boolean
  onOpenChange: (o: boolean) => void
  /** null = neu anlegen, sonst bearbeiten */
  editing: CategoryFull | null
  onSaved: () => void
}

const DEFAULT_TAGS = `[{"shop":"wholesale"}]`

export function BranchenEditDialog({ open, onOpenChange, editing, onSaved }: BranchenEditDialogProps) {
  const [name, setName] = useState("")
  const [osmTags, setOsmTags] = useState(DEFAULT_TAGS)
  const [tagsError, setTagsError] = useState<string | null>(null)
  const [weight, setWeight] = useState(50)
  const [color, setColor] = useState("#3b82f6")
  const [busy, setBusy] = useState(false)
  const [err, setErr] = useState<string | null>(null)

  useEffect(() => {
    if (open) {
      setName(editing?.name_de ?? "")
      setOsmTags(editing?.osm_tags ?? DEFAULT_TAGS)
      setWeight(editing?.probability_weight ?? 50)
      setColor(editing?.color ?? "#3b82f6")
      setTagsError(null)
      setErr(null)
    }
  }, [open, editing])

  const validateTags = (v: string): string | null => {
    try {
      const parsed = JSON.parse(v)
      if (!Array.isArray(parsed)) return "Muss ein Array sein"
      if (parsed.some(x => typeof x !== "object" || Array.isArray(x) || x === null)) {
        return "Jedes Element muss ein Objekt sein"
      }
      return null
    } catch (e) {
      return String(e)
    }
  }

  const save = async () => {
    const tErr = validateTags(osmTags)
    if (tErr) { setTagsError(tErr); return }
    if (!name.trim()) { setErr("Name darf nicht leer sein"); return }
    setBusy(true); setErr(null)
    try {
      if (editing) {
        await api.updateCategory({ id: editing.id, name_de: name.trim(), osm_tags: osmTags, probability_weight: weight, color })
      } else {
        await api.createCategory({ name_de: name.trim(), osm_tags: osmTags, probability_weight: weight, color })
      }
      onSaved()
      onOpenChange(false)
    } catch (e) {
      setErr(String(e))
      logger.error("save category failed", { e: String(e) })
    } finally {
      setBusy(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>{editing ? "Branche bearbeiten" : "Neue Branche"}</DialogTitle>
        </DialogHeader>
        <div className="space-y-4">
          <div className="space-y-1">
            <Label htmlFor="bn">Name</Label>
            <Input id="bn" value={name} onChange={(e) => setName(e.target.value)} />
          </div>
          <div className="space-y-1">
            <Label htmlFor="bc">Pin-Farbe</Label>
            <div className="flex items-center gap-2">
              <input id="bc" type="color" value={color} onChange={(e) => setColor(e.target.value)} className="h-9 w-12 rounded border cursor-pointer" />
              <Input value={color} onChange={(e) => setColor(e.target.value)} className="flex-1" />
            </div>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between">
              <Label>Gewichtung</Label>
              <span className="text-sm font-medium tabular-nums">{weight}%</span>
            </div>
            <Slider value={[weight]} min={0} max={100} step={1} onValueChange={(v) => setWeight(v[0])} />
          </div>
          <div className="space-y-1">
            <Label htmlFor="bt">OSM-Tags (JSON-Array)</Label>
            <Textarea
              id="bt"
              value={osmTags}
              onChange={(e) => { setOsmTags(e.target.value); setTagsError(null) }}
              onBlur={() => setTagsError(validateTags(osmTags))}
              rows={4}
              className="font-mono text-xs"
            />
            <p className="text-xs text-muted-foreground">
              Beispiel: <code>[{`{`}"shop":"wholesale"{`}`}, {`{`}"industrial":"warehouse"{`}`}]</code> — Liste = OR, Objekt = AND.
            </p>
            {tagsError && <p className="text-xs text-red-600">{tagsError}</p>}
          </div>
          {err && <p className="text-sm text-red-600">{err}</p>}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={busy}>Abbrechen</Button>
          <Button onClick={save} disabled={busy || !!tagsError}>
            {busy ? "Speichere…" : editing ? "Speichern" : "Anlegen"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
```

- [ ] **Step 2: `BranchenTab` implementieren**

```tsx
// src/components/settings/BranchenTab.tsx
import { useEffect, useState } from "react"
import { api, type CategoryFull } from "@/lib/tauri"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { BranchenEditDialog } from "./BranchenEditDialog"
import { Pencil, Trash2, Plus } from "lucide-react"
import { logger } from "@/lib/logger"

export function BranchenTab() {
  const [cats, setCats] = useState<CategoryFull[]>([])
  const [loading, setLoading] = useState(true)
  const [dlgOpen, setDlgOpen] = useState(false)
  const [editing, setEditing] = useState<CategoryFull | null>(null)

  const reload = () => {
    setLoading(true)
    api.listAllCategories()
      .then(setCats)
      .catch(e => logger.error("listAllCategories failed", { e: String(e) }))
      .finally(() => setLoading(false))
  }
  useEffect(reload, [])

  const toggleEnabled = async (c: CategoryFull, next: boolean) => {
    await api.setCategoryEnabled(c.id, next)
    reload()
  }

  const remove = async (c: CategoryFull) => {
    if (!confirm(`Branche "${c.name_de}" wirklich löschen? Firmen mit dieser Branche bleiben, verlieren aber die Zuordnung.`)) return
    try {
      await api.deleteCategory(c.id)
      reload()
    } catch (e) {
      alert(`Löschen fehlgeschlagen: ${e}`)
      logger.error("deleteCategory failed", { e: String(e) })
    }
  }

  const openNew = () => { setEditing(null); setDlgOpen(true) }
  const openEdit = (c: CategoryFull) => { setEditing(c); setDlgOpen(true) }

  if (loading) return <div className="text-sm text-muted-foreground">Lade…</div>

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <p className="text-sm text-muted-foreground">
          {cats.length} Branchen · {cats.filter(c => c.enabled).length} aktiv
        </p>
        <Button size="sm" onClick={openNew}><Plus className="size-3 mr-1" />Neue Branche</Button>
      </div>
      <div className="border rounded-md divide-y">
        {cats.map(c => (
          <div key={c.id} className="flex items-center gap-3 px-3 py-2">
            <Checkbox checked={c.enabled} onCheckedChange={(v) => toggleEnabled(c, v === true)} />
            <span className="inline-block size-3 rounded-sm shrink-0" style={{ background: c.color }} />
            <span className="flex-1 text-sm truncate">{c.name_de}</span>
            <span className="text-xs text-muted-foreground tabular-nums w-10 text-right">{c.probability_weight}%</span>
            <Button variant="ghost" size="icon" onClick={() => openEdit(c)} aria-label="Bearbeiten"><Pencil className="size-4" /></Button>
            <Button variant="ghost" size="icon" onClick={() => remove(c)} aria-label="Löschen"><Trash2 className="size-4" /></Button>
          </div>
        ))}
      </div>
      <BranchenEditDialog
        open={dlgOpen}
        onOpenChange={setDlgOpen}
        editing={editing}
        onSaved={reload}
      />
    </div>
  )
}
```

- [ ] **Step 3: Build + Tests**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm vite build 2>&1 | tail -5
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test 2>&1 | tail -5
```

Erwartet: grün, 24 Tests unverändert.

- [ ] **Step 4: Checkpoint**

> **Checkpoint 4b.5:** Branchen-Tab vollständig — Liste, Enable-Toggle, Edit-Dialog mit JSON-Validation, Delete mit Confirm.

---

## Such-Profile-Verwaltung

---

### Task 4b.6: `db/search_profiles.rs` — CRUD (TDD)

**Files:**
- Create: `src-tauri/src/db/search_profiles.rs`
- Modify: `src-tauri/src/db/mod.rs` (`pub mod search_profiles;`)

Zweck: CRUD für die bereits existierende `search_profiles`-Tabelle. `enabled_category_ids` wird als JSON-String gespeichert (`[1,2,3]` etc.) — einfacher als eine Join-Tabelle für diesen Use-Case.

**Tabellen-Referenz (aus 0001_initial):**
```sql
CREATE TABLE search_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    center_label TEXT NOT NULL,
    center_lat REAL NOT NULL,
    center_lng REAL NOT NULL,
    radius_km INTEGER NOT NULL CHECK (radius_km BETWEEN 1 AND 300),
    enabled_category_ids TEXT NOT NULL,
    last_run_at TEXT,
    created_at TEXT NOT NULL
);
```

- [ ] **Step 1: Modul anlegen**

```rust
// src-tauri/src/db/search_profiles.rs
use crate::error::AppResult;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchProfile {
    pub id: i64,
    pub name: String,
    pub center_label: String,
    pub center_lat: f64,
    pub center_lng: f64,
    pub radius_km: i64,
    pub enabled_category_ids: String, // JSON-Array: "[1,2,3]"
    pub last_run_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewSearchProfile {
    pub name: String,
    pub center_label: String,
    pub center_lat: f64,
    pub center_lng: f64,
    pub radius_km: i64,
    pub enabled_category_ids: String,
}

pub async fn list_all(pool: &SqlitePool) -> AppResult<Vec<SearchProfile>> {
    let rows: Vec<(i64, String, String, f64, f64, i64, String, Option<String>, String)> =
        sqlx::query_as(
            "SELECT id, name, center_label, center_lat, center_lng, radius_km,
                    enabled_category_ids, last_run_at, created_at
             FROM search_profiles ORDER BY created_at DESC",
        )
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(|(id, name, cl, lat, lng, rk, ids, lr, ca)| SearchProfile {
        id, name, center_label: cl, center_lat: lat, center_lng: lng,
        radius_km: rk, enabled_category_ids: ids, last_run_at: lr, created_at: ca,
    }).collect())
}

pub async fn get(pool: &SqlitePool, id: i64) -> AppResult<Option<SearchProfile>> {
    let row: Option<(i64, String, String, f64, f64, i64, String, Option<String>, String)> =
        sqlx::query_as(
            "SELECT id, name, center_label, center_lat, center_lng, radius_km,
                    enabled_category_ids, last_run_at, created_at
             FROM search_profiles WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|(id, name, cl, lat, lng, rk, ids, lr, ca)| SearchProfile {
        id, name, center_label: cl, center_lat: lat, center_lng: lng,
        radius_km: rk, enabled_category_ids: ids, last_run_at: lr, created_at: ca,
    }))
}

pub async fn create(pool: &SqlitePool, p: &NewSearchProfile) -> AppResult<i64> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "INSERT INTO search_profiles
         (name, center_label, center_lat, center_lng, radius_km, enabled_category_ids, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&p.name)
    .bind(&p.center_label)
    .bind(p.center_lat)
    .bind(p.center_lng)
    .bind(p.radius_km)
    .bind(&p.enabled_category_ids)
    .bind(&now)
    .execute(pool)
    .await?;
    Ok(result.last_insert_rowid())
}

pub async fn rename(pool: &SqlitePool, id: i64, new_name: &str) -> AppResult<()> {
    sqlx::query("UPDATE search_profiles SET name = ? WHERE id = ?")
        .bind(new_name)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete(pool: &SqlitePool, id: i64) -> AppResult<()> {
    sqlx::query("DELETE FROM search_profiles WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn mark_run(pool: &SqlitePool, id: i64) -> AppResult<()> {
    let now = Utc::now().to_rfc3339();
    sqlx::query("UPDATE search_profiles SET last_run_at = ? WHERE id = ?")
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    fn sample() -> NewSearchProfile {
        NewSearchProfile {
            name: "Hannover 25km".into(),
            center_label: "Hannover, Deutschland".into(),
            center_lat: 52.37,
            center_lng: 9.73,
            radius_km: 25,
            enabled_category_ids: "[1,2,3]".into(),
        }
    }

    #[tokio::test]
    async fn create_then_list_returns_one() {
        let pool = open_in_memory().await;
        let id = create(&pool, &sample()).await.unwrap();
        assert!(id > 0);
        let all = list_all(&pool).await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "Hannover 25km");
        assert_eq!(all[0].radius_km, 25);
    }

    #[tokio::test]
    async fn get_returns_specific_row() {
        let pool = open_in_memory().await;
        let id = create(&pool, &sample()).await.unwrap();
        let got = get(&pool, id).await.unwrap().unwrap();
        assert_eq!(got.enabled_category_ids, "[1,2,3]");
        assert!((got.center_lat - 52.37).abs() < 1e-6);
    }

    #[tokio::test]
    async fn get_missing_returns_none() {
        let pool = open_in_memory().await;
        assert!(get(&pool, 999).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn rename_changes_name_only() {
        let pool = open_in_memory().await;
        let id = create(&pool, &sample()).await.unwrap();
        rename(&pool, id, "Hannover neu").await.unwrap();
        let got = get(&pool, id).await.unwrap().unwrap();
        assert_eq!(got.name, "Hannover neu");
        assert_eq!(got.radius_km, 25);
    }

    #[tokio::test]
    async fn delete_removes_row() {
        let pool = open_in_memory().await;
        let id = create(&pool, &sample()).await.unwrap();
        delete(&pool, id).await.unwrap();
        assert!(list_all(&pool).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn mark_run_sets_last_run_at() {
        let pool = open_in_memory().await;
        let id = create(&pool, &sample()).await.unwrap();
        let before = get(&pool, id).await.unwrap().unwrap();
        assert!(before.last_run_at.is_none());
        mark_run(&pool, id).await.unwrap();
        let after = get(&pool, id).await.unwrap().unwrap();
        assert!(after.last_run_at.is_some());
    }
}
```

- [ ] **Step 2: Registrieren**

In `src-tauri/src/db/mod.rs` ergänzen:

```rust
pub mod search_profiles;
```

- [ ] **Step 3: Tests**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib db::search_profiles 2>&1 | tail -10
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib 2>&1 | tail -3
```

Erwartet: 6 neue + 62 bestehende = **68 Tests grün**.

- [ ] **Step 4: Checkpoint**

> **Checkpoint 4b.6:** Profile-CRUD im Backend fertig.

---

### Task 4b.7: Tauri-Commands für Profile

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs` (Handler-Liste)

- [ ] **Step 1: Commands hinzufügen**

Am Ende von `commands.rs`:

```rust
use crate::db::search_profiles::{self, NewSearchProfile, SearchProfile};

#[tauri::command]
pub async fn list_search_profiles(state: State<'_, Arc<AppState>>) -> AppResult<Vec<SearchProfile>> {
    search_profiles::list_all(&state.db).await
}

#[tauri::command]
pub async fn create_search_profile(
    state: State<'_, Arc<AppState>>,
    payload: NewSearchProfile,
) -> AppResult<i64> {
    if payload.name.trim().is_empty() {
        return Err(AppError::InvalidInput("Profil-Name darf nicht leer sein".into()));
    }
    if !(1..=300).contains(&payload.radius_km) {
        return Err(AppError::InvalidInput("Radius muss zwischen 1 und 300 km liegen".into()));
    }
    // enabled_category_ids: muss parsebares JSON-Array von Integers sein
    let parsed: serde_json::Value = serde_json::from_str(&payload.enabled_category_ids)
        .map_err(|e| AppError::InvalidInput(format!("enabled_category_ids: {e}")))?;
    if !parsed.as_array().map(|a| a.iter().all(|v| v.is_i64() || v.is_u64())).unwrap_or(false) {
        return Err(AppError::InvalidInput("enabled_category_ids muss JSON-Array von Integers sein".into()));
    }
    let id = search_profiles::create(&state.db, &payload).await?;
    tracing::info!(profile_id = id, "profile created");
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
pub async fn delete_search_profile(
    state: State<'_, Arc<AppState>>,
    id: i64,
) -> AppResult<()> {
    search_profiles::delete(&state.db, id).await?;
    tracing::info!(profile_id = id, "profile deleted");
    Ok(())
}

#[tauri::command]
pub async fn mark_search_profile_run(
    state: State<'_, Arc<AppState>>,
    id: i64,
) -> AppResult<()> {
    search_profiles::mark_run(&state.db, id).await
}
```

- [ ] **Step 2: Handler registrieren (`lib.rs`)**

```rust
commands::list_search_profiles,
commands::create_search_profile,
commands::rename_search_profile,
commands::delete_search_profile,
commands::mark_search_profile_run,
```

- [ ] **Step 3: Build + Tests**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo build --lib 2>&1 | tail -5
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib 2>&1 | tail -3
```

Erwartet: 68 Tests grün.

- [ ] **Step 4: Checkpoint**

> **Checkpoint 4b.7:** Profile-Commands registriert.

---

### Task 4b.8: Frontend-API + Profile-Tab-UI

**Files:**
- Modify: `src/lib/tauri.ts`
- Modify: `src/components/settings/ProfileTab.tsx`

- [ ] **Step 1: API-Typen + Methoden in `tauri.ts`**

```ts
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
```

Im `api`-Objekt:

```ts
  listSearchProfiles: () => invoke<SearchProfile[]>("list_search_profiles"),
  createSearchProfile: (payload: NewSearchProfileInput) =>
    invoke<number>("create_search_profile", { payload }),
  renameSearchProfile: (id: number, new_name: string) =>
    invoke<void>("rename_search_profile", { payload: { id, new_name } }),
  deleteSearchProfile: (id: number) => invoke<void>("delete_search_profile", { id }),
  markSearchProfileRun: (id: number) => invoke<void>("mark_search_profile_run", { id }),
```

- [ ] **Step 2: `ProfileTab` implementieren**

```tsx
// src/components/settings/ProfileTab.tsx
import { useEffect, useState } from "react"
import { api, type SearchProfile } from "@/lib/tauri"
import { Button } from "@/components/ui/button"
import { useUiStore } from "@/stores/uiStore"
import { formatDateDe } from "@/lib/format"
import { Pencil, Trash2, Play } from "lucide-react"
import { logger } from "@/lib/logger"

export function ProfileTab() {
  const [profiles, setProfiles] = useState<SearchProfile[]>([])
  const [loading, setLoading] = useState(true)
  const setView = useUiStore(s => s.setView)

  const reload = () => {
    setLoading(true)
    api.listSearchProfiles()
      .then(setProfiles)
      .catch(e => logger.error("listSearchProfiles failed", { e: String(e) }))
      .finally(() => setLoading(false))
  }
  useEffect(reload, [])

  const rename = async (p: SearchProfile) => {
    const next = prompt("Neuer Name:", p.name)
    if (!next || next.trim() === p.name) return
    try {
      await api.renameSearchProfile(p.id, next.trim())
      reload()
    } catch (e) { alert(`Umbenennen fehlgeschlagen: ${e}`) }
  }

  const remove = async (p: SearchProfile) => {
    if (!confirm(`Profil "${p.name}" wirklich löschen?`)) return
    try {
      await api.deleteSearchProfile(p.id)
      reload()
    } catch (e) { alert(`Löschen fehlgeschlagen: ${e}`) }
  }

  const load = (p: SearchProfile) => {
    sessionStorage.setItem("loadProfile", JSON.stringify(p))
    setView("search")
  }

  if (loading) return <div className="text-sm text-muted-foreground">Lade…</div>

  if (profiles.length === 0) {
    return (
      <div className="text-sm text-muted-foreground">
        Noch keine Profile gespeichert. In der Ansicht „Neue Suche" kannst du eine Konfiguration als Profil sichern.
      </div>
    )
  }

  return (
    <div className="space-y-3">
      <p className="text-sm text-muted-foreground">{profiles.length} gespeicherte Profile</p>
      <div className="border rounded-md divide-y">
        {profiles.map(p => (
          <div key={p.id} className="flex items-center gap-3 px-3 py-3">
            <div className="min-w-0 flex-1">
              <div className="font-medium truncate">{p.name}</div>
              <div className="text-xs text-muted-foreground truncate">
                {p.center_label} · {p.radius_km} km
                {p.last_run_at && ` · zuletzt ${formatDateDe(p.last_run_at)}`}
              </div>
            </div>
            <Button variant="outline" size="sm" onClick={() => load(p)} title="Profil laden und Suche starten">
              <Play className="size-3 mr-1" />Laden
            </Button>
            <Button variant="ghost" size="icon" onClick={() => rename(p)} aria-label="Umbenennen"><Pencil className="size-4" /></Button>
            <Button variant="ghost" size="icon" onClick={() => remove(p)} aria-label="Löschen"><Trash2 className="size-4" /></Button>
          </div>
        ))}
      </div>
    </div>
  )
}
```

Der „Laden"-Button speichert das Profil in `sessionStorage` und wechselt zur Neue-Suche-Ansicht — die sich das Profil in Task 4b.9 schnappt.

- [ ] **Step 3: Build + Tests**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm vite build 2>&1 | tail -5
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test 2>&1 | tail -5
```

Erwartet: grün.

- [ ] **Step 4: Checkpoint**

> **Checkpoint 4b.8:** Profile-Tab zeigt Liste, Laden schiebt an NewSearchPage, Rename + Delete funktionieren.

---

### Task 4b.9: „Als Profil speichern" + Profil-Laden in `NewSearchPage`

**Files:**
- Modify: `src/pages/NewSearchPage.tsx`

Zweck: Einen „Speichern als Profil"-Button neben „Suche starten". Beim Seiten-Load aus Settings-Profile-Tab wird das Profil aus `sessionStorage` gelesen und das Formular gefüllt.

- [ ] **Step 1: Datei lesen**

`src/pages/NewSearchPage.tsx` ist aktuell 128 Zeilen. Lies sie als Referenz — die Edits passen 1:1 darauf.

- [ ] **Step 2: Profile-Load aus sessionStorage**

In der bestehenden `useEffect(…)`-Mount-Funktion (die Kategorien + Progress-Listener setzt) am Ende ergänzen — nach `api.listCategories().then(...).catch(...)` und den `onSearchProgress/onSearchDone`-Listen-ups — folgenden Block einfügen:

```tsx
    // Profil aus SettingsPage übernehmen, falls vorhanden
    const raw = sessionStorage.getItem("loadProfile")
    if (raw) {
      sessionStorage.removeItem("loadProfile")
      try {
        const p = JSON.parse(raw) as {
          center_lat: number; center_lng: number; center_label: string;
          radius_km: number; enabled_category_ids: string;
        }
        setCenter({ lat: p.center_lat, lng: p.center_lng })
        setCenterLabel(p.center_label)
        setRadiusKm(p.radius_km)
        const ids: number[] = JSON.parse(p.enabled_category_ids)
        setSelectedCats(new Set(ids))
        logger.info("profile loaded", { radius_km: p.radius_km, cats: ids.length })
      } catch (e) {
        logger.error("profile load failed", { e: String(e) })
      }
    }
```

⚠️ **Reihenfolge-Detail:** `api.listCategories(...)` setzt `selectedCats` auf „alle enabled" in seinem Then-Handler. Wenn das Profil geladen wird, wollen wir diese Default-Auswahl überschreiben. Der SessionStorage-Block muss **nach** `api.listCategories(...)` laufen — aber weil `setSelectedCats` asynchron ist, überschreiben wir das Ergebnis des Category-Loads zuverlässig, wenn der SessionStorage-Read **im gleichen Effect-Body** nach dem `.then()`-Chain liegt. Am saubersten: Profil-Load in eine separate Kette hängen, die `.then()` des `listCategories` mitbenutzt, damit die Reihenfolge garantiert ist:

```tsx
// Ersetze die bestehende listCategories-Zeile
api.listCategories()
  .then(all => {
    setCats(all)
    const raw = sessionStorage.getItem("loadProfile")
    if (raw) {
      sessionStorage.removeItem("loadProfile")
      try {
        const p = JSON.parse(raw) as {
          center_lat: number; center_lng: number; center_label: string;
          radius_km: number; enabled_category_ids: string;
        }
        setCenter({ lat: p.center_lat, lng: p.center_lng })
        setCenterLabel(p.center_label)
        setRadiusKm(p.radius_km)
        const ids: number[] = JSON.parse(p.enabled_category_ids)
        setSelectedCats(new Set(ids))
        logger.info("profile loaded", { radius_km: p.radius_km, cats: ids.length })
      } catch (e) {
        logger.error("profile load failed", { e: String(e) })
        setSelectedCats(new Set(all.filter(c => c.enabled).map(c => c.id)))
      }
    } else {
      setSelectedCats(new Set(all.filter(c => c.enabled).map(c => c.id)))
    }
  })
  .catch(e => logger.error("listCategories failed", { e: String(e) }))
```

- [ ] **Step 3: „Als Profil speichern"-Button**

Oberhalb (oder direkt über) dem bestehenden `<Button onClick={runSearch} ...>` einen neuen Button einfügen, plus einen Handler:

```tsx
  const saveAsProfile = async () => {
    if (!center) return
    const name = prompt("Name für das Profil:", centerLabel ?? `${center.lat.toFixed(3)}, ${center.lng.toFixed(3)}`)
    if (!name || !name.trim()) return
    try {
      await api.createSearchProfile({
        name: name.trim(),
        center_label: centerLabel ?? `${center.lat.toFixed(3)}, ${center.lng.toFixed(3)}`,
        center_lat: center.lat,
        center_lng: center.lng,
        radius_km: radiusKm,
        enabled_category_ids: JSON.stringify(Array.from(selectedCats)),
      })
      alert("Profil gespeichert.")
      logger.info("profile saved", { radius_km: radiusKm, cats: selectedCats.size })
    } catch (e) {
      alert(`Speichern fehlgeschlagen: ${e}`)
      logger.error("saveAsProfile failed", { e: String(e) })
    }
  }
```

Im JSX, neben dem bestehenden `<Button onClick={runSearch} disabled={!canStart} className="w-full">…</Button>`:

```tsx
          <div className="flex gap-2">
            <Button onClick={runSearch} disabled={!canStart} className="flex-1">
              {busy ? "Suche läuft…" : "Suche starten"}
            </Button>
            <Button variant="outline" onClick={saveAsProfile} disabled={!center || selectedCats.size === 0}>
              Speichern
            </Button>
          </div>
```

(Das ersetzt den bestehenden einzelnen `<Button>...</Button>`. Das `className="w-full"` verschwindet, dafür nutzen die beiden Buttons nun `flex gap-2` + `flex-1` auf dem Primär-Button.)

- [ ] **Step 4: Build + Tests**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm vite build 2>&1 | tail -5
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test 2>&1 | tail -5
```

Erwartet: grün.

- [ ] **Step 5: Checkpoint**

> **Checkpoint 4b.9:** Profile können gespeichert und geladen werden. Einwand-Flow: Settings → Profile laden → NewSearchPage füllt sich → User klickt Suche starten.

---

## Daten-Tab (Backup/Restore/Öffnen)

---

### Task 4b.10: `backup.rs` — pure Helper (TDD)

**Files:**
- Create: `src-tauri/src/backup.rs`
- Modify: `src-tauri/src/lib.rs` (`pub mod backup;`)

Zweck: Die pure Pfad-Mathematik (Snapshot-Ordner, Backup-Dateiname) auslagern und testen. Die eigentlichen I/O-Calls passieren in den Commands (Task 4b.11), weil sie `AppHandle` brauchen.

- [ ] **Step 1: Datei schreiben**

```rust
// src-tauri/src/backup.rs
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
```

- [ ] **Step 2: In `lib.rs` registrieren**

Ergänze oben bei den Modul-Deklarationen:

```rust
pub mod backup;
```

- [ ] **Step 3: Tests**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib backup 2>&1 | tail -5
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib 2>&1 | tail -3
```

Erwartet: 3 neue + 68 = **71 Tests grün**.

- [ ] **Step 4: Checkpoint**

> **Checkpoint 4b.10:** Pure Pfad-Helfer stehen.

---

### Task 4b.11: Tauri-Commands für Backup/Restore/Open-Data-Dir

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs` (plugins + handler)
- Modify: `src-tauri/Cargo.toml` — evtl. plugin-Features

Zweck: Drei Commands: `backup_db` (Save-Dialog → Copy), `restore_db` (Open-Dialog → Snapshot + Replace → App-Restart), `open_data_dir` (im Finder/Explorer anzeigen).

⚠️ **Plugin-Init + Capabilities:** `tauri-plugin-dialog` ist in Cargo.toml, aber in `lib.rs::run()` NICHT initialisiert und nicht in den Capabilities freigeschaltet. Beides muss dazu.

`tauri-plugin-fs` brauchen wir **nicht** — Backup/Restore nutzt direkt `std::fs::copy`, kein Tauri-FS-Plugin.

- [ ] **Step 1a: `lib.rs` — Dialog-Plugin initialisieren**

In der `tauri::Builder::default()`-Chain (inside `pub fn run()`) neben dem bestehenden `.plugin(tauri_plugin_opener::init())` einfügen:

```rust
.plugin(tauri_plugin_dialog::init())
```

- [ ] **Step 1b: Capabilities freischalten**

`src-tauri/capabilities/default.json` hat aktuell `"permissions": ["core:default", "opener:default"]`. Ergänze `"dialog:default"`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default",
    "dialog:default"
  ]
}
```

Ohne diesen Eintrag scheitern `blocking_save_file`/`blocking_pick_file` zur Laufzeit mit Permission-Denied.

- [ ] **Step 2: Commands in `commands.rs`**

Am Ende der Datei:

```rust
use crate::backup;
use std::path::PathBuf;
use tauri_plugin_dialog::DialogExt;

// WICHTIG: muss exakt die gleiche Logik ergeben wie lib.rs::run().
// run() benutzt dirs::data_dir().join("projektalpha"). Wir replizieren das hier,
// weil Tauri's app_data_dir() einen Bundle-ID-spezifischen Pfad liefert und nicht
// mit unserem hardcoded "projektalpha"-Ordner übereinstimmt.
fn data_dir() -> PathBuf {
    dirs::data_dir()
        .map(|p| p.join("projektalpha"))
        .unwrap_or_else(|| PathBuf::from("./projektalpha"))
}

fn db_path() -> PathBuf {
    data_dir().join("data.db")
}

// Backup/Restore sind synchron: sie blockieren auf den File-Dialog und machen
// danach `std::fs::copy`. `#[tauri::command] pub fn` läuft auf Tauri's dediziertem
// Thread-Pool, damit blockieren wir KEINEN tokio-Worker. Für async commands würde
// `blocking_save_file` einen tokio-Worker parken — Anti-Pattern.
#[tauri::command]
pub fn backup_db(app: AppHandle) -> AppResult<Option<String>> {
    let src = db_path();
    if !src.exists() {
        return Err(AppError::Internal("data.db existiert nicht".into()));
    }
    let suggested = backup::backup_suggested_filename_now();
    // blocking_save_file läuft synchron; für Command-async ist das ok (MainThread blockiert kurz).
    let picked = app.dialog()
        .file()
        .set_file_name(&suggested)
        .add_filter("SQLite-Datenbank", &["db"])
        .blocking_save_file();
    let Some(target) = picked else {
        return Ok(None); // User hat abgebrochen
    };
    let target_path: PathBuf = target.into_path()
        .map_err(|e| AppError::Internal(format!("dialog path: {e}")))?;
    std::fs::copy(&src, &target_path)
        .map_err(|e| AppError::Io(e))?;
    let s = target_path.to_string_lossy().to_string();
    tracing::info!(target_len = s.len(), "backup written");
    Ok(Some(s))
}

#[tauri::command]
pub fn restore_db(app: AppHandle) -> AppResult<bool> {
    let picked = app.dialog()
        .file()
        .add_filter("SQLite-Datenbank", &["db"])
        .blocking_pick_file();
    let Some(source) = picked else {
        return Ok(false); // User abgebrochen
    };
    let source_path: PathBuf = source.into_path()
        .map_err(|e| AppError::Internal(format!("dialog path: {e}")))?;

    let dst = db_path();
    let snap_dir = backup::snapshot_dir(&data_dir());
    std::fs::create_dir_all(&snap_dir).map_err(AppError::Io)?;
    let snap_path = snap_dir.join(backup::snapshot_filename_now());

    if dst.exists() {
        std::fs::copy(&dst, &snap_path).map_err(AppError::Io)?;
        tracing::info!(snap_len = snap_path.to_string_lossy().len(), "pre-restore snapshot written");
    }
    std::fs::copy(&source_path, &dst).map_err(AppError::Io)?;
    tracing::info!("restore complete — restarting app");

    // app.restart() hat Rückgabetyp `-> !` (diverging, Tauri 2.10+). Kein Ok(...) nötig
    // danach — der `!`-Typ koerziert zu jedem Rückgabetyp.
    app.restart();
}

#[tauri::command]
pub fn open_data_dir() -> AppResult<()> {
    let p = data_dir();
    tauri_plugin_opener::open_path(&p, None::<&str>)
        .map_err(|e| AppError::Internal(format!("open_path: {e}")))?;
    tracing::info!("data dir opened");
    Ok(())
}

#[tauri::command]
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
```

- [ ] **Step 3: Handler registrieren**

In `lib.rs` (invoke_handler-Liste):

```rust
commands::backup_db,
commands::restore_db,
commands::open_data_dir,
commands::app_version,
```

- [ ] **Step 4: Build + Tests**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo build --lib 2>&1 | tail -10
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib 2>&1 | tail -3
```

Erwartet: build grün, 71 Tests grün.

- [ ] **Step 5: Checkpoint**

> **Checkpoint 4b.11:** Backup/Restore/Open-Commands hängen am Tauri-Builder und sind aufrufbar.

---

### Task 4b.12: Daten-Tab + Über-Tab

**Files:**
- Modify: `src/lib/tauri.ts`
- Modify: `src/components/settings/DatenTab.tsx`
- Modify: `src/components/settings/UeberTab.tsx`

Zweck: Frontend-Buttons für Backup/Restore/Datenordner öffnen + statischer Über-Tab.

- [ ] **Step 1: API-Erweiterungen in `tauri.ts`**

```ts
  backupDb: () => invoke<string | null>("backup_db"),
  restoreDb: () => invoke<boolean>("restore_db"),
  openDataDir: () => invoke<void>("open_data_dir"),
  appVersion: () => invoke<string>("app_version"),
```

- [ ] **Step 2: `DatenTab.tsx`**

```tsx
// src/components/settings/DatenTab.tsx
import { useState } from "react"
import { api } from "@/lib/tauri"
import { Button } from "@/components/ui/button"
import { Download, Upload, FolderOpen } from "lucide-react"
import { logger } from "@/lib/logger"

export function DatenTab() {
  const [busy, setBusy] = useState<"backup" | "restore" | null>(null)
  const [msg, setMsg] = useState<string | null>(null)

  const doBackup = async () => {
    setBusy("backup"); setMsg(null)
    try {
      const target = await api.backupDb()
      if (target) {
        setMsg(`Backup gespeichert: ${target}`)
        logger.info("backup done", { target_len: target.length })
      } else {
        setMsg("Backup abgebrochen.")
      }
    } catch (e) {
      setMsg(`Backup fehlgeschlagen: ${e}`)
      logger.error("backup failed", { e: String(e) })
    } finally { setBusy(null) }
  }

  const doRestore = async () => {
    if (!confirm("Restore ersetzt die aktuelle Datenbank vollständig. Ein Pre-Restore-Snapshot wird automatisch angelegt. Fortfahren?")) return
    setBusy("restore"); setMsg(null)
    try {
      const ok = await api.restoreDb()
      if (!ok) setMsg("Restore abgebrochen.")
      // Bei ok=true restartet die App — dieser Zweig wird gar nicht erreicht.
    } catch (e) {
      setMsg(`Restore fehlgeschlagen: ${e}`)
      logger.error("restore failed", { e: String(e) })
    } finally { setBusy(null) }
  }

  const openDir = async () => {
    try { await api.openDataDir() }
    catch (e) { alert(`Ordner konnte nicht geöffnet werden: ${e}`) }
  }

  return (
    <div className="space-y-4 max-w-xl">
      <section className="space-y-2">
        <h3 className="text-sm font-semibold">Backup</h3>
        <p className="text-sm text-muted-foreground">
          Erzeugt eine Kopie der gesamten Datenbank an einem Ort deiner Wahl.
          SQLite-Format — kann später per Restore zurückgespielt werden.
        </p>
        <Button onClick={doBackup} disabled={busy !== null}>
          <Download className="size-4 mr-2" />{busy === "backup" ? "Sichere…" : "Backup erstellen"}
        </Button>
      </section>

      <section className="space-y-2 pt-4 border-t">
        <h3 className="text-sm font-semibold">Restore</h3>
        <p className="text-sm text-muted-foreground">
          Ersetzt die aktuelle Datenbank durch eine vorher gespeicherte Backup-Datei.
          Die App startet automatisch neu. Vor dem Ersetzen wird ein Pre-Restore-Snapshot
          im Backups-Ordner abgelegt.
        </p>
        <Button variant="outline" onClick={doRestore} disabled={busy !== null}>
          <Upload className="size-4 mr-2" />{busy === "restore" ? "Spiele zurück…" : "Restore starten"}
        </Button>
      </section>

      <section className="space-y-2 pt-4 border-t">
        <h3 className="text-sm font-semibold">Datenordner</h3>
        <p className="text-sm text-muted-foreground">
          Öffnet den Ordner, in dem ProjektAlpha Datenbank, Logs und Snapshots ablegt.
        </p>
        <Button variant="outline" onClick={openDir}>
          <FolderOpen className="size-4 mr-2" />Ordner öffnen
        </Button>
      </section>

      {msg && <p className="text-sm">{msg}</p>}
    </div>
  )
}
```

- [ ] **Step 3: `UeberTab.tsx`**

```tsx
// src/components/settings/UeberTab.tsx
import { useEffect, useState } from "react"
import { api } from "@/lib/tauri"

export function UeberTab() {
  const [version, setVersion] = useState<string>("…")
  useEffect(() => {
    api.appVersion().then(setVersion).catch(() => setVersion("unbekannt"))
  }, [])
  return (
    <div className="space-y-3 max-w-xl">
      <h3 className="text-base font-semibold">ProjektAlpha</h3>
      <p className="text-sm">Version <span className="font-mono">{version}</span></p>
      <p className="text-sm text-muted-foreground">
        Lokales Lead-Management für Industrie-Tore, Verlade- und Hubbühnen, UVV-Prüfungen.
        Alle Daten bleiben auf diesem Rechner — keine Cloud, keine Telemetrie.
      </p>
      <p className="text-xs text-muted-foreground pt-2 border-t">
        Verwendet OpenStreetMap (Overpass, Nominatim, Tile-Server) unter ODbL-Lizenz.
      </p>
    </div>
  )
}
```

- [ ] **Step 4: Build + Tests**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm vite build 2>&1 | tail -5
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test 2>&1 | tail -5
```

Erwartet: grün, 24 Tests weiterhin.

- [ ] **Step 5: Checkpoint**

> **Checkpoint 4b.12:** Daten-Tab + Über-Tab voll funktional. Phase 4 Implementation-Umfang ist damit komplett.

---

### Task 4b.13: Logging- und PII-Review

**Files:** keine (Verifikation)

- [ ] **Step 1: Neue Log-Punkte durchgehen**

```bash
grep -rn "logger\." src/components/settings src/pages/SettingsPage.tsx 2>&1
grep -rn "tracing::" src-tauri/src/backup.rs 2>&1
grep -n "tracing::" src-tauri/src/commands.rs | tail -30
```

Erlaubt (Neu in Plan 4b):
- `logger.info("backup done", { target_len })` — Pfad-Länge, kein Pfad
- `logger.error("backup/restore failed", { e })` — OK
- `logger.info("profile loaded/saved", { radius_km, cats })` — Zahlen
- `tracing::info!(category_id | profile_id | target_len, "…")` — IDs / Längen
- `tracing::info!("restore complete — restarting app")` — ok

**Nicht erlaubt:**
- Profilnamen, Branchennamen, OSM-Tag-Inhalte, vollständige Pfade in Logs

Falls irgendwo ein Name oder ein Pfad-String im Log landet → durch Länge oder ID ersetzen.

- [ ] **Step 2: Checkpoint**

> **Checkpoint 4b.13:** PII-/Namen-Audit clean.

---

### Task 4b.14: Live-Smoke-Test (aufgeschoben)

**Files:** keine

User testet gemeinsam mit Plan 3 + Plan 4a am Ende der gesamten Phase-4-Arbeiten. Acceptance-Kriterien für Plan 4b:

**Settings-Shell:**
1. Sidebar → „Einstellungen" zeigt 4 Tabs → ✓
2. Tab-Wechsel funktioniert, keine Flicker → ✓

**Branchen-Tab:**
3. Liste zeigt alle 11 Seeds inkl. disabled → ✓
4. Enable-Toggle auf „Bürogebäude" → wird aktiv → beim nächsten Suche erscheint Branche → ✓
5. „+ Neue Branche" → Dialog öffnet → ohne Name Speichern ist disabled → ✓
6. Mit ungültigem OSM-Tags-JSON → roter Hinweis, Speichern disabled → ✓
7. Neue Branche anlegen (z. B. „Testbranche", `[{"shop":"computer"}]`, 60%, grüne Pin-Farbe) → erscheint in Liste → ✓
8. Edit → Name und Gewichtung ändern → gespeichert → ✓
9. Delete → Confirm → verschwindet → ✓

**Profile-Tab:**
10. Initial leer → Hinweis-Text → ✓
11. In „Neue Suche" Mittelpunkt setzen + „Speichern" klicken → Name eingeben → Profil erscheint in Settings → ✓
12. „Laden"-Button → NewSearchPage öffnet sich mit vorgefülltem Center/Radius/Branchen → ✓
13. Umbenennen via Prompt funktioniert → ✓
14. Löschen via Confirm funktioniert → ✓

**Daten-Tab:**
15. „Backup erstellen" → Dateidialog → Datei an beliebigem Ort speichern → Erfolgsmeldung → Datei existiert auf Disk → ✓
16. „Ordner öffnen" → Finder/Explorer öffnet `~/Library/Application Support/projektalpha` → ✓
17. Vorheriges Backup via Restore wieder einspielen → Confirm → App startet neu → nach Restart sind die ursprünglichen Daten da → ✓
18. In `~/Library/Application Support/projektalpha/backups/` liegt ein `pre-restore-*.db` → ✓

**Über-Tab:**
19. Zeigt Versionsnummer (aus Cargo.toml) → ✓

**Logs:**
```bash
tail -100 ~/Library/Application\ Support/projektalpha/logs/projektalpha.log.*
```
Erwartet: `category created/updated/deleted`, `profile created/deleted`, `backup/restore` ohne Pfade/Namen, nur IDs + Längen. Keine PII.

- [ ] **Step 1: Checkpoint**

> **Checkpoint 4b.14 = Plan 4b fertig.** Implementation. Smoke-Test gemeinsam mit Plan 3 + 4a am Ende.

---

## Was am Ende dieses Plans funktioniert

- ✅ Settings-UI mit vier Tabs als echte Sidebar-Ziel-Seite
- ✅ Branchen-Editor: CRUD, Enable-Toggle, JSON-Validation, Farbpicker, Gewichtungs-Slider
- ✅ Such-Profile: Create aus NewSearchPage, List/Rename/Delete in Settings, Laden zurück in NewSearchPage
- ✅ Backup/Restore zu/aus beliebiger Datei, Pre-Restore-Auto-Snapshot, Post-Restore-App-Restart
- ✅ Datenordner im Finder/Explorer öffnen
- ✅ Über-Tab mit Versionsnummer
- ✅ Rust-Test-Suite gewachsen: 57 → 71 (5 category CRUD + 6 search_profiles + 3 backup)
- ✅ Frontend-Test-Suite bleibt bei 24 (neue UI ist reine Glue-Arbeit, Unit-getestete Logik ist im Backend)
- ✅ Keine PII in Logs

## Was bewusst NICHT in diesem Plan ist

- **Dashboard mit KPIs + Today-Liste** → Plan 5
- **Auto-Updater + GitHub Actions Cross-Build (Mac-DMG + Windows-MSI)** → Plan 6
- **Profil-Scheduling** (automatisch wöchentlich laufen lassen) → YAGNI, ggf. Plan 6
- **Strukturierter OSM-Tags-Editor** (statt Textarea) → YAGNI für Single-User
- **Update-Check im Über-Tab** → Plan 6 mit Auto-Updater
- **Sidebar-Item „Profile" konsolidieren** (aktuell ist der Menüpunkt „Profile" separat, aber NotImplemented — die Profile-Verwaltung ist in Settings). Entscheidung: lassen, weil User evtl. später eine reine Profile-Startseite möchte.
- **Export/Import-Formate jenseits Raw-SQLite** (CSV, Excel) → später Polish
- **shadcn AlertDialog statt nativer `alert`/`confirm`/`prompt`** — bewusst MVP-Kompromiss. Für den Single-User-Desktop-Use-Case funktional OK; für Plan 6 Polish eingeplant, vor allem der Restore-Confirm verdient eine richtige AlertDialog.

---

## Nächster Plan

**Plan 5 — Dashboard mit KPIs + „Heute fällig"-Liste + Letzte-Aktivität-Timeline**, gefolgt von **Plan 6 — Auto-Updater + GitHub Actions Cross-Build + Code-Signing-Entscheidung**.
