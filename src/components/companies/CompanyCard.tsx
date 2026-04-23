import type { CompanyRow } from "@/lib/tauri"
import { StatusBadge } from "./StatusBadge"
import { ScoreBadge } from "./ScoreBadge"
import { formatRelativeDe } from "@/lib/format"
import { cn } from "@/lib/utils"

export function CompanyCard({
  company, selected, onClick,
}: { company: CompanyRow; selected: boolean; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "w-full text-left p-3 border-b hover:bg-accent/50 transition-colors",
        selected && "bg-accent"
      )}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="min-w-0 flex-1">
          <div className="font-medium truncate">{company.name}</div>
          <div className="text-xs text-muted-foreground truncate">
            {company.city ?? "—"} · {company.category_name ?? "Sonstige"}
          </div>
        </div>
        <ScoreBadge score={company.probability_score} />
      </div>
      <div className="mt-2 flex items-center justify-between gap-2">
        <StatusBadge status={company.status} />
        <span className="text-xs text-muted-foreground">{formatRelativeDe(company.last_contact_at)}</span>
      </div>
    </button>
  )
}
