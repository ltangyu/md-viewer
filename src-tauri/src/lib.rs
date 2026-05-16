use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::{Emitter, Manager};
use walkdir::WalkDir;

/* ── Types ──────────────────────────────────────── */

#[derive(Debug, Serialize, Clone)]
pub struct MdFileEntry {
    /// File name only (e.g. "README.md")
    pub name: String,
    /// Absolute path on disk
    pub path: String,
    /// Path relative to the chosen root, using `/` separators
    pub relative_path: String,
    /// File size in bytes
    pub size: u64,
}

/* ── Commands ───────────────────────────────────── */

/// Recursively scan a directory for `.md` files, sorted by relative path.
#[tauri::command]
async fn scan_md_files(dir_path: String) -> Result<Vec<MdFileEntry>, String> {
    let base = PathBuf::from(&dir_path);
    if !base.is_dir() {
        return Err(format!("Not a directory: {}", dir_path));
    }

    let mut entries: Vec<MdFileEntry> = Vec::new();

    for entry in WalkDir::new(&base)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext_match = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("md"))
            .unwrap_or(false);
        if !ext_match {
            continue;
        }

        let relative = path
            .strip_prefix(&base)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
            .replace('\\', "/");

        entries.push(MdFileEntry {
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            path: path.to_string_lossy().to_string(),
            relative_path: relative,
            size: entry.metadata().map(|m| m.len()).unwrap_or(0),
        });
    }

    entries.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(entries)
}

/// Read a text file and return its content as a UTF-8 string.
/// Falls back to lossy conversion if the file is not valid UTF-8.
#[tauri::command]
async fn read_text_file(file_path: String) -> Result<String, String> {
    let bytes = std::fs::read(&file_path).map_err(|e| format!("Read error: {}", e))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Read an image file and return it as a `data:<mime>;base64,...` string.
#[tauri::command]
async fn read_image_base64(file_path: String) -> Result<String, String> {
    let data = std::fs::read(&file_path).map_err(|e| format!("Read error: {}", e))?;

    let ext = Path::new(&file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    let mime = match ext.to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "ico" => "image/x-icon",
        "avif" => "image/avif",
        _ => "application/octet-stream",
    };

    let b64 = general_purpose::STANDARD.encode(&data);
    Ok(format!("data:{};base64,{}", mime, b64))
}

/// Return the .md file path passed as a CLI argument (if any).
/// Called once by the frontend on startup.
#[tauri::command]
fn get_launch_file() -> Option<String> {
    std::env::args().nth(1).filter(|p| {
        let path = Path::new(p);
        path.exists()
            && path
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("md"))
                .unwrap_or(false)
    })
}

/* ── Helpers ────────────────────────────────────── */

/// Check whether a CLI arg looks like a valid .md file path.
fn is_md_file_arg(arg: &str) -> bool {
    let path = Path::new(arg);
    path.exists()
        && path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("md"))
            .unwrap_or(false)
}

/* ── App Entry ──────────────────────────────────── */

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            // Focus the existing window
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.unminimize();
                let _ = w.set_focus();
            }
            // Forward the file path from the second instance to the frontend
            if let Some(file_path) = args.get(1) {
                if is_md_file_arg(file_path) {
                    let _ = app.emit("open-file", file_path.clone());
                }
            }
        }))
        .invoke_handler(tauri::generate_handler![
            scan_md_files,
            read_text_file,
            read_image_base64,
            get_launch_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MD Viewer");
}
