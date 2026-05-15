interface StatusBadgeProps {
  status: string;
}

export function StatusBadge({ status }: StatusBadgeProps) {
  const styles: Record<string, string> = {
    active: "bg-green-dim text-green border-green/20",
    proposed: "bg-yellow-dim text-yellow border-yellow/20",
    dead: "bg-gray-dim text-gray border-gray/20",
    rejected: "bg-red-dim text-red border-red/20",
  };

  const style = styles[status] || styles.proposed;

  return (
    <span className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-[11px] font-semibold uppercase tracking-wider border ${style}`}>
      <span className="w-1.5 h-1.5 rounded-full bg-current" />
      {status}
    </span>
  );
}
