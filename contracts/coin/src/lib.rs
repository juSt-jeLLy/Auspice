//! Standard NEP-141 token rails plus immutable Auspice allocation state.

use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_contract_standards::fungible_token::{
    FungibleToken, FungibleTokenCore, FungibleTokenResolver,
};
use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::borsh::BorshSerialize;
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde_json::json;
use near_sdk::store::{LookupMap, Vector};
use near_sdk::{
    env, near, require, AccountId, BorshStorageKey, Gas, NearToken, PanicOnDefault, Promise, PromiseOrValue,
};
use primitive_types::U256;

type Balance = u128;
const SIGNAL_REQUEST_TIMEOUT_NS: u64 = 600_000_000_000;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Token,
    GenesisRequests,
    GenesisClaimed,
    AcceptedProofs,
    SignalRequests,
    SignalRequestOrder,
    ActiveDoorRequests,
    GenesisPaid,
    RefundableNear,
    Doors,
    Events,
    HumanRequests,
    HumanApprovals,
    StreakCheckins,
    Commitments,
    CommitmentVotes,
    CommitmentVoterClaims,
    TransferTimestamps,
    WorkClaims,
    WorkNonces,
    CirculationTouched,
    ProtectionProbes,
}

#[near(serializers = [borsh, json])]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lifecycle {
    Draft,
    GenesisOpen,
    GenesisSettling,
    Live,
    DoorOpen,
    Finished,
}

/// A closed, auditable set of source adapters. Adding a source requires a new
/// verifier release and a new immutable Door; callers can never supply code.
#[near(serializers = [borsh, json])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SignalTemplate {
    GithubRelease,
    Rainfall,
    UsgsEarthquake,
    SportsScore,
    PublicAccountPost,
}

/// Template-specific structured policy. Version one executes GithubRelease;
/// the other variants are reserved so launch JSON remains explicit rather than
/// becoming an arbitrary URL/predicate language.
#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct SignalPredicate {
    pub version: u16,
    pub required_tag: Option<String>,
    pub minimum_magnitude_x10: Option<u16>,
    pub expected_winner: Option<String>,
    pub required_text: Option<String>,
    /// Rainfall in tenths of a millimetre. The source URL fixes the location;
    /// the TEE checks the latest reading against this immutable threshold.
    pub minimum_rainfall_mm_x10: Option<u16>,
}

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct DoorConfig {
    pub door_id: u16,
    pub template: SignalTemplate,
    pub source_id: String,
    pub source_url: String,
    pub predicate: SignalPredicate,
    pub verifier_wasm_url: String,
    pub verifier_wasm_hash: Base64VecU8,
    pub verifier_build_target: String,
    pub reserve: U128,
    pub user_cap: U128,
    pub claim_window_ns: u64,
    pub new_humans_only: bool,
}

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct LaunchConfig {
    pub creator_id: AccountId,
    pub maximum_mintable_supply: U128,
    pub genesis_reserve: U128,
    pub genesis_user_cap: U128,
    pub genesis_price_yocto_near_per_token: U128,
    pub genesis_open_at_ns: u64,
    pub genesis_close_at_ns: u64,
    /// Every launch pins this confidential CAPTCHA verifier. There is no
    /// server-signed pass fallback.
    pub human_verifier: HumanVerifierConfig,
    pub outlayer_account_id: AccountId,
    pub max_signal_age_ns: u64,
    pub no_creator_genesis_allocation: bool,
    pub doors: Vec<DoorConfig>,
    /// Present only for the fully on-chain daily community ritual. Its reserve
    /// is part of `maximum_mintable_supply`, just like a Signal Door reserve.
    pub streak: Option<StreakConfig>,
    /// A no-oracle launch-protection template. Commitments, deadlines,
    /// electorate, reward amounts, and failure burns are frozen at birth.
    pub proofbound: Option<ProofboundConfig>,
    /// Applies to a normal launch or alongside another compatible mechanic.
    /// No mutable owner configuration exists after initialization.
    pub protection: Option<LaunchProtectionConfig>,
    pub work_mint: Option<WorkMintConfig>,
    pub circulation: Option<CirculationConfig>,
    /// A self-contained market. Its token reserve is the post-Genesis share
    /// and every deposited NEAR stays locked in this contract except when a
    /// holder sells back through the same immutable curve.
    pub bonding_curve: Option<BondingCurveConfig>,
    pub version: u16,
}

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct WorkMintConfig { pub reserve: U128, pub difficulty_bits: u8, pub user_cap: U128 }
#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct CirculationConfig { pub daily_decay_bps: u16, pub starts_at_ns: u64 }
#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct BondingCurveConfig {
    pub token_reserve: U128,
    pub virtual_near_reserve: U128,
    pub max_buy: U128,
    pub max_sell: U128,
}
#[near(serializers = [json])]
pub struct BondingCurveStatus {
    pub token_reserve: U128,
    pub available_tokens: U128,
    pub locked_near: U128,
    pub virtual_near_reserve: U128,
    pub max_buy: U128,
    pub max_sell: U128,
}
#[near(serializers = [json])]
pub struct WorkMintStatus { pub reserve: U128, pub minted: U128, pub difficulty_bits: u8, pub user_cap: U128 }

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct LaunchProtectionConfig {
    pub creator_vesting: CreatorVestingConfig,
    pub max_wallet: Option<U128>,
    pub max_transfer: Option<U128>,
    pub transfer_cooldown_ns: Option<u64>,
    /// A transfer fee that is always burned. It can only decay on the fixed
    /// schedule below; no caller can increase or redirect it.
    pub initial_burn_fee_bps: u16,
    pub fee_decay_start_ns: u64,
    pub fee_decay_interval_ns: u64,
    pub fee_decay_bps: u16,
}

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct CreatorVestingConfig {
    pub reserve: U128,
    pub cliff_at_ns: u64,
    pub ends_at_ns: u64,
}

#[near(serializers = [json])]
pub struct ProtectionStatus {
    pub config: LaunchProtectionConfig,
    pub creator_claimed: U128,
    pub currently_vested: U128,
    pub current_burn_fee_bps: u16,
}

#[near(serializers = [json])]
pub struct ProtectionProbe {
    pub allowed: bool,
    pub reason: String,
    pub burn_fee: U128,
}

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct ProtectionProbeReceipt {
    pub probe_id: u64,
    pub sender_id: AccountId,
    pub receiver_id: AccountId,
    pub amount: U128,
    pub allowed: bool,
    pub reason: String,
    pub burn_fee: U128,
    pub timestamp_ns: u64,
}

#[near(serializers = [json])]
pub struct CirculationStatus {
    pub config: CirculationConfig,
    pub account_id: AccountId,
    pub balance: U128,
    pub last_settled_at_ns: u64,
    pub pending_days: u64,
    pub pending_decay: U128,
    pub next_settlement_at_ns: u64,
}

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct ProofboundConfig {
    pub creator_reserve: U128,
    pub voter_reserve: U128,
    pub commitments: Vec<CommitmentConfig>,
}

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct CommitmentConfig {
    pub commitment_id: u16,
    pub statement: String,
    pub evidence_requirement: String,
    pub evidence_deadline_ns: u64,
    pub vote_duration_ns: u64,
    pub quorum: u16,
    pub pass_bps: u16,
    pub creator_reward: U128,
    pub voter_reward: U128,
}

#[near(serializers = [borsh, json])]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitmentStatus { AwaitingEvidence, Voting, Passed, Failed }

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct CommitmentState {
    pub config: CommitmentConfig,
    pub status: CommitmentStatus,
    pub evidence_url: Option<String>,
    pub evidence_hash: Option<Base64VecU8>,
    pub vote_opens_at_ns: Option<u64>,
    pub vote_closes_at_ns: Option<u64>,
    pub yes_votes: u16,
    pub no_votes: u16,
    pub voter_reward_per_yes_vote: U128,
    pub creator_reward_claimed: bool,
}

/// Immutable rules for a Streak coin. There is deliberately no admin reset,
/// parameter setter, or backend clock: NEAR block time settles each day.
#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct StreakConfig {
    pub reserve: U128,
    pub starts_at_ns: u64,
    pub day_length_ns: u64,
    pub daily_target: u16,
    /// Bounds settlement work and the largest one-day mint.
    pub daily_participant_cap: u16,
    pub mint_per_checkin: U128,
    pub missed_day_burn: U128,
}

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct StreakState {
    pub next_day_index: u64,
    pub current_streak: u64,
    pub longest_streak: u64,
    pub remaining_reserve: U128,
    pub minted: U128,
    pub burned: U128,
    pub current_day_checkin_start: u32,
}

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct StreakCheckin {
    pub day_index: u64,
    pub account_id: AccountId,
}

#[near(serializers = [json])]
pub struct StreakStatus {
    pub config: StreakConfig,
    pub state: StreakState,
    pub today_checkins: u16,
    pub today_started_at_ns: u64,
    pub today_ends_at_ns: u64,
}

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct HumanVerifierConfig {
    pub wasm_url: String,
    pub wasm_hash: Base64VecU8,
    pub wasm_build_target: String,
    pub secrets_profile: String,
    pub secrets_account_id: AccountId,
    pub expected_hostname: String,
    pub genesis_action: String,
    pub signal_action: String,
    pub approval_lifetime_ns: u64,
}

#[near(serializers = [borsh, json])]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorStatus {
    Locked,
    Open,
    ClosedClaimed,
    ClosedBurned,
}

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct SignalDoor {
    pub config: DoorConfig,
    pub status: DoorStatus,
    pub opened_at_ns: Option<u64>,
    pub closes_at_ns: Option<u64>,
    pub proof_hash: Option<Base64VecU8>,
    pub minted: U128,
    pub burned: U128,
}

#[near(serializers = [borsh, json])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HumanCheckStatus { Pending, Approved, Rejected }

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct HumanCheckRequest {
    pub request_id: u64,
    pub requester_id: AccountId,
    pub kind: String,
    pub door_id: Option<u16>,
    pub nonce: String,
    pub status: HumanCheckStatus,
    pub requested_at_ns: u64,
}

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct HumanApproval {
    pub request_id: u64,
    pub kind: String,
    pub door_id: Option<u16>,
    pub nonce: String,
    pub expires_at_ns: u64,
    pub max_allocation: U128,
}

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct OutlayerHumanResult {
    pub request_id: u64,
    pub request_nonce: String,
    pub coin_id: AccountId,
    pub account_id: AccountId,
    pub kind: String,
    pub door_id: Option<u16>,
    pub approved: bool,
    pub expires_at_ns: u64,
    pub max_allocation: U128,
    pub wasm_hash: String,
    pub captcha_token_hash: String,
}

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct OutlayerSignalResult {
    pub request_id: u64,
    pub request_nonce: String,
    pub coin_id: AccountId,
    pub door_id: u16,
    pub template: SignalTemplate,
    pub source_id: String,
    pub source_url: String,
    pub predicate_version: u16,
    pub predicate_passed: bool,
    pub fetched_at_ns: u64,
    pub expires_at_ns: u64,
    pub wasm_hash: Base64VecU8,
    pub response_hash: Base64VecU8,
}

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct SignalRequest {
    pub request_id: u64,
    pub door_id: u16,
    pub nonce: String,
    pub requester_id: AccountId,
    pub requested_at_ns: u64,
    pub status: SignalRequestStatus,
    pub completed_at_ns: Option<u64>,
    pub failure_reason: Option<String>,
}

#[near(serializers = [borsh, json])]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SignalRequestStatus { Pending, Verified, Rejected }

#[near(serializers = [json])]
pub struct GenesisStatus {
    pub lifecycle: Lifecycle,
    pub reserve: U128,
    pub requested: U128,
    pub minted: U128,
    pub opens_at_ns: u64,
    pub closes_at_ns: u64,
}

#[near(serializers = [json])]
pub struct BirthCertificate {
    pub factory_id: AccountId,
    pub coin_id: AccountId,
    pub metadata: FungibleTokenMetadata,
    pub config: LaunchConfig,
    pub lifecycle: Lifecycle,
    pub minted_supply: U128,
    pub permanently_unmintable: U128,
}

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct AuspiceEvent {
    pub id: u64,
    pub kind: String,
    pub actor_id: AccountId,
    pub door_id: Option<u16>,
    pub amount: U128,
    pub timestamp_ns: u64,
}

#[near(serializers = [json])]
pub struct VerificationBundle {
    pub coin_id: AccountId,
    pub lifecycle: Lifecycle,
    pub doors: Vec<SignalDoor>,
    pub signal_requests: Vec<SignalRequest>,
    pub events: Vec<AuspiceEvent>,
}

