import { useEffect, useState } from "react"
import { api, type CompanyRow } from "@/lib/tauri"
import { useFilterStore } from "@/stores/filterStore"
import { logger } from "@/lib/logger"

export interface UseFilteredCompaniesResult {
  companies: CompanyRow[]
  loading: boolean
  refresh: () => void
}

export function useFilteredCompanies(): UseFilteredCompaniesResult {
  const [companies, setCompanies] = useState<CompanyRow[]>([])
  const [loading, setLoading] = useState(true)
  const [tick, setTick] = useState(0)
  const status = useFilterStore(s => s.status)
  const categoryIds = useFilterStore(s => s.categoryIds)
  const minScore = useFilterStore(s => s.minScore)
  const search = useFilterStore(s => s.search)

  useEffect(() => {
    let canceled = false
    setLoading(true)
    api.listCompanies({ status, category_ids: categoryIds, min_score: minScore, search })
      .then(rows => { if (!canceled) setCompanies(rows) })
      .catch(e => logger.error("listCompanies failed", { e: String(e) }))
      .finally(() => { if (!canceled) setLoading(false) })
    return () => { canceled = true }
  }, [status, categoryIds, minScore, search, tick])

  useEffect(() => {
    const un = api.onSearchDone(() => setTick(t => t + 1))
    return () => { un.then(f => f()) }
  }, [])

  return { companies, loading, refresh: () => setTick(t => t + 1) }
}
