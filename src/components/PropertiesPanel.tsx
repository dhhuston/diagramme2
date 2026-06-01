import { useCallback, useEffect, useId, useMemo, useState } from 'react'
import type { FlowEdge, FlowNode, ProjectState, WireCategory } from '../tauriIpc'
import { labelForWireCategory, WIRE_CATEGORY_OPTIONS } from '../constants/wireCategory'
import type {
  AntennaSymbolFlowNode,
  AntennaSymbolNodeData,
  AvPlateFlowNode,
  AvPlateNodeData,
  ColumnGroup,
  DeviceFlowNodeV2,
  DeviceNodeData,
  DppPatchPanelFlowNode,
  DppPatchPanelNodeData,
  DppPatchRow,
  LppPatchPanelFlowNode,
  LppPatchPanelNodeData,
  LppPatchRow,
  MicBlockFlowNode,
  MicBlockNodeData,
  MlpNormalling,
  MlpPatchPanelFlowNode,
  MlpPatchPanelNodeData,
  MlpPatchRow,
  PortRow,
  SpeakerBlockFlowNode,
  SpeakerBlockNodeData,
  SpeakerBlockSymbolKind,
  VpbNormalling,
  VpbPatchPanelFlowNode,
  VpbPatchPanelNodeData,
  VpbPatchRow,
  FlyoffNoteNodeData,
  GroupingZoneNodeData,
  JunctionNodeData,
  TextBlockNodeData,
  VolumeControlNodeData,
  WiretagFlowNode,
} from '../nodeData/types'
import { GroupingZoneForm } from './forms/GroupingZoneForm'
import { MakeModelFields } from './forms/MakeModelFields'
import { readMakeModel, type NodeMakeModelFields } from '../nodeMakeModel'
import { VolumeControlForm } from './forms/VolumeControlForm'
import { SplitInstanceControl } from './forms/SplitInstanceControl'
import { FlyoffNoteForm } from './forms/FlyoffNoteForm'
import { TextBlockForm } from './forms/TextBlockForm'
import { JunctionForm } from './forms/JunctionForm'
import { useNodeDataPatch } from './forms/useNodeDataPatch'
import { TagNotesSection } from './forms/TagNotesSection'
import { readTagNotes } from '../nodeTagNotes'
import type { WiretagNodeData } from '../wiretagGraph'
import { pairEquipmentSummaryText } from '../wiretagGraph'
import type { PropertiesPanelSelection } from '../propertiesSelection'
import './PropertiesPanel.css'

function IconClose() {
  return (
    <svg width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
      <path d="M2.5 2.5L9.5 9.5M9.5 2.5L2.5 9.5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
    </svg>
  )
}

// ─── Collapsible section ───────────────────────────────────────────────────

export function PropSection({
  label,
  children,
  defaultOpen = true,
}: {
  label: string
  children: React.ReactNode
  defaultOpen?: boolean
}) {
  const [open, setOpen] = useState(defaultOpen)
  const triggerId = useId()
  return (
    <div className="prop-section">
      <button
        type="button"
        id={triggerId}
        className={`prop-section__trigger${open ? ' prop-section__trigger--open' : ''}`}
        onClick={() => setOpen((v) => !v)}
        aria-expanded={open}
      >
        {label}
        <svg className="prop-section__chevron" width="12" height="12" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path d="M4.5 2L9 6L4.5 10" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      </button>
      <div
        className={`prop-section__body${open ? ' prop-section__body--open' : ''}`}
        role="group"
        aria-labelledby={triggerId}
      >
        <div className="prop-section__inner">
          <div className="prop-section__content">
            {children}
          </div>
        </div>
      </div>
    </div>
  )
}

// ─── Helpers ───────────────────────────────────────────────────────────────

function newRowId(prefix: string): string {
  return `${prefix}-${crypto.randomUUID().slice(0, 8)}`
}

function nodeDataOf<T>(node: FlowNode): T {
  return node.data as T
}

function useArrayField<T>(
  items: T[],
  onChange: (next: T[]) => void,
) {
  const setItem = useCallback(
    (index: number, item: T) => {
      const next = [...items]
      next[index] = item
      onChange(next)
    },
    [items, onChange],
  )

  const addItem = useCallback(
    (item: T) => {
      onChange([...items, item])
    },
    [items, onChange],
  )

  const removeItem = useCallback(
    (index: number) => {
      onChange(items.filter((_, itemIndex) => itemIndex !== index))
    },
    [items, onChange],
  )

  return { addItem, removeItem, setItem }
}

function PortRowFields({
  row,
  fieldSuffix,
  onChange,
  onRemove,
  showDirection,
}: {
  row: PortRow
  fieldSuffix: string
  onChange: (r: PortRow) => void
  onRemove: () => void
  showDirection?: boolean
}) {
  const labelField = `prop-port-label-${fieldSuffix}`
  return (
    <div className={`properties-panel__port-row${showDirection ? ' properties-panel__port-row--with-dir' : ''}`}>
      <label className="properties-panel__sr-only" htmlFor={labelField}>Port label</label>
      <input
        id={labelField}
        className="properties-panel__input"
        value={row.label}
        onChange={(e) => onChange({ ...row, label: e.target.value })}
      />
      {showDirection && (
        <select
          className="properties-panel__select-compact properties-panel__select-compact--dir"
          value={row.direction ?? 'input'}
          onChange={(e) => onChange({ ...row, direction: e.target.value as 'input' | 'output' })}
          aria-label="Direction"
        >
          <option value="input">In</option>
          <option value="output">Out</option>
        </select>
      )}
      <select
        className="properties-panel__select-compact"
        value={row.wireCategory ?? 'default'}
        onChange={(e) => onChange({ ...row, wireCategory: e.target.value as WireCategory })}
        aria-label="Wire category"
      >
        {WIRE_CATEGORY_OPTIONS.map((opt) => (
          <option key={opt.value} value={opt.value}>{opt.label}</option>
        ))}
      </select>
      <button
        type="button"
        className="properties-panel__icon-btn"
        onClick={onRemove}
        aria-label="Remove port row"
      >
        <IconClose />
      </button>
    </div>
  )
}

