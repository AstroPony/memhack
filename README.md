# MemHack

Personal memory editor for Windows games. Attach to a process, scan for values (gold, health, ammo), narrow results by rescanning, then edit or freeze them. Think ArtMoney / Cheat Engine but with a modern stack.

## Stack

| Layer | Tech |
|-------|------|
| UI | React 18 + TypeScript + Tailwind CSS |
| Desktop shell | Tauri v2 (Rust, not Electron) |
| Memory engine | Rust + `windows` crate (Win32 APIs) |
| Build | Vite |

## Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable)
- Node.js 18+
- Run as **Administrator** (required for `OpenProcess` on most game processes)

## Running

```bash
npm install
npm run tauri dev      # dev mode with HMR
npm run tauri build    # production binary
```

## Features (v0.1.0)

- Process enumeration and attachment
- Memory region mapping via `VirtualQueryEx`
- First scan — exact value match across all readable memory (4 MB chunk reads)
- Next scan — exact, changed, unchanged, increased, decreased filters
- Write single value to any address
- Value freezer — background thread holds values at 20 Hz
- Address table — collect addresses to watch/edit
- All numeric types: i8/i16/i32/i64, u8/u16/u32/u64, f32/f64, UTF-8/UTF-16 strings

## Roadmap

See [GitHub Issues](../../issues) for the prioritised feature backlog.
