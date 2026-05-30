#![cfg(test)]

use crate::{CredenceBond, CredenceBondClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Env, Symbol, Val,
};

fn setup() -> (Env, CredenceBondClient, Address) {
    let e = Env::default();
    e.mock_all_auths();
    let contract_id = e.register_contract(None, CredenceBond);
    let client = CredenceBondClient::new(&e, &contract_id);
    let admin = Address::generate(&e);
    client.initialize(&admin);
    (e, client, admin)
}

fn last_event(e: &Env) -> (Vec<Val>, Vec<Val>) {
    let events = e.events().all();
    let (_, topics, data) = events.get_unchecked(events.len() - 1);
    (topics, data)
}

fn find_event_by_name(e: &Env, event_name: &str) -> Option<(Vec<Val>, Vec<Val>)> {
    let events = e.events().all();
    let target_symbol: Val = Symbol::new(e, event_name).into_val(e);
    
    for i in (0..events.len()).rev() {
        let (_, topics, data) = events.get_unchecked(i);
        if topics.len() > 0 && topics.get_unchecked(0) == target_symbol {
            return Some((topics, data));
        }
    }
    None
}

#[test]
fn test_create_bond_emits_event() {
    let (e, client, _admin) = setup();
    let identity = Address::generate(&e);
    let amount = 1000_i128;
    let duration = 86400_u64; // 1 day
    
    e.ledger().with_mut(|li| {
        li.timestamp = 100;
    });
    
    client.create_bond(&identity, &amount, &duration, &false, &0);
    
    // Check that bond_created event was emitted
    let (topics, data) = find_event_by_name(&e, "bond_created")
        .expect("bond_created event should be emitted");
    
    let event_name: Val = Symbol::new(&e, "bond_created").into_val(&e);
    assert_eq!(topics.get_unchecked(0), event_name);
    
    // Data should contain: (identity, amount, bond_start, duration)
    let event_data: (Address, i128, u64, u64) = data.try_into_val(&e).unwrap();
    assert_eq!(event_data.0, identity);
    assert_eq!(event_data.1, amount);
    assert_eq!(event_data.2, 100); // bond_start timestamp
    assert_eq!(event_data.3, duration);
}

#[test]
fn test_withdraw_emits_event() {
    let (e, client, _admin) = setup();
    let identity = Address::generate(&e);
    let initial_amount = 1000_i128;
    let withdraw_amount = 300_i128;
    let duration = 86400_u64; // 1 day
    
    e.ledger().with_mut(|li| {
        li.timestamp = 100;
    });
    
    client.create_bond(&identity, &initial_amount, &duration, &false, &0);
    
    // Advance time past lock-up period
    e.ledger().with_mut(|li| {
        li.timestamp = 100 + duration + 1;
    });
    
    client.withdraw(&identity, &withdraw_amount);
    
    // Check that bond_withdrawn event was emitted
    let (topics, data) = find_event_by_name(&e, "bond_withdrawn")
        .expect("bond_withdrawn event should be emitted");
    
    let event_name: Val = Symbol::new(&e, "bond_withdrawn").into_val(&e);
    assert_eq!(topics.get_unchecked(0), event_name);
    
    // Data should contain: (identity, old_amount, new_amount, timestamp)
    let event_data: (Address, i128, i128, u64) = data.try_into_val(&e).unwrap();
    assert_eq!(event_data.0, identity);
    assert_eq!(event_data.1, initial_amount); // old_amount
    assert_eq!(event_data.2, initial_amount - withdraw_amount); // new_amount
    assert_eq!(event_data.3, 100 + duration + 1); // timestamp
}

