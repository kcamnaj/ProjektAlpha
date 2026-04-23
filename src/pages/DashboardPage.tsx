import { KpiCards } from "@/components/dashboard/KpiCards"
import { DueFollowupsList } from "@/components/dashboard/DueFollowupsList"
import { ActivityTimeline } from "@/components/dashboard/ActivityTimeline"
import { useDashboardData } from "@/hooks/useDashboardData"
import { Button } from "@/components/ui/button"
import { RefreshCw } from "lucide-react"

export function DashboardPage() {
  const { kpis, followups, activity, loading, error, refresh } = useDashboardData()

  return (
    <div className="h-full overflow-y-auto">
      <div className="max-w-5xl mx-auto p-6 flex flex-col gap-5">
        <div className="flex items-center justify-between">
          <h2 className="text-xl font-semibold">Dashboard</h2>
          <Button variant="ghost" size="sm" onClick={refresh} disabled={loading}>
            <RefreshCw className={loading ? "size-4 animate-spin" : "size-4"} />
            Aktualisieren
          </Button>
        </div>

        {error && (
          <div className="text-sm text-destructive bg-destructive/10 rounded-md p-3">
            Fehler beim Laden: {error}
          </div>
        )}

        {/* Banner-Slot: Heute fällig wird zuerst gerendert, damit fällige Fälle unübersehbar sind */}
        {kpis !== null && <DueFollowupsList rows={followups} />}

        {kpis !== null && <KpiCards kpis={kpis} />}

        {kpis !== null && <ActivityTimeline rows={activity} />}

        {loading && kpis === null && (
          <div className="text-sm text-muted-foreground">Lade…</div>
        )}
      </div>
    </div>
  )
}
