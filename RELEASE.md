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
`TAURI_SIGNING_PRIVATE_KEY` gespeichert (siehe Task 6B.6 im Plan).

## Keine Code-Signing-Zertifikate aktuell

macOS: Keine Apple Developer ID. Benutzer müssen beim Erstöffnen
Rechts-Klick → Öffnen benutzen (siehe README).

Windows: Kein EV-Cert. SmartScreen-Warnung beim Install — „Mehr Info"
→ „Trotzdem ausführen".

Falls später Code-Signing gewünscht ist: das wäre Plan 7 (Apple
Developer Account + Notarization für macOS; EV-Cert + sign-step
in release.yml für Windows).
