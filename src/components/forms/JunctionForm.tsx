import type { FlowNode } from '../../tauriIpc'
import type { JunctionFlowNode, JunctionNodeData } from '../../nodeData/types'
import { PropSection } from '../PropertiesPanel'
import { SplitInstanceControl } from './SplitInstanceControl'
import { MakeModelFields } from './MakeModelFields'
import { readMakeModel } from '../../nodeMakeModel'
import { useNodeDataPatch } from './useNodeDataPatch'

export function JunctionForm({
  node,
  onUpdate,
  allNodes,
}: {
  node: JunctionFlowNode
  onUpdate: (nodeId: string, data: JunctionNodeData) => void
  allNodes: FlowNode[]
}) {
  const { id } = node
  const data = node.data as JunctionNodeData
  const patch = useNodeDataPatch<JunctionNodeData>(id, data, onUpdate)
  const { manufacturer, model, datasheetLink } = readMakeModel(data)

  return (
    <div className="properties-panel">
      <PropSection label="Identity" defaultOpen={true}>
        <label className="properties-panel__label">
          Code
          <input
            className="properties-panel__input"
            value={data.tagCode}
            onChange={(e) => patch({ tagCode: e.target.value })}
          />
        </label>
        <label className="properties-panel__label">
          Number
          <input
            className="properties-panel__input"
            value={data.tagNumber}
            onChange={(e) => patch({ tagNumber: e.target.value })}
          />
        </label>
        <label className="properties-panel__label">
          Rows
          <input
            className="properties-panel__input"
            type="number"
            min="1"
            max="32"
            value={data.rowCount}
            onChange={(e) => {
              const v = Math.max(1, Math.min(32, parseInt(e.target.value, 10) || 1))
              patch({ rowCount: v })
            }}
          />
        </label>
        <SplitInstanceControl
          node={node}
          value={data.splitInstance}
          allNodes={allNodes}
          onChange={(splitInstance) => patch({ splitInstance })}
        />
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
