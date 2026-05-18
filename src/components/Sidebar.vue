<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import type { MdFileEntry } from '../types'

const props = defineProps<{
  files: MdFileEntry[]
  allCount: number
  currentIndex: number
  searchQuery: string
}>()

const emit = defineEmits<{
  select: [index: number]
  search: [query: string]
  'change-folder': []
}>()

const searchInputRef = ref<HTMLInputElement | null>(null)
const listRef = ref<HTMLElement | null>(null)
let debounceTimer: ReturnType<typeof setTimeout> | null = null

/* ── Exposed methods for parent keyboard shortcuts ── */
defineExpose({
  focusSearch() {
    searchInputRef.value?.focus()
  },
  blurSearch() {
    searchInputRef.value?.blur()
  },
})

/* ── Search with debounce ───────────────────────── */
function onSearchInput(e: Event) {
  const value = (e.target as HTMLInputElement).value
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => {
    emit('search', value)
  }, 150)
}

/* ── Scroll active item into view ───────────────── */
watch(() => props.currentIndex, async () => {
  await nextTick()
  const active = listRef.value?.querySelector('.file-item.active')
  active?.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
})

/* ── Helpers ────────────────────────────────────── */
function dirPrefix(relativePath: string): string {
  const idx = relativePath.lastIndexOf('/')
  return idx > 0 ? relativePath.substring(0, idx + 1) : ''
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}
</script>

<template>
  <aside class="sidebar">
    <!-- Header -->
    <div class="sidebar-header">
      <div class="sidebar-title">
        <span class="label">FILES</span>
        <span class="count">
          {{ files.length }}<template v-if="files.length !== allCount"> / {{ allCount }}</template>
        </span>
      </div>
      <button class="btn-icon" title="Change folder" @click="emit('change-folder')">
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
      </button>
    </div>

    <!-- Search -->
    <div class="sidebar-search">
      <input
        ref="searchInputRef"
        type="text"
        placeholder="Search files... (Ctrl+K)"
        :value="searchQuery"
        @input="onSearchInput"
      />
    </div>

    <!-- File List -->
    <div ref="listRef" class="file-list">
      <div
        v-for="(file, idx) in files"
        :key="file.path"
        class="file-item"
        :class="{ active: idx === currentIndex }"
        :title="file.relative_path"
        @click="emit('select', idx)"
      >
        <span class="file-dir" v-if="dirPrefix(file.relative_path)">
          {{ dirPrefix(file.relative_path) }}
        </span>
        <span class="file-name">{{ file.name }}</span>
        <span class="file-size">{{ formatSize(file.size) }}</span>
      </div>

      <div v-if="files.length === 0" class="empty-state">
        No matching files
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  width: var(--sidebar-width);
  min-width: var(--sidebar-width);
  height: 100%;
  display: flex;
  flex-direction: column;
  border-right: 0.5px solid var(--border);
  background: var(--bg-raised);
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--gap-4) var(--gap-4) 0;
}

.sidebar-title {
  display: flex;
  align-items: center;
  gap: var(--gap-2);
}

.sidebar-title .label {
  font-family: var(--font-label);
  font-size: var(--text-micro);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: var(--letter-uppercase);
  color: var(--fg-muted);
}

.sidebar-title .count {
  font-family: var(--font-label);
  font-size: var(--text-xs);
  color: var(--fg-subtle);
}

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: 0.5px solid transparent;
  border-radius: var(--radius);
  background: transparent;
  color: var(--fg-subtle);
  cursor: pointer;
  transition: background-color var(--t-fast), border-color var(--t-fast), color var(--t-fast);
}
.btn-icon:hover {
  background: var(--surface-hover);
  border-color: var(--border-soft);
  color: var(--fg);
}

.sidebar-search {
  padding: var(--gap-3) var(--gap-4);
}

.sidebar-search input {
  width: 100%;
  padding: var(--gap-2) var(--gap-3);
  background: var(--bg);
  border: 0.5px solid var(--border);
  border-radius: var(--radius);
  font-family: var(--font-sans);
  font-size: var(--text-base);
  color: var(--fg);
  outline: none;
  transition: border-color var(--t-fast);
}
.sidebar-search input::placeholder {
  color: var(--fg-subtle);
}
.sidebar-search input:focus {
  border-color: var(--border-focus);
}

.file-list {
  flex: 1;
  overflow-y: auto;
  padding: 0 var(--gap-2) var(--gap-4);
}

.file-item {
  display: flex;
  flex-direction: column;
  gap: 1px;
  padding: 7px var(--gap-3);
  border-radius: var(--radius);
  cursor: pointer;
  transition: background-color var(--t-fast), border-color var(--t-fast);
  border: 0.5px solid transparent;
  position: relative;
}

.file-item:hover {
  background: var(--surface-hover);
}

.file-item.active {
  background: var(--surface-active);
  border-color: var(--border-soft);
}

.file-dir {
  font-family: var(--font-label);
  font-size: var(--text-micro);
  color: var(--fg-subtle);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.3;
}

.file-name {
  font-size: var(--text-base);
  font-weight: 500;
  color: var(--fg);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  line-height: 1.4;
}

.file-item.active .file-name {
  font-weight: 600;
}

.file-size {
  font-family: var(--font-label);
  font-size: var(--text-micro);
  color: var(--fg-subtle);
  line-height: 1.3;
}

.empty-state {
  padding: var(--gap-6) var(--gap-4);
  text-align: center;
  font-size: var(--text-base);
  color: var(--fg-subtle);
}
</style>
