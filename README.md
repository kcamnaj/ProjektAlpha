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
