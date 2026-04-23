import { Calendar } from "@/components/ui/calendar"
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover"
import { Button } from "@/components/ui/button"
import { CalendarDays, X } from "lucide-react"
import { formatDateDe } from "@/lib/format"

export function FollowupEditor({
  value, onChange,
}: { value: string | null; onChange: (next: string | null) => void }) {
  const date = value ? new Date(value) : undefined
  return (
    <div className="flex items-center gap-2">
      <Popover>
        <PopoverTrigger asChild>
          <Button variant="outline" size="sm" className="w-44 justify-start">
            <CalendarDays className="size-3 mr-2" />
            {value ? formatDateDe(value) : "Wiedervorlage setzen"}
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-auto p-0" align="start">
          <Calendar
            mode="single"
            selected={date}
            onSelect={(d) => onChange(d ? d.toISOString() : null)}
            initialFocus
          />
        </PopoverContent>
      </Popover>
      {value && (
        <Button variant="ghost" size="icon" onClick={() => onChange(null)}>
          <X className="size-3" />
        </Button>
      )}
    </div>
  )
}
