import type { FlowNode } from '../tauriIpc'

export const GROUPING_ZONE_DEFAULT_W = 240
export const GROUPING_ZONE_DEFAULT_H = 180
export const GROUPING_ZONE_MIN_W = 80
export const GROUPING_ZONE_MIN_H = 60

export type GroupingZoneShape = 'rect' | 'polyline'

export interface GroupingZoneData {
  label?: string
  shape?: GroupingZoneShape
  polylinePoints?: number[]
}

export function isGroupingZoneNode(node: FlowNode | undefined): node is FlowNode {
  return node?.type === 'groupingZone'
}

export function groupingZoneData(node: FlowNode): GroupingZoneData {
  return (node.data ?? {}) as GroupingZoneData
}

export function groupingZoneShape(node: FlowNode): GroupingZoneShape {
  return groupingZoneData(node).shape === 'polyline' ? 'polyline' : 'rect'
}

export function groupingZoneDimensions(node: FlowNode) {
  return {
    x: node.position.x,
    y: node.position.y,
    width: node.width ?? GROUPING_ZONE_DEFAULT_W,
    height: node.height ?? GROUPING_ZONE_DEFAULT_H,
  }
}
