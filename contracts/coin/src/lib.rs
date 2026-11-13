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
use near_sdk::store::{LookupMap, Vector};
use near_sdk::{env, near, require, AccountId, BorshStorageKey, PanicOnDefault, PromiseOrValue};

type Balance = u128;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Token,
    GenesisRequests,
    GenesisClaimed,
    UsedPasses,
    Doors,
}

#[near(serializers = [borsh, json])]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lifecycle {
    Draft,
    GenesisOpen,
    GenesisSettling,
    Live,
    DoorOpen,
    Finished,
}

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct DoorConfig {
    pub door_id: u16,
    pub source_id: String,
    pub source_url: String,
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
    pub genesis_open_at_ns: u64,
    pub genesis_close_at_ns: u64,
    pub human_pass_public_key: Base64VecU8,
    pub signal_public_key: Base64VecU8,
    pub tee_wasm_hash: Base64VecU8,
    pub doors: Vec<DoorConfig>,
    pub version: u16,
}

#[near(serializers = [borsh, json])]
#[derive(Clone, Copy, PartialEq, Eq)]
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
pub struct SignalProof {
    pub coin_id: AccountId,
    pub door_id: u16,
    pub source_id: String,
    pub predicate_passed: bool,
    pub expires_at_ns: u64,
    pub wasm_hash: Base64VecU8,
    pub response_hash: Base64VecU8,
    pub signature: Base64VecU8,
}

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
    used_passes: LookupMap<String, bool>,
    doors: Vector<SignalDoor>,
    total_requested: Balance,
    minted_supply: Balance,
    permanently_unmintable: Balance,
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
            config.human_pass_public_key.0.len() == 32
                && config.signal_public_key.0.len() == 32
                && config.tee_wasm_hash.0.len() == 32,
            "AUSPICE[CONFIG] expected 32-byte keys and WASM hash"
        );
        metadata.assert_valid();
        let door_total: Balance = config.doors.iter().map(|d| d.reserve.0).sum();
        require!(
            config.genesis_reserve.0 + door_total == config.maximum_mintable_supply.0,
            "AUSPICE[CONFIG] reserves must equal maximum supply"
        );
        for (index, door) in config.doors.iter().enumerate() {
            require!(
                door.door_id == index as u16 && door.user_cap.0 > 0 && door.reserve.0 > 0,
                "AUSPICE[CONFIG] invalid Door configuration"
            );
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
            used_passes: LookupMap::new(StorageKey::UsedPasses),
            doors,
            total_requested: 0,
            minted_supply: 0,
            permanently_unmintable: 0,
        }
    }

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
        self.total_requested += requested_amount.0;
        self.consume(pass.nonce);
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
        self.token.internal_deposit(&account_id, allocation);
        self.minted_supply += allocation;
        self.genesis_claimed.insert(account_id, true);
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
    }

    pub fn open_signal_door(&mut self, door_id: u16, proof: SignalProof) {
        require!(
            self.lifecycle == Lifecycle::Live,
            "AUSPICE[DOOR] Genesis is not live"
        );
        self.verify_proof(&proof, door_id);
        let door = self.door_mut(door_id);
        require!(
            door.status == DoorStatus::Locked,
            "AUSPICE[DOOR] Door already opened"
        );
        door.status = DoorStatus::Open;
        door.opened_at_ns = Some(env::block_timestamp());
        door.closes_at_ns = Some(env::block_timestamp() + door.config.claim_window_ns);
        door.proof_hash = Some(Base64VecU8(env::sha256(&proof.response_hash.0)));
        self.lifecycle = Lifecycle::DoorOpen;
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
        self.token.internal_deposit(&account, amount.0);
        self.minted_supply += amount.0;
        self.consume(pass.nonce);
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
        self.lifecycle = Lifecycle::Finished;
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
    fn verify_proof(&self, proof: &SignalProof, door_id: u16) {
        require!(
            proof.coin_id == env::current_account_id()
                && proof.door_id == door_id
                && proof.predicate_passed
                && proof.expires_at_ns > env::block_timestamp()
                && proof.wasm_hash.0 == self.config.tee_wasm_hash.0
                && proof.source_id == self.door(door_id).config.source_id,
            "AUSPICE[PROOF] proof binding mismatch"
        );
        let message = format!(
            "auspice:signal:v1:{}:{}:{}:{}",
            proof.coin_id, proof.door_id, proof.source_id, proof.expires_at_ns
        );
        let signature: [u8; 64] = proof
            .signature
            .0
            .as_slice()
            .try_into()
            .unwrap_or_else(|_| env::panic_str("AUSPICE[PROOF] malformed signature"));
        let key: [u8; 32] = self
            .config
            .signal_public_key
            .0
            .as_slice()
            .try_into()
            .unwrap();
        require!(
            env::ed25519_verify(&signature, message.as_bytes(), &key),
            "AUSPICE[PROOF] signature failed"
        );
    }
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
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, NearToken};
    use std::panic::{catch_unwind, AssertUnwindSafe};

    fn account(id: &str) -> AccountId {
        id.parse().unwrap()
    }
    fn context(predecessor: &str, timestamp: u64, deposit: NearToken) {
        let mut builder = VMContextBuilder::new();
        builder.current_account_id(account("rain.auspice.testnet"));
        builder.signer_account_id(account(predecessor));
        builder.predecessor_account_id(account(predecessor));
        builder.block_timestamp(timestamp);
        builder.attached_deposit(deposit);
        testing_env!(builder.build());
    }
    fn coin(timestamp: u64) -> AuspiceCoin {
        context("creator.testnet", timestamp, NearToken::from_yoctonear(0));
        AuspiceCoin::new(
            account("auspice.testnet"),
            FungibleTokenMetadata {
                spec: "ft-1.0.0".into(),
                name: "Raincheck".into(),
                symbol: "RAIN".into(),
                icon: None,
                reference: None,
                reference_hash: None,
                decimals: 18,
            },
            LaunchConfig {
                creator_id: account("creator.testnet"),
                maximum_mintable_supply: U128(100),
                genesis_reserve: U128(100),
                genesis_user_cap: U128(50),
                genesis_open_at_ns: 100,
                genesis_close_at_ns: 200,
                human_pass_public_key: Base64VecU8(vec![0; 32]),
                signal_public_key: Base64VecU8(vec![0; 32]),
                tee_wasm_hash: Base64VecU8(vec![0; 32]),
                doors: vec![],
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
        assert_eq!(token.ft_metadata().symbol, "RAIN");
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
}
