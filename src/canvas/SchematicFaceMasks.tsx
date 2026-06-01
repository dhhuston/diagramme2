import type { ReactNode } from 'react'
import { Rect, Shape } from 'react-konva'

import { getCanvasSchematicFaceColor } from './canvasTokens'
import { faceMaskShapes } from './schematicFaceMaskUtils'
import type { HitTarget } from './sceneTypes'

/** Opaque schematic face so the diagram grid does not show through nodes. */
export function SchematicFaceMasks({ hits }: { hits: HitTarget[] }) {
  const fill = getCanvasSchematicFaceColor()
  const masks = faceMaskShapes(hits)
  const nodes: ReactNode[] = masks.map((mask) => {
    if (mask.kind === 'polygon') {
      return (
        <Shape
          key={`face-${mask.nodeId}`}
          listening={false}
          perfectDrawEnabled={false}
          sceneFunc={(ctx, shape) => {
            ctx.beginPath()
            ctx.moveTo(mask.points[0].x, mask.points[0].y)
            for (let i = 1; i < mask.points.length; i++) {
              ctx.lineTo(mask.points[i].x, mask.points[i].y)
            }
            ctx.closePath()
            ctx.fillShape(shape)
          }}
          fill={fill}
        />
      )
    }
    const { x, y, width, height } = mask.bounds
    return (
      <Rect
        key={`face-${mask.nodeId}`}
        x={x}
        y={y}
        width={width}
        height={height}
        fill={fill}
        strokeWidth={0}
        listening={false}
        perfectDrawEnabled={false}
      />
    )
  })
  return <>{nodes}</>
}
