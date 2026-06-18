# Native & Pinocchio Program Patterns

Vulnerability patterns for programs written **without Anchor** — raw `solana-program` native programs and, increasingly in 2026, **Pinocchio** programs.

> Load this file when auditing a program that does not use the `#[program]` / `#[derive(Accounts)]` macros — i.e. you see a manual `process_instruction` / `entrypoint!`, `TryFrom` account parsing, or `#![no_std]` + `pinocchio`.

The core vulnerability-patterns.md checklist still applies. **The critical difference**: Anchor enforces owner checks, discriminators, signer types, and rent automatically. Native and Pinocchio programs enforce **none of this by default** — every check is the developer's responsibility, so every check is a place a check can be *missing*. Audit native programs assuming nothing is validated until you see the line that validates it.

---

## Why this matters

| Guarantee | Anchor | Native / Pinocchio |
|-----------|--------|--------------------|
| Owner check on typed accounts | Automatic (`Account<'info, T>`) | **Manual** — must compare `account.owner()` |
| 8-byte type discriminator | Automatic | **Manual or absent** — type confusion risk |
| Signer enforcement | `Signer<'info>` type | **Manual** — must check `is_signer` |
| Rent-exempt on init | `init` constraint | **Manual** — must fund + size correctly |
| Account count / order | Generated deserializer | **Manual** — index into `accounts[]` by hand |
| Data length validation | Generated | **Manual** — slice indexing can panic |

Pinocchio is zero-dependency, `no_std`, and macro-free by design (Helius/Anza). It buys performance and CU savings by removing the safety scaffolding. That is exactly why it needs heavier audit attention.

---

## N1. Missing Owner Check (native)

**Severity**: Critical

In native programs nothing checks who owns an account before you deserialize it. The single most common native exploit (Cashio, $52M class).

```rust
// VULNERABLE — deserializes without owner check
let state = State::try_from_slice(&account.data.borrow())?;
// attacker passes an account they own, with bytes laid out like State
```

```rust
// SAFE — assert ownership before trusting the bytes
if account.owner != program_id {
    return Err(ProgramError::IllegalOwner);
}
let state = State::try_from_slice(&account.data.borrow())?;
```

Pinocchio equivalent:

```rust
if unsafe { account.owner() } != &crate::ID {
    return Err(ProgramError::IllegalOwner);
}
```

**Detection**: For every account whose data is read, trace backwards to find an `owner` comparison. No comparison = finding.

---

## N2. Missing Signer Check (native)

**Severity**: Critical

```rust
// VULNERABLE — authority never checked for signature
pub fn withdraw(accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let authority = &accounts[1];
    // proceeds without authority.is_signer
}
```

```rust
// SAFE
if !authority.is_signer {
    return Err(ProgramError::MissingRequiredSignature);
}
```

Pinocchio: `if !authority.is_signer() { return Err(...) }`.

**Detection**: every privileged instruction must check `is_signer` on its authority before mutating state or moving funds.

---

## N3. Missing Discriminator → Type Confusion

**Severity**: High

Anchor prepends an 8-byte discriminator to every account so `Account<T>` rejects the wrong type. Native/Pinocchio accounts often have **no discriminator**, so two account types with the same size are interchangeable.

```rust
// VULNERABLE — Vault and Escrow are both 80 bytes, no tag distinguishes them
let vault = Vault::try_from_slice(&account.data.borrow())?;
// attacker passes an Escrow account where a Vault is expected
```

```rust
// SAFE — first byte (or enum tag) identifies the account type; check it
#[repr(u8)]
enum AccountTag { Uninitialized = 0, Vault = 1, Escrow = 2 }

let data = account.data.borrow();
if data[0] != AccountTag::Vault as u8 {
    return Err(ProgramError::InvalidAccountData);
}
```

**Detection**: do account structs carry a type tag? Is it checked on every read? Same-size structs without a checked tag = High.

---

## N4. Manual Deserialization Panics (slice indexing / length)

**Severity**: Medium–High (DoS, sometimes worse)

Native parsing indexes into raw byte slices. An attacker-sized account triggers an out-of-bounds panic, aborting the transaction — and in loops or crank instructions can brick protocol state.

```rust
// VULNERABLE — panics if data shorter than 8 bytes
let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());

// VULNERABLE — assumes a fixed account count
let user = &accounts[3]; // panics if fewer accounts passed
```

```rust
// SAFE — bounds-check, no unwrap on attacker input
let amount_bytes: [u8; 8] = data.get(0..8)
    .ok_or(ProgramError::InvalidInstructionData)?
    .try_into()
    .map_err(|_| ProgramError::InvalidInstructionData)?;
let amount = u64::from_le_bytes(amount_bytes);

// SAFE — validate account count up front
let [config, vault, user, authority, ..] = accounts else {
    return Err(ProgramError::NotEnoughAccountKeys);
};
```

