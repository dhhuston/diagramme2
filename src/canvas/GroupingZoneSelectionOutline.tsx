import { Line, Rect } from 'react-konva'

import { getCanvasSelectionStroke } from './canvasTokens'
import {
  groupingZonePolylinePairs,
  isGroupingZoneNodeSelection,
} from './groupingZoneHitTest'
import { groupingZoneDimensions, groupingZoneShape } from './groupingZoneNode'
import { polylineToKonvaPoints } from './sceneRenderUtils'
import type { PointPx } from './sceneTypes'
import type { FlowNode } from '../tauriIpc'

const SELECTION_STROKE_PX = 2
const ZONE_BORDER_INSET = 0.25

type GroupingZoneSelectionOutlineProps = {
  node: FlowNode
}

/** Accent stroke on the dashed zone border only (v6 selected border). */
export function GroupingZoneSelectionOutline({ node }: GroupingZoneSelectionOutlineProps) {
  const stroke = getCanvasSelectionStroke()
  const dims = groupingZoneDimensions(node)

  if (groupingZoneShape(node) === 'polyline') {
    const pairs = groupingZonePolylinePairs(node)
    if (!pairs || pairs.length < 2) return null
    const points: PointPx[] = pairs.map(([x, y]) => ({ x: x + dims.x, y: y + dims.y }))
    return (
      <Line
        points={polylineToKonvaPoints(points)}
        closed
        stroke={stroke}
        strokeWidth={SELECTION_STROKE_PX}
        dash={[6, 4]}
        listening={false}
        perfectDrawEnabled={false}
      />
    )
  }

  return (
    <Rect
      x={dims.x + ZONE_BORDER_INSET}
      y={dims.y + ZONE_BORDER_INSET}
      width={Math.max(0, dims.width - ZONE_BORDER_INSET * 2)}
      height={Math.max(0, dims.height - ZONE_BORDER_INSET * 2)}
      stroke={stroke}
      strokeWidth={SELECTION_STROKE_PX}
      dash={[6, 4]}
      listening={false}
      perfectDrawEnabled={false}
    />
  )
}

export function findGroupingZoneNode(nodeId: string, nodes: FlowNode[]): FlowNode | undefined {
  return nodes.find((n) => n.id === nodeId && n.type === 'groupingZone')
}

export function isGroupingZoneSelectionTarget(
  hits: import('./sceneTypes').HitTarget[],
  nodeId: string,
): boolean {
  return isGroupingZoneNodeSelection(hits, nodeId)
}
