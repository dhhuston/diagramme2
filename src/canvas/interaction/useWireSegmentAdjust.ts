import { useCallback, useEffect, useRef, useState } from 'react'

import type { HitTarget, PointPx } from '../sceneTypes'
import { isWireGripHit } from '../wireGripUtils'

export type WireSegmentArm = {
  edgeId: string
  segmentIndex: number
  orientation: 'h' | 'v'
  startFlow: PointPx
  /** Frozen inner chain at arm time (v6 `chain0`). */
  chain0: PointPx[]
  priorCorners: PointPx[] | undefined
  gripId: string
}

export type WireSegmentAdjustHandlers = {
  onWireSegmentPreview?: (arm: WireSegmentArm, delta: PointPx) => void | Promise<void>
  onWireSegmentCommit?: (arm: WireSegmentArm, delta: PointPx) => void | Promise<void>
  onWireSegmentCancel?: (arm: WireSegmentArm) => void | Promise<void>
  readEdgeInnerCorners?: (edgeId: string) => Promise<PointPx[] | undefined>
  readWireInnerChain?: (edgeId: string) => Promise<PointPx[] | undefined>
}

const WIRE_SEGMENT_PREVIEW_MS = 60

export function useWireSegmentAdjust(handlers: WireSegmentAdjustHandlers) {
  const armRef = useRef<WireSegmentArm | null>(null)
  /** After commit via pointerdown, ignore the same gesture re-arming on the grip. */
  const suppressNextGripArmRef = useRef(false)
  const [activeGripId, setActiveGripId] = useState<string | null>(null)
  const previewTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  const previewFrame = useRef<number | null>(null)
  const pendingDelta = useRef<PointPx | null>(null)
  const handlersRef = useRef(handlers)
  handlersRef.current = handlers

  const cancelPreviewTimer = useCallback(() => {
    if (previewTimer.current != null) {
      clearTimeout(previewTimer.current)
      previewTimer.current = null
    }
    if (previewFrame.current != null) {
      cancelAnimationFrame(previewFrame.current)
      previewFrame.current = null
    }
  }, [])

  const disarm = useCallback(() => {
    cancelPreviewTimer()
    armRef.current = null
    pendingDelta.current = null
    setActiveGripId(null)
  }, [cancelPreviewTimer])

  const flushPreview = useCallback(async () => {
    const arm = armRef.current
    const delta = pendingDelta.current
    if (!arm || !delta) return
    await handlersRef.current.onWireSegmentPreview?.(arm, delta)
  }, [])

  const schedulePreview = useCallback(
    (delta: PointPx) => {
      pendingDelta.current = delta
      if (previewTimer.current != null || previewFrame.current != null) return
      previewTimer.current = setTimeout(() => {
        previewTimer.current = null
        previewFrame.current = requestAnimationFrame(() => {
          previewFrame.current = null
          void flushPreview()
        })
      }, WIRE_SEGMENT_PREVIEW_MS)
    },
    [flushPreview],
  )

  const cancelSegmentAdjust = useCallback(async () => {
    const arm = armRef.current
    if (!arm) return
    disarm()
    await handlersRef.current.onWireSegmentCancel?.(arm)
  }, [disarm])

  const commitSegmentAdjust = useCallback(async () => {
    cancelPreviewTimer()
    const arm = armRef.current
    const delta = pendingDelta.current ?? { x: 0, y: 0 }
    if (!arm) return
    suppressNextGripArmRef.current = true
    disarm()
    await handlersRef.current.onWireSegmentCommit?.(arm, delta)
  }, [cancelPreviewTimer, disarm])

  const armFromGripHit = useCallback(async (hit: HitTarget, startFlow: PointPx) => {
    if (!isWireGripHit(hit)) return false
    if (armRef.current) return false
    const [priorCorners, chain0] = await Promise.all([
      handlersRef.current.readEdgeInnerCorners
        ? handlersRef.current.readEdgeInnerCorners(hit.edge_id)
        : Promise.resolve(undefined),
      handlersRef.current.readWireInnerChain
        ? handlersRef.current.readWireInnerChain(hit.edge_id)
        : Promise.resolve(undefined),
    ])
    if (!chain0 || chain0.length < 2) return false
    const arm: WireSegmentArm = {
      edgeId: hit.edge_id,
      segmentIndex: hit.wire_grip_segment,
      orientation: hit.wire_grip_orientation,
      startFlow,
      chain0,
      priorCorners,
      gripId: hit.id,
    }
    armRef.current = arm
    pendingDelta.current = { x: 0, y: 0 }
    setActiveGripId(hit.id)
    return true
  }, [])

  const onPointerMoveWhileArmed = useCallback(
    (diagramPoint: PointPx) => {
      const arm = armRef.current
      if (!arm) return false
      const delta =
        arm.orientation === 'h'
          ? { x: 0, y: diagramPoint.y - arm.startFlow.y }
          : { x: diagramPoint.x - arm.startFlow.x, y: 0 }
      schedulePreview(delta)
      return true
    },
    [schedulePreview],
  )

  const tryArmOnGripPointerDown = useCallback(
    async (hit: HitTarget | null, diagramPoint: PointPx) => {
      if (suppressNextGripArmRef.current) {
        suppressNextGripArmRef.current = false
        return false
      }
      if (!isWireGripHit(hit)) return false
      return armFromGripHit(hit, diagramPoint)
    },
    [armFromGripHit],
  )

  const isArmed = useCallback(() => armRef.current != null, [])

  useEffect(() => {
    if (!activeGripId) return

    const onKey = (event: KeyboardEvent) => {
      if (event.key !== 'Escape') return
      event.preventDefault()
      void cancelSegmentAdjust()
    }
    const onPointerDown = (event: PointerEvent) => {
      if (event.button !== 0) return
      event.preventDefault()
      event.stopPropagation()
      void commitSegmentAdjust()
    }

    window.addEventListener('keydown', onKey, true)
    window.addEventListener('pointerdown', onPointerDown, true)
    return () => {
      window.removeEventListener('keydown', onKey, true)
      window.removeEventListener('pointerdown', onPointerDown, true)
    }
  }, [activeGripId, cancelSegmentAdjust, commitSegmentAdjust])

  return {
    activeGripId,
    isArmed,
    tryArmOnGripPointerDown,
    onPointerMoveWhileArmed,
    cancelSegmentAdjust,
    commitSegmentAdjust,
    disarm,
  }
}