#[test]
fn test_top_up_emits_event() {
    let (e, client, _admin) = setup();
    let identity = Address::generate(&e);
    let initial_amount = 1000_i128;
    let top_up_amount = 500_i128;
    let duration = 86400_u64;
    
    e.ledger().with_mut(|li| {
        li.timestamp = 100;
    });
    
    client.create_bond(&identity, &initial_amount, &duration, &false, &0);
    
    e.ledger().with_mut(|li| {
        li.timestamp = 200;
    });
    
    client.top_up(&identity, &top_up_amount);
    
    // Check that bond_topped_up event was emitted
    let (topics, data) = find_event_by_name(&e, "bond_topped_up")
        .expect("bond_topped_up event should be emitted");
    
    let event_name: Val = Symbol::new(&e, "bond_topped_up").into_val(&e);
    assert_eq!(topics.get_unchecked(0), event_name);
    
    // Data should contain: (identity, old_amount, new_amount, timestamp)
    let event_data: (Address, i128, i128, u64) = data.try_into_val(&e).unwrap();
    assert_eq!(event_data.0, identity);
    assert_eq!(event_data.1, initial_amount); // old_amount
    assert_eq!(event_data.2, initial_amount + top_up_amount); // new_amount
    assert_eq!(event_data.3, 200); // timestamp
}

#[test]
fn test_extend_duration_emits_event() {
    let (e, client, _admin) = setup();
    let identity = Address::generate(&e);
    let amount = 1000_i128;
    let initial_duration = 86400_u64; // 1 day
    let additional_duration = 43200_u64; // 12 hours
    
    e.ledger().with_mut(|li| {
        li.timestamp = 100;
    });
    
    client.create_bond(&identity, &amount, &initial_duration, &false, &0);
    
    e.ledger().with_mut(|li| {
        li.timestamp = 200;
    });
    
    client.extend_duration(&identity, &additional_duration);
    
    // Check that bond_duration_extended event was emitted
    let (topics, data) = find_event_by_name(&e, "bond_duration_extended")
        .expect("bond_duration_extended event should be emitted");
    
    let event_name: Val = Symbol::new(&e, "bond_duration_extended").into_val(&e);
    assert_eq!(topics.get_unchecked(0), event_name);
    
    // Data should contain: (identity, old_duration, new_duration, timestamp)
    let event_data: (Address, u64, u64, u64) = data.try_into_val(&e).unwrap();
    assert_eq!(event_data.0, identity);
    assert_eq!(event_data.1, initial_duration); // old_duration
    assert_eq!(event_data.2, initial_duration + additional_duration); // new_duration
    assert_eq!(event_data.3, 200); // timestamp
}

#[test]
fn test_multiple_top_ups_emit_separate_events() {
    let (e, client, _admin) = setup();
    let identity = Address::generate(&e);
    let initial_amount = 1000_i128;
    let duration = 86400_u64;
    
    e.ledger().with_mut(|li| {
        li.timestamp = 100;
    });
    
    client.create_bond(&identity, &initial_amount, &duration, &false, &0);
    
    // First top-up
    e.ledger().with_mut(|li| {
        li.timestamp = 200;
    });
    client.top_up(&identity, &100);
    
    // Second top-up
    e.ledger().with_mut(|li| {
        li.timestamp = 300;
    });
    client.top_up(&identity, &200);
    
    // Count bond_topped_up events
    let events = e.events().all();
    let target_symbol: Val = Symbol::new(&e, "bond_topped_up").into_val(&e);
    let mut count = 0;
    
    for i in 0..events.len() {
        let (_, topics, _) = events.get_unchecked(i);
        if topics.len() > 0 && topics.get_unchecked(0) == target_symbol {
            count += 1;
        }
    }
    
    assert_eq!(count, 2, "Should have emitted 2 bond_topped_up events");
}

#[test]
fn test_zero_amount_top_up_emits_event() {
    let (e, client, _admin) = setup();
    let identity = Address::generate(&e);
    let initial_amount = 1000_i128;
    let duration = 86400_u64;
    
    e.ledger().with_mut(|li| {
        li.timestamp = 100;
    });
    
    client.create_bond(&identity, &initial_amount, &duration, &false, &0);
    
    e.ledger().with_mut(|li| {
        li.timestamp = 200;
    });
    
    // Top up with zero amount (edge case)
    client.top_up(&identity, &0);
    
    // Check that event was still emitted
    let (topics, data) = find_event_by_name(&e, "bond_topped_up")
        .expect("bond_topped_up event should be emitted even for zero amount");
    
    let event_name: Val = Symbol::new(&e, "bond_topped_up").into_val(&e);
    assert_eq!(topics.get_unchecked(0), event_name);
    
    let event_data: (Address, i128, i128, u64) = data.try_into_val(&e).unwrap();
    assert_eq!(event_data.1, initial_amount); // old_amount
    assert_eq!(event_data.2, initial_amount); // new_amount (unchanged)
}

