import styles from "./stat-grid.module.css";
import type { OverviewStats } from "../lib/db";

interface StatGridProps {
  stats: OverviewStats;
}

export function StatGrid({ stats }: StatGridProps) {
  return (
    <div className={styles.grid}>
      <div className={styles.card}>
        <span className={styles.label}>Patterns Detected</span>
        <span className={`${styles.value} ${styles.accent}`}>
          {stats.totalPatterns}
        </span>
      </div>
      <div className={styles.card}>
        <span className={styles.label}>Active Rules</span>
        <span className={`${styles.value} ${styles.cyan}`}>
          {stats.activeRules}
        </span>
      </div>
      <div className={styles.card}>
        <span className={styles.label}>Effectiveness</span>
        <span className={`${styles.value} ${styles.amber}`}>
          {stats.effectiveness}%
        </span>
      </div>
      <div className={styles.card}>
        <span className={styles.label}>Total Fires</span>
        <span className={styles.value}>{stats.totalFires}</span>
      </div>
    </div>
  );
}
