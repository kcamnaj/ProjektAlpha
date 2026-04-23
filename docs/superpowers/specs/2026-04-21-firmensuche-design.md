# Firmensuche – Design-Spezifikation

**Datum:** 2026-04-21
**Status:** Draft (in Review)
**Autor:** Jan
**Zielnutzer:** Vater (Vertrieb für Tor- & Bühnen-Dienstleistungen, UVV-Prüfungen)

---

## 1. Zweck & Ziele

### 1.1 Problem
Vater verkauft Wartungs- und UVV-Dienstleistungen für **Industrie-Tore und Verlade-/Hubbühnen**. Potenzielle Kunden müssen solche Anlagen besitzen (typisch: Logistik, Lebensmittel-Großhandel, Industrie, Lager). Aktuell existiert kein systematisches Tool, um:
- Firmen in einem definierten Umkreis zu finden, die plausibel relevante Anlagen besitzen
- Kontaktinformationen zentral zu pflegen
- Status, Notizen und Wiedervorlagen pro Firma zu verwalten

### 1.2 Ziel
Eine **kostenlose, lokale Desktop-Anwendung**, die Firmen in einem benutzerdefinierten Radius aus OpenStreetMap-Daten extrahiert, sie nach Branchen-Wahrscheinlichkeit für „besitzt Tore/Bühnen" bewertet, und ein vollständiges Lead-Management ermöglicht.

### 1.3 Nicht-Ziele (YAGNI)
- Kein Cloud-Backend, keine Multi-User-Funktionalität (nur Vater nutzt es)
- Keine automatische Qualifikation per LLM oder Web-Scraping (manuelle Bewertung gewünscht)
- Keine Integration in bestehende CRM-Systeme
- Keine Mobile-App
- Kein E-Mail-/Telefon-Versand aus der App heraus (nur `tel:` / `mailto:` Verweise)

---

## 2. Erfolgs-Kriterien

| Kriterium | Messbar |
|---|---|
| Vater findet pro Such-Lauf in unter 3 Minuten neue Leads | Zeit von Klick „Suche starten" bis Liste zeigt Treffer |
| Wiedervorlagen werden nicht vergessen | Dashboard zeigt fällige Wiedervorlagen beim App-Start |
| Datenverlust ausgeschlossen | Auto-Snapshots + WAL-Modus + manuelles Backup verfügbar |
| App kostet im Betrieb 0 € | Keine kostenpflichtigen APIs, keine Cloud-Dienste |
| Cross-Platform-Build funktioniert | Windows-`.msi` und Mac-`.dmg` aus einem Codebase |
| App ist „klein": Installer < 20 MB | Tauri-Bundle gemessen |

---

## 3. Funktions-Umfang (MVP)

| ID | Feature | Beschreibung |
|---|---|---|
| F1 | Umkreis-Suche | Mittelpunkt + Radius (1–300 km) → OSM-Treffer |
| F2 | Branchen-Filter mit Score | Nur ausgewählte Branchen, jede mit Wahrscheinlichkeits-Gewicht |
| F3 | Firmen-Liste | Sortierbar, filterbar nach Status/Branche/Score |
| F4 | Karten-Ansicht | MapLibre + OSM-Tiles, Pins farbcodiert nach Status, sync mit Liste |
| F5 | Firmen-Detail | Adresse, Kontaktdaten, Status, Notizen, Activity-Log |
| F6 | Status-Verwaltung | `neu` / `angefragt` / `kunde` / `kein_kunde` |
| F7 | Activity-Log (Notizen) | Timeline mit typisierten Einträgen (Notiz, Anruf, Mail, Besuch) |
| F8 | Wiedervorlage | Datum pro Firma, Dashboard zeigt heute Fällige |
| F9 | Letzter-Kontakt-Datum | Auto-gesetzt bei Status-Änderung |
| F10 | Ansprechpartner | Freitext-Feld pro Firma |
| F11 | Manuelles Hinzufügen | Firmen, die OSM nicht gefunden hat |
| F12 | Konfigurierbare Branchen | Branchen aktivieren/deaktivieren, Score anpassen, neue hinzufügen |
| F13 | Such-Profile | Mehrere Startpunkt+Radius-Konfigurationen speichern |
| F14 | Backup & Restore | Manuell + automatische Snapshots |
| F15 | Auto-Update | Über GitHub Releases via Tauri-Updater |

**Nicht im MVP** (laut User-Entscheidung): CSV-Export (kann später als Bonus dazukommen).

---

## 4. Architektur

### 4.1 Tech-Stack

