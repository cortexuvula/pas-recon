import { useState, useEffect } from "react";
import type { ReconciliationResult, UpdateInfo } from "./types";
import { onUpdateAvailable } from "./api";

export default function App() {
  const [result, setResult] = useState<ReconciliationResult | null>(null);
  const [update, setUpdate] = useState<UpdateInfo | null>(null);

  useEffect(() => {
    const unlisten = onUpdateAvailable((info) => setUpdate(info));
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  return (
    <div className="app">
      <aside className="sidebar">
        <h1>PAS Reconciliation</h1>
        <p className="version">v0.1.0</p>
        <p>Frontend scaffold ready.</p>
      </aside>
      <main className="main-panel">
        {update && (
          <div className="update-toast">
            Update available — v{update.version}
          </div>
        )}
        <p>Drop CSV files to begin.</p>
      </main>
    </div>
  );
}
