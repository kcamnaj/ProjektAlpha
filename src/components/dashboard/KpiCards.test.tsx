import { describe, expect, it } from "vitest"
import { render, screen } from "@testing-library/react"
import { KpiCards } from "./KpiCards"

describe("KpiCards", () => {
  it("renders all four KPI tiles with German labels", () => {
    render(<KpiCards kpis={{ customers: 7, requested: 12, new_count: 33, avg_score: 57.4, total_active: 52 }} />)
    expect(screen.getByText("Kunden")).toBeInTheDocument()
    expect(screen.getByText("Angefragt")).toBeInTheDocument()
    expect(screen.getByText("Neu")).toBeInTheDocument()
    expect(screen.getByText("Ø Score")).toBeInTheDocument()
    expect(screen.getByText("7")).toBeInTheDocument()
    expect(screen.getByText("12")).toBeInTheDocument()
    expect(screen.getByText("33")).toBeInTheDocument()
    expect(screen.getByText("57,4")).toBeInTheDocument() // deutsche Komma-Formatierung
  })

  it("renders '—' for avg_score when total_active is 0", () => {
    render(<KpiCards kpis={{ customers: 0, requested: 0, new_count: 0, avg_score: 0, total_active: 0 }} />)
    expect(screen.getByText("—")).toBeInTheDocument()
  })
})
