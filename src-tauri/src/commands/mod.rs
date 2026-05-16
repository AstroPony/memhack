use crate::memory::{process, scanner, table, types::*, writer};
use std::sync::Mutex;
use tauri::State;

/// Shared application state managed by Tauri
pub struct AppState {
    /// Currently attached process handle
    pub process: Mutex<Option<process::ProcessHandle>>,
    /// Current scan results
    pub scan_state: Mutex<Option<ScanState>>,
    /// Value freezer (active when a process is attached)
    pub freezer: Mutex<Option<writer::ValueFreezer>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            process: Mutex::new(None),
            scan_state: Mutex::new(None),
            freezer: Mutex::new(None),
        }
    }
}

// ── Process Commands ──────────────────────────────────────────

#[tauri::command]
pub fn list_processes() -> Result<Vec<ProcessInfo>, String> {
    process::list_processes()
}

#[tauri::command]
pub fn attach_process(state: State<'_, AppState>, pid: u32, name: String) -> Result<String, String> {
    let handle = process::attach_process(pid, name.clone())?;

    // Create a new freezer for this process (50ms interval = 20 writes/sec)
    let freezer = writer::ValueFreezer::new(handle.handle, 50);

    *state.process.lock().unwrap() = Some(handle);
    *state.scan_state.lock().unwrap() = None; // Reset scan on new attach
    *state.freezer.lock().unwrap() = Some(freezer);

    Ok(format!("Attached to {} (PID {})", name, pid))
}

#[tauri::command]
pub fn detach_process(state: State<'_, AppState>) -> Result<(), String> {
    *state.freezer.lock().unwrap() = None; // Drop freezer first (stops bg thread)
    *state.process.lock().unwrap() = None;
    *state.scan_state.lock().unwrap() = None;
    Ok(())
}

// ── Scan Commands ─────────────────────────────────────────────

#[tauri::command]
pub fn first_scan(state: State<'_, AppState>, value: ScanValue) -> Result<ScanSummary, String> {
    let proc_guard = state.process.lock().unwrap();
    let proc = proc_guard
        .as_ref()
        .ok_or("No process attached")?;

    let results = scanner::first_scan(proc, &value, None);
    let count = results.len();

    // Determine value type from the scan value
    let value_type = match &value {
        ScanValue::I8(_) => ValueType::I8,
        ScanValue::I16(_) => ValueType::I16,
        ScanValue::I32(_) => ValueType::I32,
        ScanValue::I64(_) => ValueType::I64,
        ScanValue::U8(_) => ValueType::U8,
        ScanValue::U16(_) => ValueType::U16,
        ScanValue::U32(_) => ValueType::U32,
        ScanValue::U64(_) => ValueType::U64,
        ScanValue::F32(_) => ValueType::F32,
        ScanValue::F64(_) => ValueType::F64,
        ScanValue::StringUtf8(s) => ValueType::StringUtf8(s.len()),
        ScanValue::StringUtf16(s) => ValueType::StringUtf16(s.encode_utf16().count() * 2),
        ScanValue::ByteArray(b) => ValueType::ByteArray(b.len()),
    };

    *state.scan_state.lock().unwrap() = Some(ScanState {
        results,
        scan_count: 1,
        value_type,
    });

    Ok(ScanSummary {
        found: count,
        scan_number: 1,
    })
}

#[tauri::command]
pub fn next_scan(state: State<'_, AppState>, filter: ScanFilter) -> Result<ScanSummary, String> {
    let proc_guard = state.process.lock().unwrap();
    let proc = proc_guard
        .as_ref()
        .ok_or("No process attached")?;

    let mut scan_guard = state.scan_state.lock().unwrap();
    let scan = scan_guard
        .as_mut()
        .ok_or("No previous scan — run First Scan first")?;

    let results = scanner::next_scan(proc, &scan.results, &filter);
    let count = results.len();
    scan.results = results;
    scan.scan_count += 1;

    Ok(ScanSummary {
        found: count,
        scan_number: scan.scan_count,
    })
}

#[tauri::command]
pub fn get_scan_results(
    state: State<'_, AppState>,
    offset: usize,
    limit: usize,
) -> Result<Vec<FoundAddress>, String> {
    let scan_guard = state.scan_state.lock().unwrap();
    let scan = scan_guard
        .as_ref()
        .ok_or("No scan results")?;

    let end = std::cmp::min(offset + limit, scan.results.len());
    if offset >= scan.results.len() {
        return Ok(Vec::new());
    }

    Ok(scan.results[offset..end].to_vec())
}

#[tauri::command]
pub fn reset_scan(state: State<'_, AppState>) {
    *state.scan_state.lock().unwrap() = None;
}

// ── Write / Freeze Commands ───────────────────────────────────

#[tauri::command]
pub fn write_value(
    state: State<'_, AppState>,
    address: u64,
    value: ScanValue,
) -> Result<(), String> {
    let proc_guard = state.process.lock().unwrap();
    let proc = proc_guard
        .as_ref()
        .ok_or("No process attached")?;

    writer::write_value(proc.handle, address, &value)
}

#[tauri::command]
pub fn freeze_value(
    state: State<'_, AppState>,
    address: u64,
    value: ScanValue,
) -> Result<(), String> {
    let freezer_guard = state.freezer.lock().unwrap();
    let freezer = freezer_guard
        .as_ref()
        .ok_or("No freezer active")?;

    freezer.freeze(address, value);
    Ok(())
}

#[tauri::command]
pub fn unfreeze_value(state: State<'_, AppState>, address: u64) -> Result<(), String> {
    let freezer_guard = state.freezer.lock().unwrap();
    let freezer = freezer_guard
        .as_ref()
        .ok_or("No freezer active")?;

    freezer.unfreeze(address);
    Ok(())
}

// ── Address Table Commands ────────────────────────────────────

#[tauri::command]
pub fn save_address_table(game_name: String, entries: Vec<TableEntry>) -> Result<String, String> {
    let tbl = table::AddressTable {
        game_name,
        entries,
    };
    table::save_table(&tbl)
}

#[tauri::command]
pub fn load_address_table(game_name: String) -> Result<Vec<TableEntry>, String> {
    let tbl = table::load_table(&game_name)?;
    Ok(tbl.entries)
}

#[tauri::command]
pub fn list_saved_tables() -> Result<Vec<String>, String> {
    table::list_tables()
}

// ── Helper types ──────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct ScanSummary {
    pub found: usize,
    pub scan_number: u32,
}
