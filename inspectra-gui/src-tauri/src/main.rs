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
        .map(|p| ProcessInfo {
            pid: p.pid,
            name: p.name,
            path: p.path,
            architecture: format!("{:?}", p.architecture),
            memory_usage: p.memory_usage,
        })
        .collect())
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

    Ok(format!("Successfully attached to process {}", pid))
}

#[tauri::command]
fn scan_memory(
    state: tauri::State<SharedState>,
    value: String,
    data_type: String,
) -> Result<Vec<ScanResult>, String> {
    let app_state = state.lock().unwrap();
    let pid = app_state
        .attached_pid
        .ok_or("No process attached")?;
    drop(app_state);

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
        _ => return Err(format!("Unknown data type: {}", data_type)),
    };

    let mut config = scanner::ScanConfig::default();
    config.data_type = dt;

    let mut scanner = scanner::Scanner::new(mem, config);

    let bytes = scanner::value_to_bytes(&value, dt).map_err(|e| e.to_string())?;
    let results = scanner.scan(&bytes).map_err(|e| e.to_string())?;

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
fn write_memory(pid: u32, address: String, data: Vec<u8>) -> Result<String, String> {
    let addr = if address.starts_with("0x") {
        usize::from_str_radix(&address[2..], 16)
    } else {
        address.parse()
    }
    .map_err(|e| format!("Invalid address: {}", e))?;

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
            read_memory,
            write_memory,
            get_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
