import DropZone from "./DropZone";
import SummaryCard from "./SummaryCard";
import StatusBreakdown from "./StatusBreakdown";
import type { Summary, StatusBreakdown as StatusBreakdownType } from "../types";

interface SidebarProps {
  onFilesDropped: (files: File[]) => void;
  emrLoaded: boolean;
  pasLoaded: boolean;
  error: string | null;
  summary: Summary | null;
  statusBreakdown: StatusBreakdownType | null;
}

export default function Sidebar({
  onFilesDropped, emrLoaded, pasLoaded, error, summary, statusBreakdown
}: SidebarProps) {
  return (
    <aside className="sidebar">
      <div>
        <h1>PAS Reconciliation</h1>
        <p className="version">v0.1.0</p>
      </div>
      <DropZone
        onFilesDropped={onFilesDropped}
        emrLoaded={emrLoaded}
        pasLoaded={pasLoaded}
        error={error}
      />
      {summary && <SummaryCard summary={summary} />}
      {statusBreakdown && <StatusBreakdown breakdown={statusBreakdown} />}
      <div className="privacy-note">
        Patient data stays on this machine. Closing the window clears it.
      </div>
    </aside>
  );
}
