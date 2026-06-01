/** True when running inside the Tauri desktop shell. */
export function isTauriApp(): boolean {
  if (typeof window === 'undefined') return false
  const w = window as Window & { __TAURI_INTERNALS__?: unknown; __TAURI__?: unknown }
  if (w.__TAURI_INTERNALS__ !== undefined || w.__TAURI__ !== undefined) return true
  return Boolean(import.meta.env.TAURI_ENV_PLATFORM)
}

export function isMacOS(): boolean {
  if (typeof navigator === 'undefined') return false
  return /Mac|iPhone|iPad|iPod/.test(navigator.platform ?? navigator.userAgent)
}
