import { getTimelineEvents } from "@/lib/db";
import { PageHeader } from "@/components/page-header";
import { EmptyState } from "@/components/empty-state";

export const dynamic = "force-dynamic";

const eventConfig: Record<string, { color: string; bg: string; icon: string; label: string }> = {
  rule_created: {
    color: "text-green",
    bg: "bg-green-dim border-green/20",
    icon: "+",
    label: "Rule Created",
  },
  rule_fired: {
    color: "text-orange",
    bg: "bg-orange-dim border-orange/20",
    icon: "!",
    label: "Rule Fired",
  },
  rule_died: {
    color: "text-gray",
    bg: "bg-gray-dim border-gray/20",
    icon: "x",
    label: "Rule Died",
  },
  pattern_detected: {
    color: "text-accent",
    bg: "bg-accent/10 border-accent/20",
    icon: "~",
    label: "Pattern Detected",
  },
};

export default function TimelinePage() {
  const events = getTimelineEvents();

  // Group events by date
  const grouped = events.reduce<Record<string, typeof events>>((acc, event) => {
    const date = formatGroupDate(event.timestamp);
    if (!acc[date]) acc[date] = [];
    acc[date].push(event);
    return acc;
  }, {});

  return (
    <>
      <PageHeader
        title="Timeline"
        description="Evolution of your CLAUDE.md rules over time"
      />

      {events.length === 0 ? (
        <EmptyState
          title="No events yet"
          description="Your timeline will populate as dejavu detects patterns, creates rules, and tracks rule fires."
          icon={
            <svg className="w-6 h-6 text-text-tertiary" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" d="M12 6v6h4.5m4.5 0a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
            </svg>
          }
        />
      ) : (
        <div className="relative">
          {/* Vertical line */}
          <div className="absolute left-[23px] top-0 bottom-0 w-px bg-border" />

          <div className="space-y-8">
            {Object.entries(grouped).map(([date, dayEvents]) => (
              <div key={date}>
                {/* Date header */}
                <div className="relative flex items-center gap-4 mb-4">
                  <div className="w-[47px] flex justify-center">
                    <div className="w-3 h-3 rounded-full bg-surface-3 border-2 border-border z-10" />
                  </div>
                  <span className="text-xs font-semibold text-text-tertiary uppercase tracking-wider">{date}</span>
                </div>

                {/* Events for this date */}
                <div className="space-y-3">
                  {dayEvents.map((event, i) => {
                    const config = eventConfig[event.event_type] || eventConfig.pattern_detected;

                    return (
                      <div key={`${event.event_type}-${event.id}-${i}`} className="relative flex items-start gap-4 animate-fade-in">
                        {/* Timeline dot */}
                        <div className="w-[47px] flex justify-center shrink-0">
                          <div className={`w-7 h-7 rounded-lg ${config.bg} border flex items-center justify-center z-10`}>
                            <span className={`text-xs font-bold ${config.color}`}>{config.icon}</span>
                          </div>
                        </div>

                        {/* Event card */}
                        <div className="flex-1 bg-surface-1 border border-border rounded-xl p-4 hover:border-border-hover transition-all duration-200 group">
                          <div className="flex items-start justify-between mb-1.5">
                            <span className={`text-[11px] font-semibold uppercase tracking-wider ${config.color}`}>
                              {config.label}
                            </span>
                            <span className="text-[11px] text-text-tertiary">{formatTime(event.timestamp)}</span>
                          </div>

                          {event.text && (
                            <p className="text-sm text-text-primary leading-relaxed">{event.text}</p>
                          )}

                          {event.event_type === "pattern_detected" && (
                            <div className="flex items-center gap-2 mt-2">
                              <span className="text-xs bg-surface-2 text-text-secondary px-2 py-0.5 rounded font-mono">
                                {event.detector_type}
                              </span>
                              {event.project_path && (
                                <span className="text-xs text-text-tertiary font-mono truncate max-w-[300px]">
                                  {event.project_path}
                                </span>
                              )}
                            </div>
                          )}

                          {event.event_type === "rule_fired" && (
                            <div className="flex items-center gap-2 mt-2">
                              <span className="text-xs font-mono text-text-tertiary bg-surface-2 px-2 py-0.5 rounded">
                                {event.rule_id}
                              </span>
                              {event.prevented === 1 && (
                                <span className="text-[11px] font-semibold text-green bg-green-dim px-2 py-0.5 rounded-full">
                                  PREVENTED
                                </span>
                              )}
                            </div>
                          )}

                          {event.event_type === "rule_created" && event.confidence !== undefined && (
                            <div className="flex items-center gap-3 mt-2">
                              <div className="flex items-center gap-1.5">
                                <div className="w-12 h-1 bg-surface-3 rounded-full overflow-hidden">
                                  <div className="h-full bg-accent rounded-full" style={{ width: `${Math.round(event.confidence * 100)}%` }} />
                                </div>
                                <span className="text-[11px] text-text-tertiary font-mono">{(event.confidence * 100).toFixed(0)}%</span>
                              </div>
                            </div>
                          )}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}
    </>
  );
}

function formatGroupDate(dateStr: string): string {
  try {
    const d = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return "Today";
    if (diffDays === 1) return "Yesterday";
    if (diffDays < 7) return `${diffDays} days ago`;

    return d.toLocaleDateString("en-US", { month: "long", day: "numeric", year: "numeric" });
  } catch {
    return dateStr;
  }
}

function formatTime(dateStr: string): string {
  try {
    return new Date(dateStr).toLocaleTimeString("en-US", { hour: "numeric", minute: "2-digit" });
  } catch {
    return "";
  }
}
