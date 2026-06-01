import React, {
  useCallback,
  useEffect,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
} from 'react'
import './PropertiesPanelFrame.css'

const KEY_WIDTH = 'diagramme.propsPanel.width'
const MIN_W = 280
const MAX_W = 480
const DEFAULT_W = 320

function loadWidth(): number {
  try {
    const n = Number(localStorage.getItem(KEY_WIDTH))
    if (Number.isFinite(n) && n >= MIN_W && n <= MAX_W) return n
  } catch { /* ignore */ }
  return DEFAULT_W
}

function DeleteFooterButton({ onDelete }: { onDelete: () => void }) {
  const [confirming, setConfirming] = useState(false)

  const handleClick = () => {
    if (confirming) {
      onDelete()
    } else {
      setConfirming(true)
    }
  }

  const handleBlur = () => setConfirming(false)

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Escape') {
      e.preventDefault()
      setConfirming(false)
    }
  }

  return (
    <button
      type="button"
      className={`props-frame__delete-btn${confirming ? ' props-frame__delete-btn--confirm' : ''}`}
      onClick={handleClick}
      onBlur={handleBlur}
      onKeyDown={handleKeyDown}
      aria-label={confirming ? 'Confirm delete node' : 'Delete node'}
    >
      {confirming ? 'Confirm delete?' : 'Delete node'}
    </button>
  )
}

type Props = {
  open: boolean
  nodeLabel: string
  onClose: () => void
  onDeleteNode: () => void
  canDelete: boolean
  children: React.ReactNode
}

export function PropertiesPanelFrame({ open, nodeLabel, onClose, onDeleteNode, canDelete, children }: Props) {
  const [width, setWidth] = useState(loadWidth)
  const widthRef = useRef(width)
  const resizeRef = useRef<{ startX: number; startW: number } | null>(null)

  useEffect(() => { widthRef.current = width }, [width])

  const persistWidth = useCallback((w: number) => {
    try { localStorage.setItem(KEY_WIDTH, String(w)) } catch { /* ignore */ }
  }, [])

  const onResizePointerDown = useCallback((e: ReactPointerEvent<HTMLButtonElement>) => {
    if (e.button !== 0) return
    e.preventDefault()
    e.stopPropagation()
    resizeRef.current = { startX: e.clientX, startW: widthRef.current }
    e.currentTarget.setPointerCapture(e.pointerId)
  }, [])

  const onResizePointerMove = useCallback((e: ReactPointerEvent<HTMLButtonElement>) => {
    const d = resizeRef.current
    if (!d) return
    const next = Math.min(MAX_W, Math.max(MIN_W, d.startW + (d.startX - e.clientX)))
    widthRef.current = next
    setWidth(next)
  }, [])

  const setPanelWidth = useCallback((next: number) => {
    const clamped = Math.min(MAX_W, Math.max(MIN_W, next))
    widthRef.current = clamped
    setWidth(clamped)
    persistWidth(clamped)
  }, [persistWidth])

  const onResizeKeyDown = useCallback((e: React.KeyboardEvent<HTMLButtonElement>) => {
    const step = e.shiftKey ? 24 : 12
    switch (e.key) {
      case 'ArrowLeft':
        e.preventDefault()
        setPanelWidth(widthRef.current + step)
        break
      case 'ArrowRight':
        e.preventDefault()
        setPanelWidth(widthRef.current - step)
        break
      case 'Home':
        e.preventDefault()
        setPanelWidth(MIN_W)
        break
      case 'End':
        e.preventDefault()
        setPanelWidth(MAX_W)
        break
    }
  }, [setPanelWidth])

  const endResize = useCallback((e: ReactPointerEvent<HTMLButtonElement>) => {
    if (resizeRef.current) {
      persistWidth(widthRef.current)
      resizeRef.current = null
    }
    if (e.currentTarget.hasPointerCapture(e.pointerId)) {
      e.currentTarget.releasePointerCapture(e.pointerId)
    }
  }, [persistWidth])

  return (
    <aside
      className={`props-frame${open ? ' props-frame--open' : ''}`}
      style={{ width }}
      aria-label="Properties"
      aria-hidden={!open}
    >
      <button
        type="button"
        className="props-frame__resize"
        role="separator"
        aria-label="Resize properties panel"
        aria-valuemin={MIN_W}
        aria-valuemax={MAX_W}
        aria-valuenow={width}
        aria-valuetext={`${width}px wide`}
        aria-orientation="vertical"
        onPointerDown={onResizePointerDown}
        onPointerMove={onResizePointerMove}
        onPointerUp={endResize}
        onPointerCancel={endResize}
        onKeyDown={onResizeKeyDown}
      />
      {open && (
        <>
          <div className="props-frame__header">
            <span className="props-frame__node-label" title={nodeLabel}>{nodeLabel}</span>
            <button
              type="button"
              className="props-frame__close"
              onClick={onClose}
              aria-label="Close properties panel"
              title="Close"
            >
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
                <path d="M1.5 1.5L10.5 10.5M10.5 1.5L1.5 10.5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
              </svg>
            </button>
          </div>
          <div className="props-frame__scroll">
            {children}
          </div>
          {canDelete && (
            <div className="props-frame__footer">
              <DeleteFooterButton onDelete={onDeleteNode} />
            </div>
          )}
        </>
      )}
    </aside>
  )
}
