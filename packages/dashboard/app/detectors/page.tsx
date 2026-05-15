import { getDetectorStats } from "@/lib/db";
import { PageHeader } from "@/components/page-header";

export const dynamic = "force-dynamic";

const detectorMeta: Record<string, { label: string; description: string; color: string; bg: string; borderColor: string; icon: React.ReactNode }> = {
  revert_cycle: {
    label: "Revert Cycle",
    description: "Detects when Claude reverts changes and re-applies them, indicating confusion about the correct approach",
    color: "text-red",
    bg: "bg-red-dim",
    borderColor: "border-red/20",
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" d="M16.023 9.348h4.992v-.001M2.985 19.644v-4.992m0 0h4.992m-4.993 0 3.181 3.183a8.25 8.25 0 0 0 13.803-3.7M4.031 9.865a8.25 8.25 0 0 1 13.803-3.7l3.181 3.182" />
      </svg>
    ),
  },
  repeated_error: {
    label: "Repeated Error",
    description: "Catches the same error appearing across multiple sessions, suggesting a persistent misunderstanding",
    color: "text-orange",
    bg: "bg-orange-dim",
    borderColor: "border-orange/20",
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" d="M12 9v3.75m-9.303 3.376c-.866 1.5.217 3.374 1.948 3.374h14.71c1.73 0 2.813-1.874 1.948-3.374L13.949 3.378c-.866-1.5-3.032-1.5-3.898 0L2.697 16.126ZM12 15.75h.007v.008H12v-.008Z" />
      </svg>
    ),
  },
  silent_fix: {
    label: "Silent Fix",
    description: "Identifies patterns where Claude silently changes approach without acknowledging the correction",
    color: "text-purple",
    bg: "bg-purple-dim",
    borderColor: "border-purple/20",
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" d="M3.98 8.223A10.477 10.477 0 0 0 1.934 12C3.226 16.338 7.244 19.5 12 19.5c.993 0 1.953-.138 2.863-.395M6.228 6.228A10.451 10.451 0 0 1 12 4.5c4.756 0 8.773 3.162 10.065 7.498a10.522 10.522 0 0 1-4.293 5.774M6.228 6.228 3 3m3.228 3.228 3.65 3.65m7.894 7.894L21 21m-3.228-3.228-3.65-3.65m0 0a3 3 0 1 0-4.243-4.243m4.242 4.242L9.88 9.88" />
      </svg>
    ),
  },
  user_correction: {
    label: "User Correction",
    description: "Captures explicit corrections from the user like \"no, use X instead\" or \"don't do Y\"",
    color: "text-cyan",
    bg: "bg-cyan-dim",
    borderColor: "border-cyan/20",
    icon: (
      <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
        <path strokeLinecap="round" strokeLinejoin="round" d="M20.25 8.511c.884.284 1.5 1.128 1.5 2.097v4.286c0 1.136-.847 2.1-1.98 2.193-.34.027-.68.052-1.02.072v3.091l-3-3c-1.354 0-2.694-.055-4.02-.163a2.115 2.115 0 0 1-.825-.242m9.345-8.334a2.126 2.126 0 0 0-.476-.095 48.64 48.64 0 0 0-8.048 0c-1.131.094-1.976 1.057-1.976 2.192v4.286c0 .837.46 1.58 1.155 1.951m9.345-8.334V6.637c0-1.621-1.152-3.026-2.76-3.235A48.455 48.455 0 0 0 11.25 3c-2.115 0-4.198.137-6.24.402-1.608.209-2.76 1.614-2.76 3.235v6.226c0 1.621 1.152 3.026 2.76 3.235.577.075 1.157.14 1.74.194V21l4.155-4.155" />
      </svg>
    ),
  },
};

export default function DetectorsPage() {
  const stats = getDetectorStats();

  return (
    <>
      <PageHeader
        title="Detectors"
        description="Pattern detection engines that power rule generation"
      />

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-5">
        {stats.map((detector, i) => {
          const meta = detectorMeta[detector.type];
          if (!meta) return null;

          return (
            <div
              key={detector.type}
              className={`animate-fade-in animate-fade-in-delay-${i + 1} bg-surface-1 border border-border rounded-xl overflow-hidden hover:border-border-hover transition-all duration-200 group`}
            >
              {/* Header */}
              <div className="px-5 py-4 border-b border-border/50">
                <div className="flex items-start gap-3.5">
                  <div className={`${meta.bg} ${meta.borderColor} border rounded-lg p-2.5 shrink-0`}>
                    <div className={meta.color}>{meta.icon}</div>
                  </div>
                  <div className="min-w-0 flex-1">
                    <div className="flex items-center justify-between">
                      <h3 className="text-sm font-semibold text-text-primary">{meta.label}</h3>
                      <div className="flex items-center gap-3">
                        <div className="text-right">
                          <p className="text-xl font-bold text-text-primary">{detector.patternCount}</p>
                          <p className="text-[10px] text-text-tertiary uppercase tracking-wider">patterns</p>
                        </div>
                      </div>
                    </div>
                    <p className="text-xs text-text-secondary mt-1 leading-relaxed">{meta.description}</p>
                  </div>
                </div>
              </div>

              {/* Recent detections */}
              <div className="px-5 py-3">
                <p className="text-[10px] font-semibold uppercase tracking-wider text-text-tertiary mb-2.5">Recent detections</p>
                {detector.recentDetections.length === 0 ? (
                  <p className="text-xs text-text-tertiary py-3 text-center">No detections yet</p>
                ) : (
                  <div className="space-y-2">
                    {detector.recentDetections.map((detection) => (
                      <div
                        key={detection.id}
                        className="flex items-center justify-between py-1.5 px-2.5 rounded-lg hover:bg-surface-2/50 transition-colors -mx-0.5"
                      >
                        <div className="flex items-center gap-2.5 min-w-0">
                          <div className={`w-1.5 h-1.5 rounded-full ${meta.color === "text-red" ? "bg-red" : meta.color === "text-orange" ? "bg-orange" : meta.color === "text-purple" ? "bg-purple" : "bg-cyan"}`} />
                          <span className="text-xs text-text-secondary truncate max-w-[250px] font-mono">
                            {detection.project_path.split("/").pop()}
                          </span>
                        </div>
                        <span className="text-[11px] text-text-tertiary shrink-0 ml-3">
                          {formatRelativeTime(detection.detected_at)}
                        </span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </>
  );
}

function formatRelativeTime(dateStr: string): string {
  try {
    const d = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return "just now";
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 30) return `${diffDays}d ago`;
    return d.toLocaleDateString("en-US", { month: "short", day: "numeric" });
  } catch {
    return dateStr;
  }
}
