<script setup>
import { ref, watch, nextTick } from 'vue'

const props = defineProps({
  logs: {
    type: Array,
    required: true
  },
  loading: {
    type: Boolean,
    default: false
  },
  title: {
    type: String,
    default: 'Live Requests'
  },
  showClear: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['select', 'clear'])

const autoScroll = ref(true)
const listRef = ref(null)
const newEntryIds = ref(new Set())

// Auto-scroll to top when new logs arrive
watch(() => props.logs.length, async (newLen, oldLen) => {
  if (newLen > oldLen && autoScroll.value && listRef.value) {
    // Mark new entries for animation
    const newCount = newLen - oldLen
    for (let i = 0; i < newCount; i++) {
      if (props.logs[i]) {
        newEntryIds.value.add(props.logs[i].request_id)
      }
    }
    // Remove animation class after animation completes
    setTimeout(() => {
      newEntryIds.value.clear()
    }, 500)

    await nextTick()
    listRef.value.scrollTop = 0
  }
})

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

function isNewEntry(requestId) {
  return newEntryIds.value.has(requestId)
}

function handleClear() {
  emit('clear')
}

function toggleAutoScroll() {
  autoScroll.value = !autoScroll.value
}
</script>

<template>
  <div class="log-stream">
    <div class="stream-header">
      <div class="header-left">
        <h2>{{ title }}</h2>
        <span class="log-count">{{ logs.length }} logs</span>
      </div>
      <div class="header-actions">
        <button
          v-if="showClear && logs.length > 0"
          class="action-btn"
          @click="handleClear"
          title="Clear logs"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6"></polyline>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
          </svg>
        </button>
        <button
          class="action-btn"
          :class="{ active: autoScroll }"
          @click="toggleAutoScroll"
          :title="autoScroll ? 'Auto-scroll enabled' : 'Auto-scroll disabled'"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 19V5M5 12l7-7 7 7"/>
          </svg>
        </button>
      </div>
    </div>

    <div class="stream-body" ref="listRef">
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
          :class="{ 'new-entry': isNewEntry(log.request_id) }"
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
  box-shadow: 0 2px 8px var(--shadow);
  border: 1px solid var(--border);
  animation: slideInUp var(--transition-base);
}

.stream-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-secondary);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.stream-header h2 {
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-primary);
}

.log-count {
  font-size: 0.75rem;
  color: var(--text-muted);
  background: var(--bg-tertiary);
  padding: 4px 10px;
  border-radius: 12px;
  font-weight: 500;
}

.header-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-tertiary);
  color: var(--text-muted);
  cursor: pointer;
  transition: all var(--transition-fast);
  font-size: 0;
}

.action-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
  border-color: var(--border-light);
  transform: translateY(-1px);
}

.action-btn:active {
  transform: translateY(0);
}

.action-btn.active {
  background: var(--accent);
  color: white;
  border-color: var(--accent-dark);
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.3);
}

.stream-body {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
}

.loading,
.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-muted);
  animation: fadeIn var(--transition-base);
}

.empty-icon {
  margin-bottom: 16px;
  opacity: 0.5;
  animation: bounce var(--transition-slow);
}

.empty p {
  margin-bottom: 8px;
  font-size: 0.95rem;
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
  transition: all var(--transition-fast);
  font-size: 0.875rem;
  line-height: 1.4;
}

.log-entry:hover {
  background: var(--bg-tertiary);
  transform: translateX(2px);
  box-shadow: 0 2px 6px var(--shadow);
}

.log-entry:active {
  transform: translateX(0);
}

.log-entry.new-entry {
  animation: slideInDown var(--transition-fast), highlight 0.6s ease-out;
}

@keyframes highlight {
  0% {
    background: rgba(59, 130, 246, 0.15);
    box-shadow: inset 0 0 8px rgba(59, 130, 246, 0.2);
  }
  100% {
    background: transparent;
    box-shadow: none;
  }
}

.time {
  color: var(--text-muted);
  flex-shrink: 0;
  font-size: 0.8125rem;
}

.method {
  width: 60px;
  flex-shrink: 0;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.method-get { color: var(--method-get); }
.method-post { color: var(--method-post); }
.method-put { color: var(--method-put); }
.method-patch { color: var(--method-patch); }
.method-delete { color: var(--method-delete); }
.method-head { color: var(--method-head); }
.method-options { color: var(--method-options); }

.path {
  flex: 1;
  color: var(--text-primary);
  min-width: 0;
  font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Fira Code', monospace;
}

.truncate {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.status {
  width: 40px;
  text-align: center;
  flex-shrink: 0;
  font-weight: 700;
}

.status-success { color: var(--status-2xx); }
.status-redirect { color: var(--status-3xx); }
.status-client-error { color: var(--status-4xx); }
.status-server-error { color: var(--status-5xx); }

.duration {
  width: 60px;
  text-align: right;
  color: var(--text-muted);
  flex-shrink: 0;
  font-size: 0.8125rem;
}

.mono {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
}

/* Responsive Design */
@media (max-width: 1024px) {
  .time {
    width: 50px;
  }

  .method {
    width: 50px;
    font-size: 0.75rem;
  }

  .status {
    width: 35px;
  }

  .duration {
    width: 50px;
  }
}

@media (max-width: 768px) {
  .log-entry {
    gap: 8px;
    padding: 8px 10px;
    font-size: 0.8125rem;
  }

  .stream-header {
    padding: 12px 16px;
  }

  .time {
    width: 40px;
    font-size: 0.75rem;
  }

  .method {
    width: 45px;
    font-size: 0.7rem;
  }

  .duration {
    width: 45px;
    font-size: 0.75rem;
  }
}
</style>
