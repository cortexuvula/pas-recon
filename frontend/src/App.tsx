import { useState, useEffect, useCallback } from "react";
import Sidebar from "./components/Sidebar";
import ListTabs from "./components/ListTabs";
import PatientTable from "./components/PatientTable";
import UpdateToast from "./components/UpdateToast";
import EmptyState from "./components/EmptyState";
import ColumnPicker from "./components/ColumnPicker";
import FileConfirm from "./components/FileConfirm";
import {
  reconcileFiles,
  reconcileWithColumnOverride,
  exportList,
  getCsvHeaders,
  onUpdateAvailable,
  onDragDropEvent,
} from "./api";
import type { ReconciliationResult, UpdateInfo, ListKey } from "./types";

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
    const hasPasSignal = (headers: string[]) =>
      headers.some(h => {
        const lower = h.toLowerCase();
        return lower.includes("pas mrp status") || lower.includes("pas mrp updated");
      });
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

  const runReconciliation = useCallback(async (emr: string, pas: string) => {
    try {
      const res = await reconcileFiles(emr, pas);
      setResult(res);
      setResolved(new Set());
      setSearchQuery("");
      setActiveList("emr_no_match");
    } catch (e: any) {
      const errStr = typeof e === "string" ? e : JSON.stringify(e);
      if (errStr.includes("MissingPhnColumn") || errStr.includes("AmbiguousPhnColumns") || errStr.includes("PHN column")) {
        setShowColumnPicker(true);
        setError(null);
      } else {
        setError(errStr);
      }
    }
  }, []);

  const handlePathsDropped = useCallback(async (paths: string[]) => {
    setError(null);

    if (paths.length < 2) {
      setError("Please drop both CSV files (EMR panel and PAS patient list).");
      return;
    }

    // Take the first two files
    const [path1, path2] = paths;

    try {
      const [headers1, headers2] = await Promise.all([
        getCsvHeaders(path1),
        getCsvHeaders(path2),
      ]);

      const classified = classifyFiles(headers1, headers2, path1, path2);

      if (classified) {
        const [emrPath, pasPath] = classified;
        setEmrPath(emrPath);
        setPasPath(pasPath);
        setEmrLoaded(true);
        setPasLoaded(true);
        await runReconciliation(emrPath, pasPath);
      } else {
        // Ambiguous — store paths and show confirmation dialog.
        // Default guess: path1 = EMR, path2 = PAS (user can swap).
        setPendingEmrPath(path1);
        setPendingPasPath(path2);
        setPendingEmrFilename(basename(path1));
        setPendingPasFilename(basename(path2));
        setPendingEmrHeaders(headers1);
        setPendingPasHeaders(headers2);
        setShowFileConfirm(true);
      }
    } catch (e: any) {
      setError(`Failed to read files: ${e}`);
    }
  }, [runReconciliation]);

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
    try {
      const res = await reconcileWithColumnOverride(emrPath, pasPath, emrCol, pasCol);
      setResult(res);
      setResolved(new Set());
      setSearchQuery("");
      setActiveList("emr_no_match");
    } catch (e: any) {
      setError(typeof e === "string" ? e : JSON.stringify(e));
    }
  }, [emrPath, pasPath]);

  const handleToggleResolved = useCallback((phn: string) => {
    setResolved((prev) => {
      const next = new Set(prev);
      if (next.has(phn)) next.delete(phn);
      else next.add(phn);
      return next;
    });
  }, []);

  const handleExport = useCallback(async () => {
    if (!result) return;
    const rows = result[activeList];
    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const path = await save({
        defaultPath: `${activeList}.csv`,
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });
      if (path) {
        await exportList(rows, path);
      }
    } catch (e) {
      setError(`Export failed: ${e}`);
    }
  }, [result, activeList]);

  const currentRows = result ? result[activeList] : [];
  const showStatus = activeList === "pas_match_review" || activeList === "pas_no_match";
  const showSource = activeList === "invalid_phns";

  return (
    <div className="app">
      <Sidebar
        emrLoaded={emrLoaded}
        pasLoaded={pasLoaded}
        emrFilename={emrPath ? basename(emrPath) : ""}
        pasFilename={pasPath ? basename(pasPath) : ""}
        error={error}
        summary={result?.summary ?? null}
        statusBreakdown={result?.summary.status_breakdown ?? null}
        isDragging={isDragging}
      />
      <main className="main-panel">
        {update && (
          <UpdateToast
            info={update}
            onDownload={async () => {
              try {
                const { check } = await import("@tauri-apps/plugin-updater");
                const updateObj = await check();
                if (updateObj) {
                  await updateObj.downloadAndInstall();
                  // Restart the app to apply the update
                  const { relaunch } = await import("@tauri-apps/plugin-process");
                  await relaunch();
                }
              } catch (e) {
                setError(`Update failed: ${e}`);
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
              <button type="button" className="export-btn" onClick={handleExport}>Export CSV</button>
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