function ColumnSection({
  title,
  columnKey,
  groups,
  onChangeGroups,
  onBeforeRemoveRow,
  onBeforeRemoveGroup,
  showDirection,
}: {
  title: string
  columnKey: string
  groups: ColumnGroup[]
  onChangeGroups: (next: ColumnGroup[]) => void
  onBeforeRemoveRow?: (groupIndex: number, rowId: string) => void
  onBeforeRemoveGroup?: (groupIndex: number, rowIds: string[]) => void
  showDirection?: boolean
}) {
  const updateGroup = useCallback(
    (index: number, next: ColumnGroup) => {
      const copy = [...groups]
      copy[index] = next
      onChangeGroups(copy)
    },
    [groups, onChangeGroups]
  )

  const removeGroup = useCallback(
    (index: number) => {
      const rowIds = groups[index]?.rows.map((r) => r.id) ?? []
      if (rowIds.length > 0) onBeforeRemoveGroup?.(index, rowIds)
      onChangeGroups(groups.filter((_, i) => i !== index))
    },
    [groups, onChangeGroups, onBeforeRemoveGroup],
  )

  const addGroup = useCallback(() => {
    onChangeGroups([
      ...groups,
      { header: 'New group', rows: [{ id: newRowId(columnKey), label: '1' }] },
    ])
  }, [groups, onChangeGroups, columnKey])

  return (
    <PropSection label={title} defaultOpen={true}>
      {groups.map((group, gi) => (
        <div key={`${columnKey}-g-${gi}`} className="properties-panel__group">
          <div className="properties-panel__group-head">
            <label className="properties-panel__label">
              Group header
              <span className="properties-panel__hint">(leave empty for continuation)</span>
              <input
                className="properties-panel__input"
                value={group.header ?? ''}
                onChange={(e) =>
                  updateGroup(gi, {
                    ...group,
                    header: e.target.value.trim() === '' ? undefined : e.target.value,
                  })
                }
                placeholder="e.g. HDMI / Dante"
              />
            </label>
            <button
              type="button"
              className="properties-panel__btn properties-panel__btn--danger"
              onClick={() => removeGroup(gi)}
            >
              Remove group
            </button>
          </div>
          <div className="properties-panel__port-head">
            <span>Label</span>
            {showDirection && <span>Dir</span>}
            <span>Type</span>
            <span aria-hidden className="properties-panel__port-head-spacer" />
          </div>
          {group.rows.map((row, ri) => (
            <PortRowFields
              key={`${columnKey}-${gi}-${ri}`}
              fieldSuffix={`${columnKey}-${gi}-${ri}`}
              row={row}
              onChange={(r) => {
                const rows = [...group.rows]
                rows[ri] = r
                updateGroup(gi, { ...group, rows })
              }}
              onRemove={() => {
                onBeforeRemoveRow?.(gi, row.id)
                const rows = group.rows.filter((_, i) => i !== ri)
                updateGroup(gi, { ...group, rows })
              }}
              showDirection={showDirection}
            />
          ))}
          <button
            type="button"
            className="properties-panel__btn properties-panel__btn--small"
            onClick={() =>
              updateGroup(gi, {
                ...group,
                rows: [...group.rows, { id: newRowId(columnKey), label: '', ...(showDirection ? { direction: 'input' as const } : {}) }],
              })
            }
          >
            Add port row
          </button>
        </div>
      ))}
      <button type="button" className="properties-panel__btn" onClick={addGroup}>
        Add signal group
      </button>
    </PropSection>
  )
}

type Props = {
  selection: PropertiesPanelSelection | null
  selectedEdge?: FlowEdge | null
  diagramNodes: FlowNode[]
  diagramEdges: FlowEdge[]
  project: ProjectState | null
  onUpdateDevice: (nodeId: string, data: DeviceNodeData) => void
  onUpdateAvPlate: (nodeId: string, data: AvPlateNodeData) => void
  onUpdateMicBlock: (nodeId: string, data: MicBlockNodeData) => void
  onUpdateSpeakerBlock: (nodeId: string, data: SpeakerBlockNodeData) => void
  onUpdateAntennaSymbol: (nodeId: string, data: AntennaSymbolNodeData) => void
  onReplaceAntennaNodeType: (
    nodeId: string,
    nodeType: 'antennaTransmitterSymbol' | 'antennaReceiverSymbol',
    data: AntennaSymbolNodeData,
  ) => void
  onUpdateLppPatchPanel: (nodeId: string, data: LppPatchPanelNodeData) => void
  onUpdateDppPatchPanel: (nodeId: string, data: DppPatchPanelNodeData) => void
  onUpdateMlpPatchPanel: (nodeId: string, data: MlpPatchPanelNodeData) => void
  onUpdateVpbPatchPanel: (nodeId: string, data: VpbPatchPanelNodeData) => void
  onUpdateTextBlock: (nodeId: string, data: TextBlockNodeData) => void
  onUpdateFlyoffNote: (nodeId: string, data: FlyoffNoteNodeData) => void
  onUpdateJunction: (nodeId: string, data: JunctionNodeData) => void
  onUpdateGroupingZone: (nodeId: string, data: GroupingZoneNodeData) => void
  onUpdateVolumeControl: (nodeId: string, data: VolumeControlNodeData) => void
  onApplyWiretagPairIndex: (pairId: string, newIndex: number) => void | Promise<void>
  onDeleteWiretagPair: (pairId: string) => void | Promise<void>
  onSetWiretagPairTagDescription: (nodeId: string, tagDescription: string) => void | Promise<void>
  onSetWiretagPairSheetName: (nodeId: string, sheetName: string) => void | Promise<void>
  onSetWiretagPairShowSheetName: (nodeId: string, showSheetName: boolean) => void | Promise<void>
  onUpdateWiretag: (nodeId: string, data: WiretagNodeData) => void
  onPatchNodeTagNotes: (nodeId: string, tagNotes: string) => void
}

