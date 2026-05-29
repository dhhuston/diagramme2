import { Line, Rect, Shape } from 'react-konva'

import {
  belongsToNodeDrag,
  offsetScenePrimitive,
  type NodeDragPreview,
} from './interaction/dragNode'
import { SceneTextNode } from './SceneTextNode'
import {
  colorRgbToCss,
  konvaStrokeWidthPx,
  polylineToKonvaPoints,
  primitiveKey,
  solidLayerFillCss,
} from './sceneRenderUtils'
import type { SceneJson, ScenePrimitive } from './sceneTypes'

type SceneRendererProps = {
  scene: SceneJson
  nodeDrag?: NodeDragPreview | null
}

function renderPrimitive(primitive: ScenePrimitive, index: number) {
  if ('Polyline' in primitive) {
    const p = primitive.Polyline
    const stroke = colorRgbToCss(p.color)
    return (
      <Line
        key={primitiveKey('polyline', index, p)}
        points={polylineToKonvaPoints(p.points)}
        stroke={stroke}
        strokeWidth={konvaStrokeWidthPx(p.stroke_px, p.edge_id)}
        closed={p.closed ?? false}
        lineJoin="miter"
        lineCap="square"
        listening={false}
        perfectDrawEnabled={false}
      />
    )
  }

  if ('Rect' in primitive) {
    const r = primitive.Rect
    const stroke = colorRgbToCss(0)
    const fill = r.fill != null ? colorRgbToCss(r.fill) : undefined
    return (
      <Rect
        key={primitiveKey('rect', index)}
        x={r.rect.x}
        y={r.rect.y}
        width={r.rect.width}
        height={r.rect.height}
        stroke={stroke}
        strokeWidth={konvaStrokeWidthPx(r.stroke_px)}
        fill={fill}
        listening={false}
        perfectDrawEnabled={false}
      />
    )
  }

  if ('Solid' in primitive) {
    const s = primitive.Solid
    const [v0, v1, v2, v3] = s.vertices
    const fill = solidLayerFillCss(s.layer)
    return (
      <Shape
        key={primitiveKey('solid', index)}
        listening={false}
        perfectDrawEnabled={false}
        sceneFunc={(ctx, shape) => {
          ctx.beginPath()
          ctx.moveTo(v0.x, v0.y)
          ctx.lineTo(v1.x, v1.y)
          ctx.lineTo(v2.x, v2.y)
          ctx.lineTo(v3.x, v3.y)
          ctx.closePath()
          ctx.fillShape(shape)
        }}
        fill={fill}
      />
    )
  }

  const t = primitive.Text
  return <SceneTextNode key={primitiveKey('text', index)} text={t} />
}

function effectivePrimitive(
  primitive: ScenePrimitive,
  nodeDrag: NodeDragPreview | null | undefined,
): ScenePrimitive {
  if (!nodeDrag || (nodeDrag.dx === 0 && nodeDrag.dy === 0)) {
    return primitive
  }
  if (!belongsToNodeDrag(primitive, nodeDrag.nodeId, nodeDrag.captureBounds)) {
    return primitive
  }
  return offsetScenePrimitive(primitive, nodeDrag.dx, nodeDrag.dy)
}

/** Renders authoritative Rust scene primitives in diagram px (Y-down). */
export function SceneRenderer({ scene, nodeDrag }: SceneRendererProps) {
  return (
    <>
      {scene.primitives.map((primitive, index) =>
        renderPrimitive(effectivePrimitive(primitive, nodeDrag), index),
      )}
    </>
  )
}