#[near(serializers = [json])]
pub struct SupplyStatus {
    pub maximum_mintable_supply: U128,
    pub minted_supply: U128,
    pub permanently_unmintable: U128,
    pub remaining_mintable_supply: U128,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct AuspiceCoin {
    token: FungibleToken,
    metadata: FungibleTokenMetadata,
    factory_id: AccountId,
    config: LaunchConfig,
    lifecycle: Lifecycle,
    genesis_requests: LookupMap<AccountId, Balance>,
    genesis_claimed: LookupMap<AccountId, bool>,
    genesis_paid: LookupMap<AccountId, Balance>,
    refundable_near: LookupMap<AccountId, Balance>,
    accepted_proofs: LookupMap<String, bool>,
    signal_requests: LookupMap<u64, SignalRequest>,
    signal_request_order: Vector<u64>,
    active_door_requests: LookupMap<u16, u64>,
    doors: Vector<SignalDoor>,
    events: Vector<AuspiceEvent>,
    human_requests: LookupMap<u64, HumanCheckRequest>,
    human_approvals: LookupMap<AccountId, HumanApproval>,
    streak: Option<StreakState>,
    streak_checkins: Vector<StreakCheckin>,
    commitments: Vector<CommitmentState>,
    commitment_votes: LookupMap<String, bool>,
    commitment_voter_claims: LookupMap<String, bool>,
    creator_vesting_claimed: Balance,
    transfer_timestamps: LookupMap<AccountId, u64>,
    work_claims: LookupMap<AccountId, Balance>,
    work_nonces: LookupMap<String, bool>,
    work_minted: Balance,
    circulation_touched: LookupMap<AccountId, u64>,
    protection_probes: Vector<ProtectionProbeReceipt>,
    curve_available_tokens: Balance,
    curve_locked_near: Balance,
    total_requested: Balance,
    minted_supply: Balance,
    permanently_unmintable: Balance,
    event_count: u64,
    next_signal_request_id: u64,
    next_human_request_id: u64,
}

#[near]
impl AuspiceCoin {
    #[init]
    pub fn new(
        factory_id: AccountId,
        metadata: FungibleTokenMetadata,
        config: LaunchConfig,
    ) -> Self {
        require!(config.version == 1, "AUSPICE[CONFIG] unsupported version");
        require!(
            config.genesis_open_at_ns < config.genesis_close_at_ns,
            "AUSPICE[CONFIG] invalid Genesis schedule"
        );
        require!(
            config.max_signal_age_ns > 0,
            "AUSPICE[CONFIG] maximum signal age is required"
        );
        require!(
            config.genesis_price_yocto_near_per_token.0 > 0,
            "AUSPICE[CONFIG] Genesis price must be positive"
        );
        let verifier = &config.human_verifier;
        require!(verifier.wasm_hash.0.len() == 32 && verifier.wasm_url.starts_with("https://") && verifier.wasm_build_target == "wasm32-wasip2", "AUSPICE[CONFIG] human verifier must pin a HTTPS WASI P2 build");
        require!(!verifier.secrets_profile.is_empty() && !verifier.expected_hostname.is_empty() && !verifier.genesis_action.is_empty() && !verifier.signal_action.is_empty() && verifier.approval_lifetime_ns > 0, "AUSPICE[CONFIG] invalid human verifier policy");
        metadata.assert_valid();
        let door_total: Balance = config.doors.iter().map(|d| d.reserve.0).sum();
        let streak_total = config.streak.as_ref().map(|streak| streak.reserve.0).unwrap_or(0);
        let proofbound_total = config.proofbound.as_ref().map(|proof| proof.creator_reserve.0 + proof.voter_reserve.0).unwrap_or(0);
        let protection_total = config.protection.as_ref().map(|protection| protection.creator_vesting.reserve.0).unwrap_or(0);
        let work_total = config.work_mint.as_ref().map(|work| work.reserve.0).unwrap_or(0);
        let curve_total = config.bonding_curve.as_ref().map(|curve| curve.token_reserve.0).unwrap_or(0);
        require!(
            config.genesis_reserve.0 + door_total + streak_total + proofbound_total + protection_total + work_total + curve_total == config.maximum_mintable_supply.0,
            "AUSPICE[CONFIG] reserves must equal maximum supply"
        );
        if let Some(streak) = &config.streak {
            require!(config.doors.is_empty(), "AUSPICE[CONFIG] Streak coins cannot mix Signal Doors");
            require!(streak.reserve.0 > 0 && streak.starts_at_ns >= config.genesis_close_at_ns && streak.day_length_ns > 0 && streak.daily_target > 0 && streak.daily_target <= streak.daily_participant_cap && streak.daily_participant_cap <= 250 && streak.mint_per_checkin.0 > 0 && streak.missed_day_burn.0 > 0, "AUSPICE[CONFIG] invalid Streak configuration");
        }
        if let Some(proof) = &config.proofbound {
            require!(config.doors.is_empty() && config.streak.is_none(), "AUSPICE[CONFIG] Proofbound coins cannot mix other mechanisms");
            require!(!proof.commitments.is_empty() && proof.commitments.len() <= 10, "AUSPICE[CONFIG] Proofbound requires 1-10 commitments");
            let mut creator_total = 0u128;
            let mut voter_total = 0u128;
            for (index, commitment) in proof.commitments.iter().enumerate() {
                require!(commitment.commitment_id == index as u16 && !commitment.statement.is_empty() && commitment.statement.len() <= 280 && !commitment.evidence_requirement.is_empty() && commitment.evidence_requirement.len() <= 280 && commitment.evidence_deadline_ns >= config.genesis_close_at_ns && commitment.vote_duration_ns > 0 && commitment.quorum > 0 && commitment.pass_bps > 5_000 && commitment.pass_bps <= 10_000, "AUSPICE[CONFIG] invalid Proofbound commitment");
                creator_total = creator_total.saturating_add(commitment.creator_reward.0);
                voter_total = voter_total.saturating_add(commitment.voter_reward.0);
            }
            require!(creator_total == proof.creator_reserve.0 && voter_total == proof.voter_reserve.0 && creator_total + voter_total > 0, "AUSPICE[CONFIG] Proofbound reserves must equal commitment rewards");
        }
        if let Some(protection) = &config.protection {
            let vesting = &protection.creator_vesting;
            require!(vesting.reserve.0 > 0 && vesting.cliff_at_ns >= config.genesis_close_at_ns && vesting.ends_at_ns >= vesting.cliff_at_ns && protection.initial_burn_fee_bps <= 1_000 && protection.fee_decay_bps <= protection.initial_burn_fee_bps, "AUSPICE[CONFIG] invalid launch protection");
            if protection.initial_burn_fee_bps > 0 { require!(protection.fee_decay_interval_ns > 0, "AUSPICE[CONFIG] fee decay interval required"); }
            if let Some(max_wallet) = protection.max_wallet { require!(max_wallet.0 > 0, "AUSPICE[CONFIG] max wallet must be positive"); }
            if let Some(max_transfer) = protection.max_transfer { require!(max_transfer.0 > 0, "AUSPICE[CONFIG] max transfer must be positive"); }
        }
        if let Some(work) = &config.work_mint { require!(config.doors.is_empty() && config.streak.is_none() && config.proofbound.is_none() && work.reserve.0 > 0 && work.user_cap.0 > 0 && (4..=24).contains(&work.difficulty_bits), "AUSPICE[CONFIG] invalid Work-to-Mint configuration"); }
        if let Some(curve) = &config.bonding_curve {
            require!(config.doors.is_empty() && config.streak.is_none() && config.proofbound.is_none() && config.work_mint.is_none() && config.protection.is_none() && curve.token_reserve.0 > 0 && curve.virtual_near_reserve.0 > 0 && curve.max_buy.0 > 0 && curve.max_sell.0 > 0, "AUSPICE[CONFIG] invalid bonding curve configuration");
        }
        if let Some(circulation) = &config.circulation { require!(circulation.daily_decay_bps > 0 && circulation.daily_decay_bps <= 1_000 && circulation.starts_at_ns >= config.genesis_close_at_ns, "AUSPICE[CONFIG] invalid circulation configuration"); }
        for (index, door) in config.doors.iter().enumerate() {
            require!(door.door_id == index as u16 && door.user_cap.0 > 0 && door.reserve.0 > 0 && door.predicate.version > 0, "AUSPICE[CONFIG] invalid Door configuration");
            require!(door.verifier_wasm_hash.0.len() == 32 && door.verifier_wasm_url.starts_with("https://") && matches!(door.verifier_build_target.as_str(), "wasm32-wasip1" | "wasm32-wasip2"), "AUSPICE[CONFIG] Door must pin a HTTPS WASI verifier");
            match door.template {
                SignalTemplate::GithubRelease => require!(door.source_id.starts_with("github_release:") && door.source_url.starts_with("https://api.github.com/repos/"), "AUSPICE[CONFIG] invalid GitHub release source"),
                SignalTemplate::Rainfall => require!(door.source_id.starts_with("open_meteo_rain:") && door.source_url.starts_with("https://api.open-meteo.com/v1/forecast?") && door.predicate.minimum_rainfall_mm_x10.is_some(), "AUSPICE[CONFIG] invalid Open-Meteo rainfall source"),
                _ => env::panic_str("AUSPICE[CONFIG] this Signal template is not enabled yet"),
            }
        }
        let streak_state = config.streak.as_ref().map(|streak| StreakState { next_day_index: 0, current_streak: 0, longest_streak: 0, remaining_reserve: streak.reserve, minted: U128(0), burned: U128(0), current_day_checkin_start: 0 });
        let lifecycle = if env::block_timestamp() < config.genesis_open_at_ns {
            Lifecycle::Draft
        } else if env::block_timestamp() < config.genesis_close_at_ns {
            Lifecycle::GenesisOpen
        } else {
            Lifecycle::GenesisSettling
        };
        let mut doors = Vector::new(StorageKey::Doors);
        for door_config in config.doors.iter().cloned() {
            doors.push(SignalDoor {
                config: door_config,
                status: DoorStatus::Locked,
                opened_at_ns: None,
                closes_at_ns: None,
                proof_hash: None,
                minted: U128(0),
                burned: U128(0),
            });
        }
        let mut commitments = Vector::new(StorageKey::Commitments);
        if let Some(proof) = &config.proofbound {
            for commitment in proof.commitments.iter().cloned() {
                commitments.push(CommitmentState { config: commitment, status: CommitmentStatus::AwaitingEvidence, evidence_url: None, evidence_hash: None, vote_opens_at_ns: None, vote_closes_at_ns: None, yes_votes: 0, no_votes: 0, voter_reward_per_yes_vote: U128(0), creator_reward_claimed: false });
            }
        }
        let curve_available_tokens = config.bonding_curve.as_ref().map(|curve| curve.token_reserve.0).unwrap_or(0);
        Self {
            token: FungibleToken::new(StorageKey::Token),
            metadata,
            factory_id,
            config,
            lifecycle,
            genesis_requests: LookupMap::new(StorageKey::GenesisRequests),
            genesis_claimed: LookupMap::new(StorageKey::GenesisClaimed),
            genesis_paid: LookupMap::new(StorageKey::GenesisPaid),
            refundable_near: LookupMap::new(StorageKey::RefundableNear),
            accepted_proofs: LookupMap::new(StorageKey::AcceptedProofs),
            signal_requests: LookupMap::new(StorageKey::SignalRequests),
            signal_request_order: Vector::new(StorageKey::SignalRequestOrder),
            active_door_requests: LookupMap::new(StorageKey::ActiveDoorRequests),
            doors,
            events: Vector::new(StorageKey::Events),
            human_requests: LookupMap::new(StorageKey::HumanRequests),
            human_approvals: LookupMap::new(StorageKey::HumanApprovals),
            streak: streak_state,
            streak_checkins: Vector::new(StorageKey::StreakCheckins),
            commitments,
            commitment_votes: LookupMap::new(StorageKey::CommitmentVotes),
            commitment_voter_claims: LookupMap::new(StorageKey::CommitmentVoterClaims),
            creator_vesting_claimed: 0,
            transfer_timestamps: LookupMap::new(StorageKey::TransferTimestamps),
            work_claims: LookupMap::new(StorageKey::WorkClaims), work_nonces: LookupMap::new(StorageKey::WorkNonces), work_minted: 0,
            circulation_touched: LookupMap::new(StorageKey::CirculationTouched),
            protection_probes: Vector::new(StorageKey::ProtectionProbes),
            curve_available_tokens,
            curve_locked_near: 0,
            total_requested: 0,
            minted_supply: 0,
            permanently_unmintable: 0,
            event_count: 0,
            next_signal_request_id: 0,
            next_human_request_id: 0,
        }
    }

    /// The approval can only have been written by
    /// `on_human_check_result`, which is a private OutLayer promise callback.
    #[payable]
    pub fn request_genesis_allocation(&mut self, requested_amount: U128) {
        self.refresh_lifecycle();
        require!(self.lifecycle == Lifecycle::GenesisOpen, "AUSPICE[GENESIS] Genesis is not open");
        require!(requested_amount.0 > 0 && requested_amount.0 <= self.config.genesis_user_cap.0, "AUSPICE[GENESIS] allocation exceeds cap");
        let account = env::predecessor_account_id();
        let approval = self.human_approvals.get(&account).cloned().unwrap_or_else(|| env::panic_str("AUSPICE[HUMAN] complete the TEE human check first"));
        require!(approval.kind == "genesis" && approval.door_id.is_none() && approval.expires_at_ns > env::block_timestamp() && requested_amount.0 <= approval.max_allocation.0, "AUSPICE[HUMAN] approval is invalid or expired");
        require!(!self.config.no_creator_genesis_allocation || account != self.config.creator_id, "AUSPICE[GENESIS] creator allocation is disabled");
        require!(self.token.accounts.contains_key(&account), "AUSPICE[STORAGE] register token storage first");
        let prior = self.genesis_requests.get(&account).copied().unwrap_or(0);
        require!(prior + requested_amount.0 <= self.config.genesis_user_cap.0, "AUSPICE[GENESIS] cumulative cap exceeded");
        let attached = env::attached_deposit().as_yoctonear();
        let required = self.price_for(requested_amount.0);
        require!(attached >= required, "AUSPICE[GENESIS] attached NEAR is below the fixed allocation price");
        self.genesis_requests.insert(account.clone(), prior + requested_amount.0);
        let paid = self.genesis_paid.get(&account).copied().unwrap_or(0);
        self.genesis_paid.insert(account.clone(), paid + required);
        if attached > required { self.credit_refund(&account, attached - required); }
        self.total_requested += requested_amount.0;
        self.human_approvals.remove(&account);
        self.emit("genesis_request", None, requested_amount.0);
    }

