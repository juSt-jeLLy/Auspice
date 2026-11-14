export const DAY_NS = 86_400_000_000_000n;

export type LaunchTemplate = "Normal" | "Fortress" | "WorkToMint" | "BondingCurve" | "GithubRelease" | "Rainfall" | "Streak" | "Proofbound";

export const templates: LaunchTemplate[] = ["Normal", "Fortress", "WorkToMint", "BondingCurve", "GithubRelease", "Rainfall", "Streak", "Proofbound"];

export type LaunchDraft = Record<
  "template" | "name" | "symbol" | "supply" | "genesisCap" | "doorCap" | "repository" | "requiredTag" | "latitude" | "longitude" | "rainThreshold" | "dailyTarget" | "dailyCap" | "dailyMint" | "missedDayBurn" | "commitmentStatement" | "evidenceRequirement" | "commitmentDays" | "voteHours" | "quorum" | "passBps" | "creatorReward" | "voterReward" | "vestingCliffDays" | "vestingDays" | "maxWalletPercent" | "maxTransferPercent" | "cooldownSeconds" | "burnFeeBps" | "feeDecayDelayDays" | "feeDecayIntervalDays" | "feeDecayBps" | "workDifficulty" | "workUserCap" | "curveVirtualNear" | "curveMaxBuy" | "curveMaxSell" | "dailyDecayBps" | "decayStartDays" | "eventWasmUrl" | "eventWasmHash",
  string
> & { decayEnabled: boolean };

const rawU64 = (value: bigint) => `__AUSPICE_U64_${value}__`;

export function serializeLaunch(metadata: object, config: object) {
  return JSON.stringify({ metadata, config }).replace(/"__AUSPICE_U64_(\d+)__"/g, "$1");
}

export function templateShareUrl(origin: string, template: LaunchTemplate) {
  return `${origin}/create?template=${template}`;
}

function hexToBase64(value: string) {
  return btoa(value.replace(/\s/g, "").match(/.{1,2}/g)?.map((part) => String.fromCharCode(Number.parseInt(part, 16))).join("") ?? "");
}

function nearToYocto(value: string) {
  if (!/^\d+(\.\d{1,24})?$/.test(value)) throw new Error("Curve NEAR values must be positive decimal values.");
  const [whole, fraction = ""] = value.split(".");
  return BigInt(whole) * 10n ** 24n + BigInt((fraction + "0".repeat(24)).slice(0, 24));
}

