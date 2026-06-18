# Audit Tool Setup

Install and configure the full automated analysis suite. Run all tools in Phase 1 before any manual review.

---

## Prerequisites

```bash
# Rust toolchain
rustup update stable
rustup component add clippy rust-src

# Solana tool suite
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
solana --version  # expect 2.x

# Anchor CLI (if auditing Anchor programs)
avm install latest && avm use latest
anchor --version
```

---

## Tool 1: cargo-audit — Known CVE Scanner

```bash
# Install
cargo install cargo-audit --locked

# Run
cargo audit

# Fail CI on any advisory (not just vulnerabilities)
cargo audit --deny warnings

# Export JSON for report
cargo audit --json > audit-workspace/cargo-audit.json
```

**What it catches**: Known CVEs in dependencies, unmaintained crates, unsound crates, yanked versions.

**Interpret results**:
- `error[vulnerability]` — exploitable CVE, must investigate
- `warning[unmaintained]` — no active maintainer, medium risk
- `warning[unsound]` — unsafe code with memory safety issues

---

## Tool 2: cargo clippy — Security Lints

```bash
# Install (comes with rustup, just update)
rustup update

# Run with security-focused lints
cargo clippy --all-targets --all-features -- \
  -D clippy::arithmetic_side_effects \
  -D clippy::checked_conversions \
  -D clippy::cast_possible_truncation \
  -D clippy::cast_sign_loss \
  -D clippy::cast_possible_wrap \
  -D clippy::integer_division \
  -D clippy::modulo_arithmetic \
  -D clippy::panic \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::indexing_slicing \
  -D clippy::string_slice \
  2>&1 | tee audit-workspace/clippy-output.txt

# Count issues
grep "^error\[" audit-workspace/clippy-output.txt | wc -l
```

**Key lints explained**:

| Lint | Why it matters |
|------|---------------|
| `arithmetic_side_effects` | Catches unchecked math (overflow/underflow) |
| `cast_possible_truncation` | `u64 as u32` silently truncates |
| `unwrap_used` | Panics on `None`/`Err` in program logic |
| `indexing_slicing` | Out-of-bounds panic on user-controlled index |
| `integer_division` | Precision loss in financial calculations |

---

## Tool 3: Trident — Coverage-Guided Fuzzer

Trident is purpose-built for Anchor programs and finds bugs through instruction sequence fuzzing.

### Installation

```bash
# Install Trident CLI
cargo install trident-cli --locked

# Verify
trident --version
```

### Initialize in a Project

```bash
# From project root (where Anchor.toml lives)
trident init

# This creates:
# .trident/           — config and corpus
# trident-tests/      — fuzz harness templates
# Trident.toml        — configuration
```

### Configure Trident.toml

```toml
[fuzz]
fuzzing_with_stats = true
allow_duplicate_txs = false
exit_upon_crash = true

[[fuzz.programs_config]]
program_id = "YourProgramID11111111111111111111111111111"

[fuzz.accounts_snapshots]
# Enable account state snapshots for invariant checks
enabled = true
```

### Write a Fuzz Harness

