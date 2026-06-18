# Formal Verification for Solana Programs

Mathematical proof of correctness for critical invariants. Use formal verification when manual review and testing are not sufficient to rule out an entire class of bugs.

---

## When to Use Formal Verification

Apply to:
- **Arithmetic correctness**: "This calculation can never overflow, for all inputs"
- **State invariants**: "Total supply always equals sum of all balances"
- **Access control**: "Only the authority can call this function, for all possible callers"
- **PDA uniqueness**: "These seeds uniquely identify one account, no collisions possible"
- **Economic invariants**: "Pool value never decreases on a valid swap"

Skip formal verification for:
- Business logic that is too complex to specify completely (use fuzzing instead)
- UI / off-chain components
- Instructions with low value at risk

---

## Tool 1: Kani — Rust Model Checker

Kani proves properties about Rust code at the mathematical level. Ideal for arithmetic, state machine properties, and invariants on pure functions.

### Setup

```bash
# Install Kani
cargo install --locked kani-verifier
cargo kani setup

# Verify Kani is available
cargo kani --version
```

### Writing Kani Proofs

```rust
// In your program's test or verification module:
#[cfg(kani)]
mod verification {
    use super::*;

    // Prove: calculate_fee never overflows for any valid input
    #[kani::proof]
    fn verify_fee_calculation_no_overflow() {
        let amount: u64 = kani::any();
        let fee_bps: u16 = kani::any();

        // Constrain inputs to valid range
        kani::assume(amount <= MAX_AMOUNT);
        kani::assume(fee_bps <= 10_000);

        // This should not panic (no overflow)
        let fee = calculate_fee(amount, fee_bps);

        // Additional invariant: fee <= amount
        assert!(fee <= amount);
    }

    // Prove: deposit always increases pool total
    #[kani::proof]
    fn verify_deposit_increases_total() {
        let mut pool: Pool = kani::any();
        let amount: u64 = kani::any();

        kani::assume(pool.total <= u64::MAX - amount);  // no overflow
        kani::assume(amount > 0);

        let total_before = pool.total;
        pool.deposit(amount).unwrap();

        assert!(pool.total > total_before);
        assert!(pool.total == total_before + amount);
    }

    // Prove: PDA seeds produce unique addresses (no collision)
    #[kani::proof]
    fn verify_pda_uniqueness() {
        let user_a: Pubkey = kani::any();
        let user_b: Pubkey = kani::any();
        kani::assume(user_a != user_b);

        let (pda_a, _) = Pubkey::find_program_address(
            &[b"vault", user_a.as_ref()],
            &crate::ID,
        );
        let (pda_b, _) = Pubkey::find_program_address(
            &[b"vault", user_b.as_ref()],
            &crate::ID,
        );

        // Different users produce different PDAs
        assert_ne!(pda_a, pda_b);
    }
}
```

### Running Kani

```bash
# Verify all proofs in the project
cargo kani

# Verify a specific harness
cargo kani --harness verify_fee_calculation_no_overflow

# With unwind bound for loops
cargo kani --unwind 10

# Generate a report
cargo kani --report
```

### Interpreting Results

```
VERIFICATION RESULT: SUCCESSFUL
  — All assertions hold for all possible inputs

VERIFICATION RESULT: FAILED
  — Found a counterexample: [concrete input values]
  — Use these to write a regression test immediately
```

---

## Tool 2: Trident — Coverage-Guided Fuzzing with Property Testing

Trident is purpose-built for Anchor programs. Use it to verify invariants hold across millions of randomly generated instruction sequences.

### Setup

```toml
# Fuzz.toml
[fuzz]
fuzzing_with_stats = true
allow_duplicate_txs = false

[[fuzz.programs_config]]
program_id = "YourProgramID"

[fuzz.accounts_snapshots]
# Trident saves account state snapshots for invariant checks
```

```rust
// trident-tests/fuzz_tests/fuzz_0/src/lib.rs

use trident_client::fuzzing::*;
use your_program::*;

// Define your invariant check
fn invariant_check(pre: &AccountsSnapshots, post: &AccountsSnapshots) {
    // INVARIANT: total pool value never decreases on valid operations
    let pre_total = get_pool_total(pre);
    let post_total = get_pool_total(post);

    // This assertion fires if the invariant is violated
    assert!(
        post_total >= pre_total,
        "Pool value decreased: {} → {}",
        pre_total,
        post_total
    );
}

// Fuzz entry point
#[derive(Debug, FuzzTestExecutor, Default)]
pub struct FuzzTest {
    pub accounts: FuzzAccounts,
}

impl FuzzTest {
    fn fuzz_ix_initialize(&mut self, data: &InitializeData) -> Result<(), FuzzingError> {
        // Build and execute instruction
        let instruction = build_initialize_ix(data)?;
        execute_ix_with_invariant_check(instruction, invariant_check)
    }

    fn fuzz_ix_deposit(&mut self, data: &DepositData) -> Result<(), FuzzingError> {
        let instruction = build_deposit_ix(data)?;
        execute_ix_with_invariant_check(instruction, invariant_check)
    }

    fn fuzz_ix_withdraw(&mut self, data: &WithdrawData) -> Result<(), FuzzingError> {
        let instruction = build_withdraw_ix(data)?;
        execute_ix_with_invariant_check(instruction, invariant_check)
    }
}
```

