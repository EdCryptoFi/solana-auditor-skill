# Audit Lifecycle

Six phases from engagement kick-off to final delivery. Follow in order. Do not skip phases for small programs — scope Phase 3 and 4 proportionally, but always do 0, 1, 2, 5, 6.

---

## Phase 0: Scoping

**Goal**: Understand what you're auditing before writing a single note.

### 0.1 Codebase Survey

```bash
# Count lines, identify files
find . -name "*.rs" | sort
find . -name "*.rs" | xargs wc -l | sort -rn | head -20

# Understand the dependency graph
cargo tree --workspace

# Check for upgradeable programs
grep -r "BpfLoader\|UpgradeableLoader\|set_upgrade_authority" --include="*.rs"

# Identify all instructions
grep -r "pub fn\|#\[instruction\]" programs/ --include="*.rs" | grep -v "//"

# Identify privileged roles
grep -r "has_one\|constraint\|signer\|authority\|admin\|owner" --include="*.rs" | head -40
```

### 0.2 Architecture Mapping

Document before reviewing code:

```
Program Architecture Template
═══════════════════════════════
Program ID: ___
Network: mainnet / devnet / both
Upgradeable: yes / no
Multisig authority: yes / no

Instructions:
  ├── [instruction name] — [brief purpose] — [signer required?]
  ├── ...

Account Types (PDAs):
  ├── [account name] — [seeds] — [mutable by whom?]
  ├── ...

External Programs Called (CPIs):
  ├── [program name] — [purpose] — [trusted?]
  ├── ...

Privileged Roles:
  ├── [role name] — [capabilities]
  ├── ...

Assets at Risk:
  ├── SOL: [vaults, amounts]
  ├── SPL tokens: [mints, vaults]
  ├── NFTs / compressed: [collections]
  ├── Protocol state: [PDAs with governance power]
```

### 0.3 Threat Model

For each asset at risk, enumerate:

```
Threat Model: [Program Name]
═══════════════════════════════
Asset: [e.g., SOL vault holding 10,000 SOL]
  Threats:
  1. Unauthorized withdrawal via missing signer check
  2. Reentrancy via CPI callback draining vault
  3. PDA seed collision allowing attacker-controlled vault
  4. Admin key compromise (check upgrade authority)
  5. Economic attack via price oracle manipulation
  6. Flash loan amplification of economic attack

Asset: [SPL token mint authority]
  Threats:
  1. Unauthorized mint via missing authority check
  2. Freeze authority bypass
  3. Metadata update without consent
```

Work through STRIDE for each privileged operation:
- **S**poofing: can attacker impersonate a signer?
- **T**ampering: can attacker modify account state they don't own?
- **R**epudiation: are critical actions logged/verifiable?
- **I**nformation disclosure: does state leak sensitive data?
- **D**enial of service: can attacker lock up the protocol?
- **E**levation of privilege: can attacker gain capabilities above their role?

### 0.4 Test Coverage Baseline

```bash
# Run existing tests to understand coverage
cargo test-sbf 2>&1 | tail -20

# Check test file count vs instruction count
find . -name "*.rs" -path "*/tests/*" | wc -l

# Look for coverage tooling
cargo llvm-cov test --workspace 2>/dev/null || echo "no coverage tool"
```

Record: `X instructions, Y test files, Z% branch coverage (if measurable)`

---

## Phase 1: Automated Analysis

**Goal**: Collect raw findings from tools before reading code manually.

Run all tools before manual review — don't let manual impressions bias tool results.

### 1.1 Dependency Audit

```bash
# Known CVE check
cargo audit

# Check for unmaintained crates
cargo audit --deny unmaintained

# Check licenses (GPL could be an issue)
cargo deny check licenses 2>/dev/null || echo "cargo-deny not installed"
```

### 1.2 Linting

```bash
# Security-focused clippy lints
cargo clippy -- \
  -W clippy::arithmetic_side_effects \
  -W clippy::checked_conversions \
  -W clippy::cast_possible_truncation \
  -W clippy::cast_sign_loss \
  -W clippy::integer_division \
  -W clippy::modulo_arithmetic \
  -W clippy::panic \
  -W clippy::unwrap_used \
  -W clippy::expect_used \
  2>&1 | tee audit-clippy.txt
```

