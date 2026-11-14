"use client";

import Link from "next/link";
import { useCallback, useEffect, useState } from "react";
import { explorerAccount, formatUnits, listFactoryCoins, readCoin, type CoinSnapshot, type FactoryCoin } from "@/lib/near";

type ListedCoin = FactoryCoin & { snapshot?: CoinSnapshot; unavailable?: string };

function CoinRow({ coin }: { coin: ListedCoin }) {
  const snapshot = coin.snapshot;
  const openDoors = snapshot?.doors.filter((door) => door.status === "Open").length ?? 0;
  const mechanics = snapshot ? [snapshot.curve ? "Locked Market" : null, snapshot.work ? "Work-to-Mint" : null, snapshot.protection?.config.max_wallet ? "Fortress" : snapshot.protection ? "Normal" : null, snapshot.streak ? "Daily Streak" : null, snapshot.commitments.length ? "Proofbound" : null, snapshot.doors.length ? `${snapshot.doors.length} Signal Door${snapshot.doors.length === 1 ? "" : "s"}` : null, snapshot.launch.circulation ? `${snapshot.launch.circulation.daily_decay_bps} bps/day decay` : null].filter(Boolean) : [];
  return <article className="door"><div><div className="section-kicker">${coin.symbol} · {snapshot?.lifecycle ?? "reading"}</div><h2>{coin.name}</h2><p>{coin.account_id}</p><p>Creator: {coin.creator_id}</p>{snapshot ? <><p>{mechanics.join(" · ")} · {formatUnits(snapshot.supply.minted_supply, snapshot.metadata.decimals)} minted</p>{snapshot.work ? <p>{formatUnits(snapshot.work.minted, snapshot.metadata.decimals)} / {formatUnits(snapshot.work.reserve, snapshot.metadata.decimals)} work reserve mined at {snapshot.work.difficulty_bits} bits</p> : null}{snapshot.curve ? <p>{Number(snapshot.curve.locked_near) / 1e24} NEAR locked in the immutable curve</p> : null}{snapshot.protectionProbes.length ? <p>{snapshot.protectionProbes.length} public protection probe receipt{snapshot.protectionProbes.length === 1 ? "" : "s"}</p> : null}</> : <p className="notice">{coin.unavailable ?? "Loading contract state…"}</p>}</div><div className="door-data">{snapshot?.streak ? <span>Streak: {snapshot.streak.state.current_streak} days</span> : snapshot?.commitments.length ? <span>Public commitments: {snapshot.commitments.length}</span> : snapshot?.doors.map((door) => <span key={door.config.door_id}>{door.config.template}: {door.status}</span>)}{snapshot && openDoors > 0 ? <span>{openDoors} signal door{openDoors === 1 ? "" : "s"} open</span> : null}<Link className="button yellow" href={`/coins/${coin.account_id}`}>Open launch</Link><a className="button" href={explorerAccount(coin.account_id)} target="_blank" rel="noreferrer">NEAR Explorer</a></div></article>;
}

export function LaunchExplorer() {
  const [coins, setCoins] = useState<ListedCoin[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const refresh = useCallback(async () => { setLoading(true); try { setError(null); const entries = await listFactoryCoins(0, 100); const enriched: ListedCoin[] = await Promise.all(entries.map(async (entry): Promise<ListedCoin> => { try { const snapshot = await readCoin(entry.account_id); return snapshot ? { ...entry, snapshot } : { ...entry, unavailable: "Coin returned no readable state." }; } catch (cause) { return { ...entry, unavailable: cause instanceof Error ? cause.message : "Could not read coin state." }; } })); setCoins(enriched); } catch (cause) { setError(cause instanceof Error ? cause.message : "Could not read factory launches."); } finally { setLoading(false); } }, []);
  useEffect(() => { void refresh(); }, [refresh]);
  return <main className="section"><div className="section-kicker">Permissionless launches</div><h1>Explore immutable coins.</h1><p className="intro">The explorer reads every registered factory coin in parallel, including Normal and Fortress protection, Work-to-Mint progress, circulation decay, public probes, Signal Doors, Streak rituals, and Proofbound commitments. It has no authority to alter a launch.</p><div className="actions"><Link className="button yellow" href="/create">Launch a coin</Link><button className="button" disabled={loading} onClick={() => void refresh()}>{loading ? "Reading NEAR…" : "Refresh all"}</button></div>{error ? <p className="notice">{error}</p> : <div className="doors">{coins.length ? coins.map((coin) => <CoinRow coin={coin} key={coin.account_id} />) : !loading ? <p className="muted">No coins have been registered by this factory yet.</p> : null}</div>}</main>;
}
