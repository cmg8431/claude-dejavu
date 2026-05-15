import { getAllRules } from "@/lib/db";
import { PageHeader } from "@/components/page-header";
import { StatusBadge } from "@/components/status-badge";
import { EmptyState } from "@/components/empty-state";
import { RulesFilter } from "./filter";

export const dynamic = "force-dynamic";

interface PageProps {
  searchParams: Promise<{ status?: string }>;
}

export default async function RulesPage({ searchParams }: PageProps) {
  const params = await searchParams;
  const status = params.status || "all";
  const rules = getAllRules(status);

  return (
    <>
      <PageHeader
        title="Rules"
        description="All learned rules extracted from pattern detection"
      />

      <RulesFilter current={status} />

      <div className="animate-fade-in bg-surface-1 border border-border rounded-xl overflow-hidden">
        {rules.length === 0 ? (
          <EmptyState
            title="No rules found"
            description={status !== "all" ? `No rules with status "${status}". Try a different filter.` : "Run dejavu scan on your projects to start detecting patterns and generating rules."}
          />
        ) : (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead>
                <tr className="border-b border-border">
                  <th className="text-left px-5 py-3 text-[11px] font-semibold uppercase tracking-wider text-text-tertiary">ID</th>
                  <th className="text-left px-5 py-3 text-[11px] font-semibold uppercase tracking-wider text-text-tertiary min-w-[300px]">Rule Text</th>
                  <th className="text-left px-5 py-3 text-[11px] font-semibold uppercase tracking-wider text-text-tertiary">Scope</th>
                  <th className="text-left px-5 py-3 text-[11px] font-semibold uppercase tracking-wider text-text-tertiary">Confidence</th>
                  <th className="text-left px-5 py-3 text-[11px] font-semibold uppercase tracking-wider text-text-tertiary">Fires</th>
                  <th className="text-left px-5 py-3 text-[11px] font-semibold uppercase tracking-wider text-text-tertiary">Status</th>
                  <th className="text-left px-5 py-3 text-[11px] font-semibold uppercase tracking-wider text-text-tertiary">Created</th>
                </tr>
              </thead>
              <tbody>
                {rules.map((rule, i) => (
                  <tr
                    key={rule.id}
                    className="border-b border-border/50 last:border-0 hover:bg-surface-2/50 transition-colors group"
                  >
                    <td className="px-5 py-3.5">
                      <span className="text-[11px] font-mono text-text-tertiary bg-surface-2 px-2 py-0.5 rounded group-hover:bg-surface-3 transition-colors">{rule.id}</span>
                    </td>
                    <td className="px-5 py-3.5">
                      <p className="text-sm text-text-primary leading-relaxed">{rule.text}</p>
                      <p className="text-xs text-text-tertiary mt-1 font-mono truncate max-w-[400px]">{rule.project_path}</p>
                    </td>
                    <td className="px-5 py-3.5">
                      <span className="text-xs text-text-secondary bg-surface-2 px-2 py-1 rounded font-medium">{rule.scope}</span>
                    </td>
                    <td className="px-5 py-3.5">
                      <div className="flex items-center gap-2">
                        <div className="w-16 h-1.5 bg-surface-3 rounded-full overflow-hidden">
                          <div
                            className="h-full rounded-full transition-all duration-500"
                            style={{
                              width: `${Math.round(rule.confidence * 100)}%`,
                              backgroundColor: rule.confidence >= 0.8 ? "var(--color-green)" : rule.confidence >= 0.5 ? "var(--color-yellow)" : "var(--color-red)",
                            }}
                          />
                        </div>
                        <span className="text-xs text-text-secondary font-mono">{(rule.confidence * 100).toFixed(0)}%</span>
                      </div>
                    </td>
                    <td className="px-5 py-3.5">
                      <span className="text-sm font-mono text-text-secondary">{rule.fire_count}</span>
                    </td>
                    <td className="px-5 py-3.5">
                      <StatusBadge status={rule.status} />
                    </td>
                    <td className="px-5 py-3.5">
                      <span className="text-xs text-text-tertiary whitespace-nowrap">{formatDate(rule.created_at)}</span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      <div className="mt-4 text-xs text-text-tertiary text-right">
        {rules.length} rule{rules.length !== 1 ? "s" : ""} total
      </div>
    </>
  );
}

function formatDate(dateStr: string): string {
  try {
    const d = new Date(dateStr);
    return d.toLocaleDateString("en-US", { month: "short", day: "numeric", year: "numeric" });
  } catch {
    return dateStr;
  }
}
