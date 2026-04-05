import { FunctionDetail } from "@/components/FunctionDetail";

export default async function FunctionPage({
  params,
}: {
  params: Promise<{ owner: string; repo: string }>;
}) {
  const { owner, repo } = await params;
  return <FunctionDetail owner={owner} repo={repo} />;
}
