---
description: "Run the full automated analysis suite on a Solana program. Executes cargo-audit, clippy security lints, semgrep, and attempts to run existing tests. Collects all raw findings into audit-workspace/tool-output/. Run after /audit-init, before manual review."
---

You are running the Phase 1 automated analysis suite. Execute each tool and collect output. Do not stop on tool failures — log the error and continue.

## Step 1: Dependency Audit

```bash
echo "=== [1/5] cargo-audit ===" | tee audit-workspace/tool-output/01-cargo-audit.txt
echo "Started: $(date -u)" >> audit-workspace/tool-output/01-cargo-audit.txt

if command -v cargo-audit &> /dev/null; then
    cargo audit --json 2>&1 | tee -a audit-workspace/tool-output/01-cargo-audit.txt
    
    # Human-readable summary
    ADVISORY_COUNT=$(cargo audit 2>&1 | grep "^error\[" | wc -l | tr -d ' ')
    echo ""
    echo "Advisory count: $ADVISORY_COUNT"
    if [ "$ADVISORY_COUNT" -gt "0" ]; then
        echo "⚠️  FINDINGS: cargo-audit found $ADVISORY_COUNT advisories"
    else
        echo "✅ cargo-audit: clean"
    fi
else
    echo "⚠️  cargo-audit not installed. Run: cargo install cargo-audit --locked"
fi
```

## Step 2: Clippy Security Lints

```bash
echo ""
echo "=== [2/5] clippy security lints ===" | tee audit-workspace/tool-output/02-clippy.txt

cargo clippy --all-targets --all-features -- \
  -D clippy::arithmetic_side_effects \
  -D clippy::checked_conversions \
  -D clippy::cast_possible_truncation \
  -D clippy::cast_sign_loss \
  -D clippy::cast_possible_wrap \
  -D clippy::integer_division \
  -D clippy::modulo_arithmetic \
  -D clippy::panic \
  -D clippy::unwrap_used \
  -D clippy::expect_used \
  -D clippy::indexing_slicing \
  2>&1 | tee -a audit-workspace/tool-output/02-clippy.txt

CLIPPY_ERRORS=$(grep -c "^error\[" audit-workspace/tool-output/02-clippy.txt 2>/dev/null || echo 0)
CLIPPY_WARNINGS=$(grep -c "^warning\[" audit-workspace/tool-output/02-clippy.txt 2>/dev/null || echo 0)

echo ""
echo "Clippy: $CLIPPY_ERRORS errors, $CLIPPY_WARNINGS warnings"
if [ "$CLIPPY_ERRORS" -gt "0" ]; then
    echo "⚠️  FINDINGS: clippy found $CLIPPY_ERRORS security errors"
else
    echo "✅ clippy: no security errors"
fi
```

## Step 3: Semgrep Pattern Matching

```bash
echo ""
echo "=== [3/5] semgrep ===" | tee audit-workspace/tool-output/03-semgrep.txt

if command -v semgrep &> /dev/null; then
    # Run official Solana ruleset
    semgrep --config p/solana \
            --json \
            --output audit-workspace/tool-output/03-semgrep.json \
            . 2>&1 | tee -a audit-workspace/tool-output/03-semgrep.txt
    
    SEMGREP_FINDINGS=$(cat audit-workspace/tool-output/03-semgrep.json 2>/dev/null | python3 -c "import sys,json; d=json.load(sys.stdin); print(len(d.get('results', [])))" 2>/dev/null || echo "?")
    echo ""
    echo "Semgrep findings: $SEMGREP_FINDINGS"
else
    echo "⚠️  semgrep not installed. Run: pip3 install semgrep"
fi
```

## Step 4: Run Existing Test Suite

```bash
echo ""
echo "=== [4/5] test suite ===" | tee audit-workspace/tool-output/04-tests.txt

# Try native tests first
cargo test-sbf 2>&1 | tee -a audit-workspace/tool-output/04-tests.txt
TEST_EXIT=$?

# Try Anchor tests if available
if [ -f "Anchor.toml" ]; then
    anchor test --skip-local-validator 2>&1 | tee -a audit-workspace/tool-output/04-tests.txt
fi

if [ "$TEST_EXIT" -eq "0" ]; then
    echo "✅ Tests pass"
else
    echo "⚠️  Tests FAILING — investigate before manual review"
fi
```

## Step 5: Trident Fuzzer (if configured)

```bash
echo ""
echo "=== [5/5] trident fuzz (if available) ===" | tee audit-workspace/tool-output/05-trident.txt

if command -v trident &> /dev/null && [ -d "trident-tests" ]; then
    # Short initial fuzz run — 60 seconds to catch easy bugs
    echo "Running 60-second fuzz check..."
    trident fuzz run fuzz_0 -- -max_total_time=60 2>&1 | tee -a audit-workspace/tool-output/05-trident.txt
    
    CRASH_COUNT=$(ls .trident/fuzzing/corpus/fuzz_0/crashes/ 2>/dev/null | wc -l | tr -d ' ')
    if [ "$CRASH_COUNT" -gt "0" ]; then
        echo "🚨 CRASHES FOUND: $CRASH_COUNT crashes in .trident/fuzzing/corpus/fuzz_0/crashes/"
        echo "Run longer: trident fuzz run fuzz_0 -- -max_total_time=3600"
    else
        echo "✅ No crashes in 60-second run (run longer for thorough coverage)"
    fi
else
    echo "Trident not configured. Initialize with: trident init"
    echo "See skill/tools-setup.md for setup guide"
fi
```

## Step 6: Summary Report

```bash
echo ""
echo "╔═══════════════════════════════════════════════════╗"
echo "║  Phase 1 Automated Analysis — Summary              ║"
echo "╚═══════════════════════════════════════════════════╝"
echo ""
echo "Tool outputs saved to: audit-workspace/tool-output/"
echo ""
echo "Review tool output files:"
ls -la audit-workspace/tool-output/
echo ""
echo "Now update audit-workspace/findings-tracker.md with raw findings."
echo "Then start Phase 2 manual review using skill/vulnerability-patterns.md"
```

## Interpreting Results

After the scan completes, triage each finding:

| Finding source | Likely severity | Action |
|---------------|----------------|--------|
| cargo-audit error[vulnerability] | Medium–High | Investigate CVE, check if exploit path exists |
| clippy arithmetic_side_effects | High if in financial logic | Verify with manual review |
| clippy unwrap_used | Low–Medium | Check if on user-controlled input |
| semgrep solana rules | High | Verify manually, these are high-precision |
| Trident crash | Critical–High | Replay and analyze crash immediately |
| Failing test | Investigate | Understand WHY before proceeding |

**Do not skip manual review because tools were clean.** Automated tools miss the majority of Solana-specific bugs — they cannot reason about CPI trust boundaries, business logic, or economic attacks.
