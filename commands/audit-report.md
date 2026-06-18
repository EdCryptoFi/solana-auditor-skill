---
description: "Generate a professional audit report from the findings in audit-workspace/findings-tracker.md. Produces a publication-ready markdown report following Trail of Bits / OtterSec structure with CVSS scores, PoC references, and remediation recommendations."
---

You are generating the final audit report. Load [skill/report-generation.md](../skill/report-generation.md) before executing these steps.

## Step 1: Gather Inputs

```bash
# Show current findings state
echo "=== Current Findings ==="
cat audit-workspace/findings-tracker.md

# Show environment for report header
echo ""
echo "=== Environment ==="
cat audit-workspace/environment.md

# Count findings by severity
echo ""
echo "=== Severity Summary ==="
grep "Critical\|High\|Medium\|Low\|Informational" audit-workspace/findings-tracker.md | grep -v "^#\|^|" | head -20
```

## Step 2: Collect Report Metadata

Ask the user for any missing values:
- Protocol name and description
- Audit period dates
- Auditor name(s) 
- Report version (1.0 for initial, 1.1 for post-remediation)

## Step 3: Generate the Report

The report is created at `audit-workspace/reports/audit-report-v[VERSION].md`.

Structure to follow (from report-generation.md):

```
1. Cover Page
2. Executive Summary (severity table + top 3 findings in plain English)
3. Scope and Methodology
4. Risk Classification
5. Findings — Critical (sorted by CVSS score descending)
6. Findings — High
7. Findings — Medium
8. Findings — Low
9. Findings — Informational
10. Formal Verification Results (if applicable)
11. Appendix A: Tool Outputs
12. Appendix B: Limitations
```

For each finding, use the template:
```markdown
## [SEVERITY] FINDING-NNN: [Title]

**Severity**: [Level]  
**CVSS**: [score] ([vector])  
**Category**: [Account Validation / Arithmetic / CPI / Business Logic / etc.]  
**Location**: `[file.rs:line-range]`  
**Status**: Open / Fixed (commit `[hash]`) / Acknowledged

### Description
[2–3 sentences. What the bug is, where it lives.]

### Impact
[What an attacker can do. Quantify if possible.]

### Proof of Concept
[Code or step-by-step for Critical/High. Evidence citation for Medium/Low.]

### Recommendation
[Specific fix. Before/after code for Critical/High.]
```

## Step 4: Quality Gate

Before finalizing:

```bash
# Check all line references are real
echo "Checking file references..."
grep -oP '`[^`]+\.rs:\d+' audit-workspace/reports/*.md | while read ref; do
    file=$(echo $ref | cut -d: -f1 | tr -d '`')
    line=$(echo $ref | cut -d: -f2)
    if [ -f "$file" ]; then
        echo "✅ $ref"
    else
        echo "❌ NOT FOUND: $ref"
    fi
done

# Count findings in report vs tracker
echo ""
echo "Finding IDs in report:"
grep -oP 'FINDING-\d+' audit-workspace/reports/*.md | sort -u

echo ""
echo "Finding IDs in tracker:"
grep -oP 'FINDING-\d+' audit-workspace/findings-tracker.md | sort -u
```

Verify:
- [ ] All file:line references exist in the codebase at the right line
- [ ] Severity counts in executive summary match actual finding counts
- [ ] All Critical/High have PoC code or test name
- [ ] All CVSS scores are calculated (not estimated)
- [ ] Report version and date are correct
- [ ] Scope section is accurate

## Step 5: Output

```bash
echo "Report generated:"
ls -la audit-workspace/reports/

echo ""
echo "Word count:"
wc -w audit-workspace/reports/*.md

echo ""
echo "Finding summary:"
grep "^\*\*Severity\*\*:" audit-workspace/reports/*.md | sort | uniq -c | sort -rn
```

The report is ready for review by the lead-auditor before delivery.
