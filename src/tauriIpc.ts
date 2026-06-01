/**
 * Typed wrappers around Tauri IPC commands.
 *
 * Mutators return the full `DiagramState` `{ nodes, edges }` after Rust applies persistence/history
 * mutations. Pass `handleAttachmentUpdates` on `moveNode` / `updateDims` when handle centers change
 * so edge `data` keeps accurate attachment points. `saveDiagram` returns a JSON string;
 * `openDiagram` accepts that document shape.
 */
import { invoke } from '@tauri-apps/api/core'

import type {
  HAlign,
  HitTarget,
  PointPx,
  RectPx,
  SceneJson,
  ScenePatchJson,
  ScenePrimitive,
  SceneText,
  VAlign,
} from './canvas/sceneTypes'

export type {
  HAlign,
  HitTarget,
  PointPx,
  RectPx,
  SceneJson,
  ScenePatchJson,
  ScenePrimitive,
  SceneText,
  VAlign,
}

export type EdgeHandleAttachmentUpdate = {
  edgeId: string
  sourceHandleCenter?: RustXY
  targetHandleCenter?: RustXY
}

export interface RustXY {
  x: number
  y: number
}

export type WireCategory = 'audio' | 'video' | 'control' | 'network' | 'rf' | 'power' | 'default'

export interface BundleEdgeData {
  bundled?: true
  bundledEdgeIds?: string[]
  bundleLabel?: string
  bundledBy?: string
}

export interface FlowNode {
  id: string
  type: string
  position: { x: number; y: number }
  data: unknown
  width?: number
  height?: number
  zIndex?: number
}

export interface FlowEdge {
  id: string
  source: string
  target: string
  sourceHandle?: string
  targetHandle?: string
  type?: string
  data?: unknown
}

export interface DiagramState {
  nodes: FlowNode[]
  edges: FlowEdge[]
}

export interface Sheet {
  id: string
  name: string
  state: DiagramState
}

export interface EmbeddedPreset {
  id: string
  name: string
  nodeType: 'deviceV2' | 'avPlate'
  data: unknown
  sourceBasename?: string
}

export interface ProjectState {
  sheets: Sheet[]
  activeSheetId: string
  presetLibrary?: EmbeddedPreset[]
}

export interface NodeDimension {
  id: string
  width: number
  height: number
  position?: { x: number; y: number }
}

// ─── State ─────────────────────────────────────────────────────────────────

export const getProject = () => invoke<ProjectState>('get_project')

export const getState = () => invoke<DiagramState>('get_state')
export const getDiagramScene = () => invoke<SceneJson>('get_diagram_scene')
export const getDiagramScenePatch = (nodeId: string) =>
  invoke<ScenePatchJson>('get_diagram_scene_patch', { nodeId })
export const exportRevitDxf = () => invoke<string>('export_revit_dxf')
export const setState = (next: DiagramState) => invoke<DiagramState>('set_state', { next })

/** Replace diagram without recording undo (dirty checks, recovery flush). */
export const syncState = (next: DiagramState) => invoke<DiagramState>('sync_state', { next })

// ─── Nodes ─────────────────────────────────────────────────────────────────

export const addNode = (node: FlowNode) => invoke<DiagramState>('add_node', { node })

export const updateNode = (nodeId: string, data: unknown) =>
  invoke<DiagramState>('update_node', { nodeId, data })

export const updateGroupingZone = (
  nodeId: string,
  data: unknown,
  width: number,
  height: number,
  position: RustXY,
) =>
  invoke<DiagramState>('update_grouping_zone', {
    nodeId,
    data,
    width,
    height,
    position,
  })

/** Swap `type` while preserving node id and all edges. */
export const replaceNodeType = (nodeId: string, nodeType: string, data: unknown) =>
  invoke<DiagramState>('replace_node_type', { nodeId, nodeType, data })

export const moveNode = (
  nodeId: string,
  position: RustXY,
  handleAttachmentUpdates?: EdgeHandleAttachmentUpdate[] | null,
  isDragPreview?: boolean,
) =>
  invoke<DiagramState>('move_node', {
    nodeId,
    position,
    handleAttachmentUpdates: handleAttachmentUpdates ?? null,
    isDragPreview: isDragPreview ?? null,
  })

export const deleteNode = (nodeId: string) => invoke<DiagramState>('delete_node', { nodeId })

export const deleteEdge = (edgeId: string) => invoke<DiagramState>('delete_edge', { edgeId })

export interface SchematicEdgeConnect {
  source: string
  target: string
  sourceHandle?: string
  targetHandle?: string
}