| Schicht | Technologie | Begründung |
|---|---|---|
| Desktop-Shell | Tauri 2 | Kleinste Bundles (5–10 MB), modern, sicher, gute Cross-Compile-Story |
| Frontend | React 18 + TypeScript | Riesiges Ökosystem, übertragbar |
| UI-Komponenten | shadcn/ui + Tailwind CSS | Moderne Optik (Linear/Vercel-Style), volle Anpassbarkeit |
| State-Management | Zustand | Minimal, kein Boilerplate |
| Karte | MapLibre GL JS | GPU-beschleunigt, OSM-kompatibel, gratis |
| Animationen | Framer Motion | Sanfte Übergänge ohne Performance-Kosten |
| Backend (Core) | Rust | Tauri-Sprache, performant, sicher |
| Datenbank | SQLite (via `sqlx`) | Single-File, Backup-trivial, embedded |
| HTTP-Client (Rust) | `reqwest` | Standard, async, retry-fähig |
| Build / CI | GitHub Actions | Gratis, Mac+Windows-Runner verfügbar |
| Distribution | GitHub Releases | Gratis, Tauri-Updater integriert |
| Tests (Frontend) | Vitest + React Testing Library + Playwright | Standard-Stack |
| Tests (Backend) | `cargo test` + JSON-Fixtures | Schnell, deterministisch |

### 4.2 System-Übersicht

```
┌─────────────────────────────────────────────────────┐
│  Tauri Desktop App (Windows .msi / Mac .dmg)        │
│                                                     │
│  ┌────────────────────────┐   ┌──────────────────┐  │
│  │   React Frontend (UI)  │ ⇄ │  Rust Core       │  │
│  │   - shadcn/ui          │   │  - DB Access     │  │
│  │   - Tailwind           │   │  - OSM Fetcher   │  │
│  │   - MapLibre           │   │  - Geocoding     │  │
│  │   - Zustand            │   │  - File I/O      │  │
│  └────────────────────────┘   └────────┬─────────┘  │
│                                        │            │
│                              ┌─────────▼────────┐   │
│                              │  SQLite-Datei    │   │
│                              │  (App-Data-Pfad) │   │
│                              └──────────────────┘   │
└─────────────────────────────────────────────────────┘
                       │
                       ▼ (nur bei „Suche starten" / Karten-Tiles)
            ┌──────────────────────┐
            │  OSM Overpass API    │  (gratis, public)
            │  OSM Nominatim API   │  (Geocoding)
            │  OSM Tile Server     │  (Karten-Bilder)
            └──────────────────────┘
```

### 4.3 Schicht-Verantwortlichkeiten

**React Frontend (UI):**
- Alle sichtbaren Screens, Routing, Form-Handling
- State-Management der Anzeige (Filter, Selektion, Modal-Zustand)
- Karten-Rendering (MapLibre läuft im Browser-Kontext)
- IPC-Calls an Rust Core via Tauri `invoke()`

**Rust Core (Backend):**
- DB-Operationen: Schema-Migrations, CRUD, Queries mit Filtern
- Overpass-Integration: Query-Builder, Tile-Splitter, HTTP, Retry, Endpoint-Rotation
- Nominatim-Geocoding mit lokalem Cache
- Datei-Operationen: Backup, Restore, Snapshot-Rotation
- Settings-Persistenz

**SQLite (Datenhaltung):**
- Eine Datei im OS-typischen App-Data-Ordner (z.B. `%APPDATA%\projektalpha\data.db` auf Windows)
- WAL-Modus für Crash-Sicherheit
- Migrations-Tabelle für Schema-Versionierung

### 4.4 Modul-Grenzen (Klarheit & Testbarkeit)

Jedes Modul hat **eine klare Aufgabe**, ein **definiertes Interface**, ist **isoliert testbar**:

| Modul | Verantwortung | Eingabe | Ausgabe |
|---|---|---|---|
| `db::companies` | CRUD für Firmen | Firma-Struct / Filter | DB-Resultate |
| `db::activity` | Activity-Log | LogEntry | Liste pro Firma |
| `db::categories` | Branchen-Verwaltung | Category-Struct | Liste / einzeln |
| `db::profiles` | Such-Profile | Profile-Struct | Liste / einzeln |
| `overpass::query_builder` | OSM-Query aus Kategorien bauen | Categories + Center + Radius | Query-String |
| `overpass::tile_splitter` | Großen Radius in Kacheln | Center + Radius | Liste von Tiles |
| `overpass::client` | HTTP-Calls mit Retry/Rotation | Query | Raw JSON |
| `overpass::parser` | OSM-JSON → Firmen-Structs | JSON + Categories | Vec<Company> |
| `nominatim::client` | Adresse → Koordinaten | Suchstring | (lat, lng, label) |
| `backup::snapshot` | DB-Snapshots verwalten | DB-Pfad | Snapshot-Datei |

