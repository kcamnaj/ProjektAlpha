import { Card, CardContent, CardDescription, CardTitle } from "@/components/ui/card"
import { Users, PhoneCall, Sparkles, Gauge } from "lucide-react"
import type { DashboardKpis } from "@/lib/tauri"

interface KpiCardsProps {
  kpis: DashboardKpis
}

export function KpiCards({ kpis }: KpiCardsProps) {
  const avgDisplay =
    kpis.total_active === 0
      ? "—"
      : kpis.avg_score.toFixed(1).replace(".", ",")

  return (
    <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
      <KpiTile icon={<Users className="size-5" />} label="Kunden" value={String(kpis.customers)} />
      <KpiTile icon={<PhoneCall className="size-5" />} label="Angefragt" value={String(kpis.requested)} />
      <KpiTile icon={<Sparkles className="size-5" />} label="Neu" value={String(kpis.new_count)} />
      <KpiTile icon={<Gauge className="size-5" />} label="Ø Score" value={avgDisplay} hint={kpis.total_active > 0 ? `über ${kpis.total_active} Firmen` : "keine Daten"} />
    </div>
  )
}

interface KpiTileProps {
  icon: React.ReactNode
  label: string
  value: string
  hint?: string
}

function KpiTile({ icon, label, value, hint }: KpiTileProps) {
  return (
    <Card size="sm">
      <CardContent className="flex flex-col gap-1">
        <div className="flex items-center gap-2 text-muted-foreground">
          {icon}
          <CardTitle className="text-sm">{label}</CardTitle>
        </div>
        <div className="text-3xl font-semibold tracking-tight">{value}</div>
        {hint && <CardDescription className="text-xs">{hint}</CardDescription>}
      </CardContent>
    </Card>
  )
}
