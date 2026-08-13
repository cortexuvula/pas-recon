import { useState, useEffect, useCallback, useMemo, useRef } from "react";
import Sidebar from "./components/Sidebar";
import ListTabs from "./components/ListTabs";
import PatientTable from "./components/PatientTable";
import UpdateToast from "./components/UpdateToast";
import EmptyState from "./components/EmptyState";
import ColumnPicker from "./components/ColumnPicker";
import FileConfirm from "./components/FileConfirm";
import { save } from "@tauri-apps/plugin-dialog";
import { check as checkForUpdate } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import {
  reconcileFiles,
  reconcileWithColumnOverride,
  exportList,
  getCsvHeaders,
  browseForFile,
  onUpdateAvailable,
  onDragDropEvent,
} from "./api";
import type { ReconciliationResult, UpdateInfo, ListKey } from "./types";

/** True if a header set looks like a PAS export (PAS-specific MRP columns). */
function hasPasSignal(headers: string[]): boolean {
  return headers.some(h => {
    const lower = h.toLowerCase();
    return lower.includes("pas mrp status") || lower.includes("pas mrp updated");
  });
}

/** Convert an unknown error value to a user-friendly string. */
function humanizeError(e: unknown): string {
  const raw = typeof e === "string" ? e : String(e);
  // Engine Display messages already arrive as readable strings (e.g.,
  // "could not find a PHN column in PAS CSV"). Enhance the most common ones.
  if (raw.includes("could not find a PHN column")) {
    return raw + " Please select it manually using the column picker.";
  }
  if (raw.includes("multiple columns") && raw.includes("PHN")) {
    return raw + " Please select the correct one using the column picker.";
  }
  if (raw.includes("file is empty") || raw.includes("no data rows")) {
    return "One of the files is empty or has no data rows. Please check your files.";
  }
  if (raw.includes("CSV parse error") || raw.includes("CSV read error")) {
    return "Could not parse one of the files. Please make sure both files are valid CSV files.";
  }
  if (raw.includes("failed to read") || raw.includes("No such file")) {
    return "Could not read one of the files. The file may have been moved or deleted.";
  }
  if (raw.includes("invalid updater binary format")) {
    return "Auto-update is not supported for this Linux installation. Please download the latest version from the releases page.";
  }
  if (raw === "[object Object]") {
    return "An unexpected error occurred. Please try again.";
  }
  return raw;
}

