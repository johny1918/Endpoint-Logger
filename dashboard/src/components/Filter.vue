<script setup>
import { computed } from 'vue'

const props = defineProps({
  logs: {
    type: Array,
    required: true
  }
})

const emit = defineEmits(['update:filters'])

const methods = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'HEAD', 'OPTIONS']
const statusCodes = [200, 201, 301, 302, 400, 401, 403, 404, 500, 502, 503]

const filters = {
  methods: new Set(),
  statuses: new Set(),
  pathPattern: ''
}

function toggleMethod(method) {
  if (filters.methods.has(method)) {
    filters.methods.delete(method)
  } else {
    filters.methods.add(method)
  }
  emitFilters()
}

function toggleStatus(status) {
  if (filters.statuses.has(status)) {
    filters.statuses.delete(status)
  } else {
    filters.statuses.add(status)
  }
  emitFilters()
}

function updatePathPattern(value) {
  filters.pathPattern = value
  emitFilters()
}

function clearFilters() {
  filters.methods.clear()
  filters.statuses.clear()
  filters.pathPattern = ''
  emitFilters()
}

function emitFilters() {
  emit('update:filters', {
    methods: Array.from(filters.methods),
    statuses: Array.from(filters.statuses),
    pathPattern: filters.pathPattern
  })
}

const filteredLogs = computed(() => {
  return props.logs.filter(log => {
    // Filter by methods
    if (filters.methods.size > 0 && !filters.methods.has(log.method)) {
      return false
    }

    // Filter by status codes
    if (filters.statuses.size > 0 && !filters.statuses.has(log.status_code)) {
      return false
    }

    // Filter by path pattern
    if (filters.pathPattern.trim()) {
      const pattern = filters.pathPattern.toLowerCase()
      const path = log.path.toLowerCase()
      if (!path.includes(pattern)) {
        return false
      }
    }

    return true
  })
})

const isFiltered = computed(() => {
  return filters.methods.size > 0 || filters.statuses.size > 0 || filters.pathPattern.trim().length > 0
})

const resultCount = computed(() => filteredLogs.value.length)
</script>

<template>
  <div class="filter-container">
    <div class="filter-panel">
      <!-- Path Pattern Filter -->
      <div class="filter-section">
        <label class="filter-label">Path Pattern</label>
        <input
          type="text"
          class="filter-input"
          placeholder="e.g., /api, /users"
          :value="filters.pathPattern"
          @input="updatePathPattern($event.target.value)"
        />
      </div>

      <!-- HTTP Methods Filter -->
      <div class="filter-section">
        <label class="filter-label">Methods</label>
        <div class="filter-chips">
          <button
            v-for="method in methods"
            :key="method"
            class="filter-chip"
            :class="[
              'method-' + method.toLowerCase(),
              { active: filters.methods.has(method) }
            ]"
            @click="toggleMethod(method)"
          >
            {{ method }}
          </button>
        </div>
      </div>

      <!-- Status Codes Filter -->
      <div class="filter-section">
        <label class="filter-label">Status Codes</label>
        <div class="filter-chips">
          <button
            v-for="status in statusCodes"
            :key="status"
            class="filter-chip status-chip"
            :class="[
              'status-' + Math.floor(status / 100) * 100,
              { active: filters.statuses.has(status) }
            ]"
            @click="toggleStatus(status)"
          >
            {{ status }}
          </button>
        </div>
      </div>

      <!-- Actions -->
      <div class="filter-actions">
        <button
          v-if="isFiltered"
          class="clear-btn"
          @click="clearFilters"
        >
          Clear Filters
        </button>
        <div class="result-count">
          {{ resultCount }} result{{ resultCount !== 1 ? 's' : '' }}
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.filter-container {
  background: var(--bg-secondary);
  border-radius: 12px;
  overflow: hidden;
  margin-bottom: 16px;
}

.filter-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 16px 20px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border);
  border-radius: 12px 12px 0 0;
  margin-bottom: 12px;
  box-shadow: 0 2px 8px var(--shadow);
  animation: slideInDown var(--transition-base);
}

.filter-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.filter-label {
  font-size: 0.75rem;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.filter-input {
  padding: 10px 12px;
  background: var(--bg-primary);
  border: 1px solid var(--border);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 0.875rem;
  font-family: inherit;
  transition: all var(--transition-fast);
}

.filter-input:focus {
  outline: none;
  border-color: var(--accent);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
  background: var(--bg-tertiary);
}

.filter-input::placeholder {
  color: var(--text-muted);
}

.filter-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.filter-chip {
  padding: 7px 14px;
  background: var(--bg-tertiary);
  border: 1px solid var(--border);
  border-radius: 20px;
  font-size: 0.8125rem;
  font-weight: 600;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.filter-chip:hover {
  border-color: var(--accent);
  color: var(--accent);
  transform: translateY(-2px);
  box-shadow: 0 2px 6px var(--shadow);
}

.filter-chip.active {
  background: var(--accent);
  border-color: var(--accent);
  color: white;
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.4);
}

/* Method colors */
.filter-chip.method-get { color: var(--success); }
.filter-chip.method-get.active { background: var(--success); border-color: var(--success); color: white; }

.filter-chip.method-post { color: var(--accent); }
.filter-chip.method-post.active { background: var(--accent); border-color: var(--accent); color: white; }

.filter-chip.method-put { color: var(--warning); }
.filter-chip.method-put.active { background: var(--warning); border-color: var(--warning); color: white; }

.filter-chip.method-patch { color: #a855f7; }
.filter-chip.method-patch.active { background: #a855f7; border-color: #a855f7; color: white; }

.filter-chip.method-delete { color: var(--error); }
.filter-chip.method-delete.active { background: var(--error); border-color: var(--error); color: white; }

.filter-chip.method-head { color: var(--text-secondary); }
.filter-chip.method-head.active { background: var(--text-secondary); border-color: var(--text-secondary); color: white; }

.filter-chip.method-options { color: var(--text-secondary); }
.filter-chip.method-options.active { background: var(--text-secondary); border-color: var(--text-secondary); color: white; }

/* Status code colors */
.filter-chip.status-200 { color: var(--success); }
.filter-chip.status-200.active { background: var(--success); border-color: var(--success); color: white; }

.filter-chip.status-300 { color: var(--accent); }
.filter-chip.status-300.active { background: var(--accent); border-color: var(--accent); color: white; }

.filter-chip.status-400 { color: var(--warning); }
.filter-chip.status-400.active { background: var(--warning); border-color: var(--warning); color: white; }

.filter-chip.status-500 { color: var(--error); }
.filter-chip.status-500.active { background: var(--error); border-color: var(--error); color: white; }

.status-chip {
  min-width: 50px;
  text-align: center;
}

.filter-actions {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding-top: 12px;
  border-top: 1px solid var(--border);
  margin-top: 4px;
}

.clear-btn {
  padding: 8px 16px;
  background: var(--error);
  border: 1px solid var(--error-dark);
  border-radius: 6px;
  color: white;
  font-size: 0.8125rem;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-fast);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.clear-btn:hover {
  background: var(--error-dark);
  border-color: var(--error-dark);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.3);
}

.result-count {
  font-size: 0.875rem;
  color: var(--text-secondary);
  font-weight: 600;
}
</style>
