import "@testing-library/jest-dom/vitest"
import { vi } from "vitest"

// stub Tauri's invoke so component tests don't hit the bridge
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async () => null),
}))

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async () => () => {}),
}))

// stub logger so frontend_log invoke calls don't pollute invoke call counts in tests
vi.mock("@/lib/logger", () => ({
  logger: {
    info:  vi.fn(),
    warn:  vi.fn(),
    error: vi.fn(),
  },
}))
