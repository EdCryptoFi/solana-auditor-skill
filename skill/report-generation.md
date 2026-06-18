# Audit Report Generation

Templates and guidance for producing professional, firm-quality audit reports. Matches the structure used by Trail of Bits, OtterSec, Sec3, and Neodyme.

---

## Report Structure

```
1. Cover Page
2. Executive Summary
3. Scope and Methodology
4. Risk Classification
5. Findings (Critical → High → Medium → Low → Informational)
6. Formal Verification Results (if applicable)
7. Appendix A: Test Coverage
8. Appendix B: Tool Outputs
9. Appendix C: Limitations
```

---

## Cover Page Template

```markdown
# Security Audit Report

**Protocol**: [Protocol Name]  
**Version audited**: commit `[hash]` (branch: `[branch]`)  
**Audit period**: [Start Date] – [End Date]  
**Report version**: 1.0 (Initial) / 1.1 (Post-remediation)  
**Status**: Draft / Final

---

| Item | Detail |
|------|--------|
| Program ID | `[PublicKey]` |
| Network | Mainnet / Devnet |
| Upgradeable | Yes / No |
| Upgrade authority | `[address or multisig]` |
| Language | Rust / Anchor [version] |
| Total lines of code | [N] |
| Auditors | [Name(s)] |

---

**Disclaimer**: This audit does not constitute a guarantee of security. It represents a best-effort review of the code at the specified commit. New vulnerabilities may be discovered after this report's date.
```

---

## Executive Summary Template

```markdown
## Executive Summary

[Protocol Name] is a [brief description: e.g., "concentrated liquidity AMM on Solana"]. 
This audit reviewed [scope: e.g., "the core pool program and fee management instructions"].

### Overall Risk Assessment

| Category | Count |
|----------|-------|
| Critical | [N] |
| High | [N] |
| Medium | [N] |
| Low | [N] |
| Informational | [N] |
| **Total** | **[N]** |

### Summary of Findings

The most significant findings were:

- **[FINDING-001]** ([Critical]): [One sentence description. e.g., "Unauthorized users can drain the SOL vault due to a missing signer check on the withdraw instruction."]
- **[FINDING-002]** ([High]): [One sentence description.]
- **[FINDING-003]** ([Medium]): [One sentence description.]

[If clean audit]: No critical or high severity findings were identified. [N] medium and [N] low severity issues were found, representing [description of risk level].

### Remediation Status

At time of this report:
- [N] findings have been fixed and verified
- [N] findings are acknowledged with planned remediation
- [N] findings are acknowledged and accepted as acceptable risk
```

---

## Finding Template

Every finding follows this exact structure:

```markdown
---

## [SEVERITY] [FINDING-ID]: [Title]

**Severity**: Critical / High / Medium / Low / Informational  
**CVSS Score**: [X.X] ([Vector String])  
**Category**: [e.g., Account Validation / Arithmetic / CPI Safety / Business Logic]  
**Location**: `[file path]:[line range]`  
**Status**: Open / Fixed (commit `[hash]`) / Acknowledged / Won't Fix

---

### Description

[2–4 sentences explaining what the vulnerability is and where it exists. 
Be precise: name the instruction, account, or function.]

### Impact

[Concrete description of what an attacker can do. 
Quantify if possible: "attacker can drain up to X SOL from the vault".]

### Proof of Concept

[Working exploit code or step-by-step attack scenario. 
All Critical and High findings MUST have runnable PoC code.]

```rust
#[tokio::test]
async fn exploit_[finding_id]() {
    // Setup
    let mut context = setup().await;
    let attacker = Keypair::new();
    
    // Pre-attack state
    let vault_before = get_balance(&context, &vault_pda).await;
    
    // Attack
    let tx = Transaction::new_signed_with_payer(
        &[malicious_instruction],
        Some(&attacker.pubkey()),
        &[&attacker],
        context.last_blockhash,
    );
    context.process_transaction(tx).await.unwrap();
    
    // Verify damage
    let vault_after = get_balance(&context, &vault_pda).await;
    assert!(vault_after < vault_before, "Funds drained");
}
```

### Recommendation

[Specific fix. Show the before/after code if helpful. Be actionable.]

```rust
// Before (vulnerable)
pub authority: AccountInfo<'info>,

// After (safe)
pub authority: Signer<'info>,
```

### Remediation Verification

[Filled in after fix is reviewed]

- Fix commit: `[hash]`
- PoC result post-fix: FAILS as expected ✅
- Regression test added: ✅ / ❌
- Auditor sign-off: [Name]
```

---

## CVSS Scoring for On-Chain Programs

Use CVSS 3.1 with these on-chain adaptations:

### Attack Vector (AV)
- **Network (N)**: Exploitable via any RPC call, no special access required
- **Adjacent (A)**: Requires being in the same transaction or epoch
- **Local (L)**: Requires local key access (e.g., upgrade authority compromise)
- **Physical (P)**: Not applicable to smart contracts

