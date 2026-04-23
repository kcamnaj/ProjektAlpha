import { useFilterStore } from "@/stores/filterStore"
import { Input } from "@/components/ui/input"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Button } from "@/components/ui/button"
import { Search, X } from "lucide-react"
import { ManualAddDialog } from "@/components/manual/ManualAddDialog"

export function FilterBar({ onAdded }: { onAdded: () => void }) {
  const { status, search, minScore, setStatus, setSearch, setMinScore, reset } = useFilterStore()

  return (
    <div className="border-b p-3 flex flex-wrap items-center gap-2">
      <div className="relative flex-1 min-w-48">
        <Search className="absolute left-2.5 top-2.5 size-4 text-muted-foreground" />
        <Input
          placeholder="Firma oder Stadt suchen…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="pl-9"
        />
      </div>
      <Select value={status ?? "all"} onValueChange={(v) => setStatus(v === "all" ? null : v)}>
        <SelectTrigger className="w-40"><SelectValue /></SelectTrigger>
        <SelectContent>
          <SelectItem value="all">Alle Status</SelectItem>
          <SelectItem value="neu">Neu</SelectItem>
          <SelectItem value="angefragt">Angefragt</SelectItem>
          <SelectItem value="kunde">Kunde</SelectItem>
          <SelectItem value="kein_kunde">Kein Kunde</SelectItem>
        </SelectContent>
      </Select>
      <div className="flex items-center gap-2 text-sm">
        <label htmlFor="min-score">Min. Score</label>
        <input
          id="min-score"
          type="range" min={0} max={100} step={5}
          value={minScore} onChange={(e) => setMinScore(Number(e.target.value))}
          className="w-24"
        />
        <span className="tabular-nums w-8">{minScore}</span>
      </div>
      <Button variant="ghost" size="sm" onClick={reset}>
        <X className="size-3 mr-1" /> Reset
      </Button>
      <ManualAddDialog onAdded={onAdded} />
    </div>
  )
}
