import { useLayoutEffect, useMemo, useState } from 'react'
import { Shape } from 'react-konva'

import {
  getCanvasGridColor,
  getCanvasGridMajorColor,
  invalidateCanvasTokenCache,
} from './canvasTokens'
import { gridLinePositions, visibleDiagramBounds } from './diagramGridUtils'
import {
  CONNECTOR_LINE_PITCH_PX,
  isOnGridStep,
  PX_PER_INCH,
  SNAP_HALF_STEP_PX,
} from './paperScale'
import type { RectPx } from './sceneTypes'
import type { Viewport } from './useViewport'

type DiagramGridProps = {
  extent: RectPx
  viewport: Viewport
  stageWidth: number
  stageHeight: number
}

/** ~1 CSS pixel hairline in diagram space at the current zoom. */
function hairlineDiagramPx(scale: number): number {
  if (!Number.isFinite(scale) || scale <= 0) return 0.5
  return Math.min(1.25, Math.max(0.35, 1 / scale))
}

type GridTier = 'minor' | 'connector' | 'inch'

function tierForPosition(p: number): GridTier {
  if (isOnGridStep(p, PX_PER_INCH)) return 'inch'
  if (isOnGridStep(p, CONNECTOR_LINE_PITCH_PX)) return 'connector'
  return 'minor'
}

export function DiagramGrid({
  extent,
  viewport,
  stageWidth,
  stageHeight,
}: DiagramGridProps) {
  const [gridColors, setGridColors] = useState(() => ({
    minor: getCanvasGridColor(),
    major: getCanvasGridMajorColor(),
  }))

  useLayoutEffect(() => {
    invalidateCanvasTokenCache()
    setGridColors({
      minor: getCanvasGridColor(),
      major: getCanvasGridMajorColor(),
    })
  }, [])

  const bounds = useMemo(() => {
    const visible = visibleDiagramBounds(viewport, stageWidth, stageHeight)
    const x0 = Math.min(extent.x, visible.x)
    const y0 = Math.min(extent.y, visible.y)
    const x1 = Math.max(extent.x + extent.width, visible.x + visible.width)
    const y1 = Math.max(extent.y + extent.height, visible.y + visible.height)
    return { x0, y0, x1, y1 }
  }, [extent, stageHeight, stageWidth, viewport])

  const { xLines, yLines } = useMemo(
    () => ({
      xLines: gridLinePositions(bounds.x0, bounds.x1, SNAP_HALF_STEP_PX),
      yLines: gridLinePositions(bounds.y0, bounds.y1, SNAP_HALF_STEP_PX),
    }),
    [bounds.x0, bounds.x1, bounds.y0, bounds.y1],
  )

  const lineWidth = hairlineDiagramPx(viewport.scale)

  return (
    <Shape
      listening={false}
      perfectDrawEnabled={false}
      sceneFunc={(ctx) => {
        const drawTier = (positions: number[], vertical: boolean, tier: GridTier) => {
          const alpha = tier === 'inch' ? 0.5 : tier === 'connector' ? 0.42 : 0.22
          const widthMul = tier === 'inch' ? 1.35 : tier === 'connector' ? 1.15 : 1
          ctx.beginPath()
          ctx.strokeStyle = tier === 'minor' ? gridColors.minor : gridColors.major
          ctx.lineWidth = lineWidth * widthMul
          ctx.globalAlpha = alpha
          for (const p of positions) {
            if (tierForPosition(p) !== tier) continue
            if (vertical) {
              ctx.moveTo(p, bounds.y0)
              ctx.lineTo(p, bounds.y1)
            } else {
              ctx.moveTo(bounds.x0, p)
              ctx.lineTo(bounds.x1, p)
            }
          }
          ctx.stroke()
        }

        for (const tier of ['minor', 'connector', 'inch'] as const) {
          drawTier(xLines, true, tier)
          drawTier(yLines, false, tier)
        }
        ctx.globalAlpha = 1
      }}
    />
  )
}
