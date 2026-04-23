import { useEffect, useRef } from "react"
import maplibregl from "maplibre-gl"
import { TILE_URL, TILE_ATTRIBUTION, radiusCircleGeoJSON } from "@/lib/map"
import { logger } from "@/lib/logger"

export interface Center {
  lat: number
  lng: number
}

interface CenterPickerMapProps {
  center: Center | null
  radiusKm: number
  onCenterChange: (c: Center) => void
  /** Startansicht (vor dem ersten Klick). Default: Deutschland-Mitte. */
  initialView?: { lat: number; lng: number; zoom: number }
}

export function CenterPickerMap({ center, radiusKm, onCenterChange, initialView }: CenterPickerMapProps) {
  const containerRef = useRef<HTMLDivElement | null>(null)
  const mapRef = useRef<maplibregl.Map | null>(null)
  const markerRef = useRef<maplibregl.Marker | null>(null)
  const styleLoadedRef = useRef(false)
  const onCenterChangeRef = useRef(onCenterChange)
  useEffect(() => { onCenterChangeRef.current = onCenterChange }, [onCenterChange])

  // Map init
  useEffect(() => {
    const container = containerRef.current
    if (!container) return

    const map = new maplibregl.Map({
      container,
      style: {
        version: 8,
        sources: { osm: { type: "raster", tiles: [TILE_URL], tileSize: 256, attribution: TILE_ATTRIBUTION } },
        layers: [{ id: "osm", type: "raster", source: "osm" }],
      },
      center: [initialView?.lng ?? 10.45, initialView?.lat ?? 51.16],
      zoom: initialView?.zoom ?? 5.5,
      attributionControl: false,
    })
    map.addControl(new maplibregl.AttributionControl({ compact: true }), "bottom-right")
    map.addControl(new maplibregl.NavigationControl({ showCompass: false }), "top-right")

    map.on("load", () => {
      styleLoadedRef.current = true
      map.addSource("radius", { type: "geojson", data: { type: "FeatureCollection", features: [] } })
      map.addLayer({
        id: "radius-fill",
        type: "fill",
        source: "radius",
        paint: { "fill-color": "#3b82f6", "fill-opacity": 0.12 },
      })
      map.addLayer({
        id: "radius-line",
        type: "line",
        source: "radius",
        paint: { "line-color": "#3b82f6", "line-width": 2 },
      })
    })

    map.on("click", (e) => {
      const next = { lat: e.lngLat.lat, lng: e.lngLat.lng }
      onCenterChangeRef.current(next)
      logger.info("map center pick", { lat: next.lat.toFixed(4), lng: next.lng.toFixed(4) })
    })

    mapRef.current = map
    logger.info("map init", { page: "CenterPickerMap" })

    return () => {
      map.remove()
      mapRef.current = null
      markerRef.current = null
      styleLoadedRef.current = false
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // Marker + Kreis aktualisieren wenn sich center/radius ändern
  useEffect(() => {
    const map = mapRef.current
    if (!map) return

    const applyData = () => {
      if (!center) {
        markerRef.current?.remove()
        markerRef.current = null
        const src = map.getSource("radius") as maplibregl.GeoJSONSource | undefined
        src?.setData({ type: "FeatureCollection", features: [] })
        return
      }
      if (!markerRef.current) {
        markerRef.current = new maplibregl.Marker({ color: "#3b82f6" })
      }
      markerRef.current.setLngLat([center.lng, center.lat]).addTo(map)
      const src = map.getSource("radius") as maplibregl.GeoJSONSource | undefined
      src?.setData(radiusCircleGeoJSON(center, radiusKm))
      map.flyTo({
        center: [center.lng, center.lat],
        zoom: Math.max(map.getZoom(), 11),
        speed: 1.2,
        curve: 1.4,
      })
    }

    if (styleLoadedRef.current) applyData()
    else map.once("load", applyData)
  }, [center, radiusKm])

  return <div ref={containerRef} className="w-full h-full" />
}
