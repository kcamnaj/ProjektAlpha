# Phase 6B: Release-Infrastruktur — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Nach Plan 6B kann der Vater einen Download-Link in WhatsApp bekommen, die App von GitHub Releases ziehen (DMG auf Mac, MSI auf Windows), installieren, und bekommt zukünftige Updates ohne erneuten Download automatisch angeboten. Dev-Workflow bleibt dabei bewusst no-git — Git kommt nur für Release-Tags ins Spiel.

**Architecture:** Tauri-Updater-Plugin wird installiert und mit einem lokalen Ed25519-Keypair signiert (frei, kein OS-Store-Cert nötig). GitHub Actions baut auf Tag-Push (`v*`) parallel auf `macos-latest` und `windows-latest` via `tauri-apps/tauri-action@v0` — das produziert DMG + MSI + `latest.json`-Update-Manifest als Release-Assets. Das Frontend bekommt einen neuen „Über"-Tab-Bereich mit Update-Check-Button und startet zusätzlich einen Silent-Check 5 Sekunden nach App-Launch. End-User-README auf Deutsch erklärt Download, Install, und die einmalige Gatekeeper-/SmartScreen-Umgehung.

**Tech Stack:** `tauri-plugin-updater` (Rust crate v2) + `@tauri-apps/plugin-updater` (npm v2) + `tauri signer generate` (für Ed25519-Keypair) + GitHub Actions mit `tauri-apps/tauri-action@v0` (built-in `TAURI_SIGNING_PRIVATE_KEY`-Support) + Node-Script für Version-Bump in drei Dateien.

**Spec-Referenz:** `docs/superpowers/specs/2026-04-21-firmensuche-design.md`
- §14 Phase 6 (Auto-Updater, README, Erst-Release) · §15 (SmartScreen-Warnung-Mitigation) · §16 (Code-Signing-Zertifikat als Out-of-Scope)

**CLAUDE.md-Prinzipien:**
- **UX zuerst:** Für den Vater ist das allererste Öffnen des DMG/MSI die größte UX-Hürde. README muss eindeutig auf Deutsch beschreiben, was Gatekeeper und SmartScreen sagen und was er tut. Update-Check meldet sich nur, wenn ein Update verfügbar ist — kein „Du bist aktuell, Glückwunsch"-Toast beim Start.
- **Nicht kompliziert:** Wir nehmen `tauri-apps/tauri-action` statt selbstgeschriebenem Build-Script. Kein Code-Signing, kein Notarizing — akzeptieren Gatekeeper-/SmartScreen-Bypass als Einmalkosten. Kein separater Update-Server — GitHub Releases mit `latest.json` als Endpoint ist Standard.
- **Tests + Logs:** Das Release-Workflow ist selbst nicht Rust/TS-unit-testbar, aber das Version-Bump-Script bekommt einen kleinen Sanity-Check. App-Startup-Log enthält ohnehin schon `version = env!("CARGO_PKG_VERSION")` — das ist unser Release-Nachweis. Update-Check loggt Ergebnis (available/up-to-date/error) ohne PII.

**Scope-Begrenzung:**
- **Kein Paid Code-Signing** (Apple Developer $99/Jahr, Windows EV-Cert $300+). Doku im README klärt den Bypass. Kann zukünftig in einem Plan 7 nachgerüstet werden.
- **Kein eigener Update-Server.** GitHub Releases + `latest.json` reichen.
- **Linux-Target nicht aufgesetzt** (spec nennt nur Mac+Windows). CI-Matrix bleibt 2 Plattformen.
- **Keine E2E-Tests im CI.** Zu teuer für diesen Single-User-Fall. Manueller Smoke im Acceptance-Task.

**Kein Git (für Dev)** – Checkpoints statt Commits.
**Git nur für Release** – Ein initialer Commit, dann pro Release ein Version-Bump-Commit + Tag. Alles andere bleibt in-place-bearbeitet.

---

## Vier Release-Defaults, die dieser Plan festschreibt

Am Anfang dokumentiert, damit jederzeit reversibel:

1. **Git nur für Release.** Dev-Workflow bleibt no-git.
2. **Kein Paid Code-Signing.** macOS ad-hoc + Rechts-Klick-Öffnen; Windows unsigned + SmartScreen-Bypass.
3. **Tauri Auto-Updater AN.** Mit Tauri-eigener Ed25519-Signatur (frei, separat von OS-Code-Signing).
4. **GitHub-Repo public.** Vereinfacht Updater-Downloads (privat-mit-PAT ist nervig).

---

## Datei-Struktur (Ziel nach diesem Plan)

```
(Neu im Repo-Root)
.git/                                   # git init — nur für Release
.github/
└─ workflows/
   └─ release.yml                       # NEU: Tag-getriggerter Cross-Build
scripts/
└─ bump-version.mjs                     # NEU: Node-Script, setzt Version in 3 Dateien
README.md                               # REWRITE: End-User-Deutsch-README
RELEASE.md                              # NEU: Dev-Doku „so machst du ein Release"

(Geändert in src-tauri/)
├─ Cargo.toml                           # MODIFY: +tauri-plugin-updater
├─ tauri.conf.json                      # MODIFY: +plugins.updater {pubkey, endpoints}
├─ capabilities/default.json            # MODIFY: +"updater:default"
└─ src/
   └─ lib.rs                            # MODIFY: +plugin(updater) init

(Geändert in src/)
├─ lib/tauri.ts                         # MODIFY: +UpdaterStatus Typ + checkUpdate API-Wrapper
└─ components/settings/
   └─ UeberTab.tsx                      # MODIFY: Update-Check-Section mit Button + Status

(package.json)
├─ dependencies: +@tauri-apps/plugin-updater
└─ scripts: +"release:bump <version>" (aliased zu scripts/bump-version.mjs)

(Wichtig, nicht vergessen)
└─ .gitignore                           # MODIFY: +Tauri-Build-Artefakte (target/, *.dmg, *.msi, etc.)
```

**Nicht im Repo** (außerhalb committen):
- `~/.tauri/projektalpha.key` (privater Signing-Key für Updater-Manifest)
- `~/.tauri/projektalpha.key.pub` (öffentlicher Key — wird im `tauri.conf.json` embedded, das ist OK öffentlich)

---

# PHASE 6B — Release

## Reihenfolge-Hinweis

