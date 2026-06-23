// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use inspectra_core::{memory, process, scanner, types};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

// Process information for frontend
#[derive(Clone, Serialize, Deserialize)]
struct ProcessInfo {
    pid: u32,
    name: String,
    path: String,
    architecture: String,
    memory_usage: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon: Option<String>, // Base64 encoded icon
}

// Scan result for frontend
#[derive(Clone, Serialize, Deserialize)]
struct ScanResult {
    address: String,
    value: String,
}

// Global state
struct AppState {
    attached_pid: Option<u32>,
    scan_results: Vec<types::ScanResult>,
}

type SharedState = Arc<Mutex<AppState>>;

#[tauri::command]
fn list_processes() -> Result<Vec<ProcessInfo>, String> {
    let manager = process::get_process_manager();
    let processes = manager.list_processes().map_err(|e| e.to_string())?;

    // Extract icons with better error handling
    Ok(processes
        .into_iter()
        .map(|p| {
            // Try to extract icon, but don't fail if it doesn't work
            let icon = if !p.path.is_empty() {
                extract_process_icon_robust(&p.path, p.pid)
            } else {
                None
            };

            ProcessInfo {
                pid: p.pid,
                name: p.name,
                path: p.path,
                architecture: format!("{:?}", p.architecture),
                memory_usage: p.memory_usage,
                icon,
            }
        })
        .collect())
}

#[cfg(windows)]
fn extract_process_icon_robust(path: &str, _pid: u32) -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_SMALLICON};
    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, HICON};

    if path.is_empty() {
        return None;
    }

    unsafe {
        // Convert path to wide string
        let wide_path: Vec<u16> = OsStr::new(path).encode_wide().chain(Some(0)).collect();
        if wide_path.len() < 2 {
            return None;
        }
        let path_ptr = PCWSTR::from_raw(wide_path.as_ptr());

        let mut file_info: SHFILEINFOW = std::mem::zeroed();

        // Get icon handle - use SHGFI_SMALLICON for 16x16 icons
        let result = SHGetFileInfoW(
            path_ptr,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut file_info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_SMALLICON,
        );

        if result == 0 {
            return None;
        }

        let hicon: HICON = file_info.hIcon;
        if hicon.is_invalid() {
            return None;
        }

        // Convert icon to PNG base64 using DrawIconEx method (more reliable)
        let icon_result = icon_to_png_base64_draw(hicon);
        let _ = DestroyIcon(hicon);

        icon_result.map(|data| format!("data:image/png;base64,{}", data))
    }
}

// Test function to verify icon extraction
#[cfg(windows)]
#[allow(dead_code)]
fn test_icon_extraction() {
    let test_paths = vec![
        "C:\\Windows\\System32\\notepad.exe",
        "C:\\Windows\\System32\\calc.exe",
    ];

    for path in test_paths {
        if let Some(icon) = extract_process_icon_robust(path, 0) {
            println!("Successfully extracted icon from: {}", path);
            println!("Icon data length: {} bytes", icon.len());
        } else {
            println!("Failed to extract icon from: {}", path);
        }
    }
}

#[cfg(windows)]
unsafe fn icon_to_png_base64_draw(
    hicon: windows::Win32::UI::WindowsAndMessaging::HICON,
) -> Option<String> {
    use image::RgbaImage;
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
        ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS,
    };
    use windows::Win32::UI::WindowsAndMessaging::{DrawIconEx, DI_NORMAL};

    const ICON_SIZE: i32 = 32; // Use 32x32 for better quality

    let hdc = GetDC(None);
    if hdc.is_invalid() {
        return None;
    }

    let hdc_mem = CreateCompatibleDC(hdc);
    if hdc_mem.is_invalid() {
        ReleaseDC(None, hdc);
        return None;
    }

    // Create a 32-bit bitmap
    let hbmp = CreateCompatibleBitmap(hdc, ICON_SIZE, ICON_SIZE);
    if hbmp.is_invalid() {
        DeleteDC(hdc_mem);
        ReleaseDC(None, hdc);
        return None;
    }

    let _old_bmp = SelectObject(hdc_mem, hbmp);

    // Draw the icon onto the bitmap
    let drawn = DrawIconEx(
        hdc_mem, 0, 0, hicon, ICON_SIZE, ICON_SIZE, 0, None, DI_NORMAL,
    );

    if drawn.is_err() {
        DeleteObject(hbmp);
        DeleteDC(hdc_mem);
        ReleaseDC(None, hdc);
        return None;
    }

    // Prepare bitmap info for GetDIBits
    let mut bmp_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: ICON_SIZE,
            biHeight: -ICON_SIZE, // Negative for top-down DIB
            biPlanes: 1,
            biBitCount: 32,
            biCompression: 0, // BI_RGB
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [std::mem::zeroed(); 1],
    };

    // Allocate buffer for pixel data (BGRA format)
    let size = (ICON_SIZE * ICON_SIZE * 4) as usize;
    let mut bits: Vec<u8> = vec![0; size];

    // Get the bitmap bits
    let result = GetDIBits(
        hdc_mem,
        hbmp,
        0,
        ICON_SIZE as u32,
        Some(bits.as_mut_ptr() as *mut _),
        &mut bmp_info,
        DIB_RGB_COLORS,
    );

    // Cleanup
    SelectObject(hdc_mem, _old_bmp);
    DeleteObject(hbmp);
    DeleteDC(hdc_mem);
    ReleaseDC(None, hdc);

    if result == 0 {
        return None;
    }

    // Convert BGRA to RGBA
    for i in (0..bits.len()).step_by(4) {
        // Swap B and R channels: BGRA -> RGBA
        bits.swap(i, i + 2);
    }

    // Create image and encode to PNG
    if let Some(img) = RgbaImage::from_raw(ICON_SIZE as u32, ICON_SIZE as u32, bits) {
        let mut png_data = Vec::new();
        {
            let mut cursor = std::io::Cursor::new(&mut png_data);
            if image::write_buffer_with_format(
                &mut cursor,
                &img.into_raw(),
                ICON_SIZE as u32,
                ICON_SIZE as u32,
                image::ColorType::Rgba8,
                image::ImageFormat::Png,
            )
            .is_ok()
            {
                use base64::{engine::general_purpose, Engine as _};
                let encoded = general_purpose::STANDARD.encode(&png_data);
                // Verify the encoded data is not empty
                if !encoded.is_empty() && encoded.len() > 100 {
                    return Some(encoded);
                }
            }
        }
    }

    None
}

