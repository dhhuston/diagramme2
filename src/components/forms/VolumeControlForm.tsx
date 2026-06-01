import type { FlowNode } from '../../tauriIpc'
import type { VolumeControlFlowNode, VolumeControlNodeData } from '../../nodeData/types'
import { PropSection } from '../PropertiesPanel'
import { SplitInstanceControl } from './SplitInstanceControl'
import { MakeModelFields } from './MakeModelFields'
import { readMakeModel } from '../../nodeMakeModel'
import { useNodeDataPatch } from './useNodeDataPatch'

interface Props {
  node: VolumeControlFlowNode
  onUpdate: (nodeId: string, data: VolumeControlNodeData) => void
  allNodes: FlowNode[]
}

export function VolumeControlForm({ node, onUpdate, allNodes }: Props) {
  const { id } = node
  const data = node.data as VolumeControlNodeData
  const patch = useNodeDataPatch<VolumeControlNodeData>(id, data, onUpdate)
  const { manufacturer, model, datasheetLink } = readMakeModel(data)

  return (
    <div className="properties-panel">
      <PropSection label="Volume control" defaultOpen={true}>
        <p className="properties-panel__hint">
          Fixed VC schematic. Input on the left, output on the right.
        </p>
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
