import { useCallback, useMemo, type ReactNode } from 'react'

import { buildAppMenus } from './appMenu'
import { AppMenuBar } from './components/AppMenuBar'
import { CanvasEmptyState } from './components/CanvasEmptyState'
import { LeftPalette, type PaletteNodeActions } from './components/LeftPalette'
import { PresetToolbox } from './components/PresetToolbox'
import { PropertiesPanelFrame } from './components/PropertiesPanelFrame'
import { PropertiesPlaceholder } from './components/PropertiesPlaceholder'
import { TabStrip } from './components/TabStrip'
import type { HitTarget } from './canvas/sceneTypes'
import { useCanvasPreferences } from './hooks/useCanvasPreferences'
import { useNativeAppMenu } from './hooks/useNativeAppMenu'
import { useProject } from './hooks/useProject'
import type { AppMenuCommand } from './appMenuCommands'
import {
  addSheet,
  removeSheet,
  renameSheet,
  setActiveSheet,
  type ProjectState,
} from './tauriIpc'
import { isTauriApp } from './utils/isTauri'

import './App.css'

const noop = () => {}

const disabledPaletteActions: PaletteNodeActions = {
  onAddDevice: noop,
  onAddAvPlate: noop,
  onAddMicBlock: noop,
  onAddSpeakerBlock: noop,
  onAddVolumeControl: noop,
  onAddAntennaSymbol: noop,
  onAddLppPatchPanel: noop,
  onAddDppPatchPanel: noop,
  onAddMlpPatchPanel: noop,
  onAddVpbPatchPanel: noop,
  onAddTextBlock: noop,
  onAddFlyoffNote: noop,
  onAddWiretagPair: noop,
  onAddGroupingZone: noop,
}

export type AppShellProps = {
  scene: import('./canvas/sceneTypes').SceneJson | null
  selectedHit: HitTarget | null
  status: string | null
  busy?: boolean
  canvas: ReactNode
  onExportDxf: () => void | Promise<void>
  onUndo: () => void | Promise<void>
  onRedo: () => void | Promise<void>
  onRefreshScene: () => void | Promise<void>
  onLoadGoldenDiagram?: () => void | Promise<void>
  onLoadCafeteriaDiagram?: () => void | Promise<void>
  onLoadSplitFaceDemoDiagram?: () => void | Promise<void>
  onClearSelection?: () => void
  onMenuUnavailable?: (command: AppMenuCommand) => void
}

