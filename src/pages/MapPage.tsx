import { useUiStore } from "@/stores/uiStore"
import { useFilteredCompanies } from "@/hooks/useFilteredCompanies"
import { MapView } from "@/components/map/MapView"
import { CompanyDetailSheet } from "@/components/detail/CompanyDetailSheet"
import { EmptyState } from "@/components/common/EmptyState"

export function MapPage() {
  const { companies, refresh } = useFilteredCompanies()
  const selectedId = useUiStore(s => s.selectedCompanyId)
  const selectCompany = useUiStore(s => s.selectCompany)

  return (
    <div className="h-full w-full relative">
      <MapView
        companies={companies}
        selectedId={selectedId}
        onSelect={selectCompany}
      />
      {companies.length === 0 ? (
        <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
          <div className="pointer-events-auto">
            <EmptyState
              title="Noch keine Firmen auf der Karte"
              hint="Ergebnisse erscheinen hier, sobald eine Suche läuft."
              actionLabel="Neue Suche"
              onAction={() => useUiStore.getState().setView("search")}
            />
          </div>
        </div>
      ) : (
        <div className="absolute top-2 left-2 bg-background/90 backdrop-blur rounded-md px-3 py-1.5 text-xs shadow border">
          {companies.length} Firmen · Klick auf Pin öffnet Details
        </div>
      )}
      <CompanyDetailSheet
        companyId={selectedId}
        open={!!selectedId}
        onOpenChange={(o) => { if (!o) selectCompany(null) }}
        onChanged={refresh}
      />
    </div>
  )
}
