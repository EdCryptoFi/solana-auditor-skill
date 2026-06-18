---
name: solana-auditor
description: Full-lifecycle Solana security auditor. Orchestrates the complete audit engagement: threat modeling, automated scanning, manual deep review, formal verification, professional report generation, and remediation verification. Not a pre-audit prep tool — this IS the audit.
user-invocable: true
---

# Solana Security Auditor

Production-grade audit intelligence for Solana programs. Routes to the right skill file based on the current audit phase.

> **Extends**: [solana-dev-skill](https://github.com/solana-foundation/solana-dev-skill) — Core Solana program knowledge

**How this skill differs from others in the kit:**

| Skill | Purpose | Audience |
|-------|---------|---------|
| **solana-auditor** (this) | Conduct the full audit engagement end-to-end | Security researchers auditing third-party code |
| [trailofbits/solana-vulnerability-scanner](../trailofbits/) | Automated 6-pattern scanner | Quick automated pre-check |
| [trailofbits/audit-prep-assistant](../trailofbits/) | Prepare your code for an external audit | Builders getting ready for external review |
| [frankcastleauditor/safe-solana-builder](../safe-solana-builder/) | Write secure code from scratch | Builders preventing bugs at write-time |
| [qedgen](../qedgen/) | Spec-driven formal verification (`.qedspec` → Kani/Lean) | Teams adopting spec-first development |

This skill is for **conducting** an audit of existing code, not for writing code or preparing for someone else to audit.

---

## Default Audit Stack (2026)

| Layer | Tool | Purpose |
|-------|------|---------|
| **Static analysis** | `cargo clippy` + custom lints | Compile-time issues |
| **Fuzzing** | Trident 0.9+ | Coverage-guided fuzz testing |
| **Property testing** | Trident fuzz + Mollusk | Invariant verification |
| **Formal verification** | Kani 0.57+ | Mathematical proofs |
| **Vulnerability scanner** | semgrep + custom rules | Pattern matching |
| **Dependency audit** | `cargo audit` | Known CVEs |
| **Runtime analysis** | Surfpool / LiteSVM | Transaction simulation |

---

## Three Audit Modes

Choose your mode before loading skill files:

| Mode | When to use | Primary agent | Time frame |
|------|------------|--------------|-----------|
| **Firm** | Full professional engagement, production protocol | `lead-auditor` | Days–weeks |
| **Competition** | Code4rena / Cantina / Sherlock contest | `competition-auditor` | Hours–days |
| **Self-audit** | Builder reviewing their own code before launch | `vuln-researcher` | Hours |

---

## Audit Phases — Quick Reference

| Phase | What happens | Start here |
|-------|-------------|-----------|
| **0 — Scoping** | Understand the codebase, threat model, attack surface | [audit-lifecycle.md §0](audit-lifecycle.md#phase-0-scoping) |
| **1 — Automated** | Run all tools, collect raw findings | [tools-setup.md](tools-setup.md) + [audit-lifecycle.md §1](audit-lifecycle.md#phase-1-automated-analysis) |
| **2 — Manual review** | Systematic line-by-line review against vuln checklist | [vulnerability-patterns.md](vulnerability-patterns.md) |
| **3 — Deep analysis** | Exploit PoCs, economic attacks, cross-program risks | [audit-lifecycle.md §3](audit-lifecycle.md#phase-3-deep-analysis) |
| **4 — Formal verification** | Mathematical properties for critical invariants | [formal-verification.md](formal-verification.md) |
| **5 — Reporting** | Professional report with CVSS scores and PoCs | [report-generation.md](report-generation.md) |
| **6 — Remediation** | Verify fixes, regression-test, re-score findings | [audit-lifecycle.md §6](audit-lifecycle.md#phase-6-remediation-verification) |

---

## Skill Routing — Load Only What You Need

| User asks / audit task | Load this file |
|------------------------|---------------|
| "Start an audit", "scope this program", "threat model" | [audit-lifecycle.md](audit-lifecycle.md) |
| "Find vulnerabilities", "review this instruction", "check account validation" | [vulnerability-patterns.md](vulnerability-patterns.md) |
| "Formal verification", "prove invariants", "Kani", "property tests" | [formal-verification.md](formal-verification.md) |
| "Write the report", "format findings", "CVSS score", "executive summary" | [report-generation.md](report-generation.md) |
| "Set up tools", "Trident setup", "cargo audit", "CI for audits" | [tools-setup.md](tools-setup.md) |
| "AI agent", "session keys", "LLM oracle", "agent wallet", "prompt injection" | [ai-agent-vulnerabilities.md](ai-agent-vulnerabilities.md) |
| "I'm from Ethereum", "EVM comparison", "Slither equivalent", "SWC" | [evm-to-solana-mapping.md](evm-to-solana-mapping.md) |
| "References", "prior audits", "known exploits" | [resources.md](resources.md) |

Load **one file at a time**. Do not pre-load all files — context is precious.

---

## Agent Routing

Spawn specialized agents for long-running or parallel audit work:

| Task | Agent | Model |
|------|-------|-------|
| Orchestrate a full audit engagement (firm mode) | [lead-auditor](../agents/lead-auditor.md) | opus |
| Speed-optimized competitive audit (Code4rena / Cantina) | [competition-auditor](../agents/competition-auditor.md) | opus |
| Deep-dive vulnerability research on one instruction | [vuln-researcher](../agents/vuln-researcher.md) | sonnet |
| Generate or refine the audit report | [report-writer](../agents/report-writer.md) | sonnet |

---

## Slash Commands

| Command | What it does |
|---------|-------------|
| `/audit-init` | Scaffold audit workspace, initialize findings tracker |
| `/audit-scan` | Run all automated tools and collect raw findings |
| `/audit-report` | Generate formatted audit report from findings |

---

## Rules

All audit work follows [../rules/audit-discipline.md](../rules/audit-discipline.md):
- Never mark a finding as mitigated without running a regression test
- All Critical/High findings require a working PoC
- Never skip the account validation checklist
- Severity scores use CVSS 3.1 adapted for on-chain context

---

## Severity Classification

| Severity | On-chain meaning |
|----------|-----------------|
| **Critical** | Direct fund loss, unauthorized mint/burn, full protocol takeover |
| **High** | Partial fund loss, privilege escalation, irreversible state corruption |
| **Medium** | Griefing, DoS, economic manipulation without direct theft |
| **Low** | Best-practice deviations with limited exploitability |
| **Informational** | Code quality, documentation, non-exploitable patterns |