---

## 5. Datenmodell (SQLite-Schema)

### 5.1 Tabelle: `companies`

| Feld | Typ | Constraints | Zweck |
|---|---|---|---|
| `id` | TEXT (UUID) | PRIMARY KEY | |
| `osm_id` | TEXT | UNIQUE, NULL erlaubt | OSM-Identifier; NULL bei manueller Anlage |
| `name` | TEXT | NOT NULL | |
| `street` | TEXT | NULL | |
| `postal_code` | TEXT | NULL | |
| `city` | TEXT | NULL | |
| `country` | TEXT | DEFAULT 'DE' | |
| `lat` | REAL | NOT NULL | |
| `lng` | REAL | NOT NULL | |
| `phone` | TEXT | NULL | |
| `email` | TEXT | NULL | |
| `website` | TEXT | NULL | |
| `industry_category_id` | INT | FK → industry_categories.id, ON DELETE SET NULL | Bei Löschung einer Kategorie behält Firma alle Daten, Score wird auf 0 gesetzt und in UI als „Sonstiges (Kategorie gelöscht)" angezeigt |
| `size_estimate` | TEXT | NULL | „klein", „mittel", „groß" – aus OSM-Heuristik (z.B. Gebäudefläche) |
| `probability_score` | INT | 0–100 | Berechnet aus Kategorie-Gewicht (+ Bonus für size_estimate) |
| `status` | TEXT | NOT NULL, DEFAULT 'neu' | `neu` / `angefragt` / `kunde` / `kein_kunde` |
| `contact_person` | TEXT | NULL | Freitext |
| `last_contact_at` | DATETIME | NULL | Auto-gesetzt bei Status-Änderung |
| `next_followup_at` | DATETIME | NULL | Wiedervorlage-Datum |
| `source` | TEXT | NOT NULL | `osm` oder `manual` |
| `created_at` | DATETIME | NOT NULL | |
| `updated_at` | DATETIME | NOT NULL | |

**Indizes:** `osm_id` (UNIQUE), `status`, `next_followup_at`, `industry_category_id`, `(lat, lng)`.

### 5.2 Tabelle: `activity_log`

| Feld | Typ | Constraints | Zweck |
|---|---|---|---|
| `id` | TEXT (UUID) | PK | |
| `company_id` | TEXT | FK, NOT NULL, ON DELETE CASCADE | |
| `type` | TEXT | NOT NULL | `notiz`, `anruf`, `mail`, `besuch`, `status_änderung` |
| `content` | TEXT | NOT NULL | Freitext |
| `created_at` | DATETIME | NOT NULL | |

**Indizes:** `company_id`, `created_at` (DESC).

### 5.3 Tabelle: `industry_categories`

| Feld | Typ | Constraints | Zweck |
|---|---|---|---|
| `id` | INT | PK AUTOINCREMENT | |
| `name_de` | TEXT | NOT NULL, UNIQUE | „Logistik / Spedition" |
| `osm_tags` | TEXT (JSON) | NOT NULL | Mapping-Regeln, z.B. `[{"shop":"wholesale","wholesale":"food"}]` (Liste = OR, Object = AND) |
| `probability_weight` | INT | 0–100 | Default-Score |
| `enabled` | BOOLEAN | DEFAULT TRUE | An/Aus pro Kategorie |
| `color` | TEXT | DEFAULT '#3b82f6' | Pin-Farbe (Hex) |
| `sort_order` | INT | DEFAULT 0 | Reihenfolge in UI |

**Seed-Daten beim ersten Start:**

