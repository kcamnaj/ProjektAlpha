import { scoreColor } from "@/lib/format"
import { cn } from "@/lib/utils"

export function ScoreBadge({ score }: { score: number }) {
  return (
    <span className={cn("inline-flex items-center px-2 py-0.5 rounded text-xs font-medium tabular-nums", scoreColor(score))}>
      {score}%
    </span>
  )
}
