import { invoke } from "@tauri-apps/api/core"

type Level = "info" | "warn" | "error"

const send = (level: Level, message: string, context?: unknown) => {
  // local console
  // eslint-disable-next-line no-console
  (level === "error" ? console.error : level === "warn" ? console.warn : console.info)(
    `[${level}] ${message}`, context ?? ""
  )
  // forward to backend
  invoke("frontend_log", { payload: { level, message, context } }).catch(() => {
    // logger MUST never throw
  })
}

export const logger = {
  info:  (msg: string, ctx?: unknown) => send("info", msg, ctx),
  warn:  (msg: string, ctx?: unknown) => send("warn", msg, ctx),
  error: (msg: string, ctx?: unknown) => send("error", msg, ctx),
}
