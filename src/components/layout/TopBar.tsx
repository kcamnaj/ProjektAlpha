import { Button } from "@/components/ui/button"
import { Moon, Sun, Download } from "lucide-react"
import { useEffect, useState } from "react"

export function TopBar() {
  const [dark, setDark] = useState(() =>
    document.documentElement.classList.contains("dark")
  )
  useEffect(() => {
    document.documentElement.classList.toggle("dark", dark)
  }, [dark])

  return (
    <header className="h-12 border-b flex items-center justify-end gap-2 px-4">
      <Button variant="ghost" size="icon" onClick={() => alert("Backup-Funktion kommt in Plan 4")}>
        <Download className="size-4" />
        <span className="sr-only">Backup</span>
      </Button>
      <Button variant="ghost" size="icon" onClick={() => setDark(d => !d)}>
        {dark ? <Sun className="size-4" /> : <Moon className="size-4" />}
        <span className="sr-only">Theme umschalten</span>
      </Button>
    </header>
  )
}
