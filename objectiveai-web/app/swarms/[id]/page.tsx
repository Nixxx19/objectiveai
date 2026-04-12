import { SwarmDetail } from "@/components/SwarmDetail";

export default async function SwarmDetailPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  return <SwarmDetail id={id} />;
}
