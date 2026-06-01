/**
 * Wiretag pair: resolve partner connection and equipment display tags.
 */
import type { FlowEdge, FlowNode } from './tauriIpc'
import type { NodeMakeModelFields } from './nodeMakeModel'

export type WiretagEnd = 'a' | 'b'

export type WiretagNodeData = NodeMakeModelFields & {
  pairId: string
  pairIndex: number
  end: WiretagEnd
  tagDescription?: string
  sheetName?: string
  showSheetName?: boolean
}

export const WIRETAG_CONN_SRC = 'conn-src'
export const WIRETAG_CONN_TGT = 'conn-tgt'

const CONN_HANDLES = new Set([WIRETAG_CONN_SRC, WIRETAG_CONN_TGT])

function isConnHandle(id: string | null | undefined): boolean {
  return id != null && CONN_HANDLES.has(id)
}

export function findWiretagAttachedNodeId(wiretagNodeId: string, edges: FlowEdge[]): string | null {
  for (const e of edges) {
    if (e.source === wiretagNodeId && isConnHandle(e.sourceHandle)) {
      if (e.target !== wiretagNodeId) return e.target
    }
    if (e.target === wiretagNodeId && isConnHandle(e.targetHandle)) {
      if (e.source !== wiretagNodeId) return e.source
    }
  }
  return null
}

export function findWiretagPartnerNode(
  selfId: string,
  pairId: string,
  nodes: FlowNode[],
): FlowNode | null {
  return (
    nodes.find(
      (n) => n.type === 'wiretag' && n.id !== selfId && (n.data as WiretagNodeData).pairId === pairId,
    ) ?? null
  )
}

export function getDeviceTagLabel(node: FlowNode): string {
  const d = node.data as Record<string, unknown>
  const code = typeof d.tagCode === 'string' ? d.tagCode.trim() : ''
  const num = typeof d.tagNumber === 'string' ? d.tagNumber.trim() : ''
  if (code || num) {
    return code && num ? `${code} / ${num}` : `${code}${num}`.trim()
  }
  switch (node.type) {
    case 'micBlock':
    case 'speakerBlock':
      return String(d.line1 ?? '').trim() || node.id
    case 'volumeControl':
      return 'VC'
    case 'antennaTransmitterSymbol':
    case 'antennaReceiverSymbol':
      return String(d.line1 ?? '').trim() || 'ANT'
    default:
      return node.id
  }
}

export function resolveRemoteTagForWiretag(
  self: FlowNode,
  nodes: FlowNode[],
  edges: FlowEdge[],
): string {
  if (self.type !== 'wiretag') return ''
  const { pairId } = self.data as WiretagNodeData
  const partner = findWiretagPartnerNode(self.id, pairId, nodes)
  if (!partner) return ''
  const attachId = findWiretagAttachedNodeId(partner.id, edges)
  if (!attachId) return ''
  const attach = nodes.find((n) => n.id === attachId)
  if (!attach) return ''
  return getDeviceTagLabel(attach)
}

export function pairEquipmentSummaryText(
  self: FlowNode,
  nodes: FlowNode[],
  edges: FlowEdge[],
): string {
  if (self.type !== 'wiretag') return ''
  const { pairId } = self.data as WiretagNodeData
  const partner = findWiretagPartnerNode(self.id, pairId, nodes)
  const idA = findWiretagAttachedNodeId(self.id, edges)
  const idB = partner ? findWiretagAttachedNodeId(partner.id, edges) : null
  const labels: string[] = []
  for (const attachId of [idA, idB]) {
    if (!attachId) continue
    const n = nodes.find((x) => x.id === attachId)
    if (n) labels.push(getDeviceTagLabel(n))
  }
  const uniq = [...new Set(labels.map((s) => s.trim()).filter(Boolean))]
  uniq.sort()
  return uniq.join(' · ')
}

export function nextWiretagPairIndex(nodes: FlowNode[]): number {
  let max = 0
  for (const n of nodes) {
    if (n.type !== 'wiretag') continue
    const idx = Number((n.data as WiretagNodeData).pairIndex)
    if (Number.isFinite(idx) && idx > max) max = idx
  }
  return max + 1
}

export function wiretagIdsForPair(pairId: string, nodes: FlowNode[]): string[] {
  return nodes
    .filter((n) => n.type === 'wiretag' && (n.data as WiretagNodeData).pairId === pairId)
    .map((n) => n.id)
}

export function wiretagPairIndexTaken(
  pairIndex: number,
  excludePairId: string,
  nodes: FlowNode[],
): boolean {
  for (const n of nodes) {
    if (n.type !== 'wiretag') continue
    const d = n.data as WiretagNodeData
    if (d.pairId === excludePairId) continue
    if (Number(d.pairIndex) === pairIndex) return true
  }
  return false
}
