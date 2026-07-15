interface DropZoneProps {
  emrLoaded: boolean;
  pasLoaded: boolean;
  emrFilename: string;
  pasFilename: string;
  error: string | null;
  isDragging: boolean;
}

export default function DropZone({ emrLoaded, pasLoaded, emrFilename, pasFilename, error, isDragging }: DropZoneProps) {
  return (
    <div
      className="drop-zone"
      style={isDragging ? { borderColor: "var(--blue)", background: "#252540" } : undefined}
    >
      <div className="drop-zone-label">Drop CSV Files Here</div>
      <div style={{ display: "flex", flexDirection: "column", gap: "4px", marginTop: "6px" }}>
        <div className="drop-zone-file">
          {emrLoaded ? "✓" : "○"} {emrFilename || "EMR file"}
        </div>
        <div className="drop-zone-file">
          {pasLoaded ? "✓" : "○"} {pasFilename || "PAS file"}
        </div>
      </div>
      {error && (
        <div className="error-banner" style={{ marginTop: "8px" }}>{error}</div>
      )}
    </div>
  );
}
