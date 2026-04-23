import { useEffect, useState } from "react"
import { api, type CategoryFull } from "@/lib/tauri"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { BranchenEditDialog } from "./BranchenEditDialog"
import { Pencil, Trash2, Plus } from "lucide-react"
import { logger } from "@/lib/logger"
import { ConfirmDialog } from "@/components/common/ConfirmDialog"

export function BranchenTab() {
  const [cats, setCats] = useState<CategoryFull[]>([])
  const [loading, setLoading] = useState(true)
  const [dlgOpen, setDlgOpen] = useState(false)
  const [editing, setEditing] = useState<CategoryFull | null>(null)
  const [toDelete, setToDelete] = useState<{ id: number; name: string } | null>(null)

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

  const askDelete = (c: CategoryFull) => setToDelete({ id: c.id, name: c.name_de })

  const doDelete = async () => {
    if (!toDelete) return
    const id = toDelete.id
    setToDelete(null)
    try {
      await api.deleteCategory(id)
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
            <Button variant="ghost" size="icon" onClick={() => askDelete(c)} aria-label="Löschen"><Trash2 className="size-4" /></Button>
          </div>
        ))}
      </div>
      <BranchenEditDialog
        open={dlgOpen}
        onOpenChange={setDlgOpen}
        editing={editing}
        onSaved={reload}
      />
      <ConfirmDialog
        open={toDelete !== null}
        onOpenChange={(o) => { if (!o) setToDelete(null) }}
        title="Branche löschen?"
        description={toDelete ? `Die Branche "${toDelete.name}" wird entfernt. Firmen, die dieser Branche zugeordnet waren, bleiben bestehen (Branche wird auf „keine" gesetzt).` : ""}
        confirmLabel="Löschen"
        destructive
        onConfirm={doDelete}
      />
    </div>
  )
}
