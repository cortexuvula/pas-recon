export interface ReconciliationResult {
  summary: Summary;
  emr_no_match: DisplayRow[];
  pas_match_review: DisplayRow[];
  pas_no_match: DisplayRow[];
}

export interface Summary {
  matched: number;
  emr_only: number;
  pas_only: number;
  pas_review: number;
  status_breakdown: StatusBreakdown;
  duplicates_dropped: number;
  invalid_phn_skipped: number;
  unparseable_dates: number;
}

export interface StatusBreakdown {
  confirmed: number;
  pending: number;
  deceased: number;
  removed: number;
  not_the_mrp: number;
}

export interface DisplayRow {
  phn: string;
  first_name: string | null;
  last_name: string | null;
  dob: string | null;
  mrp_status: string | null;
  raw_fields: string[];
}

export interface UpdateInfo {
  version: string;
  current_version: string;
}

export type ListKey = "emr_no_match" | "pas_match_review" | "pas_no_match";