### Attack Complexity (AC)
- **Low (L)**: Straightforward single-transaction exploit
- **High (H)**: Requires specific state conditions, multiple transactions, or significant capital

### Privileges Required (PR)
- **None (N)**: Any wallet can exploit
- **Low (L)**: Requires a funded wallet or minor protocol interaction
- **High (H)**: Requires admin/authority role

### User Interaction (UI)
- **None (N)**: Fully permissionless attack
- **Required (R)**: Victim must sign a transaction or interact

### Impact (CIA)
- **Confidentiality**: Rarely applicable on-chain (data is public) — use N unless private state exists
- **Integrity**: Can attacker corrupt state? High if critical PDAs, Medium if peripheral state
- **Availability**: Can attacker DOS the program? DoS = High impact

### Typical Scores for Common On-Chain Findings

| Finding Type | Typical CVSS | Score Range |
|-------------|-------------|------------|
| Missing signer → fund drain | AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:H/A:H | 9.1 Critical |
| Missing owner check | AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:H/A:H | 9.1 Critical |
| Arithmetic overflow → drain | AV:N/AC:H/PR:L/UI:N/S:U/C:N/I:H/A:H | 7.5 High |
| Stale oracle → price manip | AV:N/AC:H/PR:N/UI:N/S:U/C:N/I:H/A:N | 5.9 Medium |
| Missing rent exemption | AV:N/AC:H/PR:N/UI:N/S:U/C:N/I:L/A:L | 4.8 Medium |
| Compute unit exhaustion | AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:N/A:H | 7.5 High |

---

## Scope and Methodology Section Template

