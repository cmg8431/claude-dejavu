"use client";

import { useRouter, useSearchParams } from "next/navigation";

const statuses = [
  { value: "all", label: "All" },
  { value: "active", label: "Active" },
  { value: "proposed", label: "Proposed" },
  { value: "dead", label: "Dead" },
  { value: "rejected", label: "Rejected" },
];

export function RulesFilter({ current }: { current: string }) {
  const router = useRouter();

  return (
    <div className="flex items-center gap-1.5 mb-6">
      {statuses.map((s) => (
        <button
          key={s.value}
          onClick={() => {
            const params = s.value === "all" ? "" : `?status=${s.value}`;
            router.push(`/rules${params}`);
          }}
          className={`
            px-3.5 py-1.5 rounded-lg text-xs font-medium transition-all duration-150
            ${current === s.value
              ? "bg-accent/15 text-accent border border-accent/25"
              : "bg-surface-2 text-text-secondary border border-transparent hover:text-text-primary hover:bg-surface-3"
            }
          `}
        >
          {s.label}
        </button>
      ))}
    </div>
  );
}