    /// Permissionless after the deadline: each participant settles its immutable request.
    pub fn settle_genesis(&mut self, account_id: AccountId) {
        self.refresh_lifecycle();
        require!(
            matches!(self.lifecycle, Lifecycle::GenesisSettling | Lifecycle::Live),
            "AUSPICE[GENESIS] settlement unavailable"
        );
        let requested = self.genesis_requests.get(&account_id).copied().unwrap_or(0);
        require!(
            requested > 0 && !self.genesis_claimed.contains_key(&account_id),
            "AUSPICE[GENESIS] no unsettled request"
        );
        let allocation = if self.total_requested <= self.config.genesis_reserve.0 {
            requested
        } else {
            requested * self.config.genesis_reserve.0 / self.total_requested
        };
        self.mint_to(&account_id, allocation);
        self.genesis_claimed.insert(account_id.clone(), true);
        let paid = self.genesis_paid.get(&account_id).copied().unwrap_or(0);
        let due = self.price_for(allocation);
        if paid > due {
            self.credit_refund(&account_id, paid - due);
        }
        self.emit("genesis_settled", None, allocation);
    }

    pub fn finalize_genesis(&mut self) {
        self.refresh_lifecycle();
        require!(
            self.lifecycle == Lifecycle::GenesisSettling,
            "AUSPICE[GENESIS] settlement required"
        );
        // No request can be added after the close timestamp. The unused portion
        // of the Genesis reserve is therefore permanently removed before normal
        // transfers become available; it cannot become an administrative mint.
        self.permanently_unmintable += self
            .config
            .genesis_reserve
            .0
            .saturating_sub(self.total_requested);
        self.lifecycle = Lifecycle::Live;
        self.emit("genesis_finalized", None, 0);
    }

    /// Join the current on-chain daily ritual once. A holder is an account
    /// with a positive NEP-141 balance; the Genesis human check is the
    /// protocol's demo-level Sybil control, described in the README.
    #[payable]
    pub fn check_in_streak(&mut self) {
        require!(env::attached_deposit().as_yoctonear() == 1, "AUSPICE[STREAK] attach exactly 1 yoctoNEAR");
        require!(self.lifecycle == Lifecycle::Live, "AUSPICE[STREAK] Streak is not available");
        let config = self.config.streak.clone().unwrap_or_else(|| env::panic_str("AUSPICE[STREAK] this coin has no Streak ritual"));
        let state = self.streak.as_ref().cloned().unwrap_or_else(|| env::panic_str("AUSPICE[STREAK] missing state"));
        let now = env::block_timestamp();
        let start = config.starts_at_ns.saturating_add(state.next_day_index.saturating_mul(config.day_length_ns));
        require!(now >= start && now < start.saturating_add(config.day_length_ns), "AUSPICE[STREAK] settle the prior day or wait for the next day");
        let account = env::predecessor_account_id();
        require!(self.token.ft_balance_of(account.clone()).0 > 0, "AUSPICE[STREAK] only holders can check in");
        let key = format!("{}:{}", state.next_day_index, account);
        require!(!self.accepted_proofs.contains_key(&key), "AUSPICE[STREAK] already checked in today");
        let today_count = self.streak_checkins.len().saturating_sub(state.current_day_checkin_start) as u16;
        require!(today_count < config.daily_participant_cap, "AUSPICE[STREAK] daily participant cap reached");
        self.accepted_proofs.insert(key, true);
        self.streak_checkins.push(StreakCheckin { day_index: state.next_day_index, account_id: account.clone() });
        self.emit("streak_checkin", None, 0);
    }

    /// Permissionless daily settlement. The bounded participant cap makes the
    /// deterministic mint loop safe to execute in a NEAR transaction.
    pub fn settle_streak_day(&mut self) {
        require!(self.lifecycle == Lifecycle::Live, "AUSPICE[STREAK] Streak is not available");
        let config = self.config.streak.clone().unwrap_or_else(|| env::panic_str("AUSPICE[STREAK] this coin has no Streak ritual"));
        let mut state = self.streak.as_ref().cloned().unwrap_or_else(|| env::panic_str("AUSPICE[STREAK] missing state"));
        let day_start = config.starts_at_ns.saturating_add(state.next_day_index.saturating_mul(config.day_length_ns));
        require!(env::block_timestamp() >= day_start.saturating_add(config.day_length_ns), "AUSPICE[STREAK] today's check-in window is still open");
        let start = state.current_day_checkin_start;
        let end = self.streak_checkins.len();
        let checkins = end.saturating_sub(start) as u16;
        if checkins >= config.daily_target {
            let reward = (checkins as Balance).saturating_mul(config.mint_per_checkin.0);
            if reward <= state.remaining_reserve.0 {
                for index in start..end {
                    let account_id = self.streak_checkins.get(index).unwrap_or_else(|| env::panic_str("AUSPICE[STREAK] missing check-in")).account_id.clone();
                    self.mint_to(&account_id, config.mint_per_checkin.0);
                }
                state.remaining_reserve.0 -= reward;
                state.minted.0 += reward;
                state.current_streak += 1;
                state.longest_streak = state.longest_streak.max(state.current_streak);
                self.emit("streak_mint", None, reward);
            } else {
                // A fixed mint must never be silently reduced. Once the
                // immutable reserve cannot fund a complete successful day,
                // the remainder is permanently removed and the ritual ends.
                let remainder = state.remaining_reserve.0;
                state.remaining_reserve = U128(0);
                state.burned.0 += remainder;
                self.permanently_unmintable += remainder;
                self.emit("streak_reserve_exhausted", None, remainder);
            }
        } else {
            let burn = state.remaining_reserve.0.min(config.missed_day_burn.0);
            state.remaining_reserve.0 -= burn;
            state.burned.0 += burn;
            state.current_streak = 0;
            self.permanently_unmintable += burn;
            self.emit("streak_burn", None, burn);
        }
        state.next_day_index += 1;
        state.current_day_checkin_start = end;
        let exhausted = state.remaining_reserve.0 == 0;
        self.streak = Some(state);
        if exhausted { self.lifecycle = Lifecycle::Finished; }
    }

    /// Creator-provided evidence is an append-only public claim, not an
    /// oracle verdict. Genesis-settled, CAPTCHA-checked holders decide it.
    pub fn submit_commitment_evidence(&mut self, commitment_id: u16, evidence_url: String, evidence_hash: Base64VecU8) {
        require!(env::predecessor_account_id() == self.config.creator_id, "AUSPICE[PROOFBOUND] creator only");
        require!(self.lifecycle == Lifecycle::Live && evidence_url.starts_with("https://") && evidence_url.len() <= 512 && evidence_hash.0.len() == 32, "AUSPICE[PROOFBOUND] invalid evidence");
        let state = self.commitment_mut(commitment_id);
        require!(state.status == CommitmentStatus::AwaitingEvidence && env::block_timestamp() <= state.config.evidence_deadline_ns, "AUSPICE[PROOFBOUND] evidence window closed");
        state.evidence_url = Some(evidence_url);
        state.evidence_hash = Some(evidence_hash);
        state.status = CommitmentStatus::Voting;
        state.vote_opens_at_ns = Some(env::block_timestamp());
        state.vote_closes_at_ns = Some(env::block_timestamp().saturating_add(state.config.vote_duration_ns));
        self.emit("commitment_evidence_submitted", None, 0);
    }

    /// If the creator misses the evidence deadline, anyone can still open a
    /// vote. The blank evidence itself is visible to the electorate.
    pub fn open_commitment_vote(&mut self, commitment_id: u16) {
        require!(self.lifecycle == Lifecycle::Live, "AUSPICE[PROOFBOUND] commitments are unavailable");
        let state = self.commitment_mut(commitment_id);
        require!(state.status == CommitmentStatus::AwaitingEvidence && env::block_timestamp() >= state.config.evidence_deadline_ns, "AUSPICE[PROOFBOUND] vote cannot open yet");
        state.status = CommitmentStatus::Voting;
        state.vote_opens_at_ns = Some(env::block_timestamp());
        state.vote_closes_at_ns = Some(env::block_timestamp().saturating_add(state.config.vote_duration_ns));
        self.emit("commitment_vote_opened", None, 0);
    }

    /// `request_human_check("commitment_vote", Some(commitment_id), token)`
    /// must have completed immediately before this vote. One settled Genesis
    /// recipient gets one equal vote; token balances never buy extra votes.
    pub fn vote_commitment(&mut self, commitment_id: u16, support: bool) {
        require!(self.lifecycle == Lifecycle::Live, "AUSPICE[PROOFBOUND] commitments are unavailable");
        let account = env::predecessor_account_id();
        require!(self.genesis_claimed.contains_key(&account), "AUSPICE[PROOFBOUND] only settled Genesis participants can vote");
        let approval = self.human_approvals.get(&account).cloned().unwrap_or_else(|| env::panic_str("AUSPICE[HUMAN] complete the TEE human check first"));
        require!(approval.kind == "commitment_vote" && approval.door_id == Some(commitment_id) && approval.expires_at_ns > env::block_timestamp(), "AUSPICE[HUMAN] vote approval is invalid or expired");
        let vote_key = format!("{}:{}", commitment_id, account);
        require!(!self.commitment_votes.contains_key(&vote_key), "AUSPICE[PROOFBOUND] vote already recorded");
        let state = self.commitment_mut(commitment_id);
        require!(state.status == CommitmentStatus::Voting && env::block_timestamp() < state.vote_closes_at_ns.unwrap_or(0), "AUSPICE[PROOFBOUND] voting is closed");
        if support { state.yes_votes += 1; } else { state.no_votes += 1; }
        self.commitment_votes.insert(vote_key, support);
        self.human_approvals.remove(&account);
        self.emit(if support { "commitment_yes_vote" } else { "commitment_no_vote" }, None, 0);
    }

    /// Permissionless, deterministic conclusion. Failed commitments burn both
    /// the locked creator slice and the voter-reward slice; passed commitments
    /// leave rewards individually claimable so settlement never loops voters.
    pub fn settle_commitment(&mut self, commitment_id: u16) {
        require!(self.lifecycle == Lifecycle::Live, "AUSPICE[PROOFBOUND] commitments are unavailable");
        let state = self.commitment_mut(commitment_id);
        require!(state.status == CommitmentStatus::Voting && env::block_timestamp() >= state.vote_closes_at_ns.unwrap_or(u64::MAX), "AUSPICE[PROOFBOUND] voting is still open");
        let total_votes = state.yes_votes as u128 + state.no_votes as u128;
        let passed = total_votes >= state.config.quorum as u128 && (state.yes_votes as u128).saturating_mul(10_000) >= total_votes.saturating_mul(state.config.pass_bps as u128);
        if passed {
            let per_vote = state.config.voter_reward.0 / state.yes_votes as u128;
            let dust = state.config.voter_reward.0 - per_vote * state.yes_votes as u128;
            state.voter_reward_per_yes_vote = U128(per_vote);
            state.status = CommitmentStatus::Passed;
            self.permanently_unmintable += dust;
            self.emit("commitment_passed", None, dust);
        } else {
            let burned = state.config.creator_reward.0 + state.config.voter_reward.0;
            state.status = CommitmentStatus::Failed;
            self.permanently_unmintable += burned;
            self.emit("commitment_failed_burned", None, burned);
        }
        if self.commitments.iter().all(|commitment| matches!(commitment.status, CommitmentStatus::Passed | CommitmentStatus::Failed)) {
            self.lifecycle = Lifecycle::Finished;
        }
    }

    pub fn claim_creator_commitment_reward(&mut self, commitment_id: u16) {
        require!(env::predecessor_account_id() == self.config.creator_id, "AUSPICE[PROOFBOUND] creator only");
        require!(self.token.accounts.contains_key(&self.config.creator_id), "AUSPICE[STORAGE] register token storage first");
        let creator = self.config.creator_id.clone();
        let state = self.commitment_mut(commitment_id);
        require!(state.status == CommitmentStatus::Passed && !state.creator_reward_claimed, "AUSPICE[PROOFBOUND] creator reward is unavailable");
        state.creator_reward_claimed = true;
        let reward = state.config.creator_reward.0;
        self.mint_to(&creator, reward);
        self.emit("creator_commitment_reward_claimed", None, reward);
    }

