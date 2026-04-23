import { describe, expect, it, vi } from "vitest"
import { render, screen } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { ConfirmDialog } from "./ConfirmDialog"

describe("ConfirmDialog", () => {
  it("renders title and description when open", () => {
    render(
      <ConfirmDialog
        open
        title="Sicher?"
        description="Diese Aktion ist nicht rückgängig machbar."
        confirmLabel="Löschen"
        destructive
        onConfirm={() => {}}
        onOpenChange={() => {}}
      />
    )
    expect(screen.getByText("Sicher?")).toBeInTheDocument()
    expect(screen.getByText("Diese Aktion ist nicht rückgängig machbar.")).toBeInTheDocument()
    expect(screen.getByRole("button", { name: "Löschen" })).toBeInTheDocument()
    expect(screen.getByRole("button", { name: "Abbrechen" })).toBeInTheDocument()
  })

  it("calls onConfirm when confirm button clicked", async () => {
    const onConfirm = vi.fn()
    const onOpenChange = vi.fn()
    render(
      <ConfirmDialog
        open
        title="X"
        description="y"
        confirmLabel="OK"
        onConfirm={onConfirm}
        onOpenChange={onOpenChange}
      />
    )
    await userEvent.click(screen.getByRole("button", { name: "OK" }))
    expect(onConfirm).toHaveBeenCalledOnce()
  })
})
