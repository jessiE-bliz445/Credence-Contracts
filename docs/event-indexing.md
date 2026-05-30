# Event Indexing & Consumer Guidance (v2)

This document provides technical specifications for backend services indexing events from the Credence protocol.

## 1. Event Versioning Strategy

The protocol is currently transitioning from **v1 (Legacy)** to **v2 (Indexer-Grade)**.

| Feature            | v1 (Legacy)                 | v2 (High-Fidelity)                                        |
| :----------------- | :-------------------------- | :-------------------------------------------------------- |
| **Identification** | Topic 0 is a general name.  | Topic 0 identifies the event; Topic 1 & 2 are keys.       |
| **Data Types**     | Variable/Mixed.             | Normalized (usually `i128` pairs or specialized structs). |
| **Filtering**      | Requires full data parsing. | Filterable at the ledger level via topics.                |

**Backend Recommendation:** Consumers should support "Double-Read" logic during the migration period or implement a version-aware parser that checks Topic 0 for the `_v2` suffix or the specific new Symbol name (e.g., `param_updated` vs `parameter_changed`).

---

## 2. Bond Lifecycle Events

The bond contract emits structured lifecycle events for all state-changing operations, enabling indexers to reconstruct bond state from events alone without re-reading full contract state.

### 2.1 Bond Creation (`bond_created`)

Emitted when a new bond is created.

**Event Name:** `bond_created` (Symbol)

**Data Payload:**
- `identity`: `Address` - The identity owning the bond
- `amount`: `i128` - Initial bonded amount
- `bond_start`: `u64` - Timestamp when bond was created
- `duration`: `u64` - Bond duration in seconds

**Example:**
```rust
e.events().publish(
    (Symbol::new(&e, "bond_created"),),
    (identity, amount, bond_start, duration),
);
```

### 2.2 Bond Withdrawal (`bond_withdrawn`)

Emitted when funds are withdrawn from a bond (after lock-up period).

**Event Name:** `bond_withdrawn` (Symbol)

**Data Payload:**
- `identity`: `Address` - The identity owning the bond
- `old_amount`: `i128` - Bonded amount before withdrawal
- `new_amount`: `i128` - Bonded amount after withdrawal
- `timestamp`: `u64` - Timestamp of withdrawal

**Example:**
```rust
e.events().publish(
    (Symbol::new(&e, "bond_withdrawn"),),
    (identity, old_amount, new_amount, timestamp),
);
```

### 2.3 Bond Top-Up (`bond_topped_up`)

Emitted when additional funds are added to an existing bond.

**Event Name:** `bond_topped_up` (Symbol)

**Data Payload:**
- `identity`: `Address` - The identity owning the bond
- `old_amount`: `i128` - Bonded amount before top-up
- `new_amount`: `i128` - Bonded amount after top-up
- `timestamp`: `u64` - Timestamp of top-up

**Example:**
```rust
e.events().publish(
    (Symbol::new(&e, "bond_topped_up"),),
    (identity, old_amount, new_amount, timestamp),
);
```

### 2.4 Duration Extension (`bond_duration_extended`)

Emitted when the bond duration is extended.

**Event Name:** `bond_duration_extended` (Symbol)

**Data Payload:**
- `identity`: `Address` - The identity owning the bond
- `old_duration`: `u64` - Duration before extension (seconds)
- `new_duration`: `u64` - Duration after extension (seconds)
- `timestamp`: `u64` - Timestamp of extension

**Example:**
```rust
e.events().publish(
    (Symbol::new(&e, "bond_duration_extended"),),
    (identity, old_duration, new_duration, timestamp),
);
```

### 2.5 Bond Slashing (`bond_slashed`)

Emitted when a bond is slashed by admin.

**Event Name:** `bond_slashed` (Symbol)

**Data Payload:**
- `identity`: `Address` - The identity owning the bond
- `slash_amount`: `i128` - Amount slashed in this event
- `total_slashed`: `i128` - Total slashed amount for this bond
- `timestamp`: `u64` - Timestamp of slash (implicit from ledger)

**Example:**
```rust
e.events().publish(
    (Symbol::new(&e, "bond_slashed"),),
    (identity, slash_amount, total_slashed),
);
```

### 2.6 Withdrawal Request (`withdrawal_requested`)

Emitted when a rolling bond holder requests withdrawal.

**Event Name:** `withdrawal_requested` (Symbol)

**Data Payload:**
- `identity`: `Address` - The identity owning the bond
- `timestamp`: `u64` - Timestamp when withdrawal was requested

### 2.7 Bond Renewal (`bond_renewed`)

Emitted when a rolling bond auto-renews.

**Event Name:** `bond_renewed` (Symbol)

