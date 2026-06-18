# Audit Resources

Reference links, prior audit reports, and research to inform audit work. Load on demand.

---

## Prior Solana Audit Reports (Public)

Reading prior reports on similar protocols is one of the fastest ways to identify what to look for.

### OtterSec / Neodyme / Sec3 Reports
- [OtterSec public reports](https://github.com/otter-sec/solana-audits) — collection of Solana audit reports
- [Neodyme public advisories](https://neodyme.io/en/blog/) — post-mortems and disclosed findings

### Notable Solana Exploits (Learn from History)
- **Wormhole (2022)** — $320M — signature verification bypass via `verify_signatures` sysvar forgery
- **Solend (2022)** — TWAP oracle manipulation for partial liquidations
- **Mango Markets (2022)** — $114M — self-referential price manipulation via governance
- **Cashio (2022)** — $52M — missing ownership check on collateral accounts (classic vuln #2)
- **Crema Finance (2022)** — $9M — flash loan + tick account substitution
- **Nirvana Finance (2022)** — flash loan + own AMM price manipulation
- **Slope Wallet (2022)** — key logging in SDK, not a program bug

**Pattern**: Missing owner/signer checks and oracle manipulation appear in >60% of Solana exploits by value.

---

## Official Vulnerability Databases

- [Solana Foundation Security Advisories](https://github.com/solana-labs/security-advisories)
- [RustSec Advisory Database](https://rustsec.org/advisories/) — covers Rust crate CVEs
- [Anchor Security Advisories](https://github.com/coral-xyz/anchor/security/advisories)
- [SPL Token Program Issues](https://github.com/solana-labs/solana-program-library/security/advisories)

---

## Audit Methodology References

- [Trail of Bits Solana Auditing Guide](https://github.com/trailofbits/building-secure-contracts/tree/master/not-so-smart-contracts/solana)
- [Neodyme Solana Security Workshop](https://workshop.neodyme.io/) — hands-on vulnerable program exercises
- [OtterSec Audit Checklist](https://github.com/otter-sec/sol-ctf-framework) — CTF challenges modeling real vulnerabilities
- [Anchor Security Best Practices](https://www.anchor-lang.com/docs/the-program-module)

---

## Formal Verification Tools

- [Kani Rust Verifier](https://github.com/model-checking/kani) — open-source Rust model checker from AWS
- [Trident Fuzzer](https://github.com/Ackee-Blockchain/trident) — Solana/Anchor fuzz testing framework
- [QEDGen](https://github.com/qedgen/solana-skills) — spec-driven formal verification with `.qedspec`
- [Certora Prover](https://www.certora.com/) — commercial formal verification (EVM + Solana in development)

---

## Solana Program Security Primitives

- [Solana Account Model](https://solana.com/docs/core/accounts) — understand ownership and account types
- [Cross-Program Invocations](https://solana.com/docs/core/cpi) — CPI mechanics and signer seeds
- [Program Derived Addresses](https://solana.com/docs/core/pda) — PDA derivation and canonical bumps
- [Solana BPF Loader](https://docs.solanalabs.com/runtime/programs#bpf-loader) — upgradeable program mechanics
- [SPL Token 2022 Extensions](https://spl.solana.com/token-2022/extensions) — extension types and CPI forwarding

---

## CVSS Calculator

Use the official CVSS 3.1 calculator for scoring:
- [NVD CVSS Calculator](https://nvd.nist.gov/vuln-metrics/cvss/v3-calculator)
- See [report-generation.md](report-generation.md) for on-chain CVSS adaptations

---

## CTF Practice (Skill Building)

- [Neodyme Solana Security Workshop](https://workshop.neodyme.io/) — guided vulnerable programs to exploit
- [Neodyme Solana CTF repo](https://github.com/neodyme-labs/solana-ctf) — the workshop's challenge programs
- [OtterSec CTF framework](https://github.com/otter-sec/sol-ctf-framework) — harness for building/solving Solana CTF challenges
- [Blueshift challenges](https://learn.blueshift.gg/en/challenges) — Anchor & Pinocchio security exercises

---

## Ecosystem Security Tools

| Tool | Purpose | Link |
|------|---------|------|
| `cargo-audit` | CVE scanning | crates.io/crates/cargo-audit |
| `cargo-geiger` | Unsafe code detection | crates.io/crates/cargo-geiger |
| `trident` | Anchor fuzzer | github.com/Ackee-Blockchain/trident |
| `kani` | Rust model checker | github.com/model-checking/kani |
| `semgrep` | Pattern matching | github.com/semgrep/semgrep |
| Trail of Bits lints | Solana-specific clippy lints | github.com/trailofbits/solana-lints |
| `solana-verify` | Verifiable/reproducible builds | github.com/Ellipsis-Labs/solana-verifiable-build |
| OtterSec registry | Public verified-build status | verify.osec.io |

---

## Verifiable Builds & On-Chain Verification

- [Solana Verified Builds guide](https://solana.com/developers/guides/advanced/verified-builds)
- [solana-verifiable-build (Ellipsis Labs)](https://github.com/Ellipsis-Labs/solana-verifiable-build)
- [solana-verifiable-build (Solana Foundation)](https://github.com/solana-foundation/solana-verifiable-build)
- See [onchain-analysis.md](onchain-analysis.md) for the full deployed-program workflow

## Native & Pinocchio

- [Pinocchio (anza-xyz)](https://github.com/anza-xyz/pinocchio)
- [How to Build Solana Programs with Pinocchio (Helius)](https://www.helius.dev/blog/pinocchio)
- See [native-pinocchio-patterns.md](native-pinocchio-patterns.md) for non-Anchor vuln patterns
