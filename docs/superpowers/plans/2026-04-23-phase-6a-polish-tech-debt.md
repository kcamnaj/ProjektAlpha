# Phase 6A: Polish + Tech-Debt Cleanup — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Über die Plan-3/4/5-Arbeit hinweg haben sich sechs kleine UX- und Code-Debt-Punkte angesammelt. Plan 6A räumt sie in einem Rutsch auf: (1) Karte fliegt bei Adress-Pick sauber zum neuen Mittelpunkt, (2) die vier destruktiven Bestätigungsdialoge (Restore / Firma löschen / Profil löschen / Branche löschen) laufen jetzt über eine echte shadcn `AlertDialog`-Komponente statt native `confirm()`, (3) `data_dir()`-Duplikation im Backend verschwindet in `AppState`, (4) der orphane Sidebar-Eintrag „Profile" wird entfernt (Profile sind längst in Settings → Profile), (5) inkonsistente „Lade…"-Texte und leere Zustände bekommen einheitliche `<LoadingState>` / `<EmptyState>`-Komponenten. Nach diesem Plan ist die App UX-seitig konsistent und die bekannte Tech-Debt-Liste leer — bereit für Plan 6B (Release-Infrastruktur).

**Architecture:** Reine Frontend-Arbeit plus eine kleine Backend-Refaktorierung. Neue wiederverwendbare Komponenten: `<ConfirmDialog>` (shadcn-AlertDialog-Wrapper mit `destructive`-Flag, imperative State-API via `useState` im Consumer), `<LoadingState>` (Icon + Text), `<EmptyState>` (großes Icon + Message + optional Action). `AppState` bekommt ein `pub data_dir: PathBuf`-Feld; `commands.rs::data_dir()` und `db_path()` werden zu Methoden auf `AppState` bzw. verwenden das Feld direkt. CenterPickerMap erhält eine imperative `flyTo`-Logik im bestehenden `useEffect(… , [center, radiusKm])`-Block — kein neuer Prop, kein forwardRef.

**Scope-Begrenzung (wichtig):** Plan 6A migriert NUR die **vier destruktiven Confirms**. Die 8 verbleibenden `alert()`- und 2 `prompt()`-Aufrufe (Fehlermeldungen und Umbenennen-Dialoge) bleiben bewusst bestehen — sie sind nicht destruktiv, kosten wenig UX-Schaden und eine separate Prompt-UI wäre ein eigenes YAGNI-Thema. Flagge im „Was bewusst NICHT in diesem Plan ist"-Abschnitt dokumentiert.

**Tech Stack:** React 19 (shadcn AlertDialog — neue Dep via `pnpm dlx shadcn@latest add alert-dialog`), lucide-react Icons, Tailwind. Rust (reine Struct-Erweiterung, keine neuen Crates). Tests: Vitest für die 2 neuen UI-Komponenten, kein neuer Rust-Test.