    pub fn claim_commitment_voter_reward(&mut self, commitment_id: u16) {
        let account = env::predecessor_account_id();
        require!(self.token.accounts.contains_key(&account), "AUSPICE[STORAGE] register token storage first");
        let vote_key = format!("{}:{}", commitment_id, account);
        require!(self.commitment_votes.get(&vote_key).copied() == Some(true) && !self.commitment_voter_claims.contains_key(&vote_key), "AUSPICE[PROOFBOUND] no unclaimed yes-vote reward");
        let reward = self.commitment(commitment_id).voter_reward_per_yes_vote.0;
        require!(self.commitment(commitment_id).status == CommitmentStatus::Passed && reward > 0, "AUSPICE[PROOFBOUND] voter reward is unavailable");
        self.commitment_voter_claims.insert(vote_key, true);
        self.mint_to(&account, reward);
        self.emit("commitment_voter_reward_claimed", None, reward);
    }

    /// The only path for the creator's protected allocation. The contract
    /// computes the vesting curve from immutable timestamps; there is no
    /// owner release or revoke method.
    pub fn claim_creator_vested(&mut self) {
        require!(matches!(self.lifecycle, Lifecycle::Live | Lifecycle::DoorOpen | Lifecycle::Finished), "AUSPICE[PROTECTION] vesting starts after Genesis");
        require!(env::predecessor_account_id() == self.config.creator_id, "AUSPICE[PROTECTION] creator only");
        let vested = self.currently_vested_creator_amount();
        require!(vested > self.creator_vesting_claimed, "AUSPICE[PROTECTION] no vested tokens available");
        let amount = vested - self.creator_vesting_claimed;
        self.creator_vesting_claimed = vested;
        let creator = self.config.creator_id.clone();
        self.mint_to(&creator, amount);
        self.emit("creator_vesting_claimed", None, amount);
    }

    #[payable]
    pub fn request_signal_check(&mut self, door_id: u16) -> Promise {
        require!(
            self.lifecycle == Lifecycle::Live,
            "AUSPICE[DOOR] Genesis is not live"
        );
        let door_config = {
            let door = self.door(door_id);
            require!(
                door.status == DoorStatus::Locked,
                "AUSPICE[DOOR] Door is not available"
            );
            require!(!self.active_door_requests.contains_key(&door_id), "AUSPICE[DOOR] evaluation already pending");
            door.config.clone()
        };
        let request_id = self.next_signal_request_id;
        self.next_signal_request_id += 1;
        let nonce = hex(&env::sha256(
            format!("{}:{}", env::current_account_id(), request_id).as_bytes(),
        ));
        self.signal_requests.insert(
            request_id,
            SignalRequest {
                request_id,
                door_id,
                nonce: nonce.clone(),
                requester_id: env::predecessor_account_id(),
                requested_at_ns: env::block_timestamp(),
                status: SignalRequestStatus::Pending,
                completed_at_ns: None,
                failure_reason: None,
            },
        );
        self.signal_request_order.push(request_id);
        self.active_door_requests.insert(door_id, request_id);
        self.emit("signal_submitted", Some(door_id), 0);
        let input = json!({"request_id":request_id,"request_nonce":nonce,"coin_id":env::current_account_id(),"door_id":door_id,"template":door_config.template,"source_id":door_config.source_id,"source_url":door_config.source_url,"predicate":door_config.predicate,"predicate_version":door_config.predicate.version,"expected_wasm_hash":door_config.verifier_wasm_hash}).to_string();
        // OutLayer's live ABI has seven fields. Keep the unused fields explicit
        // so an omitted argument can never shift the payer/params position.
        Promise::new(self.config.outlayer_account_id.clone()).function_call("request_execution", json!({"source":{"WasmUrl":{"url":door_config.verifier_wasm_url,"hash":hex(&door_config.verifier_wasm_hash.0),"build_target":door_config.verifier_build_target}},"resource_limits":{"max_instructions":1_000_000_000u64,"max_execution_seconds":30u64},"input_data":input,"secrets_ref":null,"response_format":"Json","payer_account_id":env::predecessor_account_id(),"params":null}).to_string().into_bytes(), env::attached_deposit(), Gas::from_tgas(200)).then(Self::ext(env::current_account_id()).with_static_gas(Gas::from_tgas(40)).on_signal_result(request_id))
    }

    /// The CAPTCHA token is evaluated only by the configured pinned WASM
    /// with its protected OutLayer secret. This method records no allocation.
    #[payable]
    pub fn request_human_check(&mut self, kind: String, door_id: Option<u16>, captcha_token: String) -> Promise {
        let verifier = self.config.human_verifier.clone();
        require!(matches!(kind.as_str(), "genesis" | "signal_door" | "commitment_vote") && captcha_token.len() <= 4096 && !captcha_token.is_empty(), "AUSPICE[HUMAN] invalid request");
        if kind == "genesis" { self.refresh_lifecycle(); require!(self.lifecycle == Lifecycle::GenesisOpen && door_id.is_none(), "AUSPICE[HUMAN] Genesis is unavailable"); }
        if kind == "signal_door" { let id = door_id.unwrap_or_else(|| env::panic_str("AUSPICE[HUMAN] Door required")); require!(self.lifecycle == Lifecycle::DoorOpen && self.door(id).status == DoorStatus::Open, "AUSPICE[HUMAN] Door is unavailable"); }
        if kind == "commitment_vote" { let id = door_id.unwrap_or_else(|| env::panic_str("AUSPICE[HUMAN] commitment required")); require!(self.lifecycle == Lifecycle::Live && self.commitment(id).status == CommitmentStatus::Voting, "AUSPICE[HUMAN] commitment vote is unavailable"); }
        let request_id = self.next_human_request_id;
        self.next_human_request_id += 1;
        let nonce = hex(&env::sha256(format!("human:{}:{}", env::current_account_id(), request_id).as_bytes()));
        let requester = env::predecessor_account_id();
        let max = if kind == "genesis" { self.config.genesis_user_cap } else if kind == "signal_door" { self.door(door_id.unwrap()).config.user_cap } else { U128(1) };
        self.human_requests.insert(request_id, HumanCheckRequest { request_id, requester_id: requester.clone(), kind: kind.clone(), door_id, nonce: nonce.clone(), status: HumanCheckStatus::Pending, requested_at_ns: env::block_timestamp() });
        self.emit("human_check_submitted", door_id, 0);
        let input = json!({"request_id":request_id,"request_nonce":nonce,"coin_id":env::current_account_id(),"account_id":requester,"kind":kind,"door_id":door_id,"captcha_token":captcha_token,"expected_hostname":verifier.expected_hostname,"expected_action":if door_id.is_some(){verifier.signal_action}else{verifier.genesis_action},"max_allocation":max,"expected_wasm_hash":hex(&verifier.wasm_hash.0)}).to_string();
        Promise::new(self.config.outlayer_account_id.clone()).function_call("request_execution", json!({"source":{"WasmUrl":{"url":verifier.wasm_url,"hash":hex(&verifier.wasm_hash.0),"build_target":verifier.wasm_build_target}},"resource_limits":{"max_instructions":1_000_000_000u64,"max_execution_seconds":30u64},"input_data":input,"secrets_ref":{"profile":verifier.secrets_profile,"account_id":verifier.secrets_account_id},"response_format":"Json","payer_account_id":env::predecessor_account_id(),"params":null}).to_string().into_bytes(), env::attached_deposit(), Gas::from_tgas(200)).then(Self::ext(env::current_account_id()).with_static_gas(Gas::from_tgas(30)).on_human_check_result(request_id))
    }

    #[private]
    pub fn on_human_check_result(&mut self, request_id: u64) {
        let mut request = self.human_requests.get(&request_id).cloned().unwrap_or_else(|| env::panic_str("AUSPICE[HUMAN] unknown request"));
        require!(request.status == HumanCheckStatus::Pending, "AUSPICE[HUMAN] already resolved");
        let verifier = self.config.human_verifier.clone();
        let bytes = match env::promise_result_checked(0, 16_384) { Ok(value) => value, Err(_) => { request.status = HumanCheckStatus::Rejected; self.human_requests.insert(request_id, request); return; } };
        let result: OutlayerHumanResult = match near_sdk::serde_json::from_slice(&bytes) { Ok(value) => value, Err(_) => { request.status = HumanCheckStatus::Rejected; self.human_requests.insert(request_id, request); return; } };
        let valid = result.approved && result.request_id == request_id && result.request_nonce == request.nonce && result.coin_id == env::current_account_id() && result.account_id == request.requester_id && result.kind == request.kind && result.door_id == request.door_id && result.expires_at_ns > env::block_timestamp() && result.expires_at_ns <= env::block_timestamp().saturating_add(verifier.approval_lifetime_ns) && result.wasm_hash == hex(&verifier.wasm_hash.0);
        if !valid { request.status = HumanCheckStatus::Rejected; self.human_requests.insert(request_id, request); self.emit("human_check_rejected", None, 0); return; }
        request.status = HumanCheckStatus::Approved;
        self.human_requests.insert(request_id, request.clone());
        self.human_approvals.insert(request.requester_id, HumanApproval { request_id, kind: result.kind, door_id: result.door_id, nonce: result.request_nonce, expires_at_ns: result.expires_at_ns, max_allocation: result.max_allocation });
        self.emit("human_check_approved", result.door_id, 0);
    }

    pub fn get_human_approval(&self, account_id: AccountId) -> Option<HumanApproval> { self.human_approvals.get(&account_id).cloned() }

    #[private]
    pub fn on_signal_result(&mut self, request_id: u64) {
        let mut request = self.signal_requests.get(&request_id).cloned().unwrap_or_else(|| env::panic_str("AUSPICE[PROOF] unknown request"));
        require!(request.status == SignalRequestStatus::Pending, "AUSPICE[PROOF] request already completed");
        self.active_door_requests.remove(&request.door_id);
        let bytes = match env::promise_result_checked(0, 16_384) { Ok(value) => value, Err(_) => return self.reject_signal_request(&mut request, "OutLayer execution failed") };
        let result: OutlayerSignalResult = match near_sdk::serde_json::from_slice(&bytes) { Ok(value) => value, Err(_) => return self.reject_signal_request(&mut request, "invalid OutLayer response") };
        let common_valid =
            result.request_id == request_id
                && result.request_nonce == request.nonce
                && result.coin_id == env::current_account_id()
                && result.door_id == request.door_id
                && result.predicate_passed
                && result.response_hash.0.len() == 32
                && result.fetched_at_ns >= request.requested_at_ns
                && result.expires_at_ns > env::block_timestamp()
                && result.fetched_at_ns <= env::block_timestamp()
                && env::block_timestamp() - result.fetched_at_ns <= self.config.max_signal_age_ns;
        if !common_valid { return self.reject_signal_request(&mut request, "response binding mismatch"); }
        let door = self.door(request.door_id);
        if !(result.template == door.config.template && result.source_id == door.config.source_id && result.source_url == door.config.source_url && result.predicate_version == door.config.predicate.version && result.wasm_hash.0 == door.config.verifier_wasm_hash.0) { return self.reject_signal_request(&mut request, "immutable policy mismatch"); }
        let proof_key = hex(&env::sha256(&result.response_hash.0));
        if self.accepted_proofs.contains_key(&proof_key) { return self.reject_signal_request(&mut request, "proof already used"); }
        let door = self.door_mut(request.door_id);
        require!(
            door.status == DoorStatus::Locked,
            "AUSPICE[DOOR] Door already opened"
        );
        door.status = DoorStatus::Open;
        door.opened_at_ns = Some(env::block_timestamp());
        door.closes_at_ns = Some(env::block_timestamp() + door.config.claim_window_ns);
        door.proof_hash = Some(Base64VecU8(env::sha256(&result.response_hash.0)));
        self.accepted_proofs.insert(proof_key, true);
        request.status = SignalRequestStatus::Verified;
        request.completed_at_ns = Some(env::block_timestamp());
        self.signal_requests.insert(request_id, request.clone());
        self.lifecycle = Lifecycle::DoorOpen;
        self.emit("door_opened", Some(request.door_id), 0);
    }

    fn reject_signal_request(&mut self, request: &mut SignalRequest, reason: &str) {
        request.status = SignalRequestStatus::Rejected;
        request.completed_at_ns = Some(env::block_timestamp());
        request.failure_reason = Some(reason.into());
        self.signal_requests.insert(request.request_id, request.clone());
        self.emit("signal_rejected", Some(request.door_id), 0);
    }

    /// A failed external execution must not leave a Door permanently locked.
    /// Anyone can release the one-request-per-Door lock after the public timeout.
    pub fn cancel_stale_signal_request(&mut self, request_id: u64) {
        let mut request = self.signal_requests.get(&request_id).cloned().unwrap_or_else(|| env::panic_str("AUSPICE[PROOF] unknown request"));
        require!(request.status == SignalRequestStatus::Pending && env::block_timestamp() >= request.requested_at_ns.saturating_add(SIGNAL_REQUEST_TIMEOUT_NS), "AUSPICE[PROOF] request is not stale");
        self.active_door_requests.remove(&request.door_id);
        self.reject_signal_request(&mut request, "request timed out");
    }

