interface DropZoneProps {
  emrLoaded: boolean;
  pasLoaded: boolean;
  emrFilename: string;
  pasFilename: string;
  isDragging: boolean;
  onBrowseEmr: () => void;
  onBrowsePas: () => void;
}

function FileRow({
  loaded, filename, fallback, onBrowse,
}: {
  loaded: boolean;
  filename: string;
  fallback: string;
  onBrowse: () => void;
}) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
      <span className="drop-zone-file" style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
        {loaded ? "\u2705" : "\u2B1C"} {filename || fallback}
      </span>
      <button type="button" className="browse-btn" onClick={onBrowse}>
        {loaded ? "Change" : "Browse"}
      </button>
    </div>
  );
}

export default function DropZone({ emrLoaded, pasLoaded, emrFilename, pasFilename, isDragging, onBrowseEmr, onBrowsePas }: DropZoneProps) {
  const prompt =
    !emrLoaded && !pasLoaded ? "Drop CSV files here, or browse:"
    : emrLoaded && !pasLoaded ? "EMR loaded — now select your PAS file:"
    : !emrLoaded && pasLoaded ? "PAS loaded — now select your EMR file:"
    : "Both files loaded.";

  return (
    <div
      className="drop-zone"
      style={isDragging ? { borderColor: "var(--blue)", background: "#252540" } : undefined}
    >
      <div className="drop-zone-label">{prompt}</div>
      <div style={{ display: "flex", flexDirection: "column", gap: "8px", marginTop: "8px" }}>
        <FileRow loaded={emrLoaded} filename={emrFilename} fallback="EMR file" onBrowse={onBrowseEmr} />
        <FileRow loaded={pasLoaded} filename={pasFilename} fallback="PAS file" onBrowse={onBrowsePas} />
      </div>
    </div>
  );
}
