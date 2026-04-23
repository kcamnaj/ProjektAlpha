import { useEffect, useState } from "react"
import { api } from "@/lib/tauri"

export function UeberTab() {
  const [version, setVersion] = useState<string>("…")
  useEffect(() => {
    api.appVersion().then(setVersion).catch(() => setVersion("unbekannt"))
  }, [])
  return (
    <div className="space-y-3 max-w-xl">
      <h3 className="text-base font-semibold">ProjektAlpha</h3>
      <p className="text-sm">Version <span className="font-mono">{version}</span></p>
      <p className="text-sm text-muted-foreground">
        Lokales Lead-Management für Industrie-Tore, Verlade- und Hubbühnen, UVV-Prüfungen.
        Alle Daten bleiben auf diesem Rechner — keine Cloud, keine Telemetrie.
      </p>
      <p className="text-xs text-muted-foreground pt-2 border-t">
        Verwendet OpenStreetMap (Overpass, Nominatim, Tile-Server) unter ODbL-Lizenz.
      </p>
    </div>
  )
}