    /// Door claim consumes the short-lived approval written solely by
    /// the private OutLayer human-verification callback. It deliberately has
    /// no server-signed pass fallback: every launch keeps Door claims behind
    /// the same Turnstile-in-TEE ceremony as Genesis.
    pub fn claim_signal_door(&mut self, door_id: u16, amount: U128) {
        require!(self.lifecycle == Lifecycle::DoorOpen, "AUSPICE[DOOR] no Door is open");
        let account = env::predecessor_account_id();
        let approval = self.human_approvals.get(&account).cloned().unwrap_or_else(|| env::panic_str("AUSPICE[HUMAN] complete the TEE human check first"));
        require!(approval.kind == "signal_door" && approval.door_id == Some(door_id) && approval.expires_at_ns > env::block_timestamp() && amount.0 > 0 && amount.0 <= approval.max_allocation.0, "AUSPICE[HUMAN] approval is invalid, expired, or exceeds the cap");
        let door = self.door(door_id);
        require!(door.status == DoorStatus::Open && env::block_timestamp() < door.closes_at_ns.unwrap_or(0), "AUSPICE[DOOR] claim window is closed");
        require!(amount.0 <= door.config.user_cap.0, "AUSPICE[DOOR] allocation exceeds cap");
        require!(!door.config.new_humans_only || !self.genesis_claimed.contains_key(&account), "AUSPICE[DOOR] Genesis participants are ineligible");
        require!(self.token.accounts.contains_key(&account), "AUSPICE[STORAGE] register token storage first");
        let door = self.door_mut(door_id);
        require!(door.minted.0 + amount.0 <= door.config.reserve.0, "AUSPICE[DOOR] reserve exhausted");
        door.minted.0 += amount.0;
        self.mint_to(&account, amount.0);
        self.human_approvals.remove(&account);
        self.emit("door_claimed", Some(door_id), amount.0);
    }

    pub fn close_signal_door(&mut self, door_id: u16) {
        let door = self.door_mut(door_id);
        require!(
            door.status == DoorStatus::Open
                && env::block_timestamp() >= door.closes_at_ns.unwrap_or(u64::MAX),
            "AUSPICE[DOOR] Door cannot close yet"
        );
        let remaining = door.config.reserve.0 - door.minted.0;
        door.burned = U128(remaining);
        door.status = if remaining == 0 {
            DoorStatus::ClosedClaimed
        } else {
            DoorStatus::ClosedBurned
        };
        self.permanently_unmintable += remaining;
        self.lifecycle = if self
            .doors
            .iter()
            .any(|door| door.status == DoorStatus::Locked)
        {
            Lifecycle::Live
        } else {
            Lifecycle::Finished
        };
        self.emit("door_closed", Some(door_id), remaining);
    }

    pub fn get_launch_config(&self) -> LaunchConfig {
        self.config.clone()
    }
    pub fn get_lifecycle(&self) -> Lifecycle {
        self.lifecycle
    }
    pub fn get_genesis_status(&self) -> GenesisStatus {
        GenesisStatus {
            lifecycle: self.lifecycle,
            reserve: self.config.genesis_reserve,
            requested: U128(self.total_requested),
            minted: U128(self.minted_supply),
            opens_at_ns: self.config.genesis_open_at_ns,
            closes_at_ns: self.config.genesis_close_at_ns,
        }
    }
    pub fn get_genesis_request(&self, account_id: AccountId) -> U128 {
        U128(self.genesis_requests.get(&account_id).copied().unwrap_or(0))
    }
    pub fn has_settled_genesis(&self, account_id: AccountId) -> bool {
        self.genesis_claimed.contains_key(&account_id)
    }
    pub fn get_door(&self, door_id: u16) -> SignalDoor {
        self.door(door_id).clone()
    }
    pub fn list_doors(&self) -> Vec<SignalDoor> {
        self.doors.iter().cloned().collect()
    }
    pub fn get_signal_request(&self, request_id: u64) -> Option<SignalRequest> {
        self.signal_requests.get(&request_id).cloned()
    }
    pub fn list_signal_requests(&self, from_index: Option<u32>, limit: Option<u32>) -> Vec<SignalRequest> {
        let from = from_index.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(50).min(100) as usize;
        self.signal_request_order.iter().skip(from).take(limit).filter_map(|id| self.signal_requests.get(id).cloned()).collect()
    }
    pub fn get_birth_certificate(&self) -> BirthCertificate {
        BirthCertificate {
            factory_id: self.factory_id.clone(),
            coin_id: env::current_account_id(),
            metadata: self.metadata.clone(),
            config: self.config.clone(),
            lifecycle: self.lifecycle,
            minted_supply: U128(self.minted_supply),
            permanently_unmintable: U128(self.permanently_unmintable),
        }
    }
    pub fn get_refundable_near(&self, account_id: AccountId) -> U128 {
        U128(self.refundable_near.get(&account_id).copied().unwrap_or(0))
    }
    pub fn withdraw_refund(&mut self) -> near_sdk::Promise {
        let account = env::predecessor_account_id();
        let amount = self.refundable_near.remove(&account).unwrap_or(0);
        require!(amount > 0, "AUSPICE[REFUND] no refundable balance");
        self.emit("refund_withdrawn", None, amount);
        near_sdk::Promise::new(account).transfer(near_sdk::NearToken::from_yoctonear(amount))
    }
    pub fn events(&self, limit: Option<u32>) -> Vec<AuspiceEvent> {
        let count = self.events.len() as usize;
        let limit = limit.unwrap_or(64).min(256) as usize;
        self.events
            .iter()
            .skip(count.saturating_sub(limit))
            .cloned()
            .collect()
    }
    pub fn get_verification_bundle(&self) -> VerificationBundle {
        VerificationBundle {
            coin_id: env::current_account_id(),
            lifecycle: self.lifecycle,
            doors: self.list_doors(),
            signal_requests: self.list_signal_requests(Some(0), Some(100)),
            events: self.events(Some(256)),
        }
    }
    pub fn get_supply_status(&self) -> SupplyStatus {
        let committed = self
            .minted_supply
            .checked_add(self.permanently_unmintable)
            .unwrap_or_else(|| env::panic_str("AUSPICE[SUPPLY] accounting overflow"));
        SupplyStatus {
            maximum_mintable_supply: self.config.maximum_mintable_supply,
            minted_supply: U128(self.minted_supply),
            permanently_unmintable: U128(self.permanently_unmintable),
            remaining_mintable_supply: U128(self.config.maximum_mintable_supply.0 - committed),
        }
    }
    pub fn get_streak_status(&self) -> Option<StreakStatus> {
        let config = self.config.streak.clone()?;
        let state = self.streak.clone()?;
        let today_checkins = self.streak_checkins.len().saturating_sub(state.current_day_checkin_start) as u16;
        let today_started_at_ns = config.starts_at_ns.saturating_add(state.next_day_index.saturating_mul(config.day_length_ns));
        Some(StreakStatus { config: config.clone(), state, today_checkins, today_started_at_ns, today_ends_at_ns: today_started_at_ns.saturating_add(config.day_length_ns) })
    }
    pub fn get_protection_status(&self) -> Option<ProtectionStatus> {
        let config = self.config.protection.clone()?;
        Some(ProtectionStatus { currently_vested: U128(self.currently_vested_creator_amount()), creator_claimed: U128(self.creator_vesting_claimed), current_burn_fee_bps: self.current_burn_fee_bps(), config })
    }
    /// A non-reverting public preflight. Unlike a failed transfer, this can
    /// persist an audit event because it never mutates token balances.
    pub fn probe_transfer_protection(&mut self, receiver_id: AccountId, amount: U128) -> ProtectionProbe {
        let sender = env::predecessor_account_id();
        let result = self.transfer_probe(&sender, &receiver_id, amount.0);
        self.protection_probes.push(ProtectionProbeReceipt {
            probe_id: self.protection_probes.len() as u64,
            sender_id: sender,
            receiver_id,
            amount,
            allowed: result.allowed,
            reason: result.reason.clone(),
            burn_fee: result.burn_fee,
            timestamp_ns: env::block_timestamp(),
        });
        self.emit(if result.allowed { "protection_probe_allowed" } else { "protection_probe_blocked" }, None, amount.0);
        result
    }
    pub fn preview_transfer_protection(&self, sender_id: AccountId, receiver_id: AccountId, amount: U128) -> ProtectionProbe {
        self.transfer_probe(&sender_id, &receiver_id, amount.0)
    }
    pub fn list_protection_probes(&self, from_index: Option<u32>, limit: Option<u32>) -> Vec<ProtectionProbeReceipt> {
        let from = from_index.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(50).min(100) as usize;
        self.protection_probes.iter().skip(from).take(limit).cloned().collect()
    }
    pub fn claim_work_mint(&mut self, nonce: String) {
        require!(self.lifecycle == Lifecycle::Live && nonce.len() <= 128, "AUSPICE[WORK] unavailable");
        let config = self.config.work_mint.clone().unwrap_or_else(|| env::panic_str("AUSPICE[WORK] not enabled"));
        let account = env::predecessor_account_id(); require!(self.token.accounts.contains_key(&account), "AUSPICE[STORAGE] register token storage first");
        let proof = hex(&env::sha256(format!("{}:{}:{}", env::current_account_id(), account, nonce).as_bytes()));
        require!(!self.work_nonces.contains_key(&proof) && leading_zero_bits(&env::sha256(proof.as_bytes())) >= config.difficulty_bits, "AUSPICE[WORK] invalid or used proof");
        let prior = self.work_claims.get(&account).copied().unwrap_or(0); let reward = config.user_cap.0.min(config.reserve.0.saturating_sub(self.work_minted));
        require!(reward > 0 && prior.saturating_add(reward) <= config.user_cap.0, "AUSPICE[WORK] cap exhausted");
        self.work_nonces.insert(proof, true); self.work_claims.insert(account.clone(), prior + reward); self.work_minted += reward; self.mint_to(&account, reward); self.emit("work_mint_claimed", None, reward);
    }
    pub fn get_work_mint_status(&self) -> Option<WorkMintStatus> { self.config.work_mint.as_ref().map(|c| WorkMintStatus { reserve: c.reserve, minted: U128(self.work_minted), difficulty_bits: c.difficulty_bits, user_cap: c.user_cap }) }
    pub fn get_work_mint_claim(&self, account_id: AccountId) -> U128 { U128(self.work_claims.get(&account_id).copied().unwrap_or(0)) }
    pub fn settle_circulation(&mut self, account_id: AccountId) { self.settle_decay(&account_id); }
    pub fn get_circulation_status(&self, account_id: AccountId) -> Option<CirculationStatus> {
        let config = self.config.circulation.clone()?;
        let balance = self.token.ft_balance_of(account_id.clone()).0;
        let last_settled_at_ns = self.circulation_touched.get(&account_id).copied().unwrap_or(config.starts_at_ns);
        let now = env::block_timestamp();
        let pending_days = if now < config.starts_at_ns { 0 } else { now.saturating_sub(last_settled_at_ns) / 86_400_000_000_000 };
        let pending_decay = balance.saturating_mul(config.daily_decay_bps as u128).saturating_mul(pending_days as u128) / 10_000;
        Some(CirculationStatus {
            config,
            account_id,
            balance: U128(balance),
            last_settled_at_ns,
            pending_days,
            pending_decay: U128(pending_decay.min(balance)),
            next_settlement_at_ns: last_settled_at_ns.saturating_add(86_400_000_000_000),
        })
    }
    /// Buys from the contract-held curve reserve. This is the only method
    /// that admits NEAR into the market reserve, and no withdrawal route
    /// exists for a creator or factory account.
    #[payable]
    pub fn buy_curve(&mut self, min_tokens_out: U128) {
        self.refresh_lifecycle();
        require!(self.lifecycle == Lifecycle::Live, "AUSPICE[CURVE] market is not live");
        let config = self.config.bonding_curve.clone().unwrap_or_else(|| env::panic_str("AUSPICE[CURVE] not enabled"));
        let buyer = env::predecessor_account_id();
        require!(self.token.accounts.contains_key(&buyer), "AUSPICE[STORAGE] register token storage first");
        let deposit = env::attached_deposit().as_yoctonear();
        require!(deposit > 0 && deposit <= config.max_buy.0, "AUSPICE[MAX_BUY] deposit is outside the immutable curve limit");
        let denominator = config.virtual_near_reserve.0.saturating_add(self.curve_locked_near).saturating_add(deposit);
        let out = self.curve_available_tokens.saturating_mul(deposit) / denominator;
        require!(out > 0 && out >= min_tokens_out.0 && out <= self.curve_available_tokens, "AUSPICE[CURVE] slippage or exhausted reserve");
        self.curve_locked_near = self.curve_locked_near.saturating_add(deposit);
        self.curve_available_tokens -= out;
        self.mint_to(&buyer, out);
        self.emit("curve_buy", None, out);
    }
    /// Burns tokens into the same curve and pays only from the contract-held
    /// market reserve. Genesis tokens may participate, but total available
    /// curve inventory can never exceed the frozen curve reserve.
    pub fn sell_curve(&mut self, token_amount: U128, min_near_out: U128) -> Promise {
        self.refresh_lifecycle();
        require!(self.lifecycle == Lifecycle::Live, "AUSPICE[CURVE] market is not live");
        let config = self.config.bonding_curve.clone().unwrap_or_else(|| env::panic_str("AUSPICE[CURVE] not enabled"));
        let seller = env::predecessor_account_id();
        require!(token_amount.0 > 0 && token_amount.0 <= config.max_sell.0, "AUSPICE[MAX_SELL] amount is outside the immutable curve limit");
        require!(token_amount.0 <= config.token_reserve.0.saturating_sub(self.curve_available_tokens), "AUSPICE[CURVE] curve inventory limit");
        let denominator = self.curve_available_tokens.saturating_add(token_amount.0);
        let theoretical = config.virtual_near_reserve.0.saturating_add(self.curve_locked_near).saturating_mul(token_amount.0) / denominator;
        let near_out = theoretical.min(self.curve_locked_near);
        require!(near_out > 0 && near_out >= min_near_out.0, "AUSPICE[CURVE] slippage or insufficient locked liquidity");
        self.token.internal_withdraw(&seller, token_amount.0);
        self.minted_supply -= token_amount.0;
        self.curve_available_tokens += token_amount.0;
        self.curve_locked_near -= near_out;
        self.emit("curve_sell", None, token_amount.0);
        Promise::new(seller).transfer(NearToken::from_yoctonear(near_out))
    }
    pub fn get_bonding_curve_status(&self) -> Option<BondingCurveStatus> {
        let config = self.config.bonding_curve.as_ref()?;
        Some(BondingCurveStatus {
            token_reserve: config.token_reserve,
            available_tokens: U128(self.curve_available_tokens),
            locked_near: U128(self.curve_locked_near),
            virtual_near_reserve: config.virtual_near_reserve,
            max_buy: config.max_buy,
            max_sell: config.max_sell,
        })
    }
    /// Deliberately retained as a callable, failing proof that deposited curve
    /// liquidity cannot be extracted by a creator, factory, or anyone else.
    pub fn withdraw_curve_liquidity(&mut self) {
        env::panic_str("AUSPICE[LIQUIDITY_LOCK] curve liquidity only leaves through sell_curve");
    }
    pub fn change_curve_config(&mut self) {
        env::panic_str("AUSPICE[IMMUTABLE_RULES] curve configuration is fixed at launch");
    }
    pub fn get_commitment(&self, commitment_id: u16) -> CommitmentState {
        self.commitment(commitment_id).clone()
    }
    pub fn list_commitments(&self) -> Vec<CommitmentState> {
        self.commitments.iter().cloned().collect()
    }

