import { PropSection } from '../PropertiesPanel'
import { isDatasheetHyperlinkUrl } from '../../export/xlsxDatasheetLinks'

type Props = {
  manufacturer: string
  model: string
  datasheetLink: string
  onManufacturerChange: (value: string) => void
  onModelChange: (value: string) => void
  onDatasheetLinkChange: (value: string) => void
  defaultOpen?: boolean
}

/** Properties-panel fields only; not used by schematic or DXF rendering. */
export function MakeModelFields({
  manufacturer,
  model,
  datasheetLink,
  onManufacturerChange,
  onModelChange,
  onDatasheetLinkChange,
  defaultOpen = false,
}: Props) {
  const link = datasheetLink.trim()
  const linkOpenable = isDatasheetHyperlinkUrl(link)

  return (
    <PropSection label="Equipment" defaultOpen={defaultOpen}>
      <p className="properties-panel__hint">
        For schedules and exports only — not shown on the canvas or in DXF. Datasheet links become
        hyperlinks in Excel exports when they start with http:// or https://.
      </p>
      <label className="properties-panel__label">
        Manufacturer
        <input
          className="properties-panel__input"
          value={manufacturer}
          onChange={(e) => onManufacturerChange(e.target.value)}
          autoComplete="off"
        />
      </label>
      <label className="properties-panel__label">
        Model
        <input
          className="properties-panel__input"
          value={model}
          onChange={(e) => onModelChange(e.target.value)}
          autoComplete="off"
        />
      </label>
      <label className="properties-panel__label">
        Datasheet link
        <input
          className="properties-panel__input"
          type="url"
          value={datasheetLink}
          onChange={(e) => onDatasheetLinkChange(e.target.value)}
          placeholder="https://…"
          autoComplete="off"
          spellCheck={false}
        />
      </label>
      {linkOpenable ? (
        <a
          className="properties-panel__link"
          href={link}
          target="_blank"
          rel="noopener noreferrer"
        >
          Open datasheet
        </a>
      ) : null}
    </PropSection>
  )
}
