# On-Chain & Deployed-Program Analysis

Audit a program that is **already deployed** — when you have a Program ID but not necessarily clean source, or when you must confirm that the published source is the code actually running on-chain.

This is the gap most audits skip: a perfect source-code review is worthless if the deployed bytecode doesn't match the source. Start every audit of a live protocol here.

> Load this file when the user says: "audit this program ID", "is this program verified", "does the deployed code match the repo", "check the upgrade authority", "this contract on mainnet", "verifiable build", "solana-verify".

---

## The Three On-Chain Trust Questions

Before reviewing a single line of source, answer these for any live program:

1. **Does the on-chain bytecode match the public source?** (verifiable build)
2. **Who can change the program, and how fast?** (upgrade authority + timelock)
3. **What is the program's blast radius right now?** (TVL, dependent programs, mint/freeze authorities it controls)

A protocol that fails #1 means your source review proves nothing. A protocol that fails #2 means today's safe code can become malicious in one transaction.

---

## Step 1 — Inspect the Deployed Program

```bash
# Basic metadata: owner loader, upgrade authority, last deployed slot, data length
solana program show <PROGRAM_ID> --url mainnet-beta

# Key fields to extract:
#   Authority:        <- the upgrade authority (THE critical field)
#   Last Deployed In Slot
#   Data Length:      <- size of the bytecode
#   Balance:          <- rent for the program data account
```

Interpretation:

| Field | What it tells you |
|-------|-------------------|
| `Authority: <pubkey>` | Program is **upgradeable** — single point of control. Check if it's a multisig. |
| `Authority: none` | Program is **immutable** — bytecode is frozen. Source review is authoritative. |
| Loader `BPFLoaderUpgradeab1e...` | Standard upgradeable loader |
| Recent `Last Deployed In Slot` | Was just upgraded — re-verify; a prior audit may be stale |

```bash
# Dump the on-chain bytecode to a file for hashing / disassembly
solana program dump <PROGRAM_ID> onchain.so --url mainnet-beta
sha256sum onchain.so
```

---

## Step 2 — Verify the Build (source ↔ bytecode)

Use `solana-verify` (the Solana Foundation / Ellipsis Labs CLI). This deterministically rebuilds the source in a pinned Docker image and compares the resulting hash to the on-chain program.

```bash
cargo install solana-verify

# Get the on-chain program hash
solana-verify get-program-hash -u mainnet-beta <PROGRAM_ID>

# Get the hash of a locally built artifact
solana-verify get-executable-hash target/deploy/my_program.so

# Verify a local checkout against the deployed program
solana-verify verify-from-repo \
  -u https://api.mainnet-beta.solana.com \
  --program-id <PROGRAM_ID> \
  https://github.com/<ORG>/<REPO>

# Program lives in a monorepo subfolder?
solana-verify verify-from-repo ... --mount-path programs/my_program
```

### Remote / public verification (OtterSec registry)

```bash
# Submit a verification job to the OtterSec API; result is public at verify.osec.io
solana-verify verify-from-repo --remote \
  -um \
  --program-id <PROGRAM_ID> \
  https://github.com/<ORG>/<REPO>
```

Check status at `https://verify.osec.io/status/<PROGRAM_ID>`.

### How to report the result

| Result | Severity | Report language |
|--------|----------|-----------------|
| Hash matches, verified | Informational (positive) | "Deployed bytecode matches source at commit `<hash>`." |
| Hash mismatch | **Critical (process)** | "Deployed program does NOT match the audited source. The audit cannot make claims about live behavior until resolved." |
| No verifiable build possible | High (process) | "Program is not reproducibly buildable; users cannot independently verify it matches source." |

> **Audit rule**: Never sign off on a "secure" verdict for a live protocol whose deployed bytecode you could not match to the reviewed source. State the mismatch as a finding.

---

## Step 3 — Upgrade Authority & Governance Risk

```bash
# Confirm the upgrade authority
solana program show <PROGRAM_ID> --url mainnet-beta | grep "Authority"
```

Then classify the authority:

