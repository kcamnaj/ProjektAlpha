import type { RecentActivityRow } from "@/lib/tauri"
import { useUiStore } from "@/stores/uiStore"
import { Card, CardContent, CardTitle } from "@/components/ui/card"
import { StickyNote, Phone, Mail, Footprints, ArrowRightLeft } from "lucide-react"
import { formatDateDe } from "@/lib/format"

const ICON: Record<string, React.ReactNode> = {
  notiz: <StickyNote className="size-4" />,
  anruf: <Phone className="size-4" />,
  mail: <Mail className="size-4" />,
  besuch: <Footprints className="size-4" />,
  "status_änderung": <ArrowRightLeft className="size-4" />,
}

const TYPE_LABEL: Record<string, string> = {
  notiz: "Notiz",
  anruf: "Anruf",
  mail: "Mail",
  besuch: "Besuch",
  "status_änderung": "Status",
}

interface ActivityTimelineProps {
  rows: RecentActivityRow[]
}

export function ActivityTimeline({ rows }: ActivityTimelineProps) {
  const setView = useUiStore(s => s.setView)
  const selectCompany = useUiStore(s => s.selectCompany)

  const openCompany = (id: string) => {
    selectCompany(id)
    setView("companies")
  }

  // Gruppierung nach Tag (yyyy-MM-dd)
  const groups: Record<string, RecentActivityRow[]> = {}
  for (const r of rows) {
    const day = r.created_at.slice(0, 10)
    groups[day] ??= []
    groups[day].push(r)
  }
  const orderedDays = Object.keys(groups).sort().reverse()

  return (
    <Card size="sm">
      <CardContent className="flex flex-col gap-4">
        <CardTitle className="text-sm">Letzte Aktivität</CardTitle>
        {rows.length === 0 && (
          <div className="text-sm text-muted-foreground">Noch keine Aktivität erfasst.</div>
        )}
        {orderedDays.map((day) => (
          <div key={day} className="flex flex-col gap-1">
            <div className="text-xs font-medium text-muted-foreground">{formatDateDe(day)}</div>
            <ul className="flex flex-col">
              {groups[day].map((r) => (
                <li key={r.id}>
                  <button
                    onClick={() => openCompany(r.company_id)}
                    className="w-full py-1.5 flex items-start gap-2 hover:bg-accent/50 rounded-md px-2 text-left"
                  >
                    <div className="text-muted-foreground pt-0.5">{ICON[r.type] ?? <StickyNote className="size-4" />}</div>
                    <div className="flex-1 min-w-0">
                      <div className="text-sm">
                        <span className="font-medium">{r.company_name}</span>
                        <span className="text-muted-foreground"> · {TYPE_LABEL[r.type] ?? r.type}</span>
                      </div>
                      <div className="text-xs text-muted-foreground truncate">{r.content}</div>
                    </div>
                  </button>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </CardContent>
    </Card>
  )
}
