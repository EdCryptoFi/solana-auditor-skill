# Solana Security Auditor

Full-lifecycle Solana security auditor skill. This is not a vulnerability scanner or pre-audit prep tool — it conducts the complete audit engagement.

> Load [skill/SKILL.md](skill/SKILL.md) for the routing table. Load individual skill files on demand.

## What This Skill Does

Six-phase audit lifecycle:
1. **Scoping** — codebase survey, architecture map, threat model (STRIDE)
2. **Automated analysis** — cargo-audit, clippy security lints, Trident fuzzing, semgrep
3. **Manual review** — systematic line-by-line review against 25-pattern checklist
4. **Deep analysis** — exploit PoC development, economic attack modeling
5. **Formal verification** — Kani proofs, Trident invariants, proptest
6. **Reporting** — CVSS-scored findings, professional report, remediation verification

## Quick Start

```
/audit-init    → scaffold workspace + codebase survey
/audit-scan    → run all automated tools
/audit-report  → generate professional report
```

Or ask naturally:
- "Audit this Solana program"
- "Review this instruction for vulnerabilities"
- "Write a PoC for this finding"
- "Generate the audit report"
- "Set up Trident for fuzzing"

## Slash Commands

| Command | When to use |
|---------|-------------|
| `/audit-doctor` | Before anything — verify the audit toolchain is installed |
| `/audit-init` | Start of every audit engagement |
| `/audit-scan` | Phase 1 — automated analysis |
| `/audit-onchain` | Deployed program — verify bytecode, authority, blast radius |
| `/audit-report` | Phase 5 — generate final report (markdown + SARIF + JSON + HTML) |

## Specialized Skill Files

Beyond the source-review core, load on demand:
- **Deployed programs / verifiable builds** → `skill/onchain-analysis.md`
- **Native / Pinocchio (non-Anchor)** → `skill/native-pinocchio-patterns.md`
- **Upgrade / differential audits** → `skill/diff-audit.md`
- **DeFi economic exploits** → `skill/defi-economic-exploits.md`

## Agents

| Agent | Model | Use for |
|-------|-------|---------|
| `lead-auditor` | opus | Scoping, threat model, audit plan, finding triage |
| `competition-auditor` | opus | Speed-optimized contest audits (Code4rena / Cantina) |
| `vuln-researcher` | sonnet | Deep per-instruction review, PoC development |
| `economic-auditor` | opus | DeFi value-extraction: flash loan, oracle, rounding, liquidation |
| `diff-auditor` | opus | Upgrade / differential audits (review the diff) |
| `report-writer` | sonnet | Professional report generation |

## How This Differs from Other Security Skills in the Kit

| Skill | What it does |
|-------|-------------|
| **solana-auditor (this)** | Conduct the full audit |
| `trailofbits/solana-vulnerability-scanner` | Automated 6-pattern scanner |
| `frankcastleauditor/safe-solana-builder` | Write safe programs from scratch |
| `qedgen` | Spec-driven formal verification |
| `trailofbits/audit-prep-assistant` | Prepare your code for external auditors |

## Rules

[rules/audit-discipline.md](rules/audit-discipline.md) — 10 non-negotiable audit discipline rules.

Key rules:
- No Critical/High without a working PoC
- Never mark "Fixed" without running the PoC regression
- Tool findings are raw material — manually verify everything

## Default Vulnerability Checklist

25 patterns in [skill/vulnerability-patterns.md](skill/vulnerability-patterns.md):
- Account validation (8 patterns): missing signer, owner, PDA, type confusion, reuse
- Arithmetic (3 patterns): overflow, precision, truncation
- CPI safety (3 patterns): reentrancy, return value, signer seeds
- Token operations (3 patterns): ownership, mint, SPL Token 2022
- Oracle (2 patterns): staleness, flash loan manipulation
- Governance (2 patterns): upgrade authority, time-lock bypass
- Solana-specific (4 patterns): rent, clock, data length, compute units
