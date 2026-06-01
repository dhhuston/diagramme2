import { useCallback, useEffect, useRef } from 'react'

import { updateNode, type DiagramState } from '../tauriIpc'

const DEFAULT_DEBOUNCE_MS = 280

type PendingEntry = {
  timer: ReturnType<typeof setTimeout>
  data: unknown
}

/**
 * Debounced `update_node` IPC — mirrors v6 `scheduleRustNodeDataSync`.
 * Optimistic local updates happen in the caller; this flushes to Rust.
 */
export function useDebouncedNodeUpdate(
  onApplied: (result: DiagramState) => void | Promise<void>,
  debounceMs = DEFAULT_DEBOUNCE_MS,
) {
  const pendingRef = useRef<Map<string, PendingEntry>>(new Map())
  const onAppliedRef = useRef(onApplied)
  onAppliedRef.current = onApplied

  const flushNode = useCallback(async (nodeId: string) => {
    const entry = pendingRef.current.get(nodeId)
    if (!entry) return
    clearTimeout(entry.timer)
    pendingRef.current.delete(nodeId)
    const result = await updateNode(nodeId, entry.data)
    await onAppliedRef.current(result)
  }, [])

  const flushAll = useCallback(async () => {
    const ids = [...pendingRef.current.keys()]
    await Promise.all(ids.map((id) => flushNode(id)))
  }, [flushNode])

  const scheduleNodeDataSync = useCallback(
    (nodeId: string, data: unknown) => {
      const existing = pendingRef.current.get(nodeId)
      if (existing) clearTimeout(existing.timer)
      const timer = setTimeout(() => {
        void flushNode(nodeId)
      }, debounceMs)
      pendingRef.current.set(nodeId, { timer, data })
    },
    [debounceMs, flushNode],
  )

  useEffect(() => {
    return () => {
      for (const entry of pendingRef.current.values()) {
        clearTimeout(entry.timer)
      }
      pendingRef.current.clear()
    }
  }, [])

  return { scheduleNodeDataSync, flushAll, flushNode }
}