**Spec-Referenz:** `docs/superpowers/specs/2026-04-21-firmensuche-design.md`
- §7 (UI-Konzept Design-Sprache Linear/Vercel/Raycast) · §14 Phase 6 („Empty-/Loading-States, Animations, Polish")

**Accumulated Tech-Debt-Quellen:**
- Plan 3 Bekannte Einschränkungen: CenterPickerMap flyTo fehlt
- Plan 4b „Was bewusst NICHT in diesem Plan ist": native `alert/confirm/prompt`, `data_dir()`-Duplikation, Sidebar-„Profile"-Konsolidierung
- Plan 5 Bekannte Einschränkungen: dieselbe Liste, erneut bestätigt

**CLAUDE.md-Prinzipien:**
- **UX zuerst:** Der kritischste Punkt ist der Restore-Confirm-Dialog — er war seit Plan 4b ein nativer `confirm()`, der auf macOS mit sehr knapp dimensioniertem Text und OS-Style erscheint. Eine shadcn AlertDialog mit „Daten werden ersetzt"-Hinweis, rotem Button, und ausreichend Luft zum Lesen ist hier **keine Kosmetik**, sondern schützt vor Daten-Unfällen.
- **Nicht kompliziert:** Der ConfirmDialog wird **pro Call-Site als lokaler State** verdrahtet — keine globale Provider/Promise-API. Kostet 4-5 extra Zeilen pro Site, aber jeder Reader versteht es sofort.
- **Tests + Logs:** Die beiden neuen UI-Komponenten bekommen je einen Render-Test. Keine neuen Rust-Tests (pure Refaktorierung). Logs: `data_dir()`-Umstellung ändert nichts an bestehenden Log-Punkten.

**Kein Git** – Checkpoints statt Commits.

---

## Datei-Struktur (Ziel nach diesem Plan)

```
src-tauri/src/
├─ lib.rs                                # MODIFY: data_dir wird lokal berechnet, in AppState gespeichert
└─ commands.rs                           # MODIFY: fn data_dir() entfernt; state.data_dir + db_path() als Helper

src/
├─ App.tsx                               # MODIFY: "profiles"-Branch raus
├─ components/
│  ├─ ui/
│  │  └─ alert-dialog.tsx                # NEU: shadcn-Auto-Gen
│  ├─ common/
│  │  ├─ ConfirmDialog.tsx               # NEU: Wrapper um AlertDialog (destructive-Flag, labels)
│  │  ├─ ConfirmDialog.test.tsx          # NEU: 2 Render-Tests
│  │  ├─ LoadingState.tsx                # NEU: Spinner + Text (einheitlich)
│  │  ├─ EmptyState.tsx                  # NEU: Icon + Message + optional Action
│  │  └─ states.test.tsx                 # NEU: Render-Tests für beide
│  ├─ layout/
│  │  └─ Sidebar.tsx                     # MODIFY: „Profile"-Item raus
│  ├─ map/
│  │  └─ CenterPickerMap.tsx             # MODIFY: flyTo im center-useEffect
│  ├─ detail/
│  │  └─ CompanyDetailSheet.tsx          # MODIFY: confirm() → <ConfirmDialog>
│  └─ settings/
│     ├─ BranchenTab.tsx                 # MODIFY: delete-confirm() → <ConfirmDialog>
│     ├─ ProfileTab.tsx                  # MODIFY: delete-confirm() → <ConfirmDialog>
│     └─ DatenTab.tsx                    # MODIFY: restore-confirm() → <ConfirmDialog>
├─ pages/
│  ├─ CompaniesPage.tsx                  # MODIFY: Lade…-Text → <LoadingState>
│  ├─ MapPage.tsx                        # MODIFY: + EmptyState wenn companies.length === 0
│  ├─ NotImplementedPage.tsx             # DELETE
│  └─ … (keine anderen Änderungen)
├─ stores/
│  └─ uiStore.ts                         # MODIFY: "profiles" aus View-Union raus
└─ components/settings/ProfileTab.tsx    # (zusätzlich zu oben) MODIFY: Lade…-Text → <LoadingState>
```

Keine neue Migration. Keine neuen Backend-Module. Keine neue Rust-Dependency.

---

# PHASE 6A — Polish + Tech-Debt

## Reihenfolge-Hinweis

Die Tasks sind bewusst so geordnet, dass jeder Task für sich testbar ist und nichts downstream blockiert:

1. **6A.1**: Sidebar/Profile entfernen — kleinster Change, Quick Win
2. **6A.2**: shadcn AlertDialog installieren + ConfirmDialog-Komponente bauen
3. **6A.3**: 4 destruktive Confirms migrieren (nutzt 6A.2)
4. **6A.4**: `data_dir()` im Backend dedupen — unabhängig
5. **6A.5**: CenterPickerMap flyTo
6. **6A.6**: LoadingState + EmptyState + 3 Migrations
7. **6A.7**: Smoke-Test + Acceptance

---

### Task 6A.1: Sidebar „Profile"-Item entfernen + NotImplementedPage löschen

**Files:**
- Modify: `src/stores/uiStore.ts`
- Modify: `src/components/layout/Sidebar.tsx`
- Modify: `src/App.tsx`
- Delete: `src/pages/NotImplementedPage.tsx`

- [ ] **Step 1: View-Union verkleinern**

`src/stores/uiStore.ts`:

```typescript
// Vorher:
// export type View = "dashboard" | "companies" | "search" | "map" | "profiles" | "settings"

// Nachher:
export type View = "dashboard" | "companies" | "search" | "map" | "settings"
```

Keine weiteren Änderungen an `uiStore.ts` — `currentView: "dashboard"` bleibt Default, niemand setzt mehr auf `"profiles"`.

- [ ] **Step 2: Sidebar-Item entfernen**

`src/components/layout/Sidebar.tsx`, `items`-Array:

```typescript
// Entferne diese Zeile:
// { key: "profiles",  label: "Profile",      Icon: FolderOpen },

// FolderOpen-Import kann bleiben oder raus — beides OK. Falls raus: aus lucide-Import-Liste streichen.
```

Vor-Check: Wird `FolderOpen` sonst wo in Sidebar.tsx benutzt? Nein (nur für das Profile-Item). Also aus der Import-Zeile entfernen.

- [ ] **Step 3: App.tsx-Branch entfernen**

`src/App.tsx`:

```tsx
// Entferne diese Zeile:
// {view === "profiles" && <NotImplementedPage view={view} />}

// Entferne den Import:
// import { NotImplementedPage } from "@/pages/NotImplementedPage"
```

- [ ] **Step 4: Datei löschen**

```bash
rm /Users/jan/Dev/Projects/ProjectAlpha/src/pages/NotImplementedPage.tsx
```

- [ ] **Step 5: TS-Check**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm exec tsc --noEmit 2>&1 | tail -5
```

Erwartet: keine Errors.

- [ ] **Step 6: Full test suite**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test 2>&1 | tail -5
```

Erwartet: **31 passed** (unverändert).

- [ ] **Step 7: Checkpoint 6A.1**

---

### Task 6A.2: shadcn AlertDialog installieren + `<ConfirmDialog>`-Komponente

**Files:**
- Auto-generated: `src/components/ui/alert-dialog.tsx`
- Create: `src/components/common/ConfirmDialog.tsx`
- Create: `src/components/common/ConfirmDialog.test.tsx`

- [ ] **Step 1: shadcn AlertDialog installieren**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm dlx shadcn@latest add alert-dialog --yes --overwrite 2>&1 | tail -5
```

Erwartet: `src/components/ui/alert-dialog.tsx` erstellt. Fügt ggf. ein `radix-ui`-Subpaket hinzu — das ist OK (radix-ui ist schon Dep).

- [ ] **Step 2: Verzeichnis anlegen**

```bash
mkdir -p /Users/jan/Dev/Projects/ProjectAlpha/src/components/common
```

- [ ] **Step 3: Failing Test schreiben**

`src/components/common/ConfirmDialog.test.tsx`:

```tsx
import { describe, expect, it, vi } from "vitest"
import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { ConfirmDialog } from "./ConfirmDialog"

describe("ConfirmDialog", () => {
  it("renders title and description when open", () => {
    render(
      <ConfirmDialog
        open
        title="Sicher?"
        description="Diese Aktion ist nicht rückgängig machbar."
        confirmLabel="Löschen"
        destructive
        onConfirm={() => {}}
        onOpenChange={() => {}}
      />
    )
    expect(screen.getByText("Sicher?")).toBeInTheDocument()
    expect(screen.getByText("Diese Aktion ist nicht rückgängig machbar.")).toBeInTheDocument()
    expect(screen.getByRole("button", { name: "Löschen" })).toBeInTheDocument()
    expect(screen.getByRole("button", { name: "Abbrechen" })).toBeInTheDocument()
  })

  it("calls onConfirm when confirm button clicked", async () => {
    const onConfirm = vi.fn()
    const onOpenChange = vi.fn()
    render(
      <ConfirmDialog
        open
        title="X"
        description="y"
        confirmLabel="OK"
        onConfirm={onConfirm}
        onOpenChange={onOpenChange}
      />
    )
    await userEvent.click(screen.getByRole("button", { name: "OK" }))
    expect(onConfirm).toHaveBeenCalledOnce()
  })
})
```

- [ ] **Step 4: Test läuft rot**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test -- ConfirmDialog 2>&1 | tail -10
```

Erwartet: Import-Fehler (ConfirmDialog existiert nicht).

- [ ] **Step 5: Komponente schreiben**

`src/components/common/ConfirmDialog.tsx`:

```tsx
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { buttonVariants } from "@/components/ui/button"
import { cn } from "@/lib/utils"

interface ConfirmDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  title: string
  description: string
  confirmLabel: string
  cancelLabel?: string
  destructive?: boolean
  onConfirm: () => void
}

export function ConfirmDialog({
  open,
  onOpenChange,
  title,
  description,
  confirmLabel,
  cancelLabel = "Abbrechen",
  destructive = false,
  onConfirm,
}: ConfirmDialogProps) {
  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{title}</AlertDialogTitle>
          <AlertDialogDescription>{description}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{cancelLabel}</AlertDialogCancel>
          <AlertDialogAction
            onClick={onConfirm}
            className={destructive ? cn(buttonVariants({ variant: "destructive" })) : undefined}
          >
            {confirmLabel}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
```

**Note:** `buttonVariants` ist bereits in `src/components/ui/button.tsx` exportiert (shadcn-Standard). Wenn dort nur `Button` (ohne `buttonVariants`) exportiert ist, muss die `destructive`-Class-Logik anders gelöst werden — prüfe die Datei vor dem Schreiben. Fallback ohne buttonVariants:

```tsx
className={destructive ? "bg-destructive text-destructive-foreground hover:bg-destructive/90" : undefined}
```

- [ ] **Step 6: Tests laufen grün**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test -- ConfirmDialog 2>&1 | tail -10
```

Erwartet: 2 passed.

- [ ] **Step 7: Full suite**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test 2>&1 | tail -5
```

Erwartet: **33 passed** (31 + 2 neue).

- [ ] **Step 8: TS-Check**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm exec tsc --noEmit 2>&1 | tail -5
```

Erwartet: clean.

- [ ] **Step 9: Checkpoint 6A.2**

---

### Task 6A.3: Vier destruktive Confirms auf `<ConfirmDialog>` migrieren

**Files:**
- Modify: `src/components/settings/DatenTab.tsx` — Restore-Confirm
- Modify: `src/components/detail/CompanyDetailSheet.tsx` — Delete-Company-Confirm
- Modify: `src/components/settings/ProfileTab.tsx` — Delete-Profile-Confirm
- Modify: `src/components/settings/BranchenTab.tsx` — Delete-Category-Confirm

**Migration-Muster (wiederholt in allen 4 Dateien):**

Ersetze:
```tsx
if (!confirm("…")) return
await api.xyz(...)
```

Durch:
```tsx
// State pro Component
const [confirmOpen, setConfirmOpen] = useState(false)

// Button-Handler setzt nur den State:
const askDelete = () => setConfirmOpen(true)

// Die tatsächliche Action, vom Dialog-OK-Button aufgerufen:
const doDelete = async () => {
  setConfirmOpen(false)
  // eigentliche await api.xyz(...) Logik
}

// JSX:
<ConfirmDialog
  open={confirmOpen}
  onOpenChange={setConfirmOpen}
  title="…"
  description="…"
  confirmLabel="Löschen" // oder "Wiederherstellen" für Restore
  destructive
  onConfirm={doDelete}
/>
```

- [ ] **Step 1: DatenTab.tsx — Restore-Confirm**

In `src/components/settings/DatenTab.tsx` (aktuell bei Zeile 28), ersetze das `confirm(...)` durch State-basierten Dialog.

**Neuer Text des Dialogs:**
- **Titel:** "Datenbank wiederherstellen?"
- **Beschreibung:** "Restore ersetzt die aktuelle Datenbank vollständig mit dem Inhalt der ausgewählten Datei. Ein Pre-Restore-Snapshot wird automatisch angelegt. Die App startet nach dem Restore neu."
- **Confirm-Label:** "Wiederherstellen"
- **destructive:** true

Importiere `ConfirmDialog` von `@/components/common/ConfirmDialog` oben in der Datei. Füge den Dialog am Ende des return-JSX hinzu (unterhalb des letzten Buttons, vor dem schließenden Fragment/Div).

- [ ] **Step 2: CompanyDetailSheet.tsx — Delete-Company-Confirm**

In `src/components/detail/CompanyDetailSheet.tsx` (aktuell bei Zeile 92), ersetze:
```tsx
if (!confirm(`Firma "${company.name}" wirklich löschen?`)) return
```

**Dialog-Text:**
- **Titel:** "Firma löschen?"
- **Beschreibung:** `"${company.name}" wird dauerhaft aus der Datenbank entfernt. Verbundene Aktivitäten und Notizen werden ebenfalls gelöscht.`
- **Confirm-Label:** "Löschen"
- **destructive:** true

Da die Beschreibung den Firmennamen enthält, muss `company` beim Öffnen des Dialogs bereits geladen sein — das ist es in diesem Component ohnehin (sonst wäre der Löschen-Button nicht sichtbar).

- [ ] **Step 3: ProfileTab.tsx — Delete-Profile-Confirm**

In `src/components/settings/ProfileTab.tsx` (aktuell bei Zeile 33), ersetze:
```tsx
if (!confirm(`Profil "${profile.name}" wirklich löschen?`)) return
```

**Dialog-Text:**
- **Titel:** "Such-Profil löschen?"
- **Beschreibung:** `Das Profil "${profile.name}" wird dauerhaft entfernt. Bereits gespeicherte Firmen bleiben unverändert.`
- **Confirm-Label:** "Löschen"
- **destructive:** true

**Wichtig:** Der Profile-Name muss im State mitgehalten werden, weil der Dialog pro Profil-Zeile gerendert wird. Einfachste Lösung: State als `{ id: number; name: string } | null` speichern statt `boolean`. Dann:
```tsx
const [toDelete, setToDelete] = useState<{ id: number; name: string } | null>(null)
// In Row-Loop:
<Button onClick={() => setToDelete({ id: p.id, name: p.name })}>Löschen</Button>
// Dialog:
<ConfirmDialog
  open={toDelete !== null}
  onOpenChange={(o) => { if (!o) setToDelete(null) }}
  title="Such-Profil löschen?"
  description={toDelete ? `Das Profil "${toDelete.name}" wird dauerhaft entfernt. Bereits gespeicherte Firmen bleiben unverändert.` : ""}
  confirmLabel="Löschen"
  destructive
  onConfirm={async () => {
    if (!toDelete) return
    try { await api.deleteSearchProfile(toDelete.id); setToDelete(null); await refresh() } catch (e) { /* existing error handling, bleibt erstmal als alert() */ }
  }}
/>
```

- [ ] **Step 4: BranchenTab.tsx — Delete-Category-Confirm**

In `src/components/settings/BranchenTab.tsx` (aktuell bei Zeile ~25, prüfe exakte Stelle), ersetze:
```tsx
if (!confirm(`Branche "${cat.name_de}" wirklich löschen?`)) return
```

Gleiches Muster wie 6A.3 Step 3: State `{ id: number; name: string } | null`.

**Dialog-Text:**
- **Titel:** "Branche löschen?"
- **Beschreibung:** `Die Branche "${toDelete.name}" wird entfernt. Firmen, die dieser Branche zugeordnet waren, bleiben bestehen (Branche wird auf „keine" gesetzt).`
- **Confirm-Label:** "Löschen"
- **destructive:** true

- [ ] **Step 5: TS-Check**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm exec tsc --noEmit 2>&1 | tail -5
```

Erwartet: clean.

- [ ] **Step 6: Full Test Suite**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test 2>&1 | tail -5
```

Erwartet: **33 passed** (unverändert — Tests prüfen ConfirmDialog-Komponente, nicht Migrationen).

- [ ] **Step 7: Vite-Build**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm vite build 2>&1 | tail -8
```

Erwartet: clean.

- [ ] **Step 8: Checkpoint 6A.3**

---

### Task 6A.4: `data_dir()` zentralisieren in `AppState`

**Files:**
- Modify: `src-tauri/src/lib.rs` — berechne `data_dir` lokal in `run()`, packe ins AppState
- Modify: `src-tauri/src/commands.rs` — entferne lokale `fn data_dir()`, nutze `state.data_dir`

- [ ] **Step 1: `AppState` erweitern**

`src-tauri/src/lib.rs`, die Struct (aktuell Zeilen 13-16):

```rust
use std::path::PathBuf;
use sqlx::SqlitePool;
use std::sync::Arc;

pub struct AppState {
    pub db: SqlitePool,
    pub nominatim: crate::nominatim::client::NominatimClient,
    pub data_dir: PathBuf,
}
```

- [ ] **Step 2: `run()` initialisiert `data_dir` einmal zentral**

Weiter in `lib.rs::run()`, ersetze den Block am Anfang:

```rust
pub fn run() {
    let data_dir: PathBuf = dirs::data_dir()
        .map(|p| p.join("projektalpha"))
        .unwrap_or_else(|| PathBuf::from("./projektalpha"));
    let log_dir = data_dir.join("logs");
    // … restlicher Log/Crash/DB-Init-Code nutzt nun `data_dir` statt `app_data`
```

Die nachfolgenden Referenzen auf die neue Variable `data_dir` umstellen. **Wichtig:** Zeilen 37-40 in `lib.rs` haben einen zweiten, separaten `dirs::data_dir().expect(...).join("projektalpha").join("crashes")`-Aufruf für `crash_dir` — dieser muss ebenfalls durch `data_dir.join("crashes")` ersetzt werden, sonst bleibt die Duplikation teilweise bestehen. Die `.manage(Arc::new(AppState { ... }))`-Zeile bekommt `data_dir: data_dir.clone()`:

```rust
.manage(Arc::new(AppState {
    db: pool,
    nominatim: crate::nominatim::client::NominatimClient::new(),
    data_dir: data_dir.clone(),
}))
```

- [ ] **Step 3: `commands.rs` entschlacken**

In `src-tauri/src/commands.rs` die lokale `fn data_dir()` (aktuell Zeilen 438-442) **entfernen**. Die `fn db_path()` (Zeile 444-446) neu implementieren als free function, die ein `&PathBuf` Argument nimmt — ODER (einfacher) die Call-Sites direkt `state.data_dir.join("data.db")` berechnen lassen.

Konkrete Änderungen an den 3 synchronen Commands:

**`backup_db` (aktuell Zeile 452):**
```rust
#[tauri::command]
pub fn backup_db(app: AppHandle, state: State<'_, Arc<AppState>>) -> AppResult<Option<String>> {
    let src = state.data_dir.join("data.db");
    // … Rest bleibt wie gehabt
}
```

**`restore_db` (aktuell Zeile 475):**
```rust
#[tauri::command]
pub fn restore_db(app: AppHandle, state: State<'_, Arc<AppState>>) -> AppResult<bool> {
    let dst = state.data_dir.join("data.db");
    let snap_dir = backup::snapshot_dir(&state.data_dir);
    // … Rest bleibt wie gehabt
}
```

**`open_data_dir` (aktuell Zeile 504):**
```rust
#[tauri::command]
pub fn open_data_dir(state: State<'_, Arc<AppState>>) -> AppResult<()> {
    tauri_plugin_opener::open_path(&state.data_dir, None::<&str>)
        .map_err(|e| AppError::Internal(format!("open_path: {e}")))?;
    tracing::info!("data dir opened");
    Ok(())
}
```

**Wichtig:** `backup_db`, `restore_db`, `open_data_dir` nehmen jetzt `state: State<'_, Arc<AppState>>`. Tauri erkennt State-Parameter automatisch — keine Änderung am Frontend nötig (die `invoke("backup_db")`-Aufrufe brauchen kein neues Argument).

Die Imports `use std::path::PathBuf` am Top des `// Backup / Restore / Utility commands`-Blocks darf stehenbleiben oder raus — PathBuf wird nur noch für State-Felder gebraucht.

- [ ] **Step 4: Build**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo build --lib 2>&1 | tail -10
```

Erwartet: `Finished` ohne neue Errors. Compiler weist darauf hin, falls Variablen unbenutzt sind — entsprechend aufräumen.

- [ ] **Step 5: Tests**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib 2>&1 | tail -5
```

Erwartet: **82 passed** (keine Regression; Tests berühren `data_dir` nicht).

- [ ] **Step 6: `cargo fmt`**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo fmt 2>&1 | tail -3
```

Erwartet: stiller Erfolg oder kleine Reformatierungen.

- [ ] **Step 7: Checkpoint 6A.4**

---

### Task 6A.5: CenterPickerMap — `flyTo` bei Center-Change

**Files:**
- Modify: `src/components/map/CenterPickerMap.tsx`

- [ ] **Step 1: Den `useEffect`-Block erweitern**

Die bestehende Struktur des useEffects (Zeilen 82-104):

```tsx
useEffect(() => {
  const map = mapRef.current
  if (!map) return

  const applyData = () => {
    if (!center) {
      // Marker entfernen + leere Radius-Source setzen
      return
    }
    // Marker setzen + Radius-Source mit Kreis füllen
    markerRef.current.setLngLat([center.lng, center.lat]).addTo(map)
    // …
  }

  if (styleLoadedRef.current) applyData()
  else map.once("load", applyData)
}, [center, radiusKm])
```

**Wichtig:** Der `flyTo`-Call gehört **innerhalb der `applyData`-Funktion**, nach dem `markerRef.current.setLngLat(...)`-Call, und nur in dem Zweig wo `center` gesetzt ist. So wird er beim ersten Mount nach `map.once("load", applyData)` gefeuert (nachdem der Style geladen ist) und bei jedem späteren `center`/`radiusKm`-Wechsel synchron.

Konkrete Änderung innerhalb `applyData()` direkt nach `src?.setData(radiusCircleGeoJSON(center, radiusKm))`:

```tsx
// NEU: sanfter Flug zum neuen Mittelpunkt
map.flyTo({
  center: [center.lng, center.lat],
  zoom: Math.max(map.getZoom(), 11), // nicht rauszoomen; min. Zoom 11
  speed: 1.2,
  curve: 1.4,
})
```

Da `applyData` die lokale Variable `map` aus dem Closure hat (Zeile 83: `const map = mapRef.current`), direkt `map.flyTo(...)` nutzen — nicht erneut `mapRef.current.flyTo(...)`.

**Zoom-Policy:** Wenn die Karte bereits näher als Zoom 11 ist (Benutzer hat hereingezoomt), bleib auf dem höheren Zoom. Wenn weiter weg als 11 (z. B. nach Mount mit initialView Zoom 6), zoom auf 11 ran. `Math.max(map.getZoom(), 11)` erfüllt das.

- [ ] **Step 2: TS-Check**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm exec tsc --noEmit 2>&1 | tail -5
```

Erwartet: clean.

- [ ] **Step 3: Full Test Suite + Vite-Build**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test 2>&1 | tail -5 && pnpm vite build 2>&1 | tail -5
```

Erwartet: **33 passed**, Vite-Build clean.

- [ ] **Step 4: Checkpoint 6A.5**

**Hinweis:** Der flyTo-Effekt ist reines visuelles Verhalten und wird beim Smoke-Test in 6A.7 geprüft — nicht unit-testbar ohne MapLibre-Mocking (YAGNI).

---

### Task 6A.6: `<LoadingState>` + `<EmptyState>` + 3 Migrations

**Files:**
- Create: `src/components/common/LoadingState.tsx`
- Create: `src/components/common/EmptyState.tsx`
- Create: `src/components/common/states.test.tsx`
- Modify: `src/pages/CompaniesPage.tsx` — `"Lade…"`-Text → `<LoadingState>`
- Modify: `src/components/settings/ProfileTab.tsx` — `"Lade…"`-Text → `<LoadingState>`
- Modify: `src/pages/MapPage.tsx` — `<EmptyState>` wenn `companies.length === 0`

- [ ] **Step 1: Failing Tests schreiben**

`src/components/common/states.test.tsx`:

```tsx
import { describe, expect, it } from "vitest"
import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { LoadingState } from "./LoadingState"
import { EmptyState } from "./EmptyState"

describe("LoadingState", () => {
  it("renders the given message", () => {
    render(<LoadingState message="Lade Firmen…" />)
    expect(screen.getByText("Lade Firmen…")).toBeInTheDocument()
  })
  it("falls back to default German 'Lade…'", () => {
    render(<LoadingState />)
    expect(screen.getByText("Lade…")).toBeInTheDocument()
  })
})

describe("EmptyState", () => {
  it("renders title and hint", () => {
    render(<EmptyState title="Keine Firmen" hint="Starte eine neue Suche." />)
    expect(screen.getByText("Keine Firmen")).toBeInTheDocument()
    expect(screen.getByText("Starte eine neue Suche.")).toBeInTheDocument()
  })
  it("fires action callback when button clicked", async () => {
    let clicked = false
    render(
      <EmptyState
        title="X"
        hint="y"
        actionLabel="Los"
        onAction={() => { clicked = true }}
      />
    )
    await userEvent.click(screen.getByRole("button", { name: "Los" }))
    expect(clicked).toBe(true)
  })
})
```

- [ ] **Step 2: Tests laufen rot**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test -- states 2>&1 | tail -15
```

Erwartet: Import-Fehler.

- [ ] **Step 3: Komponenten schreiben**

`src/components/common/LoadingState.tsx`:

```tsx
import { Loader2 } from "lucide-react"
import { cn } from "@/lib/utils"

interface LoadingStateProps {
  message?: string
  className?: string
}

export function LoadingState({ message = "Lade…", className }: LoadingStateProps) {
  return (
    <div className={cn("flex items-center gap-2 text-sm text-muted-foreground p-4", className)}>
      <Loader2 className="size-4 animate-spin" />
      <span>{message}</span>
    </div>
  )
}
```

`src/components/common/EmptyState.tsx`:

```tsx
import { Button } from "@/components/ui/button"
import { cn } from "@/lib/utils"
import type { ReactNode } from "react"

interface EmptyStateProps {
  title: string
  hint?: string
  icon?: ReactNode
  actionLabel?: string
  onAction?: () => void
  className?: string
}

export function EmptyState({ title, hint, icon, actionLabel, onAction, className }: EmptyStateProps) {
  return (
    <div className={cn("flex flex-col items-center justify-center gap-3 p-8 text-center", className)}>
      {icon && <div className="text-muted-foreground text-4xl">{icon}</div>}
      <div className="font-medium">{title}</div>
      {hint && <div className="text-sm text-muted-foreground max-w-xs">{hint}</div>}
      {actionLabel && onAction && (
        <Button variant="outline" size="sm" onClick={onAction}>{actionLabel}</Button>
      )}
    </div>
  )
}
```

- [ ] **Step 4: Tests laufen grün**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test -- states 2>&1 | tail -15
```

Erwartet: 4 passed.

- [ ] **Step 5: `CompaniesPage` migrieren**

`src/pages/CompaniesPage.tsx` Zeile 17-19 — ersetze:
```tsx
{loading
  ? <div className="p-4 text-sm text-muted-foreground">Lade…</div>
  : <CompanyList companies={companies} selectedId={selectedCompanyId} onSelect={selectCompany} />}
```

Durch:
```tsx
{loading
  ? <LoadingState message="Lade Firmen…" />
  : companies.length === 0
    ? <EmptyState
        title="Noch keine Firmen"
        hint="Starte eine neue Suche, um OSM-Leads zu laden."
        actionLabel="Neue Suche"
        onAction={() => useUiStore.getState().setView("search")}
      />
    : <CompanyList companies={companies} selectedId={selectedCompanyId} onSelect={selectCompany} />}
```

Imports oben ergänzen:
```tsx
import { LoadingState } from "@/components/common/LoadingState"
import { EmptyState } from "@/components/common/EmptyState"
```

- [ ] **Step 6: `ProfileTab` migrieren**

In `src/components/settings/ProfileTab.tsx` den `"Lade…"`-Text (Zeile ~45) durch `<LoadingState message="Lade Profile…" />` ersetzen. Import oben ergänzen.

- [ ] **Step 7: `MapPage` Empty-State**

In `src/pages/MapPage.tsx` — wenn `companies.length === 0`, zeige `<EmptyState>` über der Karte oder ersetze die Count-Box. Einfachster Weg: den bestehenden Count-Hinweis durch ein conditional ersetzen:

```tsx
{companies.length === 0 ? (
  <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
    <div className="pointer-events-auto">
      <EmptyState
        title="Noch keine Firmen auf der Karte"
        hint="Ergebnisse erscheinen hier, sobald eine Suche läuft."
        actionLabel="Neue Suche"
        onAction={() => useUiStore.getState().setView("search")}
      />
    </div>
  </div>
) : (
  <div className="absolute top-2 left-2 bg-background/90 backdrop-blur rounded-md px-3 py-1.5 text-xs shadow border">
    {companies.length} Firmen · Klick auf Pin öffnet Details
  </div>
)}
```

Imports oben ergänzen.

- [ ] **Step 8: Full Test Suite + TS-Check + Build**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test 2>&1 | tail -5 && pnpm exec tsc --noEmit 2>&1 | tail -3 && pnpm vite build 2>&1 | tail -5
```

Erwartet:
- `pnpm test` → **37 passed** (33 + 4 neue).
- `tsc` clean.
- Vite-Build clean.

- [ ] **Step 9: Checkpoint 6A.6**

---

### Task 6A.7: Acceptance / Smoke-Test

**Files:** keine.

- [ ] **Step 1: App starten**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm tauri dev
```

Parallel Log-Stream:
```bash
tail -F ~/Library/Application\ Support/projektalpha/logs/projektalpha.log.*
```

- [ ] **Step 2: Acceptance-Checkliste durchklicken**

**Sidebar / Profile-Konsolidierung (3 Punkte):**
1. Sidebar zeigt 5 Einträge (Dashboard, Firmen, Neue Suche, Karte, Einstellungen) — **kein** „Profile" mehr → ✓
2. Profile-Verwaltung nach wie vor unter Einstellungen → Profile erreichbar → ✓
3. App startet weiterhin auf Dashboard (Default-View unverändert) → ✓

**Destruktive Confirms (5 Punkte):**
4. Einstellungen → Daten → „Daten wiederherstellen" → neuer AlertDialog mit Titel „Datenbank wiederherstellen?", rotem „Wiederherstellen"-Button, „Abbrechen"-Button → ✓
5. Abbrechen schließt Dialog ohne Effekt → ✓
6. Firma im Detail-Sheet löschen → AlertDialog mit Firmennamen in Beschreibung → Löschen entfernt Firma → Detail-Sheet schließt → Liste refreshed → ✓
7. Einstellungen → Profile → Löschen → AlertDialog mit Profil-Namen → Löschen entfernt Profil → ✓
8. Einstellungen → Branchen → Löschen → AlertDialog mit Branchen-Namen → Löschen entfernt Branche → ✓

**Karte flyTo (3 Punkte):**
9. Neue Suche → Adress-Eingabe „Hamburg" → Ergebnis klicken → Karte fliegt **sanft** nach Hamburg, zoomt auf mind. Zoom 11 → ✓
10. Zoom vorher manuell auf 14 setzen, zweite Adresse picken → bleibt auf Zoom 14 (kein Zurück-Zoom) → ✓
11. Karten-Klick zur manuellen Mittelpunkt-Wahl (statt Adress-Suche): funktioniert unverändert, kein doppeltes flyTo → ✓

**Loading / Empty-States (4 Punkte):**
12. Firmen-Ansicht bei leerer DB: zeigt `<EmptyState>` „Noch keine Firmen" mit „Neue Suche"-Button → Klick wechselt zu Neue Suche → ✓
13. Firmen-Ansicht beim Laden: kurz „Lade Firmen…" mit Spinner-Icon (nicht nur Text) → ✓
14. Settings → Profile bei leerem Tab: „Lade Profile…" mit Spinner beim ersten Öffnen → ✓
15. Karte bei leerer DB: Empty-State-Card zentriert, nicht nur „0 Firmen" → Klick auf „Neue Suche"-Button wechselt zu Neue Suche → ✓

**Regressions (3 Punkte):**
16. Backup erstellen → funktioniert unverändert, Ziel-Dialog öffnet → ✓
17. Dashboard zeigt KPIs, Heute-fällig, Timeline — alles unverändert grün → ✓
18. Status-Änderungen, Activity-Log-Einträge, Notizen → weiterhin keine Regression → ✓

**Logs:**
```bash
tail -100 ~/Library/Application\ Support/projektalpha/logs/projektalpha.log.*
```
19. Keine neuen WARN/ERROR-Meldungen durch AlertDialog oder `state.data_dir`-Umstellung → ✓
20. „backup written" / „restore complete" Logs weiterhin vorhanden und PII-frei → ✓

- [ ] **Step 3: Checkpoint 6A.7 = Plan 6A fertig**

---

## Was am Ende dieses Plans funktioniert

- ✅ Sidebar ist konsistent: 5 echte Views, kein Orphan „Profile"-Eintrag, keine `NotImplementedPage`-Toter-Code-Referenz mehr
- ✅ Alle 4 destruktiven Confirms laufen über eine richtige shadcn `AlertDialog` mit rotem Button, „Abbrechen" ist Primär-Fokus, Text ist groß und lesbar — deutlich sicherer als native `confirm()`
- ✅ `AppState.data_dir` ist die einzige Quelle der Wahrheit für den Daten-Ordner; `commands.rs` enthält keinen Duplikat mehr
- ✅ Karte fliegt bei Adress-Pick sanft zum neuen Mittelpunkt, respektiert den User-Zoom-Level
- ✅ `<LoadingState>` + `<EmptyState>` vereinheitlichen drei bislang inkonsistente Stellen (CompaniesPage, ProfileTab, MapPage)
- ✅ Frontend-Test-Suite wächst: 31 → 37 (2 ConfirmDialog + 4 LoadingState/EmptyState)
- ✅ Rust-Test-Suite unverändert bei 82 (reine Refaktorierung)
- ✅ Keine neuen NPM- oder Rust-Dependencies außer shadcn AlertDialog (radix-basiert, sowieso transitively enthalten)

## Was bewusst NICHT in diesem Plan ist

- **Die 8 verbleibenden `alert()`-Calls** (Fehlermeldungen in DatenTab/BranchenTab/ProfileTab/NewSearchPage/ManualAddDialog/TopBar) bleiben. Gründe: (a) nicht destruktiv, (b) Fehler-Toast/Sonner wäre eine neue UX-Komponente, (c) Plan 6B hat Release-Polish mit höherer Priorität. Wenn später ein Toast-System eingeführt wird (Plan 7+), werden diese auf einen Schlag umgezogen.
- **Die 2 `prompt()`-Calls** (ProfileTab Rename, NewSearchPage Save-as-Profile). Ein separater `<InputDialog>` oder Inline-Rename-UI wäre ein eigenes Polish-Thema — YAGNI für MVP.
- **CompanyDetailSheet-Ladezustand** (während `get_company` lädt). Kurzer, selten sichtbarer Zustand; wenn er nerven sollte, kommt er in Plan 6B oder später.
- **Framer-Motion-Animations** (Sidebar-Transitions, Sheet-Open-Kurven). Tauri öffnet mit `blur` + `backdropFilter` schon hübsch genug; echte Micro-Animations sind post-Release-Polish.
- **E2E-Playwright-Suite**. Für Single-User-Desktop-Tool + manuellen Smoke-Test per Plan reicht das Manual Acceptance. Falls später mehrere Entwickler oder Regressions-Prävention wichtig werden: eigener Plan.

---

## Nächster Plan

**Plan 6B — Release-Infrastruktur** (kommt nach 6A):
- Git + GitHub einführen (nur für Release, Dev-Workflow bleibt untouched)
- Tauri Auto-Updater konfigurieren + Update-Check-UI im Über-Tab
- GitHub Actions Workflow: macOS DMG + Windows MSI cross-build auf Git-Tag
- End-User README (deutsch, ein Download-Link, Install-Anleitung)
- Code-Signing-Entscheidung explizit dokumentieren (ad-hoc / signed / notarized)
- Optional: Erstes getaggtes Release `v0.1.0` für den Vater — inkl. Download-Link
