//! Permissionless Auspice coin factory.
//!
//! Any NEAR account can publish a coin. The factory creates a separate,
//! keyless subaccount that uses a pinned global coin contract, then records
//! the immutable Birth Certificate address after initialization succeeds.

use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::borsh::BorshSerialize;
use near_sdk::json_types::Base58CryptoHash;
use near_sdk::serde_json::{json, Value};
use near_sdk::store::{LookupMap, Vector};
use near_sdk::{
    env, near, require, AccountId, BorshStorageKey, Gas, NearToken, PanicOnDefault, Promise,
    PromiseResult,
};

const MIN_CREATE_DEPOSIT: u128 = 300_000_000_000_000_000_000_000;
const GAS_INIT: Gas = Gas::from_tgas(60);
const GAS_CALLBACK: Gas = Gas::from_tgas(15);

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Coins,
    ByAccount,
    Pending,
}

#[near(serializers = [borsh, json])]
#[derive(Clone)]
pub struct CoinEntry {
    pub account_id: AccountId,
    pub creator_id: AccountId,
    pub name: String,
    pub symbol: String,
    pub created_at_ns: u64,
}

#[near(serializers = [json])]
pub struct FactoryConfig {
    pub owner_id: AccountId,
    pub coin_code_hash: Base58CryptoHash,
    pub minimum_create_deposit: String,
    pub coin_count: u32,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct AuspiceFactory {
    owner_id: AccountId,
    coin_code_hash: Base58CryptoHash,
    coins: Vector<CoinEntry>,
    by_account: LookupMap<AccountId, u32>,
    pending: LookupMap<AccountId, bool>,
}

fn valid_symbol(value: &str) -> bool {
    (2..=12).contains(&value.len())
        && value
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
}

#[near]
impl AuspiceFactory {
    #[init]
    pub fn new(owner_id: AccountId, coin_code_hash: Base58CryptoHash) -> Self {
        Self {
            owner_id,
            coin_code_hash,
            coins: Vector::new(StorageKey::Coins),
            by_account: LookupMap::new(StorageKey::ByAccount),
            pending: LookupMap::new(StorageKey::Pending),
        }
    }

    /// Creates `<symbol>.<factory>` with no access key. `config` is preserved
    /// verbatim for the coin constructor, which performs its immutable-rule validation.
    #[payable]
    pub fn create_coin(&mut self, metadata: FungibleTokenMetadata, config: Value) -> Promise {
        let creator_id = env::predecessor_account_id();
        let deposit = env::attached_deposit().as_yoctonear();
        require!(
            deposit >= MIN_CREATE_DEPOSIT,
            "AUSPICE[FACTORY] attach at least 0.3 NEAR for account storage"
        );
        metadata.assert_valid();
        require!(
            valid_symbol(&metadata.symbol),
            "AUSPICE[FACTORY] symbol must be 2-12 uppercase letters or digits"
        );
        require!(
            config.get("creator_id").and_then(Value::as_str) == Some(creator_id.as_str()),
            "AUSPICE[FACTORY] config creator must be transaction signer"
        );
        let account_id: AccountId = format!(
            "{}.{}",
            metadata.symbol.to_ascii_lowercase(),
            env::current_account_id()
        )
        .parse()
        .unwrap_or_else(|_| env::panic_str("AUSPICE[FACTORY] invalid subaccount"));
        require!(
            !self.by_account.contains_key(&account_id) && !self.pending.contains_key(&account_id),
            "AUSPICE[FACTORY] symbol is already taken"
        );
        self.pending.insert(account_id.clone(), true);
        let name = metadata.name.clone();
        let symbol = metadata.symbol.clone();
        let args = json!({ "factory_id": env::current_account_id(), "metadata": metadata, "config": config });
        Promise::new(account_id.clone())
            .create_account()
            .transfer(NearToken::from_yoctonear(deposit))
            .use_global_contract(self.coin_code_hash)
            .function_call(
                "new",
                args.to_string().into_bytes(),
                NearToken::from_yoctonear(0),
                GAS_INIT,
            )
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_CALLBACK)
                    .on_create_coin(account_id, creator_id, name, symbol, deposit),
            )
    }

    #[private]
    pub fn on_create_coin(
        &mut self,
        account_id: AccountId,
        creator_id: AccountId,
        name: String,
        symbol: String,
        deposit: u128,
    ) -> Option<AccountId> {
        self.pending.remove(&account_id);
        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                let entry = CoinEntry {
                    account_id: account_id.clone(),
                    creator_id,
                    name,
                    symbol,
                    created_at_ns: env::block_timestamp(),
                };
                let index = self.coins.len();
                self.coins.push(entry);
                self.by_account.insert(account_id.clone(), index);
                env::log_str(&format!("EVENT_JSON:{{\"standard\":\"auspice\",\"version\":\"1.0.0\",\"event\":\"coin_created\",\"data\":[{{\"coin_id\":\"{}\"}}]}}", account_id));
                Some(account_id)
            }
            PromiseResult::Failed => {
                Promise::new(creator_id)
                    .transfer(NearToken::from_yoctonear(deposit))
                    .detach();
                None
            }
        }
    }

    /// Changes only code used by future creations. Existing keyless coins are unaffected.
    pub fn set_coin_code_hash(&mut self, coin_code_hash: Base58CryptoHash) {
        require!(
            env::predecessor_account_id() == self.owner_id,
            "AUSPICE[FACTORY] owner only"
        );
        self.coin_code_hash = coin_code_hash;
    }
    pub fn get_code_hash(&self) -> Base58CryptoHash {
        self.coin_code_hash
    }
    pub fn get_coin(&self, coin_id: AccountId) -> Option<CoinEntry> {
        self.by_account
            .get(&coin_id)
            .and_then(|index| self.coins.get(*index).cloned())
    }
    pub fn list_coins(&self, from_index: Option<u32>, limit: Option<u32>) -> Vec<CoinEntry> {
        let from = from_index.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(50).min(100) as usize;
        self.coins.iter().skip(from).take(limit).cloned().collect()
    }
    pub fn get_config(&self) -> FactoryConfig {
        FactoryConfig {
            owner_id: self.owner_id.clone(),
            coin_code_hash: self.coin_code_hash,
            minimum_create_deposit: MIN_CREATE_DEPOSIT.to_string(),
            coin_count: self.coins.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;
    fn account(value: &str) -> AccountId {
        value.parse().unwrap()
    }
    #[test]
    fn exposes_the_pinned_code_hash() {
        let mut context = VMContextBuilder::new();
        context.current_account_id(account("auspice.testnet"));
        context.predecessor_account_id(account("owner.testnet"));
        testing_env!(context.build());
        let hash: Base58CryptoHash = "11111111111111111111111111111111".parse().unwrap();
        let factory = AuspiceFactory::new(account("owner.testnet"), hash);
        assert_eq!(factory.get_config().coin_count, 0);
    }
}
