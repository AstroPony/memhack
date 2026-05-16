use crate::memory::process::ProcessHandle;
use crate::memory::regions::enumerate_regions;
use crate::memory::types::*;
use windows::Win32::System::Diagnostics::Debug::ReadProcessMemory;

/// Read a chunk of memory from the target process
fn read_memory(proc: &ProcessHandle, address: u64, size: usize) -> Option<Vec<u8>> {
    let mut buffer = vec![0u8; size];
    let mut bytes_read = 0usize;

    unsafe {
        ReadProcessMemory(
            proc.handle,
            address as *const _,
            buffer.as_mut_ptr() as *mut _,
            size,
            Some(&mut bytes_read),
        )
        .ok()?;
    }

    buffer.truncate(bytes_read);
    if bytes_read > 0 {
        Some(buffer)
    } else {
        None
    }
}

/// First scan: find all addresses in the process that match `target`
///
/// This is the initial broad scan that reads every readable memory region.
/// Returns could be millions of results for small value types — the UI
/// should limit display and encourage narrowing.
pub fn first_scan(
    proc: &ProcessHandle,
    target: &ScanValue,
    progress_callback: Option<&dyn Fn(f64)>,
) -> Vec<FoundAddress> {
    let regions = enumerate_regions(proc);
    let total_regions = regions.len();
    let target_bytes = target.to_bytes();
    let value_size = target.byte_size();
    let alignment = match target {
        // Align to natural boundaries for faster scanning
        ScanValue::I16(_) | ScanValue::U16(_) => 2,
        ScanValue::I32(_) | ScanValue::U32(_) | ScanValue::F32(_) => 4,
        ScanValue::I64(_) | ScanValue::U64(_) | ScanValue::F64(_) => 8,
        _ => 1, // Strings and bytes: scan every offset
    };

    let mut results = Vec::new();

    // Read each region in chunks to avoid huge allocations
    const CHUNK_SIZE: usize = 4 * 1024 * 1024; // 4 MB chunks

    for (region_idx, region) in regions.iter().enumerate() {
        if let Some(cb) = progress_callback {
            cb(region_idx as f64 / total_regions as f64);
        }

        let mut offset: u64 = 0;
        while offset < region.size {
            let read_size = std::cmp::min(CHUNK_SIZE as u64, region.size - offset) as usize;
            let base = region.base_address + offset;

            if let Some(buffer) = read_memory(proc, base, read_size) {
                // Scan this chunk for matches
                let mut pos = 0;
                while pos + value_size <= buffer.len() {
                    if target.matches_bytes(&buffer, pos) {
                        results.push(FoundAddress {
                            address: base + pos as u64,
                            current_value: target.clone(),
                            previous_bytes: buffer[pos..pos + value_size].to_vec(),
                        });
                    }
                    pos += alignment;
                }
            }

            offset += read_size as u64;
        }
    }

    results
}

/// Next scan: re-read addresses from previous results and apply a filter
///
/// This is how you narrow down from millions of results to a handful.
/// The game value changes → you tell the scanner how it changed → it
/// eliminates addresses that don't match.
pub fn next_scan(
    proc: &ProcessHandle,
    previous: &[FoundAddress],
    filter: &ScanFilter,
) -> Vec<FoundAddress> {
    let mut results = Vec::new();

    for entry in previous {
        let size = entry.current_value.byte_size();

        // Re-read the current value at this address
        let Some(current_bytes) = read_memory(proc, entry.address, size) else {
            continue; // Address no longer readable — process might have freed it
        };

        let matches = match filter {
            ScanFilter::ExactValue(target) => target.matches_bytes(&current_bytes, 0),

            ScanFilter::Changed => current_bytes != entry.previous_bytes,

            ScanFilter::Unchanged => current_bytes == entry.previous_bytes,

            ScanFilter::Increased => compare_bytes_numeric(
                &current_bytes,
                &entry.previous_bytes,
                &entry.current_value,
                |curr, prev| curr > prev,
            ),

            ScanFilter::Decreased => compare_bytes_numeric(
                &current_bytes,
                &entry.previous_bytes,
                &entry.current_value,
                |curr, prev| curr < prev,
            ),

            ScanFilter::GreaterThan(target) => {
                let target_bytes = target.to_bytes();
                compare_bytes_numeric(
                    &current_bytes,
                    &target_bytes,
                    &entry.current_value,
                    |curr, threshold| curr > threshold,
                )
            }

            ScanFilter::LessThan(target) => {
                let target_bytes = target.to_bytes();
                compare_bytes_numeric(
                    &current_bytes,
                    &target_bytes,
                    &entry.current_value,
                    |curr, threshold| curr < threshold,
                )
            }

            // TODO: IncreasedBy, DecreasedBy, Between
            _ => false,
        };

        if matches {
            // Read the actual current value for display
            let current_value = entry
                .current_value
                .read_from_buffer(&current_bytes, 0)
                .unwrap_or_else(|| entry.current_value.clone());

            results.push(FoundAddress {
                address: entry.address,
                current_value,
                previous_bytes: current_bytes,
            });
        }
    }

    results
}

/// Read the current value at a single address
pub fn read_value(proc: &ProcessHandle, address: u64, value_type: &ValueType) -> Option<ScanValue> {
    let size = value_type.byte_size();
    let buffer = read_memory(proc, address, size)?;
    let template = value_type.default_value();
    template.read_from_buffer(&buffer, 0)
}

/// Helper: compare two byte buffers as numeric values using the given comparator
fn compare_bytes_numeric(
    a: &[u8],
    b: &[u8],
    type_hint: &ScanValue,
    cmp: fn(f64, f64) -> bool,
) -> bool {
    let a_val = bytes_to_f64(a, type_hint);
    let b_val = bytes_to_f64(b, type_hint);
    match (a_val, b_val) {
        (Some(a), Some(b)) => cmp(a, b),
        _ => false,
    }
}

/// Convert raw bytes to f64 for comparison purposes
fn bytes_to_f64(bytes: &[u8], type_hint: &ScanValue) -> Option<f64> {
    Some(match type_hint {
        ScanValue::I8(_) => i8::from_le_bytes(bytes.try_into().ok()?) as f64,
        ScanValue::I16(_) => i16::from_le_bytes(bytes.try_into().ok()?) as f64,
        ScanValue::I32(_) => i32::from_le_bytes(bytes.try_into().ok()?) as f64,
        ScanValue::I64(_) => i64::from_le_bytes(bytes.try_into().ok()?) as f64,
        ScanValue::U8(_) => u8::from_le_bytes(bytes.try_into().ok()?) as f64,
        ScanValue::U16(_) => u16::from_le_bytes(bytes.try_into().ok()?) as f64,
        ScanValue::U32(_) => u32::from_le_bytes(bytes.try_into().ok()?) as f64,
        ScanValue::U64(_) => u64::from_le_bytes(bytes.try_into().ok()?) as f64,
        ScanValue::F32(_) => f32::from_le_bytes(bytes.try_into().ok()?) as f64,
        ScanValue::F64(_) => f64::from_le_bytes(bytes.try_into().ok()?),
        _ => return None, // Strings/bytes don't have numeric comparison
    })
}
