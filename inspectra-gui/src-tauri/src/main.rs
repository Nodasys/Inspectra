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
    let processes = manager
        .list_processes()
        .map_err(|e| e.to_string())?;

    Ok(processes
        .into_iter()
        .map(|p| {
            let icon = extract_process_icon(&p.path, p.pid);
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
fn extract_process_icon(path: &str, _pid: u32) -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_SMALLICON};
    use windows::Win32::UI::WindowsAndMessaging::{HICON, DestroyIcon, GetIconInfo};
    use windows::Win32::Graphics::Gdi::{BITMAP, GetObjectW, GetDC, CreateCompatibleDC, SelectObject, GetDIBits, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, CreateCompatibleBitmap, DeleteObject, DeleteDC, ReleaseDC};
    use std::ptr;
    
    unsafe {
        let exe_path = if !path.is_empty() {
            path.to_string()
        } else {
            return None;
        };
        
        // Convert path to wide string
        let wide_path: Vec<u16> = OsStr::new(&exe_path).encode_wide().chain(Some(0)).collect();
        let path_ptr = PCWSTR::from_raw(wide_path.as_ptr());
        
        let mut file_info: SHFILEINFOW = std::mem::zeroed();
        
        // Get icon handle
        if SHGetFileInfoW(
            path_ptr,
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut file_info),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_SMALLICON,
        ) != 0 {
            let hicon: HICON = file_info.hIcon;
            if !hicon.is_invalid() {
                // Convert icon to PNG base64
                if let Some(icon_data) = icon_to_png_base64(hicon) {
                    let _ = DestroyIcon(hicon);
                    return Some(format!("data:image/png;base64,{}", icon_data));
                }
                let _ = DestroyIcon(hicon);
            }
        }
        
        None
    }
}

#[cfg(windows)]
unsafe fn icon_to_png_base64(hicon: windows::Win32::UI::WindowsAndMessaging::HICON) -> Option<String> {
    use windows::Win32::Graphics::Gdi::{BITMAP, GetObjectW, GetDC, CreateCompatibleDC, SelectObject, GetDIBits, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, CreateCompatibleBitmap, DeleteObject, DeleteDC, ReleaseDC};
    use windows::Win32::UI::WindowsAndMessaging::GetIconInfo;
    use image::RgbaImage;
    
    let mut icon_info = std::mem::zeroed();
    if GetIconInfo(hicon, &mut icon_info).is_err() {
        return None;
    }
    
    let hdc = GetDC(None);
    if hdc.is_invalid() {
        return None;
    }
    
    let mut bm: BITMAP = std::mem::zeroed();
    if GetObjectW(icon_info.hbmColor, std::mem::size_of::<BITMAP>() as i32, Some(&mut bm as *mut _ as *mut _)) == 0 {
        ReleaseDC(None, hdc);
        return None;
    }
    
    let width = bm.bmWidth as u32;
    let height = bm.bmHeight as u32;
    
    if width == 0 || height == 0 || width > 256 || height > 256 {
        ReleaseDC(None, hdc);
        return None;
    }
    
    let hdc_mem = CreateCompatibleDC(hdc);
    if hdc_mem.is_invalid() {
        ReleaseDC(None, hdc);
        return None;
    }
    
    let mut bmp_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width as i32,
            biHeight: -(height as i32), // Negative for top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: 0,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [std::mem::zeroed(); 1],
    };
    
    let hbmp = CreateCompatibleBitmap(hdc, width as i32, height as i32);
    if hbmp.is_invalid() {
        DeleteDC(hdc_mem);
        ReleaseDC(None, hdc);
        return None;
    }
    
    let _old = SelectObject(hdc_mem, hbmp);
    SelectObject(hdc_mem, icon_info.hbmColor);
    
    let mut bits: Vec<u8> = vec![0; (width * height * 4) as usize];
    let result = GetDIBits(
        hdc_mem,
        hbmp,
        0,
        height,
        Some(bits.as_mut_ptr() as *mut _),
        &mut bmp_info,
        DIB_RGB_COLORS,
    );
    
    if result == 0 {
        DeleteObject(hbmp);
        DeleteDC(hdc_mem);
        ReleaseDC(None, hdc);
        return None;
    }
    
    // Convert BGRA to RGBA
    for i in (0..bits.len()).step_by(4) {
        bits.swap(i, i + 2);
    }
    
    DeleteObject(hbmp);
    DeleteDC(hdc_mem);
    ReleaseDC(None, hdc);
    
    // Create image and encode to PNG
    let img = RgbaImage::from_raw(width, height, bits)?;
    let mut png_data = Vec::new();
    {
        let mut cursor = std::io::Cursor::new(&mut png_data);
        image::write_buffer_with_format(
            &mut cursor,
            &img.into_raw(),
            width,
            height,
            image::ColorType::Rgba8,
            image::ImageFormat::Png,
        ).ok()?;
    }
    
    Some(base64::encode(&png_data))
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
    let pid = app_state
        .attached_pid
        .ok_or("No process attached")?;
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

    let mut config = scanner::ScanConfig::default();
    config.data_type = dt;
    config.scan_type = st;
    config.writable_only = writable_only;
    config.aligned = aligned;

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

    let results = scanner.scan(value_bytes.as_deref(), range).map_err(|e| e.to_string())?;

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
    let pid = app_state
        .attached_pid
        .ok_or("No process attached")?;
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

    let mut config = scanner::ScanConfig::default();
    config.data_type = dt;
    config.scan_type = st;

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

    let results = scanner.rescan(value_bytes.as_deref(), range).map_err(|e| e.to_string())?;

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
    let addr = if address.starts_with("0x") {
        usize::from_str_radix(&address[2..], 16)
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
fn write_memory(state: tauri::State<SharedState>, address: String, data: Vec<u8>) -> Result<String, String> {
    let addr = if address.starts_with("0x") {
        usize::from_str_radix(&address[2..], 16)
    } else {
        address.parse()
    }
    .map_err(|e| format!("Invalid address: {}", e))?;

    let app_state = state.lock().unwrap();
    let pid = app_state
        .attached_pid
        .ok_or("No process attached")?;
    drop(app_state);

    // Recreate handle
    let manager = process::get_process_manager();
    let handle = manager.attach(pid).map_err(|e| e.to_string())?;
    let mem = memory::create_memory(handle.as_ref()).map_err(|e| e.to_string())?;

    let written = mem.write(addr, &data).map_err(|e| e.to_string())?;

    Ok(format!("Written {} bytes", written))
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
            get_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
