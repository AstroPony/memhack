use crate::memory::types::ProcessInfo;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_ACCESS_RIGHTS};

/// Required access rights for memory reading and writing
const PROCESS_ACCESS: PROCESS_ACCESS_RIGHTS = PROCESS_ACCESS_RIGHTS(
    0x0010  // PROCESS_VM_READ
    | 0x0020 // PROCESS_VM_WRITE
    | 0x0008 // PROCESS_VM_OPERATION
    | 0x0400 // PROCESS_QUERY_INFORMATION
);

/// Wrapper around a Win32 process handle that auto-closes on drop
pub struct ProcessHandle {
    pub handle: HANDLE,
    pub pid: u32,
    pub name: String,
}

impl Drop for ProcessHandle {
    fn drop(&mut self) {
        if !self.handle.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.handle);
            }
        }
    }
}

/// Enumerate all running processes
pub fn list_processes() -> Result<Vec<ProcessInfo>, String> {
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
            .map_err(|e| format!("CreateToolhelp32Snapshot failed: {e}"))?;

        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };

        let mut processes = Vec::new();

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let name = String::from_utf16_lossy(
                    &entry.szExeFile[..entry
                        .szExeFile
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(entry.szExeFile.len())],
                );

                // Skip system idle process
                if entry.th32ProcessID != 0 {
                    processes.push(ProcessInfo {
                        pid: entry.th32ProcessID,
                        name,
                    });
                }

                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }

        let _ = CloseHandle(snapshot);

        // Sort by name for easier browsing
        processes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Ok(processes)
    }
}

/// Open a process by PID with memory read/write access
pub fn attach_process(pid: u32, name: String) -> Result<ProcessHandle, String> {
    unsafe {
        let handle = OpenProcess(PROCESS_ACCESS, false, pid)
            .map_err(|e| format!("OpenProcess failed (pid {pid}): {e}. Run as Administrator?"))?;

        Ok(ProcessHandle { handle, pid, name })
    }
}