export function PropertiesPanel({
  selection,
  selectedEdge,
  diagramNodes,
  diagramEdges,
  project,
  onUpdateDevice,
  onUpdateAvPlate,
  onUpdateMicBlock,
  onUpdateSpeakerBlock,
  onUpdateAntennaSymbol,
  onReplaceAntennaNodeType,
  onUpdateLppPatchPanel,
  onUpdateDppPatchPanel,
  onUpdateMlpPatchPanel,
  onUpdateVpbPatchPanel,
  onUpdateTextBlock,
  onUpdateFlyoffNote,
  onUpdateJunction,
  onUpdateGroupingZone,
  onUpdateVolumeControl,
  onApplyWiretagPairIndex,
  onDeleteWiretagPair,
  onSetWiretagPairTagDescription,
  onSetWiretagPairSheetName,
  onSetWiretagPairShowSheetName,
  onUpdateWiretag,
  onPatchNodeTagNotes,
}: Props) {
  const allNodes = useMemo(() => {
    if (!project) return diagramNodes
    // Replace the active sheet's stale snapshot with the live React Flow state so
    // the instance picker reflects unsaved placements on the current sheet.
    return project.sheets.flatMap((s) =>
      s.id === project.activeSheetId ? diagramNodes : s.state.nodes,
    )
  }, [project, diagramNodes])

  if (selectedEdge) {
    return <EdgeForm edge={selectedEdge} />
  }

  if (!selection) return null

  const tagNotesFooter = (
    <TagNotesSection
      value={readTagNotes(selection.node.data)}
      onChange={(tagNotes) => onPatchNodeTagNotes(selection.node.id, tagNotes)}
    />
  )

  let body: React.ReactNode = null

  if (selection.kind === 'deviceV2') {
    body = <DeviceForm node={selection.node} onUpdate={onUpdateDevice} allNodes={allNodes} />
  } else if (selection.kind === 'avPlate') {
    body = <AvPlateForm node={selection.node} onUpdate={onUpdateAvPlate} allNodes={allNodes} />
  } else if (selection.kind === 'micBlock') {
    body = <MicBlockForm node={selection.node} onUpdate={onUpdateMicBlock} allNodes={allNodes} />
  } else if (selection.kind === 'speakerBlock') {
    body = <SpeakerBlockForm node={selection.node} onUpdate={onUpdateSpeakerBlock} allNodes={allNodes} />
  } else if (selection.kind === 'volumeControl') {
    body = <VolumeControlForm node={selection.node} onUpdate={onUpdateVolumeControl} allNodes={allNodes} />
  } else if (selection.kind === 'antennaTransmitterSymbol' || selection.kind === 'antennaReceiverSymbol') {
    body = (
      <AntennaSymbolForm
        node={selection.node}
        onUpdate={onUpdateAntennaSymbol}
        onReplaceType={onReplaceAntennaNodeType}
        allNodes={allNodes}
      />
    )
  } else if (selection.kind === 'lppPatchPanel') {
    body = <LppForm node={selection.node} onUpdate={onUpdateLppPatchPanel} allNodes={allNodes} />
  } else if (selection.kind === 'dppPatchPanel') {
    body = <DppForm node={selection.node} onUpdate={onUpdateDppPatchPanel} allNodes={allNodes} />
  } else if (selection.kind === 'mlpPatchPanel') {
    body = <MlpForm node={selection.node} onUpdate={onUpdateMlpPatchPanel} allNodes={allNodes} />
  } else if (selection.kind === 'vpbPatchPanel') {
    body = <VpbForm node={selection.node} onUpdate={onUpdateVpbPatchPanel} allNodes={allNodes} />
  } else if (selection.kind === 'textBlock') {
    body = <TextBlockForm node={selection.node} onUpdate={onUpdateTextBlock} />
  } else if (selection.kind === 'flyoffNote') {
    body = <FlyoffNoteForm node={selection.node} onUpdate={onUpdateFlyoffNote} />
  } else if (selection.kind === 'junction') {
    body = <JunctionForm node={selection.node} onUpdate={onUpdateJunction} allNodes={allNodes} />
  } else if (selection.kind === 'groupingZone') {
    body = <GroupingZoneForm node={selection.node} onUpdate={onUpdateGroupingZone} />
  } else if (selection.kind === 'wiretag') {
    body = (
      <WiretagForm
        node={selection.node}
        diagramNodes={diagramNodes}
        diagramEdges={diagramEdges}
        onUpdate={onUpdateWiretag}
        onApplyPairIndex={onApplyWiretagPairIndex}
        onDeletePair={onDeleteWiretagPair}
        onSetTagDescription={onSetWiretagPairTagDescription}
        onSetSheetName={onSetWiretagPairSheetName}
        onSetShowSheetName={onSetWiretagPairShowSheetName}
      />
    )
  }

  return (
    <>
      {body}
      {tagNotesFooter}
    </>
  )
}

