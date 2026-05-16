<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import type { MdFileEntry } from '../types'
import {
  selectFolder,
  scanMdFiles,
  saveLastDir,
} from '../composables/useFileSystem'
import Sidebar from './Sidebar.vue'
import ContentView from './ContentView.vue'

const props = defineProps<{
  files: MdFileEntry[]
  currentDir: string
  initialFilePath: string | null
}>()

const emit = defineEmits<{
  'change-folder': []
  'folder-opened': [dir: string, files: MdFileEntry[]]
}>()

/* ── State ──────────────────────────────────────── */
const searchQuery = ref('')
const currentIndex = ref(0)
const sidebarRef = ref<InstanceType<typeof Sidebar> | null>(null)

const filteredFiles = computed<MdFileEntry[]>(() => {
  const q = searchQuery.value.toLowerCase().trim()
  if (!q) return props.files
  return props.files.filter(f => f.relative_path.toLowerCase().includes(q))
})

const currentFile = computed<MdFileEntry | null>(() => {
  return filteredFiles.value[currentIndex.value] ?? null
})

/* ── Navigate to a specific file by its absolute path ── */
function navigateToFile(filePath: string) {
  const norm = filePath.replace(/\\/g, '/')
  const idx = filteredFiles.value.findIndex(
    f => f.path.replace(/\\/g, '/') === norm
  )
  if (idx >= 0) {
    currentIndex.value = idx
  }
}

/* ── React to initialFilePath changes (double-click / Open With) ── */
watch(
  () => props.initialFilePath,
  (newPath) => {
    if (newPath) {
      searchQuery.value = ''            // clear any active search
      // Wait a tick so filteredFiles recalculates without filter
      setTimeout(() => navigateToFile(newPath), 0)
    }
  },
  { immediate: true }
)

/* ── Navigation ─────────────────────────────────── */
function navigateFile(delta: number) {
  const len = filteredFiles.value.length
  if (len === 0) return
  const next = currentIndex.value + delta
  currentIndex.value = Math.max(0, Math.min(len - 1, next))
}

function selectFile(index: number) {
  currentIndex.value = index
}

function handleSearch(query: string) {
  searchQuery.value = query
  // Try to keep the same file selected
  if (currentFile.value) {
    const idx = filteredFiles.value.findIndex(
      f => f.path === currentFile.value!.path
    )
    currentIndex.value = idx >= 0 ? idx : 0
  } else {
    currentIndex.value = 0
  }
}

/* ── Change Folder ──────────────────────────────── */
async function handleChangeFolder() {
  const dir = await selectFolder()
  if (!dir) return
  try {
    const mdFiles = await scanMdFiles(dir)
    if (mdFiles.length === 0) return
    saveLastDir(dir)
    searchQuery.value = ''
    currentIndex.value = 0
    emit('folder-opened', dir, mdFiles)
  } catch {
    // silently ignore
  }
}

/* ── Keyboard Shortcuts ─────────────────────────── */
function handleKeydown(e: KeyboardEvent) {
  const tag = (e.target as HTMLElement)?.tagName
  const isInput = tag === 'INPUT' || tag === 'TEXTAREA'

  // Alt+Up / Alt+Down — navigate files (always)
  if (e.altKey && e.key === 'ArrowUp') {
    e.preventDefault()
    navigateFile(-1)
    return
  }
  if (e.altKey && e.key === 'ArrowDown') {
    e.preventDefault()
    navigateFile(1)
    return
  }

  // Ctrl+K or / — focus search
  if ((e.ctrlKey && e.key === 'k') || (!isInput && e.key === '/')) {
    e.preventDefault()
    sidebarRef.value?.focusSearch()
    return
  }

  // Escape — clear search & blur
  if (e.key === 'Escape') {
    searchQuery.value = ''
    sidebarRef.value?.blurSearch()
    return
  }
}

onMounted(() => window.addEventListener('keydown', handleKeydown))
onUnmounted(() => window.removeEventListener('keydown', handleKeydown))
</script>

<template>
  <div class="browser">
    <Sidebar
      ref="sidebarRef"
      :files="filteredFiles"
      :all-count="props.files.length"
      :current-index="currentIndex"
      :search-query="searchQuery"
      @select="selectFile"
      @search="handleSearch"
      @change-folder="handleChangeFolder"
    />
    <ContentView
      :file="currentFile"
      :current-index="currentIndex"
      :total-files="filteredFiles.length"
      :current-dir="props.currentDir"
      @navigate="navigateFile"
    />
  </div>
</template>

<style scoped>
.browser {
  display: flex;
  width: 100%;
  height: 100%;
  overflow: hidden;
}
</style>
