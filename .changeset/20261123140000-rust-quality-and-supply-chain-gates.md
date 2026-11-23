---
bump: patch
type: security
component: ci
---

Add Rust quality gates (check, rustdoc with warnings denied, doc tests, workspace lints) and a supply-chain baseline (cargo-deny, cargo audit, Dependabot, exception policy). Bump tokio to 1.53.3 and url to 2.5.9 for RUSTSEC-2026-0386 and RUSTSEC-2026-0592. Issues #21 and #7.
