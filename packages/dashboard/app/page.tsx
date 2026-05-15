import { StatGrid } from "../components/stat-grid";
import { FeedCard } from "../components/feed-card";
import { getOverviewStats, getFeed } from "../lib/db";

export const dynamic = "force-dynamic";

interface PageProps {
  searchParams: Promise<{ project?: string }>;
}

export default async function OverviewPage({ searchParams }: PageProps) {
  const params = await searchParams;
  const project = params.project || undefined;
  const stats = getOverviewStats(project);
  const feed = getFeed(project);

  return (
    <>
      <StatGrid stats={stats} />
      <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
        {feed.length === 0 ? (
          <div
            style={{
              textAlign: "center",
              color: "var(--text-muted)",
              padding: "48px 20px",
              fontSize: "14px",
            }}
          >
            No patterns or rules detected yet.
          </div>
        ) : (
          feed.map((item) => <FeedCard key={item.id} item={item} />)
        )}
      </div>
    </>
  );
}
