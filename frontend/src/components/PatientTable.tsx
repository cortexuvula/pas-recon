import { useState, useMemo } from "react";
import type { DisplayRow } from "../types";

interface PatientTableProps {
  rows: DisplayRow[];
  showStatus: boolean;
  resolvedSet: Set<string>;
  onToggleResolved: (phn: string) => void;
  searchQuery: string;
}

export default function PatientTable({
  rows, showStatus, resolvedSet, onToggleResolved, searchQuery
}: PatientTableProps) {
  const filtered = useMemo(() => {
    if (!searchQuery.trim()) return rows;
    const q = searchQuery.toLowerCase();
    return rows.filter((r) =>
      r.phn.toLowerCase().includes(q) ||
      (r.first_name?.toLowerCase().includes(q) ?? false) ||
      (r.last_name?.toLowerCase().includes(q) ?? false)
    );
  }, [rows, searchQuery]);

  if (filtered.length === 0) {
    return (
      <div className="empty-state">
        {rows.length === 0 ? "No patients in this list." : "No matches for your search."}
      </div>
    );
  }

  return (
    <div style={{ flex: 1, overflow: "auto", padding: "0 16px" }}>
      <table>
        <thead>
          <tr>
            <th>PHN</th>
            <th>First Name</th>
            <th>Last Name</th>
            <th>DOB</th>
            {showStatus && <th>Status</th>}
          </tr>
        </thead>
        <tbody>
          {filtered.map((row) => (
            <tr
              key={row.phn}
              className={resolvedSet.has(row.phn) ? "resolved" : ""}
              onClick={() => onToggleResolved(row.phn)}
              style={{ cursor: "pointer" }}
            >
              <td className="phn">{row.phn}</td>
              <td>{row.first_name ?? "—"}</td>
              <td>{row.last_name ?? "—"}</td>
              <td>{row.dob ?? "—"}</td>
              {showStatus && <td>{row.mrp_status ?? "—"}</td>}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
