interface DropZoneProps {
  emrLoaded: boolean;
  pasLoaded: boolean;
  emrFilename: string;
  pasFilename: string;
  isDragging: boolean;
}

export default function DropZone({ emrLoaded, pasLoaded, emrFilename, pasFilename, isDragging }: DropZoneProps) {
  return (
    <div
      className="drop-zone"
      style={isDragging ? { borderColor: "var(--blue)", background: "#252540" } : undefined}
    >
      <div className="drop-zone-label">Drop CSV Files Here</div>
      <div style={{ display: "flex", flexDirection: "column", gap: "4px", marginTop: "6px" }}>
        <div className="drop-zone-file">
          {emrLoaded ? "\u2705" : "\u2B1C"} {emrFilename || "EMR file"}
        </div>
        <div className="drop-zone-file">
          {pasLoaded ? "\u2705" : "\u2B1C"} {pasFilename || "PAS file"}
        </div>
      </div>
    </div>
  );
}
