# AWBMS documentation

Every document in this repository, what it is for, and how far it can be
trusted.

## Index

### Architecture

| Document | Status | Purpose |
|---|---|---|
| [AWBMS Rust Architecture Validation](architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md) | Conditionally approved | The binding architecture decisions and their registers. Start at §0. |
| [Architecture index](architecture/README.md) | Current | Navigation and reading order for the above. |
| [Master Blueprint](architecture/AWBMS-MASTER-BLUEPRINT.md) | Stage 1 in progress | Stage 1 formalisation of the validated architecture. |
| [Stage 1 gate register](architecture/AWBMS-STAGE-1-GATES.md) | Authoritative status | Per-gate status, backed by [`gates.json`](architecture/gates.json). |
| [Pre-production evidence specification](architecture/AWBMS-PRE-PRODUCTION-EVIDENCE.md) | Specification only | CI invariant gates and `VG-05`/`VG-09`/`VG-12` evidence shape. |


### Research (non-binding)

| Document | Status | Purpose |
|---|---|---|
| [Fuchsia/Zircon portability readiness](research/fuchsia-zircon-awbms-portability-readiness-2026-11-23.md) | Research only · 2026-11-23 | Rust target/SDK support, Linux/API ownership, trust boundaries and isolated PoC gates; not an accepted ADR |
| [Blockchain & crypto business backend readiness](research/blockchain-crypto-business-backend-readiness-2026-11-23.md) | Research only · 2026-11-23 | Read-only chain integration, provenance, custody/legal classification, OJK/BI limits and deferred operating roles |
| [Fuchsia Rust SDK and dependency portability](research/fuchsia-rust-sdk-dependency-portability-2026-11-23.md) | Research only · 2026-11-23 | `#33`: per-crate Fuchsia disposition; rustc Tier 2 vs end-developer policy; NO-GO for native Rust today |
| [Fuchsia FEMU/HTTPS spike plan](research/fuchsia-femu-https-spike-plan-2026-11-23.md) | Plan only · not executed | `#34`: isolated spike plan, blocked on `#33` and owner authorisation |
| [Blockchain business models vs Indonesian regulation](research/blockchain-business-models-indonesia-regulation-2026-11-23.md) | Research only · counsel review required | `#35`: activity classification matrix with a citation verification log |
| [Chain RPC indexer finality and reorg spec](research/chain-rpc-indexer-finality-reorg-spec-2026-11-23.md) | Spec only · not executed | `#36`: read-only EVM indexer states, cursor, reorg and fail-closed matrix |
| [Hash anchoring vs permissioned ledger](research/hash-anchoring-vs-permissioned-ledger-2026-11-23.md) | Research only · 2026-11-23 | `#37`: signed audit/anchoring vs Fabric/EVM, business trigger and no-go |
| [Crypto custody, signing and ledger safety boundary](research/crypto-custody-signing-ledger-safety-boundary-2026-11-23.md) | Research only · 2026-11-23 | `#38`: reasons not to launch custody; three-model reconciliation invariants |

### Decisions

| Document | Status |
|---|---|
| [ADR index](adr/README.md) | Current |
| [ADR-0001 — Record architecture decisions](adr/0001-record-architecture-decisions.md) | Accepted |
| [ADR-0002 — Semantic versioning with `vX.Y.Z` and changesets](adr/0002-semantic-versioning-and-changesets.md) | Accepted |
| [ADR-0003 — Documentation correctness is an executable gate](adr/0003-executable-documentation-gate.md) | Accepted |
| [ADR-0004 — Adopt product development flow v2.0.0](adr/0004-adopt-product-development-flow-v2.md) | Accepted |
| [ADR-0005 — License AWBMS under MIT and enforce a dependency licence allow-list](adr/0005-licence-mit-and-dependency-licence-allow-list.md) | Accepted |
| [ADR-0006 — Accept the Master Blueprint dispositions of OD-01, OD-02, OD-04 and OD-09](adr/0006-accept-blueprint-dispositions-od-01-02-04-09.md) | Accepted |
| [ADR-0007 — Keep the VG-01 legacy baseline pinned for Stage 1](adr/0007-keep-vg-01-legacy-baseline-pinned-for-stage-1.md) | Accepted |
| [ADR template](adr/TEMPLATE.md) | — |

