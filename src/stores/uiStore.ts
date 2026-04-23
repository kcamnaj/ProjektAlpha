import { create } from "zustand"

export type View = "dashboard" | "companies" | "search" | "map" | "settings"

interface UiState {
  currentView: View
  selectedCompanyId: string | null
  setView: (v: View) => void
  selectCompany: (id: string | null) => void
}

export const useUiStore = create<UiState>((set) => ({
  currentView: "dashboard",
  selectedCompanyId: null,
  setView: (v) => set({ currentView: v }),
  selectCompany: (id) => set({ selectedCompanyId: id }),
}))
