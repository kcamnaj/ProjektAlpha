import { describe, expect, it } from "vitest"
import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { LoadingState } from "./LoadingState"
import { EmptyState } from "./EmptyState"

describe("LoadingState", () => {
  it("renders the given message", () => {
    render(<LoadingState message="Lade Firmen…" />)
    expect(screen.getByText("Lade Firmen…")).toBeInTheDocument()
  })
  it("falls back to default German 'Lade…'", () => {
    render(<LoadingState />)
    expect(screen.getByText("Lade…")).toBeInTheDocument()
  })
})

describe("EmptyState", () => {
  it("renders title and hint", () => {
    render(<EmptyState title="Keine Firmen" hint="Starte eine neue Suche." />)
    expect(screen.getByText("Keine Firmen")).toBeInTheDocument()
    expect(screen.getByText("Starte eine neue Suche.")).toBeInTheDocument()
  })
  it("fires action callback when button clicked", async () => {
    let clicked = false
    render(
      <EmptyState
        title="X"
        hint="y"
        actionLabel="Los"
        onAction={() => { clicked = true }}
      />
    )
    await userEvent.click(screen.getByRole("button", { name: "Los" }))
    expect(clicked).toBe(true)
  })
})