function MakeModelSection<T extends NodeMakeModelFields>({
  data,
  patch,
}: {
  data: T
  patch: (partial: Partial<T>) => void
}) {
  const { manufacturer, model, datasheetLink } = readMakeModel(data)
  return (
    <MakeModelFields
      manufacturer={manufacturer}
      model={model}
      datasheetLink={datasheetLink}
      onManufacturerChange={(v) => patch({ manufacturer: v } as Partial<T>)}
      onModelChange={(v) => patch({ model: v } as Partial<T>)}
      onDatasheetLinkChange={(v) => patch({ datasheetLink: v } as Partial<T>)}
    />
  )
}

// ─── Device ────────────────────────────────────────────────────────────────

function DeviceForm({
  node,
  onUpdate,
  allNodes,
}: {
  node: DeviceFlowNodeV2
  onUpdate: (nodeId: string, data: DeviceNodeData) => void
  allNodes: FlowNode[]
}) {
  const { id } = node
  const data = nodeDataOf<DeviceNodeData>(node)
  const patch = useNodeDataPatch<DeviceNodeData>(id, data, onUpdate)

  return (
    <div className="properties-panel">
      <PropSection label="Identity" defaultOpen={false}>
        <label className="properties-panel__label">
          Code
          <input className="properties-panel__input" value={data.tagCode} onChange={(e) => patch({ tagCode: e.target.value })} />
        </label>
        <label className="properties-panel__label">
          Number
          <input className="properties-panel__input" value={data.tagNumber} onChange={(e) => patch({ tagNumber: e.target.value })} />
        </label>
        <label className="properties-panel__label">
          Description
          <span className="properties-panel__hint">shown uppercase on the block</span>
          <input className="properties-panel__input" value={data.description} onChange={(e) => patch({ description: e.target.value })} />
        </label>
        <SplitInstanceControl
          node={node}
          value={data.splitInstance}
          allNodes={allNodes}
          onChange={(splitInstance) => patch({ splitInstance })}
        />
      </PropSection>
      <MakeModelSection data={data} patch={patch} />
      <ColumnSection
        title="Inputs"
        columnKey="leftColumn"
        groups={data.leftColumn}
        onChangeGroups={(leftColumn) => patch({ leftColumn })}
      />
      <ColumnSection
        title="Outputs"
        columnKey="rightColumn"
        groups={data.rightColumn}
        onChangeGroups={(rightColumn) => patch({ rightColumn })}
      />
    </div>
  )
}

// ─── AV Plate ──────────────────────────────────────────────────────────────

function AvPlateForm({
  node,
  onUpdate,
  allNodes,
}: {
  node: AvPlateFlowNode
  onUpdate: (nodeId: string, data: AvPlateNodeData) => void
  allNodes: FlowNode[]
}) {
  const { id } = node
  const data = nodeDataOf<AvPlateNodeData>(node)
  const patch = useNodeDataPatch<AvPlateNodeData>(id, data, onUpdate)

  return (
    <div className="properties-panel">
      <PropSection label="Identity" defaultOpen={false}>
        <label className="properties-panel__label">
          Code
          <input className="properties-panel__input" value={data.tagCode} onChange={(e) => patch({ tagCode: e.target.value })} />
        </label>
        <label className="properties-panel__label">
          Number
          <input className="properties-panel__input" value={data.tagNumber} onChange={(e) => patch({ tagNumber: e.target.value })} />
        </label>
        <label className="properties-panel__label">
          Description
          <input className="properties-panel__input" value={data.description} onChange={(e) => patch({ description: e.target.value })} />
        </label>
        <SplitInstanceControl
          node={node}
          value={data.splitInstance}
          allNodes={allNodes}
          onChange={(splitInstance) => patch({ splitInstance })}
        />
      </PropSection>
      <MakeModelSection data={data} patch={patch} />
      <ColumnSection
        title="Signal groups"
        columnKey="groups"
        groups={data.groups}
        onChangeGroups={(groups) => patch({ groups })}
        showDirection
      />
    </div>
  )
}

// ─── Shared controls ───────────────────────────────────────────────────────

export function WireCategorySelect({
  value,
  onChange,
}: {
  value: WireCategory | undefined
  onChange: (cat: WireCategory) => void
}) {
  return (
    <label className="properties-panel__label">
      Signal type
      <select
        className="properties-panel__input"
        value={value ?? 'default'}
        onChange={(e) => onChange(e.target.value as WireCategory)}
      >
        {WIRE_CATEGORY_OPTIONS.map((opt) => (
          <option key={opt.value} value={opt.value}>{opt.label}</option>
        ))}
      </select>
    </label>
  )
}

// ─── Edge form ─────────────────────────────────────────────────────────────

function EdgeForm({
  edge,
}: {
  edge: FlowEdge
}) {
  const edgeData = (edge.data ?? {}) as {
    wireCategory?: WireCategory
    wireCategoryMismatch?: boolean
  }
  const category = edgeData.wireCategory ?? 'default'
  const mismatch = Boolean(edgeData.wireCategoryMismatch)
  const categoryLabel = labelForWireCategory(category)
  return (
    <div className="properties-panel">
      <PropSection label="Wire" defaultOpen={true}>
        <label className="properties-panel__label">
          Signal type
          <span className="properties-panel__readonly">{categoryLabel}</span>
        </label>
        {mismatch && (
          <p className="properties-panel__note properties-panel__note--warn">
            Source and target signal types do not match.
          </p>
        )}
      </PropSection>
    </div>
  )
}