| name_de | Weight | enabled | osm_tags (Beispiel) |
|---|---:|:---:|---|
| Logistik / Spedition | 95 | ✓ | `[{"office":"logistics"},{"shop":"wholesale"}]` |
| Lebensmittel-Großhandel | 90 | ✓ | `[{"shop":"wholesale","wholesale":"food"}]` |
| Lagerhalle / Warehouse | 85 | ✓ | `[{"industrial":"warehouse"},{"building":"warehouse"}]` |
| Industrielle Produktion | 80 | ✓ | `[{"building":"industrial"},{"landuse":"industrial"}]` |
| Baumarkt / DIY | 80 | ✓ | `[{"shop":"doityourself"},{"shop":"hardware"}]` |
| Lebensmittel-Einzelhandel | 75 | ✓ | `[{"shop":"supermarket"},{"shop":"convenience"}]` |
| Möbel-/Bauhandel | 70 | ✓ | `[{"shop":"furniture"},{"shop":"trade"}]` |
| Pharma / Kosmetik | 65 | ✓ | `[{"industrial":"chemical"},{"man_made":"works","product":"pharmaceutical"}]` |
| Bäckerei (industriell) | 60 | ✓ | `[{"craft":"bakery"},{"shop":"bakery"}]` |
| Autohaus | 40 | ✓ | `[{"shop":"car"}]` |
| Bürogebäude | 5 | ✗ | `[{"building":"office"},{"office":"company"}]` |

### 5.4 Tabelle: `search_profiles`

| Feld | Typ | Constraints | Zweck |
|---|---|---|---|
| `id` | INT | PK AUTOINCREMENT | |
| `name` | TEXT | NOT NULL | „Tour Nord" |
| `center_label` | TEXT | NOT NULL | „Hannover Hbf" |
| `center_lat` | REAL | NOT NULL | |
| `center_lng` | REAL | NOT NULL | |
| `radius_km` | INT | NOT NULL, 1–300 | |
| `enabled_category_ids` | TEXT (JSON) | NOT NULL | `[1,2,3,5,7]` |
| `last_run_at` | DATETIME | NULL | |
| `created_at` | DATETIME | NOT NULL | |

### 5.5 Tabelle: `geocode_cache`

| Feld | Typ | Constraints | Zweck |
|---|---|---|---|
| `query` | TEXT | PK | normalisierte Suchanfrage |
| `lat` | REAL | NOT NULL | |
| `lng` | REAL | NOT NULL | |
| `display_name` | TEXT | NOT NULL | Aus Nominatim |
| `cached_at` | DATETIME | NOT NULL | TTL: 30 Tage |

### 5.6 Tabelle: `app_meta`

| Feld | Typ | Zweck |
|---|---|---|
| `key` | TEXT PK | z.B. `schema_version`, `last_snapshot_at` |
| `value` | TEXT | |

---

## 6. Schlüssel-Datenflüsse

### 6.1 Such-Pipeline

```
1. UI sammelt: { center: {lat, lng}, radius_km, enabled_category_ids[] }
2. Frontend → Tauri invoke("start_search", payload)
3. Rust Core:
   a. Lade Kategorien aus DB → extrahiere osm_tags
   b. Tile-Splitter:
      - radius ≤ 50 km → 1 Tile
      - 50 < radius ≤ 150 km → 4 Quadranten (radius/2)
      - radius > 150 km → Grid 50 km × 50 km
   c. Pro Tile (sequenziell, 1s Pause dazwischen):
      i. query_builder erstellt Overpass-QL
      ii. client POST → Endpoint (mit Rotation bei Fehler)
      iii. parser extrahiert Companies
      iv. Match Tags → Kategorie → Score
      v. INSERT OR IGNORE in DB (osm_id UNIQUE)
      vi. Bei vorhandener Firma: Update einzelner Felder NUR wenn
          - source='osm' (manuelle Einträge bleiben immer unangetastet)
          - UND aktueller DB-Wert IS NULL (kein Überschreiben echter Daten)
      vii. emit("search-progress", { done, total, last_count })
   d. emit("search-done", { stats })
4. Frontend hört Events → updated UI live, refresht Liste/Karte
```

### 6.2 Status-Update mit Activity-Log

```
1. UI: Dropdown "Status" geändert von „neu" auf „angefragt"
2. Frontend → invoke("update_company_status", { id, new_status })
3. Rust Core (eine Transaktion):
   a. UPDATE companies SET status, last_contact_at = NOW(), updated_at = NOW()
   b. INSERT INTO activity_log (type='status_änderung', content="von neu auf angefragt")
4. Frontend refresht Detail-Sheet
```

### 6.3 Adress-Suche (Mittelpunkt setzen)

```
1. UI: User tippt „Hannover Hauptbahnhof"
2. Debounce 500ms → invoke("geocode", { query })
3. Rust:
   a. Lookup geocode_cache; if hit + < 30 Tage alt → return
   b. Else: HTTP GET Nominatim, User-Agent „ProjektAlpha/1.0"
   c. Parse, INSERT in cache, return
4. UI zeigt Vorschläge, Klick → setzt Mittelpunkt
```

### 6.4 Wiedervorlage-Prüfung beim App-Start

