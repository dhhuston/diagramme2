import type { HitTarget } from './canvas/sceneTypes'
import type { FlowNode } from './tauriIpc'

export type PropertiesPanelSelection = {
  kind: string
  node: FlowNode
}

const PROPERTIES_NODE_TYPES = new Set([
  'deviceV2',
  'device',
  'avPlate',
  'micBlock',
  'speakerBlock',
  'volumeControl',
  'antennaTransmitterSymbol',
  'antennaReceiverSymbol',
  'lppPatchPanel',
  'dppPatchPanel',
  'mlpPatchPanel',
  'vpbPatchPanel',
  'textBlock',
  'flyoffNote',
  'wiretag',
  'junction',
  'groupingZone',
])

/** Resolve a body-level node selection for the properties panel. */
export function resolvePropertiesSelection(
  selectedHit: HitTarget | null,
  diagramNodes: FlowNode[],
): PropertiesPanelSelection | null {
  if (!selectedHit?.node_id) return null
  if (selectedHit.handle_id != null) return null

  const node = diagramNodes.find((n) => n.id === selectedHit.node_id)
  if (!node) return null

  const kind = node.type === 'device' ? 'deviceV2' : node.type
  if (!PROPERTIES_NODE_TYPES.has(kind)) return null

  return { kind, node }
}

export function deriveNodeLabel(selection: PropertiesPanelSelection | null): string {
  if (!selection) return 'Selection'
  const { node } = selection
  const d = node.data as Record<string, unknown>
  switch (selection.kind) {
    case 'deviceV2': {
      const code = String(d.tagCode ?? '').trim()
      const num = String(d.tagNumber ?? '').trim()
      if (code || num) return `${code} / ${num}`.trim()
      return String(d.description ?? 'Device').trim() || 'Device'
    }
    case 'avPlate':
    case 'lppPatchPanel':
    case 'dppPatchPanel':
    case 'mlpPatchPanel':
    case 'vpbPatchPanel':
    case 'junction': {
      const code = String(d.tagCode ?? '').trim()
      const num = String(d.tagNumber ?? '').trim()
      return code || num ? `${code} / ${num}`.trim() : selection.kind
    }
    case 'micBlock':
    case 'speakerBlock':
      return String(d.line1 ?? selection.kind)
    case 'volumeControl':
      return 'Volume control'
    case 'antennaTransmitterSymbol':
    case 'antennaReceiverSymbol':
      return String(d.line1 ?? 'Antenna')
    case 'textBlock':
      return 'Text block'
    case 'flyoffNote':
      return 'Flyoff note'
    case 'wiretag':
      return 'Wire tag'
    case 'groupingZone':
      return String(d.label ?? 'Grouping zone')
    default:
      return selection.kind
  }
}