```rust
// trident-tests/fuzz_tests/fuzz_0/src/lib.rs

use trident_client::fuzzing::*;
use your_program::*;

#[derive(Debug, FuzzTestExecutor, Default)]
pub struct FuzzTest {
    pub accounts: FuzzAccounts,
}

#[derive(Debug, Default)]
pub struct FuzzAccounts {
    authority: AccountsStorage<Keypair>,
    vault: AccountsStorage<PdaStore>,
    user_token_account: AccountsStorage<TokenStore>,
}

impl FuzzTest {
    fn fuzz_ix_initialize(
        &mut self,
        client: &mut impl FuzzClient,
        fuzz_accounts: &mut FuzzAccounts,
    ) -> Result<(), FuzzingError> {
        let authority = fuzz_accounts.authority.get_or_create_account(0, client, 1_000_000_000)?;
        
        let ix = Initialize {
            accounts: InitializeAccounts {
                authority: authority.pubkey(),
                vault: fuzz_accounts.vault.get_or_create_account(
                    0,
                    client,
                    &[b"vault", authority.pubkey().as_ref()],
                    &your_program::ID,
                )?,
                system_program: System::id(),
            },
            data: InitializeData {},
        };
        
        let _ = client.process_transaction(ix.to_transaction(client)?);
        Ok(())
    }

    fn fuzz_ix_withdraw(
        &mut self,
        client: &mut impl FuzzClient,
        fuzz_accounts: &mut FuzzAccounts,
    ) -> Result<(), FuzzingError> {
        let authority = fuzz_accounts.authority.get_or_create_account(0, client, 0)?;
        
        let ix = Withdraw {
            accounts: WithdrawAccounts {
                authority: authority.pubkey(),
                vault: fuzz_accounts.vault.get_or_create_account(
                    0,
                    client,
                    &[b"vault", authority.pubkey().as_ref()],
                    &your_program::ID,
                )?,
                system_program: System::id(),
            },
            data: WithdrawData {
                amount: FuzzData::fuzz_u64(),
            },
        };
        
        let _ = client.process_transaction(ix.to_transaction(client)?);
        Ok(())
    }
}
```

### Run and Analyze

```bash
# Run fuzzer (starts immediately)
trident fuzz run fuzz_0

# Run with time limit
trident fuzz run fuzz_0 -- -max_total_time=3600  # 1 hour

# Check for crashes
ls .trident/fuzzing/corpus/fuzz_0/crashes/ && echo "CRASHES FOUND — investigate!"

# Replay a crash
trident fuzz debug fuzz_0 .trident/fuzzing/corpus/fuzz_0/crashes/id:000001

# View stats
trident fuzz run fuzz_0 -- -print_final_stats=1
```

---

## Tool 4: Kani — Rust Model Checker

See [formal-verification.md](formal-verification.md) for full Kani usage. Quick setup:

```bash
# Install
cargo install --locked kani-verifier
cargo kani setup

# Run all proofs
cargo kani

# Run a specific harness
cargo kani --harness my_proof_harness
```

---

## Tool 5: semgrep — Pattern Matching

```bash
# Install
pip3 install semgrep
# or: brew install semgrep

# Run Solana-specific community rules
semgrep --config p/solana .

# Run with custom rules (see below)
semgrep --config audit-workspace/rules/ .

# Output JSON for report
semgrep --config p/solana --json . > audit-workspace/semgrep.json
```

### Custom semgrep Rules for Common Patterns

semgrep is a **lead generator, not an oracle**. It is good at flagging concrete syntactic anti-patterns (`unwrap`, raw casts, raw deserialization) and bad at semantic checks like "is this authority verified", which require dataflow it can't reliably do on Rust. Write rules that surface *candidates to review*, and don't pretend a clean semgrep run means anything (see rules/audit-discipline.md Rule 9).

```yaml
# audit-workspace/rules/solana-audit.yaml
rules:
  # AccountInfo authorities are the #1 place a signer check goes missing.
  # This flags the candidate; the auditor confirms whether is_signer is enforced.
  - id: accountinfo-authority-candidate
    pattern-regex: '(authority|admin|owner|signer)\s*:\s*(AccountInfo|UncheckedAccount)'
    message: "AccountInfo/UncheckedAccount used for an authority-like account. Confirm an explicit is_signer (and ownership) check exists — Anchor does NOT enforce it for these types."
    languages: [rust]
    severity: WARNING
    paths: { include: ["programs/**"] }

  # Raw deserialization with no owner check (type confusion / missing owner, vuln #2, #6).
  - id: raw-deserialize-candidate
    patterns:
      - pattern-either:
          - pattern: $T::try_from_slice($DATA)
          - pattern: $T::deserialize(&mut $DATA)
      - pattern-not-inside: "#[cfg(test)] ..."
    message: "Raw Borsh deserialization. Confirm the account owner is checked before trusting these bytes (prefer Anchor Account<'info, T>)."
    languages: [rust]
    severity: WARNING
    paths: { include: ["programs/**"] }

  # Silent truncation (vuln #11).
  - id: lossy-numeric-cast
    pattern-either:
      - pattern: $X as u8
      - pattern: $X as u16
      - pattern: $X as u32
      - pattern: $X as u64
    message: "`as` cast can silently truncate/wrap. Use try_from with an error on overflow in value paths."
    languages: [rust]
    severity: INFO
    paths: { include: ["programs/**"] }

  # invoke without `?` — ignored CPI result (vuln #13).
  - id: invoke-result-ignored
    pattern-regex: '^\s*invoke(_signed)?\([^;]*\);'
    message: "CPI return value not propagated with `?`; a failing CPI is silently ignored."
    languages: [rust]
    severity: WARNING
    paths: { include: ["programs/**"] }

  - id: unwrap-in-program
    patterns:
      - pattern: $EXPR.unwrap()
      - pattern-not-inside: "#[cfg(test)] ..."
    message: "unwrap() will panic (DoS) on None/Err; use ? or map_err() in program code."
    languages: [rust]
    severity: WARNING
    paths: { include: ["programs/**"] }
```

