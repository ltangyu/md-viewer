<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import {
  selectFolder,
  scanMdFiles,
  saveLastDir,
  loadLastDir,
} from '../composables/useFileSystem'
import type { MdFileEntry } from '../types'

const emit = defineEmits<{
  'folder-opened': [dir: string, files: MdFileEntry[]]
}>()

const lastDir = ref<string | null>(null)
const isLoading = ref(false)
const errorMsg = ref('')

const lastDirName = computed(() => {
  if (!lastDir.value) return ''
  const parts = lastDir.value.replace(/\\/g, '/').split('/')
  return parts[parts.length - 1] || parts[parts.length - 2] || lastDir.value
})

onMounted(() => {
  lastDir.value = loadLastDir()
})

async function handleOpen() {
  errorMsg.value = ''
  const dir = await selectFolder()
  if (!dir) return
  await openDirectory(dir)
}

async function handleOpenLast() {
  if (!lastDir.value) return
  errorMsg.value = ''
  await openDirectory(lastDir.value)
}

async function openDirectory(dir: string) {
  isLoading.value = true
  try {
    const mdFiles = await scanMdFiles(dir)
    if (mdFiles.length === 0) {
      errorMsg.value = 'No .md files found in this directory.'
      return
    }
    saveLastDir(dir)
    emit('folder-opened', dir, mdFiles)
  } catch (err) {
    errorMsg.value = `Failed to scan: ${err}`
  } finally {
    isLoading.value = false
  }
}
</script>

<template>
  <div class="landing">
    <div class="landing-card">
      <div class="landing-icon">
        <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/>
          <polyline points="14 2 14 8 20 8"/>
          <line x1="16" y1="13" x2="8" y2="13"/>
          <line x1="16" y1="17" x2="8" y2="17"/>
          <polyline points="10 9 9 9 8 9"/>
        </svg>
      </div>

      <h1 class="landing-title">MD Viewer</h1>
      <p class="landing-subtitle">Lightweight Markdown file browser</p>

      <button class="btn-primary" :disabled="isLoading" @click="handleOpen">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor"
             stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
        {{ isLoading ? 'Scanning...' : 'Open Folder' }}
      </button>

      <button
        v-if="lastDir && !isLoading"
        class="btn-ghost"
        @click="handleOpenLast"
      >
        Reopen: {{ lastDirName }}
      </button>

      <p v-if="errorMsg" class="error-msg">{{ errorMsg }}</p>
    </div>
  </div>
</template>

<style scoped>
.landing {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  background: var(--bg);
}

.landing-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 48px 56px;
  background: var(--glass-bg);
  backdrop-filter: var(--glass-blur);
  -webkit-backdrop-filter: var(--glass-blur);
  border: var(--glass-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-md);
  text-align: center;
  max-width: 380px;
}

.landing-icon {
  color: var(--fg-subtle);
  margin-bottom: 4px;
}

.landing-title {
  font-family: var(--font-label);
  font-size: 28px;
  font-weight: 700;
  letter-spacing: -0.02em;
  color: var(--fg);
}

.landing-subtitle {
  font-size: 13px;
  color: var(--fg-muted);
  margin-bottom: 12px;
}

.btn-primary {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 28px;
  background: var(--fg);
  color: var(--bg);
  border: none;
  border-radius: var(--radius-md);
  font-family: var(--font-sans);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: opacity var(--trans-fast);
}
.btn-primary:hover:not(:disabled) {
  opacity: 0.85;
}
.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-ghost {
  padding: 6px 16px;
  background: none;
  border: 0.5px solid var(--border);
  border-radius: var(--radius);
  font-family: var(--font-sans);
  font-size: 12px;
  color: var(--fg-muted);
  cursor: pointer;
  transition: all var(--trans-fast);
  max-width: 260px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.btn-ghost:hover {
  background: var(--surface-hover);
  color: var(--fg);
}

.error-msg {
  font-size: 12px;
  color: #E53935;
  margin-top: 4px;
}
</style>