```
1. App startet
2. Rust: SELECT COUNT(*) FROM companies WHERE next_followup_at <= NOW() AND status != 'kein_kunde'
3. Wenn > 0 → emit("followups-due", { count })
4. UI zeigt Toast + Dashboard markiert „Heute fällig"
```

### 6.5 Backup-Snapshot

```
Auto (täglich beim ersten Start des Tages):
1. Rust: prüft app_meta.last_snapshot_at
2. Wenn > 24h vergangen:
   a. SQLite VACUUM INTO backup-Pfad/db.snapshot.{YYYYMMDD}
   b. Lösche Snapshots älter als 7 Tage
   c. Update app_meta.last_snapshot_at

Manuell:
1. UI Settings → "Backup erstellen"
2. Save-Dialog (Tauri-Dialog API)
3. Rust: VACUUM INTO chosen_path
```

---

## 7. UI-Konzept

**Design-Sprache:** Linear/Vercel/Raycast-inspiriert: Inter-Font, viel Weißraum, dezente Schatten, sanfte Transitions, Light/Dark-Mode.

### 7.1 Globales Layout

```
┌────────────────────────────────────────────────────────────┐
│  [≡]  Profil ▾ "Tour Nord"        [☀/☾]  [⤓ Backup]  [⚙]   │
├──────┬─────────────────────────────────────────────────────┤
│ ⌂    │                                                     │
│ 🏢   │                  Hauptbereich                       │
│ 🗺   │                                                     │
│ 📁   │                                                     │
│ ⚙    │                                                     │
└──────┴─────────────────────────────────────────────────────┘
```

Sidebar: **Dashboard / Firmen / Karte / Profile / Einstellungen**.

### 7.2 Screens

1. **Dashboard:** KPI-Cards (Kunden / Angefragt / Neu / Ø Score), „Heute fällig"-Liste, „Letzte Aktivität"-Timeline.
2. **Firmen-Liste:** Split-View Liste links + Karte rechts, Filter-Bar (Status, Branche, Score-Range, Volltextsuche). Klick auf Firma → Detail-Sheet von rechts.
3. **Firmen-Detail (Sheet):** Header (Name, Adresse, Score), Action-Bar (Anrufen via `tel:`, Mail via `mailto:`, Website öffnen), Status-Dropdown, Wiedervorlage-Datepicker, Ansprechpartner-Feld, Activity-Log-Timeline mit „+ Eintrag".
4. **Karten-Vollbild:** Nur Karte, Pins farbcodiert nach Status, Klick → kleines Popup mit „Details öffnen"-Button.
5. **Neue Suche (Modal):** Adress-Suche oder Karten-Klick für Mittelpunkt, Radius-Slider 1–300 km, Branchen-Checkboxen mit Score-Anzeige, Option „Als Profil speichern", Live-Progress beim Suchen, Result-Vorschau.
6. **Einstellungen (Tabs):**
   - **Branchen:** Editierbare Tabelle (Name, OSM-Tags JSON-Editor, Score-Slider, Enable-Toggle, Pin-Farbe). „+ Neue Branche".
   - **Profile:** Liste mit Edit/Duplicate/Delete.
   - **Daten:** Backup-Button, Restore-Button, DB-Pfad anzeigen + im Explorer öffnen.
   - **Über:** Version, Update-Check, Lizenz.

---

## 8. OSM-Integration im Detail

### 8.1 Overpass-API

**Endpoints (Rotation bei Fehler):**
- `https://overpass-api.de/api/interpreter`
- `https://overpass.kumi.systems/api/interpreter`
- `https://overpass.private.coffee/api/interpreter`

**Query-Format (Beispiel, 50 km um Hannover):**
```overpassql
[out:json][timeout:25];
(
  nwr["shop"="wholesale"]["wholesale"="food"](around:50000, 52.37, 9.73);
  nwr["industrial"="warehouse"](around:50000, 52.37, 9.73);
  nwr["building"="industrial"](around:50000, 52.37, 9.73);
);
out center tags;
```

**Tile-Splitting:**
- Radius ≤ 50 km → 1 Query
- 50 < Radius ≤ 150 km → 4 Quadranten mit halbiertem Radius (überlappend)
- Radius > 150 km → Grid mit 50 km × 50 km Kacheln. Algorithmus: erzeuge Bounding-Box `[center.lat ± radius, center.lng ± radius]`, teile in `ceil(2·radius / 50)²` quadratische Kacheln, **filtere** alle Kacheln deren Mittelpunkt > radius vom Suchzentrum entfernt liegt (Circle-Clip). Pro übrig gebliebener Kachel ein Overpass-Call mit `(around:25000, tile_lat, tile_lng)` (25 km = halbe Diagonale + Puffer)
- Etiquette: 1 s Pause zwischen Calls, max. 3 Retries pro Tile, dann Endpoint-Wechsel
- Cancel-Token: Frontend kann Suche abbrechen → Rust beendet nach aktueller Tile

