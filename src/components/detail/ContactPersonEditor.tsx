import { Input } from "@/components/ui/input"
import { useState, useEffect } from "react"

export function ContactPersonEditor({
  value, onCommit,
}: { value: string | null; onCommit: (next: string | null) => void }) {
  const [v, setV] = useState(value ?? "")
  useEffect(() => { setV(value ?? "") }, [value])
  return (
    <Input
      placeholder="z.B. Frau Müller"
      value={v}
      onChange={(e) => setV(e.target.value)}
      onBlur={() => {
        const next = v.trim() === "" ? null : v.trim()
        if (next !== value) onCommit(next)
      }}
    />
  )
}
