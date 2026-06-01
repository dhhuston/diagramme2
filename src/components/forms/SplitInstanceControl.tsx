import { memo, useMemo } from 'react'
import type { FlowNode } from '../../tauriIpc'
import { findUsedInstanceNumbers, nextAvailableInstance } from './splitInstanceHelpers'
import './SplitInstanceControl.css'

interface Props {
  /** The node being edited. */
  node: FlowNode
  /** Current splitInstance value from node data. */
  value: number | undefined
  /** All nodes across all sheets for cross-sheet scanning. */
  allNodes: FlowNode[]
  /** Called when the user changes the split instance value. undefined = not split. */
  onChange: (value: number | undefined) => void
}

function SplitInstanceControlInner({ node, value, allNodes, onChange }: Props) {
  const isOn = value !== undefined

  const { next, options } = useMemo(() => {
    const used = findUsedInstanceNumbers(node, allNodes)
    const n = nextAvailableInstance(used)
    const opts = Array.from(
      new Set([...used, ...(value !== undefined ? [value] : []), n])
    ).sort((a, b) => a - b)
    return { next: n, options: opts }
  }, [node, allNodes, value])

  function handleToggle() {
    if (isOn) {
      onChange(undefined)
    } else {
      onChange(next)
    }
  }

  function handleSelect(e: { target: { value: string } }) {
    const n = parseInt(e.target.value, 10)
    if (!isNaN(n)) onChange(n)
  }

  return (
    <div className="split-instance-control">
      <label className="split-instance-control__toggle">
        <input type="checkbox" checked={isOn} onChange={handleToggle} />
        Split device
      </label>
      {isOn && (
        <select
          className="split-instance-control__select"
          value={value}
          onChange={handleSelect}
          aria-label="Instance number"
        >
          {options.map((n) => (
            <option key={n} value={n}>
              #{n}
            </option>
          ))}
        </select>
      )}
    </div>
  )
}

export const SplitInstanceControl = memo(SplitInstanceControlInner)
