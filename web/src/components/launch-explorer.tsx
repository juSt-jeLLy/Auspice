"use client";

import Link from "next/link";
import { useCallback, useEffect, useState } from "react";
import { explorerAccount, listFactoryCoins, type FactoryCoin } from "@/lib/near";

export function LaunchExplorer() {
  const [coins, setCoins] = useState<FactoryCoin[]>([]);
  const [error, setError] = useState<string | null>(null);
  const refresh = useCallback(async () => { try { setError(null); setCoins(await listFactoryCoins()); } catch (cause) { setError(cause instanceof Error ? cause.message : "Could not read factory launches."); } }, []);
  useEffect(() => { void refresh(); }, [refresh]);
  return <main className="section"><div className="section-kicker">Permissionless launches</div><h1>Explore immutable coins.</h1><p className="intro">Every entry was created by the Auspice factory. Its Genesis and Signal Door rules live in the coin contract, not in this interface.</p><div className="actions"><Link className="button yellow" href="/create">Launch a coin</Link><button className="button" onClick={() => void refresh()}>Refresh</button></div>{error ? <p className="notice">{error}</p> : <div className="doors">{coins.length ? coins.map((coin) => <article className="door" key={coin.account_id}><div><div className="section-kicker">${coin.symbol}</div><h2>{coin.name}</h2><p>{coin.account_id}</p><p>Creator: {coin.creator_id}</p></div><div className="door-data"><Link className="button yellow" href={`/coins/${coin.account_id}`}>Inspect launch</Link><a className="button" href={explorerAccount(coin.account_id)} target="_blank" rel="noreferrer">NEAR Explorer</a></div></article>) : <p className="muted">No coins have been registered yet.</p>}</div>}</main>;
}
