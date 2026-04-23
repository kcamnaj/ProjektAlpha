import { useEffect, useRef, useState } from "react"
import { Input } from "@/components/ui/input"
import { api, type GeocodeSuggestion } from "@/lib/tauri"
import { logger } from "@/lib/logger"

interface AddressSearchInputProps {
  onPick: (s: GeocodeSuggestion) => void
  placeholder?: string
  /** In ms. Default 500. Für Tests über-ridable. */
  debounceMs?: number
}

export function AddressSearchInput({ onPick, placeholder, debounceMs = 500 }: AddressSearchInputProps) {
  const [value, setValue] = useState("")
  const [suggestions, setSuggestions] = useState<GeocodeSuggestion[]>([])
  const [loading, setLoading] = useState(false)
  const [open, setOpen] = useState(false)
  const timerRef = useRef<number | null>(null)

  useEffect(() => {
    if (timerRef.current) window.clearTimeout(timerRef.current)
    if (value.trim().length < 3) {
      setSuggestions([])
      setOpen(false)
      return
    }
    timerRef.current = window.setTimeout(() => {
      setLoading(true)
      api.geocode(value)
        .then(results => {
          setSuggestions(results)
          setOpen(results.length > 0)
          logger.info("geocode done", { q_len: value.length, count: results.length })
        })
        .catch(e => logger.error("geocode failed", { e: String(e) }))
        .finally(() => setLoading(false))
    }, debounceMs)
    return () => {
      if (timerRef.current) window.clearTimeout(timerRef.current)
    }
  }, [value, debounceMs])

  const pick = (s: GeocodeSuggestion) => {
    onPick(s)
    setValue(s.display_name)
    setOpen(false)
  }

  return (
    <div className="relative">
      <Input
        value={value}
        onChange={(e) => setValue(e.target.value)}
        placeholder={placeholder ?? "Adresse, Stadt oder PLZ…"}
        onFocus={() => { if (suggestions.length > 0) setOpen(true) }}
        onBlur={() => setTimeout(() => setOpen(false), 200)}
      />
      {loading && (
        <div className="absolute right-3 top-2.5 text-xs text-muted-foreground">…</div>
      )}
      {open && suggestions.length > 0 && (
        <ul className="absolute left-0 right-0 top-full mt-1 z-50 bg-popover text-popover-foreground border rounded-md shadow max-h-60 overflow-y-auto">
          {suggestions.map((s, i) => (
            <li key={i}>
              <button
                type="button"
                className="w-full text-left px-3 py-2 text-sm hover:bg-accent truncate"
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => pick(s)}
              >
                {s.display_name}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
