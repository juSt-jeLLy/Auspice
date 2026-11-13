//! Auspice coin contract.
//!
//! This contract will combine standard NEP-141 / NEP-145 / NEP-148 behavior
//! with immutable Genesis and Signal Door settlement rules. The first complete
//! implementation adds the standard token state before mint paths are exposed.

use near_sdk::{near, AccountId, PanicOnDefault};

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct AuspiceCoin {
    factory_id: AccountId,
    creator_id: AccountId,
}

#[near]
impl AuspiceCoin {
    #[init]
    pub fn new(factory_id: AccountId, creator_id: AccountId) -> Self {
        Self {
            factory_id,
            creator_id,
        }
    }

    pub fn get_factory_id(&self) -> AccountId {
        self.factory_id.clone()
    }

    pub fn get_creator_id(&self) -> AccountId {
        self.creator_id.clone()
    }
}
