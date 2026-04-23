import { describe, it, expect, vi, beforeEach } from "vitest"
import { render, screen, fireEvent } from "@testing-library/react"
import { AddressSearchInput } from "./AddressSearchInput"

// invoke-Mock ist bereits in src/tests/setup.ts aktiv — wir überschreiben das return per-Test
import { invoke } from "@tauri-apps/api/core"

// Kleine Helper: echte Zeit warten, statt Fake-Timers (die mit findByText/waitFor
// und der async invoke-Promise-Chain in der Komponente kollidieren).
const waitRealMs = (ms: number) => new Promise(r => setTimeout(r, ms))

describe("AddressSearchInput", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset()
  })

  it("renders the input with placeholder", () => {
    render(<AddressSearchInput onPick={vi.fn()} placeholder="Test" />)
    expect(screen.getByPlaceholderText("Test")).toBeInTheDocument()
  })

  it("does NOT call geocode when query is shorter than 3 chars", async () => {
    render(<AddressSearchInput onPick={vi.fn()} debounceMs={30} />)
    fireEvent.change(screen.getByRole("textbox"), { target: { value: "Ha" } })
    await waitRealMs(80)
    expect(invoke).not.toHaveBeenCalled()
  })

  it("debounces the geocode call until after the debounce window", async () => {
    vi.mocked(invoke).mockResolvedValue([])
    render(<AddressSearchInput onPick={vi.fn()} debounceMs={30} />)
    const input = screen.getByRole("textbox")
    fireEvent.change(input, { target: { value: "Han" } })
    await waitRealMs(10)
    expect(invoke).not.toHaveBeenCalled()
    fireEvent.change(input, { target: { value: "Hann" } })
    await waitRealMs(10)
    expect(invoke).not.toHaveBeenCalled()
    await waitRealMs(60)
    expect(invoke).toHaveBeenCalledTimes(1)
  })

  it("calls onPick with the chosen suggestion", async () => {
    vi.mocked(invoke).mockResolvedValue([
      { lat: 52.37, lng: 9.73, display_name: "Hannover, Deutschland" },
    ])
    const onPick = vi.fn()
    render(<AddressSearchInput onPick={onPick} debounceMs={30} />)
    fireEvent.change(screen.getByRole("textbox"), { target: { value: "Hannover" } })
    const suggestion = await screen.findByText("Hannover, Deutschland")
    fireEvent.click(suggestion)
    expect(onPick).toHaveBeenCalledWith({ lat: 52.37, lng: 9.73, display_name: "Hannover, Deutschland" })
  })
})