Die Tasks sind so geordnet, dass jeder für sich Zwischenzustand testbar ist. Task 6B.1 (Git) und 6B.6 (Release) sind die einzigen, wo Du aktiv Shell-Befehle außerhalb der üblichen Pnpm/Cargo-Befehle ausführst.

1. **6B.1:** Git + GitHub-Repo aufsetzen
2. **6B.2:** Version-Bump-Script + End-User-README
3. **6B.3:** Tauri-Updater-Plugin installieren
4. **6B.4:** Ed25519-Keypair + Updater-Config
5. **6B.5:** Frontend Update-Check im „Über"-Tab
6. **6B.6:** GitHub Actions Release-Workflow
7. **6B.7:** Erstes Release v0.1.0 + Acceptance

---

### Task 6B.1: Git + GitHub-Repo aufsetzen

**Files:**
- Create: `/Users/jan/Dev/Projects/ProjectAlpha/.gitignore` (existiert schon, aber Tauri-Artefakte müssen ergänzt werden)
- New: `.git/` Ordner (durch `git init`)

- [ ] **Step 1: `.gitignore` für Tauri erweitern**

Ans Ende von `/Users/jan/Dev/Projects/ProjectAlpha/.gitignore` anfügen (nach bestehenden Einträgen):

```gitignore

# Tauri build artifacts
src-tauri/target/
src-tauri/gen/
src-tauri/WixTools/

# Build outputs (DMG, MSI, app bundles)
*.dmg
*.msi
*.app
*.exe
*.AppImage
*.deb

# OS/IDE
.DS_Store
Thumbs.db
.vscode/
.idea/

# Local env / secrets (defensive, we don't have any yet)
.env
.env.local
.env.*.local

# Claude / Superpowers session metadata (keep in repo OR ignore? Decision: keep, it's useful context)
# (no ignore — docs/superpowers/** bleibt committed)
```

**Wichtig:** Die App-Daten des Benutzers (`~/Library/Application Support/projektalpha/`) liegen außerhalb des Repos und sind per se unbetroffen. Der Daten-Dir-Pfad muss NICHT in .gitignore.

- [ ] **Step 2: Git initialisieren**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && git init -b main
```

Erwartet: `Initialized empty Git repository in …/.git/` und Branch `main` als Default.

Falls Dein Git-Default `master` ist, erzwingt `-b main` trotzdem `main` — GitHub erwartet das.

- [ ] **Step 3: Identity setzen (lokal, falls global noch nicht konfiguriert)**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && git config user.name "Jan Mack" && git config user.email "jan-mack@web.de"
```

- [ ] **Step 4: Initialer Commit**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && git add -A && git status --short | head -40
```

Prüfen, dass **keine** `target/`, `node_modules/`, `dist/`, `*.dmg`, `*.msi` in der Staging-Liste auftauchen. Wenn doch → `.gitignore` hat ein Problem, fix und wiederhole.

Dann:

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && git commit -m "initial commit: ProjectAlpha Phases 0-6A

Baseline: Tauri 2 + React 19 + TypeScript + Rust + SQLite.
Plans 0 bis 6A implementiert (Foundation, Liste, Detail, Karte,
Nominatim, Settings, Profile, Backup, Dashboard, Polish).
82 Rust-Tests + 37 Frontend-Tests grün."
```

- [ ] **Step 5: GitHub-Repo erstellen und pushen**

Du brauchst `gh` (GitHub CLI). Falls nicht installiert: `brew install gh` und dann `gh auth login`.

Vom Terminal (du führst das interaktiv aus, weil `gh auth login` ein Browser-Login öffnet):

```bash
! gh repo create kcamnaj/ProjektAlpha --public --source=/Users/jan/Dev/Projects/ProjectAlpha --remote=origin --description "Firmensuche für Tor- & Bühnen-Dienstleister"
```

Erwartet: Output endet mit URL `https://github.com/kcamnaj/ProjektAlpha`.

- [ ] **Step 6: Push**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && git push -u origin main
```

- [ ] **Step 7: Verifikation**

Öffne `https://github.com/kcamnaj/ProjektAlpha` im Browser. Du musst sehen:
- `docs/`, `src/`, `src-tauri/`, `package.json`, `README.md`, `.gitignore`
- **KEIN** `node_modules/`, `dist/`, `src-tauri/target/`

Wenn Du doch `target/` oder `node_modules/` siehst → sofort STOP, Repo löschen (`gh repo delete kcamnaj/ProjektAlpha --yes`), `.gitignore` fixen, Task 6B.1 nochmal.

- [ ] **Step 8: Checkpoint 6B.1**

---

### Task 6B.2: Version-Bump-Script + End-User-README

**Files:**
- Create: `scripts/bump-version.mjs`
- Modify: `package.json` — neuer Script-Eintrag
- Rewrite: `README.md` — End-User-Doku auf Deutsch
- Create: `RELEASE.md` — Dev-Doku für Releases

- [ ] **Step 1: Verzeichnis anlegen**

```bash
mkdir -p /Users/jan/Dev/Projects/ProjectAlpha/scripts
```

- [ ] **Step 2: `bump-version.mjs` schreiben**

`/Users/jan/Dev/Projects/ProjectAlpha/scripts/bump-version.mjs`:

```javascript
#!/usr/bin/env node
// Bumps version in package.json, src-tauri/Cargo.toml, src-tauri/tauri.conf.json
// Usage: node scripts/bump-version.mjs 0.2.0
import { readFileSync, writeFileSync } from "node:fs"
import { resolve, dirname } from "node:path"
import { fileURLToPath } from "node:url"

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = resolve(__dirname, "..")

const newVersion = process.argv[2]
if (!newVersion || !/^\d+\.\d+\.\d+$/.test(newVersion)) {
  console.error("Usage: node scripts/bump-version.mjs X.Y.Z")
  console.error("Received:", newVersion ?? "(nothing)")
  process.exit(1)
}

// 1. package.json
{
  const file = resolve(root, "package.json")
  const j = JSON.parse(readFileSync(file, "utf8"))
  const prev = j.version
  j.version = newVersion
  writeFileSync(file, JSON.stringify(j, null, 2) + "\n")
  console.log(`package.json:        ${prev} → ${newVersion}`)
}

// 2. src-tauri/Cargo.toml — nur die [package]-version ersetzen, nicht die von Dependencies
{
  const file = resolve(root, "src-tauri/Cargo.toml")
  const src = readFileSync(file, "utf8")
  const re = /^(\[package\][\s\S]*?\nversion\s*=\s*")[^"]+(")/m
  const match = src.match(re)
  if (!match) {
    console.error("Could not find [package] version in Cargo.toml")
    process.exit(1)
  }
  const prev = src.match(/^\[package\][\s\S]*?\nversion\s*=\s*"([^"]+)"/m)[1]
  const out = src.replace(re, `$1${newVersion}$2`)
  writeFileSync(file, out)
  console.log(`Cargo.toml:          ${prev} → ${newVersion}`)
}

// 3. src-tauri/tauri.conf.json — nur top-level "version"
{
  const file = resolve(root, "src-tauri/tauri.conf.json")
  const j = JSON.parse(readFileSync(file, "utf8"))
  const prev = j.version
  j.version = newVersion
  writeFileSync(file, JSON.stringify(j, null, 2) + "\n")
  console.log(`tauri.conf.json:     ${prev} → ${newVersion}`)
}

console.log(`\n✓ Version bumped to ${newVersion}`)
console.log(`Next: git add -A && git commit -m "release: v${newVersion}" && git tag v${newVersion} && git push && git push --tags`)
```