**Erwartete Dauer:**
- 50 km: ~10 s
- 150 km: ~30–60 s
- 300 km: ~1–3 Minuten

### 8.2 Nominatim (Geocoding)

- Endpoint: `https://nominatim.openstreetmap.org/search`
- Pflicht: User-Agent-Header (`ProjektAlpha/1.0 (kontakt: <email>)`)
- Limit: 1 req/s — Frontend debounct 500 ms, Cache hilft
- Cache-TTL: 30 Tage

### 8.3 Karten-Tiles

- Default: `https://tile.openstreetmap.org/{z}/{x}/{y}.png`
- Tauri cached besuchte Tiles im App-Cache (offline-fähig für angeschaute Regionen)
- Fallback bei OSM-Tile-Server-Problemen: MapTiler Free Tier (100k Tiles/Monat) – Konfigurierbar

---

## 9. Fehlerbehandlung & Edge Cases

| Szenario | Verhalten |
|---|---|
| Kein Internet | Such-Button disabled, Toast „Keine Verbindung" |
| Overpass-Timeout | Retry × 3, dann Endpoint-Wechsel, dann Fehler-Toast mit „Später erneut versuchen" |
| Overpass liefert leere Antwort | „Keine Treffer" + Empty-State mit Tipps |
| Suche während Lauf abgebrochen | Bereits importierte Firmen bleiben erhalten |
| Firma ohne Name in OSM | Fallback „Unbenannt (PLZ Ort)" |
| Doppelte Adresse, andere osm_id | Beide importiert (kann unterschiedliche Mieter sein) |
| Manuelle Firma vs. OSM-Treffer ähnlich | Fuzzy-Match auf Name+PLZ → Warnung beim Anlegen |
| Backup-Datei korrupt beim Restore | Pre-Restore-Auto-Snapshot ermöglicht Rollback |
| DB-Migration fehlschlägt | Rollback, App startet im Safe-Mode mit nur Settings-Tab |
| Tag-Mapping greift nicht | Firma bekommt Kategorie „Sonstiges" mit Score 30, sichtbar zur manuellen Zuordnung |
| Update-Snapshot ergänzt vorhandene Daten | Nur leere Felder werden überschrieben; manuelle Eingaben sind tabu |

---

## 10. Test-Strategie

### 10.1 Coverage-Ziel
80 % über alle Schichten (gemäß User-Coding-Standards).

### 10.2 Backend (Rust)
- **Unit:** alle reinen Funktionen (Tag-Matcher, Score-Berechnung, Tile-Splitter, Query-Builder)
- **DB-Integration:** gegen In-Memory-SQLite, jeder Test in Transaction → automatisches Rollback
- **HTTP-Integration:** Overpass-Calls gegen lokale JSON-Fixtures (echte Responses einmalig kuratiert)
- **Live-Smoke-Test:** ein einziger nightly Test gegen echte Overpass-API mit 5 km Radius

### 10.3 Frontend (React)
- **Unit (Vitest):** Pure Functions in `lib/` (Formatters, Filter-Utils)
- **Component (RTL):** FirmaCard, FilterBar, ActivityLogTimeline, StatusBadge
- **E2E (Playwright):** drei kritische Flows
  1. Suche → Firmen-Import → erscheinen in Liste + Karte
  2. Status-Änderung → Activity-Log-Eintrag + last_contact_at gesetzt
  3. Backup-Erstellung + Restore → Daten unverändert

### 10.4 TDD-Workflow
RED → GREEN → REFACTOR pro Modul, gemäß User-Rules.

---

## 10a. Logging & Observability

Strukturiertes Logging ist Pflicht – kein Feature darf ohne nachvollziehbare Logs ausgeliefert werden.

### 10a.1 Backend (Rust)
- **Crate:** `tracing` + `tracing-subscriber` + `tracing-appender`
- **Format:**
  - In Konsole (Dev): human-readable mit Farben
  - In Datei (Prod + Dev): JSON, eine Zeile pro Event
