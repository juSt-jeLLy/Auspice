"use client";

import Link from "next/link";
import { useCallback, useEffect, useState } from "react";
import { coinId, explorerAccount, formatUnits, readCoin, type CoinSnapshot } from "@/lib/near";
import { useWallet } from "./wallet-provider";
import { Turnstile } from "./turnstile";

/**
 * NEAR's Rust JSON ABI accepts u64 timestamps as JSON numbers.  A timestamp in
 * nanoseconds is larger than Number.MAX_SAFE_INTEGER, so JSON.stringify would
 * either quote it or silently alter it.  The pass is server-issued; validate
 * the one raw field and preserve its exact decimal form in the payload.
 */
function genesisAllocationArgs(pass: Record<string, unknown>, requestedAmount: string) {
  const expiresAtNs = pass.expires_at_ns;
  if (typeof expiresAtNs !== "string" || !/^(0|[1-9]\d*)$/.test(expiresAtNs)) throw new Error("Human-pass expiry is invalid.");
  const marker = "__AUSPICE_EXACT_U64__";
  const payload = JSON.stringify({ pass: { ...pass, expires_at_ns: marker }, requested_amount: requestedAmount });
  return payload.replace(`"${marker}"`, expiresAtNs);
}

export function CoinDashboard({ accountId = coinId }: { accountId?: string }) {
  const [coin, setCoin] = useState<CoinSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [amount, setAmount] = useState("100");
  const [captchaToken, setCaptchaToken] = useState<string | null>(null);
  const { accountId: walletId, call, signMessage } = useWallet();
  const refresh = useCallback(async () => { try { setError(null); setCoin(await readCoin(accountId)); } catch (cause) { setError(cause instanceof Error ? cause.message : "Could not load this coin."); } }, [accountId]);
  useEffect(() => { void refresh(); }, [refresh]);
  const requestDoor = async (doorId: number) => { setBusy(true); try { await call(accountId, "request_signal_check", { door_id: doorId }, "100000000000000000000000"); await refresh(); } catch (cause) { setError(cause instanceof Error ? cause.message : "Signal submission failed."); } finally { setBusy(false); } };
  const requestGenesis = async () => {
    if (!walletId || !coin) return;
    setBusy(true);
    try {
      const requested = BigInt(amount); const deposit = (requested * BigInt(coin.launch.genesis_price_yocto_near_per_token ?? "0")).toString();
      if (coin.launch.human_verifier) {
        if (!captchaToken) throw new Error("Complete human verification first.");
        await call(accountId, "request_human_check", { kind: "genesis", door_id: null, captcha_token: captchaToken }, "100000000000000000000000");
        await call(accountId, "storage_deposit", { account_id: walletId, registration_only: true }, "1250000000000000000000");
        await call(accountId, "request_genesis_allocation_v2", { requested_amount: amount }, deposit);
        setCaptchaToken(null); await refresh(); return;
      }
      const challengeResponse = await fetch("/api/human-pass/challenge", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ coinId: accountId, accountId: walletId, doorId: null }) });
      const challenge = await challengeResponse.json(); if (!challengeResponse.ok) throw new Error(challenge.error);
      const signed = await signMessage(challenge.message, challenge.recipient, challenge.nonce);
      const passResponse = await fetch("/api/human-pass/issue", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ challenge: challenge.challenge, signed }) });
      const pass = await passResponse.json(); if (!passResponse.ok) throw new Error(pass.error);
      await call(accountId, "storage_deposit", { account_id: walletId, registration_only: true }, "1250000000000000000000");
      await call(accountId, "request_genesis_allocation", genesisAllocationArgs(pass, amount), deposit); await refresh();
    } catch (cause) { setError(cause instanceof Error ? cause.message : "Genesis request failed."); } finally { setBusy(false); }
  };
  if (!accountId) return <section className="empty"><h1>Choose a launch to inspect.</h1><p>Browse factory launches or open a coin account URL.</p><Link href="/create" className="button yellow">Create a coin</Link></section>;
  if (error) return <section className="empty"><h1>Coin unavailable</h1><p>{error}</p><button className="button" onClick={() => void refresh()}>Try again</button></section>;
  if (!coin) return <section className="empty"><p className="eyebrow">Reading NEAR state…</p></section>;
  const decimals = coin.metadata.decimals;
  const v2 = Boolean(coin.launch.human_verifier);
  return <main className="dashboard"><section className="coin-head"><div><div className="eyebrow">Birth certificate · {coin.lifecycle}</div><h1>{coin.metadata.name} <em>/${coin.metadata.symbol}</em></h1><a href={explorerAccount(accountId)} target="_blank" rel="noreferrer">{accountId} ↗</a></div><div className="supply-big"><b>{formatUnits(coin.supply.minted_supply, decimals)}</b><span>minted of {formatUnits(coin.supply.maximum_mintable_supply, decimals)}</span></div></section><section className="stats"><div><b>{formatUnits(coin.launch.genesis_reserve, decimals)}</b><span>Genesis reserve</span></div><div><b>{formatUnits(coin.supply.remaining_mintable_supply, decimals)}</b><span>Still rule-bound</span></div><div><b>{formatUnits(coin.supply.permanently_unmintable, decimals)}</b><span>Permanently unmintable</span></div></section>{coin.lifecycle === "GenesisOpen" && <section className="genesis"><div><div className="section-kicker">Genesis is open</div><h2>Request a capped allocation.</h2><p>{v2 ? "Complete Turnstile. The pinned OutLayer TEE validates it before the contract records your allocation." : "Sign a one-use wallet-bound human pass, then approve the on-chain request."}</p></div><div><input aria-label="Requested token amount" value={amount} onChange={(event) => setAmount(event.target.value.replace(/\D/g, ""))} />{v2 && <Turnstile action="auspice_genesis" disabled={busy} onToken={setCaptchaToken} />}<button className="button yellow" disabled={!walletId || busy || !amount || (v2 && !captchaToken)} onClick={() => void requestGenesis()}>{walletId ? "Request Genesis" : "Connect wallet"}</button></div></section>}<section className="doors"><div className="section-kicker">Immutable Signal Doors</div>{coin.doors.map((door) => <article className="door" key={door.config.door_id}><div><span className={`door-state ${door.status.toLowerCase()}`}>{door.status}</span><h2>{door.config.template}: {door.config.source_id}</h2><p>{door.config.source_url}</p><p>Verifier: {door.config.verifier_build_target} · pinned hash</p></div><div className="door-data"><span>{formatUnits(door.config.reserve, decimals)} reserve</span><span>{formatUnits(door.config.user_cap, decimals)} per person</span>{door.status === "Locked" && <button className="button yellow" disabled={!walletId || busy || coin.lifecycle !== "Live"} onClick={() => void requestDoor(door.config.door_id)}>{walletId ? "Submit evaluation" : "Connect to submit"}</button>}</div></article>)}</section><section className="events"><div className="section-kicker">Submitted Signals</div>{coin.signalRequests.length ? coin.signalRequests.slice().reverse().map((request) => <p key={request.request_id}><b>Door {request.door_id + 1}: {request.status}</b><span>{request.requester_id}</span><time>{request.failure_reason ?? new Date(request.requested_at_ns / 1_000_000).toLocaleString()}</time></p>) : <p className="muted">No evaluation requests yet.</p>}</section><section className="events"><div className="section-kicker">On-chain history</div>{coin.events.length ? coin.events.slice().reverse().map((event) => <p key={event.id}><b>{event.kind.replaceAll("_", " ")}</b><span>{event.actor_id}</span><time>{new Date(event.timestamp_ns / 1_000_000).toLocaleString()}</time></p>) : <p className="muted">No receipts yet.</p>}</section></main>;
}
