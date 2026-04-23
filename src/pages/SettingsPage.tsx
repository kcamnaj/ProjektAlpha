import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { BranchenTab } from "@/components/settings/BranchenTab"
import { ProfileTab } from "@/components/settings/ProfileTab"
import { DatenTab } from "@/components/settings/DatenTab"
import { UeberTab } from "@/components/settings/UeberTab"

export function SettingsPage() {
  return (
    <div className="h-full flex flex-col">
      <div className="p-4 border-b">
        <h2 className="text-lg font-semibold">Einstellungen</h2>
      </div>
      <Tabs defaultValue="branchen" className="flex-1 flex flex-col">
        <TabsList className="mx-4 mt-3 self-start">
          <TabsTrigger value="branchen">Branchen</TabsTrigger>
          <TabsTrigger value="profile">Profile</TabsTrigger>
          <TabsTrigger value="daten">Daten</TabsTrigger>
          <TabsTrigger value="ueber">Über</TabsTrigger>
        </TabsList>
        <TabsContent value="branchen" className="flex-1 overflow-y-auto p-4"><BranchenTab /></TabsContent>
        <TabsContent value="profile" className="flex-1 overflow-y-auto p-4"><ProfileTab /></TabsContent>
        <TabsContent value="daten" className="flex-1 overflow-y-auto p-4"><DatenTab /></TabsContent>
        <TabsContent value="ueber" className="flex-1 overflow-y-auto p-4"><UeberTab /></TabsContent>
      </Tabs>
    </div>
  )
}
