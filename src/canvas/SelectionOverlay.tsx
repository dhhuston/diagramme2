import type { ReactNode } from 'react'
import { Line, Rect } from 'react-konva'

import { getCanvasSelectionStroke } from './canvasTokens'
import { dragVisualDelta } from './interaction/dragNode'
import type { NodeDragTarget } from './interaction/dragNode'
import {
  findGroupingZoneNode,
  GroupingZoneSelectionOutline,
  isGroupingZoneSelectionTarget,
} from './GroupingZoneSelectionOutline'
import { nodeSelectionBounds } from './selectionBounds'
import { polylineToKonvaPoints, primitiveKey, SCHEMATIC_STROKE_PROPS } from './sceneRenderUtils'
import type { HitTarget, SceneJson } from './sceneTypes'
import type { FlowNode } from '../tauriIpc'

const SELECTION_STROKE_PX = 2

type SelectionOverlayProps = {
  scene: SceneJson
  selectedHit: HitTarget | null
  nodeDrag?: NodeDragTarget | null
  diagramNodes?: FlowNode[]
}

function selectionOffset(
  hits: HitTarget[],
  nodeId: string,
  nodeDrag: NodeDragTarget | null | undefined,
) {
  if (!nodeDrag || nodeDrag.nodeId !== nodeId) {
    return { x: 0, y: 0 }
  }
  return dragVisualDelta(hits, nodeId, nodeDrag.targetOrigin) ?? { x: 0, y: 0 }
}

function NodeSelectionOutline({
  bounds,
  offsetX,
  offsetY,
}: {
  bounds: { x: number; y: number; width: number; height: number }
  offsetX: number
  offsetY: number
}) {
  return (
    <Rect
      key="selection-node"
      x={bounds.x + offsetX}
      y={bounds.y + offsetY}
      width={bounds.width}
      height={bounds.height}
      stroke={getCanvasSelectionStroke()}
      strokeWidth={SELECTION_STROKE_PX}
      listening={false}
      perfectDrawEnabled={false}
    />
  )
}

function WireSelectionHighlight({ scene, edgeId }: { scene: SceneJson; edgeId: string }) {
  const stroke = getCanvasSelectionStroke()
  const lines: ReactNode[] = []
  scene.primitives.forEach((primitive, index) => {
    if (!('Polyline' in primitive)) {
      return
    }
    const p = primitive.Polyline
    if (p.edge_id !== edgeId || p.points.length < 2) {
      return
    }
    lines.push(
      <Line
        key={primitiveKey('selection-wire', index, p)}
        points={polylineToKonvaPoints(p.points)}
        stroke={stroke}
        {...SCHEMATIC_STROKE_PROPS}
        closed={p.closed ?? false}
      />,
    )
  })
  return <>{lines}</>
}

/** Accent outline for the current selection (nodes, ports, wires). */
export function SelectionOverlay({
  scene,
  selectedHit,
  nodeDrag,
  diagramNodes = [],
}: SelectionOverlayProps) {
  if (!selectedHit) {
    return null
  }

  if (selectedHit.edge_id) {
    return <WireSelectionHighlight scene={scene} edgeId={selectedHit.edge_id} />
  }

  if (selectedHit.handle_id && selectedHit.node_id) {
    const { bounds } = selectedHit
    return (
      <NodeSelectionOutline
        bounds={bounds}
        offsetX={0}
        offsetY={0}
      />
    )
  }

  const nodeId = selectedHit.node_id
  if (!nodeId) {
    return null
  }

  if (isGroupingZoneSelectionTarget(scene.hits, nodeId)) {
    const zoneNode =
      findGroupingZoneNode(nodeId, diagramNodes) ??
      (() => {
        const bounds = nodeSelectionBounds(scene.hits, nodeId)
        if (!bounds) return undefined
        return {
          id: nodeId,
          type: 'groupingZone',
          position: { x: bounds.x, y: bounds.y },
          width: bounds.width,
          height: bounds.height,
          data: {},
        } satisfies FlowNode
      })()
    if (zoneNode) {
      return <GroupingZoneSelectionOutline node={zoneNode} />
    }
  }

  const bounds = nodeSelectionBounds(scene.hits, nodeId)
  if (!bounds) {
    return null
  }

  const offset = selectionOffset(scene.hits, nodeId, nodeDrag)
  return (
    <NodeSelectionOutline
      bounds={bounds}
      offsetX={offset.x}
      offsetY={offset.y}
    />
  )
}
