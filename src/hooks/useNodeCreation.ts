import { useCallback, useMemo } from 'react'

import {
  GROUPING_ZONE_DEFAULT_H,
  GROUPING_ZONE_DEFAULT_W,
  type GroupingZoneData,
} from '../canvas/groupingZoneNode'
import {
  PX_PER_INCH,
  VOLUME_CONTROL_FRAME_HEIGHT_PX,
  WIRETAG_BAR_HEIGHT_PX,
  WIRETAG_INITIAL_WIDTH_PX,
  snapPoint,
} from '../canvas/paperScale'
import {
  createDefaultAntennaSymbolData,
  createDefaultAvPlateData,
  createDefaultDeviceData,
  createDefaultDppPatchPanelData,
  createDefaultLppPatchPanelData,
  createDefaultMicBlockData,
  createDefaultMlpPatchPanelData,
  createDefaultSpeakerBlockData,
  createDefaultVolumeControlData,
  createDefaultVpbPatchPanelData,
} from '../defaultNodeData'
import { addNode, type DiagramState, type FlowNode } from '../tauriIpc'
import { nextWiretagPairIndex, type WiretagNodeData } from '../wiretagGraph'

type FlowPoint = { x: number; y: number }

export type NodeCreationKey =
  | 'device'
  | 'avPlate'
  | 'micBlock'
  | 'speakerBlock'
  | 'volumeControl'
  | 'antenna'
  | 'lppPatchPanel'
  | 'dppPatchPanel'
  | 'mlpPatchPanel'
  | 'vpbPatchPanel'
  | 'textBlock'
  | 'flyoffNote'
  | 'groupingZone'

export type NodeCreationAction = (pos?: FlowPoint) => Promise<FlowNode | null>
export type NodeCreationActions = Record<NodeCreationKey, NodeCreationAction>

type NodeCreationEntry = {
  idPrefix: string
  type: string
  defaultPosition: (nodeCount: number) => FlowPoint
  count?: (nodes: FlowNode[]) => number
  createData: (count: number) => unknown
  extras?: Pick<FlowNode, 'width' | 'height' | 'zIndex'>
}

type UseNodeCreationArgs = {
  nodes: FlowNode[]
  applyRustState: (result: DiagramState) => void | Promise<void>
  getInsertPosition?: () => FlowPoint | null
}

function nextCount(nodes: FlowNode[], type: string): number {
  return nodes.filter((n) => n.type === type).length + 1
}

const nodeCreationRegistry: Record<NodeCreationKey, NodeCreationEntry> = {
  device: {
    idPrefix: 'device',
    type: 'deviceV2',
    defaultPosition: (n) => ({ x: 128 + (n % 4) * 64, y: 128 + Math.floor(n / 4) * 64 }),
    count: (nodes) => nextCount(nodes, 'deviceV2'),
    createData: createDefaultDeviceData,
  },
  avPlate: {
    idPrefix: 'avplate',
    type: 'avPlate',
    defaultPosition: (n) => ({ x: 160 + (n % 4) * 64, y: 160 + Math.floor(n / 4) * 64 }),
    count: (nodes) => nextCount(nodes, 'avPlate'),
    createData: createDefaultAvPlateData,
  },
  micBlock: {
    idPrefix: 'mic',
    type: 'micBlock',
    defaultPosition: (n) => ({ x: 64 + (n % 4) * 32, y: 128 + Math.floor(n / 4) * 64 }),
    count: (nodes) => nextCount(nodes, 'micBlock'),
    createData: createDefaultMicBlockData,
  },
  speakerBlock: {
    idPrefix: 'spk',
    type: 'speakerBlock',
    defaultPosition: (n) => ({ x: 320 + (n % 4) * 40, y: 180 + Math.floor(n / 4) * 48 }),
    count: (nodes) => nextCount(nodes, 'speakerBlock'),
    createData: createDefaultSpeakerBlockData,
  },
  volumeControl: {
    idPrefix: 'vc',
    type: 'volumeControl',
    defaultPosition: (n) => ({ x: 200 + (n % 4) * 36, y: 200 + Math.floor(n / 4) * 52 }),
    createData: createDefaultVolumeControlData,
    extras: { width: PX_PER_INCH, height: VOLUME_CONTROL_FRAME_HEIGHT_PX },
  },
  antenna: {
    idPrefix: 'ant',
    type: 'antennaTransmitterSymbol',
    defaultPosition: (n) => ({ x: 240 + (n % 4) * 36, y: 220 + Math.floor(n / 4) * 52 }),
    createData: createDefaultAntennaSymbolData,
  },
  lppPatchPanel: {
    idPrefix: 'lpp',
    type: 'lppPatchPanel',
    defaultPosition: (n) => ({ x: 640 + (n % 3) * 28, y: 60 + Math.floor(n / 3) * 120 }),
    count: (nodes) => nextCount(nodes, 'lppPatchPanel'),
    createData: createDefaultLppPatchPanelData,
  },
  dppPatchPanel: {
    idPrefix: 'dpp',
    type: 'dppPatchPanel',
    defaultPosition: (n) => ({ x: 780 + (n % 3) * 28, y: 60 + Math.floor(n / 3) * 120 }),
    count: (nodes) => nextCount(nodes, 'dppPatchPanel'),
    createData: createDefaultDppPatchPanelData,
  },
  mlpPatchPanel: {
    idPrefix: 'mlp',
    type: 'mlpPatchPanel',
    defaultPosition: (n) => ({ x: 640 + (n % 3) * 28, y: 200 + Math.floor(n / 3) * 120 }),
    count: (nodes) => nextCount(nodes, 'mlpPatchPanel'),
    createData: createDefaultMlpPatchPanelData,
  },
  vpbPatchPanel: {
    idPrefix: 'vpb',
    type: 'vpbPatchPanel',
    defaultPosition: (n) => ({ x: 640 + (n % 3) * 28, y: 340 + Math.floor(n / 3) * 120 }),
    count: (nodes) => nextCount(nodes, 'vpbPatchPanel'),
    createData: createDefaultVpbPatchPanelData,
  },
  textBlock: {
    idPrefix: 'text',
    type: 'textBlock',
    defaultPosition: (n) => ({ x: 100 + (n % 5) * 32, y: 400 + Math.floor(n / 5) * 36 }),
    createData: () => ({ text: 'Text', fontSizePx: 14 }),
    extras: { width: 260, height: 120 },
  },
  flyoffNote: {
    idPrefix: 'flyoff',
    type: 'flyoffNote',
    defaultPosition: (n) => ({ x: 120 + (n % 5) * 32, y: 360 + Math.floor(n / 5) * 36 }),
    createData: () => ({ text: 'FLYOFF NOTE', portDirection: 'output' }),
    extras: { width: 60, height: 12 },
  },
  groupingZone: {
    idPrefix: 'zone',
    type: 'groupingZone',
    defaultPosition: (n) => ({ x: 60 + (n % 3) * 48, y: 60 + Math.floor(n / 3) * 48 }),
    createData: () => ({ label: 'Zone', shape: 'rect' } satisfies GroupingZoneData),
    extras: { width: GROUPING_ZONE_DEFAULT_W, height: GROUPING_ZONE_DEFAULT_H, zIndex: -1 },
  },
}

