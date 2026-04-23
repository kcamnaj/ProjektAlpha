# ProjectAlpha — Firmensuche für Tor- & Bühnen-Dienstleister

## Was ist das

Eine kostenlose, lokale Cross-Platform-Desktop-App (Tauri + React + SQLite), die B2B-Leads aus OpenStreetMap im konfigurierbaren Umkreis findet, nach Branchen-Heuristik bewertet und vollständiges Lead-Management bietet (Status, Notizen, Wiedervorlagen, Karte, Profile).

**Einziger Nutzer:** Vater des Entwicklers. Vertriebs-Tool für Industrie-Tore / Verlade- & Hubbühnen / UVV-Prüfungen.

**Vollständige Spec:** [`docs/superpowers/specs/2026-04-21-firmensuche-design.md`](docs/superpowers/specs/2026-04-21-firmensuche-design.md)

---

## Kern-Prinzipien (in dieser Reihenfolge)

### 1. User Experience steht an erster Stelle
Jede Design-Entscheidung wird daran gemessen, ob sie für **einen einzelnen Vertriebs-Nutzer im Arbeitsalltag** besser ist. Konkret heißt das:
- **Schnell startend, sofort reagierend** – kein Spinner ohne Grund, Liste/Karte erscheinen unter 1 s
- **Wenig Klicks** – häufige Aktionen (Status setzen, Notiz erfassen, anrufen) sind ein Klick weg
- **Verzeihend** – Aktionen sind reversibel oder fragen nach (insbesondere Lösch-Operationen)
- **Klare Sprache** – UI-Texte auf Deutsch, ohne Tech-Jargon
- **Erinnert statt versteckt** – Wiedervorlagen, ungespeicherte Änderungen, fehlerhafte Sucheingaben sind sichtbar

Wenn eine technisch elegante Lösung die UX verschlechtert: andere Lösung wählen.

### 2. Nicht unnötig kompliziert
- **YAGNI ruthless** – nichts bauen, was nicht in der Spec steht (siehe Sektion 16 „Out-of-Scope")
- **Standard-Patterns vor Eigenbau** – shadcn/ui-Komponenten statt Custom-CSS, `sqlx`-Queries statt Custom-ORM, Tauri-Commands statt eigenes IPC-Protokoll
- **Wenig Abstraktionen** – drei ähnliche Funktionen sind besser als ein generischer Wrapper, der alle drei abdeckt
- **Wenige Dependencies** – jede Dependency ist eine Verpflichtung; bei Ähnlichem zur stdlib greifen
- **Kleine Dateien** – siehe globale Rules (200–400 Zeilen typisch, max. 800)

### 3. Alles mit Tests und Logs absichern
- **Tests zuerst** (TDD): RED → GREEN → REFACTOR pro Modul, gemäß globaler Test-Rule (80 % Coverage)
- **Tests-Strategie** ist in der Spec, Sektion 10, festgelegt:
  - Rust: Unit + DB-Integration (In-Memory) + Overpass-Fixtures
  - React: Vitest + React Testing Library + Playwright (3 kritische E2E-Flows)
- **Strukturiertes Logging** in Rust mit `tracing` (JSON-Format in Datei, lesbares Format in Konsole)
  - Log-Level via `RUST_LOG` env (Default: `info`)
  - Log-Datei rotiert täglich, max. 7 Tage Aufbewahrung im App-Data-Ordner
  - **Pflicht-Log-Punkte:** Such-Start/Ende mit Stats, jede Overpass-Anfrage (Endpoint, Tile, HTTP-Status, Dauer), DB-Migrationen, Backup/Restore, jeder Fehler mit Kontext
  - **Niemals personenbezogene Daten** (Ansprechpartner-Namen, Mail-Adressen) in Logs
- **Frontend-Errors** über `console.error` + Tauri-Event an Backend → wandert ebenfalls ins Log
- **Crash-Reports lokal** – kein Telemetrie-Versand, aber Datei `last_crash.txt` mit Stack-Trace im App-Data-Ordner

---

## Tech-Stack (Quick Reference)

| | |
|---|---|
| Shell | Tauri 2 |
| Frontend | React 18 + TypeScript |
| UI | shadcn/ui + Tailwind CSS |
| State | Zustand |
| Karte | MapLibre GL JS |
| Animationen | Framer Motion |
| Backend | Rust |
| DB | SQLite (`sqlx`), WAL-Modus |
| HTTP | `reqwest` |
| Logging | `tracing` + `tracing-subscriber` |
| Tests Frontend | Vitest, React Testing Library, Playwright |
| Tests Backend | `cargo test` |
| CI/Build | GitHub Actions (Mac + Windows) |
| Distribution | GitHub Releases + Tauri-Updater |

---

## Konventionen

- **Code-Sprache:** Englisch (Variablennamen, Funktionsnamen, Code-Kommentare). **UI-Sprache:** Deutsch.
- **Datums-/Zeit-Format in UI:** Deutsch (`21.04.2026`, `14:30 Uhr`). Intern: ISO 8601 / UNIX-Timestamps.
- **DB-Schema-Änderungen:** immer als Migration in `src-tauri/src/db/migrations/`, nie destruktiv ohne Snapshot.
- **Tauri-Commands:** in `src-tauri/src/commands.rs` zentral, dünn (Logik im jeweiligen Modul, Command nur Validierung + Dispatch).
- **Keine Cloud-Calls** außer den drei dokumentierten OSM-Endpoints (Overpass, Nominatim, Tile-Server).
- **Keine personenbezogenen Daten in Logs / Crash-Reports / Telemetrie.**

---

## Daten- und Sicherheits-Regeln

- **Lokale Daten only.** Niemals automatischer Upload, kein Tracking, keine Analytics.
- **Manuelle Eingaben sind heilig.** OSM-Re-Imports überschreiben sie nie.
- **Vor jedem Restore:** Auto-Snapshot der aktuellen DB.
- **Sensitive Daten (E-Mail, Telefon, Ansprechpartner)** nie in Logs, nie in Crash-Reports, nie in Test-Fixtures (für Tests synthetische Daten).

---

## Build & Run (kurz)

```bash
# Erste Einrichtung
pnpm install

# Lokale Entwicklung (Mac)
pnpm tauri dev

# Tests
pnpm test                    # Frontend (Vitest)
pnpm test:e2e                # Playwright
cd src-tauri && cargo test   # Backend

# Production-Build (lokal, optional)
pnpm tauri build
```

CI baut automatisch Windows-`.msi` und Mac-`.dmg` bei Git-Tag `v*`.
