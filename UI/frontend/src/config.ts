const fallbackHttpBase = typeof window !== 'undefined' ? window.location.origin : 'http://localhost:3000'

export const API_BASE_URL = (import.meta.env.VITE_API_BASE_URL as string | undefined) || fallbackHttpBase

const fallbackWsBase = fallbackHttpBase.startsWith('https://')
  ? fallbackHttpBase.replace('https://', 'wss://')
  : fallbackHttpBase.replace('http://', 'ws://')

export const WS_BASE_URL = (import.meta.env.VITE_WS_BASE_URL as string | undefined) || fallbackWsBase
