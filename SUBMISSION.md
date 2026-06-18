# Solana AI Kit Bounty — Submission

**GitHub repo:** https://github.com/EdCryptoFi/solana-auditor-skill

## What it does / Problem it solves

The Solana AI Kit already has skills for *preparing* for an audit (Trail of Bits' prep assistant), *scanning* for a handful of patterns automatically (ToB's vulnerability scanner), and *writing safe code* from scratch (frank castle's safe-solana-builder). What was missing: a skill that **conducts the actual audit, end to end** — including the parts real engagements hit that a clean source review ignores.

`solana-auditor-skill` is a full-lifecycle security audit skill for Claude Code. It handles the complete engagement across four audit modes (Firm, Competition, Self-audit, Differential/Upgrade):

0. **On-chain recon** — for deployed programs: verify the on-chain bytecode matches the source (`solana-verify`), classify the upgrade authority, quantify blast radius — *before* trusting any source review
1. **Scoping** — codebase survey, architecture map, STRIDE threat model
2. **Automated analysis** — cargo-audit, clippy security lints, Trident fuzzing, semgrep
3. **Manual review** — 25-pattern systematic checklist per instruction (Anchor *and* native/Pinocchio)
4. **Deep analysis** — exploit PoC development, DeFi economic attack modeling
5. **Formal verification** — Kani proofs, Trident invariants, proptest
6. **Reporting** — CVSS 3.1 scored findings, **Markdown + interactive HTML + SARIF + JSON** output
7. **Remediation** — fix verification, PoC regression, re-scoring

## Installation

```bash
git clone https://github.com/EdCryptoFi/solana-auditor-skill
cd solana-auditor-skill
./install.sh
```

Then in any Solana project:

```
/audit-doctor  → verify the audit toolchain is installed (run first)
/audit-init    → scaffold workspace + survey codebase
/audit-scan    → run all automated tools
/audit-onchain → inspect a deployed Program ID (authority, verifiable build, blast radius)
/audit-report  → generate professional report (Markdown + HTML + SARIF + JSON)
```

Or drive it in natural language: *"audit the withdraw instruction"*, *"is this program ID verified?"*, *"audit the v1→v2 upgrade"*, *"is there a flash-loan exploit here?"*. Six specialized agents (lead-auditor, competition-auditor, vuln-researcher, economic-auditor, diff-auditor, report-writer) handle long-running or parallel work.

## Judging criteria mapping

**Usefulness** — Security is the #1 blocker for protocol adoption; every serious Solana protocol needs an audit before mainnet. This skill puts production-grade audit methodology in reach of any founder or engineer, not only teams that can afford a $50k+ engagement. It also covers the recurring real-world jobs that source-only tools skip: auditing an *already-deployed* program by ID, reviewing an *upgrade diff*, and modeling *economic* exploits.

**Novelty** — Differentiators no other skill in the ecosystem properly covers:

- **`skill/onchain-analysis.md` + `/audit-onchain`** — audit a *deployed* program: dump and hash the on-chain bytecode, confirm it matches the public source via `solana-verify` / OtterSec verified builds, classify the upgrade authority (EOA / Squads / governance / immutable) and timelock, and quantify blast radius. A perfect source review is worthless if the deployed bytecode doesn't match — almost nothing in the kit addresses this gap.
- **`skill/native-pinocchio-patterns.md`** — 9 vulnerability patterns for *non-Anchor* programs (raw `solana-program` and Pinocchio), where owner/signer/discriminator checks are all manual. Pinocchio adoption is a 2026 reality and these programs lose every safety guarantee Anchor provides automatically.
- **`skill/diff-audit.md` + `diff-auditor` agent** — differential/upgrade audits with invariant-regression analysis and state-migration safety: review only the change between v1 and v2, the most common real request for a live protocol.
- **`skill/defi-economic-exploits.md` + `economic-auditor` agent** — flash-loan profitability modeling, oracle/spot-price manipulation, first-depositor share inflation, rounding-direction value leaks, liquidation abuse — the class behind Mango, Solend, Crema, Nirvana.
- **`skill/ai-agent-vulnerabilities.md`** — 12 patterns (A1–A12) for programs that interact with AI agents: on-chain prompt injection, session-key scope creep, LLM oracle manipulation, instruction hallucination, cross-agent trust escalation. 2026 attack surface no other skill touches.
- **`skill/evm-to-solana-mapping.md`** — SWC Registry → Solana mapping for auditors with EVM backgrounds, plus cross-chain DeFi patterns.
- **`agents/competition-auditor.md`** — speed-optimized agent for Code4rena / Cantina / Sherlock with per-protocol playbooks, PoC time targets, and duplicate-proofing.

**Quality** — Production-grade throughout: CVSS 3.1 scoring with on-chain adaptations, PoC-first discipline, 10 non-negotiable audit rules, refined semgrep rules (honest about what static analysis can/can't catch), CI integration (GitHub code scanning via SARIF), a publication-quality self-contained HTML report template, and a deliberately vulnerable example program (`examples/vulnerable-vault/`) with **8 clearly-mapped planted bugs** (3 Critical) spanning missing signer/owner checks, arbitrary CPI, account reuse, insecure closing, arithmetic, precision, and governance — each tagged to its pattern number, used to demo and regression-test the skill end-to-end with `/audit-doctor` → `/audit-init` → `/audit-scan` → `/audit-report`.

**Fit** — An explicit positioning table in `SKILL.md` shows exactly how it complements (not duplicates) every existing security skill in the kit. Follows the `solana-game-skill` structure exactly: `skill/` with progressive disclosure (load one file at a time), `agents/`, `commands/`, `rules/`, installer, README, MIT license. `install.sh` copies cleanly into `~/.claude/skills/` and registers itself in `~/.claude/CLAUDE.md`.
