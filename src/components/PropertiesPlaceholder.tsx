import type { HitTarget } from '../canvas/sceneTypes'

type PropertiesPlaceholderProps = {
  selection: HitTarget | null
}

export function PropertiesPlaceholder({ selection }: PropertiesPlaceholderProps) {
  if (!selection) {
    return (
      <div className="props-placeholder">
        <p className="props-placeholder__lead">Select a node or wire on the canvas.</p>
        <p className="props-placeholder__hint">
          Full properties editing will be wired in a later pass.
        </p>
      </div>
    )
  }

  return (
    <div className="props-placeholder">
      <p className="props-placeholder__lead">Selection</p>
      <dl className="props-placeholder__dl">
        <dt>Hit id</dt>
        <dd>{selection.id}</dd>
        {selection.node_id ? (
          <>
            <dt>Node</dt>
            <dd>{selection.node_id}</dd>
          </>
        ) : null}
        {selection.handle_id ? (
          <>
            <dt>Handle</dt>
            <dd>{selection.handle_id}</dd>
          </>
        ) : null}
        {selection.edge_id ? (
          <>
            <dt>Edge</dt>
            <dd>{selection.edge_id}</dd>
          </>
        ) : null}
      </dl>
    </div>
  )
}
