---
description: "Initialize a Solana audit workspace for a new engagement. Creates the folder structure, findings tracker, tool version log, and architecture map template. Run once at the start of every audit."
---

You are initializing an audit workspace for a Solana program security engagement.

## Step 1: Create Workspace Structure

```bash
# Create audit workspace
mkdir -p audit-workspace/{raw-findings,pocs,reports,tool-output}

# Create gitignore to avoid committing sensitive audit work
cat > audit-workspace/.gitignore << 'EOF'
# Audit workspace - do not commit findings to protocol repo
raw-findings/
pocs/
reports/
EOF

echo "Workspace created at audit-workspace/"
```

## Step 2: Record Environment and Tool Versions

```bash
cat > audit-workspace/environment.md << EOF
# Audit Environment

Generated: $(date -u +"%Y-%m-%d %H:%M UTC")

## Tool Versions
$(rustc --version 2>/dev/null || echo "rustc: not found")
$(cargo --version 2>/dev/null || echo "cargo: not found")
$(solana --version 2>/dev/null || echo "solana-cli: not found")
$(anchor --version 2>/dev/null || echo "anchor-cli: not found")
$(cargo audit --version 2>/dev/null || echo "cargo-audit: not installed — run: cargo install cargo-audit")
$(trident --version 2>/dev/null || echo "trident: not installed — see tools-setup.md")
$(semgrep --version 2>/dev/null || echo "semgrep: not installed — run: pip3 install semgrep")
$(kani --version 2>/dev/null || echo "kani: not installed — run: cargo install kani-verifier && cargo kani setup")

## Repository
$(git remote get-url origin 2>/dev/null || echo "No git remote")
Commit: $(git rev-parse HEAD 2>/dev/null || echo "Not a git repo")
Branch: $(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "Unknown")
EOF

echo "Environment recorded."
cat audit-workspace/environment.md
```

## Step 3: Codebase Survey

```bash
echo "=== Codebase Survey ===" | tee audit-workspace/survey.md

echo "" >> audit-workspace/survey.md
echo "## Rust files" >> audit-workspace/survey.md
find . -name "*.rs" -not -path "*/target/*" | sort | tee -a audit-workspace/survey.md

echo "" >> audit-workspace/survey.md
echo "## Line counts (top 20)" >> audit-workspace/survey.md
find . -name "*.rs" -not -path "*/target/*" | xargs wc -l 2>/dev/null | sort -rn | head -20 | tee -a audit-workspace/survey.md

echo "" >> audit-workspace/survey.md
echo "## Public instructions" >> audit-workspace/survey.md
grep -rn "pub fn" --include="*.rs" . | grep -v "target/\|/test\|//\|#\[" | head -40 | tee -a audit-workspace/survey.md

echo "" >> audit-workspace/survey.md
echo "## Upgradeable program check" >> audit-workspace/survey.md
grep -rn "BpfLoader\|UpgradeableLoader\|set_upgrade_authority" --include="*.rs" . | tee -a audit-workspace/survey.md || echo "None found" >> audit-workspace/survey.md

echo "" >> audit-workspace/survey.md
echo "## External programs called (CPI)" >> audit-workspace/survey.md
grep -rn "invoke\|invoke_signed\|CpiContext\|Program<" --include="*.rs" . | grep -v "target/\|//\|#\[" | tee -a audit-workspace/survey.md || echo "None found" >> audit-workspace/survey.md

echo "Survey complete. See audit-workspace/survey.md"
```

## Step 4: Create Architecture Map

```bash
cat > audit-workspace/architecture.md << 'TEMPLATE'
# Architecture Map

## Program Information

| Field | Value |
|-------|-------|
| Program ID | FILL: `solana program show <ID>` |
| Network | mainnet / devnet |
| Upgradeable | Yes / No |
| Upgrade authority | Single key / Multisig / Immutable |
| Anchor version | Check Cargo.toml |
| Rust edition | Check Cargo.toml |

## Instructions

| Instruction | Purpose | Signers required | Assets touched |
|-------------|---------|-----------------|----------------|
| [name] | [purpose] | [who must sign] | [SOL/tokens/PDAs] |

## Account Types (PDAs)

| Account | Seeds | Owner | Mutable by |
|---------|-------|-------|-----------|
| [name] | [seeds] | [program] | [instructions] |

## CPI Calls

| Instruction | Calls | Purpose | Program ID validated? |
|-------------|-------|---------|----------------------|
| [ix name] | [target program] | [why] | Yes / No |

## Privileged Roles

| Role | Key type | Capabilities |
|------|----------|-------------|
| [role] | Single key / Multisig | [what they can do] |

## Assets at Risk

| Asset | Location | Max value at risk |
|-------|----------|-------------------|
| SOL | [vault PDA] | [amount or "unbounded"] |
| [Token] | [token account] | [amount] |

## Threat Model Notes

(Fill during Phase 0 scoping — see audit-lifecycle.md)
TEMPLATE

echo "Architecture map template created at audit-workspace/architecture.md"
echo "Fill this in before starting Phase 1."
```

## Step 5: Create Findings Tracker

```bash
cat > audit-workspace/findings-tracker.md << 'TRACKER'
# Findings Tracker

## Summary

| Severity | Count | Fixed | Open |
|----------|-------|-------|------|
| Critical | 0 | 0 | 0 |
| High | 0 | 0 | 0 |
| Medium | 0 | 0 | 0 |
| Low | 0 | 0 | 0 |
| Informational | 0 | 0 | 0 |

## Findings

| ID | Title | Severity | Location | Phase | Status | PoC |
|----|-------|----------|----------|-------|--------|-----|
| FINDING-001 | | | | | Open | |

## Phase 1: Tool Findings (automated)

(Run `/audit-scan` to populate this section)

## Phase 2: Manual Review Findings

(Add findings here as you review each instruction)

## Phase 3: Deep Analysis Findings

(PoC-confirmed findings from exploit development)
TRACKER

echo "Findings tracker created at audit-workspace/findings-tracker.md"
```

## Step 6: Summary

```bash
echo ""
echo "╔═══════════════════════════════════════════╗"
echo "║  Audit Workspace Initialized               ║"
echo "╚═══════════════════════════════════════════╝"
echo ""
echo "Created:"
ls -la audit-workspace/
echo ""
echo "Next steps:"
echo "  1. Fill in audit-workspace/architecture.md"
echo "  2. Run /audit-scan to execute automated tools"  
echo "  3. Review audit-workspace/survey.md to plan manual review order"
echo "  4. Load skill/audit-lifecycle.md for the full workflow"
```
