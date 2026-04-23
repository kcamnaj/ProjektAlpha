import { describe, expect, it } from "vitest"
import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { FilterBar } from "./FilterBar"
import { useFilterStore } from "@/stores/filterStore"

describe("FilterBar", () => {
  it("typing in search updates the store", async () => {
    useFilterStore.getState().reset()
    render(<FilterBar onAdded={() => {}} />)
    const input = screen.getByPlaceholderText(/Firma oder Stadt/i)
    await userEvent.type(input, "müller")
    expect(useFilterStore.getState().search).toBe("müller")
  })

  it("reset clears all filters", async () => {
    useFilterStore.getState().setSearch("foo")
    useFilterStore.getState().setMinScore(50)
    render(<FilterBar onAdded={() => {}} />)
    await userEvent.click(screen.getByRole("button", { name: /Reset/i }))
    const s = useFilterStore.getState()
    expect(s.search).toBe("")
    expect(s.minScore).toBe(0)
  })
})
