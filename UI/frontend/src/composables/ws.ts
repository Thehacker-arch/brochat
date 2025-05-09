import { ref } from 'vue'

let socket: WebSocket | null = null
const messages = ref<string[]>([])

export function useWebSocket(token: string) {
  function connect() {
    const wsUrl = `ws://192.168.1.35:3000/ws?token=${encodeURIComponent(token)}`
    socket = new WebSocket(wsUrl)

    socket.onopen = () => {
      console.log('Connected to WebSocket')
    }

    socket.onmessage = (event) => {
      messages.value.push(event.data)
    }

    socket.onclose = () => {
      console.log('WebSocket connection closed')
    }

    socket.onerror = (err) => {
      console.error('WebSocket error:', err)
    }
  }

  return { connect,  messages }
}


