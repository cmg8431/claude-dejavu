import { getOverviewStats, getAllRules } from "@/lib/db";
import { StatCard } from "@/components/stat-card";
import { PageHeader } from "@/components/page-header";
import { StatusBadge } from "@/components/status-badge";

export const dynamic = "force-dynamic";

export default function HomePage() {
  const stats = getOverviewStats();
  const recentRules = getAllRules().slice(0, 5);

  return (
    <>
      <PageHeader
        title="Overview"
        description="Real-time insights into your CLAUDE.md rule engine"
      />

      {/* Stats Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-10">
        <StatCard
          title="Patterns Detected"
          value={stats.totalPatterns}
          subtitle="Across all projects"
          color="accent"
          delay={1}
          icon={
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" d="M7.5 14.25v2.25m3-4.5v4.5m3-6.75v6.75m3-9v9M6 20.25h12A2.25 2.25 0 0 0 20.25 18V6A2.25 2.25 0 0 0 18 3.75H6A2.25 2.25 0 0 0 3.75 6v12A2.25 2.25 0 0 0 6 20.25Z" />
            </svg>
          }
        />
        <StatCard
          title="Active Rules"
          value={stats.activeRules}
          subtitle={`${stats.totalRules} total rules`}
          color="green"
          delay={2}
          icon={
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" d="M9 12.75 11.25 15 15 9.75m-3-7.036A11.959 11.959 0 0 1 3.598 6 11.99 11.99 0 0 0 3 9.749c0 5.592 3.824 10.29 9 11.623 5.176-1.332 9-6.03 9-11.622 0-1.31-.21-2.571-.598-3.751h-.152c-3.196 0-6.1-1.248-8.25-3.285Z" />
            </svg>
          }
        />
        <StatCard
          title="Effectiveness"
          value={`${stats.effectiveness}%`}
          subtitle="Mistakes prevented"
          color="cyan"
          delay={3}
          icon={
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 13.5l10.5-11.25L12 10.5h8.25L9.75 21.75 12 13.5H3.75z" />
            </svg>
          }
        />
        <StatCard
          title="Total Fires"
          value={stats.totalFireCount}
          subtitle="Rules triggered"
          color="orange"
          delay={4}
          icon={
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" d="M15.362 5.214A8.252 8.252 0 0 1 12 21 8.25 8.25 0 0 1 6.038 7.047 8.287 8.287 0 0 0 9 9.601a8.983 8.983 0 0 1 3.361-6.867 8.21 8.21 0 0 0 3 2.48Z" />
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 18a3.75 3.75 0 0 0 .495-7.468 5.99 5.99 0 0 0-1.925 3.547 5.975 5.975 0 0 1-2.133-1.001A3.75 3.75 0 0 0 12 18Z" />
            </svg>
          }
        />
      </div>

      {/* Recent Rules */}
      <div className="animate-fade-in animate-fade-in-delay-4">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-text-primary">Recent Rules</h2>
          <a href="/rules" className="text-xs text-accent hover:text-accent-hover font-medium transition-colors">
            View all &rarr;
          </a>
        </div>

        <div className="bg-surface-1 border border-border rounded-xl overflow-hidden">
          {recentRules.length === 0 ? (
            <div className="px-6 py-12 text-center">
              <p className="text-sm text-text-tertiary">No rules yet. Run dejavu scan to detect patterns.</p>
            </div>
          ) : (
            <table className="w-full">
              <thead>
                <tr className="border-b border-border">
                  <th className="text-left px-5 py-3 text-[11px] font-semibold uppercase tracking-wider text-text-tertiary">Rule</th>
                  <th className="text-left px-5 py-3 text-[11px] font-semibold uppercase tracking-wider text-text-tertiary">Confidence</th>
                  <th className="text-left px-5 py-3 text-[11px] font-semibold uppercase tracking-wider text-text-tertiary">Fires</th>
                  <th className="text-left px-5 py-3 text-[11px] font-semibold uppercase tracking-wider text-text-tertiary">Status</th>
                </tr>
              </thead>
              <tbody>
                {recentRules.map((rule) => (
                  <tr key={rule.id} className="border-b border-border/50 last:border-0 hover:bg-surface-2/50 transition-colors">
                    <td className="px-5 py-3.5">
                      <div className="flex items-start gap-3">
                        <span className="text-[11px] font-mono text-text-tertiary bg-surface-2 px-1.5 py-0.5 rounded mt-0.5 shrink-0">{rule.id}</span>
                        <span className="text-sm text-text-primary leading-snug line-clamp-2">{rule.text}</span>
                      </div>
                    </td>
                    <td className="px-5 py-3.5">
                      <div className="flex items-center gap-2">
                        <div className="w-16 h-1.5 bg-surface-3 rounded-full overflow-hidden">
                          <div
                            className="h-full bg-accent rounded-full"
                            style={{ width: `${Math.round(rule.confidence * 100)}%` }}
                          />
                        </div>
                        <span className="text-xs text-text-secondary font-mono">{(rule.confidence * 100).toFixed(0)}%</span>
                      </div>
                    </td>
                    <td className="px-5 py-3.5 text-sm text-text-secondary font-mono">{rule.fire_count}</td>
                    <td className="px-5 py-3.5"><StatusBadge status={rule.status} /></td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>
    </>
  );
}
