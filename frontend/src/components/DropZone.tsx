import { useState } from "react";

interface DropZoneProps {
  onFilesDropped: (files: File[]) => void;
  emrLoaded: boolean;
  pasLoaded: boolean;
  error: string | null;
}

export default function DropZone({ onFilesDropped, emrLoaded, pasLoaded, error }: DropZoneProps) {
  const [isDragging, setIsDragging] = useState(false);

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    const files = Array.from(e.dataTransfer.files);
    if (files.length > 0) onFilesDropped(files);
  };

  return (
    <div
      className="drop-zone"
      style={isDragging ? { borderColor: "var(--blue)" } : undefined}
      onDragOver={(e) => { e.preventDefault(); setIsDragging(true); }}
      onDragLeave={() => setIsDragging(false)}
      onDrop={handleDrop}
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
