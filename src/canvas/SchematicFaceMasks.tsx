import type { ReactNode } from 'react'
import { Rect } from 'react-konva'

import { getCanvasSchematicFaceColor } from './canvasTokens'
import { faceMaskRects } from './schematicFaceMaskUtils'
import type { HitTarget } from './sceneTypes'

/** Opaque schematic face so the diagram grid does not show through nodes. */
export function SchematicFaceMasks({ hits }: { hits: HitTarget[] }) {
  const fill = getCanvasSchematicFaceColor()
  const masks = faceMaskRects(hits)
  const nodes: ReactNode[] = masks.map(({ nodeId, bounds }) => {
    const { x, y, width, height } = bounds
    return (
      <Rect
        key={`face-${nodeId}`}
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
