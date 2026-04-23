import { Loader2 } from "lucide-react"
import { cn } from "@/lib/utils"

interface LoadingStateProps {
  message?: string
  className?: string
}

export function LoadingState({ message = "Lade…", className }: LoadingStateProps) {
  return (
    <div className={cn("flex items-center gap-2 text-sm text-muted-foreground p-4", className)}>
      <Loader2 className="size-4 animate-spin" />
      <span>{message}</span>
    </div>
  )
}
