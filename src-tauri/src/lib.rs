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

/* ── Windows: rounded window corners ──────────── */
// Strategy:
// 1. Windows 11 (build 22000+): DwmSetWindowAttribute with DWMWA_WINDOW_CORNER_PREFERENCE
//    -> smooth anti-aliased rounded corners + native shadow
// 2. Windows 10: SetWindowRgn with CreateRoundRectRgn
//    -> aliased but visible rounded corners (DWM corner API is no-op on Win10)

#[cfg(target_os = "windows")]
mod win_corners {
    use std::ffi::c_void;

    #[repr(C)]
    struct RECT { left: i32, top: i32, right: i32, bottom: i32 }

    #[repr(C)]
    struct MARGINS { cx_left: i32, cx_right: i32, cy_top: i32, cy_bottom: i32 }

    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmSetWindowAttribute(hwnd: isize, attr: u32, value: *const c_void, size: u32) -> i32;
        fn DwmExtendFrameIntoClientArea(hwnd: isize, pMarInset: *const MARGINS) -> i32;
    }

    #[link(name = "user32")]
    extern "system" {
        fn GetClientRect(hwnd: isize, lpRect: *mut RECT) -> i32;
        fn SetWindowRgn(hwnd: isize, hrgn: *mut c_void, b_redraw: i32) -> i32;
    }

    #[link(name = "gdi32")]
    extern "system" {
        fn CreateRoundRectRgn(
            x1: i32, y1: i32, x2: i32, y2: i32, w: i32, h: i32,
        ) -> *mut c_void;
    }

    const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
    const DWMWCP_ROUND: u32 = 2;
    const CORNER_RADIUS: i32 = 16;

    /// Enable DWM-drawn shadow even for borderless / regioned windows.
    /// Margins of {0,0,1,0} extends the frame just 1px into the top of the
    /// client area — enough for DWM to draw a shadow around the whole window
    /// without visually affecting our custom title bar.
    fn enable_shadow(hwnd: isize) {
        unsafe {
            let m = MARGINS { cx_left: 0, cx_right: 0, cy_top: 1, cy_bottom: 0 };
            DwmExtendFrameIntoClientArea(hwnd, &m);
        }
    }

    pub fn apply(hwnd: isize) {
        unsafe {
            // 1) Try Windows 11 native rounded corners (preserves shadow automatically)
            let pref = DWMWCP_ROUND;
            let hr = DwmSetWindowAttribute(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &pref as *const u32 as *const c_void,
                std::mem::size_of::<u32>() as u32,
            );

            if hr != 0 {
                // 2) Windows 10 fallback: SetWindowRgn for rounded corners
                let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                GetClientRect(hwnd, &mut rect);
                let region = CreateRoundRectRgn(
                    0, 0,
                    rect.right + 1, rect.bottom + 1,
                    CORNER_RADIUS * 2 + 1, CORNER_RADIUS * 2 + 1,
                );
                if !region.is_null() {
                    SetWindowRgn(hwnd, region, 1);
                }
                // SetWindowRgn strips the DWM shadow — extend the frame to restore it
                enable_shadow(hwnd);
            }
        }
    }
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
        .setup(|app| {
            #[cfg(target_os = "windows")]
            {
                if let Some(window) = app.get_webview_window("main") {
                    if let Ok(hwnd) = window.hwnd() {
                        let h = hwnd.0 as isize;
                        win_corners::apply(h);

                        // Re-apply on resize so the rounded region tracks the window size
                        let win_clone = window.clone();
                        window.on_window_event(move |event| {
                            if let tauri::WindowEvent::Resized(_) = event {
                                if let Ok(hwnd) = win_clone.hwnd() {
                                    win_corners::apply(hwnd.0 as isize);
                                }
                            }
                        });
                    }
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_md_files,
            read_text_file,
            read_image_base64,
            get_launch_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MD Viewer");
}
