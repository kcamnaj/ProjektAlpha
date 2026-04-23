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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const doCheck = async (silent = false) => {
    setBusy("check")
    const result = await checkForUpdate()
    setUpdater(result)
    setBusy(null)
    if (silent && result.kind === "up-to-date") {
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
