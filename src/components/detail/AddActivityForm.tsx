import { useState } from "react"
import { Textarea } from "@/components/ui/textarea"
import { Button } from "@/components/ui/button"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Plus } from "lucide-react"

const TYPES = [
  { v: "notiz", l: "Notiz" },
  { v: "anruf", l: "Anruf" },
  { v: "mail",  l: "Mail" },
  { v: "besuch",l: "Besuch" },
] as const

export function AddActivityForm({ onSubmit }: { onSubmit: (type: string, content: string) => Promise<void> }) {
  const [type, setType] = useState<string>("notiz")
  const [text, setText] = useState("")
  const [busy, setBusy] = useState(false)

  const submit = async () => {
    if (text.trim() === "") return
    setBusy(true)
    try {
      await onSubmit(type, text.trim())
      setText("")
    } finally { setBusy(false) }
  }

  return (
    <div className="border rounded-md p-3 space-y-2">
      <div className="flex items-center gap-2">
        <Select value={type} onValueChange={setType}>
          <SelectTrigger className="w-32"><SelectValue /></SelectTrigger>
          <SelectContent>
            {TYPES.map(t => <SelectItem key={t.v} value={t.v}>{t.l}</SelectItem>)}
          </SelectContent>
        </Select>
        <Button size="sm" onClick={submit} disabled={busy || !text.trim()}>
          <Plus className="size-3 mr-1" /> Erfassen
        </Button>
      </div>
      <Textarea
        placeholder="Was ist passiert? (z.B. Frau Müller bittet um Rückruf KW 18)"
        value={text}
        onChange={(e) => setText(e.target.value)}
        rows={3}
      />
    </div>
  )
}
