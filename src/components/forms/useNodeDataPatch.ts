import { useCallback } from 'react'

export function useNodeDataPatch<TData>(
  nodeId: string,
  data: TData,
  onUpdate: (nodeId: string, data: TData) => void,
) {
  return useCallback(
    (patch: Partial<TData>) => onUpdate(nodeId, { ...data, ...patch }),
    [data, nodeId, onUpdate],
  )
}