- [ ] **Step 3: `package.json` Script-Eintrag**

In `package.json`, `scripts`-Abschnitt erweitern:

```json
"scripts": {
  "dev": "vite",
  "build": "tsc && vite build",
  "preview": "vite preview",
  "tauri": "tauri",
  "test": "vitest run",
  "test:watch": "vitest",
  "test:ui": "vitest --ui",
  "release:bump": "node scripts/bump-version.mjs"
}
```

Achte darauf, `"release:bump"` korrekt als letzten Eintrag einzufügen (Komma vor dem neuen Eintrag, nicht nach).

- [ ] **Step 4: Script-Sanity-Check**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && node scripts/bump-version.mjs 0.1.0
```

Erwartete Ausgabe:
```
package.json:        0.1.0 → 0.1.0
Cargo.toml:          0.1.0 → 0.1.0
tauri.conf.json:     0.1.0 → 0.1.0

✓ Version bumped to 0.1.0
```

Drei identische Werte, keine echte Änderung — aber beweist, dass das Script alle drei Dateien findet und parst. Anschließend ein Test mit ungültigem Input:

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && node scripts/bump-version.mjs foo 2>&1 | head -3
```

Erwartet: Fehler `Usage: node scripts/bump-version.mjs X.Y.Z`, Exit-Code ≠ 0.

- [ ] **Step 5: `README.md` — End-User-Deutsch-Doku**

Ersetze den kompletten Inhalt von `/Users/jan/Dev/Projects/ProjectAlpha/README.md`:

```markdown
# ProjektAlpha

Firmensuche für Tor- & Bühnen-Dienstleister. Findet B2B-Leads aus
OpenStreetMap im konfigurierbaren Umkreis, bewertet sie nach Branche
und hilft dabei, Kontakt, Status und Wiedervorlagen zu verwalten.

**Plattformen:** macOS (Apple Silicon + Intel) und Windows 10/11.

---

## Installation

### macOS

1. Lade die neueste `.dmg`-Datei von [Releases](https://github.com/kcamnaj/ProjektAlpha/releases/latest).
2. Öffne die DMG-Datei mit Doppelklick.
3. Ziehe `ProjektAlpha.app` in den Programme-Ordner.
4. **Beim ersten Öffnen:** Rechts-Klick auf `ProjektAlpha.app` → „Öffnen".
   macOS fragt dich nach Bestätigung, weil die App nicht bei Apple
   registriert ist. Bestätige einmal — ab dann öffnet sich die App
   per Doppelklick wie jede andere.

### Windows

1. Lade die neueste `.msi`-Datei von [Releases](https://github.com/kcamnaj/ProjektAlpha/releases/latest).
2. Doppelklick auf die MSI-Datei.
3. **SmartScreen-Warnung:** Windows warnt vor einer unbekannten App.
   Klicke auf „Mehr Info" → „Trotzdem ausführen". Das ist einmalig pro
   App-Version.
4. Installer folgt — ProjektAlpha liegt danach im Startmenü.

---

## Erster Start

Die App startet auf dem Dashboard. Dort siehst du vier Zahlen:
Kunden / Angefragt / Neu / Durchschnitts-Score. Und — sobald du eine
Suche gemacht hast — „Heute fällig"-Wiedervorlagen oben.

**So kommst du an die ersten Firmen:**

1. Seitenleiste → **„Neue Suche"**
2. Adresse tippen (z. B. „Hannover") → einen der Vorschläge anklicken.
3. Umkreis einstellen (1–300 km) und Branchen auswählen.
4. „Suche starten" klicken.

Die App lädt Daten von OpenStreetMap. Das kann je nach Umkreis ein
paar Sekunden bis etwa eine Minute dauern.

---

## Daten und Datenschutz

- **Alles bleibt lokal.** ProjektAlpha speichert alle Firmen, Notizen
  und Einstellungen in einer SQLite-Datei auf deinem Rechner
  (`~/Library/Application Support/projektalpha/` auf macOS, bzw.
  `%APPDATA%\projektalpha\` auf Windows).
- **Keine Cloud.** Es findet keine automatische Synchronisierung statt.
- **Nur drei Server werden angesprochen:**
  OpenStreetMap Overpass (Firmen-Suche), Nominatim (Adress-Suche) und
  OpenStreetMap-Tiles (Karten-Hintergrund).

### Backup

Einstellungen → Daten → „Backup erstellen" öffnet einen Speicher-Dialog.
Wähle einen Ort (z. B. iCloud Drive oder einen USB-Stick) und speichere
die `.db`-Datei. Im gleichen Tab kannst du später per „Datenbank
wiederherstellen" die Backup-Datei zurückspielen.

---

## Updates

ProjektAlpha prüft im Hintergrund bei jedem Start, ob eine neue Version
verfügbar ist. Wenn ja, erscheint eine Benachrichtigung in
Einstellungen → Über. Mit einem Klick lädst du das Update herunter und
die App startet mit der neuen Version.

Du kannst Updates jederzeit manuell mit Einstellungen → Über → „Nach
Updates suchen" prüfen.

---

## Fehler / Probleme

Bei Abstürzen oder merkwürdigem Verhalten schreibt ProjektAlpha Logs:

- **macOS:** `~/Library/Application Support/projektalpha/logs/`
- **Windows:** `%APPDATA%\projektalpha\logs\`

Schicke diese Log-Dateien bitte mit, wenn du einen Fehler meldest.

Log-Dateien enthalten keine persönlichen Daten von Kontakten — nur
technische Ereignisse (Such-Zeiten, Datenbank-Fehler, HTTP-Status).

---

## Lizenz & Mitwirken

ProjektAlpha ist ein privat entwickeltes Werkzeug für den Familienbetrieb.
Source-Code liegt aus Nachvollziehbarkeits-Gründen auf GitHub; externe
Beiträge sind derzeit nicht vorgesehen.
```

