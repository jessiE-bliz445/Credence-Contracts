# Event Indexing Migration Guide

## Overview

This document outlines the migration from v1 to v2 position lifecycle events in the Credence Contracts, which improves indexing capabilities for off-chain analytics and query efficiency.

**Latest Update (2026):** Added comprehensive lifecycle events for `create_bond`, `withdraw`, `top_up`, and `extend_duration` operations to enable full bond state reconstruction from events alone.

## Problem Statement

The original position lifecycle events (`bond_created`, `bond_withdrawn`, `bond_increased`, `bond_slashed`) had suboptimal indexing that made off-chain queries expensive and error-prone:

- Only `identity` (user address) was indexed
- Critical fields like `amount`, `timestamp`, and `balance` were only in data payload
- No efficient way to filter by amount ranges or time periods
- Required full event data scanning for common analytics queries

**Additional Gap Identified:** Several critical bond operations (`withdraw`, `top_up`, `extend_duration`) emitted no events at all, and `create_bond` only emitted a tier-change event. This made it impossible for indexers to reconstruct bond state changes without re-reading full contract storage.

## Solution: V2 Events with Enhanced Indexing

### New Event Structure

#### `bond_created` (NEW - Previously Silent)

**Event Name:** `bond_created` (Symbol)

**Data Payload:**
- `identity`: `Address` - The identity owning the bond
- `amount`: `i128` - Initial bonded amount
- `bond_start`: `u64` - Timestamp when bond was created
- `duration`: `u64` - Bond duration in seconds

**Rationale:** Previously, `create_bond` only emitted a `tier_changed` event, which didn't capture the full bond creation context. Indexers couldn't track when bonds were created or their initial parameters.

#### `bond_withdrawn` (NEW - Previously Silent)

**Event Name:** `bond_withdrawn` (Symbol)

**Data Payload:**
- `identity`: `Address` - The identity owning the bond
- `old_amount`: `i128` - Bonded amount before withdrawal
- `new_amount`: `i128` - Bonded amount after withdrawal
- `timestamp`: `u64` - Timestamp of withdrawal

**Rationale:** The `withdraw` function was completely silent. Indexers had no way to track withdrawals without polling contract state.

#### `bond_topped_up` (NEW - Previously Silent)

**Event Name:** `bond_topped_up` (Symbol)

**Data Payload:**
- `identity`: `Address` - The identity owning the bond
- `old_amount`: `i128` - Bonded amount before top-up
- `new_amount`: `i128` - Bonded amount after top-up
- `timestamp`: `u64` - Timestamp of top-up

**Rationale:** The `top_up` function was completely silent. Balance increases were invisible to indexers.

#### `bond_duration_extended` (NEW - Previously Silent)

**Event Name:** `bond_duration_extended` (Symbol)

**Data Payload:**
- `identity`: `Address` - The identity owning the bond
- `old_duration`: `u64` - Duration before extension (seconds)
- `new_duration`: `u64` - Duration after extension (seconds)
- `timestamp`: `u64` - Timestamp of extension

**Rationale:** The `extend_duration` function was completely silent. Duration changes were invisible to indexers.

#### `bond_created_v2`
**Indexed Topics:**
- `Symbol` - "bond_created_v2"
- `Address` - The identity owning the bond
- `i128` - The initial bonded amount (now indexed!)
- `u64` - The bond start timestamp (now indexed!)

**Data:**
- `u64` - The duration of the bond in seconds
- `bool` - Whether the bond is rolling
- `u64` - Bond end timestamp (calculated)

#### `bond_withdrawn_v2`
**Indexed Topics:**
- `Symbol` - "bond_withdrawn_v2"
- `Address` - The identity owning the bond
- `i128` - The amount withdrawn (now indexed!)
- `i128` - The remaining bonded amount (now indexed!)
- `u64` - The withdrawal timestamp (now indexed!)

**Data:**
- `bool` - Whether this was an early withdrawal (penalty applied)
- `i128` - Penalty amount if early withdrawal

#### `bond_increased_v2`
**Indexed Topics:**
- `Symbol` - "bond_increased_v2"
- `Address` - The identity owning the bond
- `i128` - The additional amount added (now indexed!)
- `i128` - The new total bonded amount (now indexed!)
- `u64` - The increase timestamp (now indexed!)

**Data:**
- `bool` - Whether this increase crossed a tier threshold
- `BondTier` - New bond tier after increase

