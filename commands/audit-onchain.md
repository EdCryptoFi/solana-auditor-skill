---
description: "Inspect a deployed Solana program by Program ID before reviewing its source. Records the upgrade authority, attempts a verifiable-build check (source ↔ on-chain bytecode), enumerates mint/freeze authorities the program controls, and quantifies blast radius. Run this FIRST for any audit of a live/deployed program."
---

You are performing on-chain reconnaissance on a deployed program. The goal is to answer the three on-chain trust questions before any source review: (1) does the deployed bytecode match the public source, (2) who can change the program and how fast, (3) what is at risk right now. Load [skill/onchain-analysis.md](../skill/onchain-analysis.md) for full guidance.

Ask the user for the **Program ID** and the **source repo URL** (if available) if not already provided. Default network to `mainnet-beta` unless told otherwise.

## Step 1: Program metadata

```bash
PROGRAM_ID="<PROGRAM_ID>"
NETWORK="mainnet-beta"

mkdir -p audit-workspace/onchain
echo "=== solana program show ===" | tee audit-workspace/onchain/program-show.txt
solana program show "$PROGRAM_ID" --url "$NETWORK" 2>&1 | tee -a audit-workspace/onchain/program-show.txt

# Extract and highlight the upgrade authority (the critical field)
echo ""
grep -i "Authority" audit-workspace/onchain/program-show.txt && \
  echo "⚠️  Classify this authority: EOA / Squads multisig / SPL Governance / none(immutable)"
```

## Step 2: Dump + hash the on-chain bytecode

```bash
solana program dump "$PROGRAM_ID" audit-workspace/onchain/onchain.so --url "$NETWORK"
echo "On-chain bytecode sha256:"
sha256sum audit-workspace/onchain/onchain.so | tee audit-workspace/onchain/onchain.sha256
```

## Step 3: Verifiable build check (source ↔ bytecode)

```bash
if command -v solana-verify &> /dev/null; then
    echo "=== on-chain program hash ===" | tee audit-workspace/onchain/verify.txt
    solana-verify get-program-hash -u "$NETWORK" "$PROGRAM_ID" 2>&1 | tee -a audit-workspace/onchain/verify.txt

    # If a repo URL is known, verify against it (uncomment / fill in):
    # solana-verify verify-from-repo -u "https://api.$NETWORK.solana.com" \
    #   --program-id "$PROGRAM_ID" "<REPO_URL>"  2>&1 | tee -a audit-workspace/onchain/verify.txt
else
    echo "⚠️  solana-verify not installed. Run: cargo install solana-verify"
fi
```

Interpret and record:
- **Hash matches** → note as positive Informational ("bytecode matches source at commit X").
- **Mismatch** → **Critical (process) finding** — the source review cannot speak to live behavior.
- **Not reproducibly buildable** → High (process) finding.

## Step 4: IDL recovery

```bash
anchor idl fetch "$PROGRAM_ID" --provider.cluster "$NETWORK" \
  > audit-workspace/onchain/idl.json 2>/dev/null \
  && echo "✅ IDL fetched" \
  || echo "⚠️  No on-chain IDL — closed-source or non-Anchor. Scope as black-box if no source."
```

## Step 5: Assets the program controls + blast radius

For each token mint the protocol governs (ask the user or derive from the IDL/state), check who holds mint/freeze authority:

```bash
# spl-token display <MINT> --url mainnet-beta
# Flag if Mint Authority or Freeze Authority is a program PDA, an EOA, or a multisig.
```

Summarize for the report:
- Upgrade authority type + multisig threshold + timelock (if any).
- Mint/freeze authorities controlled by the program.
- Approximate TVL in protocol-controlled accounts and number of dependent programs.

## Step 6: Record findings

Append on-chain findings (build mismatch, single-key authority, no timelock, program-controlled mint/freeze) to `audit-workspace/findings-tracker.md`. Then proceed to source review (`/audit-init` → `/audit-scan`) — or, if no source exists, a forked-validator black-box review per onchain-analysis.md.
