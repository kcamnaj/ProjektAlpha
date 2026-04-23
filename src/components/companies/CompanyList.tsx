import type { CompanyRow } from "@/lib/tauri"
import { ScrollArea } from "@/components/ui/scroll-area"
import { CompanyCard } from "./CompanyCard"
import { EmptyState } from "./EmptyState"

export function CompanyList({
  companies, selectedId, onSelect,
}: { companies: CompanyRow[]; selectedId: string | null; onSelect: (id: string) => void }) {
  if (companies.length === 0) {
    return <EmptyState message="Keine Firmen passen zum Filter. Suche starten oder Filter anpassen." />
  }
  return (
    <ScrollArea className="h-full">
      <div className="border-r">
        {companies.map(c => (
          <CompanyCard key={c.id} company={c} selected={selectedId === c.id} onClick={() => onSelect(c.id)} />
        ))}
      </div>
    </ScrollArea>
  )
}
