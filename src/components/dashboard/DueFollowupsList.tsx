import type { CompanyRow } from "@/lib/tauri"
import { useUiStore } from "@/stores/uiStore"
import { Card, CardContent } from "@/components/ui/card"
import { AlertTriangle, Phone, ArrowRight } from "lucide-react"
import { formatRelativeFollowup, statusLabel, statusColor } from "@/lib/format"
import { cn } from "@/lib/utils"

interface DueFollowupsListProps {
  rows: CompanyRow[]
}

export function DueFollowupsList({ rows }: DueFollowupsListProps) {
  const setView = useUiStore(s => s.setView)
  const selectCompany = useUiStore(s => s.selectCompany)

  const openCompany = (id: string) => {
    selectCompany(id)
    setView("companies")
  }

  if (rows.length === 0) {
    return (
      <Card size="sm">
        <CardContent>
          <div className="text-sm text-muted-foreground">Keine Wiedervorlagen heute fällig. 🎉</div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card className="ring-2 ring-destructive/60" size="sm">
      <CardContent className="flex flex-col gap-3">
        <div className="flex items-center gap-2 text-destructive">
          <AlertTriangle className="size-5" />
          <span className="font-semibold">
            {rows.length} {rows.length === 1 ? "Wiedervorlage" : "Wiedervorlagen"} fällig
          </span>
        </div>
        <ul className="flex flex-col divide-y">
          {rows.map((c) => (
            <li key={c.id}>
              <button
                onClick={() => openCompany(c.id)}
                className="w-full py-2 flex items-center gap-3 hover:bg-accent/50 rounded-md px-2 text-left"
              >
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="font-medium truncate">{c.name}</span>
                    <span className={cn("text-[10px] px-1.5 py-0.5 rounded", statusColor(c.status))}>
                      {statusLabel(c.status)}
                    </span>
                  </div>
                  <div className="text-xs text-muted-foreground flex items-center gap-2">
                    <span>{formatRelativeFollowup(c.next_followup_at)}</span>
                    {c.city && <span>· {c.city}</span>}
                    {c.phone && <span className="inline-flex items-center gap-1"><Phone className="size-3" />{c.phone}</span>}
                  </div>
                </div>
                <ArrowRight className="size-4 text-muted-foreground shrink-0" />
              </button>
            </li>
          ))}
        </ul>
      </CardContent>
    </Card>
  )
}
