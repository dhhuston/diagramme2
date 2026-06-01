import { Circle } from 'react-konva'

import type { HitTarget } from './sceneTypes'
import { WIRE_GRIP_RADIUS_PX, wireGripHits } from './wireGripUtils'

type WireRoutingGripsProps = {
  hits: HitTarget[]
  /** Only grips for the selected wire are shown (v6 `resolvedSelected`). */
  selectedEdgeId?: string | null
  /** Highlight the grip being adjusted (click-move-click). */
  activeGripId?: string | null
}

/** Mid-segment routing grips (v6 `schematic-edge__handle`). Interaction via scene hit targets. */
export function WireRoutingGrips({
  hits,
  selectedEdgeId = null,
  activeGripId = null,
}: WireRoutingGripsProps) {
  const grips = wireGripHits(hits, selectedEdgeId)
  return (
    <>
      {grips.map((hit) => {
        const cx = hit.bounds.x + hit.bounds.width / 2
        const cy = hit.bounds.y + hit.bounds.height / 2
        const active = hit.id === activeGripId
        return (
          <Circle
            key={hit.id}
            x={cx}
            y={cy}
            radius={WIRE_GRIP_RADIUS_PX}
            fill={active ? 'var(--clr-accent, #2a7)' : '#494f4b'}
            stroke="#ffffff"
            strokeWidth={1}
            listening={false}
            perfectDrawEnabled={false}
          />
        )
      })}
    </>
  )
}