- **Log-Datei:** `<app_data>/logs/projektalpha.log`, täglich rotiert, max. 7 Tage
- **Default-Level:** `info`. Override via `RUST_LOG=debug` env
- **Pflicht-Events:**
  | Event | Level | Felder |
  |---|---|---|
  | App start/stop | `info` | version, schema_version |
  | DB-Migration | `info` | from_version, to_version, dauer_ms |
  | Such-Start | `info` | center, radius_km, n_categories, n_tiles |
  | Overpass-Call | `debug` | endpoint, tile_idx, http_status, dauer_ms, n_results |
  | Overpass-Retry/Endpoint-Wechsel | `warn` | tile_idx, alter_endpoint, neuer_endpoint, fehler |
  | Such-Ende | `info` | total_found, neu_imported, duplikate_skipped, dauer_ms |
  | Backup erstellt | `info` | pfad, größe_kb |
  | Restore | `info` | von_pfad, pre_restore_snapshot_pfad |
  | Fehler (jede Art) | `error` | typ, kontext (ohne PII) |

### 10a.2 Frontend (React)
- Eigener Wrapper `lib/logger.ts` über `console` + Tauri-Event:
  - `logger.info/warn/error(message, context)` → schreibt lokal **und** sendet via Tauri-Event ans Backend
  - Backend hängt diese Events an die zentrale Log-Datei an (`source: "frontend"`)
- **Pflicht-Events:** unbehandelte React-Fehler (via Error Boundary), Promise-Rejections, kritische User-Aktionen (Backup-Klick, Restore-Klick, Such-Start)

### 10a.3 PII-Regel (zwingend)
- **Niemals** in Logs: Ansprechpartner-Namen, E-Mail-Adressen, Telefonnummern, Notiz-Inhalte
- Beim Loggen einer Firma: nur `company_id` und ggf. PLZ/Stadt
- Code-Reviewer prüft das aktiv

### 10a.4 Crash-Handling
- Rust-Panics → `last_crash.txt` im App-Data-Ordner mit Timestamp + Stack + Tauri-Version
- Frontend-Crashes (Error Boundary) → ebenfalls dort (`last_crash_frontend.txt`)
- Beim nächsten App-Start: stille Notification „Beim letzten Mal gab es einen Fehler. Crash-Log öffnen?"
- **Kein automatischer Versand** – User entscheidet manuell

---

## 11. Backup, Datensicherheit, DSGVO

### 11.1 Mechanismen

| Mechanismus | Wann | Wo |
|---|---|---|
| WAL-Modus | immer | SQLite-Pragma |
| Auto-Snapshot (rolling, max. 7) | täglich beim ersten Start | App-Data-Ordner |
| Pre-Restore-Snapshot | vor jedem Restore | App-Data-Ordner |
| Manueller Export | User-getriggert | User-gewählter Pfad |

### 11.2 DSGVO-Hinweise

App speichert Firmen-Kontaktdaten und teils personenbezogene Ansprechpartner-Namen. Da:
- Daten **nur lokal** auf Vaters PC liegen,
- die Daten ausschließlich für **B2B-Kontaktanbahnung** genutzt werden,
- keine Übertragung an Dritte stattfindet,

ist die Verarbeitung im Rahmen berechtigter Interessen (DSGVO Art. 6 Abs. 1 lit. f) zulässig. **Hinweis im Über-Tab:** „Lösch- und Auskunftsanfragen erfordern, dass Vater den Eintrag manuell entfernt." Lösch-Funktion pro Firma verfügbar.

---

## 12. Deployment

### 12.1 Lokale Entwicklung (Mac)
```bash
pnpm install
pnpm tauri dev
```

### 12.2 CI / Cross-Platform-Build
GitHub Actions Workflow:
- Trigger: Push auf `main` (Tests), Tag `v*` (Build + Release)
- Matrix: `macos-latest`, `windows-latest`
- Steps: install Node + Rust + pnpm → tests → `pnpm tauri build` → Upload Artifacts → bei Tag: GitHub Release mit `.msi` und `.dmg`