export function buildLaunchConfiguration(draft: LaunchDraft, creatorId: string) {
  const total = BigInt(draft.supply || "0");
  const genesisReserve = total * 70n / 100n;
  const mechanicReserve = total - genesisReserve;
  if (total <= 0n || BigInt(draft.genesisCap || "0") <= 0n) throw new Error("Supply and Genesis cap must be positive.");
  const now = BigInt(Date.now()) * 1_000_000n;
  const genesisClose = now + DAY_NS;
  let sourceId = ""; let sourceUrl = ""; let predicate: Record<string, unknown> = {};
  if (draft.template === "GithubRelease") {
    const parts = draft.repository.trim().split("/");
    if (parts.length !== 2 || parts.some((part) => !part)) throw new Error("Use GitHub repository format owner/repository.");
    sourceId = `github_release:${draft.repository.trim()}`;
    sourceUrl = `https://api.github.com/repos/${draft.repository.trim()}/releases/latest`;
    predicate = { version: 1, required_tag: draft.requiredTag.trim() || null, minimum_magnitude_x10: null, expected_winner: null, required_text: null, minimum_rainfall_mm_x10: null };
  }
  if (draft.template === "Rainfall") {
    const lat = Number(draft.latitude); const lon = Number(draft.longitude); const threshold = Math.round(Number(draft.rainThreshold) * 10);
    if (!Number.isFinite(lat) || lat < -90 || lat > 90 || !Number.isFinite(lon) || lon < -180 || lon > 180 || threshold <= 0 || threshold > 65_535) throw new Error("Enter valid coordinates and a positive rainfall threshold.");
    sourceId = `open_meteo_rain:${lat},${lon}`;
    sourceUrl = `https://api.open-meteo.com/v1/forecast?latitude=${encodeURIComponent(String(lat))}&longitude=${encodeURIComponent(String(lon))}&current=rain`;
    predicate = { version: 1, required_tag: null, minimum_magnitude_x10: null, expected_winner: null, required_text: null, minimum_rainfall_mm_x10: threshold };
  }
  const isSignal = draft.template === "GithubRelease" || draft.template === "Rainfall";
  if (isSignal && (!draft.eventWasmUrl.startsWith("https://") || !draft.eventWasmHash || BigInt(draft.doorCap || "0") <= 0n)) throw new Error("Signal launches need a positive claim cap and the published verifier URL/hash.");
  if (draft.template === "WorkToMint" && (Number(draft.workDifficulty) < 4 || Number(draft.workDifficulty) > 24 || BigInt(draft.workUserCap || "0") <= 0n || BigInt(draft.workUserCap) > mechanicReserve)) throw new Error("Work-to-Mint difficulty must be 4–24 bits and the wallet cap must fit its reserve.");
  if (draft.template === "Streak" && (!(Number(draft.dailyTarget) > 0) || Number(draft.dailyCap) < Number(draft.dailyTarget) || Number(draft.dailyCap) > 250 || BigInt(draft.dailyMint || "0") <= 0n || BigInt(draft.missedDayBurn || "0") <= 0n)) throw new Error("Check the Streak target, participant cap, mint, and burn values.");
  if (draft.template === "Proofbound" && (BigInt(draft.creatorReward || "0") + BigInt(draft.voterReward || "0") !== mechanicReserve || Number(draft.passBps) <= 5000 || Number(draft.passBps) > 10000 || Number(draft.quorum) < 1)) throw new Error("Proofbound rewards must equal the 30% reserve, with quorum and a pass threshold above 50%.");
  const curveVirtual = draft.template === "BondingCurve" ? nearToYocto(draft.curveVirtualNear) : 0n;
  const curveBuy = draft.template === "BondingCurve" ? nearToYocto(draft.curveMaxBuy) : 0n;
  const curveSell = draft.template === "BondingCurve" ? BigInt(draft.curveMaxSell || "0") : 0n;
  if (draft.template === "BondingCurve" && (curveVirtual <= 0n || curveBuy <= 0n || curveSell <= 0n || curveSell > mechanicReserve)) throw new Error("Curve virtual reserve and trade limits must be positive and the sell cap must fit its token reserve.");
  const humanHash = process.env.NEXT_PUBLIC_HUMAN_VERIFIER_WASM_HASH ?? ""; const humanUrl = process.env.NEXT_PUBLIC_HUMAN_VERIFIER_WASM_URL ?? ""; const humanProject = process.env.NEXT_PUBLIC_HUMAN_VERIFIER_PROJECT_ID ?? "";
  if (!humanHash || !humanUrl || !humanProject) throw new Error("Human verifier public configuration is missing.");
  const cliff = Number(draft.vestingCliffDays); const duration = Number(draft.vestingDays); const wallet = Number(draft.maxWalletPercent); const transfer = Number(draft.maxTransferPercent); const fee = Number(draft.burnFeeBps); const interval = Number(draft.feeDecayIntervalDays); const feeStep = Number(draft.feeDecayBps);
  if ((draft.template === "Normal" || draft.template === "Fortress") && (!Number.isInteger(cliff) || cliff < 0 || !Number.isInteger(duration) || duration < 1)) throw new Error("Vesting cliff and duration are invalid.");
  if (draft.template === "Fortress" && (!(wallet > 0 && wallet <= 100) || !(transfer > 0 && transfer <= wallet) || Number(draft.cooldownSeconds) < 0 || fee < 0 || fee > 1000 || interval < 1 || feeStep < 0 || feeStep > fee)) throw new Error("Check the Fortress limits and fee-decay schedule.");
  if (draft.decayEnabled && (Number(draft.dailyDecayBps) < 1 || Number(draft.dailyDecayBps) > 1000 || Number(draft.decayStartDays) < 0)) throw new Error("Circulation decay must be 1–1000 bps/day and start after Genesis.");
  const protection = draft.template === "Normal" || draft.template === "Fortress" ? { creator_vesting: { reserve: mechanicReserve.toString(), cliff_at_ns: rawU64(genesisClose + BigInt(cliff) * DAY_NS), ends_at_ns: rawU64(genesisClose + BigInt(cliff + duration) * DAY_NS) }, max_wallet: draft.template === "Fortress" ? (total * BigInt(Math.round(wallet * 100)) / 10_000n).toString() : null, max_transfer: draft.template === "Fortress" ? (total * BigInt(Math.round(transfer * 100)) / 10_000n).toString() : null, transfer_cooldown_ns: draft.template === "Fortress" ? rawU64(BigInt(Number(draft.cooldownSeconds)) * 1_000_000_000n) : null, initial_burn_fee_bps: draft.template === "Fortress" ? fee : 0, fee_decay_start_ns: rawU64(genesisClose + BigInt(Number(draft.feeDecayDelayDays)) * DAY_NS), fee_decay_interval_ns: rawU64(draft.template === "Fortress" ? BigInt(interval) * DAY_NS : 0n), fee_decay_bps: draft.template === "Fortress" ? feeStep : 0 } : null;
  const config = { creator_id: creatorId, maximum_mintable_supply: total.toString(), genesis_reserve: genesisReserve.toString(), genesis_user_cap: draft.genesisCap, genesis_price_yocto_near_per_token: "1", genesis_open_at_ns: rawU64(now + 3_600_000_000_000n), genesis_close_at_ns: rawU64(genesisClose), human_verifier: { wasm_url: humanUrl, wasm_hash: hexToBase64(humanHash), wasm_build_target: "wasm32-wasip2", secrets_profile: "production", secrets_account_id: humanProject.split("/")[0], expected_hostname: new URL(process.env.NEXT_PUBLIC_SITE_URL ?? "https://web-tau-tan-79.vercel.app").host, genesis_action: "auspice_genesis", signal_action: "auspice_signal", approval_lifetime_ns: rawU64(600_000_000_000n) }, outlayer_account_id: "outlayer.testnet", max_signal_age_ns: rawU64(900_000_000_000n), no_creator_genesis_allocation: true, doors: isSignal ? [{ door_id: 0, template: draft.template, source_id: sourceId, source_url: sourceUrl, predicate, verifier_wasm_url: draft.eventWasmUrl, verifier_wasm_hash: draft.eventWasmHash, verifier_build_target: "wasm32-wasip2", reserve: mechanicReserve.toString(), user_cap: draft.doorCap, claim_window_ns: rawU64(DAY_NS), new_humans_only: true }] : [], streak: draft.template === "Streak" ? { reserve: mechanicReserve.toString(), starts_at_ns: rawU64(genesisClose), day_length_ns: rawU64(DAY_NS), daily_target: Number(draft.dailyTarget), daily_participant_cap: Number(draft.dailyCap), mint_per_checkin: draft.dailyMint, missed_day_burn: draft.missedDayBurn } : null, proofbound: draft.template === "Proofbound" ? { creator_reserve: draft.creatorReward, voter_reserve: draft.voterReward, commitments: [{ commitment_id: 0, statement: draft.commitmentStatement.trim(), evidence_requirement: draft.evidenceRequirement.trim(), evidence_deadline_ns: rawU64(genesisClose + BigInt(Number(draft.commitmentDays)) * DAY_NS), vote_duration_ns: rawU64(BigInt(Number(draft.voteHours)) * 3_600_000_000_000n), quorum: Number(draft.quorum), pass_bps: Number(draft.passBps), creator_reward: draft.creatorReward, voter_reward: draft.voterReward }] } : null, protection, work_mint: draft.template === "WorkToMint" ? { reserve: mechanicReserve.toString(), difficulty_bits: Number(draft.workDifficulty), user_cap: draft.workUserCap } : null, bonding_curve: draft.template === "BondingCurve" ? { token_reserve: mechanicReserve.toString(), virtual_near_reserve: curveVirtual.toString(), max_buy: curveBuy.toString(), max_sell: curveSell.toString() } : null, circulation: draft.decayEnabled ? { daily_decay_bps: Number(draft.dailyDecayBps), starts_at_ns: rawU64(genesisClose + BigInt(Number(draft.decayStartDays)) * DAY_NS) } : null, version: 1 };
  return { metadata: { spec: "ft-1.0.0", name: draft.name, symbol: draft.symbol.toUpperCase(), decimals: 0 }, config };
}