- [ ] **Step 6: `RELEASE.md` — Dev-Doku**

`/Users/jan/Dev/Projects/ProjectAlpha/RELEASE.md`:

```markdown
# Release-Workflow

## Regulärer Release

1. **Version bumpen** (setzt package.json, src-tauri/Cargo.toml, src-tauri/tauri.conf.json):
   ```bash
   pnpm run release:bump 0.2.0
   ```

2. **Committen + taggen + pushen:**
   ```bash
   git add -A
   git commit -m "release: v0.2.0"
   git tag v0.2.0
   git push && git push --tags
   ```

3. **GitHub Actions** baut automatisch macOS DMG + Windows MSI und
   lädt sie als Release-Assets hoch, inklusive `latest.json` für den
   Auto-Updater. Build-Dauer: ~8–15 Minuten.

4. **Artefakte prüfen** unter
   `https://github.com/kcamnaj/ProjektAlpha/releases/tag/v0.2.0`.

5. **Smoke-Test** auf Mac und Windows (manueller Download + Install).

## Privater Updater-Signing-Key

Der private Schlüssel zum Signieren des Update-Manifests liegt unter
`~/.tauri/projektalpha.key` (Passphrase verschlüsselt). **Verliere den
nicht** — sonst können bestehende Installationen keine Updates mehr
verifizieren und müssen manuell die neue Version installieren.

Backup des Keys: kopiere `~/.tauri/projektalpha.key` und
`~/.tauri/projektalpha.key.pub` auf einen USB-Stick oder in einen
Passwort-Manager.

Für GitHub Actions ist der Private Key als Repository Secret
`TAURI_SIGNING_PRIVATE_KEY` gespeichert (siehe Task 6B.6).

## Keine Code-Signing-Zertifikate aktuell

macOS: Keine Apple Developer ID. Benutzer müssen beim Erstöffnen
Rechts-Klick → Öffnen benutzen (siehe README).

Windows: Kein EV-Cert. SmartScreen-Warnung beim Install — „Mehr Info"
→ „Trotzdem ausführen".

Falls später Code-Signing gewünscht ist: das wäre Plan 7 (Apple
Developer Account + Notarization für macOS; EV-Cert + sign-step
in release.yml für Windows).
```

- [ ] **Step 7: Dateien-Sanity**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && ls -la scripts/ README.md RELEASE.md && node scripts/bump-version.mjs 2>&1 | head -3
```

Erwartet: Script-Datei, README, RELEASE existieren. Script ohne Arg zeigt Usage.

- [ ] **Step 8: Checkpoint 6B.2** — keine Commits, sammeln für 6B.7

---

### Task 6B.3: Tauri-Updater-Plugin installieren

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `package.json`
- Modify: `src-tauri/src/lib.rs` (Plugin-Init)
- Modify: `src-tauri/capabilities/default.json`

- [ ] **Step 1: Rust-Seite — `tauri-plugin-updater` hinzufügen**

`src-tauri/Cargo.toml`, `[dependencies]`-Abschnitt, unterhalb des bestehenden `tauri`-Eintrags ergänzen:

```toml
tauri-plugin-updater = "2"
```

- [ ] **Step 2: Frontend-Seite — `@tauri-apps/plugin-updater` hinzufügen**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm add @tauri-apps/plugin-updater
```

Erwartet: Paket wird in `package.json` dependencies eingetragen.

- [ ] **Step 3: Plugin in `lib.rs` registrieren**

In `src-tauri/src/lib.rs::run()`, den `tauri::Builder::default()`-Block erweitern — an der Stelle wo bereits `.plugin(tauri_plugin_opener::init())` und `.plugin(tauri_plugin_dialog::init())` stehen, noch eine Zeile einfügen:

```rust
.plugin(tauri_plugin_updater::Builder::new().build())
```

Die vollständige Plugin-Kette sieht dann etwa so aus:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_updater::Builder::new().build())
    .manage(Arc::new(AppState { /* … */ }))
    .invoke_handler(tauri::generate_handler![/* … */])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
```

- [ ] **Step 4: Capabilities erweitern**

`src-tauri/capabilities/default.json`, `permissions`-Array:

```json
{
  "$schema": "./schemas/default.json",
  "identifier": "default",
  "description": "...",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default",
    "dialog:default",
    "updater:default"
  ]
}
```

Reihenfolge nicht kritisch, aber alphabetisch nach `dialog:default` einfügen ist konsistent.

- [ ] **Step 5: Build**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo build --lib 2>&1 | tail -10
```

Erwartet: `Finished` ohne Errors. Es gibt eine Compiler-Warnung, wenn der Updater-Plugin-Config noch nicht in `tauri.conf.json` ist — das kommt in Task 6B.4. Die Warnung ist OK, aber kein Error.

- [ ] **Step 6: Rust-Test-Suite**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib 2>&1 | tail -5
```

Erwartet: **82 passed** (unverändert).

