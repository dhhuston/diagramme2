import { listen } from '@tauri-apps/api/event'

import type { AppMenuCommand } from './appMenuCommands'
import { isAppMenuCommand } from './appMenuCommands'
import { isTauriApp } from './utils/isTauri'

type MenuHandler = (command: AppMenuCommand) => void

const MENU_DEDUPE_MS = 400

let handler: MenuHandler | null = null
let unlisten: (() => void) | null = null
let listenStarted = false
let lastDispatch: { command: AppMenuCommand; at: number } | null = null

function teardownMenuListener(): void {
  unlisten?.()
  unlisten = null
  listenStarted = false
  lastDispatch = null
}

function ensureMenuListener(): void {
  if (!isTauriApp() || listenStarted) return
  listenStarted = true
  void listen<string>('app-menu-command', (event) => {
    const id = event.payload
    if (!isAppMenuCommand(id)) return
    const now = Date.now()
    if (
      lastDispatch &&
      lastDispatch.command === id &&
      now - lastDispatch.at < MENU_DEDUPE_MS
    ) {
      return
    }
    lastDispatch = { command: id, at: now }
    handler?.(id)
  })
    .then((fn) => {
      unlisten = fn
    })
    .catch(() => {
      listenStarted = false
    })
}

/** Register the active menu handler (at most one Tauri listener for the app lifetime). */
export function setAppMenuCommandHandler(next: MenuHandler | null): void {
  handler = next
  if (next) ensureMenuListener()
}

if (import.meta.hot) {
  import.meta.hot.dispose(() => {
    handler = null
    teardownMenuListener()
  })
}
