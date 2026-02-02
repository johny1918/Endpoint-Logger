import { ref, onMounted, onUnmounted, computed } from 'vue'

const MAX_LOGS = 200

export function useWebSocket() {
  const logs = ref([])
  const connected = ref(false)
  const reconnecting = ref(false)
  let ws = null
  let reconnectTimeout = null

  const connectionStatus = computed(() => {
    if (connected.value) return 'Connected'
    if (reconnecting.value) return 'Reconnecting...'
    return 'Disconnected'
  })

  function getWebSocketUrl() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    return `${protocol}//${window.location.host}/ws`
  }

  function connect() {
    if (ws && ws.readyState === WebSocket.OPEN) return

    try {
      ws = new WebSocket(getWebSocketUrl())

      ws.onopen = () => {
        console.log('[WebSocket] Connected')
        connected.value = true
        reconnecting.value = false
      }

      ws.onmessage = (event) => {
        try {
          const logEntry = JSON.parse(event.data)
          // Add to beginning of array (newest first)
          logs.value.unshift(logEntry)
          // Limit array size
          if (logs.value.length > MAX_LOGS) {
            logs.value = logs.value.slice(0, MAX_LOGS)
          }
        } catch (e) {
          console.error('[WebSocket] Failed to parse message:', e)
        }
      }

      ws.onerror = (error) => {
        console.error('[WebSocket] Error:', error)
      }

      ws.onclose = () => {
        console.log('[WebSocket] Disconnected')
        connected.value = false
        ws = null
        scheduleReconnect()
      }
    } catch (error) {
      console.error('[WebSocket] Connection failed:', error)
      scheduleReconnect()
    }
  }

  function scheduleReconnect() {
    if (reconnectTimeout) return
    reconnecting.value = true
    reconnectTimeout = setTimeout(() => {
      reconnectTimeout = null
      connect()
    }, 2000)
  }

  function disconnect() {
    if (reconnectTimeout) {
      clearTimeout(reconnectTimeout)
      reconnectTimeout = null
    }
    if (ws) {
      ws.close()
      ws = null
    }
  }

  function clearLogs() {
    logs.value = []
  }

  onMounted(() => {
    connect()
  })

  onUnmounted(() => {
    disconnect()
  })

  return {
    logs,
    connected,
    reconnecting,
    connectionStatus,
    clearLogs
  }
}
