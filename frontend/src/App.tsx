import { useState, useEffect, useCallback, useRef } from "react";
import Sidebar from "./components/Sidebar";
import ListTabs from "./components/ListTabs";
import PatientTable from "./components/PatientTable";
import UpdateToast from "./components/UpdateToast";
import EmptyState from "./components/EmptyState";
import ColumnPicker from "./components/ColumnPicker";
import {
  reconcileFiles,
  reconcileWithColumnOverride,
  exportList,
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

  // Ref to hold the latest emrPath/pasPath so the drag-drop listener
  // (registered once) always sees current values without stale closures.
  const emrPathRef = useRef<string | null>(null);
  const pasPathRef = useRef<string | null>(null);

  useEffect(() => {
    emrPathRef.current = emrPath;
  }, [emrPath]);
  useEffect(() => {
    pasPathRef.current = pasPath;
  }, [pasPath]);

  useEffect(() => {
    const unlisten = onUpdateAvailable((info) => setUpdate(info));
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  const handlePathsDropped = useCallback(async (paths: string[]) => {
    setError(null);
    let newEmrPath = emrPathRef.current;
    let newPasPath = pasPathRef.current;

    for (const path of paths) {
      const name = path.toLowerCase();
      if (name.includes("emr")) {
        newEmrPath = path;
        setEmrLoaded(true);
      } else if (name.includes("pas")) {
        newPasPath = path;
        setPasLoaded(true);
      }
    }
    setEmrPath(newEmrPath);
    setPasPath(newPasPath);

    if (newEmrPath && newPasPath) {
      try {
        const res = await reconcileFiles(newEmrPath, newPasPath);
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
    }
  }, []);

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
    try {
      const res = await reconcileWithColumnOverride(emrPath, pasPath, emrCol, pasCol);
      setResult(res);
      setResolved(new Set());
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
  const showStatus = activeList !== "emr_no_match";

  return (
    <div className="app">
      <Sidebar
        emrLoaded={emrLoaded}
        pasLoaded={pasLoaded}
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
        {showColumnPicker && emrPath && pasPath ? (
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
              <button className="export-btn" onClick={handleExport}>Export CSV</button>
            </div>
            <PatientTable
              rows={currentRows}
              showStatus={showStatus}
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
