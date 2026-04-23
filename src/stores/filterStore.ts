import { create } from "zustand"

export interface CompanyFilter {
  status: string | null
  categoryIds: number[] | null
  minScore: number
  search: string
}

interface FilterState extends CompanyFilter {
  setStatus: (s: string | null) => void
  setCategoryIds: (ids: number[] | null) => void
  setMinScore: (n: number) => void
  setSearch: (q: string) => void
  reset: () => void
}

const initial: CompanyFilter = { status: null, categoryIds: null, minScore: 0, search: "" }

export const useFilterStore = create<FilterState>((set) => ({
  ...initial,
  setStatus: (s) => set({ status: s }),
  setCategoryIds: (ids) => set({ categoryIds: ids }),
  setMinScore: (n) => set({ minScore: n }),
  setSearch: (q) => set({ search: q }),
  reset: () => set({ ...initial }),
}))
