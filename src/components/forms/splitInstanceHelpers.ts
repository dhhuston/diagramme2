import type { FlowNode } from '../../tauriIpc'

const LINE1_TYPES = new Set([
  'micBlock',
  'speakerBlock',
  'antennaTransmitterSymbol',
  'antennaReceiverSymbol',
])

/** Each volumeControl is identified by its own node id — VCs have no user-facing tag. */
const NODE_ID_TYPES = new Set(['volumeControl'])

const EXCLUDED_TYPES = new Set([
  'groupingZone',
  'wiretag',
  'flyoffNote',
  'textBlock',
  'wireSplit',
])

/**
 * Returns a stable key for tag-matching, or null if this node type is excluded.
 *
 * Key format is prefixed to avoid collisions between node families:
 *   line1:<text>  — mic, speaker, antenna (identified by label)
 *   id:<nodeId>   — volumeControl (no user tag; each VC is its own identity)
 *   tag:<code>|<num> — all other equipment
 */
export function getTagKey(node: FlowNode): string | null {
  if (!node.type || EXCLUDED_TYPES.has(node.type)) return null
  const d = node.data as Record<string, unknown>
  if (LINE1_TYPES.has(node.type)) {
    const line1 = typeof d.line1 === 'string' ? d.line1.trim() : ''
    return line1 ? `line1:${line1}` : null
  }
  if (NODE_ID_TYPES.has(node.type)) {
    return `id:${node.id}`
  }
  const code = typeof d.tagCode === 'string' ? d.tagCode.trim() : ''
  const num = typeof d.tagNumber === 'string' ? d.tagNumber.trim() : ''
  if (!code && !num) return null
  return `tag:${code}|${num}`
}

/**
 * Returns the set of splitInstance numbers already in use by OTHER nodes
 * that share the same tag as `subject`. Excludes `subject` itself.
 */
export function findUsedInstanceNumbers(subject: FlowNode, allNodes: FlowNode[]): Set<number> {
  const subjectKey = getTagKey(subject)
  if (!subjectKey) return new Set()
  const used = new Set<number>()
  for (const node of allNodes) {
    if (node.id === subject.id) continue
    if (getTagKey(node) !== subjectKey) continue
    const d = node.data as Record<string, unknown>
    if (typeof d.splitInstance === 'number') {
      used.add(d.splitInstance)
    }
  }
  return used
}

/** Returns the lowest positive integer not present in `used`. */
export function nextAvailableInstance(used: Set<number>): number {
  let n = 1
  while (used.has(n)) n += 1
  return n
}
