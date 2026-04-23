import { useEffect, useState } from "react"
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Button } from "@/components/ui/button"
import { Textarea } from "@/components/ui/textarea"
import { Slider } from "@/components/ui/slider"
import { api, type CategoryFull } from "@/lib/tauri"
import { logger } from "@/lib/logger"

interface BranchenEditDialogProps {
  open: boolean
  onOpenChange: (o: boolean) => void
  /** null = neu anlegen, sonst bearbeiten */
  editing: CategoryFull | null
  onSaved: () => void
}

const DEFAULT_TAGS = `[{"shop":"wholesale"}]`

export function BranchenEditDialog({ open, onOpenChange, editing, onSaved }: BranchenEditDialogProps) {
  const [name, setName] = useState("")
  const [osmTags, setOsmTags] = useState(DEFAULT_TAGS)
  const [tagsError, setTagsError] = useState<string | null>(null)
  const [weight, setWeight] = useState(50)
  const [color, setColor] = useState("#3b82f6")
  const [busy, setBusy] = useState(false)
  const [err, setErr] = useState<string | null>(null)

  useEffect(() => {
    if (open) {
      setName(editing?.name_de ?? "")
      setOsmTags(editing?.osm_tags ?? DEFAULT_TAGS)
      setWeight(editing?.probability_weight ?? 50)
      setColor(editing?.color ?? "#3b82f6")
      setTagsError(null)
      setErr(null)
    }
  }, [open, editing])

  const validateTags = (v: string): string | null => {
    try {
      const parsed = JSON.parse(v)
      if (!Array.isArray(parsed)) return "Muss ein Array sein"
      if (parsed.some(x => typeof x !== "object" || Array.isArray(x) || x === null)) {
        return "Jedes Element muss ein Objekt sein"
      }
      return null
    } catch (e) {
      return String(e)
    }
  }

  const save = async () => {
    const tErr = validateTags(osmTags)
    if (tErr) { setTagsError(tErr); return }
    if (!name.trim()) { setErr("Name darf nicht leer sein"); return }
    setBusy(true); setErr(null)
    try {
      if (editing) {
        await api.updateCategory({ id: editing.id, name_de: name.trim(), osm_tags: osmTags, probability_weight: weight, color })
      } else {
        await api.createCategory({ name_de: name.trim(), osm_tags: osmTags, probability_weight: weight, color })
      }
      onSaved()
      onOpenChange(false)
    } catch (e) {
      setErr(String(e))
      logger.error("save category failed", { e: String(e) })
    } finally {
      setBusy(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>{editing ? "Branche bearbeiten" : "Neue Branche"}</DialogTitle>
        </DialogHeader>
        <div className="space-y-4">
          <div className="space-y-1">
            <Label htmlFor="bn">Name</Label>
            <Input id="bn" value={name} onChange={(e) => setName(e.target.value)} />
          </div>
          <div className="space-y-1">
            <Label htmlFor="bc">Pin-Farbe</Label>
            <div className="flex items-center gap-2">
              <input id="bc" type="color" value={color} onChange={(e) => setColor(e.target.value)} className="h-9 w-12 rounded border cursor-pointer" />
              <Input value={color} onChange={(e) => setColor(e.target.value)} className="flex-1" />
            </div>
          </div>
          <div className="space-y-2">
            <div className="flex justify-between">
              <Label>Gewichtung</Label>
              <span className="text-sm font-medium tabular-nums">{weight}%</span>
            </div>
            <Slider value={[weight]} min={0} max={100} step={1} onValueChange={(v) => setWeight(v[0])} />
          </div>
          <div className="space-y-1">
            <Label htmlFor="bt">OSM-Tags (JSON-Array)</Label>
            <Textarea
              id="bt"
              value={osmTags}
              onChange={(e) => { setOsmTags(e.target.value); setTagsError(null) }}
              onBlur={() => setTagsError(validateTags(osmTags))}
              rows={4}
              className="font-mono text-xs"
            />
            <p className="text-xs text-muted-foreground">
              Beispiel: <code>{'[{"shop":"wholesale"}, {"industrial":"warehouse"}]'}</code> — Liste = OR, Objekt = AND.
            </p>
            {tagsError && <p className="text-xs text-red-600">{tagsError}</p>}
          </div>
          {err && <p className="text-sm text-red-600">{err}</p>}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={busy}>Abbrechen</Button>
          <Button onClick={save} disabled={busy || !!tagsError}>
            {busy ? "Speichere…" : editing ? "Speichern" : "Anlegen"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