#### `bond_slashed_v2`
**Indexed Topics:**
- `Symbol` - "bond_slashed_v2"
- `Address` - The identity owning the bond
- `i128` - The amount slashed in this event (now indexed!)
- `i128` - The new total slashed amount for this bond (now indexed!)
- `u64` - The slash timestamp (now indexed!)
- `Address` - The admin who performed the slash (now indexed!)

**Data:**
- `String` - Reason for the slash
- `bool` - Whether this was a full slash (bond completely liquidated)

## Migration Strategy

### Phase 1: Add Missing Lifecycle Events (COMPLETED)

The first phase addressed the critical gap where several bond operations emitted no events:

1. **`create_bond`** - Now emits `bond_created` with full creation context
2. **`withdraw`** - Now emits `bond_withdrawn` with old/new amounts
3. **`top_up`** - Now emits `bond_topped_up` with old/new amounts
4. **`extend_duration`** - Now emits `bond_duration_extended` with old/new durations

**Event Pattern:** All new events follow the consistent pattern:
- Include identity address for filtering
- Include old and new values for auditability
- Include timestamp for temporal ordering
- Use descriptive event names matching existing conventions (`bond_slashed`, `bond_renewed`, `withdrawal_requested`)

### Phase 2: Backward Compatibility

During the migration period, both v1 and v2 events are emitted simultaneously:

```rust
// Emit both old and new events for backward compatibility during migration
events::emit_bond_created(&e, &identity, amount, duration, is_rolling);
events::emit_bond_created_v2(&e, &identity, amount, duration, is_rolling, bond_start);
```

### Indexer Migration Path

1. **Phase 1: Dual Event Processing**
   - Process both v1 and v2 events
   - Validate data consistency between versions
   - Build v2 indexing infrastructure

2. **Phase 2: V2 Priority**
   - Prioritize v2 events for new data
   - Use v1 events only for historical data
   - Implement fallback mechanisms

3. **Phase 3: V2 Complete**
   - Deprecate v1 event processing
   - Remove v1 event emission (future version)
   - Full v2 indexing utilization

### Query Improvements

#### Before (V1)
```javascript
// Inefficient - requires scanning all events
const largeBonds = events.filter(event => {
  if (event.topics[0] === 'bond_created') {
    const data = parseEventData(event.data);
    return data.amount >= 10000;
  }
});
```

#### After (V2)
```javascript
// Efficient - uses indexed amount field
const largeBonds = events.filter(event => {
  return event.topics[0] === 'bond_created_v2' && 
         event.topics[2] >= 10000; // Indexed amount
});
```

## Benefits

### For Off-Chain Indexers

1. **Complete State Reconstruction**
   - **NEW:** Can now reconstruct full bond state from events alone
   - **NEW:** No need to query contract storage for balance tracking
   - **NEW:** All state transitions are now visible via events

2. **Reduced Computational Cost**
   - Filter by amount without parsing event data
   - Time-range queries using indexed timestamps
   - Balance queries using indexed remaining amounts

3. **Improved Query Performance**
   - Database indexes on frequently queried fields
   - Complex queries become simple indexed lookups
   - Support for real-time analytics dashboards

4. **Enhanced Analytics Capabilities**
   - Amount distribution analysis
   - Time-based trend analysis
   - Tier progression tracking
   - Slash pattern analysis
   - **NEW:** Withdrawal pattern analysis
   - **NEW:** Top-up frequency tracking
   - **NEW:** Duration extension patterns

### For Smart Contract Users

1. **No Breaking Changes**
   - All existing functionality preserved
   - Gradual migration path
   - Backward compatible event emission

2. **Better Data Availability**
   - More detailed event information
   - Additional context (tier changes, penalties)
   - Improved audit trails

## Implementation Details

### Event Emission Pattern

All bond lifecycle functions now emit structured events:

