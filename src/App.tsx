import { AppLayout } from "@/components/layout/AppLayout"
import { useUiStore } from "@/stores/uiStore"
import { DashboardPage } from "@/pages/DashboardPage"
import { CompaniesPage } from "@/pages/CompaniesPage"
import { NewSearchPage } from "@/pages/NewSearchPage"
import { MapPage } from "@/pages/MapPage"
import { SettingsPage } from "@/pages/SettingsPage"

function App() {
  const view = useUiStore(s => s.currentView)
  return (
    <AppLayout>
      {view === "dashboard" && <DashboardPage />}
      {view === "companies" && <CompaniesPage />}
      {view === "search" && <NewSearchPage />}
      {view === "map" && <MapPage />}
      {view === "settings" && <SettingsPage />}
    </AppLayout>
  )
}
export default App
