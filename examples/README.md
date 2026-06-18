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
| **VULN-2** | `apply_config` | #2 Missing owner check | Critical | `config` is `AccountInfo` deserialized raw with `try_from_slice` and no owner check — an attacker passes a look-alike account they own to inject `fee_bps`. |
| **VULN-3** | `route_withdraw` | #3 Arbitrary CPI | Critical | `router_program` is caller-supplied and never validated against an expected program ID before `invoke`. |
| **VULN-5** | `transfer_internal` | #5 Account reuse / duplicate | High | No constraint preventing `from == to`; passing the same account for both aliases state. |
| **VULN-8** | `close_position` | #8 Insecure account closing | High | Lamports drained manually but data not zeroed and owner not reassigned (should use Anchor `close = receiver`). |
| **VULN-9** | `deposit`, `withdraw` | #9 Integer overflow/underflow | High | `vault.balance + amount` and `amount - fee` use unchecked `+`/`-`; `overflow-checks` is unset in `Cargo.toml`. |
| **VULN-10** | `withdraw` | #10 Precision loss | Medium | `amount * 100 / 10_000` truncates to 0 for small amounts → fee avoidance. |
| **VULN-21** | `set_authority` | #21 Governance/timelock | High | Authority can be changed instantly with no timelock (compounded by VULN-1). |

## Use it as a self-test

After an audit run, confirm the skill:

1. Flagged the three **Criticals** with PoCs — VULN-1 (missing signer), VULN-2 (missing owner check), VULN-3 (arbitrary CPI).
2. Caught the **duplicate-account** (VULN-5) and **insecure close** (VULN-8) issues.
3. Caught the **unchecked arithmetic** + **missing `overflow-checks`** config finding (VULN-9) and the **fee rounding** leak (VULN-10).
4. Noted the **instant authority change / no timelock** (VULN-21).
5. Emitted `audit-workspace/reports/` with `findings.json`, `findings.sarif`, the markdown report, and `audit-report.html`.

If it missed the Criticals, something is wrong with the install or routing — re-check `/audit-doctor` and that `~/.claude/CLAUDE.md` references the skill.

> The vulnerabilities here are intentional and clearly commented. This program exists solely to demonstrate and regression-test the auditor skill.