// ─── Mic block ─────────────────────────────────────────────────────────────

function MicBlockForm({
  node,
  onUpdate,
  allNodes,
}: {
  node: MicBlockFlowNode
  onUpdate: (nodeId: string, data: MicBlockNodeData) => void
  allNodes: FlowNode[]
}) {
  const { id } = node
  const data = nodeDataOf<MicBlockNodeData>(node)
  const patch = useNodeDataPatch<MicBlockNodeData>(id, data, onUpdate)

  return (
    <div className="properties-panel">
      <PropSection label="Labels" defaultOpen={true}>
        <label className="properties-panel__label">
          Device tag
          <span className="properties-panel__hint">large label, e.g. MIC 1</span>
          <input className="properties-panel__input" value={data.line1} onChange={(e) => patch({ line1: e.target.value })} />
        </label>
        <label className="properties-panel__label">
          Description
          <span className="properties-panel__hint">smaller label, e.g. OVERHEAD</span>
          <input className="properties-panel__input" value={data.line2} onChange={(e) => patch({ line2: e.target.value })} />
        </label>
        <label className="properties-panel__label">
          Channel number
          <input className="properties-panel__input" value={data.channelNumber} onChange={(e) => patch({ channelNumber: e.target.value })} />
        </label>
        <SplitInstanceControl
          node={node}
          value={data.splitInstance}
          allNodes={allNodes}
          onChange={(splitInstance) => patch({ splitInstance })}
        />
      </PropSection>
      <MakeModelSection data={data} patch={patch} />
      <PropSection label="Advanced" defaultOpen={false}>
        <WireCategorySelect
          value={data.wireCategory}
          onChange={(cat) => patch({ wireCategory: cat })}
        />
      </PropSection>
    </div>
  )
}

// ─── Antenna symbol ────────────────────────────────────────────────────────

function AntennaSymbolForm({
  node,
  onUpdate,
  onReplaceType,
  allNodes,
}: {
  node: AntennaSymbolFlowNode
  onUpdate: (nodeId: string, data: AntennaSymbolNodeData) => void
  onReplaceType: (
    nodeId: string,
    nodeType: 'antennaTransmitterSymbol' | 'antennaReceiverSymbol',
    data: AntennaSymbolNodeData,
  ) => void
  allNodes: FlowNode[]
}) {
  const { id, type } = node
  const data = nodeDataOf<AntennaSymbolNodeData>(node)
  const patch = useNodeDataPatch<AntennaSymbolNodeData>(id, data, onUpdate)
  const role = type === 'antennaTransmitterSymbol' ? 'transmitter' : 'receiver'

  return (
    <div className="properties-panel">
      <PropSection label="Antenna" defaultOpen={true}>
        <label className="properties-panel__label">
          Device tag
          <span className="properties-panel__hint">same size as loudspeaker line 1, opposite the L leg</span>
          <input
            className="properties-panel__input"
            value={data.line1 ?? ''}
            onChange={(e) => patch({ line1: e.target.value })}
          />
        </label>
        <label className="properties-panel__label">
          Role
          <span className="properties-panel__hint">
            Transmitter: tip on the left (target — wire from an output / source). Receiver: tip on the right (source — wire into an input / target).
          </span>
          <select
            className="properties-panel__input"
            value={role}
            onChange={(e) => {
              const next =
                e.target.value === 'transmitter' ? 'antennaTransmitterSymbol' : 'antennaReceiverSymbol'
              if (next === type) return
              const nextData: AntennaSymbolNodeData = {
                line1: data.line1,
                manufacturer: data.manufacturer,
                model: data.model,
                datasheetLink: data.datasheetLink,
                splitInstance: data.splitInstance,
              }
              onReplaceType(id, next, nextData)
            }}
          >
            <option value="transmitter">Transmitter</option>
            <option value="receiver">Receiver</option>
          </select>
        </label>
        <SplitInstanceControl
          node={node}
          value={data.splitInstance}
          allNodes={allNodes}
          onChange={(splitInstance) => patch({ splitInstance })}
        />
      </PropSection>
      <MakeModelSection data={data} patch={patch} />
    </div>
  )
}

// ─── Speaker block ─────────────────────────────────────────────────────────

function SpeakerBlockForm({
  node,
  onUpdate,
  allNodes,
}: {
  node: SpeakerBlockFlowNode
  onUpdate: (nodeId: string, data: SpeakerBlockNodeData) => void
  allNodes: FlowNode[]
}) {
  const { id } = node
  const data = nodeDataOf<SpeakerBlockNodeData>(node)
  const patch = useNodeDataPatch<SpeakerBlockNodeData>(id, data, onUpdate)

  const symbolKind: SpeakerBlockSymbolKind = data.symbolKind ?? 'standard'
  const passthruEnabled = data.passthruEnabled ?? false

  return (
    <div className="properties-panel">
      <PropSection label="Labels" defaultOpen={true}>
        <label className="properties-panel__label">
          Device tag
          <span className="properties-panel__hint">large label, e.g. SPK 1</span>
          <input className="properties-panel__input" value={data.line1} onChange={(e) => patch({ line1: e.target.value })} />
        </label>
        <label className="properties-panel__label">
          Description
          <span className="properties-panel__hint">smaller label, e.g. PLAN NORTH HL</span>
          <input className="properties-panel__input" value={data.line2} onChange={(e) => patch({ line2: e.target.value })} />
        </label>
        <label className="properties-panel__label">
          Symbol
          <span className="properties-panel__hint">low-Z cone, 70 V (X in coil box), or active (triangle in coil and cone)</span>
          <select
            className="properties-panel__input"
            value={symbolKind}
            onChange={(e) => patch({ symbolKind: e.target.value as SpeakerBlockSymbolKind })}
          >
            <option value="standard">Standard (low-Z)</option>
            <option value="70v">70 V</option>
            <option value="active">Active</option>
          </select>
        </label>
        <label className="properties-panel__label">
          <span className="properties-panel__inline-check">
            <input
              type="checkbox"
              checked={passthruEnabled}
              onChange={(e) => patch({ passthruEnabled: e.target.checked })}
            />
            Pass-through port
          </span>
          <span className="properties-panel__hint">
            Adds a daisy-chain source on the coil box, between mid height and the bottom of the rectangle
          </span>
        </label>
        <SplitInstanceControl
          node={node}
          value={data.splitInstance}
          allNodes={allNodes}
          onChange={(splitInstance) => patch({ splitInstance })}
        />
      </PropSection>
      <MakeModelSection data={data} patch={patch} />
      <PropSection label="Advanced" defaultOpen={false}>
        <WireCategorySelect
          value={data.wireCategory}
          onChange={(cat) => patch({ wireCategory: cat })}
        />
      </PropSection>
    </div>
  )
}

