/** Mirrors the Rust `MdFileEntry` struct returned by `scan_md_files`. */
export interface MdFileEntry {
  /** File name only, e.g. "README.md" */
  name: string
  /** Absolute path on disk */
  path: string
  /** Path relative to chosen root, forward-slash separated */
  relative_path: string
  /** File size in bytes */
  size: number
}

/** View state for the root App component. */
export type ViewState = 'landing' | 'browser'
