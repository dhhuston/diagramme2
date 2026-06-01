import { useEffect, useState } from 'react'

import type { EmbeddedPreset, ProjectState } from '../tauriIpc'

import './PresetToolbox.css'

const STORAGE_EXPANDED = 'diagramme.presetToolbox.expanded'

type PresetToolboxProps = {
  presets: EmbeddedPreset[]
  onProjectUpdated: (p: ProjectState) => void
  onInsertPreset: (
    preset: EmbeddedPreset,
    flowPosition: { x: number; y: number },
  ) => void | Promise<void>
  getDefaultInsertFlowPosition: () => { x: number; y: number }
  canSaveSelectionAsPreset: boolean
  onSaveSelectionAsPreset: () => void
  /** Chrome-only: show UI with actions disabled (no preset IPC). */
  shellDisabled?: boolean
}

export function PresetToolbox({
  presets,
  canSaveSelectionAsPreset,
  shellDisabled = false,
}: PresetToolboxProps) {
  const [expanded, setExpanded] = useState(() => {
    try {
      return localStorage.getItem(STORAGE_EXPANDED) !== 'false'
    } catch {
      return true
    }
  })

  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_EXPANDED, expanded ? 'true' : 'false')
    } catch {
      // ignore
    }
  }, [expanded])

  const rootClass = shellDisabled
    ? 'preset-toolbox preset-toolbox--shell-disabled'
    : 'preset-toolbox'

  return (
    <div className={rootClass}>
      <button
        type="button"
        className="preset-toolbox__toggle"
        aria-expanded={expanded}
        onClick={() => setExpanded((v) => !v)}
      >
        <span>Presets</span>
        <span className="preset-toolbox__chevron" aria-hidden>
          ›
        </span>
      </button>

      {expanded && (
        <div className="preset-toolbox__body">
          <div className="preset-toolbox__toolbar">
            <button type="button" className="preset-toolbox__import" disabled={shellDisabled}>
              Import…
            </button>
            <button
              type="button"
              className="preset-toolbox__save-selection"
              disabled={shellDisabled || !canSaveSelectionAsPreset}
            >
              Save selection as preset…
            </button>
          </div>

          <div className="preset-toolbox__scroll">
            {presets.length === 0 ? (
              <p className="preset-toolbox__empty">
                {shellDisabled
                  ? 'Preset library will be wired in a later pass.'
                  : 'No presets in this project. Import .avdevice / .plate files.'}
              </p>
            ) : null}
          </div>
        </div>
      )}
    </div>
  )
}

export const MIME_PRESET = 'application/x-diagramme-preset'
