# EVM → Solana Vulnerability Mapping

Cross-chain reference for auditors with an EVM background, teams porting protocols from Ethereum, or anyone doing cross-chain comparative security analysis.

**When to load**: Auditor has EVM background, protocol was ported from EVM, team is comparing security posture across chains, or scope includes cross-chain bridges.

---

## Mental Model Shift: Account Model vs. Contract Storage

The single biggest conceptual difference:

```
EVM (Ethereum)
──────────────
Contract = code + storage in one address
msg.sender identifies the caller
State lives inside the contract
Access control: modifier checks msg.sender

Solana
──────
Program = stateless code only (no built-in storage)
Accounts = external state, explicitly passed per instruction
Ownership: every account has an owner program
Access control: check account.is_signer + account.owner
```

**Practical implication for auditors**: On EVM, you ask "who called this function?" On Solana, you ask "who passed which accounts, and are those accounts what they claim to be?"

---

## Complete SWC Registry → Solana Mapping

### SWC-100: Function Default Visibility
**EVM**: Functions default to public in Solidity < 0.5; forgotten `private` keyword.  
**Solana equivalent**: Instructions are always callable by anyone unless you add signer checks. There's no "private" instruction — any instruction can be called if the accounts are assembled correctly. **Audit**: check every instruction for access control, not just "admin" ones.

---

### SWC-101: Integer Overflow and Underflow
**EVM**: Solidity < 0.8 wraps silently. Fixed with SafeMath or ≥ 0.8.  
**Solana**: Rust with `overflow-checks = true` panics (DoS) instead of wrapping. Use `checked_add`/`saturating_add` to return typed errors instead of panicking.  
**Key difference**: EVM overflow = silent wrong value. Solana overflow = program panic (DoS). Both are bugs, different impact.

```rust
// Solana: turn panic into typed error
let new_balance = old_balance
    .checked_add(amount)
    .ok_or(error!(ErrorCode::Overflow))?;
```

---

### SWC-102: Outdated Compiler Version
**EVM**: Using old Solidity with known bugs.  
**Solana equivalent**: Using outdated Anchor version (`anchor-lang < 0.29`) with known vulnerabilities, or outdated Rust toolchain. **Audit**: check `Cargo.toml` for pinned dependency versions, run `cargo audit`.

---

### SWC-103: Floating Pragma
**EVM**: `pragma solidity ^0.8.0` allows any compatible version.  
**Solana equivalent**: Unpinned Cargo dependencies (`anchor-lang = "*"` or `anchor-lang = ">=0.28"`). **Safe**: pin exact versions in `Cargo.lock`.

---

### SWC-104: Unchecked Return Value from External Calls
**EVM**: `address.call()` returns bool; ignoring it means failures are silent.  
**Solana**: `invoke()` returns `ProgramResult`; ignoring it with `let _ = invoke(...)` is the same pattern.  

```rust
// VULNERABLE
let _ = invoke(&ix, &accounts);

// SAFE
invoke(&ix, &accounts)?;
```

---

### SWC-105: Unprotected Ether Withdrawal
**EVM**: Anyone can call `withdraw()` because `onlyOwner` is missing.  
**Solana equivalent**: Missing signer check on withdrawal instruction — the most common Critical on Solana. `AccountInfo<'info>` for authority instead of `Signer<'info>`.

---

### SWC-106: Unprotected SELFDESTRUCT Instruction
**EVM**: `selfdestruct` sends all balance to any address and destroys contract.  
**Solana equivalent**: Closing an account with `close = receiver` — if the close target is user-supplied and unchecked, funds go to attacker. Also: upgradeable program with unprotected upgrade authority.

```rust
// VULNERABLE: attacker supplies their address as receiver
#[account(mut, close = receiver)]  // receiver must be validated
pub vault: Account<'info, Vault>,
pub receiver: SystemAccount<'info>,  // ← no check that this is legitimate
```

---

### SWC-107: Reentrancy
**EVM**: Classic fallback function reentrancy. Fixed with ReentrancyGuard or CEI pattern.  
**Solana**: CPI reentrancy — external program called mid-operation can call back. Direct same-instruction reentrancy is blocked by Solana's runtime, but calling other instructions of the same program through a CPI chain is possible.

