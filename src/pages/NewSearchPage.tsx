import { useEffect, useMemo, useState } from "react"
import { Button } from "@/components/ui/button"
import { api, type CategoryRow, type SearchStats, type ProgressEvent, type GeocodeSuggestion } from "@/lib/tauri"
import { CenterPickerMap, type Center } from "@/components/map/CenterPickerMap"
import { AddressSearchInput } from "@/components/search/AddressSearchInput"
import { RadiusSlider } from "@/components/search/RadiusSlider"
import { CategoryPicker } from "@/components/search/CategoryPicker"
import { logger } from "@/lib/logger"

export function NewSearchPage() {
  const [cats, setCats] = useState<CategoryRow[]>([])
  const [selectedCats, setSelectedCats] = useState<Set<number>>(new Set())
  const [center, setCenter] = useState<Center | null>(null)
  const [centerLabel, setCenterLabel] = useState<string | null>(null)
  const [radiusKm, setRadiusKm] = useState(25)
  const [progress, setProgress] = useState<ProgressEvent | null>(null)
  const [stats, setStats] = useState<SearchStats | null>(null)
  const [busy, setBusy] = useState(false)
  const [err, setErr] = useState<string | null>(null)

  useEffect(() => {
    api.listCategories()
      .then(all => {
        setCats(all)
        const raw = sessionStorage.getItem("loadProfile")
        if (raw) {
          sessionStorage.removeItem("loadProfile")
          try {
            const p = JSON.parse(raw) as {
              center_lat: number; center_lng: number; center_label: string;
              radius_km: number; enabled_category_ids: string;
            }
            setCenter({ lat: p.center_lat, lng: p.center_lng })
            setCenterLabel(p.center_label)
            setRadiusKm(p.radius_km)
            const ids: number[] = JSON.parse(p.enabled_category_ids)
            setSelectedCats(new Set(ids))
            logger.info("profile loaded", { radius_km: p.radius_km, cats: ids.length })
          } catch (e) {
            logger.error("profile load failed", { e: String(e) })
            setSelectedCats(new Set(all.filter(c => c.enabled).map(c => c.id)))
          }
        } else {
          setSelectedCats(new Set(all.filter(c => c.enabled).map(c => c.id)))
        }
      })
      .catch(e => logger.error("listCategories failed", { e: String(e) }))
    const unp = api.onSearchProgress(setProgress)
    const und = api.onSearchDone(setStats)
    return () => { unp.then(f => f()); und.then(f => f()) }
  }, [])

  const canStart = useMemo(
    () => !!center && selectedCats.size > 0 && !busy,
    [center, selectedCats, busy]
  )

  const onAddressPick = (s: GeocodeSuggestion) => {
    setCenter({ lat: s.lat, lng: s.lng })
    setCenterLabel(s.display_name)
    logger.info("address picked", { display_len: s.display_name.length })
  }

  const onMapClick = (c: Center) => {
    setCenter(c)
    setCenterLabel(null)
  }

  const saveAsProfile = async () => {
    if (!center) return
    const name = prompt("Name für das Profil:", centerLabel ?? `${center.lat.toFixed(3)}, ${center.lng.toFixed(3)}`)
    if (!name || !name.trim()) return
    try {
      await api.createSearchProfile({
        name: name.trim(),
        center_label: centerLabel ?? `${center.lat.toFixed(3)}, ${center.lng.toFixed(3)}`,
        center_lat: center.lat,
        center_lng: center.lng,
        radius_km: radiusKm,
        enabled_category_ids: JSON.stringify(Array.from(selectedCats)),
      })
      alert("Profil gespeichert.")
      logger.info("profile saved", { radius_km: radiusKm, cats: selectedCats.size })
    } catch (e) {
      alert(`Speichern fehlgeschlagen: ${e}`)
      logger.error("saveAsProfile failed", { e: String(e) })
    }
  }

  const runSearch = async () => {
    if (!center) return
    setBusy(true); setErr(null); setProgress(null); setStats(null)
    logger.info("search start", {
      lat: center.lat.toFixed(4),
      lng: center.lng.toFixed(4),
      radius_km: radiusKm,
      cats: selectedCats.size,
    })
    try {
      await api.startSearch({
        center_lat: center.lat,
        center_lng: center.lng,
        radius_km: radiusKm,
        category_ids: Array.from(selectedCats),
      })
    } catch (e) {
      setErr(String(e))
      logger.error("search failed", { e: String(e) })
    } finally {
      setBusy(false)
    }
  }

  return (
    <div className="h-full flex">
      {/* Linke Spalte: Formular */}
      <div className="w-96 border-r flex flex-col">
        <div className="p-4 border-b">
          <h2 className="text-lg font-semibold">Neue Suche</h2>
        </div>
        <div className="flex-1 overflow-y-auto p-4 space-y-5">
          <div className="space-y-2">
            <AddressSearchInput onPick={onAddressPick} placeholder="Adresse, Stadt oder PLZ…" />
            <p className="text-xs text-muted-foreground">
              {centerLabel ?? (center
                ? `Mittelpunkt: ${center.lat.toFixed(3)}, ${center.lng.toFixed(3)} (per Karten-Klick)`
                : "Oder klick rechts auf die Karte.")}
            </p>
          </div>

          <RadiusSlider value={radiusKm} onChange={setRadiusKm} />

          <CategoryPicker
            categories={cats}
            selected={selectedCats}
            onChange={setSelectedCats}
          />

          <div className="flex gap-2">
            <Button onClick={runSearch} disabled={!canStart} className="flex-1">
              {busy ? "Suche läuft…" : "Suche starten"}
            </Button>
            <Button variant="outline" onClick={saveAsProfile} disabled={!center || selectedCats.size === 0}>
              Speichern
            </Button>
          </div>

          {(progress || stats || err) && (
            <div className="text-sm space-y-1 pt-2 border-t">
              {progress && (
                <div>Tile {progress.tile_idx}/{progress.tile_total} · +{progress.last_count} (gesamt {progress.running_total_inserted})</div>
              )}
              {stats && (
                <div className="text-green-700 dark:text-green-400">
                  Fertig: {stats.neu_imported} neu / {stats.duplicates_skipped} Duplikate in {Math.round(stats.dauer_ms / 100) / 10}s
                </div>
              )}
              {err && <div className="text-red-600">{err}</div>}
            </div>
          )}
        </div>
      </div>

      {/* Rechte Spalte: Karte */}
      <div className="flex-1 relative">
        <CenterPickerMap
          center={center}
          radiusKm={radiusKm}
          onCenterChange={onMapClick}
        />
      </div>
    </div>
  )
}
