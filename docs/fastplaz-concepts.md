# AWBMS Rust backend evidence and toolchain — FastPlaz concepts, repository-specific architecture

## Document control
- **Version:** v1.0.0
- **Updated:** 2026-11-10
- **Status:** Proposed architecture / research only; not a production feature.
- **Document Changeset:** First reference and implementation-boundary mapping.
- **Owning issue:** [#30](https://github.com/ahliweb/awbms/issues/30).

## Summary and confirmed local baseline
AWBMS is a Rust-native backend design, presently Stage 1 with Axum/Tokio/SQLx/PostgreSQL FORCE RLS verification spike; application parity, performance and rollback evidence incomplete.

## Intent and candidate adaptation
Use IDE/compiler integration ideas only to structure Cargo diagnostic and verification artifacts; optional external local developer bridge may consume `cargo check/test/clippy` output without becoming part of AWBMS.

## Authority and prohibitions
AWBMS is NOT a shared developer IDE/agent platform and must remain independent of AWCMS source/runtime; no production MCP endpoint, new business modules, or agent runtime added by this reference task.

## Existing related issues and anti-duplication
Architecture [#30](https://github.com/ahliweb/awbms/issues/30); Stage 1 [#1](https://github.com/ahliweb/awbms/issues/1).

## Six concrete validation scenarios
1. `cargo check --locked` diagnostic captures file, line, severity and toolchain.
2. `cargo test` with non-superuser role verifies tenant cross-access denied.
3. Tooling fails closed if provenance hash or verification fixture changes.
4. Graceful shutdown test emits machine-readable result.
5. Benchmark plan records environment, p50/p95/RSS before comparing Rust and Bun.
6. Restore/rollback rehearsal required before cutover.

## Proposed checks and test evidence
Preserve VG-01 partial, VG-03/04 passed, VG-05/09/12 pending, VG-15 independence; use `scripts/check-docs.sh`. No unmeasured performance claim.

## Rollout / handoff
Only Stage-1 documents and tests; reject creation of new Rust IDE service unless a separate measured business case and ADR exists.

## Source reference and limits
This document takes **concepts only** from [FastPlaz](https://github.com/fastplaz/fastplaz): [MCP/AI Agent Plane](https://github.com/fastplaz/fastplaz/blob/master/docs/MCP.md), [IDE workflow](https://github.com/fastplaz/fastplaz/blob/master/docs/AI_AGENT_by_FASTPLAZ.md), and [standalone operator concepts](https://github.com/fastplaz/fastplaz/blob/master/docs/STANDALONE-deployment.md). No Pascal, Lazarus, CGI, MVC/theme engine, or source-code copying. [FastPlaz license text](https://github.com/fastplaz/fastplaz/blob/master/LICENSE.txt) does not demonstrate explicit source-code permissions; independent implementation only.

## Cross-portfolio engineering invariants
Applies the **internal cross-portfolio standard**, alongside this repository's own AGENTS, ADR, CONTRIBUTING and current code reality. Human-approved **Gate 9 (cross-spec Definition of Ready)** precedes production feature code; human-approved **Gate 15 (release readiness)** precedes production. One issue/branch/PR per atomic scope. Never present a specification, green documentation check, or research as working production functionality.

Local MCP/tool connector, if ever required: loopback or authenticated IPC, host/Origin restrictions for HTTP, rotating scoped credentials, canonical-path and symlink protections, read/write/destructive approval tiers, no arbitrary shell, no default SQL/raw DB/secret access, bounded resource/time/cancellation, policy at actual tool/OS runtime, fail-closed unknown and redacted logs. Human sign-off for changes to authorization, production configuration or sensitive data.

## Relevant regulatory and control references
Evaluate applicability, not compliance by assertion: Indonesian UU 27/2022 (PDP), PP 71/2019 (PSTE), Perpres 95/2018 where SPBE applies; ISO/IEC 27001, 27002, 27005, 27017, 27018, 27034, 27701, ISO 22301, ISO/IEC 20000-1, ISO/IEC 42001, ISO/IEC 15408; OWASP LLM/Agentic Security, NIST AI RMF. Obtain independent control evidence and applicable edition before any claim of certification.

## Acceptance evidence rubric
For any follow-on: link spec/ADR and owning issue; record source commit, toolchain/runtime version, commands executed, actual error/result, denied access tests and evidence SHA. Mark unrun checks **NOT EXECUTED**. Approvals and publication/deployment status are distinct. No sensitive personal data or production secrets in prompts, logs or GitHub artifacts.


## Changelog
- **v1.0.0 — 2026-11-10:** Documented evidence-backed, repo-specific adoption boundary, six practical cases and acceptance rubric.
