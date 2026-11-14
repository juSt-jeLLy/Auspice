"use client";

import Link from "next/link";
import { useCallback, useEffect, useState } from "react";
import { explorerAccount, formatUnits, listFactoryCoins, readCoin, type CoinSnapshot, type FactoryCoin } from "@/lib/near";

type ListedCoin = FactoryCoin & { snapshot?: CoinSnapshot; unavailable?: string };

function CoinCard({ coin }: { coin: ListedCoin }) {
  const snapshot = coin.snapshot;
  const mechanics = snapshot ? [snapshot.curve ? "Locked Market" : null, snapshot.work ? "Work-to-Mint" : null, snapshot.protection?.config.max_wallet ? "Fortress" : snapshot.protection ? "Normal" : null, snapshot.streak ? "Daily Streak" : null, snapshot.commitments.length ? "Proofbound" : null, snapshot.doors.length ? "Signal Door" : null, snapshot.launch.circulation ? "Circulation decay" : null].filter(Boolean) : [];
  return <article className="launch-card"><div className="launch-card-header"><span className="launch-symbol">${coin.symbol}</span><span className="launch-status">{snapshot?.lifecycle ?? "Reading"}</span></div><h2>{coin.name}</h2><p className="launch-meta">{coin.account_id}</p>{snapshot ? <><div className="mechanic-chips">{mechanics.map((mechanic) => <span key={String(mechanic)}>{mechanic}</span>)}</div><p className="launch-meta">{formatUnits(snapshot.supply.minted_supply, snapshot.metadata.decimals)} minted · Created by {coin.creator_id}</p>{snapshot.work ? <p className="launch-meta">{formatUnits(snapshot.work.minted, snapshot.metadata.decimals)} / {formatUnits(snapshot.work.reserve, snapshot.metadata.decimals)} work reserve mined</p> : null}{snapshot.curve ? <p className="launch-meta">{Number(snapshot.curve.locked_near) / 1e24} NEAR in the immutable curve</p> : null}</> : <p className="notice">{coin.unavailable ?? "Loading contract state…"}</p>}<div className="launch-actions"><Link className="button yellow" href={"/coins/" + coin.account_id}>Open launch</Link><a className="button" href={explorerAccount(coin.account_id)} target="_blank" rel="noreferrer">Explorer ↗</a></div></article>;
}

export function LaunchExplorer() {
  const [coins, setCoins] = useState<ListedCoin[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      setError(null);
      const entries = await listFactoryCoins(0, 100);
      const enriched = await Promise.all(entries.map(async (entry): Promise<ListedCoin> => {
        try { const snapshot = await readCoin(entry.account_id); return snapshot ? { ...entry, snapshot } : { ...entry, unavailable: "Coin returned no readable state." }; }
        catch (cause) { return { ...entry, unavailable: cause instanceof Error ? cause.message : "Could not read coin state." }; }
      }));
      setCoins(enriched);
    } catch (cause) { setError(cause instanceof Error ? cause.message : "Could not read factory launches."); }
    finally { setLoading(false); }
  }, []);
  useEffect(() => { void refresh(); }, [refresh]);
  return <main className="section"><section className="launch-hero"><div><div className="section-kicker">Permissionless factory</div><h1>Every launch<br />has receipts.</h1></div><p className="intro">Browse every registered coin, its frozen supply rule, and the state the contract exposes—without trusting a creator dashboard.</p></section><div className="launch-controls"><p className="launch-summary"><b>{coins.length}</b> launched on this factory · testnet</p><div className="actions"><Link className="button yellow" href="/create">Launch a coin</Link><button className="button" disabled={loading} onClick={() => void refresh()}>{loading ? "Reading NEAR…" : "Refresh"}</button></div></div>{error ? <p className="notice">{error}</p> : <div className="launch-grid">{coins.length ? coins.map((coin) => <CoinCard coin={coin} key={coin.account_id} />) : !loading ? <p className="muted">No coins have been registered by this factory yet.</p> : null}</div>}</main>;
}
