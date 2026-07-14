import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
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

export async function exportList(list: ListKey, rows: DisplayRow[], path: string): Promise<void> {
  const listKind = list === "emr_no_match" ? "EmrNoMatch"
    : list === "pas_match_review" ? "PasMatchReview"
    : "PasNoMatch";
  await invoke("export_list", { list: listKind, rows, path });
}

export async function checkForUpdates(): Promise<UpdateInfo | null> {
  return invoke<UpdateInfo | null>("check_for_updates");
}

export function onUpdateAvailable(callback: (info: UpdateInfo) => void) {
  return listen<UpdateInfo>("update-available", (event) => {
    callback(event.payload);
  });
}
