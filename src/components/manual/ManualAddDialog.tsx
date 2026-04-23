import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Button } from "@/components/ui/button"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { useEffect, useState } from "react"
import { api, type CategoryRow, type GeocodeSuggestion } from "@/lib/tauri"
import { AddressSearchInput } from "@/components/search/AddressSearchInput"
import { Plus } from "lucide-react"
import { logger } from "@/lib/logger"

export function ManualAddDialog({ onAdded }: { onAdded: () => void }) {
  const [open, setOpen] = useState(false)
  const [cats, setCats] = useState<CategoryRow[]>([])
  const [busy, setBusy] = useState(false)
  const [form, setForm] = useState({
    name: "", street: "", postal_code: "", city: "",
    phone: "", email: "", website: "",
    industry_category_id: "1", lat: "", lng: "",
  })
  const [addressHint, setAddressHint] = useState<string | null>(null)

  useEffect(() => {
    if (open) {
      api.listCategories()
        .then(setCats)
        .catch(() => logger.error("listCategories failed in ManualAddDialog"))
    }
  }, [open])

  const submit = async () => {
    if (!form.name.trim()) return
    if (!form.lat || !form.lng) {
      alert("Bitte eine Adresse suchen (oder Koordinaten per Hand eintragen).")
      return
    }
    setBusy(true)
    try {
      const cat = cats.find(c => c.id === Number(form.industry_category_id))
      await api.addManualCompany({
        osm_id: null,
        name: form.name.trim(),
        street: form.street || null,
        postal_code: form.postal_code || null,
        city: form.city || null,
        country: "DE",
        lat: Number(form.lat), lng: Number(form.lng),
        phone: form.phone || null, email: form.email || null, website: form.website || null,
        industry_category_id: Number(form.industry_category_id),
        size_estimate: null,
        probability_score: cat?.probability_weight ?? 50,
        source: "manual",
      })
      onAdded()
      setOpen(false)
      setForm({
        name: "", street: "", postal_code: "", city: "",
        phone: "", email: "", website: "",
        industry_category_id: "1", lat: "", lng: "",
      })
    } catch (e) {
      logger.error("addManualCompany failed", { name: form.name })
      alert("Konnte Firma nicht anlegen. Siehe Log.")
    } finally { setBusy(false) }
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button size="sm" variant="outline"><Plus className="size-3 mr-1" />Manuell</Button>
      </DialogTrigger>
      <DialogContent className="max-w-md max-h-[90vh] overflow-y-auto">
        <DialogHeader><DialogTitle>Firma manuell hinzufügen</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div className="space-y-1">
            <Label>Adresse suchen</Label>
            <AddressSearchInput
              onPick={(s: GeocodeSuggestion) => {
                setForm(f => ({ ...f, lat: String(s.lat), lng: String(s.lng) }))
                setAddressHint(s.display_name)
              }}
              placeholder="z.B. Bahnhofstr. 1, Hannover"
            />
            {addressHint && (
              <p className="text-xs text-muted-foreground truncate">Gefunden: {addressHint}</p>
            )}
          </div>
          {([
            ["name", "Firmenname *"],
            ["street", "Straße + Nr."],
            ["postal_code", "PLZ"],
            ["city", "Stadt"],
            ["lat", "Breitengrad (z.B. 52.3756)"],
            ["lng", "Längengrad (z.B. 9.7320)"],
            ["phone", "Telefon"],
            ["email", "E-Mail"],
            ["website", "Website (https://…)"],
          ] as const).map(([k, label]) => (
            <div key={k}>
              <Label htmlFor={k}>{label}</Label>
              <Input id={k} value={(form as Record<string,string>)[k]}
                onChange={(e) => setForm({ ...form, [k]: e.target.value })} />
            </div>
          ))}
          <div>
            <Label>Branche</Label>
            <Select value={form.industry_category_id} onValueChange={(v) => setForm({ ...form, industry_category_id: v })}>
              <SelectTrigger><SelectValue /></SelectTrigger>
              <SelectContent>
                {cats.map(c => <SelectItem key={c.id} value={String(c.id)}>{c.name_de}</SelectItem>)}
              </SelectContent>
            </Select>
          </div>
        </div>
        <Button onClick={submit} disabled={busy || !form.name.trim()}>Anlegen</Button>
      </DialogContent>
    </Dialog>
  )
}
