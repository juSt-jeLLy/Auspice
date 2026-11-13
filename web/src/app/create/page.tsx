"use client";

import { useState } from "react";
import { factoryId } from "@/lib/near";
import { useWallet } from "@/components/wallet-provider";

const ONE_DAY_NS = 86_400_000_000_000;
const hexToBase64 = (value: string) => btoa(value.replace(/\s/g, "").match(/.{1,2}/g)?.map((part) => String.fromCharCode(Number.parseInt(part, 16))).join("") ?? "");

export default function CreatePage() {
  const { accountId, call } = useWallet();
  const [name, setName] = useState("Ship Signal");
  const [symbol, setSymbol] = useState("SHIP");
  const [repository, setRepository] = useState(process.env.NEXT_PUBLIC_DEMO_GITHUB_REPOSITORY ?? "juSt-jeLLy/Auspice");
  const [requiredTag, setRequiredTag] = useState(process.env.NEXT_PUBLIC_DEMO_REQUIRED_TAG ?? "v1.0.0");
  const [humanPassKey, setHumanPassKey] = useState(process.env.NEXT_PUBLIC_HUMAN_PASS_PUBLIC_KEY ?? "");
  const [wasmUrl, setWasmUrl] = useState(process.env.NEXT_PUBLIC_VERIFIER_WASM_URL ?? "");
  const [wasmHash, setWasmHash] = useState(process.env.NEXT_PUBLIC_VERIFIER_WASM_HASH ?? "");
  const [notice, setNotice] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const submit = async (event: React.FormEvent) => {
    event.preventDefault();
    if (!accountId) return setNotice("Connect the creator wallet first.");
    const parts = repository.trim().split("/");
    if (parts.length !== 2 || parts.some((part) => !part)) return setNotice("Use GitHub repository format owner/repository.");
    setBusy(true); setNotice(null);
    try {
      const now = Date.now() * 1_000_000;
      const normalized = symbol.toUpperCase();
      const sourceId = `github_release:${repository.trim()}`;
      const sourceUrl = `https://api.github.com/repos/${repository.trim()}/releases/latest`;
      const humanVerifierHash = process.env.NEXT_PUBLIC_HUMAN_VERIFIER_WASM_HASH ?? "";
      if (!humanVerifierHash || !process.env.NEXT_PUBLIC_HUMAN_VERIFIER_WASM_URL || !process.env.NEXT_PUBLIC_HUMAN_VERIFIER_PROJECT_ID) throw new Error("The V2 human verifier is not configured.");
      const config = {
        creator_id: accountId, maximum_mintable_supply: "1000000", genesis_reserve: "700000", genesis_user_cap: "10000", genesis_price_yocto_near_per_token: "1",
        genesis_open_at_ns: now + 3_600_000_000_000, genesis_close_at_ns: now + ONE_DAY_NS, human_pass_public_key: humanPassKey,
        human_verifier: { wasm_url: process.env.NEXT_PUBLIC_HUMAN_VERIFIER_WASM_URL, wasm_hash: hexToBase64(humanVerifierHash), wasm_build_target: "wasm32-wasip2", secrets_profile: "production", secrets_account_id: "busymuseum5448.testnet", expected_hostname: new URL(process.env.NEXT_PUBLIC_SITE_URL ?? "https://web-tau-tan-79.vercel.app").host, genesis_action: "auspice_genesis", signal_action: "auspice_signal", approval_lifetime_ns: 600_000_000_000 },
        outlayer_account_id: "outlayer.testnet", max_signal_age_ns: 900_000_000_000, no_creator_genesis_allocation: true,
        doors: [{ door_id: 0, template: "GithubRelease", source_id: sourceId, source_url: sourceUrl,
          predicate: { version: 1, required_tag: requiredTag || null, minimum_magnitude_x10: null, expected_winner: null, required_text: null },
          verifier_wasm_url: wasmUrl, verifier_wasm_hash: wasmHash, verifier_build_target: "wasm32-wasip2", reserve: "300000", user_cap: "5000", claim_window_ns: ONE_DAY_NS, new_humans_only: true }], version: 1,
      };
      await call(factoryId, "create_coin", { metadata: { spec: "ft-1.0.0", name, symbol: normalized, decimals: 0 }, config }, "300000000000000000000000");
      setNotice("Transaction submitted. The keyless coin is created only if every immutable rule validates on-chain.");
    } catch (cause) { setNotice(cause instanceof Error ? cause.message : "Transaction failed."); } finally { setBusy(false); }
  };

  return <main className="create section"><div className="section-kicker">Permissionless factory</div><h1>Launch a rule,<br />not a promise.</h1><p className="intro">Auspice freezes 70% for human-gated Genesis and 30% for a real-world Signal Door. This first production template opens only when a pinned GitHub-release verifier confirms your exact immutable rule.</p><form onSubmit={submit}>
    <label>Coin name<input value={name} onChange={(event) => setName(event.target.value)} maxLength={64} required /></label><label>Symbol<input value={symbol} onChange={(event) => setSymbol(event.target.value.toUpperCase())} pattern="[A-Z0-9]{2,12}" required /></label><label>Signal template<input value="GitHub release (v1)" disabled /></label><label>Repository<input value={repository} onChange={(event) => setRepository(event.target.value)} placeholder="owner/repository" required /></label><label>Required release tag <small>(optional)</small><input value={requiredTag} onChange={(event) => setRequiredTag(event.target.value)} placeholder="v1.0.0" /></label><label>Human-pass public key (Base64, 32 bytes)<input value={humanPassKey} onChange={(event) => setHumanPassKey(event.target.value)} required /></label><label>Verifier WASM URL (published FastFS URL)<input value={wasmUrl} onChange={(event) => setWasmUrl(event.target.value)} placeholder="https://..." type="url" required /></label><label>Verifier WASM SHA-256 (Base64, 32 bytes)<input value={wasmHash} onChange={(event) => setWasmHash(event.target.value)} required /></label><div className="warning"><b>Immutable at launch</b><p>No later action can edit the source, tag, verifier URL/hash, allocation reserve, deadlines, or human policy. Other source templates are deliberately unavailable until their verifier adapter is audited and published.</p></div><button className="button yellow" disabled={busy}>{busy ? "Waiting for wallet…" : accountId ? "Publish immutable coin" : "Connect wallet to publish"}</button>{notice && <p className="notice">{notice}</p>}
  </form></main>;
}
