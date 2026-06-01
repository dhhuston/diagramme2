import type { WireCategory } from '../tauriIpc'
import type { GroupingZoneShape } from '../canvas/groupingZoneNode'
import type { NodeMakeModelFields } from '../nodeMakeModel'
import type { NodeTagNotesFields } from '../nodeTagNotes'
import type { FlowNode } from '../tauriIpc'

export type { GroupingZoneShape }

export type PortRow = {
  id: string
  label: string
  direction?: 'input' | 'output'
  wireCategory?: WireCategory
}

export type ColumnGroup = {
  header?: string
  rows: PortRow[]
  bundledRowIds?: string[][]
}

export type DeviceNodeData = NodeMakeModelFields & {
  tagCode: string
  tagNumber: string
  description: string
  splitInstance?: number
  leftColumn: ColumnGroup[]
  rightColumn: ColumnGroup[]
}

export type PlateStyle = 'input' | 'output'

export type AvPlateNodeData = NodeMakeModelFields & {
  tagCode: string
  tagNumber: string
  description: string
  splitInstance?: number
  plateStyle?: PlateStyle
  groups: ColumnGroup[]
  bundledLeft?: string[][]
  bundledRight?: string[][]
}

export type MicBlockNodeData = NodeMakeModelFields & {
  line1: string
  line2: string
  channelNumber: string
  wireCategory?: WireCategory
  splitInstance?: number
}

export type SpeakerBlockSymbolKind = 'standard' | '70v' | 'active'

export type SpeakerBlockNodeData = NodeMakeModelFields & {
  line1: string
  line2: string
  symbolKind?: SpeakerBlockSymbolKind
  passthruEnabled?: boolean
  wireCategory?: WireCategory
  splitInstance?: number
}

export type VolumeControlNodeData = NodeMakeModelFields & {
  splitInstance?: number
}

export type AntennaSymbolNodeData = NodeMakeModelFields & {
  line1?: string
  splitInstance?: number
}

export type LppPatchRow = { id: string; connected: boolean }

export type LppPatchPanelNodeData = NodeMakeModelFields & {
  tagCode: string
  tagNumber: string
  descriptionLines: string[]
  splitInstance?: number
  rows: LppPatchRow[]
  bundledLeft?: string[][]
  bundledRight?: string[][]
}

export type DppPatchRow = { id: string; label: string; direction: 'input' | 'output' }

export type DppPatchPanelNodeData = NodeMakeModelFields & {
  tagCode: string
  tagNumber: string
  descriptionLines: string[]
  splitInstance?: number
  rows: DppPatchRow[]
  bundledLeft?: string[][]
  bundledRight?: string[][]
}

export type MlpNormalling = 'HN' | 'FN' | ''
export type MlpPatchRow = { id: string; normalling: MlpNormalling }

export type MlpPatchPanelNodeData = NodeMakeModelFields & {
  tagCode: string
  tagNumber: string
  descriptionLines: string[]
  splitInstance?: number
  rows: MlpPatchRow[]
  bundledLeft?: string[][]
  bundledRight?: string[][]
}

export type VpbNormalling = 'N' | ''
export type VpbPatchRow = { id: string; normalling: VpbNormalling }

export type VpbPatchPanelNodeData = NodeMakeModelFields & {
  tagCode: string
  tagNumber: string
  descriptionLines: string[]
  splitInstance?: number
  rows: VpbPatchRow[]
  bundledLeft?: string[][]
  bundledRight?: string[][]
}

export type JunctionNodeData = NodeMakeModelFields & {
  tagCode: string
  tagNumber: string
  rowCount: number
  rowCategories?: WireCategory[]
  splitInstance?: number
}

export type TextBlockNodeData = NodeMakeModelFields &
  NodeTagNotesFields & {
    text: string
    fontSizePx: number
  }

export type FlyoffNoteNodeData = NodeMakeModelFields & {
  text: string
  portDirection?: 'input' | 'output'
  widthLocked?: boolean
  wireCategory?: WireCategory
}

export type GroupingZoneNodeData = NodeMakeModelFields &
  NodeTagNotesFields & {
    label?: string
    shape?: GroupingZoneShape
    polylinePoints?: number[]
  }

export type DeviceFlowNodeV2 = FlowNode
export type AvPlateFlowNode = FlowNode
export type MicBlockFlowNode = FlowNode
export type SpeakerBlockFlowNode = FlowNode
export type VolumeControlFlowNode = FlowNode
export type AntennaSymbolFlowNode = FlowNode
export type AntennaTransmitterFlowNode = FlowNode
export type AntennaReceiverFlowNode = FlowNode
export type LppPatchPanelFlowNode = FlowNode
export type DppPatchPanelFlowNode = FlowNode
export type MlpPatchPanelFlowNode = FlowNode
export type VpbPatchPanelFlowNode = FlowNode
export type TextBlockFlowNode = FlowNode
export type FlyoffNoteFlowNode = FlowNode
export type WiretagFlowNode = FlowNode
export type JunctionFlowNode = FlowNode
export type GroupingZoneFlowNode = FlowNode
