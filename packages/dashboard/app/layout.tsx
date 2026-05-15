import type { Metadata } from "next";
import { Suspense } from "react";
import "./globals.css";
import { Navbar } from "../components/navbar";
import { TabNav } from "../components/tab-nav";
import { getProjects } from "../lib/db";

export const metadata: Metadata = {
  title: "claude-dejavu",
  description: "Pattern detection dashboard for Claude Code",
};

export const dynamic = "force-dynamic";

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const projects = getProjects();

  return (
    <html lang="en">
      <body>
        <Suspense>
          <Navbar projects={projects} />
          <TabNav />
        </Suspense>
        <main
          style={{
            paddingTop: "calc(var(--navbar-height) + var(--tab-height) + 24px)",
            paddingBottom: "48px",
            maxWidth: "720px",
            margin: "0 auto",
            paddingLeft: "16px",
            paddingRight: "16px",
          }}
        >
          {children}
        </main>
      </body>
    </html>
  );
}
