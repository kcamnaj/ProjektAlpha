import { useEffect, useRef } from "react"
import maplibregl from "maplibre-gl"
import type { CompanyRow } from "@/lib/tauri"
import { TILE_URL, TILE_ATTRIBUTION, pinColorForStatus } from "@/lib/map"
import { logger } from "@/lib/logger"

interface MapViewProps {
  companies: CompanyRow[]
  selectedId: string | null
  onSelect: (id: string) => void
  /** Startansicht — Default: Deutschland-weit. */
  initialView?: { lat: number; lng: number; zoom: number }
}

export function MapView({ companies, selectedId, onSelect, initialView }: MapViewProps) {
  const containerRef = useRef<HTMLDivElement | null>(null)
  const mapRef = useRef<maplibregl.Map | null>(null)
  const markersRef = useRef<globalThis.Map<string, maplibregl.Marker>>(new globalThis.Map())
  // onSelect in ref halten, damit Marker-Klick-Handler immer die aktuelle Closure hat
  // (ohne Marker bei jedem onSelect-Wechsel neu zu bauen)
  const onSelectRef = useRef(onSelect)
  useEffect(() => { onSelectRef.current = onSelect }, [onSelect])

  // Map-Lifecycle: einmal bei Mount aufbauen, bei Unmount zerstören.
  useEffect(() => {
    const container = containerRef.current
    if (!container) return

    const map = new maplibregl.Map({
      container,
      style: {
        version: 8,
        sources: {
          osm: {
            type: "raster",
            tiles: [TILE_URL],
            tileSize: 256,
            attribution: TILE_ATTRIBUTION,
          },
        },
        layers: [{ id: "osm", type: "raster", source: "osm" }],
      },
      center: [initialView?.lng ?? 10.45, initialView?.lat ?? 51.16],
      zoom: initialView?.zoom ?? 5.5,
      attributionControl: false,
    })
    map.addControl(new maplibregl.AttributionControl({ compact: true }), "bottom-right")
    map.addControl(new maplibregl.NavigationControl({ showCompass: false }), "top-right")
    mapRef.current = map
    logger.info("map init", { page: "MapView" })

    return () => {
      map.remove()
      mapRef.current = null
      markersRef.current.clear()
    }
    // Bewusst nur bei Mount — initialView-Änderungen triggern keinen Remount
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // Marker-Sync: Diff-basiert
  useEffect(() => {
    const map = mapRef.current
    if (!map) return
    const current = markersRef.current
    const incoming = new Set(companies.map(c => c.id))

    // 1) Veraltete Marker entfernen
    for (const [id, marker] of current) {
      if (!incoming.has(id)) {
        marker.remove()
        current.delete(id)
      }
    }

    // 2) Neue Marker hinzufügen + existierende ggf. aktualisieren
    for (const c of companies) {
      const existing = current.get(c.id)
      if (existing) {
        existing.setLngLat([c.lng, c.lat])
        existing.getElement().style.background = pinColorForStatus(c.status)
        continue
      }
      const el = document.createElement("div")
      el.dataset.companyId = c.id
      el.style.width = "14px"
      el.style.height = "14px"
      el.style.borderRadius = "50%"
      el.style.background = pinColorForStatus(c.status)
      el.style.border = "2px solid white"
      el.style.boxShadow = "0 1px 3px rgba(0,0,0,.4)"
      el.style.cursor = "pointer"
      el.setAttribute("aria-label", `Firma ${c.name}`)
      el.addEventListener("click", (e) => {
        e.stopPropagation()
        onSelectRef.current(c.id)
        logger.info("pin click", { company_id: c.id })
      })
      const marker = new maplibregl.Marker({ element: el }).setLngLat([c.lng, c.lat]).addTo(map)
      current.set(c.id, marker)
    }
  }, [companies])

  // Hervorhebung + Pan auf selected
  useEffect(() => {
    for (const [id, marker] of markersRef.current) {
      const el = marker.getElement()
      if (id === selectedId) {
        el.style.transform = "scale(1.5)"
        el.style.boxShadow = "0 0 0 4px rgba(59,130,246,.5), 0 1px 3px rgba(0,0,0,.4)"
        el.style.zIndex = "10"
      } else {
        el.style.transform = "scale(1)"
        el.style.boxShadow = "0 1px 3px rgba(0,0,0,.4)"
        el.style.zIndex = "1"
      }
    }
    const map = mapRef.current
    if (map && selectedId) {
      const marker = markersRef.current.get(selectedId)
      if (marker) {
        const { lng, lat } = marker.getLngLat()
        map.easeTo({ center: [lng, lat], duration: 400 })
      }
    }
  }, [selectedId])

  return <div ref={containerRef} className="w-full h-full" />
}
