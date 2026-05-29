import { useCallback, useState } from 'react'
import type { KonvaEventObject } from 'konva/lib/Node'

export type Viewport = {
  scale: number
  x: number
  y: number
}

export function useViewport(initial: Viewport = { scale: 1, x: 0, y: 0 }) {
  const [viewport, setViewport] = useState<Viewport>(initial)

  const setFit = useCallback((next: Viewport) => {
    setViewport(next)
  }, [])

  const onWheel = useCallback((event: KonvaEventObject<WheelEvent>) => {
    event.evt.preventDefault()
    const stage = event.target.getStage()
    if (!stage) return

    setViewport((prev) => {
      const oldScale = prev.scale
      const pointer = stage.getPointerPosition()
      if (!pointer) return prev

      const scaleBy = 1.08
      const direction = event.evt.deltaY > 0 ? -1 : 1
      const nextScale = direction > 0 ? oldScale * scaleBy : oldScale / scaleBy
      const clampedScale = Math.min(Math.max(nextScale, 0.05), 8)

      const mousePointTo = {
        x: (pointer.x - prev.x) / oldScale,
        y: (pointer.y - prev.y) / oldScale,
      }

      return {
        scale: clampedScale,
        x: pointer.x - mousePointTo.x * clampedScale,
        y: pointer.y - mousePointTo.y * clampedScale,
      }
    })
  }, [])

  const onDragEnd = useCallback((event: KonvaEventObject<DragEvent>) => {
    setViewport((prev) => ({
      ...prev,
      x: event.target.x(),
      y: event.target.y(),
    }))
  }, [])

  return { viewport, setFit, onWheel, onDragEnd }
}
