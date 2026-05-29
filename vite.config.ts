import fs from 'node:fs'
import path from 'node:path'

import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

function fixturesDevServer() {
  const fixturesRoot = path.join(process.cwd(), 'fixtures')
  return {
    name: 'serve-fixtures',
    configureServer(server: { middlewares: { use: (fn: (req: import('http').IncomingMessage, res: import('http').ServerResponse, next: () => void) => void) => void } }) {
      server.middlewares.use((req, res, next) => {
        const url = req.url?.split('?')[0] ?? ''
        if (!url.startsWith('/fixtures/')) {
          next()
          return
        }
        const rel = decodeURIComponent(url.slice('/fixtures/'.length))
        const filePath = path.resolve(fixturesRoot, rel)
        if (!filePath.startsWith(fixturesRoot)) {
          res.statusCode = 403
          res.end()
          return
        }
        fs.readFile(filePath, (err, data) => {
          if (err) {
            res.statusCode = 404
            res.end('Not found')
            return
          }
          res.setHeader('Content-Type', 'application/json')
          res.end(data)
        })
      })
    },
  }
}

// https://tauri.app/start/frontend/vite/
export default defineConfig({
  plugins: [react(), fixturesDevServer()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ['**/src-tauri/**'] },
  },
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    target: 'es2022',
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
})
