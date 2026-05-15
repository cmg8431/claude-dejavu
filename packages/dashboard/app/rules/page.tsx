import { RulesTable } from "../../components/rules-table";
import { getAllRules } from "../../lib/db";

export const dynamic = "force-dynamic";

interface PageProps {
  searchParams: Promise<{ project?: string }>;
}

export default async function RulesPage({ searchParams }: PageProps) {
  const params = await searchParams;
  const project = params.project || undefined;
  const rules = getAllRules(project);

  return <RulesTable rules={rules} />;
}
