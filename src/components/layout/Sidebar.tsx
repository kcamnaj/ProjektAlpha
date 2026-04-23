import { useUiStore, type View } from "@/stores/uiStore"
import { Building2, Map, Settings, Search, LayoutDashboard } from "lucide-react"
import { cn } from "@/lib/utils"

const items: { key: View; label: string; Icon: typeof Building2 }[] = [
  { key: "dashboard", label: "Dashboard",    Icon: LayoutDashboard },
  { key: "companies", label: "Firmen",       Icon: Building2 },
  { key: "search",    label: "Neue Suche",   Icon: Search },
  { key: "map",       label: "Karte",        Icon: Map },
  { key: "settings",  label: "Einstellungen", Icon: Settings },
]

export function Sidebar() {
  const { currentView, setView } = useUiStore()
  return (
    <nav className="w-56 border-r bg-sidebar text-sidebar-foreground flex flex-col py-4">
      <div className="px-4 mb-6">
        <h1 className="font-semibold tracking-tight">ProjektAlpha</h1>
      </div>
      <ul className="space-y-1 px-2">
        {items.map(({ key, label, Icon }) => (
          <li key={key}>
            <button
              onClick={() => setView(key)}
              className={cn(
                "w-full flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-colors",
                "hover:bg-sidebar-accent hover:text-sidebar-accent-foreground",
                currentView === key && "bg-sidebar-accent text-sidebar-accent-foreground font-medium"
              )}
            >
              <Icon className="size-4" />
              {label}
            </button>
          </li>
        ))}
      </ul>
    </nav>
  )
}
