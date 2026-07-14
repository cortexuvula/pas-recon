import type { Summary } from "../types";

interface SummaryCardProps {
  summary: Summary;
}

export default function SummaryCard({ summary }: SummaryCardProps) {
  return (
    <div className="summary-section">
      <div className="section-label">Summary</div>
      <div className="summary-row">
        <span style={{ color: "var(--green)" }}>● Matched</span>
        <strong>{summary.matched}</strong>
      </div>
      <div className="summary-row">
        <span style={{ color: "var(--red)" }}>● EMR only</span>
        <strong>{summary.emr_only}</strong>
      </div>
      <div className="summary-row">
        <span style={{ color: "var(--amber)" }}>● PAS only</span>
        <strong>{summary.pas_only}</strong>
      </div>
      <div className="summary-row">
        <span style={{ color: "var(--purple)" }}>● Review</span>
        <strong>{summary.pas_review}</strong>
      </div>
      {(summary.duplicates_dropped > 0 || summary.invalid_phn_skipped > 0) && (
        <div style={{ marginTop: "6px", fontSize: "8px", color: "var(--text-faint)" }}>
          ⚠ {summary.duplicates_dropped} duplicates dropped, {summary.invalid_phn_skipped} invalid PHNs skipped
        </div>
      )}
    </div>
  );
}
