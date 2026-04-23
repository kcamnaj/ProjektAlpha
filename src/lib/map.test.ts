import { describe, it, expect } from "vitest"
import {
  TILE_URL,
  TILE_ATTRIBUTION,
  pinColorForStatus,
  radiusCircleGeoJSON,
} from "./map"

describe("TILE_URL", () => {
  it("points to OpenStreetMap standard tiles", () => {
    expect(TILE_URL).toBe("https://tile.openstreetmap.org/{z}/{x}/{y}.png")
  })
})

describe("TILE_ATTRIBUTION", () => {
  it("credits OpenStreetMap contributors (OSM-policy)", () => {
    expect(TILE_ATTRIBUTION).toContain("OpenStreetMap")
  })
})

describe("pinColorForStatus", () => {
  it("returns blue for neu", () => {
    expect(pinColorForStatus("neu")).toBe("#3b82f6")
  })
  it("returns yellow for angefragt", () => {
    expect(pinColorForStatus("angefragt")).toBe("#eab308")
  })
  it("returns green for kunde", () => {
    expect(pinColorForStatus("kunde")).toBe("#22c55e")
  })
  it("returns gray for kein_kunde", () => {
    expect(pinColorForStatus("kein_kunde")).toBe("#9ca3af")
  })
  it("falls back to neu color for unknown status", () => {
    expect(pinColorForStatus("something_weird")).toBe("#3b82f6")
  })
})

describe("radiusCircleGeoJSON", () => {
  it("wraps a FeatureCollection with one Polygon feature", () => {
    const fc = radiusCircleGeoJSON({ lat: 52.3756, lng: 9.732 }, 5)
    expect(fc.type).toBe("FeatureCollection")
    expect(fc.features).toHaveLength(1)
    expect(fc.features[0].geometry.type).toBe("Polygon")
  })

  it("produces a ring with at least 32 vertices (smooth circle)", () => {
    const fc = radiusCircleGeoJSON({ lat: 52.0, lng: 9.0 }, 10)
    const coords = fc.features[0].geometry.coordinates[0]
    expect(coords.length).toBeGreaterThanOrEqual(32)
  })

  it("closes the ring (first point == last point)", () => {
    const fc = radiusCircleGeoJSON({ lat: 50.0, lng: 10.0 }, 25)
    const coords = fc.features[0].geometry.coordinates[0]
    expect(coords[0]).toEqual(coords[coords.length - 1])
  })

  it("radius roughly matches requested km (± 1%)", () => {
    const center = { lat: 52.0, lng: 9.0 }
    const fc = radiusCircleGeoJSON(center, 50)
    const coords = fc.features[0].geometry.coordinates[0]
    const latSpan = Math.max(...coords.map(c => c[1])) - center.lat
    const kmPerDegLat = 111.32
    const northKm = latSpan * kmPerDegLat
    expect(northKm).toBeGreaterThan(49.5)
    expect(northKm).toBeLessThan(50.5)
  })
})
