# Differential & Upgrade Audit

Audit the **change**, not the whole program. The most common real-world audit request for a live protocol isn't "audit everything from scratch" — it's "we shipped v1, it was audited, now we're upgrading to v2; review the diff."

> Load this file when the user says: "audit this upgrade", "review the diff", "we changed X since the last audit", "audit v2 against v1", "what's the risk of this PR", "differential audit".

A diff audit is **higher leverage and higher risk** than a full audit: leverage because you focus only on what changed; risk because a small diff can break an invariant that the unchanged code silently depended on. The danger is almost never *in* the diff — it's in how the diff interacts with code you didn't read.

---

## The Differential Audit Mistake to Avoid

> "Only 40 lines changed, so it's a small audit."

Wrong frame. The correct frame: **what assumptions did the rest of the program make that this diff might now violate?**

A one-line change to a fee calculation can break an invariant relied on three instructions away. Scope by *blast radius of the change*, not by *line count of the change*.

---

## Phase D0 — Establish the Baseline

```bash
# What exactly changed, by source
git diff v1.0.0..v2.0.0 -- 'programs/**/*.rs'

# Get a file-level summary first
git diff --stat v1.0.0..v2.0.0 -- 'programs/**/*.rs'

# For a deployed program: also diff the BYTECODE, not just source
solana-verify get-program-hash -u mainnet-beta <PROGRAM_ID>   # current on-chain
solana-verify get-executable-hash target/deploy/program.so    # proposed v2
```

Record three things:
1. **Source diff** — the human-readable change set.
2. **Was v1 actually audited?** Get the prior report. If the baseline was never audited, this becomes a *scoped full audit*, not a diff audit — say so.
3. **State migration?** Does v2 change any account layout? If so, the diff includes a migration risk (see Phase D3).

---

## Phase D1 — Classify Each Changed Hunk

For every hunk in the diff, assign a category. The category sets how deep you go:

| Change category | Examples | Audit depth |
|-----------------|----------|-------------|
| **Account struct / state layout** | added/removed/reordered field, changed `space` | **Deepest** — migration + type confusion + every reader |
| **Access control** | signer/owner/PDA/authority checks added or removed | **Deepest** — re-run full account-validation checklist |
| **Arithmetic / economic** | fee math, reward calc, price logic, rounding | **Deep** — invariants + precision + overflow |
| **New instruction** | brand-new entrypoint | **Full** — treat as new code, full 25-pattern pass |
| **CPI change** | new/changed cross-program call | **Deep** — trust boundary + reentrancy + reload |
| **Logging / comments / formatting** | `msg!`, rename, whitespace | **Shallow** — confirm no behavior change only |

A "small" diff that touches account layout or access control is a **large** audit.

---

## Phase D2 — Invariant Regression Analysis (the core of a diff audit)

This is what separates a diff audit from "reading a PR". For each change, ask:

1. **What invariant did the OLD code maintain here?**
   (e.g. "`total_deposits == sum of all user balances`", "only `authority` can call `set_fee`", "`vault.bump` is canonical".)

2. **Does the new code still maintain it?**

3. **Did any UNCHANGED code depend on the old behavior?**
   This is where exploits hide. Search the unchanged codebase for every reader/writer of the fields and accounts the diff touches.

```bash
# After identifying that the diff changes `pool.fee_bps`, find everyone who reads it:
grep -rn "fee_bps" programs/ --include="*.rs"
# Each of those call sites is now in scope, even though it wasn't in the diff.
```

> **Rule**: a field touched by the diff pulls every reader of that field into scope, changed or not.

### Example failure mode

```
v1: withdraw() checked  require!(amount <= user.balance)
v2: refactor moved the balance check into a helper... that is only called on one of two paths
Diff looks clean (just extracted a function). The bug: the second path lost the check.
```

You only catch this by re-verifying the invariant ("withdraw can never exceed balance") across *all* paths, not by reading the diff in isolation.

---

## Phase D3 — State Migration Safety

If v2 changes any account layout, the upgrade is the most dangerous kind. A deployed program's existing accounts were serialized with the **old** layout.

Check:

- **Borsh deserialization compatibility**: will old accounts deserialize under the new struct? Adding a field at the end without a migration = old accounts fail to deserialize or are misread.
- **`space` / realloc**: does v2 need more space? Is there a `realloc` path, and is it funded to rent exemption and zero-initialized?
- **Migration instruction**: is there one? Is it idempotent? Is it permissioned? Can it be front-run or run twice?
- **Discriminator/version tag**: does the account carry a version byte so the program can distinguish old vs migrated accounts?

```rust
// SAFE pattern: versioned account that handles both layouts during migration
match account_version {
    1 => migrate_v1_to_v2(&mut data)?,
    2 => { /* already current */ }
    _ => return Err(ErrorCode::UnknownVersion.into()),
}
```

Report any layout change with no migration path as **Critical** (data corruption / fund lockup).

---

## Phase D4 — Upgrade Execution Safety

Even a perfect v2 can be deployed unsafely:

- **Authority**: who signs the upgrade? (See onchain-analysis.md — EOA vs multisig vs governance.)
- **Timelock**: can users exit before it lands?
- **Atomicity**: if v2 requires a migration, is there a window where the new code runs against un-migrated accounts? Order matters: deploy-then-migrate vs migrate-then-deploy can each leave an unsafe gap.
- **Rollback plan**: if v2 is buggy, can they revert? (Immutable-after-upgrade or buffer issues.)

---

## Differential Audit Report Section

Add this to the standard report (report-generation.md) for upgrade audits:

```markdown
## Upgrade Audit Summary

**Base version**: `v1.0.0` (commit `<hash>`, audited by `<firm>` on `<date>`)
**Target version**: `v2.0.0` (commit `<hash>`)
**Diff size**: <N> files, +<X>/-<Y> lines

### Change Inventory
| Hunk | File:Lines | Category | Invariant impact | Finding? |
|------|-----------|----------|------------------|----------|
| 1 | fees.rs:40-55 | Arithmetic | Rounding direction changed | FINDING-002 |

### Invariants Re-verified
| Invariant | Held in v1 | Holds in v2 | Verified by |
|-----------|-----------|-------------|-------------|

### Migration Assessment
- Layout changed: Yes/No
- Migration path: <described / none / N/A>
- Migration risk: <Critical/High/...>

### Out-of-diff code pulled into scope
<List unchanged call sites reviewed because they read/write changed state.>
```

---

## Diff-Audit Checklist

```
[ ] Source diff captured (git diff base..target)
[ ] Bytecode hashes compared (on-chain v1 vs built v2)
[ ] Prior audit report obtained; baseline confirmed audited
[ ] Every hunk categorized (layout / access / arithmetic / new ix / CPI / cosmetic)
[ ] For each change: old invariant identified, new behavior checked
[ ] All readers/writers of touched fields pulled into scope (grep-confirmed)
[ ] State migration safety assessed (Borsh compat, realloc, version tag, idempotency)
[ ] Upgrade execution safety assessed (authority, timelock, atomicity, rollback)
[ ] New instructions given a full 25-pattern pass
```
