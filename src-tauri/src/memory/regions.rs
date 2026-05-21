use crate::memory::process::ProcessHandle;
use crate::memory::types::MemoryRegion;
use windows::Win32::System::Memory::{
    VirtualQueryEx, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_GUARD, PAGE_NOACCESS,
    PAGE_PROTECTION_FLAGS,
};

/// Enumerate all readable, committed memory regions in the target process
pub fn enumerate_regions(proc: &ProcessHandle) -> Vec<MemoryRegion> {
    let mut regions = Vec::new();
    let mut address: usize = 0;
    let mut mbi = MEMORY_BASIC_INFORMATION::default();

    unsafe {
        loop {
            let result = VirtualQueryEx(
                proc.handle,
                Some(address as *const _),
                &mut mbi,
                std::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
            );

            if result == 0 {
                break;
            }

            // Only scan committed, non-guarded, accessible memory
            let is_committed = mbi.State == MEM_COMMIT;
            let is_accessible =
                mbi.Protect != PAGE_NOACCESS && (mbi.Protect & PAGE_GUARD) == PAGE_PROTECTION_FLAGS(0);

            if is_committed && is_accessible && mbi.RegionSize > 0 {
                regions.push(MemoryRegion {
                    base_address: mbi.BaseAddress as u64,
                    size: mbi.RegionSize as u64,
                });
            }

            // Advance to next region
            address = mbi.BaseAddress as usize + mbi.RegionSize;

            // Safety: prevent infinite loop on overflow
            if address <= mbi.BaseAddress as usize {
                break;
            }
        }
    }

    regions
}
