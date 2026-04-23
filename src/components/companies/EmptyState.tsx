import { Building2 } from "lucide-react"

export function EmptyState({ message }: { message: string }) {
  return (
    <div className="flex-1 flex flex-col items-center justify-center text-muted-foreground p-8">
      <Building2 className="size-12 mb-3 opacity-30" />
      <p className="text-sm">{message}</p>
    </div>
  )
}
