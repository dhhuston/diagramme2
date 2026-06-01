import type { WireCategory } from '../tauriIpc'

export const WIRE_CATEGORY_OPTIONS: { value: WireCategory; label: string }[] = [
  { value: 'default', label: '—' },
  { value: 'audio', label: 'Audio' },
  { value: 'video', label: 'Video' },
  { value: 'control', label: 'Control' },
  { value: 'network', label: 'Network' },
  { value: 'rf', label: 'RF' },
  { value: 'power', label: 'Power' },
]

export function labelForWireCategory(cat: WireCategory | string): string {
  return WIRE_CATEGORY_OPTIONS.find((o) => o.value === cat)?.label ?? cat
}
