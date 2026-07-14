import type { StatusBreakdown as StatusBreakdownType } from "../types";

interface StatusBreakdownProps {
  breakdown: StatusBreakdownType;
}

export default function StatusBreakdown({ breakdown }: StatusBreakdownProps) {
  return (
    <div className="status-section">
      <div className="section-label">PAS Status Breakdown</div>
      <div className="summary-row"><span>Confirmed</span><span>{breakdown.confirmed}</span></div>
      <div className="summary-row"><span>Pending</span><span>{breakdown.pending}</span></div>
      <div className="summary-row"><span>Deceased</span><span>{breakdown.deceased}</span></div>
      <div className="summary-row"><span>Removed</span><span>{breakdown.removed}</span></div>
      <div className="summary-row"><span>Not the MRP</span><span>{breakdown.not_the_mrp}</span></div>
    </div>
  );
}
