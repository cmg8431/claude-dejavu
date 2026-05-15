import styles from "./feed-card.module.css";
import type { FeedItem } from "../lib/db";

const BADGE_MAP: Record<string, { label: string; className: string }> = {
  revert_cycle: { label: "Revert Cycle", className: styles.badgeOrange },
  repeated_error: { label: "Repeated Error", className: styles.badgeRed },
  silent_fix: { label: "Silent Fix", className: styles.badgePurple },
  user_correction: { label: "User Correction", className: styles.badgeCyan },
  long_bash: { label: "Long Bash", className: styles.badgeBlue },
  rule: { label: "Rule", className: styles.badgeDefault },
};

function shortenProject(p: string): string {
  const parts = p.split("/");
  return parts.length > 2 ? parts.slice(-2).join("/") : p;
}

function formatTimestamp(ts: string): string {
  try {
    const d = new Date(ts);
    return d.toLocaleString("en-US", {
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  } catch {
    return ts;
  }
}

interface FeedCardProps {
  item: FeedItem;
}

export function FeedCard({ item }: FeedCardProps) {
  const badge = BADGE_MAP[item.detector_type] || {
    label: item.detector_type,
    className: styles.badgeDefault,
  };

  return (
    <div className={styles.card}>
      <div className={styles.top}>
        <span className={`${styles.badge} ${badge.className}`}>
          {badge.label}
        </span>
        <span className={styles.project}>
          {shortenProject(item.project_path)}
        </span>
      </div>
      <div className={styles.body}>{item.text}</div>
      <div className={styles.footer}>
        {item.id} &bull; {formatTimestamp(item.timestamp)}
      </div>
    </div>
  );
}
