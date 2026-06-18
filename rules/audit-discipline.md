---
description: "Mandatory rules for audit work. These override any other guidance — no exceptions."
---

# Audit Discipline Rules

These rules are non-negotiable. They apply to all audit work regardless of timeline pressure or team preference.

---

## Rule 1: No finding is Critical/High without a PoC

Every Critical and High finding MUST have a passing exploit test or a concrete step-by-step attack that the team can reproduce. If you cannot build the PoC, lower the severity to Medium and note "PoC not yet developed."

The exception: governance/upgrade authority issues where the "exploit" is "admin key is compromised" — document as High with an explicit risk statement instead of a code PoC.

---

## Rule 2: Never mark a finding "Fixed" without verifying

Before updating a finding's status to Fixed:
1. Run the original PoC — it must now fail
2. If no PoC existed: manually verify the fix addresses the root cause
3. Check that the fix doesn't introduce a new vulnerability
4. Confirm a regression test was added

"The team says it's fixed" is not verification.

---

## Rule 3: Always check account validation first

Before analyzing business logic in any instruction, complete the account validation checklist (vulnerability-patterns.md items 1–8). Account-level bugs are the most common Critical findings in Solana and the easiest to miss when distracted by complex business logic.

---

## Rule 4: Checked arithmetic is not optional to check

Check every arithmetic expression in financial-path code for overflow/underflow. Even if `overflow-checks = true` is set in the release profile, an overflow will panic (DoS) rather than wrap — which is still a bug if it can be triggered by a user.

---

## Rule 5: No `/// CHECK: safe` without justification

Every `/// CHECK:` comment on an `AccountInfo<'info>` account must have a specific justification explaining WHY no constraint is needed. A comment like `/// CHECK: safe` or `/// CHECK: not used` is unacceptable. Flag any such comment as Informational and ask for a proper justification.

---

## Rule 6: Cross-program trust boundaries must be explicit

For every CPI call, document: "Do we trust the callee to not call back?" and "Do we re-read state after the CPI?" Silence on these questions is a finding.

---

## Rule 7: Report severity is not negotiable with the client

Severity is based on exploitability and impact, not on the protocol team's comfort level. A finding that allows unauthorized fund withdrawal is Critical whether or not the team considers it "unlikely to be exploited." Document disagreements in the report under "Findings — Disputed Severity."

---

## Rule 8: Scope creep requires explicit sign-off

If during the audit you identify a significant vulnerability in out-of-scope code (e.g., an off-chain relayer, frontend key management), document it as an Informational finding with a note that it was outside scope but warrants investigation. Do not ignore it, but do not expand the audit unilaterally.

---

## Rule 9: Tool output is raw material, not findings

Never copy semgrep or clippy output directly into the report as confirmed findings. Every tool finding must be:
1. Manually verified to be exploitable
2. Triaged for severity based on actual impact
3. Investigated for false positive patterns

Automated tools miss ~70% of Critical findings in Solana programs. They are a starting point, not a conclusion.

---

## Rule 10: Incomplete audits must say so

If the audit timeline prevented full coverage, the report must explicitly state what was NOT reviewed and why. Never imply full coverage when partial coverage was done. A shorter, honest scope is better than an implied complete scope.