- [ ] **Step 7: TS-Check**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm exec tsc --noEmit 2>&1 | tail -5
```

Erwartet: clean.

- [ ] **Step 8: Checkpoint 6B.3**

---

### Task 6B.4: Ed25519-Keypair + Updater-Config

**Files:**
- Create (außerhalb des Repos): `~/.tauri/projektalpha.key` + `.key.pub`
- Modify: `src-tauri/tauri.conf.json` — `plugins.updater` Block einfügen

- [ ] **Step 1: Keypair generieren**

```bash
mkdir -p ~/.tauri && cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm tauri signer generate -w ~/.tauri/projektalpha.key
```

Das Tool fragt nach einer Passphrase — **setze eine, und schreibe sie dir auf** (sie kommt später als GitHub-Secret unter `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` und lokal in Deinen Passwort-Manager). Notiere Dir die Passphrase jetzt irgendwo sicher.

Output enthält den **Public Key** — Kopiere ihn. Sieht etwa so aus:
```
dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IEFCQ0QxMjM0...
```

Verifizieren, dass Dateien da sind:
```bash
ls -la ~/.tauri/projektalpha.key*
```
Erwartet: zwei Dateien, `projektalpha.key` (private) und `projektalpha.key.pub` (public).

- [ ] **Step 2: Public Key erneut ausgeben (falls Du den aus Step 1 vergessen hast)**

```bash
cat ~/.tauri/projektalpha.key.pub
```

Kopiere die eine lange Zeile (ohne umschließende Anführungszeichen).

- [ ] **Step 3: `tauri.conf.json` Updater-Config**

Öffne `/Users/jan/Dev/Projects/ProjectAlpha/src-tauri/tauri.conf.json`. Suche nach dem `"plugins"`-Objekt — falls es nicht existiert, lege es auf Top-Level an (parallel zu `"bundle"`, `"identifier"` etc.):

```json
{
  ...
  "plugins": {
    "updater": {
      "pubkey": "DEIN_PUBLIC_KEY_AUS_STEP_2_HIER",
      "endpoints": [
        "https://github.com/kcamnaj/ProjektAlpha/releases/latest/download/latest.json"
      ]
    }
  }
}
```

**Parameter-Erklärung:**
- `pubkey` — Dein Public Key aus Step 2 (ja, der darf öffentlich sein, das ist der ganze Sinn asymmetrischer Kryptografie)
- `endpoints` — URL zum Manifest, das GitHub Actions später erzeugt. Die `latest.json`-Datei wird als Release-Asset an `latest` angehängt, deswegen dieser Pfad.

Das v2-Updater-Plugin hat **keinen** Default-Dialog (Tauri 1 hatte einen, aber v2 liefert nur die API — die UI baut man selbst). Die Update-Check-UI kommt daher in Task 6B.5 als reguläre Komponente im Über-Tab.

Falls `"plugins"` bereits existiert (unwahrscheinlich), füge nur den `"updater"`-Unterschlüssel hinzu.

- [ ] **Step 4: Build**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo build --lib 2>&1 | tail -10
```

Erwartet: Sauberer Build. Die Warnung aus Task 6B.3 über fehlenden Updater-Config sollte jetzt weg sein.

- [ ] **Step 5: Tauri Config Validieren**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm exec tauri info 2>&1 | tail -20
```

Erwartet: Keine Parse-Errors über `tauri.conf.json`; Versionen der Tauri-Plugins werden gelistet. (Das Tool listet nicht explizit „Plugin geladen" — es prüft nur Config-Syntax und Dependency-Versionen. Wenn kein Error kommt, ist die Config valide.)

- [ ] **Step 6: Checkpoint 6B.4** — privater Key ist NICHT im Repo, prüfen:

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && git status --short | grep -i "\.key" && echo "FAIL: key file found in repo" || echo "OK: no key files in repo status"
```

Erwartet: `OK: no key files in repo status`. Falls `FAIL` → das Key-File ist versehentlich ins Repo gelandet (sollte nicht, weil Du es in `~/.tauri/` erzeugt hast) → debuggen.

---

### Task 6B.5: Frontend Update-Check im „Über"-Tab

**Files:**
- Modify: `src-tauri/Cargo.toml` — add `tauri-plugin-process`
- Modify: `src-tauri/src/lib.rs` — plugin init
- Modify: `src-tauri/capabilities/default.json` — add `process:default`
- Modify: `package.json` — add `@tauri-apps/plugin-process`
- Modify: `src/lib/tauri.ts` — API-Wrapper
- Modify: `src/components/settings/UeberTab.tsx` — Update-Check-Section

**Task 6B.5 braucht zusätzlich das `tauri-plugin-process`-Plugin, weil wir nach einem Update-Install `relaunch()` aufrufen müssen. Installation zuerst, TS-Code danach.**

- [ ] **Step 1: `tauri-plugin-process` installieren (Rust)**

`src-tauri/Cargo.toml`, `[dependencies]`-Abschnitt, bei den anderen `tauri-plugin-*`-Einträgen ergänzen:

```toml
tauri-plugin-process = "2"
```

In `src-tauri/src/lib.rs::run()`, im `tauri::Builder::default()`-Block die bestehenden `.plugin(...)`-Aufrufe um einen erweitern:

```rust
.plugin(tauri_plugin_process::init())
```

In `src-tauri/capabilities/default.json`, `permissions`-Array:

```json
"core:default",
"opener:default",
"dialog:default",
"updater:default",
"process:default"
```

Build-Check:

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo build --lib 2>&1 | tail -5
```

Erwartet: Clean build.

- [ ] **Step 2: `@tauri-apps/plugin-process` installieren (Frontend)**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm add @tauri-apps/plugin-process
```

Erwartet: Paket in `package.json` dependencies.

- [ ] **Step 3: API-Wrapper in `tauri.ts`**

Am Ende von `src/lib/tauri.ts`, nach dem `api`-Objekt, ein neuer Block:

```typescript
// Updater API — wraps @tauri-apps/plugin-updater
import { check, type Update } from "@tauri-apps/plugin-updater"
import { relaunch } from "@tauri-apps/plugin-process"

export type UpdaterResult =
  | { kind: "up-to-date" }
  | { kind: "update-available"; version: string; currentVersion: string; notes: string | null; update: Update }
  | { kind: "error"; message: string }

export async function checkForUpdate(): Promise<UpdaterResult> {
  try {
    const update = await check()
    if (update === null) {
      return { kind: "up-to-date" }
    }
    return {
      kind: "update-available",
      version: update.version,
      currentVersion: update.currentVersion,
      notes: update.body ?? null,
      update,
    }
  } catch (e) {
    return { kind: "error", message: String(e) }
  }
}

export async function installUpdate(update: Update, onProgress?: (downloadedBytes: number, total: number | null) => void): Promise<void> {
  let downloaded = 0
  await update.downloadAndInstall((event) => {
    switch (event.event) {
      case "Started":
        downloaded = 0
        if (onProgress) onProgress(0, event.data.contentLength ?? null)
        break
      case "Progress":
        downloaded += event.data.chunkLength
        if (onProgress) onProgress(downloaded, null)
        break
      case "Finished":
        break
    }
  })
  // Nach Install App neu starten
  await relaunch()
}
```

