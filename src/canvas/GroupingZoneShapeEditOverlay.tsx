import { Circle, Group, Line, Rect } from 'react-konva'

import { getCanvasSelectionStroke } from './canvasTokens'
import { getClosedPolylineSegments, type Pt } from './groupingZoneGeometry'
import { groupingZoneDimensions, groupingZoneShape } from './groupingZoneNode'
import {
  RECT_RESIZE_HANDLES,
  rectResizeHandlePosition,
  type DiagramRect,
} from './groupingZoneRectResize'
import type { FlowNode } from '../tauriIpc'

const HANDLE_SIZE = 8

type GroupingZoneShapeEditOverlayProps = {
  node: FlowNode
  liveRect: DiagramRect | null
  polyPairs: Pt[] | null
}

/** Edit chrome for grouping zones (v6 NodeResizer + polyline vertex dots). Interaction via scene-level pointer routing. */
export function GroupingZoneShapeEditOverlay({
  node,
  liveRect,
  polyPairs,
}: GroupingZoneShapeEditOverlayProps) {
  const stroke = getCanvasSelectionStroke()
  const shape = groupingZoneShape(node)
  const dims = groupingZoneDimensions(node)

  if (shape === 'rect') {
    const rect = liveRect ?? dims
    return (
      <Group listening={false}>
        <Rect
          x={rect.x}
          y={rect.y}
          width={rect.width}
          height={rect.height}
          stroke={stroke}
          strokeWidth={1.5}
          listening={false}
        />
        {RECT_RESIZE_HANDLES.map((handle) => {
          const pos = rectResizeHandlePosition(rect, handle)
          return (
            <Rect
              key={handle}
              x={pos.x - HANDLE_SIZE / 2}
              y={pos.y - HANDLE_SIZE / 2}
              width={HANDLE_SIZE}
              height={HANDLE_SIZE}
              fill="#ffffff"
              stroke={stroke}
              strokeWidth={1.5}
              cornerRadius={2}
              listening={false}
            />
          )
        })}
      </Group>
    )
  }

  if (!polyPairs || polyPairs.length < 2) {
    return null
  }

  const flatPoints = polyPairs.flatMap(([x, y]) => [x + dims.x, y + dims.y])

  return (
    <Group listening={false}>
      <Line points={flatPoints} closed stroke={stroke} strokeWidth={1.5} listening={false} />
      {getClosedPolylineSegments(polyPairs).map(({ ax, ay, bx, by, i }) => (
        <Line
          key={`seg-${i}`}
          points={[ax + dims.x, ay + dims.y, bx + dims.x, by + dims.y]}
          stroke={stroke}
          strokeWidth={1}
          opacity={0.35}
          listening={false}
        />
      ))}
      {polyPairs.map(([vx, vy], i) => (
        <Circle
          key={`v-${i}`}
          x={vx + dims.x}
          y={vy + dims.y}
          radius={6}
          fill="#ffffff"
          stroke={stroke}
          strokeWidth={1.5}
          listening={false}
          perfectDrawEnabled={false}
        />
      ))}
    </Group>
  )
}
