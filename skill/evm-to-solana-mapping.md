# EVM → Solana Vulnerability Mapping

Cross-chain reference for auditors coming from Ethereum or working on multi-chain protocols. Maps SWC Registry / Slither / EVM vulnerability classes to their Solana equivalents.

**When to load**: Auditor has EVM background, protocol was ported from EVM, or audit scope includes cross-chain comparison.

---

## Direct Equivalents

| EVM Vulnerability | SWC # | Solana Equivalent | Severity (Solana) |
|------------------|-------|------------------|-------------------|
| Reentrancy | SWC-107 | CPI reentrancy (vuln #12) | High |
| Access Control | SWC-115 | Missing signer/owner check (#1, #2) | Critical |
| Integer Overflow | SWC-101 | Integer overflow (#9) | Critical |
| Unchecked Return Value | SWC-104 | CPI return value ignored (#13) | Medium |
| Unprotected Upgrade | SWC-124 | Unconstrained upgrade authority (#20) | High |
| Time Manipulation | SWC-116 | Clock manipulation (#23) | Medium |
| Tx.origin | SWC-115 | No direct equivalent — Solana has no tx.origin |
| Delegatecall | SWC-112 | Arbitrary CPI (#3) | Critical |
| DoS with Block Gas | SWC-128 | Compute unit exhaustion (#25) | Medium |
| Uninitialized Storage | SWC-109 | Missing initialization check (#7) | High |
| Front-Running | SWC-114 | Pre-signed tx MEV — different model |
| Short Address | SWC-133 | Account data length mismatch (#24) | Medium |

---

## Deep Comparison: Reentrancy

### EVM (Solidity)
```solidity
// VULNERABLE: EVM reentrancy via fallback function
contract Vault {
    mapping(address => uint) balances;
    
    function withdraw() external {
        uint amount = balances[msg.sender];
        // External call BEFORE state update
        (bool success,) = msg.sender.call{value: amount}("");
        require(success);
        balances[msg.sender] = 0;  // ← too late, already re-entered
    }
}
```

### Solana (Anchor)
```rust
// VULNERABLE: Solana reentrancy via CPI
pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
    let rewards = calculate_rewards(&ctx.accounts.user_stake);
    
    // CPI to token program (or other program that can call back)
    token::transfer(ctx.accounts.into_cpi_ctx(), rewards)?;
    
    // State update AFTER CPI — same pattern, different mechanism
    ctx.accounts.user_stake.last_claimed = Clock::get()?.unix_timestamp;
    Ok(())
}
```

**Key difference**: On Solana, reentrancy requires a malicious target program in the CPI call. The System Program and SPL Token Program are safe, but custom programs in CPI chains can call back. Solana's runtime prevents direct re-entry into the same instruction, but can call OTHER instructions of the same program.

**Solana-specific mitigation**: Checks-effects-interactions pattern is identical. Update state before ANY CPI.

---

## Deep Comparison: Access Control

### EVM
```solidity
// VULNERABLE: missing onlyOwner modifier
function setFee(uint newFee) external {
    // No access control — anyone can call
    fee = newFee;
}
```

### Solana
```rust
// VULNERABLE: missing signer constraint
pub fn set_fee(ctx: Context<SetFee>, new_fee: u64) -> Result<()> {
    // No check that admin signed — anyone can call
    ctx.accounts.config.fee = new_fee;
    Ok(())
}
```

**Key difference**: EVM uses `msg.sender` for identity. Solana uses account ownership + `is_signer` flag. Anchor's `Signer<'info>` enforces signing, but only if used — `AccountInfo<'info>` does NOT enforce it. The bug manifests differently: on EVM you forget the modifier, on Solana you use the wrong account type.

---

## Deep Comparison: Integer Overflow

### EVM
In Solidity < 0.8.0, arithmetic wraps silently. Solidity 0.8.0+ added automatic revert on overflow. Most EVM contracts now rely on this by default.

### Solana
In Rust with the SBF target, `overflow-checks = true` is set by default since Solana 1.14, making overflows panic (like a revert) rather than wrap. **However**: this causes a DoS (program panics) rather than a security bypass. You must still use `checked_add`/etc. to return a proper error instead of panicking.

```rust
// Causes panic (DoS) — bad for program liveness
let total = a + b;  // panics if a + b > u64::MAX

// Returns typed error — correct
let total = a.checked_add(b).ok_or(ErrorCode::Overflow)?;
```

**Audit focus**: On Solana, unchecked arithmetic is a DoS risk even if overflow detection is enabled. Check for it in any loop or user-controlled value path.

---

## EVM-Only Patterns (No Solana Equivalent)

| EVM Pattern | Why Solana is Different |
|-------------|------------------------|
| `tx.origin` phishing | Solana has no `tx.origin` — signers are explicit per-account |
| Proxy storage collision | Solana uses explicit PDA seeds, no storage slots |
| Signature malleability | Ed25519 signatures on Solana are not malleable |
| Selfdestruct | Solana programs can be upgraded but not self-destructed |
| Block hash randomness | Solana uses VRF (Switchboard) — different model |
| Create2 collision | PDA derivation is deterministic by design, collisions impossible |

---

## Solana-Only Patterns (No EVM Equivalent)

| Solana Pattern | Description |
|---------------|-------------|
| PDA bump canonicalization | User-supplied bump seed allows different PDA derivation |
| Account type confusion | Two account types with same layout — discriminant bypass |
| CPI signer seed mismatch | invoke_signed uses wrong seeds for PDA |
| Rent exemption | Accounts below rent-exempt threshold get garbage-collected |
| SPL Token 2022 extensions | Transfer hooks, fees, confidential transfers |
| Account ownership | Solana's account ownership model has no EVM equivalent |

---

## Formal Verification Cross-Chain

| Chain | Tool | Approach |
|-------|------|---------|
| EVM | Certora Prover | Specification language (CVL), symbolic execution |
| EVM | Echidna | Property-based fuzzing |
| Move/Sui | Move Prover | Built-in formal spec language |
| CosmWasm | cosmwasm-check | Static analysis |
| Solana | Kani | Rust model checking |
| Solana | Trident | Coverage-guided fuzzing |
| Solana | QEDGen | Spec-driven (`.qedspec` → Kani/Lean) |

**Key gap**: Solana's formal verification ecosystem is less mature than EVM (Certora) or Move (Move Prover). Kani requires manual harness writing; there is no Solana equivalent of Certora's automatic CVL verification. QEDGen is the closest attempt at a specification-first approach.

---

## For Ethereum Auditors Transitioning to Solana

**Reframe your mental model**:

| EVM concept | Solana equivalent |
|-------------|------------------|
| Smart contract | Program (stateless executable) |
| Contract storage | Account data (owned by program) |
| `msg.sender` | Signer account in `Context` |
| `address(this)` | `program_id` |
| `transfer()` | `system_instruction::transfer` via CPI |
| ERC-20 | SPL Token |
| ERC-721 | Metaplex NFT |
| Proxy | Upgradeable BPF program |
| Event | `emit!()` macro (Anchor) / log_data |

**What's strictly harder on Solana**:
- Account ownership model requires explicit validation at every instruction
- CPI trust boundaries require careful reasoning for every external call
- PDA derivation and canonical bumps require understanding to audit correctly
- No EVM-level transaction revert — panics are DoS, proper errors require `?` propagation
- SPL Token 2022 extensions add complexity that most audit tools don't fully cover
