import styles from "./rules-table.module.css";
import type { Rule } from "../lib/db";

function getGrade(confidence: number): { letter: string; className: string } {
  if (confidence >= 0.8) return { letter: "A", className: styles.gradeA };
  if (confidence >= 0.6) return { letter: "B", className: styles.gradeB };
  if (confidence >= 0.4) return { letter: "C", className: styles.gradeC };
  return { letter: "D", className: styles.gradeD };
}

function getStatusPill(status: string): string {
  if (status === "active") return styles.pillActive;
  if (status === "proposed") return styles.pillProposed;
  return styles.pillDead;
}

interface RulesTableProps {
  rules: Rule[];
}

export function RulesTable({ rules }: RulesTableProps) {
  if (rules.length === 0) {
    return <div className={styles.empty}>No rules found.</div>;
  }

  return (
    <div className={styles.wrapper}>
      <table className={styles.table}>
        <thead>
          <tr>
            <th>Grade</th>
            <th>Rule</th>
            <th>Confidence</th>
            <th>Fires</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          {rules.map((rule) => {
            const grade = getGrade(rule.confidence);
            return (
              <tr key={rule.id}>
                <td>
                  <span className={`${styles.grade} ${grade.className}`}>
                    {grade.letter}
                  </span>
                </td>
                <td className={styles.ruleText}>
                  {rule.text.length > 140
                    ? rule.text.slice(0, 140) + "..."
                    : rule.text}
                </td>
                <td>
                  <span className={styles.confidenceBar}>
                    <span
                      className={styles.confidenceFill}
                      style={{ width: `${Math.round(rule.confidence * 100)}%` }}
                    />
                  </span>
                  <span className={styles.confidenceValue}>
                    {(rule.confidence * 100).toFixed(0)}%
                  </span>
                </td>
                <td className={styles.fires}>{rule.fire_count}</td>
                <td>
                  <span className={`${styles.pill} ${getStatusPill(rule.status)}`}>
                    {rule.status}
                  </span>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
