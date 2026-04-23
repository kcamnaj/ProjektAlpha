import { describe, expect, it } from "vitest"
import { formatDateDe, formatRelativeDe, statusLabel, statusColor, scoreColor, formatRelativeFollowup } from "./format"

describe("formatDateDe", () => {
  it("formats ISO to dd.MM.yyyy", () => {
    expect(formatDateDe("2026-04-21T14:30:00Z")).toBe("21.04.2026")
  })
  it("returns dash on null/undefined", () => {
    expect(formatDateDe(null)).toBe("—")
    expect(formatDateDe(undefined)).toBe("—")
  })
})

describe("formatRelativeDe", () => {
  it("returns 'heute' for today", () => {
    const today = new Date().toISOString()
    expect(formatRelativeDe(today)).toBe("heute")
  })
  it("returns 'vor X Tagen' for past dates", () => {
    const d = new Date(); d.setDate(d.getDate() - 5)
    expect(formatRelativeDe(d.toISOString())).toBe("vor 5 Tagen")
  })
})

describe("statusLabel", () => {
  it("maps status keys to German labels", () => {
    expect(statusLabel("neu")).toBe("Neu")
    expect(statusLabel("angefragt")).toBe("Angefragt")
    expect(statusLabel("kunde")).toBe("Kunde")
    expect(statusLabel("kein_kunde")).toBe("Kein Kunde")
  })
})

describe("statusColor", () => {
  it("returns distinct colors per status", () => {
    const colors = ["neu","angefragt","kunde","kein_kunde"].map(statusColor)
    expect(new Set(colors).size).toBe(4)
  })
})

describe("scoreColor", () => {
  it("returns red/yellow/green by tier", () => {
    expect(scoreColor(20)).toContain("red")
    expect(scoreColor(60)).toContain("yellow")
    expect(scoreColor(90)).toContain("green")
  })
})

describe("formatRelativeFollowup", () => {
  it("returns 'heute fällig' for today", () => {
    const today = new Date(); today.setHours(12, 0, 0, 0)
    expect(formatRelativeFollowup(today.toISOString())).toBe("heute fällig")
  })
  it("returns 'überfällig (vor X Tagen)' for past dates", () => {
    const d = new Date(); d.setDate(d.getDate() - 3); d.setHours(12, 0, 0, 0)
    expect(formatRelativeFollowup(d.toISOString())).toBe("überfällig (vor 3 Tagen)")
  })
  it("returns 'überfällig (seit gestern)' for yesterday", () => {
    const d = new Date(); d.setDate(d.getDate() - 1); d.setHours(12, 0, 0, 0)
    expect(formatRelativeFollowup(d.toISOString())).toBe("überfällig (seit gestern)")
  })
  it("returns 'in 1 Tag' for tomorrow", () => {
    const d = new Date(); d.setDate(d.getDate() + 1); d.setHours(12, 0, 0, 0)
    expect(formatRelativeFollowup(d.toISOString())).toBe("in 1 Tag")
  })
  it("returns '—' for null", () => {
    expect(formatRelativeFollowup(null)).toBe("—")
  })
})
