import { expect, test, type Page } from "@playwright/test";

test("create exposes presets, protection, work, locked market, and optional decay configuration", async ({ page }) => {
  await page.goto("/create");
  const template = page.getByLabel("Launch template");
  await expect(template).toHaveValue("Normal");
  await expect(template.locator("option")).toHaveCount(8);
  await expect(page.getByLabel("Creator vesting cliff (days after Genesis)")).toBeVisible();

  await template.selectOption("Fortress");
  await expect(page.getByLabel("Maximum wallet (% supply)")).toBeVisible();
  await expect(page.getByLabel("Fee reduction per interval (bps)")).toBeVisible();

  await template.selectOption("WorkToMint");
  await expect(page.getByLabel("Proof difficulty (leading zero bits, 4–24)")).toHaveValue("16");
  await expect(page.getByLabel("Mint cap per wallet")).toBeVisible();

  await template.selectOption("BondingCurve");
  await expect(page.getByLabel("Virtual NEAR reserve")).toHaveValue("1");
  await expect(page.getByLabel("Maximum buy (NEAR)")).toBeVisible();
  await expect(page.getByLabel("Maximum sell (tokens)")).toHaveValue("1000");

  await page.getByLabel("Enable immutable circulation decay").check();
  await expect(page.getByLabel("Daily decay (basis points, max 1000)")).toHaveValue("100");
  await expect(page.getByLabel("Decay starts (days after Genesis)")).toBeVisible();
});

async function mockCoinRpc(page: Page) {
  const values: Record<string, unknown> = {
    ft_metadata: { spec: "ft-1.0.0", name: "Module Coin", symbol: "MOD", decimals: 0 },
    get_launch_config: { creator_id: "creator.testnet", genesis_open_at_ns: 1, genesis_close_at_ns: 2, maximum_mintable_supply: "1000000", genesis_reserve: "700000", genesis_price_yocto_near_per_token: "1", human_verifier: { expected_hostname: "localhost", genesis_action: "auspice_genesis", signal_action: "auspice_signal" }, work_mint: { reserve: "300000", difficulty_bits: 16, user_cap: "1000" }, circulation: { daily_decay_bps: 100, starts_at_ns: 2 } },
    get_lifecycle: "Live",
    list_doors: [],
    get_supply_status: { maximum_mintable_supply: "1000000", minted_supply: "701000", permanently_unmintable: "100", remaining_mintable_supply: "298900" },
    events: [{ id: 0, kind: "protection_probe_blocked", actor_id: "alice.testnet", door_id: null, amount: "5000", timestamp_ns: 1_700_000_000_000_000_000 }],
    list_signal_requests: [],
    get_streak_status: null,
    list_commitments: [],
    get_protection_status: { config: { creator_vesting: { reserve: "300000", cliff_at_ns: 2, ends_at_ns: 3 }, max_wallet: "20000", max_transfer: "10000", transfer_cooldown_ns: 60_000_000_000, initial_burn_fee_bps: 300, fee_decay_start_ns: 4, fee_decay_interval_ns: 5, fee_decay_bps: 100 }, creator_claimed: "0", currently_vested: "150000", current_burn_fee_bps: 200 },
    get_work_mint_status: { reserve: "300000", minted: "1000", difficulty_bits: 16, user_cap: "1000" },
    get_bonding_curve_status: null,
    list_protection_probes: [{ probe_id: 0, sender_id: "alice.testnet", receiver_id: "bob.testnet", amount: "5000", allowed: false, reason: "TRANSFER_COOLDOWN", burn_fee: "0", timestamp_ns: 1_700_000_000_000_000_000 }],
  };
  await page.route("**/*", async (route) => {
    const request = route.request();
    if (request.method() === "OPTIONS") return route.fulfill({ status: 204, headers: { "access-control-allow-origin": "*", "access-control-allow-methods": "POST, OPTIONS", "access-control-allow-headers": "content-type" } });
    if (request.method() !== "POST") return route.continue();
    let body: { method?: string; params?: { method_name?: string } };
    try { body = request.postDataJSON(); } catch { return route.continue(); }
    if (body.method !== "query" || !body.params?.method_name) return route.continue();
    const value = values[body.params.method_name];
    const bytes = Array.from(Buffer.from(JSON.stringify(value)));
    await route.fulfill({ status: 200, contentType: "application/json", headers: { "access-control-allow-origin": "*" }, body: JSON.stringify({ jsonrpc: "2.0", id: "auspice", result: { result: bytes } }) });
  });
}

test("coin dashboard renders mining, decay, protection preflight, and public probe audit", async ({ page }) => {
  await mockCoinRpc(page);
  await page.goto("/coins/module.auspice.testnet");
  await expect(page.getByRole("heading", { name: /Module Coin/ })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Mine a wallet-bound nonce." })).toBeVisible();
  await expect(page.getByText("16 leading zero bits")).toBeVisible();
  await expect(page.getByRole("button", { name: "Connect wallet to mine" })).toBeDisabled();
  await expect(page.getByRole("heading", { name: "100 bps settles each elapsed day." })).toBeVisible();
  await expect(page.getByText("Connect a wallet to read its exact pending lazy settlement from the contract.")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Preflight the immutable rules." })).toBeVisible();
  await expect(page.getByRole("button", { name: "Preview rules" })).toBeDisabled();
  await expect(page.getByText("Blocked · TRANSFER COOLDOWN")).toBeVisible();
  await expect(page.getByText(/alice.testnet → bob.testnet/)).toBeVisible();
});