### 1.3 Fuzzing with Trident

```bash
# Initialize fuzzing (once per project)
trident init

# Run fuzz targets
trident fuzz run fuzz_0

# Check for crashes
ls .trident/fuzzing/corpus/fuzz_0/crashes/ 2>/dev/null && echo "CRASHES FOUND"
```

See [tools-setup.md](tools-setup.md) for full Trident configuration.

### 1.4 Semgrep Pattern Matching

```bash
# Run Solana-specific rules
semgrep --config p/solana scan . 2>/dev/null

# Run custom audit rules
semgrep --config rules/solana-audit.yaml . 2>/dev/null || true
```

### 1.5 Record Raw Findings

Create `audit-workspace/findings-raw.md` with all tool output, tagged by tool:
```
[CARGO-AUDIT] advisory RUSTSEC-2024-XXXX in crate foo 0.1.0
[CLIPPY] src/lib.rs:45: arithmetic_side_effects: ...
[TRIDENT] crash in fuzz_0/crashes/id:000001 — ...
[SEMGREP] src/instructions/withdraw.rs:89: missing-signer-check
```

---

## Phase 2: Manual Review

**Goal**: Systematic line-by-line review. Load [vulnerability-patterns.md](vulnerability-patterns.md) for this phase.

### Review Order (highest risk first)

1. **Withdrawal / fund-transfer instructions** — highest impact
2. **Mint / burn instructions** — token supply risk
3. **Admin / privileged instructions** — governance risk
4. **Initialization instructions** — state setup risk
5. **CPI-heavy instructions** — trust boundary risk
6. **All other instructions**

### Per-Instruction Checklist

For every instruction, answer each question. Mark: ✅ Safe | ❌ Finding | ⚠️ Needs investigation

```
Instruction: [name]
─────────────────────────────────────────────────
Account Validation
  [ ] All accounts have owner checks where required
  [ ] All signing authorities have signer checks
  [ ] PDAs validated against expected seeds
  [ ] PDA bump loaded from account, not recalculated (canonical bump)
  [ ] No account type confusion (passing wrong account type)
  [ ] No account reuse between roles that should be distinct

Arithmetic
  [ ] No unchecked addition / subtraction / multiplication
  [ ] No integer truncation via `as u64` on user-supplied values
  [ ] Division by zero handled
  [ ] Interest/fee calculations use checked math

State Management
  [ ] Cannot reinitialize an already-initialized account
  [ ] Closing accounts: lamports reclaimed, data zeroed
  [ ] No stale data from previous instruction in single tx

CPI Safety
  [ ] CPI target program ID validated
  [ ] Signer seeds correct for PDA signers
  [ ] Return values / post-CPI state re-read if used
  [ ] No reentrancy: program state not in invalid mid-state during CPI

Token Operations
  [ ] Token account owner validated
  [ ] Token mint matches expected mint
  [ ] Token amount cannot overflow token supply
  [ ] SPL Token 2022 extensions respected (transfer hooks, fees)

Business Logic
  [ ] Invariants hold after every state transition
  [ ] Cannot double-spend within single transaction
  [ ] Economic assumptions can't be violated via flash loans
  [ ] Oracle / price feed staleness checked
  [ ] Slippage / output amount validated
```

### Cross-Instruction Analysis

After reviewing each instruction individually:

```
Cross-Instruction Analysis
───────────────────────────
[ ] Atomicity: Are multi-step operations vulnerable to partial execution?
[ ] Ordering: Do instructions have dangerous ordering dependencies?
[ ] Composability: Can instructions be chained unexpectedly?
[ ] Front-running: Do high-value operations leak info before execution?
[ ] Flash loan: Can flash-borrowed funds amplify any attack?
```

---

## Phase 3: Deep Analysis

**Goal**: Build working exploits for all Critical/High findings.

### 3.1 Proof of Concept Development

Every Critical and High finding needs a PoC that proves exploitability:

