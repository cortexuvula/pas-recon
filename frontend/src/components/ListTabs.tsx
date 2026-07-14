import type { ListKey, Summary } from "../types";

interface ListTabsProps {
  active: ListKey;
  onSelect: (key: ListKey) => void;
  summary: Summary;
}

const TAB_CONFIG: { key: ListKey; label: string; countKey: keyof Summary }[] = [
  { key: "emr_no_match", label: "EMR No Match", countKey: "emr_only" },
  { key: "pas_match_review", label: "PAS Match - Review", countKey: "pas_review" },
  { key: "pas_no_match", label: "PAS No Match", countKey: "pas_only" },
];

const CONTEXT_LINES: Record<ListKey, string> = {
  emr_no_match: "Patients in your EMR panel but not found in PAS. These may need a 98990 bill submitted, or have incorrect status/MRP in the EMR.",
  pas_match_review: "Matched patients with a status of Pending, Not the MRP, Deceased, or Removed. These may need updating in your EMR.",
  pas_no_match: "Patients in PAS but not in your EMR panel. These may have left the clinic, or the EMR status/MRP is incorrect.",
};

export { CONTEXT_LINES };

export default function ListTabs({ active, onSelect, summary }: ListTabsProps) {
  return (
    <>
      <div className="list-tabs">
        {TAB_CONFIG.map((tab) => {
          const count = summary[tab.countKey] as number;
          return (
            <button
              key={tab.key}
              className={`tab ${active === tab.key ? "active" : ""}`}
              onClick={() => onSelect(tab.key)}
            >
              {tab.label}
              <span className="tab-badge">({count})</span>
            </button>
          );
        })}
      </div>
      <div className="context-line">{CONTEXT_LINES[active]}</div>
    </>
  );
}
