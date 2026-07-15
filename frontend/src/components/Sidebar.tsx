import DropZone from "./DropZone";
import SummaryCard from "./SummaryCard";
import StatusBreakdown from "./StatusBreakdown";
import type { Summary, StatusBreakdown as StatusBreakdownType } from "../types";

interface SidebarProps {
  emrLoaded: boolean;
  pasLoaded: boolean;
  emrFilename: string;
  pasFilename: string;
  error: string | null;
  summary: Summary | null;
  statusBreakdown: StatusBreakdownType | null;
  isDragging: boolean;
}

export default function Sidebar({
  emrLoaded, pasLoaded, emrFilename, pasFilename, error, summary, statusBreakdown, isDragging
}: SidebarProps) {
  return (
    <aside className="sidebar">
      <div>
        <h1>PAS Reconciliation</h1>
        <p className="version">v0.1.0</p>
      </div>
      <DropZone
        emrLoaded={emrLoaded}
        pasLoaded={pasLoaded}
        emrFilename={emrFilename}
        pasFilename={pasFilename}
        error={error}
        isDragging={isDragging}
      />
      {summary && <SummaryCard summary={summary} />}
      {statusBreakdown && <StatusBreakdown breakdown={statusBreakdown} />}
      <div className="privacy-note">
        Patient data stays on this machine. Closing the window clears it.
      </div>
    </aside>
  );
}
