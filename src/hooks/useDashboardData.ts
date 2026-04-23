import { useCallback, useEffect, useState } from "react"
import { api, type CompanyRow, type DashboardKpis, type RecentActivityRow } from "@/lib/tauri"
import { logger } from "@/lib/logger"

export interface DashboardData {
  kpis: DashboardKpis | null
  followups: CompanyRow[]
  activity: RecentActivityRow[]
  loading: boolean
  error: string | null
  refresh: () => Promise<void>
}

export function useDashboardData(): DashboardData {
  const [kpis, setKpis] = useState<DashboardKpis | null>(null)
  const [followups, setFollowups] = useState<CompanyRow[]>([])
  const [activity, setActivity] = useState<RecentActivityRow[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const refresh = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const [k, f, a] = await Promise.all([
        api.dashboardKpis(),
        api.listDueFollowups(),
        api.listRecentActivity(20),
      ])
      setKpis(k)
      setFollowups(f)
      setActivity(a)
      logger.info("dashboard loaded", {
        customers: k.customers, requested: k.requested, new_count: k.new_count,
        followup_count: f.length, activity_count: a.length,
      })
    } catch (e) {
      const msg = String(e)
      setError(msg)
      logger.error("dashboard load failed", { e: msg })
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => { refresh() }, [refresh])

  return { kpis, followups, activity, loading, error, refresh }
}
