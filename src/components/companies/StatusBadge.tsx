import { statusLabel, statusColor } from "@/lib/format"
import { cn } from "@/lib/utils"

export function StatusBadge({ status }: { status: string }) {
  return (
    <span className={cn("inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium", statusColor(status))}>
      {statusLabel(status)}
    </span>
  )
}
