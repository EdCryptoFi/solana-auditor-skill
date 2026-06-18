# Examples — Demo the Auditor End-to-End

A deliberately vulnerable program so you can see the `solana-auditor` skill work on real code in one sitting. **This code is for teaching only — never deploy it.**

## `vulnerable-vault/`

A minimal Anchor "vault" with planted bugs. Use it to demo the full workflow:

```
# 1. Check your toolchain
/audit-doctor

# 2. From inside examples/vulnerable-vault/
/audit-init
/audit-scan

# 3. Ask the auditor to review it
"Audit the withdraw and set_authority instructions in this program"

# 4. Generate the report (markdown + SARIF + JSON + HTML)
/audit-report
```

A successful demo finds the planted vulnerabilities below and produces a report with at least one Critical.

## Answer key

Each tag in `src/lib.rs` maps to a pattern in [`../skill/vulnerability-patterns.md`](../skill/vulnerability-patterns.md):

| Tag | Location | Pattern | Severity | What's wrong |
|-----|----------|---------|----------|--------------|
| **VULN-1** | `withdraw` / `set_authority` accounts | #1 Missing signer check | Critical | `authority` is `UncheckedAccount`, not `Signer`. The pubkey is public, so anyone can pass it without signing — `require_keys_eq!` checks identity, not signature. Lets any caller withdraw or seize control. |
| **VULN-9** | `deposit`, `withdraw` | #9 Integer overflow/underflow | High | `vault.balance + amount` and `amount - fee` use unchecked `+`/`-`; `overflow-checks` is unset in `Cargo.toml`. |
| **VULN-10** | `withdraw` | #10 Precision loss | Medium | `amount * 100 / 10_000` truncates to 0 for small amounts → fee avoidance. |
| **VULN-21** | `set_authority` | #21 Governance/timelock | High | Authority can be changed instantly with no timelock (compounded by VULN-1). |

## Use it as a self-test

After an audit run, confirm the skill:

1. Flagged **VULN-1 as Critical with a PoC** (the headline finding).
2. Caught the **unchecked arithmetic** and the **missing `overflow-checks`** config finding.
3. Noted the **fee rounding** direction leak.
4. Emitted `audit-workspace/reports/` with `findings.json`, `findings.sarif`, the markdown report, and `audit-report.html`.

If it missed VULN-1, something is wrong with the install or routing — re-check `/audit-doctor` and that `~/.claude/CLAUDE.md` references the skill.

> The vulnerabilities here are intentional and clearly commented. This program exists solely to demonstrate and regression-test the auditor skill.