function createRegisteredNode(key: NodeCreationKey, nodes: FlowNode[], pos?: FlowPoint): FlowNode {
  const entry = nodeCreationRegistry[key]
  const count = entry.count?.(nodes) ?? 0
  const rawPosition = pos ?? entry.defaultPosition(nodes.length)
  return {
    id: `${entry.idPrefix}-${crypto.randomUUID().slice(0, 8)}`,
    type: entry.type,
    position: snapPoint(rawPosition),
    data: entry.createData(count),
    ...entry.extras,
  }
}

export function useNodeCreation({
  nodes,
  applyRustState,
  getInsertPosition,
}: UseNodeCreationArgs) {
  const resolvePosition = useCallback(
    (pos?: FlowPoint) => pos ?? getInsertPosition?.() ?? undefined,
    [getInsertPosition],
  )

  const createNode = useCallback(
    async (key: NodeCreationKey, pos?: FlowPoint): Promise<FlowNode | null> => {
      const node = createRegisteredNode(key, nodes, resolvePosition(pos))
      const result = await addNode(node)
      await applyRustState(result)
      return node
    },
    [applyRustState, nodes, resolvePosition],
  )

  const createWiretagPair = useCallback(
    async (pos?: FlowPoint): Promise<FlowNode | null> => {
      const pairId = crypto.randomUUID()
      const pairIndex = nextWiretagPairIndex(nodes)
      const n = nodes.length
      const base = resolvePosition(pos)
      const baseX = base?.x ?? 100 + (n % 5) * 40
      const baseY = base?.y ?? 320 + Math.floor(n / 5) * 24
      const idA = `wiretag-${crypto.randomUUID().slice(0, 8)}`
      const idB = `wiretag-${crypto.randomUUID().slice(0, 8)}`
      const w = WIRETAG_INITIAL_WIDTH_PX
      const h = WIRETAG_BAR_HEIGHT_PX
      const dataBase = {
        pairId,
        pairIndex,
        tagDescription: '',
        sheetName: '',
        showSheetName: false,
      }
      const r1 = await addNode({
        id: idA,
        type: 'wiretag',
        position: snapPoint({ x: baseX, y: baseY }),
        width: w,
        height: h,
        data: { ...dataBase, end: 'a' } satisfies WiretagNodeData,
      })
      await applyRustState(r1)
      const r2 = await addNode({
        id: idB,
        type: 'wiretag',
        position: snapPoint({ x: baseX + w + 24, y: baseY }),
        width: w,
        height: h,
        data: { ...dataBase, end: 'b' } satisfies WiretagNodeData,
      })
      await applyRustState(r2)
      return { id: idB, type: 'wiretag', position: { x: baseX + w + 24, y: baseY }, data: dataBase }
    },
    [applyRustState, nodes, resolvePosition],
  )

  const nodeActions = useMemo(
    () =>
      ({
        device: (pos?: FlowPoint) => createNode('device', pos),
        avPlate: (pos?: FlowPoint) => createNode('avPlate', pos),
        micBlock: (pos?: FlowPoint) => createNode('micBlock', pos),
        speakerBlock: (pos?: FlowPoint) => createNode('speakerBlock', pos),
        volumeControl: (pos?: FlowPoint) => createNode('volumeControl', pos),
        antenna: (pos?: FlowPoint) => createNode('antenna', pos),
        lppPatchPanel: (pos?: FlowPoint) => createNode('lppPatchPanel', pos),
        dppPatchPanel: (pos?: FlowPoint) => createNode('dppPatchPanel', pos),
        mlpPatchPanel: (pos?: FlowPoint) => createNode('mlpPatchPanel', pos),
        vpbPatchPanel: (pos?: FlowPoint) => createNode('vpbPatchPanel', pos),
        textBlock: (pos?: FlowPoint) => createNode('textBlock', pos),
        flyoffNote: (pos?: FlowPoint) => createNode('flyoffNote', pos),
        groupingZone: (pos?: FlowPoint) => createNode('groupingZone', pos),
      }) satisfies NodeCreationActions,
    [createNode],
  )

  return {
    createNode,
    createWiretagPair,
    nodeActions,
  }
}
