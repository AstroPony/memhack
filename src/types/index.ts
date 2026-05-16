// ── Value Types ───────────────────────────────────────────────

export type ScanValue =
  | { type: "I8"; value: number }
  | { type: "I16"; value: number }
  | { type: "I32"; value: number }
  | { type: "I64"; value: number }
  | { type: "U8"; value: number }
  | { type: "U16"; value: number }
  | { type: "U32"; value: number }
  | { type: "U64"; value: number }
  | { type: "F32"; value: number }
  | { type: "F64"; value: number }
  | { type: "StringUtf8"; value: string }
  | { type: "StringUtf16"; value: string }
  | { type: "ByteArray"; value: number[] };

export type ValueTypeName =
  | "I8" | "I16" | "I32" | "I64"
  | "U8" | "U16" | "U32" | "U64"
  | "F32" | "F64"
  | "StringUtf8" | "StringUtf16";

// ── Scan Filter ──────────────────────────────────────────────

export type ScanFilter =
  | { ExactValue: ScanValue }
  | "Changed"
  | "Unchanged"
  | "Increased"
  | "Decreased"
  | { GreaterThan: ScanValue }
  | { LessThan: ScanValue };

// ── Results ──────────────────────────────────────────────────

export interface ProcessInfo {
  pid: number;
  name: string;
}

export interface FoundAddress {
  address: number; // u64 — JS handles up to 2^53 safely
  current_value: ScanValue;
  previous_bytes: number[];
}

export interface ScanSummary {
  found: number;
  scan_number: number;
}

export interface TableEntry {
  label: string;
  address: number;
  value_type: ValueTypeName;
  frozen: boolean;
  freeze_value: ScanValue | null;
}

// ── UI State ─────────────────────────────────────────────────

export interface AppState {
  // Connection
  attached: ProcessInfo | null;

  // Scanning
  scanActive: boolean;
  scanCount: number;
  resultCount: number;
  results: FoundAddress[];

  // Address table
  table: TableEntry[];

  // UI
  selectedValueType: ValueTypeName;
  scanInput: string;
}
