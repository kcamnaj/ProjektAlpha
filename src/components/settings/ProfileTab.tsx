import { useEffect, useState } from "react"
import { api, type SearchProfile } from "@/lib/tauri"
import { Button } from "@/components/ui/button"
import { useUiStore } from "@/stores/uiStore"
import { formatDateDe } from "@/lib/format"
import { Pencil, Trash2, Play } from "lucide-react"
import { logger } from "@/lib/logger"
import { ConfirmDialog } from "@/components/common/ConfirmDialog"
import { LoadingState } from "@/components/common/LoadingState"

export function ProfileTab() {
  const [profiles, setProfiles] = useState<SearchProfile[]>([])
  const [loading, setLoading] = useState(true)
  const [toDelete, setToDelete] = useState<{ id: number; name: string } | null>(null)
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

  const askDelete = (p: SearchProfile) => setToDelete({ id: p.id, name: p.name })

  const doDelete = async () => {
    if (!toDelete) return
    const id = toDelete.id
    setToDelete(null)
    try {
      await api.deleteSearchProfile(id)
      reload()
    } catch (e) { alert(`Löschen fehlgeschlagen: ${e}`) }
  }

  const load = (p: SearchProfile) => {
    sessionStorage.setItem("loadProfile", JSON.stringify(p))
    setView("search")
  }

  if (loading) return <LoadingState message="Lade Profile…" />

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
            <Button variant="ghost" size="icon" onClick={() => askDelete(p)} aria-label="Löschen"><Trash2 className="size-4" /></Button>
          </div>
        ))}
      </div>
      <ConfirmDialog
        open={toDelete !== null}
        onOpenChange={(o) => { if (!o) setToDelete(null) }}
        title="Such-Profil löschen?"
        description={toDelete ? `Das Profil "${toDelete.name}" wird dauerhaft entfernt. Bereits gespeicherte Firmen bleiben unverändert.` : ""}
        confirmLabel="Löschen"
        destructive
        onConfirm={doDelete}
      />
    </div>
  )
}