**Detection**: grep for `unwrap()`, `[n..m]` slice ranges, and `accounts[n]` direct indexing in instruction parsing.

---

## N5. `TryFrom` Validation Gaps (Pinocchio idiom)

**Severity**: varies by what's missing

Pinocchio programs commonly centralize validation in a `TryFrom<&[AccountInfo]>` impl that builds a typed context struct. This is good practice — **but it only protects you for the checks it actually contains**. Audit the `TryFrom` as the security boundary.

```rust
impl<'a> TryFrom<&'a [AccountInfo]> for Withdraw<'a> {
    type Error = ProgramError;
    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, vault, dest, ..] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // CHECK each invariant — every missing line below is a potential finding:
        if !authority.is_signer() { return Err(ProgramError::MissingRequiredSignature); }
        if vault.owner() != &crate::ID { return Err(ProgramError::IllegalOwner); }
        // PDA derivation check?  Mint check?  dest ownership check?  <-- verify presence

        Ok(Self { authority, vault, dest })
    }
}
```

**Audit method**: build the expected-checks matrix (signer, owner, PDA, mint, balance, tag) for each instruction, then tick off which the `TryFrom` performs. The gaps are your findings.

---

## N6. Unverified PDA Derivation (native)

**Severity**: High

No `seeds`/`bump` macro means PDAs must be derived and compared by hand. A missing comparison lets an attacker pass an arbitrary account where a specific PDA is required.

```rust
// VULNERABLE — uses the passed account without confirming it's the expected PDA
let vault = &accounts[1];

// SAFE — derive and compare
let (expected, bump) = Pubkey::find_program_address(
    &[b"vault", user.key.as_ref()], program_id);
if vault.key != &expected {
    return Err(ProgramError::InvalidSeeds);
}
```

For `invoke_signed`, confirm the seeds used to sign exactly match the derivation (see vulnerability-patterns.md #14).

---

## N7. Manual CPI Without Program-ID Validation

**Severity**: Critical

Native CPIs build the `Instruction` and call `invoke` manually. If the target program account comes from the caller and isn't pinned, it's arbitrary CPI (vulnerability-patterns.md #3).

```rust
// SAFE — pin the program id before invoking
if token_program.key != &spl_token::ID {
    return Err(ProgramError::IncorrectProgramId);
}
invoke(&ix, accounts)?;
```

Pinocchio exposes typed CPI helpers (e.g. `pinocchio_token`, `pinocchio_system`) — prefer them; if the program builds raw instructions, scrutinize each target key.

---

## N8. Rent & Account-Init Mistakes (native)

**Severity**: Medium–High

Without `init`, the program must create accounts with `create_account`, size them, and fund them to rent exemption itself.

```rust
// SAFE init path
let rent = Rent::get()?;
let space = State::LEN;
let lamports = rent.minimum_balance(space);
invoke_signed(
    &system_instruction::create_account(payer.key, new.key, lamports, space as u64, program_id),
    &[payer.clone(), new.clone(), system_program.clone()],
    &[&[b"state", user.key.as_ref(), &[bump]]],
)?;
```

Audit for: under-funding (account purgeable), wrong `space` (truncated/over-read state), and **re-init**: native programs must explicitly reject an already-initialized account (check the type tag/`is_initialized` is `Uninitialized` before writing).

---

## N9. `no_std` Panic & Arithmetic Behavior

**Severity**: contextual

`#![no_std]` Pinocchio programs use a custom panic handler. Confirm:

- Arithmetic still uses `checked_*` — `overflow-checks` must be set in the release profile; a panic from overflow in `no_std` still aborts the tx (DoS) and can be triggered by user input.
- No `std`-only assumptions (allocator behavior, formatting) leak in.
- The panic handler doesn't silently swallow errors that should propagate as `ProgramError`.

---

## Native / Pinocchio Quick Checklist

```
[ ] Every account read is preceded by an owner check
[ ] Every authority is checked for is_signer
[ ] Account type tag/discriminator present AND checked (type confusion)
[ ] No unwrap()/panic on attacker-controlled length or account count
[ ] TryFrom (if used) validates signer+owner+PDA+mint for every instruction
[ ] PDAs derived and compared (not trusted from input)
[ ] invoke targets validate program id
[ ] Account init: correct space, rent-exempt funding, re-init guard
[ ] checked arithmetic; overflow-checks enabled in release profile
```

---

## References

- [Pinocchio (anza-xyz)](https://github.com/anza-xyz/pinocchio)
- [How to Build Solana Programs with Pinocchio (Helius)](https://www.helius.dev/blog/pinocchio)
- [Pinocchio docs.rs](https://docs.rs/pinocchio/latest/pinocchio/)
- Trail of Bits "not-so-smart-contracts" Solana — native program pitfalls
