import { useState, useEffect } from "react";
import { getCsvHeaders } from "../api";

interface ColumnPickerProps {
  emrPath: string;
  pasPath: string;
  onResolved: (emrCol: number, pasCol: number) => void;
  onCancel: () => void;
}

export default function ColumnPicker({ emrPath, pasPath, onResolved, onCancel }: ColumnPickerProps) {
  const [emrHeaders, setEmrHeaders] = useState<string[]>([]);
  const [pasHeaders, setPasHeaders] = useState<string[]>([]);
  const [emrCol, setEmrCol] = useState(0);
  const [pasCol, setPasCol] = useState(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    // Sequence header fetches so a slow response for an older path can't
    // overwrite headers from a newer path, and so we don't update state after
    // unmount. `cancelled` covers both cases.
    let cancelled = false;
    setLoading(true);
    setError(null);
    Promise.all([getCsvHeaders(emrPath), getCsvHeaders(pasPath)])
      .then(([emr, pas]) => {
        if (cancelled) return;
        setEmrHeaders(emr);
        setPasHeaders(pas);
        setLoading(false);
      })
      .catch((e) => {
        if (cancelled) return;
        // Normalize the rejection into a stable string regardless of shape.
        const msg = e instanceof Error ? e.message : String(e);
        setError(`Failed to read file headers: ${msg}`);
        setLoading(false);
      });
    return () => { cancelled = true; };
  }, [emrPath, pasPath]);

  if (loading) return <div className="empty-state">Loading headers…</div>;

  if (error) return (
    <div style={{ padding: "24px" }}>
      <div className="error-banner">{error}</div>
      <button type="button" className="tab" onClick={onCancel}>Back</button>
    </div>
  );

  // Disable confirmation when either file has no selectable columns; otherwise
  // the picker would silently emit index 0 (no meaningful column).
  const noEmr = emrHeaders.length === 0;
  const noPas = pasHeaders.length === 0;
  const confirmDisabled = noEmr || noPas;

  return (
    <div style={{ padding: "24px", maxWidth: "600px", margin: "0 auto" }}>
      <h2 style={{ fontSize: "16px", marginBottom: "8px" }}>Select PHN Columns</h2>
      <p style={{ fontSize: "11px", color: "var(--text-dim)", marginBottom: "20px" }}>
        We couldn't auto-detect the PHN column in one or both files. Please identify them manually.
      </p>
      <div style={{ display: "flex", gap: "24px" }}>
        <div style={{ flex: 1 }}>
          <label style={{ fontSize: "11px", color: "var(--text-dim)" }}>EMR PHN column:</label>
          <select
            value={emrCol}
            onChange={(e) => setEmrCol(Number(e.target.value))}
            className="search-input"
            style={{ display: "block", marginTop: "4px" }}
            disabled={noEmr}
          >
            {emrHeaders.map((h, i) => (
              <option key={i} value={i}>{i}: {h}</option>
            ))}
          </select>
          {noEmr && (
            <div style={{ fontSize: "11px", color: "var(--red)", marginTop: "4px" }}>
              No columns found in EMR file.
            </div>
          )}
        </div>
        <div style={{ flex: 1 }}>
          <label style={{ fontSize: "11px", color: "var(--text-dim)" }}>PAS PHN column:</label>
          <select
            value={pasCol}
            onChange={(e) => setPasCol(Number(e.target.value))}
            className="search-input"
            style={{ display: "block", marginTop: "4px" }}
            disabled={noPas}
          >
            {pasHeaders.map((h, i) => (
              <option key={i} value={i}>{i}: {h}</option>
            ))}
          </select>
          {noPas && (
            <div style={{ fontSize: "11px", color: "var(--red)", marginTop: "4px" }}>
              No columns found in PAS file.
            </div>
          )}
        </div>
      </div>
      <div style={{ marginTop: "24px", display: "flex", gap: "8px", justifyContent: "flex-end" }}>
        <button type="button" className="tab" onClick={onCancel}>Cancel</button>
        <button
          type="button"
          className="export-btn"
          onClick={() => onResolved(emrCol, pasCol)}
          disabled={confirmDisabled}
          style={confirmDisabled ? { opacity: 0.5, cursor: "not-allowed" } : undefined}
        >
          Confirm
        </button>
      </div>
    </div>
  );
}