export const addEdge = (connect: SchematicEdgeConnect) =>
  invoke<DiagramState>('add_edge', {
    source: connect.source,
    target: connect.target,
    sourceHandle: connect.sourceHandle ?? null,
    targetHandle: connect.targetHandle ?? null,
  })

export const getWireInnerChain = (edgeId: string) =>
  invoke<RustXY[] | null>('get_wire_inner_chain', { edgeId })

export const dragWireSegment = (
  edgeId: string,
  segmentIndex: number,
  orientation: 'h' | 'v',
  delta: RustXY,
  baseChain: RustXY[] | null | undefined,
  isDragPreview?: boolean,
) =>
  invoke<DiagramState>('drag_wire_segment', {
    edgeId,
    segmentIndex,
    orientation,
    delta,
    baseChain: baseChain ?? null,
    isDragPreview: isDragPreview ?? null,
  })

export const updateEdgeInnerCorners = (
  edgeId: string,
  innerCorners: RustXY[] | null,
  isDragPreview?: boolean,
) =>
  invoke<DiagramState>('update_edge_inner_corners', {
    edgeId,
    innerCorners,
    isDragPreview: isDragPreview ?? null,
  })

export interface NodeMove {
  nodeId: string
  position: RustXY
}

export const moveNodes = (
  moves: NodeMove[],
  handleAttachmentUpdates?: EdgeHandleAttachmentUpdate[] | null,
) =>
  invoke<DiagramState>('move_nodes', {
    moves,
    handleAttachmentUpdates: handleAttachmentUpdates ?? null,
  })

export const deleteNodes = (nodeIds: string[]) =>
  invoke<DiagramState>('delete_nodes', { nodeIds })

export const updateDims = (
  dims: NodeDimension[],
  handleAttachmentUpdates?: EdgeHandleAttachmentUpdate[] | null,
  isDragPreview?: boolean,
) =>
  invoke<DiagramState>('update_dims', {
    dims,
    handleAttachmentUpdates: handleAttachmentUpdates ?? null,
    isDragPreview: isDragPreview ?? null,
  })

// ─── History ───────────────────────────────────────────────────────────────

export const undo = () => invoke<DiagramState>('undo')
export const redo = () => invoke<DiagramState>('redo')

/** Revert an in-progress drag preview without recording undo. */
export const cancelDragPreview = () => invoke<DiagramState>('cancel_drag_preview')

// ─── Sheets ────────────────────────────────────────────────────────────────

export const addSheet = (name: string) => invoke<ProjectState>('add_sheet', { name })

export const removeSheet = (id: string) => invoke<ProjectState>('remove_sheet', { id })

export const renameSheet = (id: string, name: string) =>
  invoke<ProjectState>('rename_sheet', { id, name })

export const setActiveSheet = (id: string) => invoke<ProjectState>('set_active_sheet', { id })

// ─── Embedded presets (project file) ────────────────────────────────────────

export const addProjectPreset = (preset: EmbeddedPreset) =>
  invoke<ProjectState>('add_project_preset', { preset })

export const updateProjectPreset = (
  id: string,
  name: string,
  nodeType: string,
  data: unknown,
  sourceBasename?: string | null,
) =>
  invoke<ProjectState>('update_project_preset', {
    id,
    name,
    nodeType,
    data,
    sourceBasename: sourceBasename ?? null,
  })

export const removeProjectPreset = (id: string) =>
  invoke<ProjectState>('remove_project_preset', { id })

// ─── File I/O ───────────────────────────────────────────────────────────────

export const newDiagram = () => invoke<ProjectState>('new_diagram')

export const openDiagram = (json: string) => invoke<ProjectState>('open_diagram', { json })

/** Replace full project in Rust without pushing undo history. */
export const setProject = (project: ProjectState) => invoke<ProjectState>('set_project', { project })

export const saveDiagram = () => invoke<string>('save_diagram')

/** Compact JSON for baselines and dirty comparison. */
export const saveDiagramCompact = () => invoke<string>('save_diagram_compact')

// Desktop shell: quit gate + crash recovery snapshot (app data dir).
export const grantNextClose = () => invoke<void>('grant_next_close')

export const closeWindowAllowingGate = () => invoke<void>('close_window_allowing_gate')

export const writeRecoverySnapshot = (json: string) => invoke<void>('write_recovery_snapshot', { json })

export const readRecoverySnapshot = () => invoke<string | null>('read_recovery_snapshot')

export const clearRecoverySnapshot = () => invoke<void>('clear_recovery_snapshot')
