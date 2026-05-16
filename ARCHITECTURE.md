# MemHack — Personal Memory Editor

## Overview

A Tauri + Rust memory editor for Windows. Modern web UI, fast Rust backend for process memory operations. Built for personal use — no telemetry, no accounts, no bullshit.

## Stack

| Layer | Tech | Why |
|-------|------|-----|
| UI | React 18 + TypeScript + Tailwind | Familiar, fast to iterate |
| Desktop shell | Tauri v2 | Tiny binary, native perf, no Electron bloat |
| Memory engine | Rust + `windows` crate | Direct Win32 API, safe abstractions, fast scanning |
| State | Zustand | Lightweight, no boilerplate |
| Build | Vite | Fast HMR for dev |

## Win32 APIs Used

All accessed via the `windows` Rust crate:

```
CreateToolhelp32Snapshot  → enumerate processes
Process32First / Next     → iterate process list
OpenProcess               → attach with VM_READ | VM_WRITE | VM_OPERATION | QUERY_INFORMATION
VirtualQueryEx            → map readable memory regions
ReadProcessMemory         → read values from target
WriteProcessMemory        → modify values in target
CloseHandle               → cleanup
```

That's it. ~7 API calls power the entire tool.

## Architecture

```
┌─────────────────────────────────────┐
│           React Frontend            │
│  ┌─────────┐ ┌──────┐ ┌─────────┐  │
│  │ Process  │ │ Scan │ │ Address │  │
│  │ Selector │ │ View │ │  Table  │  │
│  └────┬─────┘ └──┬───┘ └────┬────┘  │
│       │          │           │       │
│  ─────┴──────────┴───────────┴────── │
│           Tauri invoke() bridge      │
└──────────────────┬───────────────────┘
                   │ IPC (JSON)
┌──────────────────┴───────────────────┐
│           Rust Backend               │
│  ┌──────────────────────────────┐    │
│  │      Tauri Commands          │    │
│  │  #[tauri::command] handlers  │    │
│  └──────────┬───────────────────┘    │
│             │                        │
│  ┌──────────┴───────────────────┐    │
│  │      Memory Engine           │    │
│  │  • ProcessHandle             │    │
│  │  • MemoryScanner             │    │
│  │  • ValueFreezer (bg thread)  │    │
│  │  • PointerScanner            │    │
│  │  • AddressTable (save/load)  │    │
│  └──────────────────────────────┘    │
└──────────────────────────────────────┘
```

## Data Types

```rust
enum ScanValue {
    I8(i8), I16(i16), I32(i32), I64(i64),
    U8(u8), U16(u16), U32(u32), U64(u64),
    F32(f32), F64(f64),
    StringUtf8(String),
    StringUtf16(String),
    ByteArray(Vec<u8>),
}
```

## Scan Modes

| Mode | Description |
|------|-------------|
| Exact Value | Find all addresses containing X |
| Range | Find values between A and B |
| Unknown Initial | Mark all readable addresses, then filter |
| Changed | Value differs from last scan |
| Unchanged | Value same as last scan |
| Increased | Value > last scan |
| Decreased | Value < last scan |
| Increased By | Value == last + N |
| Decreased By | Value == last - N |

## Roadmap

### Phase 1 — Core Scanner (Week 1-2)
- [ ] Tauri project scaffolding with React + Vite
- [ ] Process enumeration and selection
- [ ] Memory region mapping (VirtualQueryEx)
- [ ] First scan (exact value, i32)
- [ ] Next scan (filter changed/unchanged)
- [ ] Write single value
- [ ] Basic UI: process list, scan panel, results table

### Phase 2 — Full Data Types + Address Table (Week 3)
- [ ] All integer types (i8–i64, u8–u64)
- [ ] Float types (f32, f64)
- [ ] String scanning (UTF-8 + UTF-16)
- [ ] Hex / byte pattern scanning
- [ ] Address table (add, remove, edit, rename)
- [ ] Save/load address tables as JSON per game
- [ ] All scan modes (range, unknown, increased by, etc.)

### Phase 3 — Freeze + Polish (Week 4)
- [ ] Value freezing (background Rust thread, configurable interval)
- [ ] Freeze toggle per address in the table
- [ ] Scan progress indicator (large games = lots of memory)
- [ ] Keyboard shortcuts
- [ ] Dark theme refinement
- [ ] Hex viewer for inspecting memory around an address

### Phase 4 — Advanced (Future)
- [ ] Pointer scanning (multi-level)
- [ ] Pointer maps / chain resolution
- [ ] Assembly view / instruction patching (stretch)
- [ ] Emulator support (PCSX2, Dolphin, etc. — just scan their process)
- [ ] Hotkeys (global, even when game is focused)

## Running

```bash
# Prerequisites: Rust toolchain, Node.js 18+, Tauri CLI
npm install
npm run tauri dev     # dev mode with HMR
npm run tauri build   # production binary
```

**Important**: Must run as Administrator for memory access to most game processes.

## File Structure

```
memhack/
├── ARCHITECTURE.md
├── package.json
├── vite.config.ts
├── tsconfig.json
├── tailwind.config.js
├── index.html
├── src/                          # Frontend (React + TS)
│   ├── main.tsx
│   ├── App.tsx
│   ├── components/
│   │   ├── ProcessSelector.tsx
│   │   ├── ScanPanel.tsx
│   │   ├── ResultsTable.tsx
│   │   ├── AddressTable.tsx
│   │   └── HexViewer.tsx
│   ├── hooks/
│   │   └── useMemory.ts
│   ├── types/
│   │   └── index.ts
│   └── lib/
│       └── commands.ts           # Typed Tauri invoke wrappers
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── commands/
│   │   │   └── mod.rs            # #[tauri::command] handlers
│   │   └── memory/
│   │       ├── mod.rs
│   │       ├── process.rs        # Process enumeration
│   │       ├── scanner.rs        # Memory scanning engine
│   │       ├── writer.rs         # Write + freeze
│   │       ├── types.rs          # ScanValue, ScanMode, etc.
│   │       ├── regions.rs        # VirtualQueryEx wrapper
│   │       └── table.rs          # Address table persistence
```
