import { useCallback } from 'react'

import { TEXT_BLOCK_FONT_SIZES } from '../constants/textBlockConstants'
import type {
  AntennaSymbolNodeData,
  GroupingZoneNodeData,
  TextBlockNodeData,
} from '../nodeData/types'
import {
  deleteNode,
  replaceNodeType,
  updateNode,
  type DiagramState,
  type FlowNode,
} from '../tauriIpc'
import type { WiretagNodeData } from '../wiretagGraph'
import { wiretagIdsForPair, wiretagPairIndexTaken } from '../wiretagGraph'
import { useDebouncedNodeUpdate } from './useDebouncedNodeUpdate'

type UsePropertiesUpdateArgs = {
  diagramNodes: FlowNode[]
  setDiagramNodes: React.Dispatch<React.SetStateAction<FlowNode[]>>
  applyRustState: (result: DiagramState) => void | Promise<void>
  onSelectionCleared?: () => void
}

export function usePropertiesUpdate({
  diagramNodes,
  setDiagramNodes,
  applyRustState,
  onSelectionCleared,
}: UsePropertiesUpdateArgs) {
  const { scheduleNodeDataSync, flushAll } = useDebouncedNodeUpdate(applyRustState)

  const updateNodeData = useCallback(
    (nodeId: string, data: unknown) => {
      setDiagramNodes((nodes) =>
        nodes.map((n) => (n.id === nodeId ? { ...n, data } : n)),
      )
      scheduleNodeDataSync(nodeId, data)
    },
    [scheduleNodeDataSync, setDiagramNodes],
  )

  const handleReplaceAntennaNodeType = useCallback(
    async (
      nodeId: string,
      nodeType: 'antennaTransmitterSymbol' | 'antennaReceiverSymbol',
      data: AntennaSymbolNodeData,
    ) => {
      const result = await replaceNodeType(nodeId, nodeType, data)
      setDiagramNodes(result.nodes)
      await applyRustState(result)
    },
    [applyRustState, setDiagramNodes],
  )

  const handleUpdateTextBlock = useCallback(
    (nodeId: string, data: TextBlockNodeData) => {
      const fontSizePx = (TEXT_BLOCK_FONT_SIZES as readonly number[]).includes(data.fontSizePx)
        ? data.fontSizePx
        : 14
      updateNodeData(nodeId, { ...data, fontSizePx })
    },
    [updateNodeData],
  )

  const handleUpdateGroupingZone = useCallback(
    (nodeId: string, data: GroupingZoneNodeData) => {
      updateNodeData(nodeId, data)
    },
    [updateNodeData],
  )

  const patchNodeTagNotes = useCallback(
    (nodeId: string, tagNotes: string) => {
      const node = diagramNodes.find((n) => n.id === nodeId)
      if (!node) return
      if (node.type === 'textBlock') {
        handleUpdateTextBlock(nodeId, { ...(node.data as TextBlockNodeData), tagNotes })
        return
      }
      if (node.type === 'groupingZone') {
        handleUpdateGroupingZone(nodeId, { ...(node.data as GroupingZoneNodeData), tagNotes })
        return
      }
      updateNodeData(nodeId, { ...(node.data as Record<string, unknown>), tagNotes })
    },
    [diagramNodes, handleUpdateGroupingZone, handleUpdateTextBlock, updateNodeData],
  )

  const handleApplyWiretagPairIndex = useCallback(
    async (pairId: string, newIndex: number) => {
      const nInt = Math.round(Number(newIndex))
      if (!Number.isFinite(nInt) || nInt < 1) {
        window.alert('Pair index must be a positive integer.')
        return
      }
      if (wiretagPairIndexTaken(nInt, pairId, diagramNodes)) {
        window.alert('That pair index is already used by another wire tag pair.')
        return
      }
      const mates = diagramNodes.filter(
        (n) => n.type === 'wiretag' && (n.data as WiretagNodeData).pairId === pairId,
      )
      let result: DiagramState | undefined
      for (const n of mates) {
        const d = { ...(n.data as WiretagNodeData), pairIndex: nInt }
        result = await updateNode(n.id, d)
      }
      if (result) {
        setDiagramNodes(result.nodes)
        await applyRustState(result)
      }
    },
    [applyRustState, diagramNodes, setDiagramNodes],
  )

  const handleDeleteWiretagPair = useCallback(
    async (pairId: string) => {
      const ids = wiretagIdsForPair(pairId, diagramNodes)
      let result: DiagramState | undefined
      for (const id of ids) {
        result = await deleteNode(id)
      }
      if (result) {
        setDiagramNodes(result.nodes)
        await applyRustState(result)
      }
      onSelectionCleared?.()
    },
    [applyRustState, diagramNodes, onSelectionCleared, setDiagramNodes],
  )

  const handleSetWiretagPairTagDescription = useCallback(
    async (nodeId: string, tagDescription: string) => {
      const n = diagramNodes.find((x) => x.id === nodeId)
      if (!n) return
      const d = { ...(n.data as WiretagNodeData), tagDescription }
      const result = await updateNode(nodeId, d)
      setDiagramNodes(result.nodes)
      await applyRustState(result)
    },
    [applyRustState, diagramNodes, setDiagramNodes],
  )

  const handleSetWiretagPairSheetName = useCallback(
    async (nodeId: string, sheetName: string) => {
      const n = diagramNodes.find((x) => x.id === nodeId)
      if (!n) return
      const d = { ...(n.data as WiretagNodeData), sheetName }
      const result = await updateNode(nodeId, d)
      setDiagramNodes(result.nodes)
      await applyRustState(result)
    },
    [applyRustState, diagramNodes, setDiagramNodes],
  )

  const handleSetWiretagPairShowSheetName = useCallback(
    async (nodeId: string, showSheetName: boolean) => {
      const n = diagramNodes.find((x) => x.id === nodeId)
      if (!n) return
      const d = { ...(n.data as WiretagNodeData), showSheetName }
      const result = await updateNode(nodeId, d)
      setDiagramNodes(result.nodes)
      await applyRustState(result)
    },
    [applyRustState, diagramNodes, setDiagramNodes],
  )

  return {
    flushPropertiesPending: flushAll,
    updateNodeData,
    handleReplaceAntennaNodeType,
    handleUpdateTextBlock,
    handleUpdateGroupingZone,
    patchNodeTagNotes,
    handleApplyWiretagPairIndex,
    handleDeleteWiretagPair,
    handleSetWiretagPairTagDescription,
    handleSetWiretagPairSheetName,
    handleSetWiretagPairShowSheetName,
  }
}
