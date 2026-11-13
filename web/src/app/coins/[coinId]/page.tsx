import { CoinDashboard } from "@/components/coin-dashboard";
export default async function CoinPage({ params }: { params: Promise<{ coinId: string }> }) { const { coinId } = await params; return <CoinDashboard accountId={coinId} />; }
