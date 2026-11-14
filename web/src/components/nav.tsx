"use client";

import Link from "next/link";
import { useWallet } from "./wallet-provider";

export function Nav() {
  const { accountId, accounts, ready, connect, disconnect, switchAccount } = useWallet();
  return <header className="site-nav"><Link href="/" className="brand"><span>A</span> Auspice</Link><nav><Link href="/coins">Coins</Link><Link href="/create">Create</Link><Link href="/arena">Arena</Link><Link href="/verify">Verify</Link></nav>{accountId ? <div className="wallet-cluster">{accounts.length > 1 && <select aria-label="Active NEAR account" value={accountId} onChange={(event) => switchAccount(event.target.value)}>{accounts.map((item) => <option key={item} value={item}>{item}</option>)}</select>}<button className="wallet" onClick={disconnect}>{accountId.length > 18 ? `${accountId.slice(0, 8)}…${accountId.slice(-6)}` : accountId}</button></div> : <button className="wallet" disabled={!ready} onClick={connect}>{ready ? "Connect wallet" : "Loading wallet"}</button>}</header>;
}
