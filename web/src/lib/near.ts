export const networkId = process.env.NEXT_PUBLIC_NETWORK_ID ?? "testnet";
// All application reads travel through the same-origin relay. It keeps the
// official RPC's lack of browser CORS, and third-party free-tier limits, out
// of the user-facing dashboard and wallet setup.
export const rpcUrl = process.env.NEXT_PUBLIC_RPC_URL ?? "/api/near-rpc";
export const factoryId = process.env.NEXT_PUBLIC_FACTORY_ID ?? "auspice.testnet";
export const coinId = process.env.NEXT_PUBLIC_COIN_ID ?? "";

export type Lifecycle = "Draft" | "GenesisOpen" | "GenesisSettling" | "Live" | "DoorOpen" | "Finished";

export type SignalTemplate = "GithubRelease" | "Rainfall" | "UsgsEarthquake" | "SportsScore" | "PublicAccountPost";
export type StreakStatus = {
  config: { reserve: string; starts_at_ns: number; day_length_ns: number; daily_target: number; daily_participant_cap: number; mint_per_checkin: string; missed_day_burn: string };
  state: { next_day_index: number; current_streak: number; longest_streak: number; remaining_reserve: string; minted: string; burned: string; current_day_checkin_start: number };
  today_checkins: number;
  today_started_at_ns: number;
  today_ends_at_ns: number;
};
export type Commitment = {
  config: { commitment_id: number; statement: string; evidence_requirement: string; evidence_deadline_ns: number; vote_duration_ns: number; quorum: number; pass_bps: number; creator_reward: string; voter_reward: string };
  status: "AwaitingEvidence" | "Voting" | "Passed" | "Failed";
  evidence_url?: string | null;
  evidence_hash?: string | null;
  vote_opens_at_ns?: number | null;
  vote_closes_at_ns?: number | null;
  yes_votes: number;
  no_votes: number;
  voter_reward_per_yes_vote: string;
  creator_reward_claimed: boolean;
};
export type ProtectionStatus = { config: { creator_vesting: { reserve: string; cliff_at_ns: number; ends_at_ns: number }; max_wallet?: string | null; max_transfer?: string | null; transfer_cooldown_ns?: number | null; initial_burn_fee_bps: number; fee_decay_start_ns: number; fee_decay_interval_ns: number; fee_decay_bps: number }; creator_claimed: string; currently_vested: string; current_burn_fee_bps: number };
export type WorkMintStatus = { reserve: string; minted: string; difficulty_bits: number; user_cap: string };
export type CirculationStatus = { config: { daily_decay_bps: number; starts_at_ns: number }; account_id: string; balance: string; last_settled_at_ns: number; pending_days: number; pending_decay: string; next_settlement_at_ns: number };
export type BondingCurveStatus = { token_reserve: string; available_tokens: string; locked_near: string; virtual_near_reserve: string; max_buy: string; max_sell: string };
export type ProtectionProbe = { allowed: boolean; reason: string; burn_fee: string };
export type ProtectionProbeReceipt = ProtectionProbe & { probe_id: number; sender_id: string; receiver_id: string; amount: string; timestamp_ns: number };
export type Door = {
  config: { door_id: number; template: SignalTemplate; source_id: string; source_url: string; predicate: { version: number; required_tag?: string | null; minimum_rainfall_mm_x10?: number | null }; verifier_wasm_url: string; verifier_wasm_hash: string; verifier_build_target: string; reserve: string; user_cap: string; claim_window_ns: number; new_humans_only: boolean };
  status: "Locked" | "Open" | "ClosedClaimed" | "ClosedBurned";
  opened_at_ns?: number | null;
  closes_at_ns?: number | null;
  proof_hash?: string | null;
  minted: string;
  burned: string;
};

export type CoinSnapshot = {
  lifecycle: Lifecycle;
  metadata: { name: string; symbol: string; decimals: number };
  launch: { creator_id: string; genesis_open_at_ns: number; genesis_close_at_ns: number; maximum_mintable_supply: string; genesis_reserve: string; genesis_price_yocto_near_per_token: string; human_verifier: { expected_hostname: string; genesis_action: string; signal_action: string }; work_mint?: { reserve: string; difficulty_bits: number; user_cap: string } | null; circulation?: { daily_decay_bps: number; starts_at_ns: number } | null; bonding_curve?: { token_reserve: string; virtual_near_reserve: string; max_buy: string; max_sell: string } | null };
  doors: Door[];
  supply: { maximum_mintable_supply: string; minted_supply: string; permanently_unmintable: string; remaining_mintable_supply: string };
  events: Array<{ id: number; kind: string; actor_id: string; door_id?: number | null; amount: string; timestamp_ns: number }>;
  signalRequests: Array<{ request_id: number; door_id: number; nonce: string; requester_id: string; requested_at_ns: number; status: "Pending" | "Verified" | "Rejected"; completed_at_ns?: number | null; failure_reason?: string | null }>;
  streak?: StreakStatus | null;
  commitments: Commitment[];
  protection?: ProtectionStatus | null;
  work?: WorkMintStatus | null;
  workClaimed: string;
  circulation?: CirculationStatus | null;
  curve?: BondingCurveStatus | null;
  protectionProbes: ProtectionProbeReceipt[];
  genesis?: { requested: string; settled: boolean };
  humanApproval?: { request_id: number; kind: string; door_id?: number | null; expires_at_ns: number; max_allocation: string } | null;
};

