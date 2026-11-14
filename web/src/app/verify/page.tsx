import Link from "next/link";

const checks = [
  ["1", "Birth Certificate", "Inspect reserves, dates, vesting, Fortress limits, Work-to-Mint difficulty and cap, circulation decay, TEE policy, and Signal verifier hashes."],
  ["2", "Mechanism state", "Compare minted work reserve, per-wallet claims, pending lazy decay, current fee, vesting, Doors, Streak, and commitments against the frozen config."],
  ["3", "Probe receipts", "Preview transfer protection, then inspect durable allowed or blocked receipts with sender, receiver, amount, reason, burn fee, and timestamp."],
  ["4", "Supply invariant", "Confirm minted plus permanently unmintable supply never exceeds the immutable maximum after claims, decay, fees, or failed mechanics."],
];

export default function VerifyPage() {
  return <main className="verify section"><div className="section-kicker">Independent verification</div><h1>Read every rule before you trust the launch.</h1><p>Auspice exposes immutable launch configuration, mechanism state, settlement projections, public probes, and event history through contract views. The frontend reads and composes transactions; it cannot alter a template after launch.</p><div className="verify-grid" style={{ alignItems: "start" }}>{checks.map(([number, title, description]) => <article key={number} style={{ minHeight: 0, padding: 26, alignSelf: "start" }}><b>{number}</b><h2>{title}</h2><p>{description}</p></article>)}</div><Link href="/coins" className="button yellow">Inspect a deployed coin</Link></main>;
}