// ─── Patch panel shared rows ───────────────────────────────────────────────

function PatchIdentitySection({
  tagCode,
  tagNumber,
  descriptionLines,
  onTagCode,
  onTagNumber,
  onDescriptionLines,
  children,
}: {
  tagCode: string
  tagNumber: string
  descriptionLines: string[]
  onTagCode: (v: string) => void
  onTagNumber: (v: string) => void
  onDescriptionLines: (v: string[]) => void
  children?: React.ReactNode
}) {
  return (
    <PropSection label="Identity" defaultOpen={false}>
      <label className="properties-panel__label">
        Code
        <input className="properties-panel__input" value={tagCode} onChange={(e) => onTagCode(e.target.value)} />
      </label>
      <label className="properties-panel__label">
        Number
        <input className="properties-panel__input" value={tagNumber} onChange={(e) => onTagNumber(e.target.value)} />
      </label>
      <label className="properties-panel__label">
        Description
        <span className="properties-panel__hint">one line per row; shown uppercase on the block</span>
        <textarea
          className="properties-panel__input properties-panel__textarea"
          rows={3}
          value={descriptionLines.join('\n')}
          onChange={(e) => onDescriptionLines(e.target.value.split(/\r?\n/))}
          spellCheck={false}
        />
      </label>
      {children}
    </PropSection>
  )
}

// ─── LPP patch panel ───────────────────────────────────────────────────────

function LppForm({
  node,
  onUpdate,
  allNodes,
}: {
  node: LppPatchPanelFlowNode
  onUpdate: (nodeId: string, data: LppPatchPanelNodeData) => void
  allNodes: FlowNode[]
}) {
  const { id } = node
  const data = nodeDataOf<LppPatchPanelNodeData>(node)
  const patch = useNodeDataPatch<LppPatchPanelNodeData>(id, data, onUpdate)
  const rows = useArrayField<LppPatchRow>(data.rows, (next) => patch({ rows: next }))

  return (
    <div className="properties-panel">
      <PatchIdentitySection
          tagCode={data.tagCode}
          tagNumber={data.tagNumber}
          descriptionLines={data.descriptionLines}
          onTagCode={(v) => patch({ tagCode: v })}
          onTagNumber={(v) => patch({ tagNumber: v })}
          onDescriptionLines={(v) => patch({ descriptionLines: v })}
        >
          <SplitInstanceControl
            node={node}
            value={data.splitInstance}
            allNodes={allNodes}
            onChange={(splitInstance) => patch({ splitInstance })}
          />
        </PatchIdentitySection>
      <MakeModelSection data={data} patch={patch} />
      <PropSection label="Rows" defaultOpen={true}>
        <div className="properties-panel__lpp-grid-head" aria-hidden="true">
          <span>#</span><span>Conn.</span><span />
        </div>
        {data.rows.map((row, index) => (
          <div key={row.id} className="properties-panel__lpp-row">
            <span>{index + 1}</span>
            <label className="properties-panel__lpp-check">
              <input
                type="checkbox"
                checked={row.connected}
                onChange={(e) => rows.setItem(index, { ...row, connected: e.target.checked })}
                aria-label={`Row ${index + 1} connected`}
              />
            </label>
            <button type="button" className="properties-panel__icon-btn" onClick={() => rows.removeItem(index)} aria-label={`Remove row ${index + 1}`}><IconClose /></button>
          </div>
        ))}
        <button type="button" className="properties-panel__btn" onClick={() => rows.addItem({ id: newRowId('lpp'), connected: true })}>Add row</button>
      </PropSection>
      <PropSection label="Advanced" defaultOpen={false}>
        <div className="properties-panel__hint">Signal type: Audio (fixed for loudspeaker patch panels)</div>
      </PropSection>
    </div>
  )
}

// ─── DPP patch panel ───────────────────────────────────────────────────────