export function AppShell({
  scene,
  selectedHit,
  status,
  busy: _busy,
  canvas,
  onExportDxf,
  onUndo,
  onRedo,
  onRefreshScene,
  onLoadGoldenDiagram,
  onLoadCafeteriaDiagram,
  onLoadSplitFaceDemoDiagram,
  onClearSelection,
  onMenuUnavailable,
}: AppShellProps) {
  const {
    focusMode,
    showEmptyHint,
    setShowEmptyHint,
    alignmentGuides,
    toggleFocusMode,
    toggleAlignmentGuides,
    toggleWiringMode,
    wiringMode,
  } = useCanvasPreferences()

  const { project, setProject, refreshProject } = useProject()

  const wrapDiagramLoad = useCallback(
    (load?: () => void | Promise<void>) => {
      if (!load) return undefined
      return async () => {
        await load()
        const next = await refreshProject()
        setProject(next)
      }
    },
    [refreshProject, setProject],
  )

  const handleLoadGoldenDiagram = useMemo(
    () => wrapDiagramLoad(onLoadGoldenDiagram),
    [onLoadGoldenDiagram, wrapDiagramLoad],
  )
  const handleLoadCafeteriaDiagram = useMemo(
    () => wrapDiagramLoad(onLoadCafeteriaDiagram),
    [onLoadCafeteriaDiagram, wrapDiagramLoad],
  )
  const handleLoadSplitFaceDemoDiagram = useMemo(
    () => wrapDiagramLoad(onLoadSplitFaceDemoDiagram),
    [onLoadSplitFaceDemoDiagram, wrapDiagramLoad],
  )

  useNativeAppMenu({
    onExportDxf,
    onUndo,
    onRedo,
    onLoadGoldenDiagram: handleLoadGoldenDiagram,
    toggleWiringMode,
    toggleFocusMode,
    toggleAlignmentGuides,
    onUnavailable: onMenuUnavailable,
  })

  const menus = useMemo(
    () =>
      buildAppMenus({
        onExportDxf: () => void onExportDxf(),
        onUndo: () => void onUndo(),
        onRedo: () => void onRedo(),
        toggleWiringMode,
        toggleFocusMode,
        toggleAlignmentGuides,
        onLoadGoldenDiagram: handleLoadGoldenDiagram
          ? () => void handleLoadGoldenDiagram()
          : undefined,
        onLoadCafeteriaDiagram: handleLoadCafeteriaDiagram
          ? () => void handleLoadCafeteriaDiagram()
          : undefined,
        onLoadSplitFaceDemoDiagram: handleLoadSplitFaceDemoDiagram
          ? () => void handleLoadSplitFaceDemoDiagram()
          : undefined,
        wiringMode,
        focusMode,
        alignmentGuides,
      }),
    [
      alignmentGuides,
      focusMode,
      handleLoadCafeteriaDiagram,
      handleLoadGoldenDiagram,
      handleLoadSplitFaceDemoDiagram,
      onExportDxf,
      onRedo,
      onUndo,
      toggleAlignmentGuides,
      toggleFocusMode,
      toggleWiringMode,
      wiringMode,
    ],
  )

  const tabs = useMemo(
    () => project?.sheets.map((s) => ({ id: s.id, name: s.name })) ?? [],
    [project],
  )

  const handleSheetSwitch = useCallback(
    async (id: string) => {
      if (!project || id === project.activeSheetId) return
      const next = await setActiveSheet(id)
      setProject(next)
      await onRefreshScene()
    },
    [onRefreshScene, project, setProject],
  )

  const handleSheetAdd = useCallback(async () => {
    const name = `Sheet ${(project?.sheets.length ?? 0) + 1}`
    const next = await addSheet(name)
    setProject(next)
    await onRefreshScene()
  }, [onRefreshScene, project?.sheets.length, setProject])

  const handleSheetRemove = useCallback(
    async (id: string) => {
      const next = await removeSheet(id)
      setProject(next)
      await onRefreshScene()
    },
    [onRefreshScene, setProject],
  )

  const handleSheetRename = useCallback(
    async (id: string, newName: string) => {
      const next = await renameSheet(id, newName)
      setProject(next)
    },
    [setProject],
  )

  const handleProjectUpdated = useCallback((p: ProjectState) => {
    setProject(p)
  }, [setProject])

  const propsOpen = Boolean(selectedHit)
  const nodeLabel = selectedHit?.node_id ?? selectedHit?.id ?? 'Selection'

  const showEmptyState = !scene && showEmptyHint

  const layoutClass = `app-layout${focusMode ? ' app-layout--focus' : ''}`

  return (
    <div className="app-shell">
      <div className={layoutClass}>
        <div className="app-main-column">
          {!isTauriApp() && menus.length > 0 ? <AppMenuBar menus={menus} /> : null}
          <div className="app-main-content">
            <LeftPalette nodeActions={disabledPaletteActions} disabled />
            <main className={`app-canvas${wiringMode ? ' app-canvas--wiring' : ''}`}>
              {scene ? (
                <div className="diagram-stage-host">{canvas}</div>
              ) : showEmptyState ? (
                <CanvasEmptyState
                  onDismiss={() => setShowEmptyHint(false)}
                  onDismissForever={() => setShowEmptyHint(false)}
                />
              ) : (
                <div className="diagram-stage-host" />
              )}
              {project && tabs.length > 0 ? (
                <TabStrip
                  tabs={tabs}
                  activeId={project.activeSheetId}
                  onSwitch={(id) => void handleSheetSwitch(id)}
                  onAdd={() => void handleSheetAdd()}
                  onRemove={(id) => void handleSheetRemove(id)}
                  onRename={(id, name) => void handleSheetRename(id, name)}
                />
              ) : null}
              {status ? <div className="canvas-status">{status}</div> : null}
            </main>
            <PropertiesPanelFrame
              open={propsOpen}
              nodeLabel={nodeLabel}
              onClose={() => onClearSelection?.()}
              onDeleteNode={noop}
              canDelete={false}
            >
              <PropertiesPlaceholder selection={selectedHit} />
            </PropertiesPanelFrame>
          </div>
          <PresetToolbox
            presets={project?.presetLibrary ?? []}
            onProjectUpdated={handleProjectUpdated}
            onInsertPreset={noop}
            getDefaultInsertFlowPosition={() => ({ x: 0, y: 0 })}
            canSaveSelectionAsPreset={false}
            onSaveSelectionAsPreset={noop}
            shellDisabled
          />
        </div>
      </div>
      {focusMode ? (
        <button type="button" className="focus-hud" onClick={toggleFocusMode}>
          Focus mode <kbd>F</kbd>
        </button>
      ) : null}
    </div>
  )
}
