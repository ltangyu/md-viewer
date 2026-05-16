import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import type { MdFileEntry } from '../types'

const STORAGE_KEY = 'md-viewer-last-dir'

/** Open a native folder-picker dialog. Returns the chosen path or `null`. */
export async function selectFolder(): Promise<string | null> {
  const selected = await open({
    directory: true,
    multiple: false,
    title: 'Select Markdown Folder',
  })
  return selected as string | null
}

/** Recursively scan `dirPath` for .md files (sorted by relative path). */
export async function scanMdFiles(dirPath: string): Promise<MdFileEntry[]> {
  return invoke<MdFileEntry[]>('scan_md_files', { dirPath })
}

/** Read a text file and return its content. */
export async function readTextFile(filePath: string): Promise<string> {
  return invoke<string>('read_text_file', { filePath })
}

/** Read an image and return a `data:` URL. */
export async function readImageBase64(filePath: string): Promise<string> {
  return invoke<string>('read_image_base64', { filePath })
}

/**
 * Resolve a relative path (from a markdown image src) against
 * the directory of the current .md file.
 *
 * Example:
 *   resolvePath("D:/docs/guide.md", "../img/fig.png")
 *   => "D:/docs/../img/fig.png" => "D:/img/fig.png"
 */
export function resolvePath(mdFilePath: string, relativeSrc: string): string {
  const normalized = mdFilePath.replace(/\\/g, '/')
  const dir = normalized.substring(0, normalized.lastIndexOf('/'))
  const combined = `${dir}/${relativeSrc.replace(/\\/g, '/')}`

  const parts = combined.split('/')
  const resolved: string[] = []
  for (const part of parts) {
    if (part === '.' || part === '') continue
    if (part === '..') {
      resolved.pop()
    } else {
      resolved.push(part)
    }
  }
  return resolved.join('/')
}

/** Persist the last-opened directory name to localStorage. */
export function saveLastDir(dirPath: string): void {
  try {
    localStorage.setItem(STORAGE_KEY, dirPath)
  } catch {
    // localStorage may be unavailable in some contexts
  }
}

/** Retrieve the last-opened directory path from localStorage. */
export function loadLastDir(): string | null {
  try {
    return localStorage.getItem(STORAGE_KEY)
  } catch {
    return null
  }
}