> Triage every hit. The authority and raw-deserialize rules are intentionally noisy — their value is forcing you to *look at* each authority account and each manual deserialization, which is exactly where Critical findings hide.

---

## Tool 6: Mollusk — Fast Program Testing

```bash
# Add to dev-dependencies in Cargo.toml
# [dev-dependencies]
# mollusk-svm = "0.3"

# Run tests
cargo test --features test-sbf
```

See [formal-verification.md](formal-verification.md) for Mollusk invariant test patterns.

---

## Tool 7: LiteSVM — Simulation for Security Tests

```bash
# Add to dev-dependencies
# [dev-dependencies]
# litesvm = "0.5"
# solana-sdk = "2"

# Run security tests
cargo test
```

LiteSVM allows time manipulation (for time-lock bypass tests) and account injection (for testing with real mainnet state snapshots).

---

## Audit Workspace Setup Script

Run this once at the start of every audit to create a consistent workspace:

```bash
#!/bin/bash
# audit-workspace/setup.sh

mkdir -p audit-workspace/{raw-findings,pocs,reports}

# Record tool versions
echo "=== Audit Tool Versions ===" > audit-workspace/versions.txt
date >> audit-workspace/versions.txt
echo "" >> audit-workspace/versions.txt
rustc --version >> audit-workspace/versions.txt
cargo --version >> audit-workspace/versions.txt
solana --version >> audit-workspace/versions.txt
anchor --version >> audit-workspace/versions.txt 2>/dev/null || echo "anchor: not installed" >> audit-workspace/versions.txt
cargo audit --version >> audit-workspace/versions.txt
trident --version >> audit-workspace/versions.txt 2>/dev/null || echo "trident: not installed" >> audit-workspace/versions.txt
semgrep --version >> audit-workspace/versions.txt 2>/dev/null || echo "semgrep: not installed" >> audit-workspace/versions.txt

cat audit-workspace/versions.txt

# Create findings tracker
cat > audit-workspace/findings-tracker.md << 'EOF'
# Findings Tracker

| ID | Title | Severity | Phase Found | Status | File:Line |
|----|-------|----------|------------|--------|-----------|
| FINDING-001 | | | | Open | |

## Raw Tool Findings
(populated during Phase 1)

## Manual Review Findings
(populated during Phase 2)
EOF

echo "Audit workspace initialized at audit-workspace/"
```

---

## CI Integration

Add to `.github/workflows/audit-ci.yml`:

```yaml
name: Security Audit CI

on: [push, pull_request]

jobs:
  security-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: cargo audit
        run: |
          cargo install cargo-audit --locked
          cargo audit --deny warnings

      - name: clippy security lints
        run: |
          cargo clippy --all-targets -- \
            -D clippy::arithmetic_side_effects \
            -D clippy::unwrap_used \
            -D clippy::cast_possible_truncation

      - name: semgrep
        uses: semgrep/semgrep-action@v1
        with:
          config: p/solana
```
