import type { FlyoffNoteFlowNode, FlyoffNoteNodeData } from '../../nodeData/types'
import { PropSection, WireCategorySelect } from '../PropertiesPanel'
import { MakeModelFields } from './MakeModelFields'
import { readMakeModel } from '../../nodeMakeModel'
import { useNodeDataPatch } from './useNodeDataPatch'

export function FlyoffNoteForm({
  node,
  onUpdate,
}: {
  node: FlyoffNoteFlowNode
  onUpdate: (nodeId: string, data: FlyoffNoteNodeData) => void
}) {
  const { id } = node
  const data = node.data as FlyoffNoteNodeData
  const patch = useNodeDataPatch<FlyoffNoteNodeData>(id, data, onUpdate)

  const portDirection = data.portDirection === 'input' ? 'input' : 'output'
  const { manufacturer, model, datasheetLink } = readMakeModel(data)

  return (
    <div className="properties-panel">
      <PropSection label="Flyoff" defaultOpen={true}>
        <label className="properties-panel__label">
          Text
          <span className="properties-panel__hint">multiple lines; triangle stays aligned to the first line</span>
          <textarea
            className="properties-panel__input properties-panel__textarea properties-panel__textarea--text-block"
            value={data.text}
            onChange={(e) => patch({ text: e.target.value })}
            rows={6}
            spellCheck={true}
          />
        </label>

        <label className="properties-panel__label">
          Port
          <select
            className="properties-panel__input"
            value={portDirection}
            onChange={(e) => patch({ portDirection: e.target.value as 'input' | 'output' })}
            aria-label="Input or output"
          >
            <option value="output">Output</option>
            <option value="input">Input</option>
          </select>
        </label>

        <WireCategorySelect
          value={data.wireCategory}
          onChange={(cat) => patch({ wireCategory: cat })}
        />

        <p className="properties-panel__hint">
          Width fits the text until you drag the side handles; then that width is kept (text wraps inside). Height
          always follows the text.
        </p>
        {data.widthLocked === true && (
          <button type="button" className="properties-panel__btn" onClick={() => patch({ widthLocked: false })}>
            Fit width to text
          </button>
        )}
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
