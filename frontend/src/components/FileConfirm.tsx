interface FileConfirmProps {
  emrFilename: string;
  pasFilename: string;
  emrHeaders: string[];
  pasHeaders: string[];
  onConfirm: () => void;
  onSwap: () => void;
}

function FilePanel({
  label, labelColor, filename, headers,
}: {
  label: string;
  labelColor: string;
  filename: string;
  headers: string[];
}) {
  return (
    <div style={{ flex: 1 }}>
      <div style={{ fontSize: "12px", fontWeight: 600, color: labelColor, marginBottom: "6px" }}>
        {label}
      </div>
      <div style={{ fontSize: "13px", color: "var(--text)", marginBottom: "6px" }}>
        {filename}
      </div>
      <div style={{ fontSize: "11px", color: "var(--text-faint)" }}>
        {headers.slice(0, 5).join(", ")}
        {headers.length > 5 && `… (+${headers.length - 5} more)`}
      </div>
    </div>
  );
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
        <FilePanel label="EMR Panel" labelColor="var(--amber)" filename={emrFilename} headers={emrHeaders} />
        <FilePanel label="PAS Patient List" labelColor="var(--blue)" filename={pasFilename} headers={pasHeaders} />
      </div>
      <div style={{ display: "flex", gap: "8px", justifyContent: "flex-end" }}>
        <button type="button" className="tab" onClick={onSwap}>Swap</button>
        <button type="button" className="export-btn" onClick={onConfirm}>Confirm</button>
      </div>
    </div>
  );
}