function DppForm({
  node,
  onUpdate,
  allNodes,
}: {
  node: DppPatchPanelFlowNode
  onUpdate: (nodeId: string, data: DppPatchPanelNodeData) => void
  allNodes: FlowNode[]
}) {
  const { id } = node
  const data = nodeDataOf<DppPatchPanelNodeData>(node)
  const patch = useNodeDataPatch<DppPatchPanelNodeData>(id, data, onUpdate)
  const rows = useArrayField<DppPatchRow>(data.rows, (next) => patch({ rows: next }))

  return (
    <div className="properties-panel">
      <PatchIdentitySection
        tagCode={data.tagCode}
        tagNumber={data.tagNumber}
        descriptionLines={data.descriptionLines}
        onTagCode={(v) => patch({ tagCode: v })}
        onTagNumber={(v) => patch({ tagNumber: v })}
        onDescriptionLines={(v) => patch({ descriptionLines: v })}
      >
        <SplitInstanceControl
          node={node}
          value={data.splitInstance}
          allNodes={allNodes}
          onChange={(splitInstance) => patch({ splitInstance })}
        />
      </PatchIdentitySection>
      <MakeModelSection data={data} patch={patch} />
      <PropSection label="Rows" defaultOpen={true}>
        <div className="properties-panel__dpp-grid-head" aria-hidden="true">
          <span>Port #</span><span>Direction</span><span />
        </div>
        {data.rows.map((row, index) => (
          <div key={row.id} className="properties-panel__dpp-row">
            <input
              className="properties-panel__input"
              aria-label={`Row ${index + 1} label`}
              value={row.label}
              onChange={(e) => rows.setItem(index, { ...row, label: e.target.value })}
            />
            <select
              className="properties-panel__select-compact"
              aria-label={`Row ${index + 1} direction`}
              value={row.direction}
              onChange={(e) => rows.setItem(index, { ...row, direction: e.target.value as 'input' | 'output' })}
            >
              <option value="output">Out →</option>
              <option value="input">← In</option>
            </select>
            <button type="button" className="properties-panel__icon-btn" onClick={() => rows.removeItem(index)} aria-label={`Remove row ${index + 1}`}><IconClose /></button>
          </div>
        ))}
        <button
          type="button"
          className="properties-panel__btn"
          onClick={() => rows.addItem({ id: newRowId('dpp'), label: String(data.rows.length + 1), direction: 'output' })}
        >
          Add row
        </button>
      </PropSection>
      <PropSection label="Advanced" defaultOpen={false}>
        <div className="properties-panel__hint">Signal type: Network (fixed for data patch panels)</div>
      </PropSection>
    </div>
  )
}

// ─── MLP patch panel ───────────────────────────────────────────────────────

function MlpForm({
  node,
  onUpdate,
  allNodes,
}: {
  node: MlpPatchPanelFlowNode
  onUpdate: (nodeId: string, data: MlpPatchPanelNodeData) => void
  allNodes: FlowNode[]
}) {
  const { id } = node
  const data = nodeDataOf<MlpPatchPanelNodeData>(node)
  const patch = useNodeDataPatch<MlpPatchPanelNodeData>(id, data, onUpdate)
  const rows = useArrayField<MlpPatchRow>(data.rows, (next) => patch({ rows: next }))
  const toNormalling = useCallback((raw: string): MlpNormalling => {
    if (raw === 'HN' || raw === 'FN') return raw
    return ''
  }, [])

  return (
    <div className="properties-panel">
      <PatchIdentitySection
        tagCode={data.tagCode}
        tagNumber={data.tagNumber}
        descriptionLines={data.descriptionLines}
        onTagCode={(v) => patch({ tagCode: v })}
        onTagNumber={(v) => patch({ tagNumber: v })}
        onDescriptionLines={(v) => patch({ descriptionLines: v })}
      >
        <SplitInstanceControl
          node={node}
          value={data.splitInstance}
          allNodes={allNodes}
          onChange={(splitInstance) => patch({ splitInstance })}
        />
      </PatchIdentitySection>
      <MakeModelSection data={data} patch={patch} />
      <PropSection label="Rows" defaultOpen={true}>
        <div className="properties-panel__mlp-grid-head" aria-hidden="true">
          <span>#</span><span>Norm.</span><span />
        </div>
        {data.rows.map((row, index) => (
          <div key={row.id} className="properties-panel__mlp-row">
            <span>{index + 1}</span>
            <select
              className="properties-panel__select-compact"
              aria-label={`Row ${index + 1} normalling`}
              value={row.normalling === 'HN' || row.normalling === 'FN' ? row.normalling : ''}
              onChange={(e) => rows.setItem(index, { ...row, normalling: toNormalling(e.target.value) })}
            >
              <option value="">—</option>
              <option value="HN">HN</option>
              <option value="FN">FN</option>
            </select>
            <button type="button" className="properties-panel__icon-btn" onClick={() => rows.removeItem(index)} aria-label={`Remove row ${index + 1}`}><IconClose /></button>
          </div>
        ))}
        <button type="button" className="properties-panel__btn" onClick={() => rows.addItem({ id: newRowId('mlp'), normalling: '' })}>Add row</button>
      </PropSection>
      <PropSection label="Advanced" defaultOpen={false}>
        <div className="properties-panel__hint">Signal type: Audio (fixed for mic/line patch panels)</div>
      </PropSection>
    </div>
  )
}

// ─── VPB patch panel ───────────────────────────────────────────────────────

