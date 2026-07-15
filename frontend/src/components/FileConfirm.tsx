interface FileConfirmProps {
  emrFilename: string;
  pasFilename: string;
  emrHeaders: string[];
  pasHeaders: string[];
  onConfirm: () => void;
  onSwap: () => void;
}

export default function FileConfirm({
  emrFilename, pasFilename, emrHeaders, pasHeaders, onConfirm, onSwap
}: FileConfirmProps) {
  return (
    <div style={{ padding: "24px", maxWidth: "600px", margin: "0 auto" }}>
      <h2 style={{ fontSize: "16px", marginBottom: "8px" }}>Confirm File Assignment</h2>
      <p style={{ fontSize: "13px", color: "var(--text-dim)", marginBottom: "20px" }}>
        We couldn't automatically determine which file is which. Please verify the assignment below.
      </p>
      <div style={{ display: "flex", gap: "24px", marginBottom: "20px" }}>
        <div style={{ flex: 1 }}>
          <div style={{ fontSize: "12px", fontWeight: 600, color: "var(--amber)", marginBottom: "6px" }}>
            EMR Panel
          </div>
          <div style={{ fontSize: "13px", color: "var(--text)", marginBottom: "6px" }}>
            {emrFilename}
          </div>
          <div style={{ fontSize: "11px", color: "var(--text-faint)" }}>
            {emrHeaders.slice(0, 5).join(", ")}
            {emrHeaders.length > 5 && `… (+${emrHeaders.length - 5} more)`}
          </div>
        </div>
        <div style={{ flex: 1 }}>
          <div style={{ fontSize: "12px", fontWeight: 600, color: "var(--blue)", marginBottom: "6px" }}>
            PAS Patient List
          </div>
          <div style={{ fontSize: "13px", color: "var(--text)", marginBottom: "6px" }}>
            {pasFilename}
          </div>
          <div style={{ fontSize: "11px", color: "var(--text-faint)" }}>
            {pasHeaders.slice(0, 5).join(", ")}
            {pasHeaders.length > 5 && `… (+${pasHeaders.length - 5} more)`}
          </div>
        </div>
      </div>
      <div style={{ display: "flex", gap: "8px", justifyContent: "flex-end" }}>
        <button className="tab" onClick={onSwap}>Swap</button>
        <button className="export-btn" onClick={onConfirm}>Confirm</button>
      </div>
    </div>
  );
}
