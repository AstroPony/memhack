use crate::memory::types::ScanValue;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;


/// Write a value to a specific address in the target process
pub fn write_value(handle: HANDLE, address: u64, value: &ScanValue) -> Result<(), String> {
    let bytes = value.to_bytes();
    let mut bytes_written = 0usize;

    unsafe {
        WriteProcessMemory(
            handle,
            address as *const _,
            bytes.as_ptr() as *const _,
            bytes.len(),
            Some(&mut bytes_written),
        )
        .map_err(|e| format!("WriteProcessMemory failed at 0x{address:X}: {e}"))?;
    }

    if bytes_written != bytes.len() {
        return Err(format!(
            "Partial write: {bytes_written}/{} bytes at 0x{address:X}",
            bytes.len()
        ));
    }

    Ok(())
}

/// Manages frozen addresses — a background thread repeatedly writes
/// the frozen value at a configurable interval.
pub struct ValueFreezer {
    frozen: Arc<Mutex<HashMap<u64, FreezeEntry>>>,
    running: Arc<Mutex<bool>>,
}

struct FreezeEntry {
    value: ScanValue,
}

impl ValueFreezer {
    /// Create a new freezer for the given process handle.
    /// Starts the background write thread immediately.
    pub fn new(handle: HANDLE, interval_ms: u64) -> Self {
        let frozen: Arc<Mutex<HashMap<u64, FreezeEntry>>> = Arc::new(Mutex::new(HashMap::new()));
        let running = Arc::new(Mutex::new(true));

        let frozen_clone = frozen.clone();
        let running_clone = running.clone();
        // Cast HANDLE (*mut c_void) to isize so the closure is Send.
        // Win32 handles are opaque kernel IDs, safe to use from any thread.
        let handle_isize = handle.0 as isize;

        // Background thread that continuously writes frozen values
        thread::spawn(move || {
            let handle = HANDLE(handle_isize as *mut _);
            while *running_clone.lock().unwrap() {
                if let Ok(entries) = frozen_clone.lock() {
                    for (address, entry) in entries.iter() {
                        let bytes = entry.value.to_bytes();
                        unsafe {
                            let _ = WriteProcessMemory(
                                handle,
                                *address as *const _,
                                bytes.as_ptr() as *const _,
                                bytes.len(),
                                None,
                            );
                        }
                    }
                }
                thread::sleep(Duration::from_millis(interval_ms));
            }
        });

        Self {
            frozen,
            running,
        }
    }

    /// Freeze an address — it will be continuously written with the given value
    pub fn freeze(&self, address: u64, value: ScanValue) {
        if let Ok(mut map) = self.frozen.lock() {
            map.insert(address, FreezeEntry { value });
        }
    }

    /// Unfreeze an address
    pub fn unfreeze(&self, address: u64) {
        if let Ok(mut map) = self.frozen.lock() {
            map.remove(&address);
        }
    }

    /// Check if an address is currently frozen
    pub fn is_frozen(&self, address: u64) -> bool {
        self.frozen
            .lock()
            .map(|map| map.contains_key(&address))
            .unwrap_or(false)
    }

    /// Get all currently frozen addresses
    pub fn frozen_addresses(&self) -> Vec<u64> {
        self.frozen
            .lock()
            .map(|map| map.keys().cloned().collect())
            .unwrap_or_default()
    }
}

impl Drop for ValueFreezer {
    fn drop(&mut self) {
        // Signal the background thread to stop
        if let Ok(mut running) = self.running.lock() {
            *running = false;
        }
    }
}