**Data Payload:**
- `identity`: `Address` - The identity owning the bond
- `bond_start`: `u64` - New bond start timestamp
- `bond_duration`: `u64` - Bond duration

---

## 3. Indexing Strategy for Bond State Reconstruction

Indexers can reconstruct complete bond state from events without querying contract storage:

### 3.1 Balance Tracking

Track bonded balance through lifecycle events:

```javascript
let balance = 0;

// Process events in chronological order
events.forEach(event => {
  switch(event.name) {
    case 'bond_created':
      balance = event.data.amount;
      break;
    case 'bond_topped_up':
      balance = event.data.new_amount;
      break;
    case 'bond_withdrawn':
      balance = event.data.new_amount;
      break;
  }
});
```

### 3.2 Duration Tracking

Track bond duration and extensions:

```javascript
let duration = 0;

events.forEach(event => {
  switch(event.name) {
    case 'bond_created':
      duration = event.data.duration;
      break;
    case 'bond_duration_extended':
      duration = event.data.new_duration;
      break;
  }
});
```

### 3.3 Slashing Tracking

Track slashed amounts separately:

```javascript
let totalSlashed = 0;

events.forEach(event => {
  if (event.name === 'bond_slashed') {
    totalSlashed = event.data.total_slashed;
  }
});

// Available balance = bonded_amount - total_slashed
let availableBalance = balance - totalSlashed;
```

---

## 4. Parameter Updates (`param_updated`)

The most critical events for protocol health.

### Topic Structure (Indexed)

1. **Event Name:** `param_updated` (Symbol)
2. **Parameter Key:** Specific identifier (e.g., `fee_prot`, `max_lev`, `th_gold`)
3. **Category:** Grouping for filtering (e.g., `fee`, `risk`, `tier`, `cooldown`)
4. **Admin:** The `Address` that authorized the change.

### Data Payload (Unindexed)

- `old_value`: `i128`
- `new_value`: `i128`

**Indexing Tip:** Use the **Category** topic to build specialized dashboards. For example, a "Risk Dashboard" should only subscribe to events where Topic 2 == `risk`.

---

## 5. Recommended Keys & Symbols

To maintain consistency across the ecosystem, use these standardized `symbol_short!` keys:

### Fee Category (`fee`)

- `fee_prot`: Protocol-wide fees.
- `fee_att`: Attestation/Validator fees.

### Risk Category (`risk`)

- `max_lev`: Maximum allowed leverage.
- `slsh_p`: Slashing penalty percentages.

### Tier Category (`tier`)

- `th_brnz`, `th_slvr`, `th_gold`, `th_plat`: Collateral/Bond thresholds.

---

## 6. Idempotency & Reliable Processing

To avoid double-counting or missing events during re-orgs or service restarts:

1. **The Unique Identity:** Every event's unique ID is a combination of:
   `LedgerSequence` + `TransactionHash` + `EventIndexWithinTx`
2. **Order of Truth:** Always use the `LedgerTimestamp` provided in the event data as the canonical time of the state change.
3. **Re-org Handling:** Only mark an event as "Final" after it has reached a depth of 12+ ledgers (Standard Stellar Finality).

---

## 7. Backend Schema Example (JSON)

When indexing into a database (PostgreSQL/MongoDB), normalize to this structure:

### Bond Lifecycle Event Schema

```json
{
  "contract": "C...",
  "event_type": "bond_topped_up",
  "identity": "G...",
  "values": {
    "old_amount": "1000",
    "new_amount": "1500",
    "delta": "500"
  },
  "blockchain": {
    "ledger": 123456,
    "tx_hash": "...",
    "timestamp": 1713985850
  }
}
```

### Parameter Update Event Schema

```json
{
  "contract": "C...",
  "version": "v2",
  "event_type": "param_updated",
  "meta": {
    "key": "fee_prot",
    "category": "fee",
    "admin": "G..."
  },
  "values": {
    "old": "50",
    "new": "100",
    "delta": "50"
  },
  "blockchain": {
    "ledger": 123456,
    "tx_hash": "...",
    "timestamp": 1713985850
  }
}
```

---

## 8. Event Consistency Guarantees

All bond lifecycle events follow these consistency rules:

1. **Old/New Pattern:** Events that modify state include both old and new values for auditability
2. **Timestamp Inclusion:** All events include the ledger timestamp for temporal ordering
3. **Identity Tracking:** All bond events include the identity address for filtering
4. **Atomic State:** Events are emitted after state changes are committed to storage

This ensures indexers can:
- Reconstruct exact state at any point in time
- Audit all state transitions
- Detect and handle any inconsistencies
- Build reliable analytics without contract queries
