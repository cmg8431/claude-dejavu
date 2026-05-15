"use client";

import { useState, useRef, useEffect } from "react";
import { useRouter, useSearchParams } from "next/navigation";
import styles from "./project-selector.module.css";

interface ProjectSelectorProps {
  projects: string[];
}

function shortenPath(p: string): string {
  const parts = p.split("/");
  return parts.length > 2 ? parts.slice(-2).join("/") : p;
}

export function ProjectSelector({ projects }: ProjectSelectorProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const router = useRouter();
  const searchParams = useSearchParams();
  const current = searchParams.get("project") || "";

  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, []);

  function select(project: string) {
    const params = new URLSearchParams(searchParams.toString());
    if (project) {
      params.set("project", project);
    } else {
      params.delete("project");
    }
    const q = params.toString();
    router.push(q ? `?${q}` : window.location.pathname);
    setOpen(false);
  }

  return (
    <div className={styles.wrapper} ref={ref}>
      <button className={styles.button} onClick={() => setOpen(!open)}>
        <span className={styles.label}>
          {current ? shortenPath(current) : "All projects"}
        </span>
        <span className={styles.chevron}>&#9662;</span>
      </button>
      {open && (
        <div className={styles.dropdown}>
          <button
            className={`${styles.option} ${!current ? styles.optionActive : ""}`}
            onClick={() => select("")}
          >
            All projects
          </button>
          {projects.map((p) => (
            <button
              key={p}
              className={`${styles.option} ${current === p ? styles.optionActive : ""}`}
              onClick={() => select(p)}
            >
              {shortenPath(p)}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
