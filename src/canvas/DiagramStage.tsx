import { useEffect, useRef, useState } from 'react'
import { Layer, Stage } from 'react-konva'

import { fitExtentToStage } from './sceneRenderUtils'
import { SceneRenderer } from './SceneRenderer'
import type { SceneJson } from './sceneTypes'
import { useViewport } from './useViewport'

type DiagramStageProps = {
  scene: SceneJson
}

/** Konva stage: 1 diagram px = 1 unit at scale 1; wheel zoom + drag pan. */
export function DiagramStage({ scene }: DiagramStageProps) {
  const hostRef = useRef<HTMLDivElement>(null)
  const [size, setSize] = useState({ width: 800, height: 600 })
  const { viewport, setFit, onWheel, onDragEnd } = useViewport()

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
      <Stage width={size.width} height={size.height} onWheel={onWheel}>
        <Layer
          x={viewport.x}
          y={viewport.y}
          scaleX={viewport.scale}
          scaleY={viewport.scale}
          draggable
          onDragMove={(event) => {
            setViewport((prev) => ({
              ...prev,
              x: event.target.x(),
              y: event.target.y(),
            }))
          }}
          onDragEnd={onDragEnd}
        >
          <SceneRenderer scene={scene} />
        </Layer>
      </Stage>
    </div>
  )
}
