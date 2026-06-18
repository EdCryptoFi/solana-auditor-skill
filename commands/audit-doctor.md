---
description: "Preflight check for the Solana audit toolchain. Verifies that Rust, Solana CLI, Anchor, and the audit tools (cargo-audit, clippy, trident, kani, semgrep, solana-verify) are installed and reports the exact install command for anything missing. Run once before /audit-scan so the automated phase doesn't silently skip tools."
---

You are running the audit toolchain preflight. Detect what is installed, print versions, and give the exact install command for anything missing. Do not install anything automatically — show the user the commands and let them choose. Nothing here is fatal; `/audit-scan` degrades gracefully, but missing tools mean reduced coverage, so surface them clearly.

Run this check and present the results as a table:

```bash
echo "=== Solana Auditor — toolchain doctor ==="
printf "%-16s %-10s %s\n" "TOOL" "STATUS" "DETAIL / FIX"

check () {
  # $1 = label, $2 = command to test, $3 = version cmd, $4 = install hint, $5 = role
  if command -v "$2" >/dev/null 2>&1; then
    ver=$($3 2>/dev/null | head -1)
    printf "%-16s \033[0;32m%-10s\033[0m %s\n" "$1" "OK" "$ver"
  else
    printf "%-16s \033[0;31m%-10s\033[0m %s\n" "$1" "MISSING" "$5 — install: $4"
  fi
}

# Core toolchain (required)
check "rustc"        rustc        "rustc --version"        "https://rustup.rs"                                  "REQUIRED: build programs"
check "cargo"        cargo        "cargo --version"        "https://rustup.rs"                                  "REQUIRED: build/test"
check "solana"       solana       "solana --version"       'sh -c \"\$(curl -sSfL https://release.anza.xyz/stable/install)\"'  "REQUIRED: CLI / on-chain"
check "anchor"       anchor       "anchor --version"       "avm install latest && avm use latest"               "for Anchor programs"

# Audit tools
check "cargo-audit"  cargo-audit  "cargo audit --version"  "cargo install cargo-audit --locked"                 "Phase 1: known CVEs"
check "trident"      trident      "trident --version"      "cargo install trident-cli --locked"                 "Phase 1/4: fuzzing"
check "kani"         "cargo-kani" "cargo kani --version"   "cargo install --locked kani-verifier && cargo kani setup"  "Phase 4: formal verification"
check "semgrep"      semgrep      "semgrep --version"      "pip3 install semgrep"                               "Phase 1: pattern matching"
check "solana-verify" solana-verify "solana-verify --version" "cargo install solana-verify"                    "verifiable builds / on-chain"

# clippy is a rustup component, not a standalone binary
if cargo clippy --version >/dev/null 2>&1; then
  printf "%-16s \033[0;32m%-10s\033[0m %s\n" "clippy" "OK" "$(cargo clippy --version 2>/dev/null)"
else
  printf "%-16s \033[0;31m%-10s\033[0m %s\n" "clippy" "MISSING" "Phase 1: lints — install: rustup component add clippy"
fi
```

After running:

1. **Summarize** which tools are present and which are missing, grouped by REQUIRED vs. optional.
2. If anything REQUIRED is missing, tell the user the audit can't proceed meaningfully until it's installed.
3. If only optional audit tools are missing, note exactly which audit phases will be skipped (e.g. "no trident → no fuzzing in Phase 1/4; no kani → no formal verification in Phase 4").
4. Offer to run the install commands for the missing tools if the user wants (ask first — never install silently).

Then point the user to `/audit-init` (new audit) or `/audit-onchain` (deployed program).
