const isTauriRuntime =
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

/** Same host as API, for Tauri APK when env is missing (override with .env.production). */
const TAURI_DEFAULT_API = 'https://brochat.duckdns.org'

function deriveWsFromHttp(http: string): string {
  if (http.startsWith('https://')) return http.replace('https://', 'wss://')
  if (http.startsWith('http://')) return http.replace('http://', 'ws://')
  return http
}

const browserOrServerFallback =
  typeof window !== 'undefined' ? window.location.origin : 'http://localhost:3000'

const explicitApi = import.meta.env.VITE_API_BASE_URL as string | undefined
const explicitWs = import.meta.env.VITE_WS_BASE_URL as string | undefined

// Tauri WebView origin (e.g. https://tauri.localhost) is not your backend — do not use it for API.
export const API_BASE_URL =
  explicitApi ||
  (isTauriRuntime ? TAURI_DEFAULT_API : browserOrServerFallback)

export const WS_BASE_URL =
  explicitWs || deriveWsFromHttp(API_BASE_URL)

if (isTauriRuntime && !explicitApi && import.meta.env.PROD) {
  console.warn(
    '[BroChat] Using default API URL for Tauri. Set VITE_API_BASE_URL in .env.production for other hosts.'
  )
}
