import { Checkbox } from "@/components/ui/checkbox"
import { Label } from "@/components/ui/label"
import { Button } from "@/components/ui/button"
import type { CategoryRow } from "@/lib/tauri"

interface CategoryPickerProps {
  categories: CategoryRow[]
  selected: Set<number>
  onChange: (next: Set<number>) => void
}

export function CategoryPicker({ categories, selected, onChange }: CategoryPickerProps) {
  const toggle = (id: number, checked: boolean) => {
    const next = new Set(selected)
    if (checked) next.add(id)
    else next.delete(id)
    onChange(next)
  }
  const setAll = (val: boolean) => {
    onChange(val ? new Set(categories.map(c => c.id)) : new Set())
  }

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <Label>Branchen ({selected.size}/{categories.length})</Label>
        <div className="flex gap-2">
          <Button type="button" variant="ghost" size="sm" onClick={() => setAll(true)}>Alle</Button>
          <Button type="button" variant="ghost" size="sm" onClick={() => setAll(false)}>Keine</Button>
        </div>
      </div>
      <div className="border rounded-md max-h-64 overflow-y-auto divide-y">
        {categories.map(c => {
          const id = `cat-${c.id}`
          const isOn = selected.has(c.id)
          return (
            <label key={c.id} htmlFor={id} className="flex items-center gap-3 px-3 py-2 cursor-pointer hover:bg-accent/40">
              <Checkbox
                id={id}
                checked={isOn}
                onCheckedChange={(v) => toggle(c.id, v === true)}
              />
              <span className="inline-block size-3 rounded-sm shrink-0" style={{ background: c.color }} />
              <span className="flex-1 text-sm truncate">{c.name_de}</span>
              <span className="text-xs text-muted-foreground tabular-nums">{c.probability_weight}%</span>
            </label>
          )
        })}
      </div>
    </div>
  )
}
