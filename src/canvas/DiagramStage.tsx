import { useEffect, useRef, useState } from 'react'
import { Layer, Stage } from 'react-konva'

import { useDiagramInteraction } from './interaction/useDiagramInteraction'
import { fitExtentToStage } from './sceneRenderUtils'
import { SceneRenderer } from './SceneRenderer'
import type { HitTarget, PointPx, SceneJson } from './sceneTypes'
import { useViewport } from './useViewport'

type DiagramStageProps = {
  scene: SceneJson
  onHit?: (hit: HitTarget | null) => void
  onNodeMove?: (nodeId: string, position: PointPx) => void | Promise<void>
}

/** Konva stage: 1 diagram px = 1 unit at scale 1; wheel zoom + drag pan. */
export function DiagramStage({ scene, onHit, onNodeMove }: DiagramStageProps) {
  const hostRef = useRef<HTMLDivElement>(null)
  const [size, setSize] = useState({ width: 800, height: 600 })
  const { viewport, setFit, setPan, onWheel } = useViewport()

  const {
    nodeDrag,
    handlePointerDown,
    handlePointerMove,
    handlePointerUp,
    handleStageClick,
  } = useDiagramInteraction({
    scene,
    viewport,
    onHit,
    onNodeMove,
    onPan: setPan,
  })

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

  return (
    <div ref={hostRef} className="diagram-stage-host">
      <Stage
        width={size.width}
        height={size.height}
        onWheel={onWheel}
        onPointerDown={handlePointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerUp}
        onClick={handleStageClick}
      >
        <Layer x={viewport.x} y={viewport.y} scaleX={viewport.scale} scaleY={viewport.scale}>
          <SceneRenderer scene={scene} nodeDrag={nodeDrag} />
        </Layer>
      </Stage>
    </div>
  )
}
