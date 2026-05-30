//! Nonce tracking for replay prevention in the credence bond contract.

use credence_errors::ContractError;
use soroban_sdk::panic_with_error;
use soroban_sdk::{Address, Env};

use crate::DataKey;

/// Safety buffer added on top of the nonce TTL.
const MIN_NONCE_TTL: u32 = 518_400;

/// Safety buffer added on top of the expiry-derived TTL.
const LEDGER_BUMP_BUFFER: u32 = 17_280;

/// Maximum persistent TTL allowed by the Soroban network.
/// ~6 months at 5 s/ledger.
const MAX_TTL: u32 = 3_110_400;

fn ttl_for_expiry(e: &Env, expires_at: u64) -> u32 {
    let now = e.ledger().timestamp();
    const SECONDS_PER_LEDGER: u64 = 5;

    let remaining_secs = expires_at.saturating_sub(now);
    let ledgers_until_expiry = (remaining_secs / SECONDS_PER_LEDGER) as u32;
    let desired = ledgers_until_expiry.saturating_add(LEDGER_BUMP_BUFFER);
    desired.min(MAX_TTL)
}

fn bump_nonce_ttl(e: &Env, key: &DataKey, expires_at: u64) {
    if !e.storage().persistent().has(key) {
        return;
    }
    let extend_to = ttl_for_expiry(e, expires_at).max(MIN_NONCE_TTL);
    let threshold = extend_to / 2;
    e.storage()
        .persistent()
        .extend_ttl(key, threshold, extend_to);
}

/// Returns the current nonce for `identity` (starts at 0).
#[allow(dead_code)]
pub fn get_nonce(e: &Env, identity: &Address) -> u64 {
    let key = DataKey::Nonce(identity.clone());
    let nonce: u64 = e.storage().persistent().get(&key).unwrap_or(0);
    bump_nonce_ttl(e, &key, 0);
    nonce
}

/// Consume the next nonce for `identity`, panicking on mismatch.
pub fn consume_nonce(e: &Env, identity: &Address, expected_nonce: u64) {
    let key = DataKey::Nonce(identity.clone());
    let current: u64 = e.storage().persistent().get(&key).unwrap_or(0);
    if current != expected_nonce {
        panic_with_error!(e, ContractError::InvalidNonce);
    }
    let next = current
        .checked_add(1)
        .unwrap_or_else(|| panic_with_error!(e, ContractError::Overflow));
    e.storage().persistent().set(&key, &next);
    bump_nonce_ttl(e, &key, 0);
}

/// Advance nonce to `new_nonce`, invalidating the skipped range.
#[allow(dead_code)]
pub fn invalidate_nonce_range(
    e: &Env,
    identity: &Address,
    new_nonce: u64,
    max_span: u64,
) -> (u64, u64) {
    let key = DataKey::Nonce(identity.clone());
    let current: u64 = e.storage().persistent().get(&key).unwrap_or(0);
    if new_nonce <= current {
        panic_with_error!(e, ContractError::InvalidNonce);
    }
    let span = new_nonce
        .checked_sub(current)
        .unwrap_or_else(|| panic_with_error!(e, ContractError::Underflow));
    if span > max_span {
        panic_with_error!(e, ContractError::InvalidNonce);
    }

    e.storage().persistent().set(&key, &new_nonce);
    bump_nonce_ttl(e, &key, 0);
    (current, new_nonce)
}
