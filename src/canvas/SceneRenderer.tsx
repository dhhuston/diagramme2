import { Group, Line, Rect, Shape } from 'react-konva'

import { dragOffset, nodeBodyBounds, type NodeDragTarget } from './interaction/dragNode'
import { SceneTextNode } from './SceneTextNode'
import {
  colorRgbToCss,
  konvaStrokeWidthPx,
  polylineToKonvaPoints,
  primitiveKey,
  solidLayerFillCss,
} from './sceneRenderUtils'
import type { HitTarget, SceneJson, ScenePrimitive } from './sceneTypes'

type SceneRendererProps = {
  scene: SceneJson
  /** Pointer target while dragging — used for a dashed outline only, never geometry clone. */
  nodeDrag?: NodeDragTarget | null
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

function dragTargetOutline(
  hits: HitTarget[],
  nodeDrag: NodeDragTarget,
): { x: number; y: number; width: number; height: number } | null {
  const body = nodeBodyBounds(hits, nodeDrag.nodeId)
  if (!body) {
    return null
  }
  return {
    x: nodeDrag.targetOrigin.x,
    y: nodeDrag.targetOrigin.y,
    width: body.width,
    height: body.height,
  }
}

/** Renders authoritative Rust scene primitives in diagram px (Y-down). */
export function SceneRenderer({ scene, nodeDrag }: SceneRendererProps) {
  const outline = nodeDrag ? dragTargetOutline(scene.hits, nodeDrag) : null
  const localIndices = new Set(nodeDrag?.localPrimitiveIndices ?? [])
  const offset = nodeDrag != null ? dragOffset(nodeDrag.startOrigin, nodeDrag.targetOrigin) : null
  const hasOffset =
    offset != null && (Math.abs(offset.x) >= 0.01 || Math.abs(offset.y) >= 0.01)

  return (
    <>
      {scene.primitives.map((primitive, index) => {
        const el = renderPrimitive(primitive, index)
        if (hasOffset && localIndices.has(index)) {
          return (
            <Group key={`local-${index}`} x={offset.x} y={offset.y} listening={false}>
              {el}
            </Group>
          )
        }
        return el
      })}
      {outline ? (
        <Rect
          key="drag-target-outline"
          x={outline.x}
          y={outline.y}
          width={outline.width}
          height={outline.height}
          stroke="rgba(30, 100, 220, 0.75)"
          strokeWidth={1}
          dash={[6, 4]}
          listening={false}
          perfectDrawEnabled={false}
        />
      ) : null}
    </>
  )
}
