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
    Promise.all([getCsvHeaders(emrPath), getCsvHeaders(pasPath)])
      .then(([emr, pas]) => {
        setEmrHeaders(emr);
        setPasHeaders(pas);
        setLoading(false);
      })
      .catch((e) => {
        setError(`Failed to read file headers: ${e}`);
        setLoading(false);
      });
  }, [emrPath, pasPath]);

  if (loading) return <div className="empty-state">Loading headers…</div>;

  if (error) return (
    <div style={{ padding: "24px" }}>
      <div className="error-banner">{error}</div>
      <button type="button" className="tab" onClick={onCancel}>Back</button>
    </div>
  );

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
          >
            {emrHeaders.map((h, i) => (
              <option key={i} value={i}>{i}: {h}</option>
            ))}
          </select>
        </div>
        <div style={{ flex: 1 }}>
          <label style={{ fontSize: "11px", color: "var(--text-dim)" }}>PAS PHN column:</label>
          <select
            value={pasCol}
            onChange={(e) => setPasCol(Number(e.target.value))}
            className="search-input"
            style={{ display: "block", marginTop: "4px" }}
          >
            {pasHeaders.map((h, i) => (
              <option key={i} value={i}>{i}: {h}</option>
            ))}
          </select>
        </div>
      </div>
      <div style={{ marginTop: "24px", display: "flex", gap: "8px", justifyContent: "flex-end" }}>
        <button type="button" className="tab" onClick={onCancel}>Cancel</button>
        <button type="button" className="export-btn" onClick={() => onResolved(emrCol, pasCol)}>Confirm</button>
      </div>
    </div>
  );
}
