import { Line } from 'react-konva'

import type { WireConnectPreview } from './interaction/useDiagramInteraction'
import { colorRgbToCss, polylineToKonvaPoints, SCHEMATIC_STROKE_PROPS } from './sceneRenderUtils'

type WireConnectOverlayProps = {
  preview: WireConnectPreview | null
}

/** Rubber-band wire while dragging from port to port. */
export function WireConnectOverlay({ preview }: WireConnectOverlayProps) {
  if (!preview) {
    return null
  }
  const points = polylineToKonvaPoints([preview.fromPoint, preview.toPoint])
  return (
    <Line
      points={points}
      stroke={colorRgbToCss(0x494f4b)}
      {...SCHEMATIC_STROKE_PROPS}
      dash={[6, 4]}
    />
  )
}
