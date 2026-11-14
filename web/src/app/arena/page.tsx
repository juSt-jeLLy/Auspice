"use client";

import Link from "next/link";
import { useCallback, useEffect, useState, type ReactNode } from "react";
import { formatUnits, readCoin, type CoinSnapshot } from "@/lib/near";
import { useWallet } from "@/components/wallet-provider";

const demoCoinId = process.env.NEXT_PUBLIC_ARENA_COIN_ID ?? "";

function ChallengeCard({ label, title, children, action }: { label: string; title: string; children: ReactNode; action: ReactNode }) {
  return (
    <article style={{ minHeight: 0, padding: 26, alignSelf: "start", display: "grid", alignContent: "start", gap: 12 }}>
      <div className="section-kicker">{label}</div>
      <h2 style={{ margin: 0 }}>{title}</h2>
      <div>{children}</div>
      <div style={{ paddingTop: 4 }}>{action}</div>
    </article>
  );
}

export default function ArenaPage() {
  const { accountId, call } = useWallet();
  const [coin, setCoin] = useState<CoinSnapshot | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    if (!demoCoinId) return;
    try {
      setCoin(await readCoin(demoCoinId, accountId));
    } catch (cause) {
      setNotice(cause instanceof Error ? cause.message : "Could not load the demo coin.");
    }
  }, [accountId]);

  useEffect(() => { void refresh(); }, [refresh]);

  const act = async (work: () => Promise<void>, success: string) => {
    setBusy(true);
    setNotice(null);
    try {
      await work();
      setNotice(success);
      await refresh();
    } catch (cause) {
      setNotice(cause instanceof Error ? cause.message : "The real transaction was rejected.");
    } finally {
      setBusy(false);
    }
  };

  if (!demoCoinId) {
    return <main className="section"><div className="section-kicker">Demo Arena</div><h1>Arm a public<br />Fortress demo.</h1><p className="intro">Create a Fortress launch after deployment, then set <code>NEXT_PUBLIC_ARENA_COIN_ID</code> to its account ID. The Arena will use real wallet calls and on-chain probe receipts.</p><Link href="/create?template=Fortress&preset=fortress" className="button yellow">Create the demo configuration</Link></main>;
  }

  const protection = coin?.protection?.config;
  const overTransfer = protection?.max_transfer ? (BigInt(protection.max_transfer) + 1n).toString() : "1";
  const overWallet = protection?.max_wallet ? (BigInt(protection.max_wallet) + 1n).toString() : "1";

  return (
    <main className="section">
      <div className="section-kicker">Live testnet challenge</div>
      <h1>Try to break<br />the rules.</h1>
      <p className="intro">These actions target <code>{demoCoinId}</code>. Transfer probes create durable on-chain receipts without moving tokens; the liquidity and immutability actions deliberately submit real failing calls.</p>
      {coin ? <p className="notice">Loaded {"$"}{coin.metadata.symbol} · {coin.lifecycle} · {formatUnits(coin.supply.minted_supply, coin.metadata.decimals)} minted</p> : <p className="muted">Reading the demo coin…</p>}
      <div className="arena-grid" style={{ alignItems: "start" }}>
        <ChallengeCard label="01 / Max transfer" title="Push past the cap" action={<button className="button yellow" disabled={busy || !accountId || !coin || !protection} onClick={() => void act(() => call(demoCoinId, "probe_transfer_protection", { receiver_id: accountId, amount: overTransfer }), "Probe recorded on-chain. Open the coin audit feed to inspect it.")}>Record blocked probe</button>}><p style={{ margin: 0 }}>Record a public proof that a transfer of {overTransfer} tokens is blocked.</p></ChallengeCard>
        <ChallengeCard label="02 / Max wallet" title="Try to create a whale" action={<button className="button yellow" disabled={busy || !accountId || !coin || !protection} onClick={() => void act(() => call(demoCoinId, "probe_transfer_protection", { receiver_id: accountId, amount: overWallet }), "Wallet-cap probe recorded on-chain.")}>Record wallet probe</button>}><p style={{ margin: 0 }}>Record the result of a transfer that would make the recipient exceed the immutable wallet rule.</p></ChallengeCard>
        <ChallengeCard label="03 / Liquidity lock" title="Withdraw the reserve" action={<button className="button" disabled={busy || !accountId || !coin?.curve} onClick={() => void act(() => call(demoCoinId, "withdraw_curve_liquidity", {}), "Unexpected success — investigate immediately.")}>Attempt liquidity withdrawal</button>}><p style={{ margin: 0 }}>This submits the forbidden method. The transaction must fail with <code>AUSPICE[LIQUIDITY_LOCK]</code>.</p></ChallengeCard>
        <ChallengeCard label="04 / Immutable rules" title="Rewrite the market" action={<button className="button" disabled={busy || !accountId || !coin?.curve} onClick={() => void act(() => call(demoCoinId, "change_curve_config", {}), "Unexpected success — investigate immediately.")}>Attempt rule rewrite</button>}><p style={{ margin: 0 }}>This submits the forbidden configuration method. It must fail with <code>AUSPICE[IMMUTABLE_RULES]</code>.</p></ChallengeCard>
      </div>
      {notice ? <p className="notice" role="status">{notice}</p> : null}
      <div className="actions"><Link className="button yellow" href={"/coins/" + demoCoinId}>Open launch passport</Link><button className="button" onClick={() => void refresh()}>Refresh arena</button></div>
    </main>
  );
}
