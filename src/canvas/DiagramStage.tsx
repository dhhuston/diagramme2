import { useEffect, useRef, useState } from 'react'
import { Layer, Stage } from 'react-konva'
import type { KonvaEventObject } from 'konva/lib/Node'

import { hitTestScene, stagePointerToDiagramPx } from './hitTest'
import { fitExtentToStage } from './sceneRenderUtils'
import { SceneRenderer } from './SceneRenderer'
import type { HitTarget, SceneJson } from './sceneTypes'
import { useViewport } from './useViewport'

type DiagramStageProps = {
  scene: SceneJson
  onHit?: (hit: HitTarget | null) => void
}

/** Konva stage: 1 diagram px = 1 unit at scale 1; wheel zoom + drag pan. */
export function DiagramStage({ scene, onHit }: DiagramStageProps) {
  const hostRef = useRef<HTMLDivElement>(null)
  const [size, setSize] = useState({ width: 800, height: 600 })
  const { viewport, setFit, onWheel, onDragMove, onDragEnd } = useViewport()
  const dragMoved = useRef(false)

  useEffect(() => {
    const el = hostRef.current
    if (!el) return

    const observer = new ResizeObserver((entries) => {
      const entry = entries[0]
      if (!entry) return
      const { width, height } = entry.contentRect
      setSize({ width: Math.max(width, 1), height: Math.max(height, 1) })
    })
    observer.observe(el)
    return () => observer.disconnect()
  }, [])

  useEffect(() => {
    setFit(fitExtentToStage(scene.extent, size.width, size.height))
  }, [scene.extent, size.width, size.height, setFit])

  const handleLayerDragStart = () => {
    dragMoved.current = false
  }

  const handleLayerDragMove = (event: KonvaEventObject<DragEvent>) => {
    dragMoved.current = true
    onDragMove(event)
  }

  const handleLayerDragEnd = (event: KonvaEventObject<DragEvent>) => {
    onDragEnd(event)
    window.setTimeout(() => {
      dragMoved.current = false
    }, 0)
  }

  const handleStageClick = (event: KonvaEventObject<MouseEvent>) => {
    if (dragMoved.current || !onHit) return
    const stage = event.target.getStage()
    const pointer = stage?.getPointerPosition()
    if (!pointer) return
    const diagramPoint = stagePointerToDiagramPx(pointer, viewport)
    onHit(hitTestScene(scene.hits, diagramPoint))
  }

  return (
    <div ref={hostRef} className="diagram-stage-host">
      <Stage width={size.width} height={size.height} onWheel={onWheel} onClick={handleStageClick}>
        <Layer
          x={viewport.x}
          y={viewport.y}
          scaleX={viewport.scale}
          scaleY={viewport.scale}
          draggable
          onDragStart={handleLayerDragStart}
          onDragMove={handleLayerDragMove}
          onDragEnd={handleLayerDragEnd}
        >
          <SceneRenderer scene={scene} />
        </Layer>
      </Stage>
    </div>
  )
}