```rust
// create_bond - NEW EVENT
pub fn create_bond(...) -> IdentityBond {
    // ... bond creation logic ...
    
    e.events().publish(
        (Symbol::new(&e, "bond_created"),),
        (identity, amount, bond_start, duration),
    );
    
    bond
}

// withdraw - NEW EVENT
pub fn withdraw(...) -> IdentityBond {
    let old_amount = bond.bonded_amount;
    // ... withdrawal logic ...
    
    e.events().publish(
        (Symbol::new(&e, "bond_withdrawn"),),
        (identity, old_amount, bond.bonded_amount, timestamp),
    );
    
    bond
}

// top_up - NEW EVENT
pub fn top_up(...) -> IdentityBond {
    let old_amount = bond.bonded_amount;
    // ... top-up logic ...
    
    e.events().publish(
        (Symbol::new(&e, "bond_topped_up"),),
        (identity, old_amount, bond.bonded_amount, timestamp),
    );
    
    bond
}

// extend_duration - NEW EVENT
pub fn extend_duration(...) -> IdentityBond {
    let old_duration = bond.bond_duration;
    // ... extension logic ...
    
    e.events().publish(
        (Symbol::new(&e, "bond_duration_extended"),),
        (identity, old_duration, bond.bond_duration, timestamp),
    );
    
    bond
}
```

### Event Emission Pattern (V2 Enhanced Indexing)

```rust
// In contract functions
pub fn create_bond_with_rolling(...) -> IdentityBond {
    // ... bond creation logic ...
    
    // Emit both old and new events for backward compatibility during migration
    events::emit_bond_created(&e, &identity, amount, duration, is_rolling);
    events::emit_bond_created_v2(&e, &identity, amount, duration, is_rolling, bond_start);
    
    bond
}
```

### Testing Strategy

The migration includes comprehensive tests:

1. **Lifecycle Event Tests (NEW)**
   - Verify all bond operations emit appropriate events
   - Validate event data structure and content
   - Test old/new value pairs for accuracy
   - Test timestamp inclusion
   - Test edge cases (zero amounts, repeated operations)
   - **Test state reconstruction from events alone**

2. **Backward Compatibility Tests**
   - Verify both v1 and v2 events are emitted
   - Validate data consistency between versions
   - Test existing functionality unchanged

3. **Indexing Efficiency Tests**
   - Test amount-based filtering using indexed fields
   - Test time-based queries using indexed timestamps
   - Verify query performance improvements

4. **Schema Validation Tests**
   - Validate v2 event structure
   - Test data field accuracy
   - Ensure proper type handling

## Timeline

- **Phase 1 (COMPLETED)**: Add missing lifecycle events
  - ✅ Implement `bond_created`, `bond_withdrawn`, `bond_topped_up`, `bond_duration_extended` events
  - ✅ Add comprehensive test coverage
  - ✅ Update documentation
- **Phase 2 (Week 1-2)**: Implement v2 events with enhanced indexing
- **Phase 3 (Week 3-4)**: Indexer migration and testing
- **Phase 4 (Week 5-6)**: Production deployment and monitoring
- **Phase 5 (Week 7-8)**: Performance validation and optimization

## Risk Mitigation

1. **Data Consistency**
   - Comprehensive test coverage
   - Data validation between v1 and v2 events
   - Rollback procedures if issues detected

2. **Indexer Compatibility**
   - Gradual migration path
   - Fallback mechanisms
   - Extensive testing with indexer teams

3. **User Impact**
   - No breaking changes to existing functionality
   - Clear communication about migration
   - Documentation and support

## Future Considerations

1. **V1 Event Deprecation**
   - Plan for eventual removal of v1 events
   - Communication timeline for indexer teams
   - Clean-up of deprecated code

2. **Additional Event Enhancements**
   - Consider adding more indexed fields based on usage patterns
   - Evaluate other contract events for similar improvements
   - Standardize event indexing patterns across contracts

3. **Performance Monitoring**
   - Track query performance improvements
   - Monitor indexer resource usage
   - Collect feedback from analytics teams

## Conclusion

The lifecycle event improvements provide immediate value by making all bond state transitions visible to indexers. The addition of `bond_created`, `bond_withdrawn`, `bond_topped_up`, and `bond_duration_extended` events closes a critical gap that previously required indexers to poll contract state.

The v2 event indexing improvements will further enhance these capabilities with optimized filtering and querying, while maintaining full backward compatibility. The gradual migration approach ensures minimal risk while delivering immediate value to indexers and analytics consumers.

The enhanced indexing capabilities enable more sophisticated analytics, better user experiences, and reduced infrastructure costs for off-chain data processing.

### Key Achievements

1. **Complete Event Coverage:** All bond operations now emit events
2. **State Reconstruction:** Indexers can reconstruct bond state from events alone
3. **Consistent Patterns:** All events follow the same old/new value pattern
4. **Backward Compatible:** No breaking changes to existing functionality
5. **Well Tested:** Comprehensive test coverage including edge cases and state reconstruction
