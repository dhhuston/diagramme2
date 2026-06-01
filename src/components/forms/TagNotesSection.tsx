import { PropSection } from '../PropertiesPanel'

type Props = {
  value: string
  onChange: (value: string) => void
}

export function TagNotesSection({ value, onChange }: Props) {
  return (
    <PropSection label="Notes" defaultOpen={false}>
      <p className="properties-panel__hint">Shown in tag reports only; not on the diagram.</p>
      <label className="properties-panel__label">
        Notes
        <textarea
          className="properties-panel__input properties-panel__textarea"
          rows={3}
          value={value}
          onChange={(e) => onChange(e.target.value)}
          spellCheck
        />
      </label>
    </PropSection>
  )
}
