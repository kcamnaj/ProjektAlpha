import { useState } from "react"
import { api } from "@/lib/tauri"
import { Button } from "@/components/ui/button"
import { Download, Upload, FolderOpen } from "lucide-react"
import { logger } from "@/lib/logger"
import { ConfirmDialog } from "@/components/common/ConfirmDialog"

export function DatenTab() {
  const [busy, setBusy] = useState<"backup" | "restore" | null>(null)
  const [msg, setMsg] = useState<string | null>(null)
  const [restoreConfirmOpen, setRestoreConfirmOpen] = useState(false)

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

  const doRestore = () => {
    setRestoreConfirmOpen(true)
  }

  const doRestoreConfirmed = async () => {
    setRestoreConfirmOpen(false)
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

      <ConfirmDialog
        open={restoreConfirmOpen}
        onOpenChange={setRestoreConfirmOpen}
        title="Datenbank wiederherstellen?"
        description="Restore ersetzt die aktuelle Datenbank vollständig mit dem Inhalt der ausgewählten Datei. Ein Pre-Restore-Snapshot wird automatisch angelegt. Die App startet nach dem Restore neu."
        confirmLabel="Wiederherstellen"
        destructive
        onConfirm={doRestoreConfirmed}
      />
    </div>
  )
}
