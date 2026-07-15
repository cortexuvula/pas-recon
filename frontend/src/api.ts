import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import type { UnlistenFn } from "@tauri-apps/api/event";
import type { ReconciliationResult, UpdateInfo, DisplayRow, ListKey } from "./types";

export async function reconcileFiles(emrPath: string, pasPath: string): Promise<ReconciliationResult> {
  return invoke<ReconciliationResult>("reconcile_files", {
    emrPath,
    pasPath,
  });
}

export async function reconcileWithColumnOverride(
  emrPath: string,
  pasPath: string,
  emrPhnColumn: number | null,
  pasPhnColumn: number | null,
): Promise<ReconciliationResult> {
  return invoke<ReconciliationResult>("reconcile_with_column_override", {
    emrPath,
    pasPath,
    emrPhnColumn,
    pasPhnColumn,
  });
}

export async function exportList(rows: DisplayRow[], path: string): Promise<void> {
  await invoke("export_list", { rows, path });
}

export async function checkForUpdates(): Promise<UpdateInfo | null> {
  return invoke<UpdateInfo | null>("check_for_updates");
}

export async function getCsvHeaders(path: string): Promise<string[]> {
  return invoke<string[]>("get_csv_headers", { path });
}

export function onUpdateAvailable(callback: (info: UpdateInfo) => void) {
  return listen<UpdateInfo>("update-available", (event) => {
    callback(event.payload);
  });
}

/**
 * Register a callback for Tauri-native drag-and-drop events.
 * Unlike HTML5 drag events, this provides real file paths from the OS.
 * The callback receives the drop type and an array of file paths.
 *
 * Types: "enter" (drag entered window), "over" (dragging over),
 *        "drop" (files dropped), "leave" (drag left window)
 */
export function onDragDropEvent(
  callback: (event: { type: "enter" | "over" | "drop" | "leave"; paths: string[] }) => void,
): Promise<UnlistenFn> {
  return getCurrentWebview().onDragDropEvent((event) => {
    const payload = event.payload;
    callback({
      type: payload.type,
      paths: "paths" in payload ? payload.paths : [],
    });
  });
}
