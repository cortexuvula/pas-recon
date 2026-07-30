import DropZone from "./DropZone";
import SummaryCard from "./SummaryCard";
import StatusBreakdown from "./StatusBreakdown";
import type { Summary, StatusBreakdown as StatusBreakdownType } from "../types";

interface SidebarProps {
  emrLoaded: boolean;
  pasLoaded: boolean;
  emrFilename: string;
  pasFilename: string;
  summary: Summary | null;
  statusBreakdown: StatusBreakdownType | null;
  isDragging: boolean;
  onBrowseEmr: () => void;
  onBrowsePas: () => void;
}

export default function Sidebar({
  emrLoaded, pasLoaded, emrFilename, pasFilename, summary, statusBreakdown, isDragging,
  onBrowseEmr, onBrowsePas,
}: SidebarProps) {
  return (
    <aside className="sidebar">
      <div>
        <h1>PAS Reconciliation</h1>
        <p className="version">v{__APP_VERSION__}</p>
      </div>
      <DropZone
        emrLoaded={emrLoaded}
        pasLoaded={pasLoaded}
        emrFilename={emrFilename}
        pasFilename={pasFilename}
        isDragging={isDragging}
        onBrowseEmr={onBrowseEmr}
        onBrowsePas={onBrowsePas}
      />
      {summary && <SummaryCard summary={summary} />}
      {statusBreakdown && <StatusBreakdown breakdown={statusBreakdown} />}
      <div className="privacy-note">
        Patient data stays on this machine. Closing the window clears it.
      </div>
    </aside>
  );
}
