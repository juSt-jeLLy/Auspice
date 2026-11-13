//! Auspice factory contract.
//!
//! The factory will create independently inspectable coin accounts. Its public
//! interface is intentionally small: launch configuration is fixed at creation
//! and each coin owns its own NEP-141 state.

use near_sdk::{near, AccountId, PanicOnDefault};

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct AuspiceFactory {
    owner_id: AccountId,
}

#[near]
impl AuspiceFactory {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self { owner_id }
    }

    pub fn get_owner_id(&self) -> AccountId {
        self.owner_id.clone()
    }
}
