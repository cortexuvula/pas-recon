# Smart File Ingestion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace filename-based file classification with header-based detection so dropped CSVs are correctly identified as EMR or PAS regardless of filename.

**Architecture:** Frontend-only change. When files are dropped, read each file's headers via the existing `get_csv_headers` backend command. Detect the PAS file by checking for "PAS MRP Status" / "PAS MRP Updated" headers. The other file is EMR by elimination. Show a confirmation dialog only when classification is ambiguous.

**Tech Stack:** React, TypeScript, Tauri IPC (existing `get_csv_headers` command).

**Spec:** `docs/superpowers/specs/2026-07-15-smart-file-ingestion-design.md`

---

## File Structure

```
frontend/src/
├── App.tsx                        # Modify: replace filename logic with header-based classification
├── components/
│   └── FileConfirm.tsx            # Create: confirmation dialog for ambiguous cases
└── api.ts                         # No change (get_csv_headers already exists)
```

---

## Task 1: Create FileConfirm component

**Files:**
- Create: `frontend/src/components/FileConfirm.tsx`

This is the confirmation dialog shown when the app can't confidently classify which dropped file is EMR vs PAS. It shows each filename with its detected headers and offers Confirm / Swap buttons.

- [ ] **Step 1: Create FileConfirm.tsx**

Create `frontend/src/components/FileConfirm.tsx`:

```tsx
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
```

- [ ] **Step 2: Verify the frontend builds**

Run:
```bash
cd frontend && npm run build
```
Expected: builds successfully (the component is unused so far, but must compile).

- [ ] **Step 3: Commit**

```bash
git add frontend/src/components/FileConfirm.tsx
git commit -m "feat(frontend): add FileConfirm component for ambiguous file classification"
```

---

## Task 2: Header-based classification in App.tsx

**Files:**
- Modify: `frontend/src/App.tsx`

Replace the `name.includes("emr")` / `name.includes("pas")` logic in `handlePathsDropped` with header-based detection. When two files are dropped:
1. Fetch headers for both via `get_csv_headers`
2. Detect PAS file by checking for "PAS MRP Status" / "PAS MRP Updated" headers
3. If confident → proceed with reconciliation
4. If ambiguous → show FileConfirm dialog

- [ ] **Step 1: Add imports and state**

In `frontend/src/App.tsx`, add the import for FileConfirm at the top (with the other component imports):

```tsx
import FileConfirm from "./components/FileConfirm";
```

Add `getCsvHeaders` to the imports from api:

```tsx
import {
  reconcileFiles,
  reconcileWithColumnOverride,
  exportList,
  getCsvHeaders,
  onUpdateAvailable,
  onDragDropEvent,
} from "./api";
```

Add state for the confirmation dialog (near the other `useState` declarations):

```tsx
const [showFileConfirm, setShowFileConfirm] = useState(false);
const [pendingEmrPath, setPendingEmrPath] = useState<string | null>(null);
const [pendingPasPath, setPendingPasPath] = useState<string | null>(null);
const [pendingEmrFilename, setPendingEmrFilename] = useState("");
const [pendingPasFilename, setPendingPasFilename] = useState("");
const [pendingEmrHeaders, setPendingEmrHeaders] = useState<string[]>([]);
const [pendingPasHeaders, setPendingPasHeaders] = useState<string[]>([]);
```

- [ ] **Step 2: Add the classification helper function**

Add this helper function inside the `App` component, before `handlePathsDropped`:

```tsx
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
      const emr1 = hasFilenameSignal(path1, "emr") || hasFilenameSignal(path1, "active patient");
      const emr2 = hasFilenameSignal(path2, "emr") || hasFilenameSignal(path2, "active patient");
      const pas1name = hasFilenameSignal(path1, "pas");
      const pas2name = hasFilenameSignal(path2, "pas");

      if (emr1 && pas2name) return [path1, path2];
      if (emr2 && pas1name) return [path2, path1];
    }

    // Both have PAS headers, or no signal at all → ambiguous
    return null;
  };
```

- [ ] **Step 3: Replace handlePathsDropped with header-based logic**

Replace the ENTIRE existing `handlePathsDropped` callback with:

```tsx
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
  }, []);
```

- [ ] **Step 4: Add the runReconciliation helper**

