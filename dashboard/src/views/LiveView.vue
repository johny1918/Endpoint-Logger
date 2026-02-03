<script setup>
import { ref, computed, onMounted } from 'vue'
import { useWebSocket } from '../composables/useWebSocket'
import { useApi } from '../composables/useApi'
import LogStream from '../components/LogStream.vue'
import LogDetail from '../components/LogDetail.vue'
import Filter from '../components/Filter.vue'

const { logs, connected, clearLogs } = useWebSocket()
const { fetchRecentLogs, loading, error } = useApi()

const selectedLog = ref(null)
const showDetail = ref(false)
const loadError = ref(null)
const historicalLogsLoaded = ref(false)
const filters = ref({
  methods: [],
  statuses: [],
  pathPattern: ''
})

/**
 * Load historical logs on component mount
 * Merges with WebSocket logs and avoids duplicates
 */
async function loadHistoricalLogs() {
  try {
    loadError.value = null
    const historicalLogs = await fetchRecentLogs(50)
    
    if (!historicalLogs || historicalLogs.length === 0) {
      historicalLogsLoaded.value = true
      return
    }

    // Avoid duplicate logs by checking request_id
    const existingIds = new Set(logs.value.map(l => l.request_id))
    const newLogs = historicalLogs.filter(l => !existingIds.has(l.request_id))
    
    // Add new logs and maintain sorted order (newest first)
    if (newLogs.length > 0) {
      logs.value.unshift(...newLogs)
      // Sort by timestamp DESC to ensure newest logs are first
      logs.value.sort((a, b) => b.timestamp - a.timestamp)
    }
    
    historicalLogsLoaded.value = true
  } catch (e) {
    console.error('[LiveView] Failed to load historical logs:', e)
    loadError.value = 'Failed to load historical logs. WebSocket will receive new entries as they arrive.'
    historicalLogsLoaded.value = true
  }
}

// Load historical logs when component mounts
onMounted(() => {
  loadHistoricalLogs()
})

const filteredLogs = computed(() => {
  return logs.value.filter(log => {
    // Filter by methods
    if (filters.value.methods.length > 0 && !filters.value.methods.includes(log.method)) {
      return false
    }

    // Filter by status codes
    if (filters.value.statuses.length > 0 && !filters.value.statuses.includes(log.status_code)) {
      return false
    }

    // Filter by path pattern
    if (filters.value.pathPattern.trim()) {
      const pattern = filters.value.pathPattern.toLowerCase()
      const path = log.path.toLowerCase()
      if (!path.includes(pattern)) {
        return false
      }
    }

    return true
  })
})

/**
 * Check if initial loading is complete
 * Used to show loading spinner
 */
const isInitialLoading = computed(() => {
  return loading.value && !historicalLogsLoaded.value
})

const selectLog = (log) => {
  selectedLog.value = log
  showDetail.value = true
}

const closeDetail = () => {
  showDetail.value = false
  selectedLog.value = null
}

const handleClear = () => {
  clearLogs()
}

const updateFilters = (newFilters) => {
  filters.value = newFilters
}
</script>

<template>
  <div class="live-view">
    <!-- Loading indicator for initial historical log fetch -->
    <div v-if="isInitialLoading" class="loading-overlay">
      <div class="spinner"></div>
      <p>Loading historical logs...</p>
    </div>

    <!-- Error message for failed historical load -->
    <div v-if="loadError && historicalLogsLoaded" class="error-banner">
      <span class="error-icon">⚠️</span>
      {{ loadError }}
    </div>

    <!-- Filter component -->
    <Filter
      :logs="logs"
      @update:filters="updateFilters"
    />

    <!-- Log stream with live updates -->
    <LogStream
      :logs="filteredLogs"
      :connected="connected"
      :show-clear="true"
      title="Live Requests"
      @select="selectLog"
      @clear="handleClear"
    />

    <!-- Log detail modal -->
    <LogDetail
      v-if="showDetail"
      :log="selectedLog"
      @close="closeDetail"
    />
  </div>
</template>

<style scoped>
.live-view {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
}

.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.95);
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  gap: 20px;
  z-index: 100;
  border-radius: 4px;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 4px solid #f0f0f0;
  border-top: 4px solid #0066cc;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}

.loading-overlay p {
  font-size: 14px;
  color: #666;
  margin: 0;
}

.error-banner {
  background: #fff3cd;
  border: 1px solid #ffc107;
  border-radius: 4px;
  padding: 12px 16px;
  margin: 8px 8px 0 8px;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: #856404;
}

.error-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}
</style>
