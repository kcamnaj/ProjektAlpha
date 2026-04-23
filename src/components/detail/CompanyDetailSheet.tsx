import { Sheet, SheetContent, SheetHeader, SheetTitle, SheetDescription } from "@/components/ui/sheet"
import { Button } from "@/components/ui/button"
import { Phone, Mail, Globe, Trash2 } from "lucide-react"
import { useEffect, useState } from "react"
import { api, type CompanyRow, type ActivityRow } from "@/lib/tauri"
import { ScoreBadge } from "@/components/companies/ScoreBadge"
import { StatusEditor } from "./StatusEditor"
import { FollowupEditor } from "./FollowupEditor"
import { ContactPersonEditor } from "./ContactPersonEditor"
import { ActivityTimeline } from "./ActivityTimeline"
import { AddActivityForm } from "./AddActivityForm"
import { logger } from "@/lib/logger"
import { ConfirmDialog } from "@/components/common/ConfirmDialog"

export function CompanyDetailSheet({
  companyId, open, onOpenChange, onChanged,
}: {
  companyId: string | null
  open: boolean
  onOpenChange: (open: boolean) => void
  onChanged: () => void
}) {
  const [company, setCompany] = useState<CompanyRow | null>(null)
  const [activity, setActivity] = useState<ActivityRow[]>([])
  const [busy, setBusy] = useState(false)
  const [deleteConfirmOpen, setDeleteConfirmOpen] = useState(false)

  const loadActivity = async (id: string) => {
    try {
      setActivity(await api.listActivity(id))
    } catch {
      logger.error("listActivity failed", { id })
    }
  }

  useEffect(() => {
    if (!companyId) { setCompany(null); setActivity([]); return }
    api.getCompany(companyId)
      .then(setCompany)
      .catch(() => logger.error("getCompany failed", { id: companyId }))
    loadActivity(companyId)
  }, [companyId])

  const updateStatus = async (next: string) => {
    if (!company) return
    setBusy(true)
    try {
      await api.updateCompanyStatus(company.id, next)
      const refreshed = await api.getCompany(company.id)
      setCompany(refreshed)
      await loadActivity(company.id)
      onChanged()
    } catch {
      logger.error("updateCompanyStatus failed", { id: company.id })
    } finally {
      setBusy(false)
    }
  }

  const addActivity = async (type: string, content: string) => {
    if (!company) return
    try {
      await api.addActivity({ company_id: company.id, type, content })
      await loadActivity(company.id)
    } catch {
      logger.error("addActivity failed", { id: company.id })
    }
  }

  const updateFollowup = async (when: string | null) => {
    if (!company) return
    try {
      await api.updateCompanyFollowup(company.id, when)
      setCompany(await api.getCompany(company.id))
      onChanged()
    } catch {
      logger.error("updateCompanyFollowup failed", { id: company.id })
    }
  }

  const updateContactPerson = async (person: string | null) => {
    if (!company) return
    try {
      await api.updateCompanyContactPerson(company.id, person)
      setCompany(await api.getCompany(company.id))
      onChanged()
    } catch {
      logger.error("updateCompanyContactPerson failed", { id: company.id })
    }
  }

  const onDelete = () => {
    if (!company) return
    setDeleteConfirmOpen(true)
  }

  const onDeleteConfirmed = async () => {
    if (!company) return
    setDeleteConfirmOpen(false)
    try {
      await api.deleteCompany(company.id)
      onOpenChange(false)
      onChanged()
    } catch {
      logger.error("deleteCompany failed", { id: company.id })
    }
  }

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent className="w-[480px] sm:max-w-none overflow-y-auto">
        {company ? (
          <>
            <SheetHeader>
              <div className="flex items-start gap-2">
                <SheetTitle className="text-lg">{company.name}</SheetTitle>
                <ScoreBadge score={company.probability_score} />
              </div>
              <SheetDescription>
                {[company.street, company.postal_code, company.city].filter(Boolean).join(", ") || "Adresse unbekannt"}
              </SheetDescription>
            </SheetHeader>

            <div className="mt-4 flex flex-wrap gap-2">
              {company.phone && (
                <Button variant="outline" size="sm" asChild>
                  <a href={`tel:${company.phone}`}><Phone className="size-3 mr-1" />{company.phone}</a>
                </Button>
              )}
              {company.email && (
                <Button variant="outline" size="sm" asChild>
                  <a href={`mailto:${company.email}`}><Mail className="size-3 mr-1" />Mail</a>
                </Button>
              )}
              {company.website && (
                <Button variant="outline" size="sm" asChild>
                  <a href={company.website} target="_blank" rel="noopener noreferrer">
                    <Globe className="size-3 mr-1" />Website
                  </a>
                </Button>
              )}
            </div>

            <div className="mt-6 space-y-4">
              <div>
                <div className="text-sm font-medium mb-1">Status</div>
                <StatusEditor value={company.status} onChange={updateStatus} />
              </div>
              <div>
                <div className="text-sm font-medium mb-1">Wiedervorlage</div>
                <FollowupEditor value={company.next_followup_at} onChange={updateFollowup} />
              </div>
              <div>
                <div className="text-sm font-medium mb-1">Ansprechpartner</div>
                <ContactPersonEditor value={company.contact_person} onCommit={updateContactPerson} />
              </div>
              <div>
                <div className="text-sm font-medium mb-2">Verlauf</div>
                <ActivityTimeline entries={activity} />
              </div>
              <AddActivityForm onSubmit={addActivity} />
            </div>

            <div className="mt-8 pt-4 border-t">
              <Button variant="destructive" size="sm" onClick={onDelete} disabled={busy}>
                <Trash2 className="size-3 mr-1" /> Firma löschen
              </Button>
            </div>
          </>
        ) : (
          <div className="text-sm text-muted-foreground">Lade…</div>
        )}
      </SheetContent>
      {company && (
        <ConfirmDialog
          open={deleteConfirmOpen}
          onOpenChange={setDeleteConfirmOpen}
          title="Firma löschen?"
          description={`"${company.name}" wird dauerhaft aus der Datenbank entfernt. Verbundene Aktivitäten und Notizen werden ebenfalls gelöscht.`}
          confirmLabel="Löschen"
          destructive
          onConfirm={onDeleteConfirmed}
        />
      )}
    </Sheet>
  )
}