### 12.3 Erst-Installation Vater
- Download `.msi` von GitHub Release
- Doppelklick → Windows SmartScreen-Warnung („Trotzdem ausführen", weil unsigniert)
- Installation → Startmenü-Eintrag
- Beim ersten Start: leere DB wird mit Seed-Branchen erstellt

### 12.4 Auto-Updates
Tauri-Updater via GitHub Releases. App prüft beim Start, fragt bei Verfügbarkeit, lädt im Hintergrund.

---

## 13. Projekt-Struktur

```
projektalpha/
├─ src/                      # React Frontend (TypeScript)
│  ├─ pages/                 # Dashboard, Firmen, Karte, Profile, Settings
│  ├─ components/            # FirmaCard, MapView, FilterBar, ActivityLog, …
│  ├─ stores/                # Zustand (filter-store, ui-store)
│  ├─ lib/                   # Tauri-IPC-Wrapper, Formatters, Constants
│  └─ App.tsx
├─ src-tauri/                # Rust Core
│  ├─ src/
│  │  ├─ db/                 # schema, migrations, companies, activity, …
│  │  ├─ overpass/           # query_builder, tile_splitter, client, parser
│  │  ├─ nominatim/          # client + cache
│  │  ├─ backup/             # snapshot, restore
│  │  ├─ commands.rs         # Tauri-Commands (UI ↔ Core API)
│  │  └─ main.rs
│  └─ tauri.conf.json
├─ docs/superpowers/specs/   # Diese Spec
├─ .github/workflows/        # ci.yml, release.yml
├─ tests/                    # E2E (Playwright)
└─ package.json
```

---

## 14. Phasen-Plan

| Phase | Inhalt | Liefer-Ergebnis |
|---|---|---|
| **0 — Setup** | Tauri-Projekt initialisieren, GitHub Actions, SQLite + Migrations, shadcn/ui-Setup, Theme | App startet lokal, leere UI mit Sidebar |
| **1 — Core-Suche** | Overpass-Integration, Tile-Splitter, Branchen-Mapping, DB-Insert, Seed-Daten | CLI-test: Suche mit Hannover 50 km liefert echte Firmen in DB |
| **2 — Liste + Detail** | Firmen-Liste mit Filter, Detail-Sheet, Status-Updates, Activity-Log | Vater kann mit Daten aus Phase 1 produktiv arbeiten |
| **3 — Karte** | MapLibre, Pins synchronisiert mit Liste, Mittelpunkt-Picker | Karten-Ansicht funktional |
| **4 — Profile + Einstellungen** | Such-Profile, Branchen-Editor, Backup/Restore | Konfiguration komplett |
| **5 — Dashboard + Wiedervorlage** | KPIs, Today-Liste, Toast beim App-Start | Vater wird an Wiedervorlagen erinnert |
| **6 — Polish + Release** | Empty-/Loading-States, Animations, Auto-Updater, README, Erst-Release | Vater bekommt Download-Link |

Jede Phase ist für sich abgeschlossen; nach Phase 2 ist die App produktiv nutzbar.

---

## 15. Offene Punkte / Risiken

| Punkt | Risiko | Mitigation |
|---|---|---|
| OSM-Datenqualität in DE | Mittel — kleine Firmen oft fehlend | F11 (manuelles Hinzufügen) deckt das ab |
| Overpass-Public-Server-Verfügbarkeit | Niedrig–mittel — gelegentlich überlastet | Endpoint-Rotation + Retry |
| Score-Treffsicherheit der Heuristik | Mittel — kann irreführen | Score ist nur Hinweis; Vater entscheidet, F12 erlaubt Justage |
| SmartScreen-Warnung Windows | Niedrig — nur kosmetisch | Doku im README, ggf. später Code-Signing |
| Mac-Entwickler testet Windows-Build | Mittel | GitHub-Actions-Build + Windows-VM (UTM/Parallels) für Smoke-Test |
| Nominatim-Rate-Limit | Niedrig | Cache + Debounce decken einzelnen Nutzer locker ab |

---

## 16. Out-of-Scope / Bewusst weggelassen

- **CSV-Export:** kann später kommen, nicht im MVP
- **LLM-Qualifikation:** explizit nicht gewollt
- **Multi-User / Cloud-Sync:** nicht benötigt
- **Mobile-Companion:** kein Bedarf
- **Code-Signing-Zertifikat:** Kostengrund
- **Web-Scraping von Firmen-Websites:** keine Auto-Anreicherung jenseits OSM

---

## 17. Anhang: Glossar

| Begriff | Bedeutung |
|---|---|
| OSM | OpenStreetMap |
| Overpass | OSM-Query-API für strukturierte Daten |
| Nominatim | OSM-Geocoding-Service |
| OSM-Tag | Schlüssel-Wert-Paar an OSM-Objekten (z.B. `shop=wholesale`) |
| MSI | Windows-Installer-Format |
| WAL | Write-Ahead Logging (SQLite-Modus, crash-sicher) |
| UVV | Unfallverhütungsvorschriften (DGUV-Vorgaben für jährliche Prüfungen) |
| Wiedervorlage | Datum, an dem ein Lead erneut kontaktiert werden soll |
