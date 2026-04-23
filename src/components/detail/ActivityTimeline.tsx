import type { ActivityRow } from "@/lib/tauri"
import { formatDateDe } from "@/lib/format"
import { Phone, Mail, NotebookPen, Footprints, Activity } from "lucide-react"

const ICONS: Record<string, typeof Phone> = {
  notiz: NotebookPen,
  anruf: Phone,
  mail: Mail,
  besuch: Footprints,
  status_änderung: Activity,
}
const LABELS: Record<string, string> = {
  notiz: "Notiz",
  anruf: "Anruf",
  mail: "Mail",
  besuch: "Besuch",
  status_änderung: "Status",
}

export function ActivityTimeline({ entries }: { entries: ActivityRow[] }) {
  if (entries.length === 0) {
    return <div className="text-sm text-muted-foreground">Noch keine Einträge.</div>
  }
  return (
    <ol className="space-y-3">
      {entries.map(e => {
        const Icon = ICONS[e.type] ?? NotebookPen
        return (
          <li key={e.id} className="flex gap-3">
            <div className="mt-0.5">
              <Icon className="size-4 text-muted-foreground" />
            </div>
            <div className="flex-1 min-w-0">
              <div className="text-xs text-muted-foreground">
                {LABELS[e.type] ?? e.type} · {formatDateDe(e.created_at)}
              </div>
              <div className="text-sm whitespace-pre-wrap">{e.content}</div>
            </div>
          </li>
        )
      })}
    </ol>
  )
}
