import Link from "next/link";

const principles = [
  ["Birth", "A capped, wallet-bound Genesis allocation opens to verified participants on equal terms."],
  ["Signal", "A public weather rule is fixed in the coin’s Birth Certificate before anyone can participate."],
  ["Door", "When an attested signal passes, a capped human allocation opens. Unclaimed supply becomes unmintable."],
];

export default function Home() {
  return (
    <main className="min-h-screen overflow-hidden">
      <header className="mx-auto flex max-w-6xl items-center justify-between px-6 py-6">
        <Link href="/" className="flex items-center gap-3 font-semibold tracking-[0.22em] uppercase">
          <span className="grid h-9 w-9 place-items-center rounded-full border border-[var(--signal)] text-sm text-[var(--signal)]">A</span>
          Auspice
        </Link>
        <span className="rounded-full border border-[var(--line)] px-3 py-1.5 font-mono text-[11px] tracking-wide text-[var(--rain)]">NEAR TESTNET</span>
      </header>

      <section className="rain-grid border-y border-[var(--line)]">
        <div className="mx-auto max-w-6xl px-6 py-24 sm:py-32">
          <p className="font-mono text-xs tracking-[0.18em] text-[var(--signal)] uppercase">Raincheck / $RAIN</p>
          <h1 className="mt-5 max-w-4xl text-6xl font-semibold leading-[0.93] tracking-[-0.06em] sm:text-8xl">
            Supply follows reality,<br />not insiders.
          </h1>
          <p className="mt-8 max-w-2xl text-lg leading-8 text-[var(--paper)]/75">
            Auspice gives a memecoin an equal-access birth and immutable rules for every future release of supply.
          </p>
          <div className="mt-10 flex flex-wrap gap-3">
            <button disabled className="rounded-full bg-[var(--signal)] px-5 py-3 text-sm font-semibold text-[var(--ink)] opacity-60">Genesis opens soon</button>
            <a href="#how-it-works" className="rounded-full border border-[var(--line)] px-5 py-3 text-sm font-semibold hover:border-[var(--rain)]">Inspect the mechanism</a>
          </div>
        </div>
      </section>

      <section id="how-it-works" className="mx-auto max-w-6xl px-6 py-24">
        <p className="font-mono text-xs tracking-[0.18em] text-[var(--rain)] uppercase">How it works</p>
        <div className="mt-8 grid gap-px overflow-hidden rounded-2xl border border-[var(--line)] bg-[var(--line)] md:grid-cols-3">
          {principles.map(([title, body], index) => (
            <article key={title} className="min-h-64 bg-[var(--ink)] p-6">
              <span className="font-mono text-xs text-[var(--signal)]">0{index + 1}</span>
              <h2 className="mt-12 text-3xl font-semibold tracking-[-0.04em]">{title}</h2>
              <p className="mt-4 max-w-xs leading-7 text-[var(--paper)]/65">{body}</p>
            </article>
          ))}
        </div>
      </section>

      <section className="mx-auto max-w-6xl px-6 pb-24">
        <div className="rounded-2xl border border-[var(--line)] p-7 sm:p-10">
          <p className="font-mono text-xs tracking-[0.18em] text-[var(--rain)] uppercase">First Signal Door</p>
          <h2 className="mt-4 text-3xl font-semibold tracking-[-0.04em]">Rain in Mumbai on Thursday.</h2>
          <p className="mt-4 max-w-2xl leading-7 text-[var(--paper)]/65">The final interface will show the immutable predicate, proof hash, allocation cap, countdown, and every on-chain receipt.</p>
        </div>
      </section>
    </main>
  );
}