```markdown
## Scope

### In Scope

| Component | Files | Commit |
|-----------|-------|--------|
| Core program | `programs/[name]/src/**` | `[hash]` |
| Instruction handlers | `programs/[name]/src/instructions/**` | `[hash]` |
| State accounts | `programs/[name]/src/state/**` | `[hash]` |

### Out of Scope

- Frontend / web application
- Off-chain indexers or bots
- Third-party programs called via CPI (Solana Token Program, System Program)
- Deployment infrastructure

### Methodology

This audit employed the following techniques:

1. **Automated Analysis**: cargo-audit, cargo-clippy, Trident fuzzer, semgrep
2. **Manual Code Review**: Line-by-line review against the 25-item vulnerability checklist
3. **Threat Modeling**: STRIDE analysis for each privileged operation and asset
4. **Formal Verification**: [Kani / Trident invariants] for arithmetic and state properties
5. **Proof of Concept**: Working exploits developed for all Critical and High findings
6. **Economic Modeling**: Flash loan and oracle manipulation scenarios analyzed
```

---

## Remediation Status Table

For the final (post-remediation) version of the report:

```markdown
## Remediation Summary

| ID | Title | Severity | Status | Fix Commit |
|----|-------|----------|--------|-----------|
| FINDING-001 | Missing signer on withdraw | Critical | Fixed ✅ | `abc1234` |
| FINDING-002 | Integer overflow in fee calc | High | Fixed ✅ | `def5678` |
| FINDING-003 | Stale oracle price accepted | Medium | Acknowledged | N/A |
| FINDING-004 | Missing rent check on close | Low | Fixed ✅ | `ghi9012` |
| FINDING-005 | Unchecked program upgrade auth | Info | Acknowledged | N/A |
```

---

## Generating the Report: Step-by-Step

1. **Collect all findings** from your `audit-workspace/findings-raw.md`
2. **Triage**: Assign severity and CVSS score to each finding
3. **Deduplicate**: Merge findings that share root cause
4. **Order**: Critical → High → Medium → Low → Informational
5. **Write PoCs**: All Critical/High need runnable test code
6. **Write recommendations**: Specific, actionable, with code examples
7. **Draft executive summary**: After all findings are written
8. **Review pass**: Check that every code reference has correct line numbers
9. **Share with team**: Internal review before sending to protocol
10. **Post-remediation update**: Add fix status and re-run tool suite

---

## Machine-Readable Output (SARIF / JSON)

Always emit a structured findings file alongside the human report. CI pipelines, GitHub code scanning, and dashboards consume SARIF; protocol tooling often prefers a simpler JSON. Generate both from the same findings tracker so they never drift.

### Why
- **GitHub Code Scanning**: a `.sarif` uploaded via `github/codeql-action/upload-sarif` renders findings inline on the PR diff.
- **Regression gating**: CI can fail a build if a finding of severity ≥ High reappears.
- **Diff audits**: machine-readable findings make v1→v2 finding deltas computable (see diff-audit.md).

### SARIF 2.1.0 template

```json
{
  "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
  "version": "2.1.0",
  "runs": [
    {
      "tool": {
        "driver": {
          "name": "solana-auditor",
          "informationUri": "https://github.com/solanabr/solana-auditor-skill",
          "version": "1.0.0",
          "rules": [
            {
              "id": "SOL-001-missing-signer",
              "name": "MissingSignerCheck",
              "shortDescription": { "text": "Missing signer check" },
              "helpUri": "https://github.com/solanabr/solana-auditor-skill/blob/main/skill/vulnerability-patterns.md",
              "properties": { "security-severity": "9.1" }
            }
          ]
        }
      },
      "results": [
        {
          "ruleId": "SOL-001-missing-signer",
          "level": "error",
          "message": { "text": "withdraw() transfers from the vault without checking authority.is_signer; any caller can drain funds." },
          "locations": [
            {
              "physicalLocation": {
                "artifactLocation": { "uri": "programs/vault/src/instructions/withdraw.rs" },
                "region": { "startLine": 24, "endLine": 31 }
              }
            }
          ],
          "properties": {
            "cvss": "9.1",
            "cvssVector": "AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:H/A:H",
            "severity": "Critical",
            "status": "Open"
          }
        }
      ]
    }
  ]
}
```

**Severity mapping** (SARIF `level` ← audit severity): Critical/High → `error`, Medium → `warning`, Low/Informational → `note`. Always also set `properties.security-severity` (string CVSS, e.g. `"9.1"`) so GitHub buckets it correctly.

### Simpler `findings.json` (protocol-friendly)

```json
{
  "audit": { "protocol": "Example", "commit": "abc1234", "auditor": "solana-auditor", "date": "2026-06-18" },
  "summary": { "critical": 1, "high": 0, "medium": 2, "low": 1, "informational": 3 },
  "findings": [
    {
      "id": "FINDING-001",
      "title": "Missing signer on withdraw",
      "severity": "Critical",
      "cvss": 9.1,
      "cvssVector": "AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:H/A:H",
      "category": "Account Validation",
      "location": "programs/vault/src/instructions/withdraw.rs:24-31",
      "status": "Open",
      "hasPoc": true,
      "recommendation": "Change `authority: AccountInfo` to `authority: Signer`."
    }
  ]
}
```

### CI consumption

```yaml
# In .github/workflows/audit-ci.yml
      - name: Upload audit findings to code scanning
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: audit-workspace/reports/findings.sarif

      - name: Fail build on unresolved High+ findings
        run: |
          jq -e '[.findings[] | select(.status=="Open" and (.severity=="Critical" or .severity=="High"))] | length == 0' \
            audit-workspace/reports/findings.json
```

> Keep `findings.json` as the source of truth in `audit-workspace/`. Generate the markdown report, the SARIF file, and the remediation table from it — one dataset, three views.

---

## Interactive HTML Report (optional deliverable)

For a client- or PR-friendly view, emit a **single self-contained HTML file** alongside the markdown. It has severity-filter cards, a search box, and collapsible findings with PoCs — no server, no CDN, no network. Many protocol teams prefer this to a PDF.

The template ships with the skill at [report-template.html](report-template.html). It reads from the same `findings.json`, so it never drifts from the markdown report.

```bash
# Generate the HTML report from findings.json (run from the audited project)
cp ~/.claude/skills/solana-auditor/skill/report-template.html \
   audit-workspace/reports/audit-report.html

# Replace the inline `const DATA = {...}` placeholder with the real findings.json.
# Robust approach — inject the JSON between the two marker comments:
python3 - <<'PY'
import json, re, pathlib
data = pathlib.Path("audit-workspace/reports/findings.json").read_text()
html = pathlib.Path("audit-workspace/reports/audit-report.html").read_text()
html = re.sub(r"const DATA = \{[\s\S]*?\n\};",
              "const DATA = " + data.strip().rstrip(";") + ";",
              html, count=1)
pathlib.Path("audit-workspace/reports/audit-report.html").write_text(html)
print("audit-report.html written")
PY
```

**Where it lives**: the *template* stays in the skill repo (`skill/report-template.html`); the *generated report* lands in the audited project at `audit-workspace/reports/audit-report.html`. Never commit client reports into the skill.

The `findings[]` objects support these optional fields for richer rendering: `description`, `impact`, `poc`, `recommendation` (in addition to the core `id`, `title`, `severity`, `cvss`, `cvssVector`, `category`, `location`, `status`, `hasPoc`).

---

## Quality Gate

Before sending the report:

```
[ ] Every finding has a location with exact file:line
[ ] Every Critical/High has a working PoC
[ ] Every finding has a concrete recommendation
[ ] CVSS scores assigned and calculated (not estimated)
[ ] Executive summary matches findings count
[ ] Scope section accurately describes what was reviewed
[ ] Limitations section is honest about what was NOT checked
[ ] Report version matches remediation round (1.0 initial, 1.1 post-fix)
[ ] Machine-readable findings.json + findings.sarif emitted from the same data
```