| Authority type | How to detect | Risk |
|----------------|---------------|------|
| Single EOA wallet | Plain pubkey, owned by System Program | **High** — one key compromise = full takeover |
| Squads multisig | Account owned by Squads program (`SMPLecH...` / v4 program ID) | Lower — check threshold (e.g. 3/5) |
| SPL Governance | Account owned by `GovER5Lthms3bLBqWub97yVrMmEogzX7xNjdXpPPCVZw` | Lower — check realm config & voting delay |
| `none` (immutable) | No authority field | None — but no patching either |

Investigate further:

- **Multisig threshold**: a "3/5 multisig" with 5 keys held by one person is theater. Note signer overlap if discoverable.
- **Timelock**: is there a delay between proposing and executing an upgrade? No delay = users can't exit before a malicious upgrade lands.
- **Authority over assets**: does the program PDA hold `mint_authority` / `freeze_authority` of a token? Check with `spl-token display <MINT>`. A program that can mint or freeze user tokens is a systemic risk even if "the code looks fine".

```bash
# Inspect a mint controlled by the protocol
spl-token display <MINT_ADDRESS> --url mainnet-beta
# Look at: Mint Authority, Freeze Authority — are they a program PDA? A multisig? An EOA?
```

---

## Step 4 — IDL & Interface Recovery

```bash
# Fetch the on-chain Anchor IDL (if the program published one)
anchor idl fetch <PROGRAM_ID> --provider.cluster mainnet > idl.json

# No IDL on-chain? Try to find it in the repo, or reconstruct the interface
# from transaction history (instruction discriminators) via an RPC explorer.
```

If no source and no IDL exist, the program is **closed-source**: scope the audit honestly as a black-box review (behavioral testing + bytecode disassembly), and say so in Limitations.

### Bytecode disassembly (closed-source last resort)

```bash
# sBPF disassembly — read control flow, find syscalls, spot missing checks
llvm-objdump -d onchain.so | head -200
# Or use the Solana sBPF tooling / a sealevel disassembler for symbol recovery.
```

Black-box audits can still surface: missing signer enforcement (replay an instruction with a non-signer), missing owner checks (substitute a foreign account), and arithmetic edge cases (fuzz instruction data on a forked validator).

---

## Step 5 — Live State & Blast Radius

Snapshot what's actually at risk, today, on a forked validator so you can run PoCs against real state:

```bash
# Clone the deployed program + key accounts into a local validator
solana-test-validator \
  --clone <PROGRAM_ID> \
  --clone <CONFIG_PDA> \
  --clone <VAULT_TOKEN_ACCOUNT> \
  --url mainnet-beta \
  --reset

# Or use Surfpool / LiteSVM with mainnet account injection for faster PoC iteration.
```

Quantify exposure for the report:

- **TVL in protocol-controlled accounts** (sum vault/treasury token balances).
- **Number of dependent programs** that CPI into this one (a bug here cascades).
- **Open positions / user accounts** that a state-corruption bug would affect.

This turns "this is exploitable" into "this is exploitable and ~$X is at risk right now" — the difference between a Medium and a Critical in practice.

---

## On-Chain Audit Checklist

```
[ ] solana program show — recorded authority, loader, last-deployed slot
[ ] Bytecode dumped and hashed
[ ] solana-verify: on-chain hash vs source hash (match / mismatch / not reproducible)
[ ] Upgrade authority classified (EOA / multisig / governance / immutable)
[ ] Multisig threshold and timelock checked
[ ] Mint/freeze authorities the program controls enumerated
[ ] IDL fetched or interface reconstructed
[ ] Forked-validator environment built for live PoCs
[ ] Blast radius quantified (TVL, dependents, user accounts)
```

---

## References

- [Solana Verified Builds guide](https://solana.com/developers/guides/advanced/verified-builds)
- [solana-verifiable-build (Ellipsis Labs)](https://github.com/Ellipsis-Labs/solana-verifiable-build)
- [solana-verifiable-build (Solana Foundation fork)](https://github.com/solana-foundation/solana-verifiable-build)
- [OtterSec verification registry](https://verify.osec.io)
- Squads Multisig v4 — upgrade authority custody
