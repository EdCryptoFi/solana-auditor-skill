# Solana Auditor Skill

**Full-lifecycle security audit skill for Solana programs.**

This skill turns Claude Code into a senior Solana security auditor. It conducts the complete audit engagement — from threat modeling and tool setup through manual vulnerability review, exploit development, formal verification, and professional report generation.

> Part of the [Solana AI Kit](https://github.com/solanabr/solana-ai-kit) by Superteam Brasil.

---

## What It Does

| Phase | What happens |
|-------|-------------|
| **0 — Scoping** | Codebase survey, architecture map, STRIDE threat model |
| **1 — Automated** | cargo-audit, clippy security lints, Trident fuzzing, semgrep |
| **2 — Manual review** | 25-pattern systematic checklist per instruction |
| **3 — Deep analysis** | Exploit PoC development, economic attack modeling |
| **4 — Formal verification** | Kani proofs, Trident invariants, proptest |
| **5 — Reporting** | CVSS-scored findings, publication-quality report + SARIF/JSON |
| **6 — Remediation** | Fix verification, PoC regression, re-scoring |

### Beyond a source review

Real audits aren't always a clean source tree. This skill also covers:

- **Deployed programs** — audit by Program ID: confirm the on-chain bytecode matches the source (`solana-verify`), classify the upgrade authority, and quantify blast radius before reviewing a line. ([onchain-analysis.md](skill/onchain-analysis.md))
- **Native & Pinocchio** — non-Anchor programs where owner/signer/discriminator checks are all manual. ([native-pinocchio-patterns.md](skill/native-pinocchio-patterns.md))
- **Upgrade / differential audits** — review only the diff between v1 and v2, with invariant-regression analysis and state-migration safety. ([diff-audit.md](skill/diff-audit.md))
- **Economic exploits** — flash-loan/oracle manipulation, first-depositor inflation, rounding leaks, liquidation abuse. ([defi-economic-exploits.md](skill/defi-economic-exploits.md))

## How It Differs from Other Security Skills

| Skill | What it does |
|-------|-------------|
| **solana-auditor (this)** | Conduct the full audit |
| `trailofbits/solana-vulnerability-scanner` | Automated 6-pattern quick scanner |
| `frankcastleauditor/safe-solana-builder` | Write safe programs at build time |
| `qedgen` | Spec-driven formal verification (.qedspec) |
| `trailofbits/audit-prep-assistant` | Prepare code for an external auditor |

## Installation

```bash
# Clone and install
git clone https://github.com/solanabr/solana-auditor-skill
cd solana-auditor-skill
./install.sh
```

The installer copies the skill to `~/.claude/skills/solana-auditor/` and updates your `~/.claude/CLAUDE.md`.

## Usage

### Slash Commands (run in your Solana project)

```
/audit-init    — initialize workspace, survey codebase
/audit-scan    — run all automated analysis tools
/audit-onchain — inspect a deployed Program ID (authority, verifiable build, blast radius)
/audit-report  — generate the final audit report (markdown + SARIF + JSON)
```

### Natural Language

```
"Audit this Solana program"
"Review the withdraw instruction for vulnerabilities"
"Write a PoC for the missing signer finding"
"Set up Trident fuzzing for this Anchor program"
"Generate a professional audit report"
"Calculate CVSS score for this finding"
```

### Agents

```
Spawn lead-auditor   → scope, threat model, audit plan (opus)
Spawn vuln-researcher → deep instruction review + PoC (sonnet)
Spawn report-writer  → professional report generation (sonnet)
```

## Skill Files

```
solana-auditor-skill/
├── CLAUDE.md                       # Skill entry point
├── README.md                       # This file
├── install.sh                      # Installer
├── skill/
│   ├── SKILL.md                    # Routing hub (load first)
│   ├── audit-lifecycle.md          # 6-phase workflow
│   ├── vulnerability-patterns.md   # 25-pattern checklist
│   ├── onchain-analysis.md         # Deployed program + verifiable builds
│   ├── native-pinocchio-patterns.md# Non-Anchor / Pinocchio vuln patterns
│   ├── diff-audit.md               # Differential / upgrade audits
│   ├── defi-economic-exploits.md   # Flash-loan / oracle / rounding modeling
│   ├── formal-verification.md      # Kani, Trident, proptest
│   ├── report-generation.md        # Report templates + CVSS + SARIF/JSON
│   ├── tools-setup.md              # Tool installation + CI
│   ├── ai-agent-vulnerabilities.md # Agent wallets, session keys, prompt injection
│   ├── evm-to-solana-mapping.md    # For auditors coming from Ethereum
│   └── resources.md                # Prior audits, references
├── agents/
│   ├── lead-auditor.md             # Opus: orchestrates audit
│   ├── competition-auditor.md      # Opus: speed-optimized contest audits
│   ├── vuln-researcher.md          # Sonnet: deep vuln analysis + PoC
│   └── report-writer.md            # Sonnet: professional reports
├── commands/
│   ├── audit-init.md               # /audit-init
│   ├── audit-scan.md               # /audit-scan
│   ├── audit-onchain.md            # /audit-onchain
│   └── audit-report.md             # /audit-report
└── rules/
    └── audit-discipline.md         # 10 non-negotiable audit rules
```

## Vulnerability Coverage

The 25-pattern checklist covers:

**Account Validation (8)**
- Missing signer check
- Missing owner check  
- Arbitrary CPI (unchecked program)
- PDA bump canonicalization
- Account reuse
- Type confusion
- Reinitialize attack
- Insecure account closing

**Arithmetic (3)**
- Integer overflow/underflow
- Precision loss / integer division
- Type casting truncation

**CPI Safety (3)**
- Reentrancy via CPI
- CPI return value ignored
- Signer seed mismatch

**Token Operations (3)**
- Token account ownership
- Mint mismatch
- SPL Token 2022 extension bypass

**Oracle / Price Feeds (2)**
- Stale oracle price
- Oracle manipulation via flash loan

**Program Upgrade & Governance (2)**
- Unconstrained upgrade authority
- Time-lock bypass

**Solana-Specific (4)**
- Rent exemption not enforced
- Clock manipulation
- Account data length mismatch
- Compute unit exhaustion

## Requirements

- Rust + cargo
- Solana CLI 2.x
- Anchor CLI (for Anchor programs)
- `cargo-audit` (`cargo install cargo-audit`)
- Trident (`cargo install trident-cli`) — for fuzzing
- Kani (`cargo install kani-verifier`) — for formal verification
- semgrep (`pip3 install semgrep`) — for pattern matching

## License

MIT — see [LICENSE](LICENSE)

---

*Built for the [Solana AI Kit Bounty](https://earn.superteam.fun) by Superteam Brasil.*
