<<<<<<< HEAD
//! Early Exit Penalty Mechanism
//!
//! Charges a configurable fee when users withdraw before the lock-up period ends.
//! Penalty is proportional to remaining lock time and is transferred to the treasury.

use soroban_sdk::{Address, Env, Symbol};

use crate::math;

/// Storage key for treasury address.
const KEY_TREASURY: &str = "treasury";
/// Storage key for early exit penalty rate in basis points (e.g. 500 = 5%).
const KEY_PENALTY_BPS: &str = "early_exit_penalty_bps";

/// Returns (treasury, penalty_bps). Panics if config not set.
pub fn get_config(e: &Env) -> (Address, u32) {
    let treasury = e
        .storage()
        .instance()
        .get::<_, Address>(&Symbol::new(e, KEY_TREASURY))
        .unwrap_or_else(|| panic!("early exit config not set"));
    let bps = e
        .storage()
        .instance()
        .get::<_, u32>(&Symbol::new(e, KEY_PENALTY_BPS))
        .unwrap_or_else(|| panic!("early exit penalty bps not set"));
    (treasury, bps)
}

/// Set early exit config. Only admin should call (enforced by caller).
pub fn set_config(e: &Env, treasury: Address, penalty_bps: u32) {
    if penalty_bps > math::BPS_DENOMINATOR as u32 {
        panic!("penalty_bps must be <= {}", math::BPS_DENOMINATOR);
    }
    e.storage()
        .instance()
        .set(&Symbol::new(e, KEY_TREASURY), &treasury);
    e.storage()
        .instance()
        .set(&Symbol::new(e, KEY_PENALTY_BPS), &penalty_bps);
}

/// Calculate early exit penalty based on remaining lock time.
/// penalty = (amount * penalty_bps / BPS_DENOMINATOR) * remaining_time / total_duration
/// Uses integer math to avoid overflow: (amount * penalty_bps / BPS_DENOMINATOR) * remaining_time / total_duration
#[must_use]
pub fn calculate_penalty(
    amount: i128,
    remaining_time: u64,
    total_duration: u64,
    penalty_bps: u32,
) -> i128 {
    if total_duration == 0 || penalty_bps == 0 {
        return 0;
    }
    let base = math::bps(
        amount,
        penalty_bps,
        "early exit penalty overflow",
        "early exit penalty div-by-zero",
    );
    let scaled = math::mul_i128(base, remaining_time as i128, "early exit penalty overflow");
    math::div_i128(
        scaled,
        total_duration as i128,
        "early exit penalty div-by-zero",
    )
}

/// Emit early exit penalty event.
pub fn emit_penalty_event(
    e: &Env,
    identity: &Address,
    withdraw_amount: i128,
    penalty_amount: i128,
=======
use credence_errors::ContractError;
use soroban_sdk::{contracttype, panic_with_error, Address, Env, Symbol};

use crate::DataKey;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EarlyExitConfig {
    pub treasury: Address,
    pub penalty_bps: u32,
}

const MAX_PENALTY_BPS: u32 = 10_000;
const PENALTY_BASIS_POINTS_DENOMINATOR: i128 = 10_000;

pub fn set_config(e: &Env, treasury: Address, penalty_bps: u32) {
    if penalty_bps > MAX_PENALTY_BPS {
        panic!("penalty_bps must be <= 10000");
    }
    let key = DataKey::EarlyExitConfig;
    e.storage().instance().set(
        &key,
        &EarlyExitConfig {
            treasury: treasury.clone(),
            penalty_bps,
        },
    );
    e.events().publish(
        (Symbol::new(e, "early_exit_config_set"),),
        (treasury, penalty_bps),
    );
}

pub fn get_config(e: &Env) -> (Address, u32) {
    let key = DataKey::EarlyExitConfig;
    e.storage()
        .instance()
        .get(&key)
        .unwrap_or_else(|| panic_with_error!(e, ContractError::NotInitialized))
}

pub fn calculate_penalty(amount: i128, remaining: u64, duration: u64, penalty_bps: u32) -> i128 {
    if duration == 0 {
        return 0;
    }
    let charge = amount
        .checked_mul(penalty_bps as i128)
        .unwrap_or(0)
        .checked_div(PENALTY_BASIS_POINTS_DENOMINATOR)
        .unwrap_or(0);
    charge
        .checked_mul(remaining as i128)
        .unwrap_or(0)
        .checked_div(duration as i128)
        .unwrap_or(0)
}

pub fn emit_penalty_event(
    e: &Env,
    identity: &Address,
    amount: i128,
    penalty: i128,
>>>>>>> main
    treasury: &Address,
) {
    e.events().publish(
        (Symbol::new(e, "early_exit_penalty"),),
<<<<<<< HEAD
        (
            identity.clone(),
            withdraw_amount,
            penalty_amount,
            treasury.clone(),
        ),
=======
        (identity.clone(), amount, penalty, treasury.clone()),
>>>>>>> main
    );
}
