import { TEXT_BLOCK_FONT_SIZES } from '../../constants/textBlockConstants'
import type { TextBlockFlowNode, TextBlockNodeData } from '../../nodeData/types'
import { PropSection } from '../PropertiesPanel'
import { MakeModelFields } from './MakeModelFields'
import { readMakeModel } from '../../nodeMakeModel'
import { useNodeDataPatch } from './useNodeDataPatch'

export function TextBlockForm({
  node,
  onUpdate,
}: {
  node: TextBlockFlowNode
  onUpdate: (nodeId: string, data: TextBlockNodeData) => void
}) {
  const { id } = node
  const data = node.data as TextBlockNodeData
  const patch = useNodeDataPatch<TextBlockNodeData>(id, data, onUpdate)

  const sizeFieldId = `text-block-font-${id}`
  const fontSizePx = (TEXT_BLOCK_FONT_SIZES as readonly number[]).includes(data.fontSizePx) ? data.fontSizePx : 14
  const { manufacturer, model, datasheetLink } = readMakeModel(data)

  return (
    <div className="properties-panel">
      <PropSection label="Text" defaultOpen={true}>
        <label className="properties-panel__label">
          Content
          <textarea
            className="properties-panel__input properties-panel__textarea properties-panel__textarea--text-block"
            value={data.text}
            onChange={(e) => patch({ text: e.target.value })}
            rows={6}
            spellCheck={true}
          />
        </label>
        <label className="properties-panel__label" htmlFor={sizeFieldId}>
          Font size
          <select
            id={sizeFieldId}
            className="properties-panel__input"
            value={fontSizePx}
            onChange={(e) => patch({ fontSizePx: Number(e.target.value) })}
          >
            {TEXT_BLOCK_FONT_SIZES.map((px) => (
              <option key={px} value={px}>{px}px</option>
            ))}
          </select>
        </label>
        <p className="properties-panel__hint">Drag the handles on the block to resize the box.</p>
      </PropSection>
      <MakeModelFields
        manufacturer={manufacturer}
        model={model}
        datasheetLink={datasheetLink}
        onManufacturerChange={(v) => patch({ manufacturer: v })}
        onModelChange={(v) => patch({ model: v })}
        onDatasheetLinkChange={(v) => patch({ datasheetLink: v })}
      />
    </div>
  )
}