#[test]
fn test_rolling_bond_withdraw_emits_event() {
    let (e, client, _admin) = setup();
    let identity = Address::generate(&e);
    let amount = 1000_i128;
    let duration = 86400_u64; // 1 day
    let notice_period = 3600_u64; // 1 hour
    
    e.ledger().with_mut(|li| {
        li.timestamp = 100;
    });
    
    // Create rolling bond
    client.create_bond(&identity, &amount, &duration, &true, &notice_period);
    
    // Advance past bond period
    e.ledger().with_mut(|li| {
        li.timestamp = 100 + duration + 1;
    });
    
    // Request withdrawal
    client.request_withdrawal(&identity);
    
    // Advance past notice period
    e.ledger().with_mut(|li| {
        li.timestamp = 100 + duration + 1 + notice_period + 1;
    });
    
    // Withdraw
    client.withdraw(&identity, &500);
    
    // Check that bond_withdrawn event was emitted
    let (topics, _data) = find_event_by_name(&e, "bond_withdrawn")
        .expect("bond_withdrawn event should be emitted for rolling bond");
    
    let event_name: Val = Symbol::new(&e, "bond_withdrawn").into_val(&e);
    assert_eq!(topics.get_unchecked(0), event_name);
}

#[test]
fn test_event_indexing_reconstruction() {
    // This test demonstrates that indexers can reconstruct bond balance from events alone
    let (e, client, _admin) = setup();
    let identity = Address::generate(&e);
    let initial_amount = 1000_i128;
    let duration = 86400_u64;
    
    e.ledger().with_mut(|li| {
        li.timestamp = 100;
    });
    
    // Create bond
    client.create_bond(&identity, &initial_amount, &duration, &false, &0);
    
    // Top up twice
    e.ledger().with_mut(|li| {
        li.timestamp = 200;
    });
    client.top_up(&identity, &300);
    
    e.ledger().with_mut(|li| {
        li.timestamp = 300;
    });
    client.top_up(&identity, &200);
    
    // Withdraw after lock-up
    e.ledger().with_mut(|li| {
        li.timestamp = 100 + duration + 1;
    });
    client.withdraw(&identity, &500);
    
    // Reconstruct balance from events
    let events = e.events().all();
    let mut reconstructed_balance = 0_i128;
    
    for i in 0..events.len() {
        let (_, topics, data) = events.get_unchecked(i);
        if topics.len() > 0 {
            let event_name_val = topics.get_unchecked(0);
            
            if event_name_val == Symbol::new(&e, "bond_created").into_val(&e) {
                let event_data: (Address, i128, u64, u64) = data.try_into_val(&e).unwrap();
                reconstructed_balance = event_data.1;
            } else if event_name_val == Symbol::new(&e, "bond_topped_up").into_val(&e) {
                let event_data: (Address, i128, i128, u64) = data.try_into_val(&e).unwrap();
                reconstructed_balance = event_data.2; // new_amount
            } else if event_name_val == Symbol::new(&e, "bond_withdrawn").into_val(&e) {
                let event_data: (Address, i128, i128, u64) = data.try_into_val(&e).unwrap();
                reconstructed_balance = event_data.2; // new_amount
            }
        }
    }
    
    // Verify reconstructed balance matches actual state
    let bond = client.get_identity_state(&identity);
    assert_eq!(reconstructed_balance, bond.bonded_amount);
    assert_eq!(reconstructed_balance, 1000); // 1000 + 300 + 200 - 500 = 1000
}