export default function App() {
  const [result, setResult] = useState<ReconciliationResult | null>(null);
  const [emrLoaded, setEmrLoaded] = useState(false);
  const [pasLoaded, setPasLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [emrPath, setEmrPath] = useState<string | null>(null);
  const [pasPath, setPasPath] = useState<string | null>(null);
  const [activeList, setActiveList] = useState<ListKey>("emr_no_match");
  const [searchQuery, setSearchQuery] = useState("");
  const [resolved, setResolved] = useState<Set<string>>(new Set());
  const [update, setUpdate] = useState<UpdateInfo | null>(null);
  const [showColumnPicker, setShowColumnPicker] = useState(false);
  const [exportStatusFilter, setExportStatusFilter] = useState("all");
  const [isDragging, setIsDragging] = useState(false);
  const [showFileConfirm, setShowFileConfirm] = useState(false);
  const [pendingEmrPath, setPendingEmrPath] = useState<string | null>(null);
  const [pendingPasPath, setPendingPasPath] = useState<string | null>(null);
  const [pendingEmrFilename, setPendingEmrFilename] = useState("");
  const [pendingPasFilename, setPendingPasFilename] = useState("");
  const [pendingEmrHeaders, setPendingEmrHeaders] = useState<string[]>([]);
  const [pendingPasHeaders, setPendingPasHeaders] = useState<string[]>([]);

  useEffect(() => {
    const unlisten = onUpdateAvailable((info) => setUpdate(info));
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  /** Extract just the filename from a full path. */
  const basename = (path: string) => path.split("/").pop()?.split("\\").pop() ?? path;

  /**
   * Classify two files as EMR and PAS by inspecting their headers.
   * Returns [emrPath, pasPath] if confident, or null if ambiguous.
   */
  const classifyFiles = (headers1: string[], headers2: string[], path1: string, path2: string): [string, string] | null => {
    const hasFilenameSignal = (path: string, type: "emr" | "pas") =>
      path.toLowerCase().includes(type);

    const pas1 = hasPasSignal(headers1);
    const pas2 = hasPasSignal(headers2);

    // One file clearly has PAS headers → the other is EMR
    if (pas1 && !pas2) return [path2, path1];
    if (pas2 && !pas1) return [path1, path2];

    // Neither has PAS headers → try filename fallback
    if (!pas1 && !pas2) {
      const emr1 = hasFilenameSignal(path1, "emr") || path1.toLowerCase().includes("active patient");
      const emr2 = hasFilenameSignal(path2, "emr") || path2.toLowerCase().includes("active patient");
      const pas1name = hasFilenameSignal(path1, "pas");
      const pas2name = hasFilenameSignal(path2, "pas");

      if (emr1 && pas2name) return [path1, path2];
      if (emr2 && pas1name) return [path2, path1];
    }

    // Both have PAS headers, or no signal at all → ambiguous
    return null;
  };

  // Sequence reconciliation runs so a slower, older run can't overwrite the
  // result of a newer one (rapid drops/browses can otherwise race). Each run
  // captures a monotonic id; on resolve it applies state only if it is still
  // the latest run initiated.
  const latestRunIdRef = useRef(0);

  const applyResult = useCallback((res: ReconciliationResult) => {
    setResult(res);
    setResolved(new Set());
    setSearchQuery("");
    setActiveList("emr_no_match");
  }, []);

  const runReconciliation = useCallback(async (emr: string, pas: string) => {
    const reqId = ++latestRunIdRef.current;
    try {
      const res = await reconcileFiles(emr, pas);
      if (reqId !== latestRunIdRef.current) return; // superseded by a newer run
      applyResult(res);
    } catch (e: any) {
      if (reqId !== latestRunIdRef.current) return; // superseded
      const errStr = typeof e === "string" ? e : JSON.stringify(e);
      if (errStr.includes("PHN column") || errStr.includes("MissingPhnColumn") || errStr.includes("AmbiguousPhnColumns")) {
        setResult(null);  // clear stale results from a previous reconciliation
        setShowColumnPicker(true);
        setError(null);
      } else {
        setError(humanizeError(errStr));
      }
    }
  }, [applyResult]);

  const handlePathsDropped = useCallback(async (paths: string[]) => {
    setError(null);

    if (paths.length === 0) return;

    if (paths.length === 1) {
      // Single file — classify it and ask for the next
      await ingestSingleFile(paths[0]);
    } else {
      // Two or more files — classify both and reconcile
      const [path1, path2] = paths;
      try {
        const [headers1, headers2] = await Promise.all([
          getCsvHeaders(path1),
          getCsvHeaders(path2),
        ]);

        const classified = classifyFiles(headers1, headers2, path1, path2);

        if (classified) {
          const [emrP, pasP] = classified;
          setEmrPath(emrP);
          setPasPath(pasP);
          setEmrLoaded(true);
          setPasLoaded(true);
          await runReconciliation(emrP, pasP);
        } else {
          setPendingEmrPath(path1);
          setPendingPasPath(path2);
          setPendingEmrFilename(basename(path1));
          setPendingPasFilename(basename(path2));
          setPendingEmrHeaders(headers1);
          setPendingPasHeaders(headers2);
          setShowFileConfirm(true);
        }
      } catch (e: any) {
        setError(humanizeError(`Failed to read files: ${e}`));
      }
    }
  }, [runReconciliation]);

  /** Ingest a single file — auto-classify as EMR or PAS by headers.
   *  If the other file is already loaded, run reconciliation.
   *  Uses refs to read current paths (avoids stale closure in event listener). */
  const emrPathRef = useRef<string | null>(null);
  const pasPathRef = useRef<string | null>(null);
  useEffect(() => { emrPathRef.current = emrPath; }, [emrPath]);
  useEffect(() => { pasPathRef.current = pasPath; }, [pasPath]);

  const ingestSingleFile = useCallback(async (path: string) => {
    setError(null);
    try {
      const headers = await getCsvHeaders(path);
      const isPas = hasPasSignal(headers);

      if (isPas) {
        // This is the PAS file
        setPasPath(path);
        setPasLoaded(true);
        const currentEmr = emrPathRef.current;
        if (currentEmr) {
          await runReconciliation(currentEmr, path);
        }
      } else {
        // Assume EMR
        setEmrPath(path);
        setEmrLoaded(true);
        const currentPas = pasPathRef.current;
        if (currentPas) {
          await runReconciliation(path, currentPas);
        }
      }
    } catch (e: any) {
      setError(humanizeError(`Failed to read file: ${e}`));
    }
  }, [runReconciliation]);

  /** Browse for a file using the native file picker. */
  const handleBrowse = useCallback(async (which: "emr" | "pas") => {
    const title = which === "emr"
      ? "Select your EMR Active Patient List CSV"
      : "Select your PAS Patient List CSV";
    const path = await browseForFile(title);
    if (path) {
      await ingestSingleFile(path);
    }
  }, [ingestSingleFile]);

  const handleFileConfirm = useCallback(async () => {
    setShowFileConfirm(false);
    if (pendingEmrPath && pendingPasPath) {
      setEmrPath(pendingEmrPath);
      setPasPath(pendingPasPath);
      setEmrLoaded(true);
      setPasLoaded(true);
      await runReconciliation(pendingEmrPath, pendingPasPath);
    }
  }, [pendingEmrPath, pendingPasPath, runReconciliation]);

  const handleFileSwap = useCallback(async () => {
    setShowFileConfirm(false);
    if (pendingEmrPath && pendingPasPath) {
      setEmrPath(pendingPasPath);
      setPasPath(pendingEmrPath);
      setEmrLoaded(true);
      setPasLoaded(true);
      await runReconciliation(pendingPasPath, pendingEmrPath);
    }
  }, [pendingEmrPath, pendingPasPath, runReconciliation]);

  // Register Tauri-native drag-and-drop at the window level.
  // Registered once; uses refs to avoid stale closures.
  useEffect(() => {
    const unlisten = onDragDropEvent((event) => {
      if (event.type === "enter" || event.type === "over") {
        setIsDragging(true);
      } else if (event.type === "leave") {
        setIsDragging(false);
      } else if (event.type === "drop") {
        setIsDragging(false);
        handlePathsDropped(event.paths);
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [handlePathsDropped]);

  const handleColumnPickerResolved = useCallback(async (emrCol: number, pasCol: number) => {
    if (!emrPath || !pasPath) return;
    setShowColumnPicker(false);
    setError(null);
    const reqId = ++latestRunIdRef.current;
    try {
      const res = await reconcileWithColumnOverride(emrPath, pasPath, emrCol, pasCol);
      if (reqId !== latestRunIdRef.current) return; // superseded by a newer run
      applyResult(res);
    } catch (e: any) {
      if (reqId !== latestRunIdRef.current) return; // superseded
      setError(humanizeError(e));
    }
  }, [emrPath, pasPath, applyResult]);

  const handleToggleResolved = useCallback((phn: string) => {
    setResolved((prev) => {
      const next = new Set(prev);
      if (next.has(phn)) next.delete(phn);
      else next.add(phn);
      return next;
    });
  }, []);

  const currentRows = result ? result[activeList] : [];
  const showStatus = activeList === "pas_match_review" || activeList === "pas_no_match";
  const showSource = activeList === "invalid_phns";

  // Compute distinct statuses for the export filter dropdown
  const distinctStatuses = useMemo(() => {
    if (!result || !showStatus) return [];
    const rows = result[activeList];
    const statuses = new Set<string>();
    for (const r of rows) {
      if (r.mrp_status) statuses.add(r.mrp_status);
    }
    return Array.from(statuses).sort();
  }, [result, activeList, showStatus]);

  const handleExportWithFormat = useCallback(async (format: "csv" | "pdf") => {
    if (!result) return;
    let rows = result[activeList];
    // Filter by selected status if not "all"
    if (exportStatusFilter !== "all") {
      rows = rows.filter(r => r.mrp_status === exportStatusFilter);
    }
    try {
      const suffix = exportStatusFilter !== "all" ? `-${exportStatusFilter.replace(/\s+/g, "-").toLowerCase()}` : "";
      const ext = format === "pdf" ? "html" : "csv";
      const selected = await save({
        defaultPath: `${activeList}${suffix}.${ext}`,
        filters: [{ name: format === "pdf" ? "HTML (Print to PDF)" : "CSV", extensions: [ext] }],
      });
      if (selected) {
        // Build a human-readable title for the PDF header
        const tabLabel = activeList === "emr_no_match" ? "EMR No Match"
          : activeList === "pas_match_review" ? "PAS Match - Review"
          : activeList === "pas_no_match" ? "PAS No Match"
          : activeList === "invalid_phns" ? "Invalid PHNs"
          : activeList;
        const statusLabel = exportStatusFilter !== "all" ? ` (${exportStatusFilter})` : "";
        const title = `PAS Reconciliation — ${tabLabel}${statusLabel}`;
        await exportList(rows, selected, format, title);
      }
    } catch (e) {
      setError(humanizeError(`Export failed: ${e}`));
    }
  }, [result, activeList, exportStatusFilter]);

  return (
    <div className="app">
      <Sidebar
        emrLoaded={emrLoaded}
        pasLoaded={pasLoaded}
        emrFilename={emrPath ? basename(emrPath) : ""}
        pasFilename={pasPath ? basename(pasPath) : ""}
        summary={result?.summary ?? null}
        statusBreakdown={result?.summary.status_breakdown ?? null}
        isDragging={isDragging}
        onBrowseEmr={() => handleBrowse("emr")}
        onBrowsePas={() => handleBrowse("pas")}
      />
      <main className="main-panel">
        {error && (
          <div className="error-main">
            <span>{error}</span>
            <button type="button" className="error-dismiss" onClick={() => setError(null)} aria-label="Dismiss error">
              {"\u00D7"}
            </button>
          </div>
        )}
        {update && (
          <UpdateToast
            info={update}
            onDownload={async () => {
              try {
                const updateObj = await checkForUpdate();
                if (updateObj) {
                  await updateObj.downloadAndInstall();
                  await relaunch();
                }
              } catch (e) {
                const errMsg = String(e);
                if (errMsg.includes("invalid updater binary format")) {
                  setError("Auto-update is not supported for this Linux installation (.deb/.rpm). Please download the latest version manually from: https://github.com/cortexuvula/pas-recon/releases/latest");
                  setUpdate(null);
                } else {
                  setError(humanizeError(`Update failed: ${e}`));
                }
              }
            }}
            onDismiss={() => setUpdate(null)}
          />
        )}
        {showFileConfirm ? (
          <FileConfirm
            emrFilename={pendingEmrFilename}
            pasFilename={pendingPasFilename}
            emrHeaders={pendingEmrHeaders}
            pasHeaders={pendingPasHeaders}
            onConfirm={handleFileConfirm}
            onSwap={handleFileSwap}
          />
        ) : showColumnPicker && emrPath && pasPath ? (
          <ColumnPicker
            emrPath={emrPath}
            pasPath={pasPath}
            onResolved={handleColumnPickerResolved}
            onCancel={() => setShowColumnPicker(false)}
          />
        ) : result ? (
          <>
            <ListTabs
              active={activeList}
              onSelect={setActiveList}
              summary={result.summary}
            />
            <div className="toolbar">
              <input
                className="search-input"
                placeholder="Search PHN or name…"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
              />
              {showStatus && distinctStatuses.length > 0 && (
                <select
                  className="search-input"
                  style={{ width: "auto", minWidth: "120px" }}
                  value={exportStatusFilter}
                  onChange={(e) => setExportStatusFilter(e.target.value)}
                  title="Filter export by status"
                >
                  <option value="all">All statuses</option>
                  {distinctStatuses.map(s => (
                    <option key={s} value={s}>{s}</option>
                  ))}
                </select>
              )}
              <button type="button" className="export-btn" onClick={() => handleExportWithFormat("csv")}>Export CSV</button>
              <button type="button" className="export-btn" onClick={() => handleExportWithFormat("pdf")} style={{ background: "var(--red)" }}>Export PDF</button>
              <span style={{ fontSize: "11px", color: "var(--text-faint)", marginLeft: "4px" }}>(opens in browser)</span>
            </div>
            <PatientTable
              rows={currentRows}
              showStatus={showStatus}
              showSource={showSource}
              resolvedSet={resolved}
              onToggleResolved={handleToggleResolved}
              searchQuery={searchQuery}
            />
            <div className="status-bar">
              <span>Showing {currentRows.length} patients · sorted by last name</span>
              <span>Data in memory only · not saved to disk</span>
            </div>
          </>
        ) : (
          <EmptyState message={error ?? "Drop both CSV files to begin reconciliation."} />
        )}
      </main>
    </div>
  );
}