export type FactoryCoin = { account_id: string; creator_id: string; name: string; symbol: string; created_at_ns: number };

export async function listFactoryCoins(fromIndex = 0, limit = 50): Promise<FactoryCoin[]> {
  return view<FactoryCoin[]>(factoryId, "list_coins", { from_index: fromIndex, limit });
}

async function rpc<T>(method: string, params: object): Promise<T> {
  const response = await fetch(rpcUrl, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ jsonrpc: "2.0", id: "auspice", method, params }),
    cache: "no-store",
  });
  const payload = await response.json();
  if (!response.ok || payload.error) throw new Error(payload.error?.message ?? "NEAR RPC request failed");
  return payload.result as T;
}

export async function view<T>(accountId: string, methodName: string, args: object = {}): Promise<T> {
  const result = await rpc<{ result: number[] }>("query", {
    request_type: "call_function",
    finality: "optimistic",
    account_id: accountId,
    method_name: methodName,
    args_base64: btoa(JSON.stringify(args)),
  });
  return JSON.parse(new TextDecoder().decode(new Uint8Array(result.result))) as T;
}

export async function readCoin(accountId = coinId, walletId?: string | null): Promise<CoinSnapshot | null> {
  if (!accountId) return null;
  const [metadata, launch, lifecycle, doors, supply, events, signalRequests, requested, settled, humanApproval, streak, commitments, protection, work, workClaimed, circulation, protectionProbes, curve] = await Promise.all([
    view<CoinSnapshot["metadata"]>(accountId, "ft_metadata"),
    view<CoinSnapshot["launch"]>(accountId, "get_launch_config"),
    view<Lifecycle>(accountId, "get_lifecycle"),
    view<Door[]>(accountId, "list_doors"),
    view<CoinSnapshot["supply"]>(accountId, "get_supply_status"),
    view<CoinSnapshot["events"]>(accountId, "events", { limit: 24 }),
    view<CoinSnapshot["signalRequests"]>(accountId, "list_signal_requests", { from_index: 0, limit: 50 }),
    walletId ? view<string>(accountId, "get_genesis_request", { account_id: walletId }) : Promise.resolve("0"),
    walletId ? view<boolean>(accountId, "has_settled_genesis", { account_id: walletId }) : Promise.resolve(false),
    walletId ? view<CoinSnapshot["humanApproval"]>(accountId, "get_human_approval", { account_id: walletId }) : Promise.resolve(null),
    view<StreakStatus | null>(accountId, "get_streak_status"),
    view<Commitment[]>(accountId, "list_commitments").catch(() => []),
    view<ProtectionStatus | null>(accountId, "get_protection_status").catch(() => null),
    view<WorkMintStatus | null>(accountId, "get_work_mint_status").catch(() => null),
    walletId ? view<string>(accountId, "get_work_mint_claim", { account_id: walletId }).catch(() => "0") : Promise.resolve("0"),
    walletId ? view<CirculationStatus | null>(accountId, "get_circulation_status", { account_id: walletId }).catch(() => null) : Promise.resolve(null),
    view<ProtectionProbeReceipt[]>(accountId, "list_protection_probes", { from_index: 0, limit: 100 }).catch(() => []),
    view<BondingCurveStatus | null>(accountId, "get_bonding_curve_status").catch(() => null),
  ]);
  return { metadata, launch, lifecycle, doors, supply, events, signalRequests, genesis: { requested, settled }, humanApproval, streak, commitments, protection, work, workClaimed, circulation, protectionProbes, curve };
}

export async function previewProtection(accountId: string, senderId: string, receiverId: string, amount: string) {
  return view<ProtectionProbe>(accountId, "preview_transfer_protection", { sender_id: senderId, receiver_id: receiverId, amount });
}

export const explorerAccount = (accountId: string) => `https://testnet.nearblocks.io/address/${accountId}`;
export const formatUnits = (value: string, decimals = 0) => {
  if (decimals === 0) return new Intl.NumberFormat().format(Number(value));
  const raw = BigInt(value);
  const divisor = 10n ** BigInt(decimals);
  const whole = raw / divisor;
  const fraction = (raw % divisor).toString().padStart(decimals, "0").replace(/0+$/, "").slice(0, 4);
  return `${new Intl.NumberFormat().format(Number(whole))}${fraction ? `.${fraction}` : ""}`;
};
