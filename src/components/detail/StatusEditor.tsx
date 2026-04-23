import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { statusLabel } from "@/lib/format"

export function StatusEditor({
  value, onChange,
}: { value: string; onChange: (next: string) => void }) {
  return (
    <Select value={value} onValueChange={onChange}>
      <SelectTrigger className="w-48"><SelectValue /></SelectTrigger>
      <SelectContent>
        {(["neu","angefragt","kunde","kein_kunde"] as const).map(s => (
          <SelectItem key={s} value={s}>{statusLabel(s)}</SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}
