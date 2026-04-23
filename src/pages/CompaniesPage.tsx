import { useUiStore } from "@/stores/uiStore"
import { useFilteredCompanies } from "@/hooks/useFilteredCompanies"
import { CompanyList } from "@/components/companies/CompanyList"
import { FilterBar } from "@/components/companies/FilterBar"
import { CompanyDetailSheet } from "@/components/detail/CompanyDetailSheet"
import { MapView } from "@/components/map/MapView"
import { LoadingState } from "@/components/common/LoadingState"
import { EmptyState } from "@/components/common/EmptyState"

export function CompaniesPage() {
  const { companies, loading, refresh } = useFilteredCompanies()
  const selectedCompanyId = useUiStore(s => s.selectedCompanyId)
  const selectCompany = useUiStore(s => s.selectCompany)

  return (
    <div className="h-full flex">
      <div className="w-96 flex flex-col border-r">
        <FilterBar onAdded={refresh} />
        {loading
          ? <LoadingState message="Lade Firmen…" />
          : companies.length === 0
            ? <EmptyState
                title="Noch keine Firmen"
                hint="Starte eine neue Suche, um OSM-Leads zu laden."
                actionLabel="Neue Suche"
                onAction={() => useUiStore.getState().setView("search")}
              />
            : <CompanyList companies={companies} selectedId={selectedCompanyId} onSelect={selectCompany} />}
      </div>
      <div className="flex-1 relative">
        <MapView
          companies={companies}
          selectedId={selectedCompanyId}
          onSelect={selectCompany}
        />
      </div>
      <CompanyDetailSheet
        companyId={selectedCompanyId}
        open={!!selectedCompanyId}
        onOpenChange={(o) => { if (!o) selectCompany(null) }}
        onChanged={refresh}
      />
    </div>
  )
}