### Versioning

| Document | Purpose |
|---|---|
| [Versioning policy](versioning/VERSIONING.md) | The `vX.Y.Z` model, bump rules, release procedure |
| [Writing changesets](versioning/CHANGESETS.md) | When a changeset is required and how to write one |

### Security

| Document | Status |
|---|---|
| [Security documentation index](security/README.md) | Placeholder — see its own gap list |
| [Security policy and vulnerability reporting](../SECURITY.md) | Current — pre-release, NOT PRODUCTION READY |
| [Supply-chain policy](security/supply-chain.md) | Current — `cargo-deny`, `cargo audit`, Dependabot, exception policy |

## How to read a document here

Claims are labelled. The vocabulary is defined in [§0.2 of the
validation](architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#02-claim-labels)
and used across all documents:

| Label | Trust it because | Do not trust it for |
|---|---|---|
| `[F]` | It was checked against this repository at a stated commit | Anything about AWCMS or the outside world |
| `[C]` | Someone recorded it | Anything, until its gate is discharged |
| `[A]` | The design depends on it | It has not been established |
| `[D]` | It is binding | It may still be wrong; check the gate |
| `[R]` | It is a sensible default | A team may override it with a recorded decision |
| `[O]` | It is deliberately unresolved | Do not resolve it in passing |
| `[G]` | It names the evidence that would settle the matter | A gate reference is not a passed gate |

The most important thing to understand about this documentation set: of sixteen
verification gates, **four pass, one is partial, and eleven are open.**

- `VG-03`, `VG-04` — the Rust stack composes as assumed. Real evidence, narrow scope.
- `VG-15`, `VG-16` — standing structural conditions checked on every commit. They
  say nothing about whether an architectural claim is true.
- `VG-01` — **partial**. The AWCMS inventory is frozen and confirms `C-01`–`C-03`;
  `C-04`, `C-05` and `C-07` remain unverified.

**No gate covering AWBMS behaviour has passed.** Parity, performance and rollback
are all open, and no AWBMS module exists to test.

## Known gaps

These are absences, recorded so that nobody mistakes them for oversights.

| Gap | Blocked on | Tracked as |
|---|---|---|
| AWCMS inventory frozen and consumed by checks, but `VG-01` still **partial** — gate classification, executable authorization harness, route ownership inventory and behaviour fixtures missing ([coverage note](architecture/evidence/VG-01-executable-coverage-2026-11-23.md)) | a `vg01-freeze` run and parity-harness work | `VG-01` |
| No threat model, privacy analysis, ERD, RBAC/ABAC matrix, or SLOs | Master Blueprint, §50 steps 2–3 and 5–8 | §49.2, `OD-06` |
| No AWBMS application module — the workspace holds only `crates/verification` | Blueprint sign-off, §51 step 10 | `VG-02` |
| No performance baseline for AWBMS or AWCMS (acceptance policy decided: no-regression, ADR-0006) | benchmark corpus, §23.1 | `VG-09`, `OD-06` |
| No approver named on the architecture validation | an owner accepting the conditional approval | Appendix F |

Licensing is decided by [ADR-0005](adr/0005-licence-mit-and-dependency-licence-allow-list.md): AWBMS is MIT
([`LICENSE`](../LICENSE)), and dependencies are held to the allow-list in `deny.toml`, enforced in CI.

## Documents that do not exist yet

The [validation's §9](architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#9-proposed-cargo-workspace)
proposes `docs/adr/`, `docs/architecture/` and `docs/security/`. The first two
now exist. `docs/security/` holds the supply-chain policy; the rest of it is a placeholder.

The [engineering lifecycle in
§50](architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#50-mandatory-engineering-lifecycle)
lists eighteen stages (product development flow v2.0.0, [`ADR-0004`](adr/0004-adopt-product-development-flow-v2.md)). Stage 1 — the Master Blueprint — is in progress (issue #1). Its
required contents are in §49.2, and its entry conditions in §49.1.