**Key difference**: On Solana, only **custom** programs in CPI chains can reenter. SPL Token and System Program are safe. A DeFi program calling a custom hook program that calls back is vulnerable.

**Fix**: Checks-Effects-Interactions — update all state before any CPI call.

---

### SWC-108: State Variable Default Visibility
**EVM**: `uint x;` defaults to `internal`; layout can leak state.  
**Solana equivalent**: All on-chain account data is fully public regardless of how it's structured. No concept of private state. **Audit implication**: any "sensitive" data stored in an account (private keys, admin plans, strategy parameters) is public to all validators and indexers.

---

### SWC-109: Uninitialized Storage Pointer
**EVM**: Uninitialized local storage variable in Solidity < 0.5 points to storage slot 0.  
**Solana equivalent**: Reinitialize attack — calling `init` on an already-initialized account resets its state. Anchor's `init` constraint prevents this; native programs must check manually.

```rust
// SAFE with Anchor
#[account(
    init,  // fails if account already has data
    payer = payer,
    space = MyAccount::LEN,
)]
pub account: Account<'info, MyAccount>,
```

---

### SWC-110: Assert Violation
**EVM**: `assert()` uses all remaining gas; use `require()` instead.  
**Solana equivalent**: `panic!()` consumes all remaining compute units; use `require!()` (Anchor) or return `Err(...)` instead. Never `unwrap()` on user-controlled paths.

---

### SWC-111: Use of Deprecated Functions
**EVM**: `throw`, `sha3`, `suicide`, `callcode`.  
**Solana equivalent**: `invoke` with a token program when `invoke_signed` is needed for PDA signers; using `solana_program::program_pack::Pack::unpack_unchecked` instead of checked variants; using deprecated `spl_token::instruction::transfer` for Token 2022 accounts instead of `transfer_checked`.

---

### SWC-112: Delegatecall to Untrusted Callee
**EVM**: `delegatecall` executes external code in caller's storage context — perfect for proxies, dangerous when callee is attacker-controlled.  
**Solana equivalent**: Arbitrary CPI — calling an untrusted program that has access to the caller's accounts. The CPI context includes the caller's accounts, so a malicious callee can read/write them.

**Detection**:
```bash
grep -rn "invoke\b\|CpiContext::new\b" programs/ --include="*.rs" | grep -v "spl_token::\|system_instruction::\|anchor_spl::\|target/"
```
Every `invoke` to a user-supplied program ID is a potential arbitrary CPI.

---

### SWC-113: DoS with Failed Call
**EVM**: Looping over addresses and calling each — one failure reverts all.  
**Solana equivalent**: Iterating over accounts where one invalid account causes the entire instruction to fail, bricking the protocol for all users. Also: compute unit exhaustion via unbounded loops.

```rust
// VULNERABLE: O(n) loop on user-supplied array
for item in ctx.accounts.items.iter() {
    process(item)?;  // one failure bricks everything
}

// SAFER: bound the array + skip-on-error
require!(ctx.accounts.items.len() <= MAX_ITEMS, Error::TooMany);
for item in ctx.accounts.items.iter() {
    if let Err(_) = process(item) {
        emit!(ItemSkipped { item: item.key() });
        // continue rather than abort all
    }
}
```

---

### SWC-114: Transaction Order Dependence (Front-Running)
**EVM**: Public mempool allows miners/searchers to reorder transactions.  
**Solana equivalent**: Jito bundles enable MEV; validators can order within a block. Less severe than EVM due to fast finality (~400ms slots), but slot leaders can front-run within their slot. AMM swaps, oracle updates, and liquidations are all front-runnable.

