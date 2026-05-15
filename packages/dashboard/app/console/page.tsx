import { LogViewer } from "../../components/log-viewer";
import { getLogEntries } from "../../lib/db";

export const dynamic = "force-dynamic";

interface PageProps {
  searchParams: Promise<{ project?: string }>;
}

export default async function ConsolePage({ searchParams }: PageProps) {
  const params = await searchParams;
  const project = params.project || undefined;
  const entries = getLogEntries(project);

  return <LogViewer entries={entries} />;
}
