interface StatCardProps {
  title: string;
  value: string | number;
  subtitle?: string;
  icon: React.ReactNode;
  color: "accent" | "green" | "yellow" | "purple" | "cyan" | "orange" | "blue" | "red";
  delay?: number;
}

const colorMap = {
  accent: { bg: "bg-accent/10", text: "text-accent", border: "border-accent/20" },
  green: { bg: "bg-green-dim", text: "text-green", border: "border-green/20" },
  yellow: { bg: "bg-yellow-dim", text: "text-yellow", border: "border-yellow/20" },
  purple: { bg: "bg-purple-dim", text: "text-purple", border: "border-purple/20" },
  cyan: { bg: "bg-cyan-dim", text: "text-cyan", border: "border-cyan/20" },
  orange: { bg: "bg-orange-dim", text: "text-orange", border: "border-orange/20" },
  blue: { bg: "bg-blue-dim", text: "text-blue", border: "border-blue/20" },
  red: { bg: "bg-red-dim", text: "text-red", border: "border-red/20" },
};

export function StatCard({ title, value, subtitle, icon, color, delay = 0 }: StatCardProps) {
  const colors = colorMap[color];
  const delayClass = delay > 0 ? `animate-fade-in-delay-${delay}` : "";

  return (
    <div className={`animate-fade-in ${delayClass} group relative bg-surface-1 border border-border rounded-xl p-5 hover:border-border-hover transition-all duration-200 hover:shadow-lg hover:shadow-black/20`}>
      <div className="flex items-start justify-between mb-4">
        <div className={`${colors.bg} ${colors.border} border rounded-lg p-2.5`}>
          <div className={colors.text}>{icon}</div>
        </div>
      </div>
      <div className="space-y-1">
        <p className="text-[13px] text-text-secondary font-medium">{title}</p>
        <p className="text-3xl font-bold tracking-tight text-text-primary">{value}</p>
        {subtitle && (
          <p className="text-xs text-text-tertiary">{subtitle}</p>
        )}
      </div>
      <div className={`absolute inset-x-0 bottom-0 h-[2px] ${colors.bg} rounded-b-xl opacity-0 group-hover:opacity-100 transition-opacity`} />
    </div>
  );
}
