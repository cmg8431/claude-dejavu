"use client";

import { useState, useMemo } from "react";
import styles from "./log-viewer.module.css";

interface LogEntry {
  timestamp: string;
  level: string;
  component: string;
  message: string;
}

interface LogViewerProps {
  entries: LogEntry[];
}

const LEVELS = ["DEBUG", "INFO", "WARN", "ERROR"] as const;
const COMPONENTS = ["Detector", "Parser", "DB", "Hook"] as const;

const levelStyles: Record<string, { base: string; active: string }> = {
  DEBUG: { base: styles.levelDebug, active: styles.levelDebugActive },
  INFO: { base: styles.levelInfo, active: styles.levelInfoActive },
  WARN: { base: styles.levelWarn, active: styles.levelWarnActive },
  ERROR: { base: styles.levelError, active: styles.levelErrorActive },
};

const logLevelClass: Record<string, string> = {
  DEBUG: styles.logLevelDebug,
  INFO: styles.logLevelInfo,
  WARN: styles.logLevelWarn,
  ERROR: styles.logLevelError,
};

function formatTime(ts: string): string {
  try {
    const d = new Date(ts);
    return d.toLocaleTimeString("en-US", {
      hour12: false,
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  } catch {
    return ts;
  }
}

export function LogViewer({ entries }: LogViewerProps) {
  const [activeLevels, setActiveLevels] = useState<Set<string>>(
    new Set(LEVELS)
  );
  const [activeComponents, setActiveComponents] = useState<Set<string>>(
    new Set(COMPONENTS)
  );

  function toggleLevel(level: string) {
    setActiveLevels((prev) => {
      const next = new Set(prev);
      if (next.has(level)) {
        next.delete(level);
      } else {
        next.add(level);
      }
      return next;
    });
  }

  function toggleComponent(comp: string) {
    setActiveComponents((prev) => {
      const next = new Set(prev);
      if (next.has(comp)) {
        next.delete(comp);
      } else {
        next.add(comp);
      }
      return next;
    });
  }

  const filtered = useMemo(
    () =>
      entries.filter(
        (e) => activeLevels.has(e.level) && activeComponents.has(e.component)
      ),
    [entries, activeLevels, activeComponents]
  );

  return (
    <div className={styles.wrapper}>
      <div className={styles.filters}>
        <div className={styles.filterGroup}>
          {LEVELS.map((level) => {
            const isActive = activeLevels.has(level);
            const ls = levelStyles[level];
            return (
              <button
                key={level}
                className={`${styles.filterBtn} ${ls.base} ${isActive ? `${styles.filterBtnActive} ${ls.active}` : ""}`}
                onClick={() => toggleLevel(level)}
              >
                {level}
              </button>
            );
          })}
        </div>
        <div className={styles.separator} />
        <div className={styles.filterGroup}>
          {COMPONENTS.map((comp) => {
            const isActive = activeComponents.has(comp);
            return (
              <button
                key={comp}
                className={`${styles.filterBtn} ${styles.componentBtn} ${isActive ? `${styles.filterBtnActive} ${styles.componentBtnActive}` : ""}`}
                onClick={() => toggleComponent(comp)}
              >
                {comp}
              </button>
            );
          })}
        </div>
      </div>
      <div className={styles.logArea}>
        {filtered.length === 0 ? (
          <div className={styles.empty}>No log entries match the current filters.</div>
        ) : (
          filtered.map((entry, i) => (
            <div key={i} className={styles.logLine}>
              <span className={styles.logTimestamp}>
                [{formatTime(entry.timestamp)}]
              </span>{" "}
              <span className={logLevelClass[entry.level] || styles.logLevelInfo}>
                [{entry.level.padEnd(5)}]
              </span>{" "}
              <span className={styles.logComponent}>
                [{entry.component.padEnd(8)}]
              </span>{" "}
              <span className={styles.logMessage}>{entry.message}</span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
