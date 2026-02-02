import { ref, computed, onMounted, onUnmounted } from 'vue'

// Maximum logs to keep in memory (per EP-002-04 spec)
const MAX_LOGS = 500
// Auto-reconnect delay in milliseconds
const RECONNECT_DELAY = 2000

// Shared state - singleton pattern to avoid multiple WebSocket connections
// These refs are created once and shared across all components using this composable
const logs = ref([])
const connected = ref(false)
const reconnecting = ref(false)
const lastError = ref(null)
let ws = null
let reconnectTimeout = null
let autoReconnect = true
let activeSubscribers = 0

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

  lastError.value = null

  try {
    ws = new WebSocket(getWebSocketUrl())

    ws.onopen = () => {
      console.log('[WebSocket] Connected')
      connected.value = true
      reconnecting.value = false
      lastError.value = null
    }

    ws.onmessage = (event) => {
      try {
        const logEntry = JSON.parse(event.data)
        // Add to beginning of array (newest first)
        logs.value.unshift(logEntry)
        // Limit array size to MAX_LOGS
        if (logs.value.length > MAX_LOGS) {
          logs.value = logs.value.slice(0, MAX_LOGS)
        }
      } catch (e) {
        console.error('[WebSocket] Failed to parse message:', e)
      }
    }

    ws.onerror = (error) => {
      console.error('[WebSocket] Error:', error)
      lastError.value = 'Connection error'
    }

    ws.onclose = (event) => {
      console.log('[WebSocket] Disconnected', event.code, event.reason)
      connected.value = false
      ws = null
      if (autoReconnect && activeSubscribers > 0) {
        scheduleReconnect()
      }
    }
  } catch (error) {
    console.error('[WebSocket] Connection failed:', error)
    lastError.value = error.message || 'Connection failed'
    if (autoReconnect && activeSubscribers > 0) {
      scheduleReconnect()
    }
  }
}

function scheduleReconnect() {
  if (reconnectTimeout) return
  reconnecting.value = true
  reconnectTimeout = setTimeout(() => {
    reconnectTimeout = null
    connect()
  }, RECONNECT_DELAY)
}

function disconnect() {
  autoReconnect = false
  if (reconnectTimeout) {
    clearTimeout(reconnectTimeout)
    reconnectTimeout = null
  }
  reconnecting.value = false
  if (ws) {
    ws.close()
    ws = null
  }
  connected.value = false
}

function reconnect() {
  // Force reconnect - disconnect first then connect
  autoReconnect = true
  if (reconnectTimeout) {
    clearTimeout(reconnectTimeout)
    reconnectTimeout = null
  }
  if (ws) {
    ws.close()
    ws = null
  }
  reconnecting.value = false
  connect()
}

function clearLogs() {
  logs.value = []
}

/**
 * WebSocket composable with shared state
 * Multiple components can use this composable and they will all share
 * the same WebSocket connection and logs array.
 *
 * The connection is established when the first subscriber mounts
 * and disconnected when the last subscriber unmounts.
 */
export function useWebSocket() {
  onMounted(() => {
    activeSubscribers++
    if (activeSubscribers === 1) {
      // First subscriber - establish connection
      autoReconnect = true
      connect()
    }
  })

  onUnmounted(() => {
    activeSubscribers--
    if (activeSubscribers === 0) {
      // Last subscriber - close connection
      disconnect()
    }
  })

  return {
    // Reactive state (shared)
    logs,
    connected,
    reconnecting,
    connectionStatus,
    lastError,
    // Methods
    clearLogs,
    reconnect,
    disconnect
  }
}
