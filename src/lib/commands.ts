import { invoke } from "@tauri-apps/api/core";
import type {
  ProcessInfo,
  ScanValue,
  ScanFilter,
  ScanSummary,
  FoundAddress,
  TableEntry,
} from "../types";

// ── Process ──────────────────────────────────────────────────

export const listProcesses = () =>
  invoke<ProcessInfo[]>("list_processes");

export const attachProcess = (pid: number, name: string) =>
  invoke<string>("attach_process", { pid, name });

export const detachProcess = () =>
  invoke<void>("detach_process");

// ── Scanning ─────────────────────────────────────────────────

export const firstScan = (value: ScanValue) =>
  invoke<ScanSummary>("first_scan", { value });

export const nextScan = (filter: ScanFilter) =>
  invoke<ScanSummary>("next_scan", { filter });

export const getScanResults = (offset: number, limit: number) =>
  invoke<FoundAddress[]>("get_scan_results", { offset, limit });

export const resetScan = () =>
  invoke<void>("reset_scan");

// ── Writing ──────────────────────────────────────────────────

export const writeValue = (address: number, value: ScanValue) =>
  invoke<void>("write_value", { address, value });

export const freezeValue = (address: number, value: ScanValue) =>
  invoke<void>("freeze_value", { address, value });

export const unfreezeValue = (address: number) =>
  invoke<void>("unfreeze_value", { address });

// ── Address Tables ───────────────────────────────────────────

export const saveAddressTable = (gameName: string, entries: TableEntry[]) =>
  invoke<string>("save_address_table", { gameName, entries });

export const loadAddressTable = (gameName: string) =>
  invoke<TableEntry[]>("load_address_table", { gameName });

export const listSavedTables = () =>
  invoke<string[]>("list_saved_tables");

// ── Helpers ──────────────────────────────────────────────────

/** Build a ScanValue from a type name and raw input string */
export function parseScanValue(
  type_name: string,
  input: string
): ScanValue | null {
  const num = Number(input);

  switch (type_name) {
    case "I8":  return { type: "I8", value: num };
    case "I16": return { type: "I16", value: num };
    case "I32": return { type: "I32", value: num };
    case "I64": return { type: "I64", value: num };
    case "U8":  return { type: "U8", value: num };
    case "U16": return { type: "U16", value: num };
    case "U32": return { type: "U32", value: num };
    case "U64": return { type: "U64", value: num };
    case "F32": return { type: "F32", value: num };
    case "F64": return { type: "F64", value: num };
    case "StringUtf8":  return { type: "StringUtf8", value: input };
    case "StringUtf16": return { type: "StringUtf16", value: input };
    default: return null;
  }
}

/** Format an address as hex */
export function formatAddress(addr: number): string {
  return "0x" + addr.toString(16).toUpperCase().padStart(8, "0");
}

/** Extract the display value from a ScanValue */
export function displayValue(sv: ScanValue): string {
  return String(sv.value);
}
