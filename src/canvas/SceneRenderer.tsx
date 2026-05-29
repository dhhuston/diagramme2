import { Line, Rect, Shape, Text } from 'react-konva'

import {
  colorRgbToCss,
  konvaTextAlign,
  konvaVerticalAlign,
  polylineToKonvaPoints,
  primitiveKey,
} from './sceneRenderUtils'
import type { SceneJson, ScenePrimitive } from './sceneTypes'

type SceneRendererProps = {
  scene: SceneJson
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
        strokeWidth={p.stroke_px}
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
        strokeWidth={r.stroke_px}
        fill={fill}
        listening={false}
        perfectDrawEnabled={false}
      />
    )
  }

  if ('Solid' in primitive) {
    const s = primitive.Solid
    const [v0, v1, v2, v3] = s.vertices
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
          ctx.fillStrokeShape(shape)
        }}
        fill="#000000"
        stroke="#000000"
        strokeWidth={1}
      />
    )
  }

  const t = primitive.Text
  return (
    <Text
      key={primitiveKey('text', index)}
      x={t.position.x}
      y={t.position.y}
      text={t.content}
      fontFamily={t.font}
      fontSize={t.height_px}
      align={konvaTextAlign(t.halign)}
      verticalAlign={konvaVerticalAlign(t.valign)}
      fill="#000000"
      listening={false}
      perfectDrawEnabled={false}
    />
  )
}

/** Renders authoritative Rust scene primitives in diagram px (Y-down). */
export function SceneRenderer({ scene }: SceneRendererProps) {
  return <>{scene.primitives.map((primitive, index) => renderPrimitive(primitive, index))}</>
}
