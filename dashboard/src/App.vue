<script setup>
import { ref, computed } from 'vue'
import { useWebSocket } from './composables/useWebSocket'
import { useApi } from './composables/useApi'
import LogStream from './components/LogStream.vue'
import LogDetail from './components/LogDetail.vue'

const { logs, connected, connectionStatus } = useWebSocket()
const { fetchRecentLogs, loading } = useApi()

const selectedLog = ref(null)
const showDetail = ref(false)

// Load historical logs on mount
fetchRecentLogs(50).then(historicalLogs => {
  // Merge with existing logs, avoiding duplicates
  const existingIds = new Set(logs.value.map(l => l.request_id))
  const newLogs = historicalLogs.filter(l => !existingIds.has(l.request_id))
  logs.value.push(...newLogs)
  // Sort by timestamp descending
  logs.value.sort((a, b) => new Date(b.timestamp) - new Date(a.timestamp))
})

const selectLog = (log) => {
  selectedLog.value = log
  showDetail.value = true
}

const closeDetail = () => {
  showDetail.value = false
  selectedLog.value = null
}

const statusColor = computed(() => {
  if (connected.value) return 'var(--success)'
  return 'var(--error)'
})
</script>

<template>
  <header class="header">
    <div class="header-left">
      <h1>Endpoint <span class="accent">Logger</span></h1>
    </div>
    <div class="header-right">
      <div class="connection-status">
        <span class="status-dot" :style="{ background: statusColor }"></span>
        <span class="status-text">{{ connectionStatus }}</span>
      </div>
    </div>
  </header>

  <main class="main">
    <LogStream
      :logs="logs"
      :loading="loading"
      @select="selectLog"
    />
  </main>

  <LogDetail
    v-if="showDetail"
    :log="selectedLog"
    @close="closeDetail"
  />
</template>

<style scoped>
.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
}

.header h1 {
  font-size: 1.5rem;
  font-weight: 600;
}

.header .accent {
  color: var(--accent);
}

.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}

.connection-status {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.875rem;
  color: var(--text-secondary);
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.main {
  flex: 1;
  padding: 24px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}
</style>