```rust
// Template: Exploit PoC
#[cfg(test)]
mod exploit_poc {
    use super::*;

    #[tokio::test]
    async fn poc_[finding_id]_[short_description]() {
        // Setup: describe initial state
        let mut context = setup_test_context().await;

        // Exploit: demonstrate the attack
        let attacker = Keypair::new();
        let malicious_tx = /* ... */;
        let result = context.process_transaction(malicious_tx).await;

        // Verify: show the bad outcome
        assert!(result.is_ok(), "exploit succeeded");
        // assert attacker gained funds / corrupted state / etc.
    }
}
```

### 3.2 Economic Attack Modeling

For DeFi / protocol programs:

```
Economic Attack Checklist
───────────────────────────
[ ] Flash loan amplification: Can $0-cost capital amplify any price/ratio attack?
[ ] Sandwich attack: Are swaps/oracles manipulable within the same block?
[ ] Griefing: Can attacker cause permanent economic loss to others?
[ ] Inflation: Can attacker mint unbounded tokens?
[ ] Drain: What is the maximum extractable value (MEV) from this program?
[ ] Oracle: Is the price feed TWAP-resistant? What's the manipulation cost?
```

### 3.3 Cross-Program Interaction Analysis

```bash
# Find all CPI calls
grep -r "invoke\|invoke_signed\|CpiContext" --include="*.rs" -n

# Find all cross-program accounts accepted
grep -r "Program<" --include="*.rs" -n
```

For each CPI:
- What if the target program is malicious?
- What if the target program calls back (reentrancy)?
- Is the returned state re-validated?

### 3.4 Upgrade/Governance Risk

```bash
# Check upgrade authority
solana program show <PROGRAM_ID> --url mainnet-beta

# Check if multisig protects upgrade
# Squads / SPL Governance / custom?
grep -r "upgrade_authority\|set_upgrade_authority" --include="*.rs"
```

---

## Phase 4: Formal Verification

Load [formal-verification.md](formal-verification.md) for this phase.

Short guide: use formal verification for:
- Critical invariants that must hold mathematically (e.g., "total supply never decreases")
- Arithmetic correctness proofs (overflow-free math)
- PDA uniqueness (no seed collisions)

---

## Phase 5: Reporting

Load [report-generation.md](report-generation.md) for this phase.

Key deliverables:
1. Executive Summary (1-2 pages)
2. Findings with severity, CVSS, PoC, and recommendations
3. Methodology appendix
4. Scope and limitations statement

---

## Phase 6: Remediation Verification

**Goal**: Verify every fix is correct, complete, and doesn't introduce new issues.

### 6.1 Per-Finding Verification Checklist

```
Finding [ID]: [title]
Original severity: [Critical/High/Medium/Low]
Fix commit: [hash]

Verification:
  [ ] Root cause addressed (not just symptom patched)
  [ ] Fix is complete (no partial fix)
  [ ] Fix does not introduce new vulnerabilities
  [ ] Regression test added that would have caught original issue
  [ ] PoC from Phase 3 now fails with the fix applied
  [ ] Re-test all adjacent instructions for side-effects
```

### 6.2 Re-run Automated Tools

```bash
# Re-run all Phase 1 tools on the patched code
cargo audit
cargo clippy -- [same flags as Phase 1]
trident fuzz run fuzz_0 --timeout 300
semgrep --config p/solana scan .
```

### 6.3 Re-scoring

For each finding, document the final state:
```
Finding [ID]: [title]
Original: Critical
Status: Fixed / Partially Fixed / Acknowledged / Won't Fix
Final severity: N/A (if fixed) / [downgraded severity if partially fixed]
Verification: PoC fails post-fix ✅ / Regression test added ✅
```

### 6.4 Audit Certificate Criteria

Mark audit complete only when:
- All Critical and High findings: Fixed or Acknowledged with written risk acceptance
- Regression tests added for every Critical/High fix
- All Medium findings: Fixed, Acknowledged, or accepted with mitigation plan
- Tool suite re-run clean (or known false positives documented)
- Final report updated with remediation status
