interface DropZoneProps {
  emrLoaded: boolean;
  pasLoaded: boolean;
  error: string | null;
  isDragging: boolean;
}

export default function DropZone({ emrLoaded, pasLoaded, error, isDragging }: DropZoneProps) {
  return (
    <div
      className="drop-zone"
      style={isDragging ? { borderColor: "var(--blue)", background: "#252540" } : undefined}
    >
      <div className="drop-zone-label">Drop CSV Files Here</div>
      <div style={{ display: "flex", flexDirection: "column", gap: "4px", marginTop: "6px" }}>
        <div className="drop-zone-file">
          {emrLoaded ? "✓" : "○"} EMR Active Patient List.csv
        </div>
        <div className="drop-zone-file">
          {pasLoaded ? "✓" : "○"} PAS Patient List.csv
        </div>
      </div>
      {error && (
        <div className="error-banner" style={{ marginTop: "8px" }}>{error}</div>
      )}
    </div>
  );
}
