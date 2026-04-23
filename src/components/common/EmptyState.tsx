import { Button } from "@/components/ui/button"
import { cn } from "@/lib/utils"
import type { ReactNode } from "react"

interface EmptyStateProps {
  title: string
  hint?: string
  icon?: ReactNode
  actionLabel?: string
  onAction?: () => void
  className?: string
}

export function EmptyState({ title, hint, icon, actionLabel, onAction, className }: EmptyStateProps) {
  return (
    <div className={cn("flex flex-col items-center justify-center gap-3 p-8 text-center", className)}>
      {icon && <div className="text-muted-foreground text-4xl">{icon}</div>}
      <div className="font-medium">{title}</div>
      {hint && <div className="text-sm text-muted-foreground max-w-xs">{hint}</div>}
      {actionLabel && onAction && (
        <Button variant="outline" size="sm" onClick={onAction}>{actionLabel}</Button>
      )}
    </div>
  )
}
