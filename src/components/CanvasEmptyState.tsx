import './CanvasEmptyState.css'

const DISMISS_KEY = 'diagramme.emptyState.dismissed'

export function isEmptyStateDismissed(): boolean {
  try {
    return localStorage.getItem(DISMISS_KEY) === 'true'
  } catch {
    return false
  }
}

type Props = {
  onDismiss: () => void
  onDismissForever: () => void
}

export function CanvasEmptyState({ onDismiss, onDismissForever }: Props) {
  function handleForever() {
    try {
      localStorage.setItem(DISMISS_KEY, 'true')
    } catch { /* */ }
    onDismissForever()
  }

  return (
    <div className="canvas-empty" role="status" aria-live="polite">
      <div className="canvas-empty__card">
        <div className="canvas-empty__hints">
          <div className="canvas-empty__hint">
            <span className="canvas-empty__hint-key">Drag</span>
            Pan the canvas
          </div>
          <div className="canvas-empty__hint">
            <span className="canvas-empty__hint-key">Scroll</span>
            Zoom in and out
          </div>
          <div className="canvas-empty__hint">
            <span className="canvas-empty__hint-key">Palette</span>
            Click any item to add it to the canvas
          </div>
        </div>
        <div className="canvas-empty__actions">
          <button type="button" className="canvas-empty__got-it" onClick={onDismiss}>
            Got it
          </button>
          <button type="button" className="canvas-empty__skip" onClick={handleForever}>
            Don't show again
          </button>
        </div>
      </div>
    </div>
  )
}
