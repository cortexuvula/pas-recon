import type { UpdateInfo } from "../types";

interface UpdateToastProps {
  info: UpdateInfo;
  onDownload: () => void;
  onDismiss: () => void;
}

export default function UpdateToast({ info, onDownload, onDismiss }: UpdateToastProps) {
  return (
    <div className="update-toast" style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
      <span>Update available — v{info.version} (you have v{info.current_version})</span>
      <span style={{ display: "flex", gap: "8px" }}>
        <button onClick={onDownload} className="export-btn">Download & Install</button>
        <button onClick={onDismiss} className="tab">Later</button>
      </span>
    </div>
  );
}