- [ ] **Step 4: Über-Tab erweitern**

Aktueller Inhalt von `src/components/settings/UeberTab.tsx` ansehen — vermutlich zeigt er nur Version + Kurzbeschreibung. Erweitere ihn um eine Update-Sektion:

```tsx
import { useEffect, useState } from "react"
import { api, checkForUpdate, installUpdate, type UpdaterResult } from "@/lib/tauri"
import { Button } from "@/components/ui/button"
import { logger } from "@/lib/logger"
import { CheckCircle2, Download, AlertCircle, RefreshCw } from "lucide-react"

export function UeberTab() {
  const [version, setVersion] = useState<string>("…")
  const [updater, setUpdater] = useState<UpdaterResult | null>(null)
  const [busy, setBusy] = useState<"check" | "install" | null>(null)
  const [progress, setProgress] = useState<{ downloaded: number; total: number | null } | null>(null)

  useEffect(() => {
    api.appVersion().then(setVersion)
    // Silent check on mount — 5s delayed, damit die App erst ganz startet
    const t = setTimeout(() => { void doCheck(true) }, 5000)
    return () => clearTimeout(t)
  }, [])

  const doCheck = async (silent = false) => {
    setBusy("check")
    const result = await checkForUpdate()
    setUpdater(result)
    setBusy(null)
    if (silent && result.kind === "up-to-date") {
      // Bei Silent-Check keine UI-Reaktion wenn alles ok
      logger.info("silent update check: up-to-date", { version })
      return
    }
    logger.info("update check", { kind: result.kind })
  }

  const doInstall = async () => {
    if (updater?.kind !== "update-available") return
    setBusy("install")
    setProgress({ downloaded: 0, total: null })
    try {
      await installUpdate(updater.update, (downloaded, total) => {
        setProgress({ downloaded, total })
      })
      // Bei Erfolg restartet die App — dieser Code-Pfad wird nicht mehr erreicht
    } catch (e) {
      logger.error("update install failed", { e: String(e) })
      setUpdater({ kind: "error", message: String(e) })
      setBusy(null)
    }
  }

  return (
    <div className="flex flex-col gap-6 max-w-2xl">
      <div>
        <h3 className="font-semibold mb-1">ProjektAlpha</h3>
        <p className="text-sm text-muted-foreground">
          Firmensuche für Tor- & Bühnen-Dienstleister. OpenStreetMap-basierte
          B2B-Lead-Generierung mit Status- und Wiedervorlage-Management.
        </p>
        <p className="text-xs text-muted-foreground mt-2">Version {version}</p>
      </div>

      <div className="border-t pt-4">
        <h4 className="font-medium mb-2">Updates</h4>
        <div className="flex items-center gap-3 mb-3">
          <Button variant="outline" size="sm" onClick={() => doCheck(false)} disabled={busy !== null}>
            <RefreshCw className={busy === "check" ? "size-4 animate-spin" : "size-4"} />
            Nach Updates suchen
          </Button>
        </div>

        {updater?.kind === "up-to-date" && (
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <CheckCircle2 className="size-4 text-green-600" />
            Du bist auf der aktuellen Version ({version}).
          </div>
        )}

        {updater?.kind === "update-available" && (
          <div className="flex flex-col gap-2 p-3 border rounded-md bg-accent/30">
            <div className="flex items-center gap-2 text-sm font-medium">
              <Download className="size-4" />
              Update verfügbar: {updater.version}
            </div>
            {updater.notes && (
              <pre className="text-xs text-muted-foreground whitespace-pre-wrap max-h-32 overflow-y-auto">
                {updater.notes}
              </pre>
            )}
            <div className="flex items-center gap-3">
              <Button size="sm" onClick={doInstall} disabled={busy === "install"}>
                {busy === "install" ? "Lade…" : "Update jetzt installieren"}
              </Button>
              {progress && (
                <span className="text-xs text-muted-foreground">
                  {progress.total
                    ? `${Math.round((progress.downloaded / progress.total) * 100)} %`
                    : `${Math.round(progress.downloaded / 1024)} KB`}
                </span>
              )}
            </div>
          </div>
        )}

        {updater?.kind === "error" && (
          <div className="flex items-start gap-2 text-sm text-destructive">
            <AlertCircle className="size-4 mt-0.5" />
            <div>
              <div className="font-medium">Update-Check fehlgeschlagen</div>
              <div className="text-xs text-muted-foreground break-all">{updater.message}</div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
```

- [ ] **Step 5: TS-Check + Tests**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm exec tsc --noEmit 2>&1 | tail -5 && pnpm test 2>&1 | tail -5
```

Erwartet: TS clean, 37 tests passed (keine neuen Tests in diesem Task — Update-Check ist integration-level, manueller Smoke in 6B.7).

- [ ] **Step 6: Vite-Build**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm vite build 2>&1 | tail -5
```

Erwartet: clean.

- [ ] **Step 7: Local Dev-Test**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm tauri dev
```

Navigiere zu Einstellungen → Über. Klicke „Nach Updates suchen". Erwartet:
- Kein Crash
- Bei noch nicht veröffentlichtem Release: Fehler „404 Not Found" oder ähnlich (weil `latest.json` noch nicht auf GitHub liegt) — das ist erwartet und wird nach Task 6B.7 verschwinden.
- Error-State erscheint in der UI

Stoppe mit Ctrl+C.

- [ ] **Step 8: Checkpoint 6B.5**

---

### Task 6B.6: GitHub Actions Release-Workflow

**Files:**
- Create: `.github/workflows/release.yml`
- Set up GitHub Secrets (manuell im GitHub-UI)

- [ ] **Step 1: Secrets setzen**

Gehe zu `https://github.com/kcamnaj/ProjektAlpha/settings/secrets/actions` und lege zwei neue Repository-Secrets an:

1. **`TAURI_SIGNING_PRIVATE_KEY`** — Inhalt von `~/.tauri/projektalpha.key` (die gesamte Datei per `cat ~/.tauri/projektalpha.key` kopieren und als Secret-Value einfügen). Das ist eine lange mehrzeilige Base64-Zeichenkette.

2. **`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`** — die Passphrase, die Du in Task 6B.4 Step 1 gesetzt hast.

Falls Du die Passphrase vergessen hast: Task 6B.4 nochmal ausführen (neuer Key = alle bestehenden Installs bekommen neue Update-Quelle, ist beim ersten Release aber egal weil es noch keine Installs gibt).