**Mitigation**: Slippage parameters enforced on-chain; TWAP oracles instead of spot price; private RPC endpoints (Jito's protected bundle submission).

---

### SWC-115: Authorization through tx.origin
**EVM**: Using `tx.origin` instead of `msg.sender` — phishing via intermediate contract.  
**Solana equivalent**: No direct equivalent — Solana has no `tx.origin`. But: validating that a PDA signer's seeds match without verifying the PDA was derived from the *expected* program creates a similar bypass path.

---

### SWC-116: Block Values as a Proxy for Time
**EVM**: `block.timestamp` can be manipulated by miners ±~30 seconds.  
**Solana equivalent**: `Clock::get()?.unix_timestamp` can be manipulated by validators within ~1-2 seconds. For locks, vesting, and time-based conditions:  
- Safe for day-scale granularity (vesting cliff in days)  
- Risky for second-scale precision (flash loan within one block)  
- Use slot number (monotonic) for ordering, not timing

---

### SWC-120: Weak Sources of Randomness from Chain Attributes
**EVM**: Using `blockhash` or `block.timestamp` as randomness source.  
**Solana equivalent**: Using `recent_blockhash` or `Clock::get()?.slot` as randomness — validators can influence these. **Safe alternative**: Switchboard VRF or Pyth randomness. Never use chain values for high-value randomness (NFT rarity, lottery outcomes).

```rust
// VULNERABLE: predictable "randomness"
let pseudo_random = recent_blockhash[0] as u64 % 100;

// SAFE: Switchboard VRF or commit-reveal scheme
```

---

### SWC-124: Write to Arbitrary Storage Location
**EVM**: Unchecked array index writes to arbitrary storage slot.  
**Solana equivalent**: Writing to account data at an attacker-controlled offset — rare in Anchor (Account type prevents this), but possible in native programs with raw `data.borrow_mut()` access.

---

### SWC-128: DoS With Block Gas Limit
**EVM**: Unbounded loop or large array causes transaction to exceed gas limit.  
**Solana equivalent**: Compute unit exhaustion — unbounded loops or deeply nested CPIs exceed the 1.4M compute unit limit. Also: stack frame overflow (>4096 bytes).

```bash
# Check for compute budget requests
grep -rn "ComputeBudgetInstruction\|request_compute_units" --include="*.rs" -r
# Programs doing complex math should explicitly request and document their CU budget
```

---

## DeFi-Specific Cross-Chain Patterns

### AMM Security

| Pattern | EVM (Uniswap) | Solana (Orca/Raydium) |
|---------|--------------|----------------------|
| Price oracle | `slot0` TWAP | Pyth / Switchboard |
| Spot price manipulation | Flash loan via `swap` callback | Flash loan via CPI within same tx |
| Slippage enforcement | `amountOutMinimum` | `minimum_amount_out` on-chain |
| Fee accounting | `feeGrowthGlobal0X128` | Per-tick fee growth in Whirlpools |
| Position management | ERC-721 NFT | PDA position account |

**Common AMM bugs on both chains**:
- Missing slippage check (allows sandwich attacks)
- Spot price used as oracle (flash loan manipulation)
- Fee rounding in favor of attacker (precision loss)

### Lending Protocol Security

| Pattern | EVM (Aave/Compound) | Solana (Kamino/Marginfi) |
|---------|---------------------|--------------------------|
| Collateral oracle | Chainlink price feed | Pyth / Switchboard |
| Health factor | On-chain calculation | On-chain calculation |
| Liquidation | Callback to liquidator | CPI to liquidator |
| Interest accrual | Per-block index | Per-slot or per-second |
| Bad debt | Socialized or reserve fund | Reserve fund / insurance |

**Lending-specific bugs to audit**:
1. Oracle staleness in liquidation path (both chains)
2. Flash loan + price manipulation → self-liquidation → profit
3. Interest accrual rounding always favoring borrower
4. Bad debt socialization calculation overflow

### Bridge Security (Cross-Chain)

This is where Solana-specific patterns diverge most significantly from EVM:

| EVM Bridge | Solana Bridge (Wormhole) |
|------------|--------------------------|
| Signature verification | Guardian VAA validation |
| Replay protection | Sequence number on VAA |
| Token locking | Token vault program |
| Mint authority | Token mint authority PDA |

**Wormhole-specific audit points**:
- VAA signature verification: were all required guardians verified?
- Sequence number checked to prevent VAA replay?
- Token bridge: verify that minted amount matches locked amount
- Program upgrade authority: is the bridge upgradeable, and by whom?

The **$320M Wormhole exploit (2022)** was a missing signature verification check — the classic Solana pattern #1 (missing signer check) at the bridge level.

---

## Testing Tool Equivalences

| EVM Tool | Purpose | Solana Equivalent |
|----------|---------|------------------|
| Slither | Static analysis | cargo clippy + semgrep (custom rules) |
| Echidna | Property fuzzing | Trident |
| Foundry Fuzz | Unit test fuzzing | proptest + Mollusk |
| Hardhat | Development framework | Anchor |
| Foundry | Test framework | LiteSVM / solana-program-test |
| Certora Prover | Formal verification | Kani (less automatic) |
| MythX | Automated scanner | No direct equivalent (Sec3 is closest) |
| Tenderly | Simulation | Surfpool / litesvm simulation |
| 4naly3er | Gas/CU profiling | LiteSVM CU measurement |

---

## Formal Verification Cross-Chain Comparison

| Chain | Tool | Approach | Maturity |
|-------|------|---------|---------|
| EVM | Certora Prover | CVL spec language → automated proof | Production-grade, widely used |
| EVM | Echidna | Property-based fuzzing | Production-grade |
| Move/Sui | Move Prover | Built-in spec language, automated | Production-grade |
| Move/Aptos | Move Prover | Same as Sui | Production-grade |
| CosmWasm | cosmwasm-check | Static analysis only | Limited |
| Solana | Kani | Rust model checking, manual harness | Beta, improving |
| Solana | Trident | Coverage-guided fuzzing | Production-grade |
| Solana | QEDGen | Spec-first (`.qedspec`), multi-backend | Early production |

**Key gap**: Solana has no equivalent to Certora's CVL — a high-level specification language that automatically generates proofs. Kani requires manual Rust harnesses for each property. QEDGen is the most ambitious attempt to close this gap.

---

## Gas / Compute Unit Comparison

| Concept | EVM | Solana |
|---------|-----|--------|
| Unit | Gas | Compute Units (CU) |
| Limit per tx | 30M gas (mainnet) | 1.4M CU |
| Typical simple transfer | ~21,000 gas | ~300 CU |
| Complex DeFi tx | 200k–500k gas | 50k–300k CU |
| Pricing | Dynamic EIP-1559 | 0.000001 SOL per 10k CU (priority fee optional) |
| Stack limit | Unlimited (heap) | 4096 bytes (stack overflow = crash) |

**Audit implication**: Solana CU exhaustion is DoS, not just waste. A loop over N user-supplied accounts that hits the CU limit causes the entire transaction to fail — bricking a protocol for all users if N is unbounded.

---

## Toolchain and Security Configuration Comparison

### Dependency Security

| EVM | Solana |
|-----|--------|
| `npm audit` for JS packages | `cargo audit` for Rust crates |
| `yarn.lock` / `package-lock.json` | `Cargo.lock` |
| OpenZeppelin (audited library) | Anchor (audited, widely used) |
| Hardhat / Foundry plugins | Trident, LiteSVM |

### Build Security Config

```toml
# Cargo.toml — security-relevant settings
[profile.release]
overflow-checks = true   # panic on overflow (equivalent to Solidity 0.8+ checked math)
debug = false            # don't leak debug info in production

[dependencies]
anchor-lang = "=0.30.1"  # pin exact version, don't use ^
```

---

## For EVM Auditors: What to Unlearn

Coming from EVM, these intuitions will mislead you on Solana:

| EVM intuition | Solana reality |
|---------------|---------------|
| "Storage variables have access control" | All Solana account data is public |
| "msg.sender is the authenticated caller" | No `msg.sender` — check `is_signer` on specific accounts |
| "Contracts own their state" | Programs don't have built-in storage — state is in separate accounts |
| "Modifiers add access control" | Constraints in `#[derive(Accounts)]` or manual checks in the function body |
| "ERC-20 approval is sufficient" | SPL Token has different approval model + Token 2022 has extensions |
| "Flash loans need a callback" | Solana: all flash loan operations in a single atomic transaction (no callback needed) |
| "Reentrancy requires same contract" | Solana: reentrancy via CPI to any custom program that calls back |