```bash
# Initialize Trident workspace
trident init

# Run fuzzer
trident fuzz run fuzz_0

# Run with increased intensity
trident fuzz run fuzz_0 -- -max_total_time=3600  # 1 hour

# Run with corpus minimization
trident fuzz minimize fuzz_0
```

---

## Tool 3: Mollusk — Lightweight Program Testing for Invariants

Mollusk allows fast, deterministic property testing without the full SBF runtime overhead.

```rust
use mollusk_svm::Mollusk;

fn test_invariant_total_supply_constant_on_transfer() {
    let mollusk = Mollusk::new(&id(), "target/deploy/your_program");

    let mut accounts = setup_accounts();
    let initial_total = sum_balances(&accounts);

    // Execute 100 random transfers
    for _ in 0..100 {
        let (from, to, amount) = random_transfer_params();
        let result = mollusk.process_instruction(
            &build_transfer_ix(from, to, amount),
            &accounts_slice(&accounts),
        );
        update_accounts(&mut accounts, &result);
    }

    let final_total = sum_balances(&accounts);
    assert_eq!(
        initial_total, final_total,
        "Total supply changed: {} → {}",
        initial_total, final_total
    );
}
```

---

## Tool 4: Rust's Built-in Property Testing (proptest / quickcheck)

For pure functions (math, validation logic) that don't require on-chain context:

```toml
# Cargo.toml
[dev-dependencies]
proptest = "1"
```

```rust
use proptest::prelude::*;

proptest! {
    // Verify fee calculation is commutative with rounding direction
    #[test]
    fn fee_never_exceeds_amount(amount in 0u64..u64::MAX, fee_bps in 0u16..10_000) {
        let fee = calculate_fee_safe(amount, fee_bps as u64);
        prop_assert!(fee <= amount);
    }

    // Verify deposit/withdraw roundtrip
    #[test]
    fn deposit_withdraw_roundtrip(amount in 1u64..1_000_000_000) {
        let mut pool = Pool::default();
        let shares = pool.deposit(amount).unwrap();
        let withdrawn = pool.withdraw(shares).unwrap();
        // Should get back at most the same amount (rounding may cause 1-lamport diff)
        prop_assert!(withdrawn >= amount - 1 && withdrawn <= amount);
    }
}
```

---

## What to Formally Verify — Priority List

| Property | Tool | Priority |
|----------|------|----------|
| Arithmetic never overflows | Kani | Critical |
| Fee calculations correct | Kani + proptest | High |
| Pool invariants (AMM x*y=k) | Trident invariants | High |
| PDA seed uniqueness | Kani | Medium |
| Access control for all callers | Kani | High |
| State machine transitions | Kani | Medium |
| Economic invariants (supply) | Trident | High |
| Round-trip correctness | proptest | Medium |

---

## Writing Good Invariants

An invariant is a property that must hold **before and after every valid operation**.

```
Examples of good invariants:
✅ "Total supply = sum of all user balances"
✅ "Pool liquidity × price = constant (AMM)"
✅ "A closed account cannot be reopened"
✅ "Authority cannot change without authority's signature"
✅ "Fee amount is always ≤ transaction amount"

Bad invariants (too loose or untestable):
❌ "Things work correctly"
❌ "No bugs"
❌ "Users are satisfied"
```

For each invariant:
1. Write it as a mathematical formula
2. Implement it as an assertion function
3. Call it before and after every state-changing instruction in your fuzz harness
4. When Trident finds a violation, write a regression test immediately

---

## Formal Verification Report Section

When formal verification is part of the audit, include a section in the final report:

```markdown
## Formal Verification

### Tools Used
- Kani 0.57.0 — model checking for arithmetic invariants
- Trident 0.9.x — coverage-guided fuzzing with invariant assertions

### Properties Verified

| Property | Tool | Result | Harness |
|----------|------|--------|---------|
| fee_calculation_no_overflow | Kani | VERIFIED | verify_fee_calculation_no_overflow |
| deposit_increases_total | Kani | VERIFIED | verify_deposit_increases_total |
| total_supply_constant_on_transfer | Trident | VERIFIED (10M iterations) | fuzz_0 |
| withdraw_cannot_exceed_deposit | proptest | VERIFIED (1M cases) | fee_never_exceeds_amount |

### Properties NOT Formally Verified (and why)

| Property | Reason |
|----------|--------|
| Oracle price manipulation | Requires external oracle state — use manual review |
| Governance time-lock | Multi-transaction invariant — requires integration testing |
```