- [ ] **Step 2: Workflow-Datei anlegen**

```bash
mkdir -p /Users/jan/Dev/Projects/ProjectAlpha/.github/workflows
```

`/Users/jan/Dev/Projects/ProjectAlpha/.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - "v*"

permissions:
  contents: write

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: macos-latest
            args: "--target aarch64-apple-darwin"
            rust-target: aarch64-apple-darwin
          - platform: macos-latest
            args: "--target x86_64-apple-darwin"
            rust-target: x86_64-apple-darwin
          - platform: windows-latest
            args: "--bundles msi"
            rust-target: x86_64-pc-windows-msvc

    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4

      - name: Install pnpm
        uses: pnpm/action-setup@v4
        with:
          version: 9

      - name: Install Node
        uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.rust-target }}

      - name: Rust cache
        uses: swatinem/rust-cache@v2
        with:
          workspaces: "./src-tauri -> target"

      - name: Install frontend deps
        run: pnpm install --frozen-lockfile

      - name: Build + bundle + publish
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: "ProjektAlpha ${{ github.ref_name }}"
          releaseBody: "Automatisch gebaut via GitHub Actions. Siehe README für Install-Hinweise."
          releaseDraft: true
          prerelease: false
          args: ${{ matrix.args }}
```

**Wichtig:**
- `releaseDraft: true` — der Release wird als Draft erstellt. Du prüfst vor Veröffentlichung die Assets und klickst dann manuell „Publish". Das ist eine Sicherheitsnetz-Einstellung.
- Mac baut beide Architekturen separat — dadurch zwei DMGs pro Release (`arm64` und `x64`). Das ist kompatibler als ein Universal-Binary, weil Tauri-Updater auf Architektur-Ebene matched.
- Windows baut nur `x86_64` (Arm-Windows-Support ist noch experimentell, überspringen wir).
- Windows-`args: "--bundles msi"` erzwingt **nur** MSI (Tauri würde sonst per Default auch NSIS bauen und tauri-action würde ggf. NSIS statt MSI ins `latest.json` eintragen — für den Vater ist MSI der erwartete Installer-Typ, deshalb explizit festgenagelt).