function VpbForm({
  node,
  onUpdate,
  allNodes,
}: {
  node: VpbPatchPanelFlowNode
  onUpdate: (nodeId: string, data: VpbPatchPanelNodeData) => void
  allNodes: FlowNode[]
}) {
  const { id } = node
  const data = nodeDataOf<VpbPatchPanelNodeData>(node)
  const patch = useNodeDataPatch<VpbPatchPanelNodeData>(id, data, onUpdate)
  const rows = useArrayField<VpbPatchRow>(data.rows, (next) => patch({ rows: next }))
  const toNormalling = useCallback((raw: string): VpbNormalling => raw === 'N' ? 'N' : '', [])

  return (
    <div className="properties-panel">
      <PatchIdentitySection
        tagCode={data.tagCode}
        tagNumber={data.tagNumber}
        descriptionLines={data.descriptionLines}
        onTagCode={(v) => patch({ tagCode: v })}
        onTagNumber={(v) => patch({ tagNumber: v })}
        onDescriptionLines={(v) => patch({ descriptionLines: v })}
      >
        <SplitInstanceControl
          node={node}
          value={data.splitInstance}
          allNodes={allNodes}
          onChange={(splitInstance) => patch({ splitInstance })}
        />
      </PatchIdentitySection>
      <MakeModelSection data={data} patch={patch} />
      <PropSection label="Rows" defaultOpen={true}>
        <div className="properties-panel__mlp-grid-head" aria-hidden="true">
          <span>#</span><span>Norm.</span><span />
        </div>
        {data.rows.map((row, index) => (
          <div key={row.id} className="properties-panel__mlp-row">
            <span>{index + 1}</span>
            <select
              className="properties-panel__select-compact"
              aria-label={`Row ${index + 1} normalling`}
              value={row.normalling === 'N' ? 'N' : ''}
              onChange={(e) => rows.setItem(index, { ...row, normalling: toNormalling(e.target.value) })}
            >
              <option value="">—</option>
              <option value="N">N</option>
            </select>
            <button type="button" className="properties-panel__icon-btn" onClick={() => rows.removeItem(index)} aria-label={`Remove row ${index + 1}`}><IconClose /></button>
          </div>
        ))}
        <button type="button" className="properties-panel__btn" onClick={() => rows.addItem({ id: newRowId('vpb'), normalling: '' })}>Add row</button>
      </PropSection>
      <PropSection label="Advanced" defaultOpen={false}>
        <div className="properties-panel__hint">Signal type: Video (fixed for video patch panels)</div>
      </PropSection>
    </div>
  )
}

// ─── Wire tag pair ─────────────────────────────────────────────────────────

function WiretagForm({
  node,
  diagramNodes,
  diagramEdges,
  onUpdate,
  onApplyPairIndex,
  onDeletePair,
  onSetTagDescription,
  onSetSheetName,
  onSetShowSheetName,
}: {
  node: WiretagFlowNode
  diagramNodes: FlowNode[]
  diagramEdges: FlowEdge[]
  onUpdate: (nodeId: string, data: WiretagNodeData) => void
  onApplyPairIndex: (pairId: string, index: number) => void | Promise<void>
  onDeletePair: (pairId: string) => void | Promise<void>
  onSetTagDescription: (nodeId: string, tagDescription: string) => void | Promise<void>
  onSetSheetName: (nodeId: string, sheetName: string) => void | Promise<void>
  onSetShowSheetName: (nodeId: string, showSheetName: boolean) => void | Promise<void>
}) {
  const { id } = node
  const data = nodeDataOf<WiretagNodeData>(node)
  const patch = useNodeDataPatch<WiretagNodeData>(id, data, onUpdate)
  const tagDescription = data.tagDescription ?? ''
  const sheetName = data.sheetName ?? ''
  const showSheetName = Boolean(data.showSheetName)
  const [draftIndex, setDraftIndex] = useState(() => String(data.pairIndex))

  useEffect(() => {
    setDraftIndex(String(data.pairIndex))
  }, [data.pairIndex])

  const equipment = useMemo(
    () => pairEquipmentSummaryText(node, diagramNodes, diagramEdges),
    [node, diagramNodes, diagramEdges],
  )

  const applyIndex = useCallback(() => {
    const n = parseInt(draftIndex, 10)
    if (!Number.isFinite(n)) return
    void onApplyPairIndex(data.pairId, n)
  }, [draftIndex, data.pairId, onApplyPairIndex])

  return (
    <div className="properties-panel">
      <PropSection label="Wire tag" defaultOpen>
        <p className="properties-panel__hint">
          Remote equipment: <strong>{equipment.trim() ? equipment : '—'}</strong>
        </p>
        <label className="properties-panel__label">
          Tag description
          <span className="properties-panel__hint">
            shown uppercase in the tag band when set; leave blank to show the remote equipment label
          </span>
          <input
            className="properties-panel__input"
            value={tagDescription}
            onChange={(e) => void onSetTagDescription(node.id, e.target.value)}
          />
        </label>
        <label className="properties-panel__label properties-panel__label--checkbox-row">
          <input
            type="checkbox"
            checked={showSheetName}
            onChange={(e) => void onSetShowSheetName(node.id, e.target.checked)}
          />
          Show sheet name segment
        </label>
        <label className="properties-panel__label">
          Sheet name
          <span className="properties-panel__hint">shown in a dedicated segment left of the main tag text</span>
          <input
            className="properties-panel__input"
            value={sheetName}
            onChange={(e) => void onSetSheetName(node.id, e.target.value)}
          />
        </label>
        <label className="properties-panel__label">
          Pair index
          <span className="properties-panel__hint">same value on both tags; unique per pair on the diagram</span>
          <input
            className="properties-panel__input"
            inputMode="numeric"
            value={draftIndex}
            onChange={(e) => setDraftIndex(e.target.value)}
          />
        </label>
        <button type="button" className="properties-panel__btn" onClick={applyIndex}>
          Apply index to pair
        </button>
        <button
          type="button"
          className="properties-panel__btn properties-panel__btn--danger"
          onClick={() => void onDeletePair(data.pairId)}
        >
          Delete pair
        </button>
      </PropSection>
      <MakeModelSection data={data} patch={patch} />
    </div>
  )
}

// ─── Spatial group ─────────────────────────────────────────────────────────
// (Form component moved to SpatialGroupForm.tsx)
