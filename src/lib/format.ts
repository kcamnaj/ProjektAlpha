export function formatDateDe(iso?: string | null): string {
  if (!iso) return "—"
  const d = new Date(iso)
  if (isNaN(d.getTime())) return "—"
  const dd = String(d.getDate()).padStart(2, "0")
  const mm = String(d.getMonth() + 1).padStart(2, "0")
  return `${dd}.${mm}.${d.getFullYear()}`
}

export function formatRelativeDe(iso?: string | null): string {
  if (!iso) return "—"
  const d = new Date(iso)
  const now = new Date()
  const days = Math.floor((now.getTime() - d.getTime()) / (1000 * 60 * 60 * 24))
  if (days === 0) return "heute"
  if (days === 1) return "gestern"
  if (days > 1) return `vor ${days} Tagen`
  if (days === -1) return "morgen"
  return `in ${-days} Tagen`
}

const STATUS_LABELS = { neu: "Neu", angefragt: "Angefragt", kunde: "Kunde", kein_kunde: "Kein Kunde" } as const
export type Status = keyof typeof STATUS_LABELS

export function statusLabel(s: string): string {
  return STATUS_LABELS[s as Status] ?? s
}

const STATUS_COLORS: Record<string, string> = {
  neu: "bg-blue-100 text-blue-900 dark:bg-blue-900/40 dark:text-blue-200",
  angefragt: "bg-yellow-100 text-yellow-900 dark:bg-yellow-900/40 dark:text-yellow-200",
  kunde: "bg-green-100 text-green-900 dark:bg-green-900/40 dark:text-green-200",
  kein_kunde: "bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-400",
}

export function statusColor(s: string): string {
  return STATUS_COLORS[s] ?? STATUS_COLORS.neu
}

export function scoreColor(score: number): string {
  if (score < 40) return "bg-red-100 text-red-900 dark:bg-red-900/30 dark:text-red-200"
  if (score < 75) return "bg-yellow-100 text-yellow-900 dark:bg-yellow-900/30 dark:text-yellow-200"
  return "bg-green-100 text-green-900 dark:bg-green-900/30 dark:text-green-200"
}

/**
 * Erzeugt ein Label für next_followup_at bezogen auf heute.
 * Tag-basiert (nicht stundengenau) — DATE-Grenzen zählen.
 */
export function formatRelativeFollowup(iso?: string | null): string {
  if (!iso) return "—"
  const target = new Date(iso)
  if (isNaN(target.getTime())) return "—"
  // Tag-Grenzen vergleichen, keine Stunden
  const t = new Date(target.getFullYear(), target.getMonth(), target.getDate())
  const now = new Date()
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  const days = Math.round((t.getTime() - today.getTime()) / (1000 * 60 * 60 * 24))
  if (days === 0) return "heute fällig"
  if (days === -1) return "überfällig (seit gestern)"
  if (days < -1) return `überfällig (vor ${-days} Tagen)`
  if (days === 1) return "in 1 Tag"
  return `in ${days} Tagen`
}
