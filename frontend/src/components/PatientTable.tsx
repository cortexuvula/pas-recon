import { useMemo, useState } from "react";
import type { DisplayRow } from "../types";

type SortKey = "phn" | "first_name" | "last_name" | "dob" | "mrp_status" | "source";
type SortDir = "asc" | "desc";

interface PatientTableProps {
  rows: DisplayRow[];
  showStatus: boolean;
  showSource: boolean;
  resolvedSet: Set<string>;
  onToggleResolved: (phn: string) => void;
  searchQuery: string;
}

export default function PatientTable({
  rows, showStatus, showSource, resolvedSet, onToggleResolved, searchQuery
}: PatientTableProps) {
  const [sortKey, setSortKey] = useState<SortKey>("last_name");
  const [sortDir, setSortDir] = useState<SortDir>("asc");

  const handleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortDir(prev => prev === "asc" ? "desc" : "asc");
    } else {
      setSortKey(key);
      setSortDir("asc");
    }
  };

  const filtered = useMemo(() => {
    let result = rows;
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      result = result.filter((r) =>
        r.phn.toLowerCase().includes(q) ||
        (r.first_name?.toLowerCase().includes(q) ?? false) ||
        (r.last_name?.toLowerCase().includes(q) ?? false) ||
        (r.source?.toLowerCase().includes(q) ?? false)
      );
    }
    return result;
  }, [rows, searchQuery]);

  const sorted = useMemo(() => {
    const getVal = (r: DisplayRow): string => {
      const v = r[sortKey];
      return (v ?? "").toLowerCase();
    };
    return [...filtered].sort((a, b) => {
      const av = getVal(a);
      const bv = getVal(b);
      // Empty/null values sort to bottom in ascending, top in descending
      if (!av && !bv) return 0;
      if (!av) return 1;
      if (!bv) return -1;
      const cmp = av.localeCompare(bv);
      return sortDir === "asc" ? cmp : -cmp;
    });
  }, [filtered, sortKey, sortDir]);

  const SortHeader = ({ label, sortKey: key }: { label: string; sortKey: SortKey }) => (
    <th>
      <button
        type="button"
        onClick={() => handleSort(key)}
        style={{
          background: "none",
          border: "none",
          color: sortKey === key ? "var(--text)" : "var(--text-faint)",
          cursor: "pointer",
          font: "inherit",
          padding: 0,
          textAlign: "left",
          fontWeight: sortKey === key ? 700 : 500,
          fontSize: "12px",
          whiteSpace: "nowrap",
        }}
      >
        {label}
        {sortKey === key && (sortDir === "asc" ? " \u25B2" : " \u25BC")}
      </button>
    </th>
  );

  if (sorted.length === 0) {
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
            {showSource && <SortHeader label="Source" sortKey="source" />}
            <SortHeader label="PHN" sortKey="phn" />
            <SortHeader label="First Name" sortKey="first_name" />
            <SortHeader label="Last Name" sortKey="last_name" />
            <SortHeader label="DOB" sortKey="dob" />
            {showStatus && <SortHeader label="Status" sortKey="mrp_status" />}
          </tr>
        </thead>
        <tbody>
          {sorted.map((row, i) => {
            const isResolved = resolvedSet.has(row.phn);
            return (
            <tr
              key={`${row.phn}-${i}`}
              className={isResolved ? "resolved" : ""}
              tabIndex={0}
              role="switch"
              aria-checked={isResolved}
              aria-label={`${row.phn} ${row.first_name ?? ""} ${row.last_name ?? ""}`}
              onClick={() => onToggleResolved(row.phn)}
              onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); onToggleResolved(row.phn); } }}
              style={{ cursor: "pointer" }}
            >
              {showSource && (
                <td style={{ fontWeight: 600, color: row.source === "EMR" ? "var(--amber)" : "var(--blue)" }}>
                  {row.source ?? "\u2014"}
                </td>
              )}
              <td className="phn">{row.phn}</td>
              <td>{row.first_name ?? "\u2014"}</td>
              <td>{row.last_name ?? "\u2014"}</td>
              <td>{row.dob ?? "\u2014"}</td>
              {showStatus && <td>{row.mrp_status ?? "\u2014"}</td>}
            </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