#[cfg(not(windows))]
fn extract_process_icon(_path: &str, _pid: u32) -> Option<String> {
    None
}

#[tauri::command]
fn attach_process(state: tauri::State<SharedState>, pid: u32) -> Result<String, String> {
    let manager = process::get_process_manager();
    let handle = manager.attach(pid).map_err(|e| e.to_string())?;

    if !handle.is_alive() {
        return Err("Process is not alive".to_string());
    }

    let mut app_state = state.lock().unwrap();
    app_state.attached_pid = Some(pid);
    app_state.scan_results.clear();
    // Handle will be recreated when needed

    Ok(format!("Successfully attached to process {}", pid))
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn scan_memory(
    state: tauri::State<SharedState>,
    scan_type: String,
    value: Option<String>,
    data_type: String,
    range_min: Option<f64>,
    range_max: Option<f64>,
    writable_only: bool,
    aligned: bool,
) -> Result<Vec<ScanResult>, String> {
    let app_state = state.lock().unwrap();
    let pid = app_state.attached_pid.ok_or("No process attached")?;
    drop(app_state);

    // Recreate handle - it's safe to reopen
    let manager = process::get_process_manager();
    let handle = manager.attach(pid).map_err(|e| e.to_string())?;
    let mem = memory::create_memory(handle.as_ref()).map_err(|e| e.to_string())?;

    let dt = match data_type.as_str() {
        "i8" => types::DataType::I8,
        "i16" => types::DataType::I16,
        "i32" => types::DataType::I32,
        "i64" => types::DataType::I64,
        "f32" => types::DataType::F32,
        "f64" => types::DataType::F64,
        "string" => types::DataType::String,
        "bytes" => types::DataType::Bytes,
        _ => return Err(format!("Unknown data type: {}", data_type)),
    };

    let st = match scan_type.as_str() {
        "exact" => types::ScanType::Exact,
        "bigger" => types::ScanType::Bigger,
        "smaller" => types::ScanType::Smaller,
        "between" => types::ScanType::Range,
        "unknown" => types::ScanType::Unknown,
        "changed" => types::ScanType::Changed,
        "unchanged" => types::ScanType::Unchanged,
        "increased" => types::ScanType::Increased,
        "decreased" => types::ScanType::Decreased,
        _ => types::ScanType::Exact,
    };

    let config = scanner::ScanConfig {
        data_type: dt,
        scan_type: st,
        writable_only,
        aligned,
        ..Default::default()
    };

    let mut scanner = scanner::Scanner::new(mem, config);

    let value_bytes = if let Some(val) = &value {
        Some(scanner::value_to_bytes(val, dt).map_err(|e| e.to_string())?)
    } else {
        None
    };

    let range = if let (Some(min), Some(max)) = (range_min, range_max) {
        Some((min, max))
    } else {
        None
    };

    let results = scanner
        .scan(value_bytes.as_deref(), range)
        .map_err(|e| e.to_string())?;

    let frontend_results: Vec<ScanResult> = results
        .iter()
        .take(1000) // Limit to 1000 results
        .map(|r| ScanResult {
            address: format!("0x{:X}", r.address),
            value: scanner::bytes_to_string(&r.value, r.data_type),
        })
        .collect();

    let mut app_state = state.lock().unwrap();
    app_state.scan_results = results;

    Ok(frontend_results)
}

#[tauri::command]
fn rescan_memory(
    state: tauri::State<SharedState>,
    scan_type: String,
    value: Option<String>,
    data_type: String,
    range_min: Option<f64>,
    range_max: Option<f64>,
) -> Result<Vec<ScanResult>, String> {
    let app_state = state.lock().unwrap();
    let pid = app_state.attached_pid.ok_or("No process attached")?;
    drop(app_state);

    // Recreate handle - it's safe to reopen
    let manager = process::get_process_manager();
    let handle = manager.attach(pid).map_err(|e| e.to_string())?;
    let mem = memory::create_memory(handle.as_ref()).map_err(|e| e.to_string())?;

    let dt = match data_type.as_str() {
        "i8" => types::DataType::I8,
        "i16" => types::DataType::I16,
        "i32" => types::DataType::I32,
        "i64" => types::DataType::I64,
        "f32" => types::DataType::F32,
        "f64" => types::DataType::F64,
        "string" => types::DataType::String,
        "bytes" => types::DataType::Bytes,
        _ => return Err(format!("Unknown data type: {}", data_type)),
    };

    let st = match scan_type.as_str() {
        "exact" => types::ScanType::Exact,
        "bigger" => types::ScanType::Bigger,
        "smaller" => types::ScanType::Smaller,
        "between" => types::ScanType::Range,
        "unknown" => types::ScanType::Unknown,
        "changed" => types::ScanType::Changed,
        "unchanged" => types::ScanType::Unchanged,
        "increased" => types::ScanType::Increased,
        "decreased" => types::ScanType::Decreased,
        _ => types::ScanType::Exact,
    };

    let config = scanner::ScanConfig {
        data_type: dt,
        scan_type: st,
        ..Default::default()
    };

    let mut scanner = scanner::Scanner::new(mem, config);

    // Restore previous results
    let app_state = state.lock().unwrap();
    scanner.set_results(app_state.scan_results.clone());
    drop(app_state);

    let value_bytes = if let Some(val) = &value {
        Some(scanner::value_to_bytes(val, dt).map_err(|e| e.to_string())?)
    } else {
        None
    };

    let range = if let (Some(min), Some(max)) = (range_min, range_max) {
        Some((min, max))
    } else {
        None
    };

    let results = scanner
        .rescan(value_bytes.as_deref(), range)
        .map_err(|e| e.to_string())?;

    let frontend_results: Vec<ScanResult> = results
        .iter()
        .take(1000)
        .map(|r| ScanResult {
            address: format!("0x{:X}", r.address),
            value: scanner::bytes_to_string(&r.value, r.data_type),
        })
        .collect();

    let mut app_state = state.lock().unwrap();
    app_state.scan_results = results;

    Ok(frontend_results)
}

#[tauri::command]
fn read_memory(pid: u32, address: String, size: usize) -> Result<Vec<u8>, String> {
    let addr = if let Some(hex) = address.strip_prefix("0x") {
        usize::from_str_radix(hex, 16)
    } else {
        address.parse()
    }
    .map_err(|e| format!("Invalid address: {}", e))?;

    let manager = process::get_process_manager();
    let handle = manager.attach(pid).map_err(|e| e.to_string())?;
    let mem = memory::create_memory(handle.as_ref()).map_err(|e| e.to_string())?;

    mem.read(addr, size).map_err(|e| e.to_string())
}

#[tauri::command]
fn write_memory(
    state: tauri::State<SharedState>,
    address: String,
    data: Vec<u8>,
) -> Result<String, String> {
    let addr = if let Some(hex) = address.strip_prefix("0x") {
        usize::from_str_radix(hex, 16)
    } else {
        address.parse()
    }
    .map_err(|e| format!("Invalid address: {}", e))?;

    let app_state = state.lock().unwrap();
    let pid = app_state.attached_pid.ok_or("No process attached")?;
    drop(app_state);

    // Recreate handle
    let manager = process::get_process_manager();
    let handle = manager.attach(pid).map_err(|e| e.to_string())?;
    let mem = memory::create_memory(handle.as_ref()).map_err(|e| e.to_string())?;

    let written = mem.write(addr, &data).map_err(|e| e.to_string())?;

    Ok(format!("Written {} bytes", written))
}

#[tauri::command]
fn get_memory_regions(pid: u32) -> Result<Vec<types::MemoryRegion>, String> {
    let manager = process::get_process_manager();
    let handle = manager.attach(pid).map_err(|e| e.to_string())?;
    let mem = memory::create_memory(handle.as_ref()).map_err(|e| e.to_string())?;

    mem.query_regions().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_version() -> String {
    format!("Inspectra v{}", env!("CARGO_PKG_VERSION"))
}

fn main() {
    inspectra_core::init().expect("Failed to initialize Inspectra");

    let state = Arc::new(Mutex::new(AppState {
        attached_pid: None,
        scan_results: Vec::new(),
    }));

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            list_processes,
            attach_process,
            scan_memory,
            rescan_memory,
            read_memory,
            write_memory,
            get_memory_regions,
            get_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
