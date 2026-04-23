import { Component, type ErrorInfo, type ReactNode } from "react"
import { invoke } from "@tauri-apps/api/core"

type Props = { children: ReactNode }
type State = { hasError: boolean; error: Error | null }

export class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false, error: null }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, info: ErrorInfo): void {
    invoke("report_frontend_crash", {
      payload: {
        message: error.message,
        stack: `${error.stack ?? ""}\n${info.componentStack ?? ""}`,
      },
    }).catch(() => {
      /* never throw from boundary */
    })
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="p-8 max-w-2xl">
          <h1 className="text-2xl font-bold text-red-600">Etwas ist schiefgelaufen</h1>
          <p className="mt-2 text-sm text-gray-600">
            Die App hat einen unerwarteten Fehler erlebt. Der Fehler wurde lokal gespeichert.
          </p>
          <pre className="mt-4 p-3 bg-gray-100 dark:bg-gray-800 rounded text-xs overflow-auto">
            {this.state.error?.message}
          </pre>
          <button
            onClick={() => window.location.reload()}
            className="mt-4 px-4 py-2 bg-blue-600 text-white rounded"
          >
            App neu laden
          </button>
        </div>
      )
    }
    return this.props.children
  }
}
