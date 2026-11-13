"use client";

import { useState } from "react";
import { factoryId } from "@/lib/near";
import { useWallet } from "@/components/wallet-provider";

export default function CreatePage() {
  const { accountId, call } = useWallet();
  const [name, setName] = useState("Raincheck");
  const [symbol, setSymbol] = useState("RAIN");
  const [humanPassKey, setHumanPassKey] = useState("");
  const [verifierHash, setVerifierHash] = useState("");
  const [outlayerProject, setOutlayerProject] = useState("");
  const [notice, setNotice] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    if (!accountId) return setNotice("Connect the creator wallet first.");
    setBusy(true); setNotice(null);
    try {
      const now = Date.now() * 1_000_000;
      const normalized = symbol.toUpperCase();
      const config = { creator_id: accountId, maximum_mintable_supply: "1000000", genesis_reserve: "600000", genesis_user_cap: "1000", genesis_price_yocto_near_per_token: "1", genesis_open_at_ns: now + 3_600_000_000_000, genesis_close_at_ns: now + 86_400_000_000_000, human_pass_public_key: humanPassKey, tee_wasm_hash: verifierHash, outlayer_account_id: "outlayer.testnet", outlayer_project_id: outlayerProject, max_signal_age_ns: 900_000_000_000, no_creator_genesis_allocation: true, doors: [{ door_id: 0, source_id: "open_meteo_mumbai", source_url: "https://api.open-meteo.com/v1/forecast", reserve: "400000", user_cap: "1000", claim_window_ns: 86_400_000_000_000, new_humans_only: true, predicate_version: 1 }], version: 1 };
      await call(factoryId, "create_coin", { metadata: { spec: "ft-1.0.0", name, symbol: normalized, decimals: 0 }, config }, "300000000000000000000000");
      setNotice("Transaction submitted. The factory will create the keyless coin account if immutable configuration checks pass.");
    } catch (cause) { setNotice(cause instanceof Error ? cause.message : "Transaction failed."); } finally { setBusy(false); }
  };
  return <main className="create section"><div className="section-kicker">Permissionless factory</div><h1>Publish rules,<br />not promises.</h1><p className="intro">Every field below becomes immutable in the new coin. A valid 32-byte Base64 human-pass public key, verifier WASM hash, and OutLayer project are required before the contract can be initialized.</p><form onSubmit={submit}><label>Coin name<input value={name} onChange={(event) => setName(event.target.value)} maxLength={64} required /></label><label>Symbol<input value={symbol} onChange={(event) => setSymbol(event.target.value.toUpperCase())} pattern="[A-Z0-9]{2,12}" required /></label><label>Human-pass public key (Base64, 32 bytes)<input value={humanPassKey} onChange={(event) => setHumanPassKey(event.target.value)} required /></label><label>Verifier WASM hash (Base64, 32 bytes)<input value={verifierHash} onChange={(event) => setVerifierHash(event.target.value)} required /></label><label>OutLayer project ID<input value={outlayerProject} onChange={(event) => setOutlayerProject(event.target.value)} required /></label><div className="warning"><b>Immutable at launch</b><p>The creator cannot edit dates, reserves, source policy, verification hash, or human-pass policy after the coin exists. Test on testnet before publishing a public configuration.</p></div><button className="button yellow" disabled={busy}>{busy ? "Waiting for wallet…" : accountId ? "Publish immutable coin" : "Connect wallet to publish"}</button>{notice && <p className="notice">{notice}</p>}</form></main>;
}
