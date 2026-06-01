use crate::IdentityBond;

pub fn is_period_ended(now: u64, bond_start: u64, bond_duration: u64) -> bool {
    let end = bond_start.checked_add(bond_duration).expect("overflow");
    now >= end
}

pub fn apply_renewal(bond: &mut IdentityBond, now: u64) {
    // Ensure the new bond start plus duration does not overflow to avoid invalid timestamps.
    if now.checked_add(bond.bond_duration).is_none() {
        panic_with_error!(Env::default(), ContractError::Overflow);
    }
    bond.bond_start = now;
}
