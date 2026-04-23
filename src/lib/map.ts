export const TILE_URL = "https://tile.openstreetmap.org/{z}/{x}/{y}.png"

export const TILE_ATTRIBUTION =
  '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'

const PIN_COLORS: Record<string, string> = {
  neu: "#3b82f6",
  angefragt: "#eab308",
  kunde: "#22c55e",
  kein_kunde: "#9ca3af",
}

export function pinColorForStatus(status: string): string {
  return PIN_COLORS[status] ?? PIN_COLORS.neu
}

/**
 * Approximates a geodesic circle as a polygon in WGS84.
 * Ausreichend für Darstellungszwecke bis ~300 km Radius — für größere
 * Distanzen müsste eine echte Greatcircle-Mathematik her (hier YAGNI).
 */
export function radiusCircleGeoJSON(
  center: { lat: number; lng: number },
  radiusKm: number,
  steps = 64,
): GeoJSON.FeatureCollection<GeoJSON.Polygon> {
  const earthRadiusKm = 6371.0088
  const deltaLatDeg = (radiusKm / earthRadiusKm) * (180 / Math.PI)
  const latRad = (center.lat * Math.PI) / 180
  const deltaLngDeg = deltaLatDeg / Math.max(Math.cos(latRad), 1e-6)

  const coords: [number, number][] = []
  for (let i = 0; i <= steps; i++) {
    const theta = (i / steps) * 2 * Math.PI
    const lat = center.lat + deltaLatDeg * Math.sin(theta)
    const lng = center.lng + deltaLngDeg * Math.cos(theta)
    coords.push([lng, lat])
  }

  return {
    type: "FeatureCollection",
    features: [
      {
        type: "Feature",
        properties: {},
        geometry: { type: "Polygon", coordinates: [coords] },
      },
    ],
  }
}
