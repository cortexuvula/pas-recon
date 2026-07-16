# Click-to-Sort Table Columns — Design Spec

**Date:** 2026-07-16
**Status:** Approved design, pending implementation plan

## Problem

Table columns are not sortable. Rows render in the engine's default last-name sort order with no way for the user to re-sort. The user specifically wants to sort the PAS No Match list by the Status column.

## Solution

Make all column headers clickable to sort. Frontend-only change in `PatientTable.tsx`.

### Behavior

- Click a column header → sort ascending by that column
- Click again → sort descending
- Click a different header → switch to that column, ascending
- Sort indicator (`▲` ascending / `▼` descending) on the active column
- Default: last name ascending (engine's default order, preserved on first load)
- Sorting applies on top of the search filter (filter first, then sort)
- Case-insensitive for text fields
- Null/empty values sort to the bottom in ascending order (top in descending)

### What changes

| File | Change |
|---|---|
| `frontend/src/components/PatientTable.tsx` | Add `sortKey` and `sortDir` state. Render headers as clickable `<th>` with sort indicators. Apply sort in the `useMemo` after filtering. |

### What stays the same

- No engine or backend changes
- No new files or components
- Search filter behavior unchanged
- Row click-to-toggle-resolved unchanged

## Out of scope

- Multi-column sort (sort by X then Y)
- Persistent sort preference across sessions
