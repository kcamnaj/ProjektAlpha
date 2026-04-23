import { Slider } from "@/components/ui/slider"
import { Label } from "@/components/ui/label"

interface RadiusSliderProps {
  value: number
  onChange: (km: number) => void
  min?: number
  max?: number
}

export function RadiusSlider({ value, onChange, min = 1, max = 300 }: RadiusSliderProps) {
  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <Label>Radius</Label>
        <span className="text-sm font-medium tabular-nums">{value} km</span>
      </div>
      <Slider
        value={[value]}
        min={min}
        max={max}
        step={1}
        onValueChange={(v) => onChange(v[0])}
      />
      <div className="flex justify-between text-xs text-muted-foreground">
        <span>{min} km</span>
        <span>{max} km</span>
      </div>
    </div>
  )
}
