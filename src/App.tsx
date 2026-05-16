import { useState, useEffect, useCallback } from "react";
import type {
  ProcessInfo,
  FoundAddress,
  ScanValue,
  ScanFilter,
  ValueTypeName,
  TableEntry,
} from "./types";
import {
  listProcesses,
  attachProcess,
  detachProcess,
  firstScan,
  nextScan,
  getScanResults,
  resetScan,
  writeValue,
  freezeValue,
  unfreezeValue,
  parseScanValue,
  formatAddress,
  displayValue,
} from "./lib/commands";

// ── Value type options ───────────────────────────────────────

const VALUE_TYPES: { value: ValueTypeName; label: string }[] = [
  { value: "I32", label: "Int 32" },
  { value: "I16", label: "Int 16" },
  { value: "I64", label: "Int 64" },
  { value: "F32", label: "Float" },
  { value: "F64", label: "Double" },
  { value: "U32", label: "UInt 32" },
  { value: "U16", label: "UInt 16" },
  { value: "U8", label: "Byte" },
  { value: "StringUtf8", label: "String" },
];

// ── Main App ─────────────────────────────────────────────────

export default function App() {
  // Process state
  const [processes, setProcesses] = useState<ProcessInfo[]>([]);
  const [processFilter, setProcessFilter] = useState("");
  const [attached, setAttached] = useState<ProcessInfo | null>(null);

  // Scan state
  const [valueType, setValueType] = useState<ValueTypeName>("I32");
  const [scanInput, setScanInput] = useState("");
  const [scanCount, setScanCount] = useState(0);
  const [resultCount, setResultCount] = useState(0);
  const [results, setResults] = useState<FoundAddress[]>([]);
  const [scanning, setScanning] = useState(false);

  // Address table
  const [table, setTable] = useState<TableEntry[]>([]);

  // Status bar
  const [status, setStatus] = useState("Ready — select a process");

  // ── Process List ─────────────────────────────────────────

  const refreshProcesses = useCallback(async () => {
    try {
      const procs = await listProcesses();
      setProcesses(procs);
    } catch (e) {
      setStatus(`Error listing processes: ${e}`);
    }
  }, []);

  useEffect(() => {
    refreshProcesses();
  }, [refreshProcesses]);

  const handleAttach = async (proc: ProcessInfo) => {
    try {
      const msg = await attachProcess(proc.pid, proc.name);
      setAttached(proc);
      setScanCount(0);
      setResultCount(0);
      setResults([]);
      setStatus(msg);
    } catch (e) {
      setStatus(`Attach failed: ${e}`);
    }
  };

  const handleDetach = async () => {
    try {
      await detachProcess();
      setAttached(null);
      setScanCount(0);
      setResults([]);
      setStatus("Detached");
    } catch (e) {
      setStatus(`Detach error: ${e}`);
    }
  };

  // ── Scanning ─────────────────────────────────────────────

  const handleFirstScan = async () => {
    const value = parseScanValue(valueType, scanInput);
    if (!value) {
      setStatus("Invalid scan value");
      return;
    }

    setScanning(true);
    setStatus("Scanning...");
    try {
      const summary = await firstScan(value);
      setScanCount(summary.scan_number);
      setResultCount(summary.found);

      // Load first page of results
      const page = await getScanResults(0, 200);
      setResults(page);
      setStatus(`Found ${summary.found.toLocaleString()} results`);
    } catch (e) {
      setStatus(`Scan error: ${e}`);
    }
    setScanning(false);
  };

  const handleNextScan = async (filter: ScanFilter) => {
    setScanning(true);
    setStatus("Filtering...");
    try {
      const summary = await nextScan(filter);
      setScanCount(summary.scan_number);
      setResultCount(summary.found);

      const page = await getScanResults(0, 200);
      setResults(page);
      setStatus(`Narrowed to ${summary.found.toLocaleString()} results`);
    } catch (e) {
      setStatus(`Filter error: ${e}`);
    }
    setScanning(false);
  };

  const handleReset = async () => {
    await resetScan();
    setScanCount(0);
    setResultCount(0);
    setResults([]);
    setStatus("Scan reset");
  };

  // ── Write / Freeze ──────────────────────────────────────

  const handleWrite = async (address: number, newVal: string) => {
    const value = parseScanValue(valueType, newVal);
    if (!value) return;
    try {
      await writeValue(address, value);
      setStatus(`Wrote ${newVal} to ${formatAddress(address)}`);
    } catch (e) {
      setStatus(`Write error: ${e}`);
    }
  };

  const handleAddToTable = (addr: FoundAddress) => {
    const entry: TableEntry = {
      label: `Address ${formatAddress(addr.address)}`,
      address: addr.address,
      value_type: valueType,
      frozen: false,
      freeze_value: null,
    };
    setTable((prev) => [...prev, entry]);
  };

  // ── Filtered process list ───────────────────────────────

  const filteredProcesses = processFilter
    ? processes.filter((p) =>
        p.name.toLowerCase().includes(processFilter.toLowerCase())
      )
    : processes;

  // ── Render ──────────────────────────────────────────────

  return (
    <div className="h-screen flex flex-col bg-surface-0 text-gray-200">
      {/* ── Header ────────────────────────────────────── */}
      <header className="flex items-center justify-between px-4 py-2 bg-surface-1 border-b border-surface-3">
        <div className="flex items-center gap-3">
          <h1 className="text-accent font-mono font-semibold text-sm tracking-wider">
            MEMHACK
          </h1>
          {attached && (
            <div className="flex items-center gap-2">
              <span className="w-2 h-2 rounded-full bg-accent animate-pulse" />
              <span className="text-xs text-gray-400 mono">
                {attached.name} (PID {attached.pid})
              </span>
              <button
                onClick={handleDetach}
                className="text-xs text-red-400 hover:text-red-300 ml-2"
              >
                Detach
              </button>
            </div>
          )}
        </div>
        <button
          onClick={refreshProcesses}
          className="text-xs text-gray-500 hover:text-gray-300"
        >
          Refresh Processes
        </button>
      </header>

      <div className="flex flex-1 overflow-hidden">
        {/* ── Left: Process List ──────────────────────── */}
        <aside className="w-56 border-r border-surface-3 flex flex-col bg-surface-1">
          <div className="p-2">
            <input
              type="text"
              placeholder="Filter processes..."
              value={processFilter}
              onChange={(e) => setProcessFilter(e.target.value)}
              className="w-full text-xs"
            />
          </div>
          <div className="flex-1 overflow-y-auto">
            {filteredProcesses.map((proc) => (
              <button
                key={`${proc.pid}-${proc.name}`}
                onClick={() => handleAttach(proc)}
                className={`w-full text-left px-3 py-1.5 text-xs hover:bg-surface-2 transition-colors
                  ${attached?.pid === proc.pid ? "bg-surface-3 text-accent" : "text-gray-400"}`}
              >
                <span className="mono text-gray-600 mr-2">
                  {proc.pid.toString().padStart(5)}
                </span>
                {proc.name}
              </button>
            ))}
          </div>
        </aside>

        {/* ── Center: Scan Panel + Results ────────────── */}
        <main className="flex-1 flex flex-col overflow-hidden">
          {/* Scan Controls */}
          <div className="p-3 bg-surface-1 border-b border-surface-3 flex items-center gap-2 flex-wrap">
            <select
              value={valueType}
              onChange={(e) => setValueType(e.target.value as ValueTypeName)}
              className="text-xs"
            >
              {VALUE_TYPES.map((vt) => (
                <option key={vt.value} value={vt.value}>
                  {vt.label}
                </option>
              ))}
            </select>

            <input
              type="text"
              placeholder="Value to scan..."
              value={scanInput}
              onChange={(e) => setScanInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  scanCount === 0 ? handleFirstScan() : handleNextScan({ ExactValue: parseScanValue(valueType, scanInput)! });
                }
              }}
              className="flex-1 min-w-[120px] text-xs"
              disabled={!attached || scanning}
            />

            {scanCount === 0 ? (
              <button
                onClick={handleFirstScan}
                disabled={!attached || scanning || !scanInput}
                className="px-3 py-1.5 bg-accent text-surface-0 text-xs font-semibold rounded
                  hover:bg-accent-dim disabled:opacity-30 disabled:cursor-not-allowed transition-colors"
              >
                First Scan
              </button>
            ) : (
              <>
                <button
                  onClick={() => {
                    const val = parseScanValue(valueType, scanInput);
                    if (val) handleNextScan({ ExactValue: val });
                  }}
                  disabled={scanning || !scanInput}
                  className="px-3 py-1.5 bg-accent text-surface-0 text-xs font-semibold rounded
                    hover:bg-accent-dim disabled:opacity-30 transition-colors"
                >
                  Exact
                </button>
                <button
                  onClick={() => handleNextScan("Changed")}
                  disabled={scanning}
                  className="px-2 py-1.5 bg-surface-3 text-xs rounded hover:bg-surface-2 transition-colors"
                >
                  Changed
                </button>
                <button
                  onClick={() => handleNextScan("Unchanged")}
                  disabled={scanning}
                  className="px-2 py-1.5 bg-surface-3 text-xs rounded hover:bg-surface-2 transition-colors"
                >
                  Unchanged
                </button>
                <button
                  onClick={() => handleNextScan("Increased")}
                  disabled={scanning}
                  className="px-2 py-1.5 bg-surface-3 text-xs rounded hover:bg-surface-2 transition-colors"
                >
                  ▲
                </button>
                <button
                  onClick={() => handleNextScan("Decreased")}
                  disabled={scanning}
                  className="px-2 py-1.5 bg-surface-3 text-xs rounded hover:bg-surface-2 transition-colors"
                >
                  ▼
                </button>
                <button
                  onClick={handleReset}
                  className="px-2 py-1.5 bg-surface-3 text-red-400 text-xs rounded hover:bg-surface-2 transition-colors"
                >
                  Reset
                </button>
              </>
            )}

            <span className="text-xs text-gray-500 ml-auto mono">
              {resultCount > 0
                ? `${resultCount.toLocaleString()} found (scan #${scanCount})`
                : scanCount > 0
                ? "0 results"
                : ""}
            </span>
          </div>

          {/* Results Table */}
          <div className="flex-1 overflow-y-auto">
            {results.length > 0 ? (
              <table className="w-full text-xs">
                <thead className="sticky top-0 bg-surface-2">
                  <tr className="text-gray-500">
                    <th className="text-left py-2 px-3 font-medium">Address</th>
                    <th className="text-left py-2 px-3 font-medium">Value</th>
                    <th className="text-left py-2 px-3 font-medium">Previous</th>
                    <th className="py-2 px-3 w-20"></th>
                  </tr>
                </thead>
                <tbody>
                  {results.map((r, i) => (
                    <tr
                      key={i}
                      className="border-t border-surface-3 hover:bg-surface-2 transition-colors"
                    >
                      <td className="py-1.5 px-3 mono text-accent-dim">
                        {formatAddress(r.address)}
                      </td>
                      <td className="py-1.5 px-3 mono">
                        {displayValue(r.current_value)}
                      </td>
                      <td className="py-1.5 px-3 mono text-gray-500">
                        {r.previous_bytes
                          .map((b) => b.toString(16).padStart(2, "0"))
                          .join(" ")}
                      </td>
                      <td className="py-1.5 px-3">
                        <button
                          onClick={() => handleAddToTable(r)}
                          className="text-gray-500 hover:text-accent text-xs"
                          title="Add to table"
                        >
                          + Table
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            ) : (
              <div className="flex items-center justify-center h-full text-gray-600 text-sm">
                {!attached
                  ? "Attach to a process to begin"
                  : scanCount === 0
                  ? "Enter a value and hit First Scan"
                  : "No results — try different filter"}
              </div>
            )}
          </div>

          {/* ── Bottom: Address Table ────────────────── */}
          {table.length > 0 && (
            <div className="border-t border-surface-3 max-h-48 overflow-y-auto bg-surface-1">
              <div className="flex items-center px-3 py-1.5 bg-surface-2 sticky top-0">
                <span className="text-xs font-semibold text-gray-400">
                  ADDRESS TABLE
                </span>
                <span className="text-xs text-gray-600 ml-2">
                  ({table.length})
                </span>
              </div>
              <table className="w-full text-xs">
                <tbody>
                  {table.map((entry, i) => (
                    <tr
                      key={i}
                      className="border-t border-surface-3 hover:bg-surface-2"
                    >
                      <td className="py-1.5 px-3 w-44">
                        <input
                          type="text"
                          value={entry.label}
                          onChange={(e) => {
                            const updated = [...table];
                            updated[i] = { ...entry, label: e.target.value };
                            setTable(updated);
                          }}
                          className="bg-transparent border-none text-xs text-gray-300 w-full p-0"
                        />
                      </td>
                      <td className="py-1.5 px-3 mono text-accent-dim">
                        {formatAddress(entry.address)}
                      </td>
                      <td className="py-1.5 px-3 mono">{entry.value_type}</td>
                      <td className="py-1.5 px-3">
                        <button
                          onClick={() =>
                            setTable(table.filter((_, idx) => idx !== i))
                          }
                          className="text-red-400/60 hover:text-red-400 text-xs"
                        >
                          ✕
                        </button>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </main>
      </div>

      {/* ── Status Bar ────────────────────────────────── */}
      <footer className="px-4 py-1.5 bg-surface-1 border-t border-surface-3 text-xs text-gray-500 mono">
        {status}
      </footer>
    </div>
  );
}