**Falls ein Build fehlschlägt und einen kaputten Draft-Release hinterlässt:** bevor Du denselben Tag erneut pushst, musst Du den Draft-Release vorher löschen (GitHub UI → Releases → „⋯" → Delete release). Sonst bricht `tauri-action` beim nächsten Run mit „release already exists" ab.

- [ ] **Step 3: Workflow syntax-check**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && gh workflow view 2>&1 || echo "workflow not yet pushed — that's OK"
```

- [ ] **Step 4: Commit vorbereitend**

Sammle die Changes von 6B.1 bis 6B.6 in einem Commit — den machen wir in 6B.7, zusammen mit dem Version-Tag.

- [ ] **Step 5: Checkpoint 6B.6** — Workflow ist angelegt, wird in 6B.7 scharf geschaltet.

---

### Task 6B.7: Erstes Release v0.1.0 + Acceptance

**Files:** keine neuen. Git-Operationen + manuelles Testing.

- [ ] **Step 1: Finale lokale Sanity**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha/src-tauri && cargo test --lib 2>&1 | tail -3
```
Erwartet: **82 passed**.

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm test 2>&1 | tail -5
```
Erwartet: **37 passed**.

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm exec tsc --noEmit 2>&1 | tail -3
```
Erwartet: clean.

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && pnpm vite build 2>&1 | tail -5
```
Erwartet: ✓ built.

- [ ] **Step 2: Alle Plan-6B-Changes commiten**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && git add -A && git status --short
```

Du solltest sehen: `.github/workflows/release.yml`, `scripts/bump-version.mjs`, `README.md`, `RELEASE.md`, Änderungen an `package.json`, `pnpm-lock.yaml`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`, `src-tauri/src/lib.rs`, `src/lib/tauri.ts`, `src/components/settings/UeberTab.tsx`, Plan-Dokumente.

Commit:

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && git commit -m "release infra: updater plugin + GH Actions + v0.1.0 prep

- Tauri Updater plugin installed (Rust + TS + capabilities)
- Ed25519 signing key outside repo; public key in tauri.conf.json
- GitHub Actions matrix build (macOS arm64/x64, Windows x64)
- Auto-Update check in Settings -> Über
- End-user README (Deutsch) + RELEASE.md (Dev workflow)
- Version bump helper: pnpm run release:bump <X.Y.Z>"
```

- [ ] **Step 3: Tag erstellen und pushen**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && git tag v0.1.0 && git push && git push --tags
```

Das triggert GitHub Actions.

- [ ] **Step 4: Build beobachten**

```bash
cd /Users/jan/Dev/Projects/ProjectAlpha && gh run list --workflow=release.yml --limit 3
```

Oder im Browser: `https://github.com/kcamnaj/ProjektAlpha/actions`.

Build-Dauer: ~8–15 Minuten (Mac arm64 + Mac x64 + Windows parallel, mit Rust-Cache beim zweiten Run deutlich schneller).

Bei Failures: `gh run view --log-failed` zeigt die Fehler. Typische Probleme:
- Secret nicht gesetzt: `TAURI_SIGNING_PRIVATE_KEY not found` → Step 1 von 6B.6 nachholen
- Rust-Target nicht verfügbar: Workflow-File auf Tippfehler prüfen
- `pnpm install --frozen-lockfile` failt: `pnpm-lock.yaml` committen (in Step 2 oben sollte das aber schon passiert sein)

- [ ] **Step 5: Release-Draft prüfen**

Wenn der Workflow grün ist, öffne `https://github.com/kcamnaj/ProjektAlpha/releases`. Du siehst einen Draft-Release `v0.1.0` mit Assets:

- `ProjektAlpha_0.1.0_aarch64.dmg` (Mac Apple Silicon)
- `ProjektAlpha_0.1.0_x64.dmg` (Mac Intel)
- `ProjektAlpha_0.1.0_x64-setup.exe` (Windows) — hängt vom Bundle-Target ab
- `ProjektAlpha_0.1.0_x64_en-US.msi` (Windows)
- `latest.json` — das Update-Manifest
- `.sig`-Dateien für jedes Binary (Signatur-Dateien, die `latest.json` referenziert)

- [ ] **Step 6: `latest.json` prüfen**

Download `latest.json` vom Release und öffne es im Editor. Struktur sollte so aussehen:

```json
{
  "version": "v0.1.0",
  "notes": "Automatisch gebaut via GitHub Actions...",
  "pub_date": "2026-04-...",
  "platforms": {
    "darwin-aarch64": { "signature": "...", "url": "...aarch64.dmg" },
    "darwin-x86_64":  { "signature": "...", "url": "...x64.dmg" },
    "windows-x86_64": { "signature": "...", "url": "...x64-setup.exe" }
  }
}
```

Wenn `platforms` leer ist oder Signaturen fehlen → Signing-Secrets waren in GH nicht korrekt gesetzt. Workflow nochmal laufen lassen nach Secret-Fix.

- [ ] **Step 7: Release publizieren**

Im GitHub-UI: Draft → „Edit" → „Publish release".

Endpoint-Check: `https://github.com/kcamnaj/ProjektAlpha/releases/latest/download/latest.json` muss jetzt eine valide JSON-Antwort liefern (nicht 404).

- [ ] **Step 8: Mac-Installation testen**

1. DMG runterladen (entweder von GitHub UI oder direkt von `https://github.com/kcamnaj/ProjektAlpha/releases/latest`)
2. Doppelklick → Drag-to-Applications
3. `~/Applications/ProjektAlpha.app` → **Rechts-Klick** → Öffnen → „Öffnen" bestätigen
4. App sollte starten, Dashboard zeigen
5. Einstellungen → Über → „Nach Updates suchen" → zeigt „Du bist auf der aktuellen Version (0.1.0)"

- [ ] **Step 9: Windows-Installation testen**

Wenn Du keinen Windows-Rechner hast, überspringe und überlass es dem Vater:
1. MSI runterladen
2. Doppelklick → SmartScreen: „Mehr Info" → „Trotzdem ausführen"
3. Install-Dialog folgen
4. Startmenü → ProjektAlpha → App startet
5. Über-Tab → Update-Check wie in Step 8

- [ ] **Step 10: Auto-Update-End-to-End testen (später)**

Das ist der eigentliche Beweis, dass das ganze System funktioniert. Nachdem v0.1.0 installiert ist:

1. Lokal: `pnpm run release:bump 0.1.1`
2. Commit + Tag `v0.1.1` + Push
3. Warten bis GH Actions fertig + Release publiziert
4. In der installierten v0.1.0-App: Einstellungen → Über → „Nach Updates suchen"
5. Erwartung: Update-Available-UI erscheint, „Update jetzt installieren" klickt → Download-Progress → App restartet als v0.1.1

Diesen End-to-End-Test kannst Du beim ersten Release noch nicht machen (erst nach v0.1.1), also notiere Dir den Test als „nach erster Folge-Version nachholen".

- [ ] **Step 11: Checkpoint 6B.7 = Plan 6B fertig**

---

## Was am Ende dieses Plans funktioniert

- ✅ Repo auf GitHub (`kcamnaj/ProjektAlpha`), public, initialer Commit plus v0.1.0 Release-Commit + Tag
- ✅ Tauri Auto-Updater-Plugin installiert und Ed25519-signiert
- ✅ Public Key in `tauri.conf.json`, Private Key sicher außerhalb (`~/.tauri/projektalpha.key`) und als GitHub-Secret
- ✅ Über-Tab hat Update-Check-Button + Silent-Check beim App-Start
- ✅ GitHub Actions baut auf Tag-Push macOS (arm64 + x64) und Windows MSI, erstellt Release-Draft mit allen Assets + `latest.json`
- ✅ End-User-README auf Deutsch erklärt Download, Install und Gatekeeper/SmartScreen-Bypass
- ✅ RELEASE.md dokumentiert den Dev-Release-Workflow für zukünftige Versionen
- ✅ `pnpm run release:bump X.Y.Z` setzt alle drei Version-Referenzen synchron
- ✅ Frontend-Test-Suite bei 37 (unverändert — Update-UI ist Integration-level, smoke-getestet)
- ✅ Rust-Test-Suite bei 82 (unverändert)

## Was bewusst NICHT in diesem Plan ist

- **Apple Developer ID / Notarization** ($99/Jahr). User muss beim ersten Öffnen Rechts-Klick machen — dokumentiert im README. Wenn mehrere Personen die App nutzen oder IT-Compliance das fordert: Plan 7.
- **Windows EV-Cert** ($300+/Jahr). SmartScreen-Warnung bleibt. Alternative in Plan 7: Signtool mit Non-EV-Cert (billiger, aber reduziert SmartScreen-Warnung nicht sofort — User-Volume muss erst „Reputation" aufbauen).
- **Linux-Build** (AppImage/DEB). Spec nennt nur Mac+Windows.
- **Releases-Auto-Publish** statt Draft. Bewusst draft, damit Du vor Freigabe Asset-Liste prüfen kannst.
- **Ältere Macs (<10.15)**. Tauri 2 braucht macOS 10.15+ minimum. Für Monterey/Ventura/Sonoma getestet.
- **Release Notes generieren aus commits**. Aktuell `releaseBody` ist statisch. Später mit conventional-commit-tool-setup möglich.
- **E2E im CI**. Manuell in 6B.7 Step 8/9/10.
- **Rollback-Mechanismus** falls ein Release kaputt ist. User kann manuell ältere Version von der Releases-Seite nehmen. Automated Rollback wäre Plan-8-Thema.
- **Versionsprüfung beim App-Start gegenüber DB-Schema-Migrationen.** Wenn mal eine Migration irreversibel wird, kann ein Downgrade die DB beschädigen. Aktuell nicht der Fall (Migrations sind additiv), aber ein „DB-Version >= App-Version → Warning"-Check wäre sinnvoll sobald das relevant wird.

---

## Phase 6 ist damit komplett

Plans 6A (Polish + Tech-Debt) und 6B (Release-Infrastruktur) zusammen schließen Phase 6 aus Spec §14. Der Vater hat die App als Installer, bekommt Updates automatisch, und Tech-Debt ist abgebaut.

**Was dann noch übrig ist:**
- Nächste Ideen wären in Plan 7+ zusammengefasst (z.B. E2E Playwright, Code-Signing, OS-Native-Toasts, CSV-Export) — aber explizit Out-of-Scope des MVP, pure Nice-to-haves, werden erst geschrieben wenn der Vater das Tool nutzt und Anforderungen formuliert.
