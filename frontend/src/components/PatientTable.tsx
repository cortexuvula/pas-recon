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

// Hoisted out of PatientTable so it isn't redefined (and its button remounted,
// losing focus/canceling clicks) on every parent render.
function SortHeader({
  label,
  column,
  activeKey,
  dir,
  onSort,
}: {
  label: string;
  column: SortKey;
  activeKey: SortKey;
  dir: SortDir;
  onSort: (key: SortKey) => void;
}) {
  const active = activeKey === column;
  return (
    <th>
      <button
        type="button"
        onClick={() => onSort(column)}
        style={{
          background: "none",
          border: "none",
          color: active ? "var(--text)" : "var(--text-faint)",
          cursor: "pointer",
          font: "inherit",
          padding: 0,
          textAlign: "left",
          fontWeight: active ? 700 : 500,
          fontSize: "12px",
          whiteSpace: "nowrap",
        }}
      >
        {label}
        {active && (dir === "asc" ? " \u25B2" : " \u25BC")}
      </button>
    </th>
  );
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
    // Filter on the trimmed, lowercased query so leading/trailing spaces don't
    // cause the previous bug (trim checked for emptiness but matching used the
    // raw value).
    const q = searchQuery.trim().toLowerCase();
    if (!q) return rows;
    return rows.filter((r) =>
      r.phn.toLowerCase().includes(q) ||
      (r.first_name?.toLowerCase().includes(q) ?? false) ||
      (r.last_name?.toLowerCase().includes(q) ?? false) ||
      (r.source?.toLowerCase().includes(q) ?? false)
    );
  }, [rows, searchQuery]);

  const sorted = useMemo(() => {
    const getVal = (r: DisplayRow): string => {
      const v = r[sortKey];
      return (v ?? "").toLowerCase();
    };
    return [...filtered].sort((a, b) => {
      const av = getVal(a);
      const bv = getVal(b);
      // Empty/null values sort to the bottom in ascending and to the top in
      // descending, consistent with the sort direction (previously they were
      // pinned to the bottom in both directions).
      if (!av && !bv) return 0;
      if (!av) return sortDir === "asc" ? 1 : -1;
      if (!bv) return sortDir === "asc" ? -1 : 1;
      const cmp = av.localeCompare(bv);
      return sortDir === "asc" ? cmp : -cmp;
    });
  }, [filtered, sortKey, sortDir]);

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
            {showSource && <SortHeader label="Source" column="source" activeKey={sortKey} dir={sortDir} onSort={handleSort} />}
            <SortHeader label="PHN" column="phn" activeKey={sortKey} dir={sortDir} onSort={handleSort} />
            <SortHeader label="First Name" column="first_name" activeKey={sortKey} dir={sortDir} onSort={handleSort} />
            <SortHeader label="Last Name" column="last_name" activeKey={sortKey} dir={sortDir} onSort={handleSort} />
            <SortHeader label="DOB" column="dob" activeKey={sortKey} dir={sortDir} onSort={handleSort} />
            {showStatus && <SortHeader label="Status" column="mrp_status" activeKey={sortKey} dir={sortDir} onSort={handleSort} />}
          </tr>
        </thead>
        <tbody>
          {sorted.map((row) => {
            const isResolved = resolvedSet.has(row.phn);
            return (
            <tr
              key={row.phn}
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
