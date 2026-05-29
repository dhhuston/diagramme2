import { useCallback, useState } from 'react'

import { DiagramStage } from './canvas/DiagramStage'
import type { SceneJson } from './canvas/sceneTypes'
import {
  exportRevitDxf,
  getDiagramScene,
  openDiagram,
} from './tauriIpc'
import './App.css'

export default function App() {
  const [scene, setScene] = useState<SceneJson | null>(null)
  const [status, setStatus] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)

  const loadGoldenDiagram = useCallback(async () => {
    setBusy(true)
    setStatus(null)
    try {
      const json = await fetch('/fixtures/golden/Comp Gym F102A.diagramme').then((r) => {
        if (!r.ok) throw new Error(`fixture fetch ${r.status}`)
        return r.text()
      })
      await openDiagram(json)
      const next = await getDiagramScene()
      setScene(next)
      setStatus(`Loaded Comp Gym (${next.primitives.length} primitives)`)
    } catch (err) {
      setStatus(`Load failed: ${String(err)}`)
    } finally {
      setBusy(false)
    }
  }, [])

  const handleExportDxf = useCallback(async () => {
    setBusy(true)
    setStatus(null)
    try {
      const dxf = await exportRevitDxf()
      setStatus(`DXF exported (${dxf.length.toLocaleString()} chars)`)
    } catch (err) {
      setStatus(`Export failed: ${String(err)}`)
    } finally {
      setBusy(false)
    }
  }, [])

  return (
    <div className="app-shell">
      <header className="app-toolbar">
        <h1>Diagramme v2</h1>
        <div className="app-toolbar-actions">
          <button type="button" disabled={busy} onClick={() => void loadGoldenDiagram()}>
            Load Comp Gym
          </button>
          <button type="button" disabled={busy} onClick={() => void handleExportDxf()}>
            Export DXF
          </button>
        </div>
        {status ? <p className="app-status">{status}</p> : null}
      </header>
      <main className="app-canvas">
        {scene ? (
          <DiagramStage scene={scene} />
        ) : (
          <div className="app-placeholder">
            <p>Load Comp Gym to render the Rust scene on Konva.</p>
          </div>
        )}
      </main>
    </div>
  )
}
