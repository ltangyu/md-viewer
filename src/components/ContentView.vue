<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import type { MdFileEntry } from '../types'
import { readTextFile, readImageBase64, resolvePath } from '../composables/useFileSystem'
import { useMarkdown } from '../composables/useMarkdown'

const props = defineProps<{
  file: MdFileEntry | null
  currentIndex: number
  totalFiles: number
  currentDir: string
}>()

const emit = defineEmits<{
  navigate: [delta: number]
}>()

const { parse } = useMarkdown()

const contentHtml = ref('')
const contentRef = ref<HTMLElement | null>(null)
const isLoading = ref(false)

/* ── Image data URL cache (cleared on directory change) ── */
const imageCache = new Map<string, string>()

/* ── Watch file changes ─────────────────────────── */
watch(() => props.file, async (newFile) => {
  if (!newFile) {
    contentHtml.value = ''
    return
  }

  isLoading.value = true
  try {
    const content = await readTextFile(newFile.path)

    if (!content.trim()) {
      contentHtml.value = '<p class="empty-hint">This file is empty.</p>'
      return
    }

    // Size guard: truncate files > 200 MB
    const MAX = 200 * 1024 * 1024
    const displayContent = content.length > MAX
      ? content.slice(0, MAX) + '\n\n---\n\n> **File truncated** — showing first 200 MB of ' +
        `${(content.length / 1024 / 1024).toFixed(1)} MB`
      : content

    contentHtml.value = parse(displayContent)

    await nextTick()
    if (contentRef.value) {
      contentRef.value.scrollTop = 0
      await processImages(contentRef.value, newFile.path)
    }
  } catch (err) {
    contentHtml.value = `<p class="error-hint">Failed to read file:<br/>${err}</p>`
  } finally {
    isLoading.value = false
  }
}, { immediate: true })

/* ── Clear image cache when directory changes ───── */
watch(() => props.currentDir, () => {
  imageCache.clear()
})

/* ── Post-process images: resolve relative paths ── */
async function processImages(container: HTMLElement, mdFilePath: string) {
  const images = container.querySelectorAll('img')
  if (images.length === 0) return

  const tasks = Array.from(images).map(async (img) => {
    const src = img.getAttribute('src')
    if (!src) return
    if (src.startsWith('http://') || src.startsWith('https://') || src.startsWith('data:')) return

    const absolutePath = resolvePath(mdFilePath, src)

    // Check cache
    const cached = imageCache.get(absolutePath)
    if (cached) {
      img.src = cached
      return
    }

    try {
      const dataUrl = await readImageBase64(absolutePath)
      imageCache.set(absolutePath, dataUrl)
      img.src = dataUrl
    } catch {
      img.style.opacity = '0.3'
      if (!img.alt || img.alt === src) {
        img.alt = `[Image not found: ${src}]`
      }
    }
  })

  await Promise.all(tasks)
}

/* ── Helpers ────────────────────────────────────── */
function displayName(file: MdFileEntry): string {
  return file.relative_path || file.name
}
</script>

<template>
  <main class="content">
    <!-- Navigation Header -->
    <header class="content-header">
      <div class="nav-controls">
        <button
          class="nav-btn"
          :disabled="currentIndex <= 0"
          title="Previous file (Alt+Up)"
          @click="emit('navigate', -1)"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor"
               stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="15 18 9 12 15 6"/>
          </svg>
        </button>

        <span class="nav-counter" v-if="totalFiles > 0">
          {{ currentIndex + 1 }} / {{ totalFiles }}
        </span>

        <button
          class="nav-btn"
          :disabled="currentIndex >= totalFiles - 1"
          title="Next file (Alt+Down)"
          @click="emit('navigate', 1)"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor"
               stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="9 18 15 12 9 6"/>
          </svg>
        </button>
      </div>

      <span class="file-path" v-if="file" :title="file.path">
        {{ displayName(file) }}
      </span>
    </header>

    <!-- Rendered Content -->
    <article
      ref="contentRef"
      class="content-body md-body"
      v-html="contentHtml"
    />

    <!-- Loading Overlay -->
    <div v-if="isLoading" class="loading-overlay">
      <span class="loading-dot" />
    </div>

    <!-- Empty State -->
    <div v-if="!file && !isLoading" class="empty-state">
      <p>Select a file from the sidebar</p>
    </div>
  </main>
</template>

<style scoped>
.content {
  flex: 1;
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  position: relative;
}

/* ── Header ─────────────────────────────────────── */
.content-header {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 10px 24px;
  border-bottom: 0.5px solid var(--border-soft);
  background: var(--bg);
  flex-shrink: 0;
  min-height: 44px;
}

.nav-controls {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.nav-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: 0.5px solid var(--border-soft);
  border-radius: var(--radius);
  background: transparent;
  color: var(--fg-muted);
  cursor: pointer;
  transition: all var(--trans-fast);
}
.nav-btn:hover:not(:disabled) {
  background: var(--surface-hover);
  color: var(--fg);
  border-color: var(--border);
}
.nav-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.nav-counter {
  font-family: var(--font-label);
  font-size: 12px;
  color: var(--fg-subtle);
  min-width: 52px;
  text-align: center;
  user-select: none;
}

.file-path {
  font-size: 12px;
  color: var(--fg-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}

/* ── Content Body ───────────────────────────────── */
.content-body {
  flex: 1;
  overflow-y: auto;
  padding: 30px 40px;
  max-width: 100%;
}

/* Center the markdown content for readability */
.content-body :deep(> *) {
  max-width: 860px;
}

/* ── Empty & Loading States ─────────────────────── */
.empty-state {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  color: var(--fg-subtle);
}

.loading-overlay {
  position: absolute;
  top: 44px;
  right: 16px;
  z-index: 10;
}

.loading-dot {
  display: block;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--fg-subtle);
  animation: pulse 1s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 0.3; }
  50% { opacity: 1; }
}

/* ── Error & Empty Hints (inside md-body) ───────── */
:deep(.empty-hint) {
  color: var(--fg-subtle);
  font-style: italic;
}
:deep(.error-hint) {
  color: #E53935;
  font-size: 13px;
}
</style>
