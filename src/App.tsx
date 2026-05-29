import { useState } from 'react'
import './App.css'
import { exportRevitDxf } from './tauriIpc'

export default function App() {
  const [exportStatus, setExportStatus] = useState<string | null>(null)

  const handleExportDxf = async () => {
    try {
      const dxf = await exportRevitDxf()
      setExportStatus(`DXF exported (${dxf.length.toLocaleString()} chars)`)
    } catch (err) {
      setExportStatus(`Export failed: ${String(err)}`)
    }
  }

  return (
    <main className="app">
      <h1>Diagramme v2</h1>
      <p>Scaffold ready — Konva canvas and Rust core coming next.</p>
      <button type="button" onClick={handleExportDxf}>
        Export DXF
      </button>
      {exportStatus ? <p>{exportStatus}</p> : null}
    </main>
  )
}
