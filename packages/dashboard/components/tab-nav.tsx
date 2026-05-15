"use client";

import Link from "next/link";
import { usePathname, useSearchParams } from "next/navigation";
import styles from "./tab-nav.module.css";

const tabs = [
  { label: "Overview", href: "/" },
  { label: "Rules", href: "/rules" },
  { label: "Console", href: "/console" },
];

export function TabNav() {
  const pathname = usePathname();
  const searchParams = useSearchParams();
  const q = searchParams.toString();

  return (
    <div className={styles.tabNav}>
      {tabs.map((t) => {
        const isActive =
          t.href === "/"
            ? pathname === "/"
            : pathname.startsWith(t.href);
        const href = q ? `${t.href}?${q}` : t.href;
        return (
          <Link
            key={t.href}
            href={href}
            className={`${styles.tab} ${isActive ? styles.tabActive : ""}`}
          >
            {t.label}
          </Link>
        );
      })}
    </div>
  );
}
