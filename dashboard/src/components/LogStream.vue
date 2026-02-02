<script setup>
import { computed } from 'vue'

const props = defineProps({
  logs: {
    type: Array,
    required: true
  },
  loading: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['select'])

function formatTime(timestamp) {
  const date = new Date(timestamp)
  return date.toLocaleTimeString('en-US', { hour12: false })
}

function getMethodClass(method) {
  const classes = {
    GET: 'method-get',
    POST: 'method-post',
    PUT: 'method-put',
    PATCH: 'method-patch',
    DELETE: 'method-delete'
  }
  return classes[method] || ''
}

function getStatusClass(status) {
  if (status >= 200 && status < 300) return 'status-success'
  if (status >= 300 && status < 400) return 'status-redirect'
  if (status >= 400 && status < 500) return 'status-client-error'
  if (status >= 500) return 'status-server-error'
  return ''
}

function formatPath(log) {
  let path = log.path
  if (log.query_string) {
    path += '?' + log.query_string
  }
  return path
}
</script>

<template>
  <div class="log-stream">
    <div class="stream-header">
      <h2>Live Requests</h2>
      <span class="log-count">{{ logs.length }} logs</span>
    </div>

    <div class="stream-body">
      <div v-if="loading && logs.length === 0" class="loading">
        Loading logs...
      </div>

      <div v-else-if="logs.length === 0" class="empty">
        <div class="empty-icon">
          <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
          </svg>
        </div>
        <p>Waiting for requests...</p>
        <p class="empty-hint">Make HTTP requests through the proxy to see them here</p>
      </div>

      <div v-else class="log-list">
        <div
          v-for="log in logs"
          :key="log.request_id"
          class="log-entry"
          @click="emit('select', log)"
        >
          <span class="time mono">{{ formatTime(log.timestamp) }}</span>
          <span class="method mono" :class="getMethodClass(log.method)">{{ log.method }}</span>
          <span class="path mono truncate">{{ formatPath(log) }}</span>
          <span class="status mono" :class="getStatusClass(log.status_code)">{{ log.status_code }}</span>
          <span class="duration mono">{{ log.duration_ms }}ms</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.log-stream {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  border-radius: 12px;
  overflow: hidden;
}

.stream-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
}

.stream-header h2 {
  font-size: 1rem;
  font-weight: 600;
}

.log-count {
  font-size: 0.75rem;
  color: var(--text-muted);
  background: var(--bg-tertiary);
  padding: 4px 10px;
  border-radius: 12px;
}

.stream-body {
  flex: 1;
  overflow-y: auto;
}

.loading,
.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-muted);
}

.empty-icon {
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty p {
  margin-bottom: 8px;
}

.empty-hint {
  font-size: 0.875rem;
  opacity: 0.7;
}

.log-list {
  padding: 8px;
}

.log-entry {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s;
  font-size: 0.875rem;
}

.log-entry:hover {
  background: var(--bg-tertiary);
}

.time {
  color: var(--text-muted);
  flex-shrink: 0;
}

.method {
  width: 60px;
  flex-shrink: 0;
  font-weight: 600;
}

.method-get { color: var(--success); }
.method-post { color: var(--accent); }
.method-put { color: var(--warning); }
.method-patch { color: #a855f7; }
.method-delete { color: var(--error); }

.path {
  flex: 1;
  color: var(--text-primary);
  min-width: 0;
}

.status {
  width: 40px;
  text-align: center;
  flex-shrink: 0;
  font-weight: 600;
}

.status-success { color: var(--success); }
.status-redirect { color: var(--accent); }
.status-client-error { color: var(--warning); }
.status-server-error { color: var(--error); }

.duration {
  width: 60px;
  text-align: right;
  color: var(--text-muted);
  flex-shrink: 0;
}
</style>