    fn refresh_lifecycle(&mut self) {
        if self.lifecycle == Lifecycle::Draft
            && env::block_timestamp() >= self.config.genesis_open_at_ns
        {
            self.lifecycle = Lifecycle::GenesisOpen;
        }
        if self.lifecycle == Lifecycle::GenesisOpen
            && env::block_timestamp() >= self.config.genesis_close_at_ns
        {
            self.lifecycle = Lifecycle::GenesisSettling;
        }
    }
    fn door(&self, id: u16) -> &SignalDoor {
        self.doors
            .get(id as u32)
            .unwrap_or_else(|| env::panic_str("AUSPICE[DOOR] unknown Door"))
    }
    fn door_mut(&mut self, id: u16) -> &mut SignalDoor {
        self.doors
            .get_mut(id as u32)
            .unwrap_or_else(|| env::panic_str("AUSPICE[DOOR] unknown Door"))
    }
    fn commitment(&self, id: u16) -> &CommitmentState {
        self.commitments.get(id as u32).unwrap_or_else(|| env::panic_str("AUSPICE[PROOFBOUND] unknown commitment"))
    }
    fn commitment_mut(&mut self, id: u16) -> &mut CommitmentState {
        self.commitments.get_mut(id as u32).unwrap_or_else(|| env::panic_str("AUSPICE[PROOFBOUND] unknown commitment"))
    }
    fn credit_refund(&mut self, account: &AccountId, amount: Balance) {
        let existing = self.refundable_near.get(account).copied().unwrap_or(0);
        self.refundable_near
            .insert(account.clone(), existing + amount);
    }
    fn currently_vested_creator_amount(&self) -> Balance {
        let Some(protection) = &self.config.protection else { return 0; };
        let vesting = &protection.creator_vesting;
        let now = env::block_timestamp();
        if now < vesting.cliff_at_ns { return 0; }
        if vesting.ends_at_ns == vesting.cliff_at_ns || now >= vesting.ends_at_ns { return vesting.reserve.0; }
        mul_div(vesting.reserve.0, (now - vesting.cliff_at_ns) as u128, (vesting.ends_at_ns - vesting.cliff_at_ns) as u128)
    }
    fn current_burn_fee_bps(&self) -> u16 {
        let Some(protection) = &self.config.protection else { return 0; };
        if protection.initial_burn_fee_bps == 0 || env::block_timestamp() < protection.fee_decay_start_ns { return protection.initial_burn_fee_bps; }
        let steps = (env::block_timestamp() - protection.fee_decay_start_ns) / protection.fee_decay_interval_ns;
        let reduction = (steps as u128).saturating_mul(protection.fee_decay_bps as u128);
        protection.initial_burn_fee_bps.saturating_sub(reduction.min(u16::MAX as u128) as u16)
    }
    fn enforce_transfer_protection(&mut self, sender: &AccountId, receiver: &AccountId, amount: Balance) -> Balance {
        let Some(protection) = self.config.protection.clone() else { return 0; };
        if let Some(max) = protection.max_transfer { require!(amount <= max.0, "AUSPICE[PROTECTION] MAX_TRANSFER"); }
        if let Some(cooldown) = protection.transfer_cooldown_ns {
            let prior = self.transfer_timestamps.get(sender).copied().unwrap_or(0);
            require!(env::block_timestamp() >= prior.saturating_add(cooldown), "AUSPICE[PROTECTION] TRANSFER_COOLDOWN");
        }
        let fee = mul_div(amount, self.current_burn_fee_bps() as u128, 10_000);
        let received = amount.saturating_sub(fee);
        if let Some(max) = protection.max_wallet {
            if receiver != &self.config.creator_id {
                let balance = self.token.ft_balance_of(receiver.clone()).0;
                require!(balance.saturating_add(received) <= max.0, "AUSPICE[PROTECTION] MAX_WALLET");
            }
        }
        self.transfer_timestamps.insert(sender.clone(), env::block_timestamp());
        fee
    }
    fn transfer_probe(&self, sender: &AccountId, receiver: &AccountId, amount: Balance) -> ProtectionProbe {
        let Some(protection) = &self.config.protection else { return ProtectionProbe { allowed: true, reason: "NO_PROTECTION_RULES".into(), burn_fee: U128(0) }; };
        if let Some(max) = protection.max_transfer { if amount > max.0 { return ProtectionProbe { allowed: false, reason: "MAX_TRANSFER".into(), burn_fee: U128(0) }; } }
        if let Some(cooldown) = protection.transfer_cooldown_ns { if env::block_timestamp() < self.transfer_timestamps.get(sender).copied().unwrap_or(0).saturating_add(cooldown) { return ProtectionProbe { allowed: false, reason: "TRANSFER_COOLDOWN".into(), burn_fee: U128(0) }; } }
        let fee = mul_div(amount, self.current_burn_fee_bps() as u128, 10_000);
        if let Some(max) = protection.max_wallet { if receiver != &self.config.creator_id && self.token.ft_balance_of(receiver.clone()).0.saturating_add(amount - fee) > max.0 { return ProtectionProbe { allowed: false, reason: "MAX_WALLET".into(), burn_fee: U128(fee) }; } }
        ProtectionProbe { allowed: true, reason: "ALLOWED".into(), burn_fee: U128(fee) }
    }
    fn settle_decay(&mut self, account: &AccountId) {
        let Some(config) = self.config.circulation.clone() else { return; }; let now = env::block_timestamp(); if now < config.starts_at_ns { return; }
        let prior = self.circulation_touched.get(account).copied().unwrap_or(config.starts_at_ns); let days = (now.saturating_sub(prior)) / 86_400_000_000_000; if days == 0 { return; }
        let balance = self.token.ft_balance_of(account.clone()).0; let decay = (balance.saturating_mul(config.daily_decay_bps as u128).saturating_mul(days as u128) / 10_000).min(balance);
        if decay > 0 { self.token.internal_withdraw(account, decay); self.minted_supply -= decay; self.permanently_unmintable += decay; self.emit("circulation_decay_burned", None, decay); }
        self.circulation_touched.insert(account.clone(), prior.saturating_add(days * 86_400_000_000_000));
    }
    fn mint_to(&mut self, account: &AccountId, amount: Balance) {
        let updated_supply = self
            .minted_supply
            .checked_add(amount)
            .unwrap_or_else(|| env::panic_str("AUSPICE[SUPPLY] mint overflow"));
        require!(
            updated_supply + self.permanently_unmintable <= self.config.maximum_mintable_supply.0,
            "AUSPICE[SUPPLY] mint exceeds immutable cap"
        );
        self.token.internal_deposit(account, amount);
        self.minted_supply = updated_supply;
    }
    fn price_for(&self, amount: Balance) -> Balance {
        mul_div(
            amount,
            self.config.genesis_price_yocto_near_per_token.0,
            10u128.pow(self.metadata.decimals as u32),
        )
    }
    fn emit(&mut self, kind: &str, door_id: Option<u16>, amount: Balance) {
        self.event_count += 1;
        let event = AuspiceEvent {
            id: self.event_count,
            kind: kind.into(),
            actor_id: env::predecessor_account_id(),
            door_id,
            amount: U128(amount),
            timestamp_ns: env::block_timestamp(),
        };
        self.events.push(event);
        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"auspice\",\"version\":\"1.0.0\",\"event\":\"{}\"}}",
            kind
        ));
    }
    fn assert_live(&self) {
        require!(
            matches!(
                self.lifecycle,
                Lifecycle::Live | Lifecycle::DoorOpen | Lifecycle::Finished
            ),
            "AUSPICE[LIFECYCLE] transfers start after Genesis"
        );
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn leading_zero_bits(bytes: &[u8]) -> u8 { let mut total = 0; for byte in bytes { if *byte == 0 { total += 8; } else { total += byte.leading_zeros() as u8; break; } } total }
fn mul_div(a: u128, b: u128, denominator: u128) -> u128 {
    require!(denominator > 0, "AUSPICE[MATH] zero denominator");
    ((U256::from(a) * U256::from(b)) / U256::from(denominator)).as_u128()
}

#[near]
impl FungibleTokenCore for AuspiceCoin {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        self.assert_live();
        let sender = env::predecessor_account_id();
        self.settle_decay(&sender); self.settle_decay(&receiver_id);
        let fee = self.enforce_transfer_protection(&sender, &receiver_id, amount.0);
        self.token.ft_transfer(receiver_id, U128(amount.0 - fee), memo);
        if fee > 0 {
            self.token.internal_withdraw(&sender, fee);
            self.minted_supply -= fee;
            self.permanently_unmintable += fee;
            self.emit("transfer_fee_burned", None, fee);
        }
    }
    #[payable]
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.assert_live();
        let sender = env::predecessor_account_id();
        self.settle_decay(&sender); self.settle_decay(&receiver_id);
        let fee = self.enforce_transfer_protection(&sender, &receiver_id, amount.0);
        // A fee-bearing transfer-call must hand the receiver only its net
        // amount; the standard resolver can then return that same net amount.
        let result = self.token.ft_transfer_call(receiver_id, U128(amount.0 - fee), memo, msg);
        if fee > 0 {
            self.token.internal_withdraw(&sender, fee);
            self.minted_supply -= fee;
            self.permanently_unmintable += fee;
            self.emit("transfer_fee_burned", None, fee);
        }
        result
    }
    fn ft_total_supply(&self) -> U128 {
        self.token.ft_total_supply()
    }
    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.token.ft_balance_of(account_id)
    }
}
#[near]
impl FungibleTokenResolver for AuspiceCoin {
    #[private]
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        self.token
            .internal_ft_resolve_transfer(&sender_id, receiver_id, amount)
            .0
            .into()
    }
}
#[near]
impl StorageManagement for AuspiceCoin {
    #[payable]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        self.token.storage_deposit(account_id, registration_only)
    }
    #[payable]
    fn storage_withdraw(&mut self, amount: Option<near_sdk::NearToken>) -> StorageBalance {
        self.token.storage_withdraw(amount)
    }
    #[payable]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        self.token.internal_storage_unregister(force).is_some()
    }
    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        self.token.storage_balance_bounds()
    }
    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        self.token.storage_balance_of(account_id)
    }
}
#[near]
impl FungibleTokenMetadataProvider for AuspiceCoin {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, NearToken, PromiseResult};
    use std::panic::{catch_unwind, AssertUnwindSafe};

    fn account(id: &str) -> AccountId {
        id.parse().unwrap()
    }
    fn context(predecessor: &str, timestamp: u64, deposit: NearToken) {
        let mut builder = VMContextBuilder::new();
        builder.current_account_id(account("ship.auspice.testnet"));
        builder.signer_account_id(account(predecessor));
        builder.predecessor_account_id(account(predecessor));
        builder.block_timestamp(timestamp);
        builder.attached_deposit(deposit);
        testing_env!(builder.build());
    }
    fn callback_context(timestamp: u64, result: PromiseResult) {
        let mut builder = VMContextBuilder::new();
        builder.current_account_id(account("ship.auspice.testnet"));
        builder.signer_account_id(account("ship.auspice.testnet"));
        builder.predecessor_account_id(account("ship.auspice.testnet"));
        builder.block_timestamp(timestamp);
        testing_env!(
            builder.build(),
            near_sdk::test_vm_config(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![result]
        );
    }
    fn coin(timestamp: u64) -> AuspiceCoin {
        context("creator.testnet", timestamp, NearToken::from_yoctonear(0));
        AuspiceCoin::new(
            account("auspice.testnet"),
            FungibleTokenMetadata {
                spec: "ft-1.0.0".into(),
                name: "Ship Signal".into(),
                symbol: "SHIP".into(),
                icon: None,
                reference: None,
                reference_hash: None,
                decimals: 0,
            },
            LaunchConfig {
                creator_id: account("creator.testnet"),
                maximum_mintable_supply: U128(100),
                genesis_reserve: U128(100),
                genesis_user_cap: U128(50),
                genesis_price_yocto_near_per_token: U128(1),
                genesis_open_at_ns: 100,
                genesis_close_at_ns: 200,
                human_verifier: HumanVerifierConfig {
                    wasm_url: "https://fastfs.outlayer.ai/human.wasm".into(),
                    wasm_hash: Base64VecU8(vec![9; 32]),
                    wasm_build_target: "wasm32-wasip2".into(),
                    secrets_profile: "production".into(),
                    secrets_account_id: account("secrets.testnet"),
                    expected_hostname: "auspice.example".into(),
                    genesis_action: "auspice_genesis".into(),
                    signal_action: "auspice_signal".into(),
                    approval_lifetime_ns: 1_000,
                },
                outlayer_account_id: account("outlayer.testnet"),
                max_signal_age_ns: 60,
                no_creator_genesis_allocation: true,
                doors: vec![],
                streak: None,
                proofbound: None,
                protection: None,
                work_mint: None,
                circulation: None,
                bonding_curve: None,
                version: 1,
            },
        )
    }
    fn register(coin: &mut AuspiceCoin, who: &str) {
        context(who, 150, NearToken::from_near(1));
        coin.storage_deposit(Some(account(who)), Some(true));
    }

    #[test]
    fn exposes_nep148_metadata_and_nep145_storage() {
        let mut token = coin(150);
        assert_eq!(token.ft_metadata().symbol, "SHIP");
        register(&mut token, "alice.testnet");
        assert!(token.storage_balance_of(account("alice.testnet")).is_some());
    }

    #[test]
    fn transfers_reject_before_live_and_work_after_live() {
        let mut token = coin(150);
        register(&mut token, "alice.testnet");
        register(&mut token, "bob.testnet");
        token.token.internal_deposit(&account("alice.testnet"), 100);
        context("alice.testnet", 150, NearToken::from_yoctonear(1));
        assert!(catch_unwind(AssertUnwindSafe(|| token.ft_transfer(
            account("bob.testnet"),
            U128(10),
            None
        )))
        .is_err());
        token.lifecycle = Lifecycle::Live;
        context("alice.testnet", 250, NearToken::from_yoctonear(1));
        token.ft_transfer(account("bob.testnet"), U128(25), None);
        assert_eq!(token.ft_balance_of(account("bob.testnet")), U128(25));
    }

    fn coin_with_door(timestamp: u64) -> AuspiceCoin {
        let mut token = coin(timestamp);
        token.config.maximum_mintable_supply = U128(150);
        token.config.genesis_reserve = U128(100);
        token.config.doors = vec![DoorConfig {
            door_id: 0,
            template: SignalTemplate::GithubRelease,
            source_id: "github_release:auspice/ship-signal".into(),
            source_url: "https://api.github.com/repos/auspice/ship-signal/releases/latest".into(),
            predicate: SignalPredicate { version: 1, required_tag: Some("v1.0.0".into()), minimum_magnitude_x10: None, expected_winner: None, required_text: None, minimum_rainfall_mm_x10: None },
            verifier_wasm_url: "https://fastfs.outlayer.ai/auspice-verifier.wasm".into(),
            verifier_wasm_hash: Base64VecU8(vec![0; 32]),
            verifier_build_target: "wasm32-wasip2".into(),
            reserve: U128(50),
            user_cap: U128(10),
            claim_window_ns: 20,
            new_humans_only: true,
        }];
        token.doors.push(SignalDoor {
            config: token.config.doors[0].clone(),
            status: DoorStatus::Locked,
            opened_at_ns: None,
            closes_at_ns: None,
            proof_hash: None,
            minted: U128(0),
            burned: U128(0),
        });
        token.lifecycle = Lifecycle::Live;
        token
    }

    fn pending_signal_request(token: &mut AuspiceCoin) -> u64 {
        let request_id = 0;
        token.signal_requests.insert(
            request_id,
            SignalRequest {
                request_id,
                door_id: 0,
                nonce: "request-nonce".into(),
                requester_id: account("alice.testnet"),
                requested_at_ns: 250,
                status: SignalRequestStatus::Pending,
                completed_at_ns: None,
                failure_reason: None,
            },
        );
        request_id
    }

    fn signal_result(coin_id: &str, fetched_at_ns: u64) -> OutlayerSignalResult {
        OutlayerSignalResult {
            request_id: 0,
            request_nonce: "request-nonce".into(),
            coin_id: account(coin_id),
            door_id: 0,
            template: SignalTemplate::GithubRelease,
            source_id: "github_release:auspice/ship-signal".into(),
            source_url: "https://api.github.com/repos/auspice/ship-signal/releases/latest".into(),
            predicate_version: 1,
            predicate_passed: true,
            fetched_at_ns,
            expires_at_ns: 400,
            wasm_hash: Base64VecU8(vec![0; 32]),
            response_hash: Base64VecU8(vec![42; 32]),
        }
    }

    fn set_human_verifier(token: &mut AuspiceCoin) {
        token.config.human_verifier = HumanVerifierConfig {
            wasm_url: "https://fastfs.outlayer.ai/human.wasm".into(),
            wasm_hash: Base64VecU8(vec![9; 32]),
            wasm_build_target: "wasm32-wasip2".into(),
            secrets_profile: "production".into(),
            secrets_account_id: account("secrets.testnet"),
            expected_hostname: "auspice.example".into(),
            genesis_action: "auspice_genesis".into(),
            signal_action: "auspice_signal".into(),
            approval_lifetime_ns: 1_000,
        };
    }

    fn approved_human_result(door_id: u16) -> OutlayerHumanResult {
        OutlayerHumanResult {
            request_id: 0,
            request_nonce: "human-request-nonce".into(),
            coin_id: account("ship.auspice.testnet"),
            account_id: account("carol.testnet"),
            kind: "signal_door".into(),
            door_id: Some(door_id),
            approved: true,
            expires_at_ns: 300,
            max_allocation: U128(10),
            wasm_hash: hex(&[9; 32]),
            captcha_token_hash: "token-hash".into(),
        }
    }

    #[test]
    fn only_a_bound_outlayer_result_opens_a_signal_door() {
        let mut token = coin_with_door(250);
        let request_id = pending_signal_request(&mut token);
        let result =
            near_sdk::serde_json::to_vec(&signal_result("ship.auspice.testnet", 280)).unwrap();
        callback_context(300, PromiseResult::Successful(result));
        token.on_signal_result(request_id);
        assert_eq!(token.get_door(0).status, DoorStatus::Open);
        assert_eq!(token.get_lifecycle(), Lifecycle::DoorOpen);
        assert_eq!(token.events(Some(1))[0].kind, "door_opened");
    }

    #[test]
    fn rejects_stale_or_cross_coin_outlayer_results_without_bricking_the_door() {
        let mut stale = coin_with_door(250);
        pending_signal_request(&mut stale);
        let stale_result =
            near_sdk::serde_json::to_vec(&signal_result("ship.auspice.testnet", 200)).unwrap();
        callback_context(300, PromiseResult::Successful(stale_result));
        stale.on_signal_result(0);
        assert_eq!(stale.get_signal_request(0).unwrap().status, SignalRequestStatus::Rejected);

        let mut wrong_coin = coin_with_door(250);
        pending_signal_request(&mut wrong_coin);
        let wrong_result =
            near_sdk::serde_json::to_vec(&signal_result("other.testnet", 280)).unwrap();
        callback_context(300, PromiseResult::Successful(wrong_result));
        wrong_coin.on_signal_result(0);
        assert_eq!(wrong_coin.get_signal_request(0).unwrap().status, SignalRequestStatus::Rejected);

        let mut missing_hash = coin_with_door(250);
        pending_signal_request(&mut missing_hash);
        let mut result = signal_result("ship.auspice.testnet", 280);
        result.response_hash = Base64VecU8(vec![]);
        callback_context(
            300,
            PromiseResult::Successful(near_sdk::serde_json::to_vec(&result).unwrap()),
        );
        missing_hash.on_signal_result(0);
        assert_eq!(missing_hash.get_signal_request(0).unwrap().status, SignalRequestStatus::Rejected);
    }

    #[test]
    fn releases_a_stuck_signal_request_after_the_public_timeout() {
        let mut token = coin_with_door(250);
        pending_signal_request(&mut token);
        context("watcher.testnet", 250 + SIGNAL_REQUEST_TIMEOUT_NS, NearToken::from_yoctonear(0));
        token.cancel_stale_signal_request(0);
        assert_eq!(token.get_signal_request(0).unwrap().status, SignalRequestStatus::Rejected);
        assert!(!token.active_door_requests.contains_key(&0));
    }

    #[test]
    fn supply_status_never_exceeds_the_immutable_cap() {
        let mut token = coin_with_door(250);
        register(&mut token, "alice.testnet");
        token.mint_to(&account("alice.testnet"), 100);
        let status = token.get_supply_status();
        assert_eq!(status.minted_supply, U128(100));
        assert_eq!(status.remaining_mintable_supply, U128(50));
        assert!(catch_unwind(AssertUnwindSafe(
            || token.mint_to(&account("alice.testnet"), 51)
        ))
        .is_err());
    }

    #[test]
    fn completes_genesis_signal_door_and_unmintable_supply_flow() {
        let mut token = coin_with_door(150);
        token.lifecycle = Lifecycle::GenesisOpen;
        register(&mut token, "alice.testnet");
        register(&mut token, "bob.testnet");
        register(&mut token, "carol.testnet");

        context("alice.testnet", 150, NearToken::from_yoctonear(30));
        token.human_approvals.insert(account("alice.testnet"), HumanApproval { request_id: 1, kind: "genesis".into(), door_id: None, nonce: "alice-genesis".into(), expires_at_ns: 190, max_allocation: U128(30) });
        token.request_genesis_allocation(U128(30));
        context("bob.testnet", 150, NearToken::from_yoctonear(20));
        token.human_approvals.insert(account("bob.testnet"), HumanApproval { request_id: 2, kind: "genesis".into(), door_id: None, nonce: "bob-genesis".into(), expires_at_ns: 190, max_allocation: U128(20) });
        token.request_genesis_allocation(U128(20));
        context("watcher.testnet", 210, NearToken::from_yoctonear(0));
        token.settle_genesis(account("alice.testnet"));
        token.settle_genesis(account("bob.testnet"));
        token.finalize_genesis();
        assert_eq!(token.ft_balance_of(account("alice.testnet")), U128(30));
        assert_eq!(token.get_lifecycle(), Lifecycle::Live);

        pending_signal_request(&mut token);
        let result =
            near_sdk::serde_json::to_vec(&signal_result("ship.auspice.testnet", 250)).unwrap();
        callback_context(260, PromiseResult::Successful(result));
        token.on_signal_result(0);
        token.human_approvals.insert(account("carol.testnet"), HumanApproval { request_id: 3, kind: "signal_door".into(), door_id: Some(0), nonce: "carol-door".into(), expires_at_ns: 280, max_allocation: U128(10) });
        context("carol.testnet", 270, NearToken::from_yoctonear(0));
        token.claim_signal_door(0, U128(10));
        context("watcher.testnet", 281, NearToken::from_yoctonear(0));
        token.close_signal_door(0);
        assert_eq!(token.get_door(0).status, DoorStatus::ClosedBurned);
        assert_eq!(token.get_supply_status().permanently_unmintable, U128(90));
        assert_eq!(token.get_lifecycle(), Lifecycle::Finished);
    }

    #[test]
    fn door_claim_requires_and_consumes_tee_human_approval() {
        let mut token = coin_with_door(250);
        set_human_verifier(&mut token);
        register(&mut token, "carol.testnet");
        let request_id = pending_signal_request(&mut token);
        callback_context(260, PromiseResult::Successful(near_sdk::serde_json::to_vec(&signal_result("ship.auspice.testnet", 250)).unwrap()));
        token.on_signal_result(request_id);
        token.human_requests.insert(0, HumanCheckRequest { request_id: 0, requester_id: account("carol.testnet"), kind: "signal_door".into(), door_id: Some(0), nonce: "human-request-nonce".into(), status: HumanCheckStatus::Pending, requested_at_ns: 265 });
        callback_context(270, PromiseResult::Successful(near_sdk::serde_json::to_vec(&approved_human_result(0)).unwrap()));
        token.on_human_check_result(0);
        assert!(token.get_human_approval(account("carol.testnet")).is_some());
        context("carol.testnet", 279, NearToken::from_yoctonear(0));
        token.claim_signal_door(0, U128(10));
        assert_eq!(token.get_door(0).minted, U128(10));
        assert!(token.get_human_approval(account("carol.testnet")).is_none());
        assert!(catch_unwind(AssertUnwindSafe(|| token.claim_signal_door(0, U128(1)))).is_err());
    }

    #[test]
    fn door_claim_rejects_wrong_door_or_expired_approval() {
        let mut token = coin_with_door(250);
        set_human_verifier(&mut token);
        register(&mut token, "carol.testnet");
        token.human_approvals.insert(account("carol.testnet"), HumanApproval { request_id: 7, kind: "signal_door".into(), door_id: Some(1), nonce: "wrong-door".into(), expires_at_ns: 999, max_allocation: U128(10) });
        context("carol.testnet", 280, NearToken::from_yoctonear(0));
        assert!(catch_unwind(AssertUnwindSafe(|| token.claim_signal_door(0, U128(1)))).is_err());
        token.human_approvals.insert(account("carol.testnet"), HumanApproval { request_id: 8, kind: "signal_door".into(), door_id: Some(0), nonce: "expired".into(), expires_at_ns: 280, max_allocation: U128(10) });
        assert!(catch_unwind(AssertUnwindSafe(|| token.claim_signal_door(0, U128(1)))).is_err());
    }

    #[test]
    fn settles_the_immutable_streak_ritual_and_burns_on_a_miss() {
        let mut token = coin(250);
        token.config.maximum_mintable_supply = U128(150);
        token.config.streak = Some(StreakConfig {
            reserve: U128(50), starts_at_ns: 200, day_length_ns: 10,
            daily_target: 2, daily_participant_cap: 3,
            mint_per_checkin: U128(5), missed_day_burn: U128(7),
        });
        token.streak = Some(StreakState { next_day_index: 0, current_streak: 0, longest_streak: 0, remaining_reserve: U128(50), minted: U128(0), burned: U128(0), current_day_checkin_start: 0 });
        token.lifecycle = Lifecycle::Live;
        register(&mut token, "alice.testnet");
        register(&mut token, "bob.testnet");
        token.token.internal_deposit(&account("alice.testnet"), 1);
        token.token.internal_deposit(&account("bob.testnet"), 1);
        context("alice.testnet", 205, NearToken::from_yoctonear(1));
        token.check_in_streak();
        assert!(catch_unwind(AssertUnwindSafe(|| token.check_in_streak())).is_err());
        context("bob.testnet", 206, NearToken::from_yoctonear(1));
        token.check_in_streak();
        context("watcher.testnet", 210, NearToken::from_yoctonear(0));
        token.settle_streak_day();
        let won = token.get_streak_status().unwrap();
        assert_eq!(won.state.current_streak, 1);
        assert_eq!(won.state.longest_streak, 1);
        assert_eq!(won.state.minted, U128(10));
        assert_eq!(token.ft_balance_of(account("alice.testnet")), U128(6));

        context("watcher.testnet", 220, NearToken::from_yoctonear(0));
        token.settle_streak_day();
        let missed = token.get_streak_status().unwrap();
        assert_eq!(missed.state.current_streak, 0);
        assert_eq!(missed.state.longest_streak, 1);
        assert_eq!(missed.state.burned, U128(7));
        assert_eq!(token.get_supply_status().permanently_unmintable, U128(7));
    }

    #[test]
    fn proofbound_commitment_uses_human_votes_and_burns_no_admin_reserve() {
        let mut token = coin(250);
        let commitment = CommitmentConfig { commitment_id: 0, statement: "Ship the public demo".into(), evidence_requirement: "HTTPS demo or release URL plus SHA-256".into(), evidence_deadline_ns: 260, vote_duration_ns: 20, quorum: 2, pass_bps: 6_000, creator_reward: U128(10), voter_reward: U128(20) };
        token.config.maximum_mintable_supply = U128(130);
        token.config.proofbound = Some(ProofboundConfig { creator_reserve: U128(10), voter_reserve: U128(20), commitments: vec![commitment.clone()] });
        token.commitments.push(CommitmentState { config: commitment, status: CommitmentStatus::AwaitingEvidence, evidence_url: None, evidence_hash: None, vote_opens_at_ns: None, vote_closes_at_ns: None, yes_votes: 0, no_votes: 0, voter_reward_per_yes_vote: U128(0), creator_reward_claimed: false });
        token.lifecycle = Lifecycle::Live;
        register(&mut token, "creator.testnet");
        register(&mut token, "alice.testnet");
        register(&mut token, "bob.testnet");
        token.genesis_claimed.insert(account("alice.testnet"), true);
        token.genesis_claimed.insert(account("bob.testnet"), true);
        context("creator.testnet", 250, NearToken::from_yoctonear(0));
        token.submit_commitment_evidence(0, "https://example.test/demo".into(), Base64VecU8(vec![7; 32]));
        for voter in ["alice.testnet", "bob.testnet"] {
            token.human_approvals.insert(account(voter), HumanApproval { request_id: 1, kind: "commitment_vote".into(), door_id: Some(0), nonce: voter.into(), expires_at_ns: 270, max_allocation: U128(1) });
            context(voter, 255, NearToken::from_yoctonear(0));
            token.vote_commitment(0, true);
        }
        context("watcher.testnet", 270, NearToken::from_yoctonear(0));
        token.settle_commitment(0);
        assert_eq!(token.get_commitment(0).status, CommitmentStatus::Passed);
        assert_eq!(token.get_commitment(0).voter_reward_per_yes_vote, U128(10));
        context("creator.testnet", 271, NearToken::from_yoctonear(0));
        token.claim_creator_commitment_reward(0);
        context("alice.testnet", 271, NearToken::from_yoctonear(0));
        token.claim_commitment_voter_reward(0);
        assert_eq!(token.ft_balance_of(account("creator.testnet")), U128(10));
        assert_eq!(token.ft_balance_of(account("alice.testnet")), U128(10));
    }

    #[test]
    fn fortress_rules_burn_only_downward_fee_enforce_limits_and_vest_creator_tokens() {
        let mut token = coin(250);
        token.config.maximum_mintable_supply = U128(130);
        token.config.protection = Some(LaunchProtectionConfig {
            creator_vesting: CreatorVestingConfig { reserve: U128(30), cliff_at_ns: 260, ends_at_ns: 270 },
            max_wallet: Some(U128(15)), max_transfer: Some(U128(10)), transfer_cooldown_ns: Some(10),
            initial_burn_fee_bps: 1_000, fee_decay_start_ns: 300, fee_decay_interval_ns: 10, fee_decay_bps: 500,
        });
        token.lifecycle = Lifecycle::Live;
        register(&mut token, "creator.testnet");
        register(&mut token, "alice.testnet");
        register(&mut token, "bob.testnet");
        token.token.internal_deposit(&account("alice.testnet"), 20);
        token.minted_supply = 20;
        context("alice.testnet", 250, NearToken::from_yoctonear(1));
        token.ft_transfer(account("bob.testnet"), U128(10), None);
        assert_eq!(token.ft_balance_of(account("bob.testnet")), U128(9));
        assert_eq!(token.get_supply_status().permanently_unmintable, U128(1));
        assert!(catch_unwind(AssertUnwindSafe(|| token.ft_transfer(account("bob.testnet"), U128(1), None))).is_err());
        context("creator.testnet", 265, NearToken::from_yoctonear(0));
        token.claim_creator_vested();
        assert_eq!(token.ft_balance_of(account("creator.testnet")), U128(15));
        assert_eq!(token.get_protection_status().unwrap().current_burn_fee_bps, 1_000);
    }

    #[test]
    fn work_mint_decay_and_public_probe_follow_immutable_rules() {
        let mut token = coin(250);
        token.config.maximum_mintable_supply = U128(120);
        token.config.work_mint = Some(WorkMintConfig { reserve: U128(20), difficulty_bits: 4, user_cap: U128(20) });
        token.config.circulation = Some(CirculationConfig { daily_decay_bps: 100, starts_at_ns: 250 });
        token.lifecycle = Lifecycle::Live;
        register(&mut token, "alice.testnet"); register(&mut token, "bob.testnet");
        token.token.internal_deposit(&account("alice.testnet"), 100); token.minted_supply = 100;
        let nonce = (0u64..).map(|value| value.to_string()).find(|candidate| {
            let proof = hex(&env::sha256(format!("{}:{}:{}", env::current_account_id(), account("alice.testnet"), candidate).as_bytes()));
            leading_zero_bits(&env::sha256(proof.as_bytes())) >= 4
        }).unwrap();
        context("alice.testnet", 250, NearToken::from_yoctonear(0));
        token.claim_work_mint(nonce);
        assert_eq!(token.get_work_mint_claim(account("alice.testnet")), U128(20));
        assert_eq!(token.get_work_mint_status().unwrap().minted, U128(20));
        assert_eq!(token.ft_balance_of(account("alice.testnet")), U128(120));
        context("alice.testnet", 250 + 86_400_000_000_000, NearToken::from_yoctonear(0));
        token.settle_circulation(account("alice.testnet"));
        assert_eq!(token.ft_balance_of(account("alice.testnet")), U128(119));
        let probe = token.probe_transfer_protection(account("bob.testnet"), U128(1));
        assert!(probe.allowed);
        let receipts = token.list_protection_probes(Some(0), Some(10));
        assert_eq!(receipts.len(), 1);
        assert_eq!(receipts[0].sender_id, account("alice.testnet"));
        assert_eq!(receipts[0].receiver_id, account("bob.testnet"));
        assert_eq!(receipts[0].reason, "NO_PROTECTION_RULES");
        context("watcher.testnet", 250 + 2 * 86_400_000_000_000, NearToken::from_yoctonear(0));
        let circulation = token.get_circulation_status(account("alice.testnet")).unwrap();
        assert_eq!(circulation.pending_days, 1);
        assert_eq!(circulation.balance, U128(119));
    }

    #[test]
    fn immutable_curve_locks_liquidity_and_only_pays_sells() {
        let mut token = coin(250);
        token.config.genesis_reserve = U128(50);
        token.config.bonding_curve = Some(BondingCurveConfig {
            token_reserve: U128(50), virtual_near_reserve: U128(10), max_buy: U128(20), max_sell: U128(20),
        });
        token.curve_available_tokens = 50;
        token.lifecycle = Lifecycle::Live;
        register(&mut token, "alice.testnet");
        context("alice.testnet", 250, NearToken::from_yoctonear(10));
        token.buy_curve(U128(20));
        let status = token.get_bonding_curve_status().unwrap();
        assert_eq!(status.locked_near, U128(10));
        assert_eq!(status.available_tokens, U128(25));
        assert_eq!(token.ft_balance_of(account("alice.testnet")), U128(25));
        context("alice.testnet", 251, NearToken::from_yoctonear(0));
        let _ = token.sell_curve(U128(10), U128(5));
        let after = token.get_bonding_curve_status().unwrap();
        assert_eq!(after.locked_near, U128(5));
        assert_eq!(after.available_tokens, U128(35));
        assert!(catch_unwind(AssertUnwindSafe(|| token.withdraw_curve_liquidity())).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| token.change_curve_config())).is_err());
    }

    /// Offline end-to-end rehearsal: TEE approvals are the only authority
    /// for both Genesis and the later real-world Door allocation.
    #[test]
    fn completes_genesis_then_tee_opened_door_lifecycle() {
        let mut token = coin_with_door(150);
        set_human_verifier(&mut token);
        token.lifecycle = Lifecycle::GenesisOpen;
        register(&mut token, "alice.testnet");
        register(&mut token, "carol.testnet");
        token.human_approvals.insert(account("alice.testnet"), HumanApproval { request_id: 1, kind: "genesis".into(), door_id: None, nonce: "genesis-approval".into(), expires_at_ns: 190, max_allocation: U128(30) });
        context("alice.testnet", 150, NearToken::from_near(1));
        token.request_genesis_allocation(U128(30));
        assert!(token.get_human_approval(account("alice.testnet")).is_none());
        context("watcher.testnet", 210, NearToken::from_yoctonear(0));
        token.settle_genesis(account("alice.testnet"));
        token.finalize_genesis();
        assert_eq!(token.get_lifecycle(), Lifecycle::Live);
        pending_signal_request(&mut token);
        callback_context(260, PromiseResult::Successful(near_sdk::serde_json::to_vec(&signal_result("ship.auspice.testnet", 250)).unwrap()));
        token.on_signal_result(0);
        token.human_approvals.insert(account("carol.testnet"), HumanApproval { request_id: 2, kind: "signal_door".into(), door_id: Some(0), nonce: "door-approval".into(), expires_at_ns: 279, max_allocation: U128(10) });
        context("carol.testnet", 270, NearToken::from_yoctonear(0));
        token.claim_signal_door(0, U128(10));
        context("watcher.testnet", 281, NearToken::from_yoctonear(0));
        token.close_signal_door(0);
        assert_eq!(token.get_lifecycle(), Lifecycle::Finished);
        assert_eq!(token.get_supply_status().minted_supply, U128(40));
        assert_eq!(token.get_supply_status().permanently_unmintable, U128(110));
    }
}