Add this helper (extracted from the old inline logic so both the auto and confirm paths can use it). Place it after `handlePathsDropped`:

```tsx
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
```

- [ ] **Step 5: Add FileConfirm handlers**

Add these handlers after `runReconciliation`:

```tsx
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
      // Swap: what we guessed was EMR is actually PAS and vice versa
      setEmrPath(pendingPasPath);
      setPasPath(pendingEmrPath);
      setEmrLoaded(true);
      setPasLoaded(true);
      await runReconciliation(pendingPasPath, pendingEmrPath);
    }
  }, [pendingEmrPath, pendingPasPath, runReconciliation]);
```

- [ ] **Step 6: Add FileConfirm to the JSX**

In the main panel's JSX, add the FileConfirm rendering. It should appear when `showFileConfirm` is true, BEFORE the column picker and result/empty-state ternary. Find the line:

```tsx
        {showColumnPicker && emrPath && pasPath ? (
```

Insert BEFORE it:

```tsx
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
```

Make sure the closing `)` for the FileConfirm ternary is added before the existing `showColumnPicker` ternary closes. The full structure should be:

```tsx
        {showFileConfirm ? (
          <FileConfirm ... />
        ) : showColumnPicker && emrPath && pasPath ? (
          <ColumnPicker ... />
        ) : result ? (
          // ... existing result rendering
        ) : (
          <EmptyState ... />
        )}
```

- [ ] **Step 7: Remove stale refs that are no longer needed**

The `emrPathRef` and `pasPathRef` refs were needed when the drag-drop event listener captured stale state. Now that `handlePathsDropped` uses `getCsvHeaders` and doesn't reference `emrPath`/`pasPath` directly (it reads headers fresh each time), the refs are no longer needed. Remove:

- The `emrPathRef` and `pasPathRef` declarations
- The two `useEffect` blocks that sync them
- Update the `useEffect` drag-drop listener deps: `}, [handlePathsDropped]);` stays as the dependency

- [ ] **Step 8: Verify the frontend builds**

Run:
```bash
cd frontend && npm run build
```
Expected: builds successfully.

- [ ] **Step 9: Commit**

```bash
git add frontend/src/App.tsx frontend/src/components/FileConfirm.tsx
git commit -m "feat: header-based file classification replaces filename matching

Files dropped into the app are now classified by inspecting their CSV
headers rather than relying on filenames containing 'emr'/'pas'. The
PAS file is detected by 'PAS MRP Status' or 'PAS MRP Updated' headers.
When classification is ambiguous, a confirmation dialog shows both
files with their detected headers and offers Confirm/Swap buttons."
```

---

## Self-Review

**1. Spec coverage:**

| Spec requirement | Covered by |
|---|---|
| §2.1 PAS detection by "PAS MRP Status"/"PAS MRP Updated" headers | Task 2 Step 2: `classifyFiles` |
| §2.1 EMR by elimination | Task 2 Step 2: `classifyFiles` returns `[other, pas]` |
| §2.1 Filename fallback | Task 2 Step 2: `hasFilenameSignal` fallback |
| §2.2 Ambiguous → show dialog | Task 2 Step 3: `setShowFileConfirm(true)` |
| §2.2 Auto-classify when confident → proceed silently | Task 2 Step 3: `classified` truthy path |
| §2.3 Confirmation dialog with headers + Confirm/Swap | Task 1: FileConfirm component + Task 2 Step 5 |
| §2.4 No engine/backend changes | Confirmed — only frontend files touched |

All sections covered. ✓

**2. Placeholder scan:** No TBDs, TODOs, or vague steps. All code shown in full. ✓

**3. Type consistency:**
- `FileConfirmProps`: `emrFilename`, `pasFilename`, `emrHeaders`, `pasHeaders`, `onConfirm`, `onSwap` — matches usage in Task 2 Steps 5 and 6. ✓
- `classifyFiles` returns `[string, string] | null` — destructured as `[emrPath, pasPath]` in Task 2 Step 3. ✓
- `runReconciliation(emr: string, pas: string)` — called from `handlePathsDropped`, `handleFileConfirm`, and `handleFileSwap` with correct args. ✓
- State variables `pendingEmrPath` etc. — set in Step 3, read in Steps 5 and 6. ✓

All consistent. ✓
