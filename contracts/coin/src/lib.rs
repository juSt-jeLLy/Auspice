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
    env, near, require, AccountId, BorshStorageKey, Gas, PanicOnDefault, Promise, PromiseOrValue,
};
use primitive_types::U256;

type Balance = u128;
const SIGNAL_REQUEST_TIMEOUT_NS: u64 = 600_000_000_000;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Token,
    GenesisRequests,
    GenesisClaimed,
    UsedPasses,
    AcceptedProofs,
    SignalRequests,
    SignalRequestOrder,
    ActiveDoorRequests,
    GenesisPaid,
    RefundableNear,
    Doors,
    Events,
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
    pub human_pass_public_key: Base64VecU8,
    pub outlayer_account_id: AccountId,
    pub max_signal_age_ns: u64,
    pub no_creator_genesis_allocation: bool,
    pub doors: Vec<DoorConfig>,
    pub version: u16,
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
#[derive(Clone)]
pub struct HumanPass {
    pub kind: String,
    pub coin_id: AccountId,
    pub door_id: Option<u16>,
    pub account_id: AccountId,
    pub nonce: String,
    pub expires_at_ns: u64,
    pub max_allocation: U128,
    pub signature: Base64VecU8,
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
    used_passes: LookupMap<String, bool>,
    accepted_proofs: LookupMap<String, bool>,
    signal_requests: LookupMap<u64, SignalRequest>,
    signal_request_order: Vector<u64>,
    active_door_requests: LookupMap<u16, u64>,
    doors: Vector<SignalDoor>,
    events: Vector<AuspiceEvent>,
    total_requested: Balance,
    minted_supply: Balance,
    permanently_unmintable: Balance,
    event_count: u64,
    next_signal_request_id: u64,
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
        require!(config.human_pass_public_key.0.len() == 32, "AUSPICE[CONFIG] expected 32-byte human-pass key");
        metadata.assert_valid();
        let door_total: Balance = config.doors.iter().map(|d| d.reserve.0).sum();
        require!(
            config.genesis_reserve.0 + door_total == config.maximum_mintable_supply.0,
            "AUSPICE[CONFIG] reserves must equal maximum supply"
        );
        for (index, door) in config.doors.iter().enumerate() {
            require!(door.door_id == index as u16 && door.user_cap.0 > 0 && door.reserve.0 > 0 && door.predicate.version > 0, "AUSPICE[CONFIG] invalid Door configuration");
            require!(door.verifier_wasm_hash.0.len() == 32 && door.verifier_wasm_url.starts_with("https://") && matches!(door.verifier_build_target.as_str(), "wasm32-wasip1" | "wasm32-wasip2"), "AUSPICE[CONFIG] Door must pin a HTTPS WASI verifier");
            match door.template {
                SignalTemplate::GithubRelease => require!(door.source_id.starts_with("github_release:") && door.source_url.starts_with("https://api.github.com/repos/"), "AUSPICE[CONFIG] invalid GitHub release source"),
                _ => env::panic_str("AUSPICE[CONFIG] this Signal template is not enabled yet"),
            }
        }
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
            used_passes: LookupMap::new(StorageKey::UsedPasses),
            accepted_proofs: LookupMap::new(StorageKey::AcceptedProofs),
            signal_requests: LookupMap::new(StorageKey::SignalRequests),
            signal_request_order: Vector::new(StorageKey::SignalRequestOrder),
            active_door_requests: LookupMap::new(StorageKey::ActiveDoorRequests),
            doors,
            events: Vector::new(StorageKey::Events),
            total_requested: 0,
            minted_supply: 0,
            permanently_unmintable: 0,
            event_count: 0,
            next_signal_request_id: 0,
        }
    }

    #[payable]
    pub fn request_genesis_allocation(&mut self, pass: HumanPass, requested_amount: U128) {
        self.refresh_lifecycle();
        require!(
            self.lifecycle == Lifecycle::GenesisOpen,
            "AUSPICE[GENESIS] Genesis is not open"
        );
        require!(
            requested_amount.0 > 0 && requested_amount.0 <= self.config.genesis_user_cap.0,
            "AUSPICE[GENESIS] allocation exceeds cap"
        );
        require!(
            !self.config.no_creator_genesis_allocation
                || env::predecessor_account_id() != self.config.creator_id,
            "AUSPICE[GENESIS] creator allocation is disabled"
        );
        self.verify_pass(&pass, "genesis", None, requested_amount.0);
        let account = env::predecessor_account_id();
        require!(
            self.token.accounts.contains_key(&account),
            "AUSPICE[STORAGE] register token storage first"
        );
        let prior = self.genesis_requests.get(&account).copied().unwrap_or(0);
        require!(
            prior + requested_amount.0 <= self.config.genesis_user_cap.0,
            "AUSPICE[GENESIS] cumulative cap exceeded"
        );
        self.genesis_requests
            .insert(account, prior + requested_amount.0);
        let attached = env::attached_deposit().as_yoctonear();
        let required = self.price_for(requested_amount.0);
        require!(
            attached >= required,
            "AUSPICE[GENESIS] attached NEAR is below the fixed allocation price"
        );
        let account = env::predecessor_account_id();
        let paid = self.genesis_paid.get(&account).copied().unwrap_or(0);
        self.genesis_paid.insert(account.clone(), paid + required);
        if attached > required {
            self.credit_refund(&account, attached - required);
        }
        self.total_requested += requested_amount.0;
        self.consume(pass.nonce);
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

    pub fn claim_signal_door(&mut self, door_id: u16, pass: HumanPass, amount: U128) {
        require!(
            self.lifecycle == Lifecycle::DoorOpen,
            "AUSPICE[DOOR] no Door is open"
        );
        let account = env::predecessor_account_id();
        let door = self.door(door_id);
        require!(
            door.status == DoorStatus::Open
                && env::block_timestamp() < door.closes_at_ns.unwrap_or(0),
            "AUSPICE[DOOR] claim window is closed"
        );
        require!(
            amount.0 > 0 && amount.0 <= door.config.user_cap.0,
            "AUSPICE[DOOR] allocation exceeds cap"
        );
        require!(
            !door.config.new_humans_only || !self.genesis_claimed.contains_key(&account),
            "AUSPICE[DOOR] Genesis participants are ineligible"
        );
        require!(
            self.token.accounts.contains_key(&account),
            "AUSPICE[STORAGE] register token storage first"
        );
        self.verify_pass(&pass, "signal_door", Some(door_id), amount.0);
        let door = self.door_mut(door_id);
        require!(
            door.minted.0 + amount.0 <= door.config.reserve.0,
            "AUSPICE[DOOR] reserve exhausted"
        );
        door.minted.0 += amount.0;
        self.mint_to(&account, amount.0);
        self.consume(pass.nonce);
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
    pub fn is_pass_consumed(&self, nonce: String) -> bool {
        self.used_passes.contains_key(&nonce)
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
    fn consume(&mut self, nonce: String) {
        self.used_passes.insert(nonce, true);
    }
    fn credit_refund(&mut self, account: &AccountId, amount: Balance) {
        let existing = self.refundable_near.get(account).copied().unwrap_or(0);
        self.refundable_near
            .insert(account.clone(), existing + amount);
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
    fn verify_pass(&self, pass: &HumanPass, kind: &str, door_id: Option<u16>, amount: Balance) {
        require!(
            pass.kind == kind
                && pass.coin_id == env::current_account_id()
                && pass.door_id == door_id
                && pass.account_id == env::predecessor_account_id(),
            "AUSPICE[PASS] pass binding mismatch"
        );
        require!(
            pass.expires_at_ns > env::block_timestamp()
                && amount <= pass.max_allocation.0
                && (16..=128).contains(&pass.nonce.len())
                && !self.used_passes.contains_key(&pass.nonce),
            "AUSPICE[PASS] invalid, expired, or used pass"
        );
        let message = format!(
            "auspice:v1:{}:{}:{}:{}:{}:{}:{}",
            pass.kind,
            pass.coin_id,
            pass.door_id.map(|id| id.to_string()).unwrap_or_default(),
            pass.account_id,
            pass.nonce,
            pass.expires_at_ns,
            pass.max_allocation.0
        );
        let signature: [u8; 64] = pass
            .signature
            .0
            .as_slice()
            .try_into()
            .unwrap_or_else(|_| env::panic_str("AUSPICE[PASS] malformed signature"));
        let key: [u8; 32] = self
            .config
            .human_pass_public_key
            .0
            .as_slice()
            .try_into()
            .unwrap();
        require!(
            env::ed25519_verify(&signature, message.as_bytes(), &key),
            "AUSPICE[PASS] signature failed"
        );
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn mul_div(a: u128, b: u128, denominator: u128) -> u128 {
    require!(denominator > 0, "AUSPICE[MATH] zero denominator");
    ((U256::from(a) * U256::from(b)) / U256::from(denominator)).as_u128()
}

#[near]
impl FungibleTokenCore for AuspiceCoin {
    #[payable]
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        self.assert_live();
        self.token.ft_transfer(receiver_id, amount, memo)
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
        self.token.ft_transfer_call(receiver_id, amount, memo, msg)
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
    use ed25519_dalek::{Signer, SigningKey};
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
                human_pass_public_key: Base64VecU8(
                    SigningKey::from_bytes(&[7; 32])
                        .verifying_key()
                        .to_bytes()
                        .to_vec(),
                ),
                outlayer_account_id: account("outlayer.testnet"),
                max_signal_age_ns: 60,
                no_creator_genesis_allocation: true,
                doors: vec![],
                version: 1,
            },
        )
    }
    fn register(coin: &mut AuspiceCoin, who: &str) {
        context(who, 150, NearToken::from_near(1));
        coin.storage_deposit(Some(account(who)), Some(true));
    }

    fn genesis_pass(who: &str, amount: u128) -> HumanPass {
        let nonce = format!("a-unique-test-nonce-{who}");
        let expires_at_ns = 190;
        let message = format!(
            "auspice:v1:genesis:ship.auspice.testnet::{}:{}:{}:{}",
            who, nonce, expires_at_ns, amount
        );
        let signature = SigningKey::from_bytes(&[7; 32]).sign(message.as_bytes());
        HumanPass {
            kind: "genesis".into(),
            coin_id: account("ship.auspice.testnet"),
            door_id: None,
            account_id: account(who),
            nonce,
            expires_at_ns,
            max_allocation: U128(amount),
            signature: Base64VecU8(signature.to_bytes().to_vec()),
        }
    }

    fn door_pass(who: &str, door_id: u16, amount: u128) -> HumanPass {
        let nonce = format!("a-unique-door-pass-{who}-{door_id}");
        let expires_at_ns = 280;
        let message = format!(
            "auspice:v1:signal_door:ship.auspice.testnet:{door_id}:{who}:{nonce}:{expires_at_ns}:{amount}"
        );
        let signature = SigningKey::from_bytes(&[7; 32]).sign(message.as_bytes());
        HumanPass {
            kind: "signal_door".into(),
            coin_id: account("ship.auspice.testnet"),
            door_id: Some(door_id),
            account_id: account(who),
            nonce,
            expires_at_ns,
            max_allocation: U128(amount),
            signature: Base64VecU8(signature.to_bytes().to_vec()),
        }
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

    #[test]
    fn credits_genesis_overpayment_as_a_withdrawable_refund() {
        let mut token = coin(150);
        register(&mut token, "alice.testnet");
        context("alice.testnet", 150, NearToken::from_yoctonear(15));
        token.request_genesis_allocation(genesis_pass("alice.testnet", 10), U128(10));
        assert_eq!(token.get_refundable_near(account("alice.testnet")), U128(5));
    }

    #[test]
    fn rejects_genesis_before_open_and_replayed_passes() {
        let mut token = coin(50);
        register(&mut token, "alice.testnet");
        context("alice.testnet", 50, NearToken::from_yoctonear(10));
        assert!(catch_unwind(AssertUnwindSafe(
            || token.request_genesis_allocation(genesis_pass("alice.testnet", 10), U128(10))
        ))
        .is_err());

        let mut token = coin(150);
        register(&mut token, "alice.testnet");
        let pass = genesis_pass("alice.testnet", 10);
        context("alice.testnet", 150, NearToken::from_yoctonear(10));
        token.request_genesis_allocation(pass.clone(), U128(10));
        context("alice.testnet", 150, NearToken::from_yoctonear(10));
        assert!(catch_unwind(AssertUnwindSafe(
            || token.request_genesis_allocation(pass, U128(10))
        ))
        .is_err());
    }

    #[test]
    fn rejects_a_pass_bound_to_another_wallet() {
        let mut token = coin(150);
        register(&mut token, "alice.testnet");
        register(&mut token, "bob.testnet");
        context("bob.testnet", 150, NearToken::from_yoctonear(10));
        assert!(catch_unwind(AssertUnwindSafe(
            || token.request_genesis_allocation(genesis_pass("alice.testnet", 10), U128(10))
        ))
        .is_err());
    }

    #[test]
    fn rejects_a_creator_genesis_request_when_the_launch_disables_it() {
        let mut token = coin(150);
        register(&mut token, "creator.testnet");
        context("creator.testnet", 150, NearToken::from_yoctonear(10));
        assert!(catch_unwind(AssertUnwindSafe(
            || token.request_genesis_allocation(genesis_pass("creator.testnet", 10), U128(10))
        ))
        .is_err());
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
            predicate: SignalPredicate { version: 1, required_tag: Some("v1.0.0".into()), minimum_magnitude_x10: None, expected_winner: None, required_text: None },
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
        token.request_genesis_allocation(genesis_pass("alice.testnet", 30), U128(30));
        context("bob.testnet", 150, NearToken::from_yoctonear(20));
        token.request_genesis_allocation(genesis_pass("bob.testnet", 20), U128(20));
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
        let pass = door_pass("carol.testnet", 0, 10);
        context("carol.testnet", 270, NearToken::from_yoctonear(0));
        token.claim_signal_door(0, pass, U128(10));
        context("watcher.testnet", 281, NearToken::from_yoctonear(0));
        token.close_signal_door(0);
        assert_eq!(token.get_door(0).status, DoorStatus::ClosedBurned);
        assert_eq!(token.get_supply_status().permanently_unmintable, U128(90));
        assert_eq!(token.get_lifecycle(), Lifecycle::Finished);
    }
}
