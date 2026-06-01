import type { GroupingZoneFlowNode, GroupingZoneNodeData, GroupingZoneShape } from '../../nodeData/types'
import { PropSection } from '../PropertiesPanel'
import { MakeModelFields } from './MakeModelFields'
import { readMakeModel } from '../../nodeMakeModel'
import { useNodeDataPatch } from './useNodeDataPatch'

export function GroupingZoneForm({
  node,
  onUpdate,
}: {
  node: GroupingZoneFlowNode
  onUpdate: (nodeId: string, data: GroupingZoneNodeData) => void
}) {
  const { id } = node
  const data = node.data as GroupingZoneNodeData
  const patch = useNodeDataPatch<GroupingZoneNodeData>(id, data, onUpdate)
  const { manufacturer, model, datasheetLink } = readMakeModel(data)

  return (
    <div className="properties-panel">
      <PropSection label="Grouping zone" defaultOpen={true}>
        <label className="properties-panel__label">
          Label
          <input
            className="properties-panel__input"
            value={data.label ?? ''}
            onChange={(e) => patch({ label: e.target.value })}
            placeholder="Zone label"
          />
        </label>
        <label className="properties-panel__label">
          Shape
          <select
            className="properties-panel__input"
            value={data.shape ?? 'rect'}
            onChange={(e) => patch({ shape: e.target.value as GroupingZoneShape })}
          >
            <option value="rect">Rectangle</option>
            <option value="polyline">Polyline</option>
          </select>
        </label>
        <p className="properties-panel__hint">
          Resize by dragging the handles. Nodes inside the boundary are grouped for reporting.
        </p>
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
