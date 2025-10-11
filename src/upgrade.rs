use crate::*;

#[near]
impl Contract {
    #[private]
    #[init(ignore_state)]
    pub fn migrate_state() -> Self {
        env::state_read().unwrap()
    }

    /// Returns semver of this contract.
    pub fn get_version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}

// Note: Low-level upgrade functionality has been removed for near-sdk 5.x compatibility
// Contract upgrades should be handled through standard deployment mechanisms
