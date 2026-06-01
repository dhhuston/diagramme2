import type { HitTarget, PointPx } from '../sceneTypes'

export type PortEndpoint = {
  nodeId: string
  handleId: string
}

export function portFromHit(hit: HitTarget | null): PortEndpoint | null {
  if (hit?.node_id == null || hit.handle_id == null) {
    return null
  }
  return { nodeId: hit.node_id, handleId: hit.handle_id }
}

export function portCenterFromHit(hit: HitTarget): PointPx {
  return {
    x: hit.bounds.x + hit.bounds.width / 2,
    y: hit.bounds.y + hit.bounds.height / 2,
  }
}

export function canConnectPorts(from: PortEndpoint, to: PortEndpoint): boolean {
  if (from.nodeId === to.nodeId && from.handleId === to.handleId) {
    return false
  }
  return true
}
