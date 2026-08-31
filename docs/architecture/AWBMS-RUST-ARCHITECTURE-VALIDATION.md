# AWBMS Rust Backend Architecture Validation

| Field | Value |
|---|---|
| Project | AWBMS — AhliWeb Backend Management System |
| Repository | `ahliweb/awbms` |
| Specification version | v0.3.0 |
| Status | Architecture validation — **conditionally approved** to enter Stage 1 (see [§49](#49-master-blueprint-entry-conditions) and [Appendix F](#appendix-f--approval-record)) |
| Original validation date | 2026-08-29 |
| Last revision | 2026-08-31 (register completion and defect correction; no architectural decision reversed) |
| Evidence basis | This repository at the version above, re-derived by `scripts/check-docs.sh`, plus external claims recorded by the original validation (not re-verified here) |
| Decision scope | Rust-based backend architecture, ecosystem selection, scalability, maintainability, security, AWCMS compatibility, migration strategy |
| Explicitly out of scope | Product requirements, UX, pricing/commercial strategy, org/staffing, per-module functional design, SLO target values |
| Owner / approver | *unassigned — see [Appendix F](#appendix-f--approval-record)* |
| Review cadence | Re-validate at the start of each major migration or specification stage, and whenever any gate in [Appendix E](#appendix-e--verification-gate-register) changes state |

---

## 0. How to read this document

### 0.1 Repository state (verified)

**[F]** The `ahliweb/awbms` repository now contains a pinned Rust toolchain (`rust-toolchain.toml`), a Cargo workspace with a committed `Cargo.lock`, a `crates/verification` spike with executable PostgreSQL RLS and graceful-shutdown tests, a frozen AWCMS inventory under `contracts/legacy/awcms/`, the Stage 1 Master Blueprint, and the governance material described in [§0.5](#05-document-control). There is still no `migrations/` directory, no application module, and no git submodule.

> **This section was rewritten at v0.3.0, and the reason matters more than the content.** Up to v0.2.0 it read *"contains exactly one tracked file — this document"*, pinned to commit `5f8f9aa`. That was true when written and false shortly afterwards. A commit cannot state its own SHA, so a hand-maintained commit reference in a document header is guaranteed to lag by at least one revision.
>
> The claim is therefore split: what the repository does **not** contain is re-derived on every push by `scripts/check-docs.sh` (`VG-16`) and `tools/vg15_independence.rs` (`VG-15`); what it **does** contain is described in prose that a reader should treat as a snapshot. Run the scripts rather than trusting this paragraph.

Two consequences follow. **Both were weakened by the Stage 1 merge and neither has been retired:**

1. **Almost nothing in this document describes implemented behavior.** The exception is now real but narrow: the `VG-04` spike proves that Axum, Tokio, SQLx and PostgreSQL FORCE RLS compose as this architecture assumes. No AWBMS *module* exists, no benchmark has been run, and no parity test exists. Every other architectural statement remains a proposal.
2. **Claims about systems outside this repository were unverifiable from it, and several no longer are.** `VG-01` has frozen a machine-generated AWCMS inventory with recorded source commits and SHA-256 digests, which confirms `C-01`, `C-02`, `C-03` and `C-06` — see [Appendix A](#appendix-a--recorded-external-claims). The remaining claims still carry gates and are not treated as established facts.

### 0.2 Claim labels

Statements are labelled where their epistemic status matters. Unlabelled prose in [§4](#4-rust-ecosystem-validation)–[§58](#58-final-recommendation) is design recommendation, equivalent to **[R]**. (The range previously stopped at §56, silently leaving §57 and §58 unclassified.)

| Label | Meaning | Register |
|---|---|---|
| **[F]** | Verified fact — checked against this repository at the stated commit | — |
| **[C]** | Recorded external claim — asserted by the original validation, **not** verified here; carries a gate | [Appendix A](#appendix-a--recorded-external-claims) |
| **[A]** | Assumption — relied upon, not established; falsification changes the design | [Appendix B](#appendix-b--assumption-register) |
| **[D]** | Decision — binding unless superseded by a recorded successor decision | [Appendix C](#appendix-c--decision-register) |
| **[R]** | Recommendation — a default that a team may override by recording a new decision | — |
| **[O]** | Open decision — deliberately unresolved; must close before the stage named in the register | [Appendix D](#appendix-d--open-decision-register) |
| **[G]** | Verification gate — the executable or documentary evidence that discharges a claim or decision | [Appendix E](#appendix-e--verification-gate-register) |

### 0.3 Normative language

`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY` and `OPTIONAL` in capitals carry RFC 2119 / RFC 8174 force. (The original list named `MUST` twice and omitted the permissive keywords entirely, which left capitalised `SHOULD` and `MAY` undefined in a document that uses both.) Lowercase "should", "must" and "may" in the discussion sections are ordinary prose and carry **no** normative force — normative force lives in the decision register ([Appendix C](#appendix-c--decision-register)) and the gate register ([Appendix E](#appendix-e--verification-gate-register)). This split is deliberate: it keeps the discussion readable while making the binding surface small enough to review and audit.

### 0.4 Validation status at a glance

| Category | Count | Where |
|---|---:|---|
| Facts verified against this repository | 4 | [§0.1](#01-repository-state-verified), [§3.1](#31-current-state-verified) (×2), [§23](#23-performance-expectations) |
| Recorded external claims awaiting verification | 7 | [Appendix A](#appendix-a--recorded-external-claims) |
| Assumptions | 8 | [Appendix B](#appendix-b--assumption-register) |
| Binding decisions | 32 | [Appendix C](#appendix-c--decision-register) |
| Open decisions — total | 9 | [Appendix D](#appendix-d--open-decision-register) |
| Open decisions blocking Blueprint sign-off | 6 | [Appendix D](#appendix-d--open-decision-register) |
| Verification gates | 16 | [Appendix E](#appendix-e--verification-gate-register) |
| Verification gates **passed** | 4 | `VG-03`, `VG-04`, `VG-15`, `VG-16` |
| Verification gates **partial** | 1 | `VG-01` |
| Verification gates **open** | 11 | [Appendix E](#appendix-e--verification-gate-register) |

The gate rows are the ones that matter, and they changed at v0.3.0. Up to v0.2.0 every gate was open and this table's honest summary was *"none has produced evidence"*. Stage 1 changed that for four of them.

Read the four passes narrowly. `VG-03` and `VG-04` establish that the *stack* works — a pinned toolchain, and a spike in which Axum, Tokio, SQLx and FORCE RLS compose as assumed. `VG-15` and `VG-16` are standing structural conditions, not architectural evidence. **No gate covering AWBMS behaviour has passed:** parity (`VG-05`), performance (`VG-09`), rollback (`VG-12`) and every module-level gate remain open, and `VG-01` is partial.

**What this validation does and does not establish.** It establishes that a coherent, internally consistent Rust architecture *can* be specified for AWBMS, and it records the trade-off reasoning for each major selection. It does **not** establish that the selected crates meet AWBMS requirements under load, that AWCMS behavior has been inventoried, or that migration is feasible on any particular schedule. Those are the open items in Appendices A, D and E.

### 0.5 Document control

**[D]** This document is versioned as part of the repository, not separately (`AD-31`). The header's *Specification version* field is written by `scripts/version-bump.sh` and is mechanically asserted equal to the root `VERSION` file and the latest `CHANGELOG.md` section.

This replaces the standalone `Document version` field carried by revisions up to 1.1. That field versioned one file inside a repository that had no version of its own, which is the drift pattern [§2.1](#21-documentation-drift-warning) argues against: two independently maintained version numbers for one artifact, with nothing keeping them honest.

**[D]** The structural invariants of this document are enforced by an executable gate rather than by review attention (`AD-32`, `VG-16`). `scripts/check-docs.sh` verifies that every `AD-`, `OD-`, `VG-`, `C-` and `AS-` identifier referenced here resolves to a register row, that no register row has outlived every reference to it, that every internal anchor resolves, and that the repository has acquired no dependency on AWCMS (`VG-15`).

The gate is binding about *form* and silent about *content*. It can confirm that `AD-04` exists, is defined once and is cited; it cannot confirm that `AD-04` is a good decision. Reviewers retain that job — the gate exists so they can spend attention on it.

| Change to this document | Version effect |
|---|---|
| A binding decision is reversed or superseded | MAJOR |
| A decision, gate, assumption or claim is added | MINOR |
| A correction that changes no conclusion | PATCH |

The full policy, including the `0.x` rule and the reservation of `1.0.0` for the first accepted production cutover, is `docs/versioning/VERSIONING.md`.

---

## 1. Executive decision

**[D]** AWBMS is to be developed as a **new, independent Rust-native backend platform**, not as a source-code translation of AWCMS (`AD-01`).

The recommended target architecture is summarised below. Each line is binding as the referenced decision; full context, rationale and consequences are in [Appendix C](#appendix-c--decision-register).

| Element | Decision |
|---|---|
| **Rust 2024 Edition**, pinned stable toolchain | `AD-02` |
| **Tokio** async runtime | `AD-03` |
| **Axum + Tower** HTTP foundation | `AD-04` |
| **PostgreSQL** as the authoritative system of record | `AD-05` |
| **SQLx** as the primary PostgreSQL data-access layer | `AD-06` |
| **Modular monolith**, microservice-extractable | `AD-07` |
| **RBAC + ABAC + Separation of Duties + PostgreSQL FORCE RLS** | `AD-10` |
| **Opaque server-side sessions** for human users | `AD-11` |
| **Transactional PostgreSQL outbox** for domain events | `AD-13` |
| **PostgreSQL-backed job queues** with horizontally scalable Rust workers | `AD-14` |
| **OpenAPI and AsyncAPI contract-first** interfaces | `AD-19` |
| **`tracing` + OpenTelemetry/OTLP** observability | `AD-20` |
| **Redis/Valkey optional**, never an initial correctness dependency | `AD-18` |
| **Cloudflare R2 behind a storage port**, not domain-level provider coupling | `AD-17` |
| **Docker + Coolify + Traefik + Cloudflare** deployment and edge | `AD-30` |
| **AWCMS compatibility layer + parity suite**, no source/runtime dependency on the old repository | `AD-24` |

**Confidence and basis.** These selections rest on documented ecosystem characteristics and on the architectural fit reasoning in [§5](#5-http-framework-decision-axum), [§6](#6-database-access-decision-sqlx) and [§8](#8-modular-monolith-decision). They do **not** rest on any AWBMS measurement — no prototype, benchmark or load test has been run ([§0.1](#01-repository-state-verified)). `AD-04` and `AD-06` are the two selections most exposed to that gap and are gated by `VG-04`.

The architectural principle is:

> **AWBMS SHALL preserve proven security and behavioral invariants from AWCMS where they remain valid, while redesigning the implementation natively for Rust.**

AWBMS is therefore **not “AWCMS rewritten line-by-line in Rust.”** It is a new backend platform whose requirements and controls are informed by the engineering lessons already proven in AWCMS.

---

## 2. Source-of-truth review

> **Status of this section.** Everything below was observed by the original validation against the external repositories `ahliweb/awcms` and `ahliweb/awcms-astro`. **None of it is verifiable from `ahliweb/awbms`**, which contains no AWCMS fixtures ([§0.1](#01-repository-state-verified)). The observations are preserved verbatim as recorded claims `C-01`–`C-05` so that a later reader can re-check them rather than inherit them. Treat this section as *the input to* `VG-01`, not as its output.

**[C]** `C-01` — the state reviewed was `ahliweb/awcms` at release `v10.1.0`, commit `11f2e95a47b1328a820f976d60f978c38a067903` dated 2026-08-28, together with `ahliweb/awcms-astro`. No commit reference was recorded for `awcms-astro`; that omission is itself a gap, since the consumer contract surface in [§39](#39-awcms--awbms-parity-harness) depends on it.

**[C]** `C-02` — the AWCMS implementation contains **24 registered modules**:

1. `logging`
2. `tenant_admin`
3. `profile_identity`
4. `identity_access`
5. `module_management`
6. `domain_event_runtime`
7. `sync_storage`
8. `workflow_approval`
9. `email`
10. `reporting`
11. `theming`
12. `media_library`
13. `blog_content`
14. `tenant_domain`
15. `visitor_analytics`
16. `data_lifecycle`
17. `seo_distribution`
18. `form_drafts`
19. `site_search`
20. `newsletter`
21. `site_profile`
22. `comments`
23. `idn_admin_regions`
24. `push_delivery`

**[C]** `C-03` — the AWCMS *architecture documentation* reports migrations through `sql/148`, PostgreSQL `FORCE ROW LEVEL SECURITY` for tenant-scoped tables, separated database roles, module composition rules, OpenAPI/AsyncAPI contracts, audit/event systems, and a read/write SYSTEM administration surface.

> **Resolved at v0.3.0 — the caution below is retained because its reasoning still applies elsewhere.** `C-03` was sourced from AWCMS prose, which [§2.1](#21-documentation-drift-warning) states is known to lag implementation, so `sql/148` was treated as a documentation figure of unknown accuracy rather than a verified schema state. `VG-01` has now re-derived it from the migration ledger: **148 migrations, 134 tables with RLS, all 134 `FORCE`**. The prose was right.
>
> That outcome does not vindicate trusting prose. It means this particular figure survived checking, and the only reason anyone knows that is that it was checked. The capability descriptions in `C-03` remain prose and remain unverified.

**[C]** `C-04` — AWCMS contains machine-enforced architectural gates, including checks for:

- module DAG integrity;
- route ownership;
- table-write ownership;
- module composition;
- job ownership and environment allow-lists;
- tenant transaction context;
- RBAC/ABAC enforcement;
- authorization chokepoints;
- access-decision logging;
- subject-data coverage;
- data lifecycle coverage;
- OpenAPI contract and consumer coverage;
- migration immutability;
- environment configuration coverage;
- dependency/security readiness;
- documentation and generated-artifact drift.

**[C]** `C-05` — these gates were created in response to engineering failures found in production or review, and therefore encode invariants that a feature list does not capture.

`C-05` carries the strategic weight of this whole section: it is the reason AWBMS treats AWCMS as a *requirements source* rather than a codebase to translate (`AD-01`). If `C-05` proves overstated — if the gates are aspirational rather than enforced — then the AWCMS invariant set is weaker evidence than assumed, and the AWBMS gate design in [§28](#28-maintainability-and-code-quality-gates) must be re-derived from threat modelling instead of inheritance. `VG-01` MUST therefore record, for each listed gate, whether it is *enforced in CI*, *advisory*, or *absent*.

**[D]** AWBMS adopts the proven invariants while implementing them with Rust-native mechanisms (`AD-01`, and the gate set in [§28](#28-maintainability-and-code-quality-gates)). The mapping from AWCMS invariant to AWBMS control is in [Appendix G](#appendix-g--awcms-invariant-traceability).

### 2.1 Documentation drift warning

**[C]** `C-03` (above) illustrates the problem: the AWCMS architecture prose contains historical sections whose counts and capability descriptions lag the actual module registry. Therefore:

> **The current implementation registry, migrations, contracts, executable tests, and generated inventories SHALL outrank prose during AWBMS requirements extraction.**

This document is subject to the same rule it imposes. Its own §2 is prose, and [Appendix A](#appendix-a--recorded-external-claims) exists so that its claims expire rather than harden.

### 2.2 Required AWCMS source inventory (`VG-01`)

Before migration work begins, AWBMS MUST generate and freeze a machine-verifiable AWCMS source inventory. Prose summaries do not discharge this gate. The inventory MUST be produced by a re-runnable script committed to `ahliweb/awbms`, MUST record the AWCMS commit SHA it was generated from, and MUST contain at minimum:

| Inventory item | Derived from (not from prose) | Supersedes |
|---|---|---|
| Module registry: key, kind, dependencies, capabilities | the registry source file | `C-02` |
| Applied migration list with per-file SHA-256 and ledger state | `migrations/` + migration history table | `C-03` |
| Tables with RLS and `FORCE ROW LEVEL SECURITY` enabled | live `pg_class` / `pg_policy` introspection | `C-03` |
| Database roles and grants | live `information_schema` introspection | `C-03` |
| Route inventory with owning module and auth requirement | router source or generated route table | `C-03` |
| OpenAPI + AsyncAPI documents | contract files as committed | `C-03` |
| Architectural gate list, each marked enforced / advisory / absent | CI configuration + gate scripts | `C-04`, `C-05` |
| Authorization vectors (principal × resource × expected decision) | executable authorization tests | — |
| Rate-limit and abuse-control dimensions per endpoint class | rate-limit implementation and its tests | `C-07` |
| Password and machine-credential storage formats in use | authentication implementation | — |

Each artifact MUST be stored under `contracts/legacy/awcms/` with the provenance fields required by [§3](#3-repository-independence). Once frozen, the inventory — not this section — is the requirements baseline.

---

## 3. Repository independence

`ahliweb/awbms` is a separate product repository (`AD-24`).

### 3.1 Current state (verified)

**[F]** This repository has **no** dependency on AWCMS of any kind: no git submodule, no `.gitmodules`, no Cargo manifest and therefore no path or git dependency, and no vendored AWCMS source. Independence is currently a fact, not merely an intention — and unlike the original statement of it, which named a commit and aged immediately, it is now re-checked on every push by `scripts/check-docs.sh` under gate `VG-15`.

**[F]** `contracts/legacy/awcms/` now exists and is populated. The fixture pipeline described below is no longer prospective: frozen static and live inventories are committed under `contracts/legacy/awcms/frozen/`, each carrying source commits (`awcms` `11f2e95a…`, `awcms-astro` `7b753be6…`), SHA-256 manifests, and regeneration tooling under `tools/`. `VG-01` is **PARTIAL**, not discharged — see [Appendix E](#appendix-e--verification-gate-register).

### 3.2 Prohibited couplings

AWBMS MUST NOT introduce source or runtime dependencies such as:

```text
../awcms
Git submodule -> ahliweb/awcms
Cargo git dependency -> ahliweb/awcms
runtime import from awcms
shared production filesystem coupling
```

Instead, compatibility artifacts MUST be imported as versioned, frozen fixtures:

```text
AWCMS
  │
  ├── OpenAPI snapshot
  ├── AsyncAPI snapshot
  ├── authorization vectors
  ├── schema/migration inventory
  ├── representative response fixtures
  └── consumer contract fixtures
        │
        ▼
contracts/legacy/awcms/
        │
        ▼
AWBMS compatibility + parity tests
```

Every imported legacy artifact MUST record:

- source repository;
- source commit SHA;
- import date;
- SHA-256 digest;
- contract/schema version where available;
- the AWBMS parity test(s) that consume it, so an unused fixture is detectable.

This makes compatibility reproducible without turning AWCMS into a build dependency. The provenance fields are what let a future reader distinguish "AWBMS matches AWCMS" from "AWBMS matches an undated snapshot of AWCMS."

---

## 4. Rust ecosystem validation

**[R]** The Rust backend ecosystem is judged sufficiently mature for AWBMS.

The basis for that judgement is qualitative: the crates below are widely deployed, actively maintained, and cover every capability AWBMS needs without requiring a bespoke implementation of transport, TLS, or database protocol. The judgement is *not* based on any AWBMS-specific evaluation, prototype or measurement ([§0.1](#01-repository-state-verified)). Maturity in general does not imply fitness for this workload; `VG-04` converts the judgement into evidence before the Blueprint freezes the stack.

### 4.1 Recommended baseline

**[R]** — no version numbers are pinned here deliberately. Concrete versions belong in `Cargo.toml`, `Cargo.lock` and `rust-toolchain.toml`, which are the only artifacts that can be verified by a build. A version table duplicated into prose is a drift source, and [§2.1](#21-documentation-drift-warning) forbids exactly that pattern.

| Concern | Recommendation |
|---|---|
| Language | Rust stable, pinned by `rust-toolchain.toml` |
| Edition | Rust 2024 |
| Async runtime | Tokio |
| HTTP framework | Axum |
| Middleware | Tower + Tower HTTP |
| Database | PostgreSQL |
| PostgreSQL access | SQLx |
| Serialization | Serde |
| Domain errors | `thiserror` |
| Bootstrap/application error aggregation | `anyhow` selectively |
| HTTP client | Reqwest |
| TLS | rustls |
| Password hashing | Argon2id |
| OIDC | `openidconnect` |
| Logging/tracing | `tracing` |
| Telemetry | OpenTelemetry / OTLP |
| Local cache | Moka when justified |
| Distributed cache | Redis/Valkey only when measured need exists |
| Object storage abstraction | `object_store` or a narrow AWBMS port |
| CLI | `clap` |
| Dependency security | RustSec, cargo-audit, cargo-deny, cargo-vet |

**[C]** `C-06` — the original validation recorded that Rust stable `1.98.0` had been released on 2026-08-20, and that Tokio, Axum, SQLx, Tower, Reqwest, Serde, rustls and OpenTelemetry were production-ready at that date. This is an external, time-sensitive claim; it is not verifiable from this repository and it decays with every release cycle. It MUST NOT be copied into the Blueprint. Resolve the actual toolchain version at bootstrap time (`VG-03`) and record it in `rust-toolchain.toml`, which then becomes the single source of truth.

**[D]** AWBMS pins the toolchain and critical dependencies rather than tracking latest releases (`AD-02`, `AD-22`).

### 4.2 Runtime and upgrade policy

**[R]** Initial policy:

```text
Rust          : pinned stable toolchain, recorded in rust-toolchain.toml
Edition       : 2024
Tokio         : single pinned minor line per release train
```

> **[O] `OD-02`** — "supported/LTS line where practical", as originally written, is not an actionable policy: it names no cadence, no owner and no upgrade trigger. `OD-02` MUST define the pin granularity, the routine upgrade cadence, and the expedited path for a RustSec advisory, before CI is established (step 2 of [§51](#51-initial-implementation-sequence-after-definition-of-ready)).

Dependency upgrades MUST pass:

1. compile/type checks;
2. unit and integration tests;
3. RLS/security regression tests;
4. contract tests;
5. concurrency/idempotency tests;
6. representative performance benchmarks.

---

## 5. HTTP framework decision: Axum

### 5.1 Alternatives reviewed

- Axum
- Actix Web

Both are viable production frameworks. Neither was disqualified on capability grounds.

### 5.2 Decision matrix

> **Scale definition.** Ratings are **reviewer judgement recorded at validation time, not measurements**. They mean: *Excellent* = directly supported, idiomatic, no adaptation needed; *Very good* = supported with minor adaptation; *Good* = supported with a documented workaround or companion crate; *Moderate/Possible* = achievable but against the grain. No rating in this document derives from a benchmark, and the ratings are not weighted or summed — the decision rests on the architectural argument in [§5.3](#53-decision), with the matrix as supporting context. Ratings that would change the outcome if wrong are gated by `VG-04`.

| Criterion | Axum | Actix Web | AWBMS decision |
|---|---:|---:|---|
| Throughput potential | Very high | Very high | No material blocker |
| Tokio alignment | Excellent | Good | Axum |
| Tower integration | Native | Different middleware model | Axum |
| Hyper ecosystem alignment | Native | Less direct | Axum |
| Tonic/gRPC future compatibility | Natural | Possible | Axum |
| Middleware composition | Excellent | Excellent | Slight Axum advantage |
| Architecture transparency | Excellent | Good | Axum |
| Maintainability for this design | Excellent | Excellent | Axum |

### 5.3 Decision

Use:

```text
Axum
Tower
Tower HTTP
Tokio
```

The reason is not that Actix is slow—it is not. Axum is preferred because the AWBMS architecture benefits from the common Tower/Hyper/Tokio service ecosystem for:

- tracing;
- request IDs;
- timeout layers;
- concurrency limits;
- compression;
- CORS;
- security headers;
- load shedding/backpressure;
- future RPC integration.

Raw HTTP benchmark differences are not an adequate reason to optimize away from architectural maintainability.

---

## 6. Database access decision: SQLx

### 6.1 Alternatives reviewed

- SQLx
- Diesel
- SeaORM

### 6.2 Decision matrix

> **Scale definition** as in [§5.2](#52-decision-matrix); ratings are reviewer judgement, not measurement. Two rows are known to be sensitive to how the criterion is read and MUST be re-checked against current upstream documentation under `VG-04` before `AD-06` is frozen: *Native async model* (the candidates differ in whether async is provided by the core crate or a companion crate, which materially changes the integration story) and *Compile-time query checks* (the three candidates verify different things — query text against a live schema, versus schema-derived types — so a single ordinal rating flattens a real distinction).

| Requirement | SQLx | Diesel | SeaORM |
|---|---:|---:|---:|
| Explicit SQL visibility | Excellent | Very good | Moderate |
| Native async model | Excellent | Good | Excellent |
| PostgreSQL-specific features | Excellent | Very good | Good |
| RLS transparency | Excellent | Very good | Good |
| Transaction control | Excellent | Excellent | Very good |
| Complex CTE/locking | Excellent | Very good | Good |
| `FOR UPDATE SKIP LOCKED` use | Natural | Good | Possible |
| Transactional outbox work | Natural | Good | Possible |
| Compile-time query checks | Strong | Strong | Different model |
| Fit for AWBMS | **Best** | Good | Secondary |

### 6.3 Why SQL must stay visible

AWBMS depends on database behavior that is part of the security/domain architecture:

- PostgreSQL RLS;
- `SET LOCAL` / transaction-local tenant context;
- advisory locks;
- CTEs;
- JSONB;
- `tsvector`/GIN;
- `pg_trgm`;
- `FOR UPDATE`;
- `SKIP LOCKED`;
- conditional mutation;
- legal hold enforcement;
- idempotency;
- transactionally coupled audit/outbox writes.

These should not be hidden behind a generic ORM abstraction.

### 6.4 Decision

Use **SQLx** as the primary persistence layer.

Prefer statically checked queries where practical:

```rust
sqlx::query!()
sqlx::query_as!()
```

Dynamic SQL is permitted only when bounded and tested.

SQLx offline query metadata should be generated and checked in CI where appropriate.

---

## 7. Recommended macro architecture

```text
                         Cloudflare
                     DNS / CDN / WAF
                            │
                         Traefik
                            │
               ┌────────────┴────────────┐
               │                         │
               ▼                         ▼
         AWBMS SERVER              API consumers
        Rust / Axum                 awcms-astro
             │                     mobile/web/etc.
       ┌─────┼─────────┐
       │     │         │
       ▼     ▼         ▼
     Auth  Modules   Contracts
       │     │         │
       └─────┼─────────┘
             │
           SQLx
             │
             ▼
        PostgreSQL
   FORCE RLS + RBAC/ABAC
             │
       ┌─────┴──────────┐
       │                │
       ▼                ▼
 transactional      jobs/read
    outbox           projections
       │                │
       └────────┬───────┘
                ▼
          AWBMS WORKER
               Rust
                │
       ┌────────┼─────────┐
       ▼        ▼         ▼
      R2      Email   External APIs
```

The initial production deployment should expose separate binaries/process roles while retaining one repository/workspace:

```text
awbms-server
awbms-worker
awbms-cli
```

They share domain and infrastructure crates but have different runtime permissions and responsibilities.

---

## 8. Modular monolith decision

AWBMS should start as a **modular monolith**.

### 8.1 Comparison

| Architecture | Scale potential | Transaction integrity | Operational simplicity | Development complexity | AWBMS fit |
|---|---:|---:|---:|---:|---:|
| Modular monolith | High | Excellent | Excellent | Moderate | **Best** |
| Microservices | Very high | Difficult | Poor initially | High | Premature |
| Coarse SOA | High | Moderate | Moderate | Moderate/high | Future option |
| Serverless functions | High in specific workloads | Weak for cross-function transactions | Moderate | High | Weak core fit |
| Unstructured monolith | Moderate | Excellent | Good | Becomes high | Rejected |

The primary bottlenecks of a backend such as AWBMS will usually be:

- PostgreSQL query/index design;
- lock contention;
- connection capacity;
- authorization complexity;
- external API latency;
- report generation;
- event backlog;
- storage and search operations.

Splitting modules into network services does not automatically solve these bottlenecks.

### 8.2 Microservice extraction rule

A module may be extracted only when evidence demonstrates one or more of:

- independently different scaling profile;
- independently different availability SLO;
- separate security/trust boundary;
- severe resource-isolation requirement;
- separate organizational ownership;
- materially different deployment cadence;
- high external workload whose isolation improves the system.

Likely future candidates, if measured:

- media processing;
- notification delivery;
- analytics ingestion;
- report/export generation;
- large-scale indexing/search — note that whether search stays in PostgreSQL (`tsvector`/GIN/`pg_trgm`) or moves to a dedicated index is itself unresolved (`OD-07`), and the answer changes whether this is an extraction candidate at all.

AWBMS should therefore be **microservice-extractable, not microservice-first**.

---

## 9. Proposed Cargo workspace

The initial structure should avoid both a giant flat crate and unnecessary crate explosion.

```text
awbms/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── deny.toml
│
├── apps/
│   ├── server/
│   ├── worker/
│   └── cli/
│
├── crates/
│   ├── kernel/
│   ├── config/
│   ├── database/
│   ├── security/
│   ├── authorization/
│   ├── observability/
│   ├── contracts/
│   ├── integrations/
│   └── modules/
│       ├── logging/
│       ├── tenant-admin/
│       ├── identity-access/
│       ├── workflow/
│       ├── blog-content/
│       └── ...
│
├── migrations/
│
├── contracts/
│   ├── openapi/
│   ├── asyncapi/
│   └── legacy/
│       └── awcms/
│
├── tests/
│   ├── contract/
│   ├── integration/
│   ├── parity/
│   ├── rls/
│   ├── security/
│   ├── concurrency/
│   └── e2e/
│
├── docs/
│   ├── adr/
│   ├── architecture/
│   └── security/
│
└── ops/
    ├── Dockerfile.production
    ├── compose/
    └── coolify/
```

A domain should retain vertical locality:

```text
blog-content/
├── domain/
├── application/
├── repository/
├── http/
├── events/
└── module.rs
```

Avoid global folders such as:

```text
controllers/
models/
repositories/
services/
```

because they encourage horizontal coupling across domains.

---

## 10. Module composition model

AWBMS should preserve the strongest AWCMS module-composition ideas in a Rust-native form.

Each module should expose an explicit descriptor containing at least:

```text
key
kind/type
dependencies
capabilities
permissions
routes
events
jobs
data ownership
lifecycle/subject-data descriptors
```

**[D]** The registry must be explicit and reviewed at compile time, not populated by runtime plugin discovery (`AD-08`). Runtime discovery would make the module set a deployment-time property, which defeats every gate below: a DAG that is only known at startup cannot be checked in CI.

Required architectural gates:

- unique module keys;
- acyclic dependency graph;
- declared capability dependencies;
- route ownership without collision;
- one module owns each mutable table;
- job ownership;
- event ownership/versioning;
- permission declaration/enforcement consistency.

### 10.1 One-table-one-writer principle

**[D]** A mutable table has exactly one authoritative module owner (`AD-09`).

Other modules should interact through:

- application ports;
- capabilities;
- read models;
- domain events;
- explicit cross-domain services.

Direct cross-module mutation should fail architecture checks.

---

## 11. PostgreSQL architecture

PostgreSQL remains the recommended database.

Changing runtime language does not justify changing the database.

### 11.1 Database roles

For a new AWBMS-native database, define at minimum:

```text
awbms_migrator
awbms_app
awbms_worker
awbms_operator
```

`awbms_app` should be:

```text
NOSUPERUSER
NOBYPASSRLS
not database/schema/table owner
no DDL
minimum required DML
```

Worker privileges should be narrower than migrator/operator privileges and explicit per workload.

### 11.2 Tenant security

Defense in depth should use all three layers:

```text
application authorization
        +
explicit tenant predicate
        +
PostgreSQL FORCE RLS
```

RLS is a defense layer, not an excuse to omit tenant predicates.

This design assumes the tenant is the isolation unit, with no requirement for sub-tenant partitions or deliberate cross-tenant data sharing (`AS-05`). A product requirement for either would change the policy model rather than merely extend it. The isolation evidence is gate `VG-06`.

### 11.3 Tenant transaction wrapper

Tenant context should be transaction-local and bound as data, not interpolated into SQL.

Illustrative pattern:

```rust
sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
    .bind(tenant_id.to_string())
    .execute(&mut *tx)
    .await?;
```

The architecture should expose something like:

```text
PgTransaction
      ↓
TenantTransaction
```

Tenant-scoped module repositories should receive the `TenantTransaction` abstraction instead of unrestricted direct database access.

The expected transactional order is:

```text
BEGIN
  ↓
set tenant context
  ↓
authorization
  ↓
domain mutation/query
  ↓
audit
  ↓
outbox
  ↓
COMMIT
```

---

## 12. Authentication architecture

Human sessions should use **opaque, revocable server-side session tokens**, not long-lived self-contained JWT sessions.

AWBMS requires immediate support for:

- explicit logout;
- administrator revocation;
- password-reset revocation;
- MFA assurance changes;
- MFA step-up;
- tenant membership changes;
- risk/session lifecycle controls.

Recommended token flow:

```text
cryptographically random token
          ↓
returned to client once
          ↓
hash stored server-side
          ↓
session metadata in PostgreSQL
```

JWT remains appropriate where required by external protocols such as OIDC.

### 12.1 Password hashing

**[D]** Use Argon2id for new AWBMS-native password storage unless a reviewed compatibility requirement dictates otherwise (`AD-12`).

During AWCMS migration, AWBMS should support the existing password format and opportunistically rehash on successful authentication rather than forcing all users to reset passwords.

> **[O] `OD-03`** — *which* legacy formats and credential shapes AWBMS must accept is undecided, and it cannot be decided from this document: it depends on what `VG-01` finds in the AWCMS implementation. The scope chosen determines how much compatibility surface [§36](#36-awcms-compatibility-architecture) carries and how large the parity corpus in [§39](#39-awcms--awbms-parity-harness) must be, so it blocks Blueprint sign-off.

### 12.2 Machine credentials

Machine credentials should be:

- tenant-bound;
- scope-limited;
- purpose-specific;
- optionally CIDR-bound for high-risk write credentials;
- expiry-bounded;
- revocable;
- raw secret shown once;
- stored only as secure verification material.

Compatibility with existing AWCMS machine-token forms belongs in the legacy compatibility adapter, not in the core domain model.

---

## 13. Authorization architecture

All protected operations should pass a single authorization service/chokepoint.

Recommended evaluation pipeline:

```text
principal/session
      ↓
assurance level
      ↓
tenant membership
      ↓
module enabled
      ↓
permission declaration
      ↓
RBAC
      ↓
ABAC
      ↓
business scope
      ↓
Separation of Duties
      ↓
resource ownership/domain constraint
      ↓
ALLOW / DENY
      ↓
decision audit
```

Rules:

- default deny;
- explicit deny overrides allow;
- missing policy data fails closed;
- sensitive decisions are logged;
- no scattered `is_admin` bypasses;
- platform-scoped actions cannot be silently seeded into tenant roles;
- assignment-time and action-time SoD checks should both exist where required.

That every protected operation actually passes this chokepoint — rather than most of them, with the exceptions undocumented — is gate `VG-07`.

---

## 14. Audit architecture

Audit logs and operational logs are distinct.

Audit events should capture security/business-significant actions with fields such as:

- actor/principal;
- tenant;
- action;
- resource type/id where allowed;
- allow/deny result;
- correlation/request ID;
- timestamp;
- relevant policy/control identifier;
- reason code;
- safe before/after metadata where required.

Sensitive secrets and unnecessary PII must be redacted or omitted.

Audit retention should be governed through explicit lifecycle rules and legal-hold controls.

---

## 15. Transactional events

Use a PostgreSQL transactional outbox as the default domain-event mechanism.

```text
business transaction
      │
      ├── domain mutation
      └── outbox INSERT
             │
           COMMIT
             │
             ▼
          worker
             │
          consumer
```

This provides atomicity between business state and event publication without introducing distributed transactions.

Required properties:

- explicit event names and versions;
- deterministic ordering key where ordering matters;
- bounded retry with backoff;
- dead-letter state;
- audited replay;
- idempotent consumers;
- event payload privacy review;
- observable event lag.

A message broker may be added later as an adapter when volume or isolation requirements justify it. Kafka/NATS/RabbitMQ are **not required dependencies for AWBMS v1**.

---

## 16. Job architecture

Do not make the foundational design dependent on a third-party Rust job scheduler abstraction.

Implement jobs around PostgreSQL semantics with explicit AWBMS contracts:

- registry;
- schedule;
- claim;
- lease;
- heartbeat where needed;
- retry/backoff;
- idempotency;
- dead-letter/failure state;
- run history;
- tenant context;
- correlation/trace context;
- observability.

Multiple workers should claim safely using transaction/locking semantics such as `FOR UPDATE SKIP LOCKED` where appropriate.

```text
worker A ─┐
worker B ─┼── PostgreSQL job queue
worker C ─┘
```

A given logical job must have a single runtime owner during migration; AWCMS and AWBMS workers must never concurrently consume the same production workload unless the protocol explicitly guarantees it.

---

## 17. External I/O rule

**[D]** External provider HTTP calls MUST NOT execute while a business database transaction is being held open, unless a formally reviewed algorithm requires it (`AD-15`).

Preferred design:

```text
BEGIN
business mutation
outbox/job enqueue
COMMIT
     ↓
worker
     ↓
provider HTTP
```

Benefits:

- shorter locks;
- less connection starvation;
- safe retries;
- provider outage isolation;
- better idempotency;
- easier dead-letter/replay handling.

---

## 18. Outbound HTTP/TLS

**[D]** Use Reqwest with rustls and explicit feature configuration (`AD-16`).

Do not depend implicitly on whichever TLS backend transitive Cargo features happen to select.

Outbound calls must define:

- connect timeout;
- request timeout;
- response/body limits;
- redirect policy;
- retry policy based on idempotency;
- circuit-breaker policy where justified;
- SSRF/DNS/IP restrictions for attacker-controlled destinations;
- correlation IDs;
- safe error normalization.

No provider error response should leak secrets into application logs or client-facing errors.

---

## 19. Object storage

Domain logic should depend on an AWBMS storage port rather than directly on Cloudflare-specific APIs.

The port design assumes the providers AWBMS depends on offer the capabilities it asks of them — presigned upload/download from R2, a deliverable email path, a push transport, and an OIDC provider where federation is required (`AS-07`). Each is plausible and none is confirmed.

Example interface responsibilities:

```text
put/upload
stat
get/download when appropriate
delete
presign upload/download
verify metadata/checksum
```

Adapters may provide:

```text
production → Cloudflare R2
local/LAN  → filesystem-compatible implementation
tests      → temporary/in-memory implementation
```

Media upload flows should retain security properties such as:

- size limits;
- MIME/magic-byte validation;
- checksum verification;
- object ownership;
- tenant isolation;
- controlled object keys;
- quarantine/scan capability where applicable;
- presigned URL expiry.

---

## 20. Cache architecture

Caching must never become a hidden source of authorization truth.

### 20.1 Initial recommendation

```text
PostgreSQL authoritative state
+
optional local Moka caches
```

### 20.2 Redis/Valkey

Introduce Redis/Valkey only when measured needs justify:

- distributed rate limiting;
- distributed ephemeral counters;
- shared cache;
- specialized coordination.

Do not make it the only source for:

- permissions;
- tenant membership;
- critical session state;
- workflow truth;
- audit;
- financial/accounting state.

AWBMS must remain correct after cache eviction.

---

## 21. API contracts

AWBMS should remain **contract-first**.

### 21.1 OpenAPI

Use reviewed YAML/JSON contracts under:

```text
contracts/openapi/
```

Recommended initial target: OpenAPI 3.1.x unless a documented compatibility need requires a different frozen contract.

Do not make Rust derive macros/annotations the sole source of API truth during the migration phase.

Required gates include:

- unique `operationId`;
- every implemented route is specified;
- every specified route is implemented or explicitly planned;
- authentication/security declaration consistency;
- standard error envelope;
- path/query/body validation consistency;
- consumer-contract coverage;
- backward compatibility checks for committed/consumed APIs.

Conformance between the specified and implemented route sets is gate `VG-08`; it is the check that keeps "contract-first" from degrading into "contract-alongside".

### 21.2 AsyncAPI

Use:

```text
contracts/asyncapi/
```

for event channels/messages and compatibility contracts.

Validate against the relevant AsyncAPI schema and add AWBMS-specific semantic checks for event ownership/versioning/order/retry/privacy.

---

## 22. Observability

AWBMS should use:

```text
structured logs
+
tracing
+
metrics
```

Recommended Rust foundation:

- `tracing`;
- `tracing-subscriber`;
- OpenTelemetry;
- OTLP exporter.

Every request/job/event should support correlation across boundaries.

Safe telemetry fields include, where appropriate:

```text
request_id
correlation_id
trace_id
tenant_id
module
operation
latency
result/error code
```

Raw passwords, access tokens, refresh tokens, MFA secrets, recovery codes, API keys and unnecessary PII must never be logged.

---

## 23. Performance expectations

> **[F] No performance data exists.** No AWBMS benchmark has been designed or run, and no AWCMS baseline measurement is recorded anywhere in this repository. This section states *expectations and a measurement plan only*. No number in it may be quoted as a result, and the Blueprint MUST NOT carry a performance commitment that is not backed by `VG-09` output. Comparative statements about AWCMS are meaningless until a baseline exists on identical infrastructure.

**[R]** Rust is expected to improve the platform's potential for:

- lower memory overhead;
- efficient CPU use;
- predictable latency;
- high concurrency;
- absence of garbage-collector pauses;
- efficient worker processes;
- compile-time memory/thread-safety guarantees.

However, AWBMS performance must not be estimated from HTTP hello-world benchmarks.

A typical business request is more likely to spend time in:

```text
routing + middleware
authentication
authorization
PostgreSQL query/locks
serialization
external integration
```

than in pure framework routing.

Therefore the project should optimize **end-to-end business flows**, not benchmark headlines.

This rests on an assumption about workload shape: that AWBMS requests are dominated by database and authorization work rather than by framework routing (`AS-04`). If a real workload turns out to be routing-bound or serialization-bound, the reasoning behind `AD-04` weakens considerably, because that reasoning explicitly discounts raw HTTP throughput.

**[O] `OD-06`** — no SLO target exists for any capability. [§49.2](#492-blueprint-contents) lists SLOs as Blueprint content, and `OD-04` cannot set benchmark acceptance thresholds without them: a threshold is meaningless until someone states what the system is required to achieve.

### 23.1 Required benchmark corpus

At minimum benchmark:

1. authenticated tenant-scoped GET;
2. protected RBAC/ABAC mutation;
3. transactional write + audit + outbox;
4. PostgreSQL full-text search;
5. event/job worker dispatch;
6. media metadata operations;
7. representative reporting queries.

Capture:

```text
requests/sec
p50
p95
p99
CPU/request
memory footprint
DB connections
DB wait
query time
lock wait
event lag
job throughput
error rate
```

Use identical infrastructure, dataset and workload when comparing AWCMS and AWBMS. A comparison that varies infrastructure alongside implementation measures nothing.

**[O] `OD-04`** — no acceptance threshold is defined for any metric above. "Benchmark improves a defined metric" ([§48](#48-performance-validation-gates)) is unenforceable until targets exist. The Blueprint MUST set, per benchmark scenario, an explicit SLO or a "baseline-only, no regression beyond X%" rule. Until then `VG-09` verifies only that measurements were *taken*, not that they were *acceptable*.

Correctness and security parity come before performance optimization; a faster implementation that fails `VG-05` or `VG-12` is not a candidate for cutover.

---

## 24. Horizontal scalability

`awbms-server` should be stateless enough for multiple instances:

```text
             load balancer
                 │
       ┌─────────┼─────────┐
       ▼         ▼         ▼
    server 1  server 2  server 3
       │         │         │
       └─────────┼─────────┘
                 ▼
             PostgreSQL
```

Critical shared state stays in PostgreSQL or explicitly designed infrastructure.

Workers scale similarly:

```text
worker 1
worker 2
worker 3
   │
atomic claim / SKIP LOCKED
   │
PostgreSQL
```

Do not introduce sticky sessions unless a specific feature requires them.

The single-region, single-primary topology sketched here assumes no availability requirement forces multi-region or active-active deployment in v1 (`AS-06`). That assumption is cheap to hold now and expensive to discover false later, because active-active would reopen `AD-05` — a single authoritative PostgreSQL primary is what most of this architecture's transactional guarantees rest on.

---

## 25. PostgreSQL connection scaling

Start with SQLx `PgPool` directly to PostgreSQL.

Do not add PgBouncer simply because it is available.

Add it only when measured connection patterns justify it.

Because AWBMS tenant context should use transaction-local configuration, the model can be compatible with transaction pooling, but prepared-statement behavior and pool semantics must be integration-tested before production activation.

---

## 26. Rust security posture

Rust eliminates broad classes of memory-safety vulnerabilities, but does not automatically prevent:

- broken access control;
- IDOR/BOLA;
- cross-tenant access;
- insecure business logic;
- SQL logic errors;
- SSRF;
- race conditions;
- replay attacks;
- privacy leaks;
- weak configuration;
- supply-chain compromise.

Security architecture remains mandatory.

### 26.1 Unsafe-code policy

AhliWeb-owned AWBMS crates should initially use:

```rust
#![forbid(unsafe_code)]
```

where practical.

This is binding as `AD-29`. Any future need for unsafe Rust requires:

- explicit ADR;
- narrow isolation;
- threat/security review;
- dedicated tests;
- fuzz/property testing where appropriate;
- documented justification.

---

## 27. Supply-chain security

Rust applications commonly depend on a substantial crate graph. Dependency governance is therefore a first-class security concern.

Required controls:

```text
Cargo.lock committed
cargo audit
cargo deny
cargo vet
license allow-list
source registry allow-list
SBOM generation
container scan
reviewed automated update PRs
```

Do not assume that using a memory-safe language makes third-party dependencies trustworthy.

Evidence that these controls run, rather than merely appear in a list, is gate `VG-13`.

---

## 28. Maintainability and code-quality gates

The baseline CI should include, as applicable:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo nextest run --workspace
cargo sqlx prepare --check
cargo audit
cargo deny check
cargo vet
```

plus project-specific checks for:

- module DAG;
- table ownership;
- route ownership;
- OpenAPI;
- AsyncAPI;
- migration integrity;
- PostgreSQL integration;
- tenant RLS;
- authorization;
- subject-data/privacy coverage;
- concurrency/idempotency;
- security regression;
- E2E flows;
- Docker image build;
- SBOM/container scan.

Recommended code policies:

- avoid `unwrap()` in request/job processing paths;
- use `expect()` only for documented impossible invariants/bootstrap conditions;
- environment reads centralized in typed configuration;
- SQL limited to owned repositories/modules;
- no direct cross-module writes;
- no external provider I/O inside ordinary business DB transactions;
- no blocking I/O on Tokio executor threads;
- typed domain identifiers rather than interchangeable raw strings/UUIDs where practical.

---

## 29. Error model

AWBMS should separate:

1. domain errors;
2. authorization/security errors;
3. validation errors;
4. infrastructure/provider errors;
5. internal unexpected errors.

Client errors should use stable codes and safe messages.

Internal errors should retain correlation information without exposing:

- SQL internals;
- stack traces;
- credentials;
- provider secrets;
- private network locations;
- sensitive object metadata.

**[D]** A standard `ApiError` envelope and a stable client-facing error code taxonomy MUST be defined before module implementation begins (`AD-21`). Defining it afterwards means retrofitting every handler already written, and error shape is a published contract under `AD-19` whether or not anyone intended it to be.

---

## 30. Backpressure and overload behavior

AWBMS should retain the AWCMS lesson that database saturation must be an explicit state.

Design controls for:

- bounded request bodies;
- request concurrency limits;
- work classes;
- database pool limits;
- query timeouts;
- worker concurrency;
- queue depth;
- external-provider concurrency;
- circuit breakers;
- load shedding.

Where overload occurs, return predictable retryable responses rather than allowing unconstrained resource exhaustion.

---

## 31. Idempotency

**[D]** Idempotency is a reusable platform capability, not a per-module concern, and applies to high-risk or retry-prone mutations (`AD-23`). Evidence is gate `VG-11`.

Example:

```text
POST invoice
POST invoice retry
POST invoice retry
```

must resolve to:

```text
one committed business mutation
one invoice identity
one expected event sequence
```

not duplicate financial/business state.

The algorithm should define:

- idempotency-key scope;
- principal/tenant binding;
- request fingerprint;
- in-flight conflict handling;
- response replay semantics;
- expiry/retention;
- transaction behavior.

---

## 32. Abuse resistance

Rate limiting must target the resource being abused, not only the requester identity/IP.

**[C]** `C-07` — AWCMS newsletter hardening is the motivating example: per-IP limits alone could not prevent repeated email delivery to a victim address, so recipient-oriented cooldown semantics were required. (Recorded as reported by the original validation; the AWCMS change is not identified by commit or date here, so `VG-01` MUST capture the resulting rate-limit dimensions from the AWCMS implementation rather than from this paragraph. The word "recent" was removed as unmaintainable — it silently ages.)

AWBMS rate/abuse controls may need dimensions such as:

```text
IP
principal
tenant
recipient
resource
operation
provider account
```

Neutral/non-enumerating responses should be used where endpoint behavior could expose subscriber/user/resource existence.

---

## 33. Data lifecycle and privacy

Privacy must be designed into module admission.

Each data-owning module should declare:

- data categories;
- tenant/global scope;
- PII/sensitive classification;
- processing purpose;
- source;
- retention;
- archive rules;
- erasure/anonymization behavior;
- legal-hold behavior;
- data-subject export behavior;
- audit implications.

A central data-lifecycle engine can coordinate policy, but the owning module remains responsible for domain-correct purge/anonymization semantics.

Legal hold must be non-bypassable by ordinary purge jobs.

Coverage — every data-owning module having declared these descriptors, and the declarations matching actual behaviour — is gate `VG-14`.

---

## 34. Open source and licensing governance

Every runtime/build dependency should be subject to:

- license policy;
- maintenance activity review;
- security advisory review;
- transitive dependency review;
- source registry verification;
- replacement feasibility.

Core architecture should prefer well-supported ecosystem components over niche crates when the feature can be implemented safely with standard Tokio/Tower/SQLx primitives.

**[O] `OD-08`** — neither AWBMS's own licence nor its dependency licence allow-list has been decided. The first is a commercial and ownership question outside this document's scope but inside somebody's; the second is a prerequisite for `cargo-deny` and therefore for step 2 of [§51](#51-initial-implementation-sequence-after-definition-of-ready). Until AWBMS carries a licence, default copyright applies and no third party has any right to use it — which is a decision by default, not an absence of one.

---

## 35. Standards and regulatory mapping

AWBMS should map controls to applicable standards without claiming compliance before evidence exists.

Relevant standards include:

- ISO/IEC 27001:2022;
- ISO/IEC 27002;
- ISO/IEC 27005;
- ISO/IEC 27017;
- ISO/IEC 27018;
- ISO/IEC 27034;
- ISO/IEC 27701;
- ISO/IEC 20000-1;
- ISO 22301;
- ISO/IEC 15408 concepts where applicable;
- ISO/IEC 25010;
- OWASP Top 10;
- OWASP ASVS;
- OWASP API Security Top 10;
- NIST CSF;
- NIST SSDF;
- CIS Controls;
- WCAG for relevant user-facing surfaces.

Relevant Indonesian regulatory baseline includes:

- UU No. 27 Tahun 2022 tentang Pelindungan Data Pribadi;
- PP No. 71 Tahun 2019 tentang Penyelenggaraan Sistem dan Transaksi Elektronik;
- UU No. 1 Tahun 2024 as the second amendment to the ITE law;
- additional sector-specific regulations where AWBMS is used.

Compliance claims require deployment, operational and organizational evidence in addition to software controls.

---

## 36. AWCMS compatibility architecture

Compatibility code must be isolated from the AWBMS-native domain.

Recommended boundary:

```text
legacy AWCMS HTTP/contracts
          ↓
   compat-awcms adapter
          ↓
 AWBMS application/domain
```

The compatibility layer may support:

- legacy headers;
- legacy cookie names;
- legacy machine credentials;
- response field shapes;
- legacy error codes;
- old identifiers;
- API version translation.

It must not become the place where new domain rules are implemented.

Once migration and compatibility windows are complete, it can be deprecated independently.

---

## 37. Database migration strategy

Because AWBMS is a separate product, more than one migration mode is defensible. This section describes three; [§37.3](#373-preferred-long-term-approach) is preferred.

> **[O] `OD-01` — the choice is open and it is consequential.** The three modes below are *not* interchangeable: §37.1 forces schema-name compatibility and expand-contract coexistence ([§40.1](#401-expand-contract-rule)) onto every migration, §37.2 forgoes coexistence on one database entirely, and §37.3 requires a reconciliation pipeline that neither of the others needs. Sequencing, rollback design ([§42](#42-rollback-model)) and the parity harness scope ([§39](#39-awcms--awbms-parity-harness)) all depend on which is chosen. `OD-01` MUST be closed in the Master Blueprint, per deployment, with the choice and its rationale recorded as a successor decision to `AD-27`. Do not begin migration engineering while all three remain open.

### 37.1 Existing AWCMS production database takeover

If AWBMS temporarily operates directly on an existing AWCMS database, preserve existing `awcms_*` schema names and migration ledger semantics until the migration is complete.

Do not combine runtime-language migration with mass schema renaming.

### 37.2 New AWBMS-native deployment

A new AWBMS database may use native names such as:

```text
awbms_*
```

from the beginning.

### 37.3 Preferred long-term approach

For a truly independent product, prefer deterministic migration into an AWBMS-native schema:

```text
AWCMS database
      ↓
versioned export/migration
      ↓
validation/reconciliation
      ↓
AWBMS database
```

Migration must be:

- deterministic;
- resumable;
- idempotent where practical;
- checksum/version controlled;
- reconciliation-driven;
- rollback-aware;
- tested against production-like snapshots.

No destructive transformation should be performed without a verified recovery point.

All three modes assume the AWBMS team can obtain what migration engineering needs: read access to the AWCMS repository for `VG-01`, and production-like database snapshots to test against (`AS-03`). Neither has been confirmed available, and `VG-01` cannot be discharged without the first.

---

## 38. Migration ledger

AWBMS-native migrations should preserve strong migration properties already proven useful in AWCMS:

- ordered migrations;
- immutable applied migration contents;
- SHA-256 checksum verification;
- cross-process migration lock;
- one controlled transaction per migration where PostgreSQL permits;
- explicit migration history table;
- startup/deploy checks for schema version.

**[D]** These properties are binding (`AD-25`). Do not rely solely on a migration framework default if it weakens them — verify against gate `VG-10` instead, which checks applied-migration immutability by checksum rather than by convention.

---

## 39. AWCMS ↔ AWBMS parity harness

Build a first-class golden/parity test system.

For compatible behavior:

```text
same request corpus
       │
       ├── AWCMS against isolated DB A
       └── AWBMS against isolated DB B
                   │
                   ▼
             normalize safe
          nondeterministic fields
                   │
                   ▼
               compare
```

Compare as relevant:

- HTTP status;
- headers;
- response shape;
- error codes;
- database state delta;
- audit events;
- domain events;
- job enqueueing;
- idempotency behavior;
- authorization decision;
- side-effect intent.

Normalize only fields legitimately nondeterministic, for example:

- generated UUIDs;
- timestamps;
- random tokens;
- request IDs;
- cryptographic nonces.

The harness must not hide semantic differences.

**[D]** The parity harness is a first-class deliverable, not a testing convenience (`AD-26`). Its scope depends on `OD-03`, and the fixtures it runs against come from `VG-01`; it cannot be built before either.

---

## 40. Coexistence and cutover

Use a strangler-style migration rather than one massive replacement.

Rules:

1. only one implementation owns production writes for a capability at a time;
2. read traffic may be shadowed to AWBMS and compared;
3. production write traffic must not be blindly mirrored into the same database;
4. captured write requests may be sanitized/replayed into isolated database clones for comparison;
5. worker ownership transfers explicitly and atomically;
6. database changes must remain compatible with both implementations during coexistence if both share a schema.

### 40.1 Expand-contract rule

When AWCMS and AWBMS coexist on one schema:

```text
expand schema
    ↓
make both implementations compatible
    ↓
migrate data/behavior
    ↓
retire old implementation
    ↓
contract/remove obsolete schema later
```

Never drop a field/table that the old implementation may still require.

---

## 41. Worker cutover safety

The incumbent AWCMS workers and the new Rust/AWBMS workers MUST never concurrently consume the same production queue or job. (AWCMS is assumed to run on a TypeScript/Bun runtime — `AS-01`; the cutover protocol below does not depend on which runtime it is, only on there being exactly one owner at a time.)

For each worker class:

```text
identify current owner
      ↓
stop new claims
      ↓
drain in-flight work
      ↓
activate AWBMS owner
      ↓
observe
      ↓
accept or rollback
```

Apply this to:

- email;
- push;
- events;
- reporting;
- retention;
- analytics;
- search reconciliation;
- media jobs;
- domain sync/integration workloads.

The protocol assumes a single operational team controls both implementations during coexistence, so that ownership transfer can be made atomic by agreement rather than by mechanism (`AS-08`). If AWCMS and AWBMS were operated by separate teams, "never concurrently consume" would need enforcement in the queue itself, not in a runbook.

---

## 42. Rollback model

Every production migration wave must define rollback before deployment.

Minimum rollback preparation:

- recovery point/backup;
- database compatibility status;
- previous deployable AWCMS artifact where applicable;
- ability to stop AWBMS workers;
- ability to restore traffic routing;
- event/job ownership status;
- post-rollback verification checklist.

**[D]** Rollback is defined and rehearsed before a wave deploys, never designed during an incident (`AD-28`). Rollback is not merely “redeploy old image” if database state or queue ownership has changed; the evidence that a rollback actually works is gate `VG-12`.

---

## 43. Deployment architecture

Recommended baseline:

```text
Cloudflare
   ↓
Traefik / Coolify routing
   ↓
AWBMS container(s)
   ↓
PostgreSQL
```

Production binaries:

```text
awbms-server
awbms-worker
awbms-cli
```

Images should use multi-stage builds and minimal runtime contents.

This baseline assumes the Docker/Coolify/Traefik/Cloudflare stack is available to AWBMS and operable by whoever runs it (`AS-02`). **[O] `OD-05`** — how many deployments exist, and whether tenants are separated by deployment or only by tenant ID within one, is undecided. It determines what "tenant isolation" has to mean operationally as well as in SQL, so it blocks Blueprint sign-off.

Runtime container principles:

- non-root user;
- read-only filesystem where practical;
- no compiler/toolchain in final image;
- secrets injected at runtime;
- health/readiness endpoints;
- bounded resources;
- graceful shutdown;
- SBOM/attestation where supported;
- immutable release tag/digest promotion.

---

## 44. Health and readiness

Separate:

- liveness: process/event loop is alive;
- readiness: instance may safely receive traffic;
- dependency health: database/providers/queues;
- module health: critical module capability state.

Do not make liveness fail merely because an external optional provider is unavailable; otherwise orchestration can amplify provider outages by restart loops.

---

## 45. Security examples

### 45.1 Cross-tenant object access

Attempt:

```text
tenant A principal
→ request tenant B resource
```

Expected controls:

```text
authorization deny
+
explicit tenant predicate
+
FORCE RLS
+
audit/decision log
```

### 45.2 Provider timeout

Preferred behavior:

```text
BEGIN
create/update business record
enqueue outbox job
COMMIT
   ↓
worker calls provider
```

Provider outage does not hold unrelated DB locks open.

### 45.3 Duplicate high-risk mutation

```text
POST
network timeout
client retries POST
```

Expected result:

```text
one business transaction
one stable result
```

### 45.4 Storage outage

R2 outage after business commit should produce retryable queued work rather than corrupting unrelated domain state.

### 45.5 Enumeration-sensitive endpoint

Login/reset/newsletter-like endpoints should return neutral responses when distinguishable outcomes would leak account/subscriber/resource existence.

---

## 46. Testing strategy

Required test layers include:

### Unit/domain

- pure domain invariants;
- validation;
- state transitions;
- policy calculations.

### PostgreSQL integration

- real PostgreSQL;
- migrations;
- transactions;
- indexes/constraints;
- RLS;
- role privileges;
- concurrency locks (`VG-06` for the RLS and role-privilege evidence specifically).

### Authorization

- RBAC;
- ABAC;
- SoD;
- tenant isolation;
- module enablement;
- ownership;
- deny-overrides.

### Contract

- OpenAPI;
- AsyncAPI;
- AWCMS compatibility;
- consumer contracts.

### Concurrency/idempotency

- duplicate requests;
- parallel updates;
- job claims;
- event dispatch;
- lock conflicts.

### Security regression

Every significant threat/control should map to an automated security test where possible.

### E2E/UAT

Validate actual workflows, not only API primitives.

---

## 47. Property testing, concurrency modelling and fuzzing

Use advanced Rust testing where it adds concrete value:

- `proptest` for bounded domain/property tests;
- `loom` for selected concurrency primitives if custom synchronization is introduced;
- `cargo-fuzz` for parsers, token formats, complex protocol inputs and other high-risk surfaces.

Do not fuzz ordinary CRUD merely for coverage metrics; target parsers and trust boundaries.

---

## 48. Performance validation gates

A performance improvement is accepted only if:

1. correctness tests remain green;
2. authorization/RLS behavior is unchanged or intentionally revised;
3. data integrity is preserved;
4. representative benchmark improves a defined metric;
5. no unacceptable CPU/memory/latency regression appears elsewhere.

Optimize SQL/indexes and transaction boundaries before introducing architectural distribution.

---

## 49. Master Blueprint entry conditions

The architecture is considered sufficiently validated to enter Stage 1 — Master Blueprint — **conditionally**. The conditions are stated here rather than left implicit, because "validated" in the original document was not distinguishable from "approved without preconditions".

### 49.1 Entry conditions

Stage 1 may **begin** immediately: Blueprint authoring is itself the work that closes most open decisions.

Stage 1 may **not complete** — the Blueprint may not be signed off — until:

| # | Condition | Reference |
|---|---|---|
| 1 | `VG-01` discharged: the frozen, machine-generated AWCMS inventory exists in `contracts/legacy/awcms/`, superseding claims `C-01`–`C-05` and `C-07` | [§2.2](#22-required-awcms-source-inventory-vg-01) |
| 2 | Every Blueprint-blocking open decision in [Appendix D](#appendix-d--open-decision-register) is closed and recorded as a decision | [Appendix D](#appendix-d--open-decision-register) |
| 3 | Every assumption in [Appendix B](#appendix-b--assumption-register) is confirmed, or the dependent decision is revisited | [Appendix B](#appendix-b--assumption-register) |
| 4 | `VG-03` and `VG-04` discharged: toolchain resolved and pinned; the `AD-04` / `AD-06` sensitivity checks completed | [§4.2](#42-runtime-and-upgrade-policy), [§6.2](#62-decision-matrix) |
| 5 | `C-06` removed rather than carried forward; versions live only in build manifests | [§4.1](#41-recommended-baseline) |
| 6 | AWBMS v1 module scope defined — `OD-09`; the 24 AWCMS modules of `C-02` are an inventory, not a v1 commitment | [§2](#2-source-of-truth-review) |

If condition 1 falsifies a material claim — for example if the AWCMS gates of `C-04`/`C-05` prove advisory rather than enforced — this validation MUST be revised before the Blueprint proceeds, not patched inside it.

### 49.2 Blueprint contents

The Blueprint must formalize:

- AWBMS product mission and non-goals;
- system context;
- Rust/toolchain ADRs;
- modular monolith boundaries;
- module admission rules;
- database/tenant architecture;
- auth/authz;
- audit;
- events;
- jobs;
- storage/integrations;
- API/event contracts;
- AWCMS compatibility strategy;
- migration/cutover/rollback;
- deployment;
- observability;
- SLOs;
- security/privacy controls;
- standards mapping;
- implementation roadmap.

---

## 50. Mandatory engineering lifecycle

AWBMS is a new platform and must follow the complete lifecycle:

1. Master Blueprint
2. PRD
3. Initial Threat Model + Privacy Analysis
4. ERD + Data Dictionary
5. RBAC + ABAC + RLS Matrix
6. Domain / Verification Algorithm Specification
7. OpenAPI + AsyncAPI
8. UX/UI
9. Cross-Spec Review / Definition of Ready
10. Atomic GitHub Issues
11. Implementation + Automated Tests
12. PR + Review + CI
13. Deploy Staging
14. Internal Human Testing / UAT
15. Release Readiness / Go-No-Go
16. Deploy Production
17. Production Validation
18. Monitoring + Post-Release Review

Then continuous improvement loops findings back into the affected specifications, tests and controls.

---

## 51. Initial implementation sequence after Definition of Ready

A recommended execution order is:

1. Bootstrap Cargo workspace and pinned toolchain.
2. Establish formatting/lint/test/dependency-security CI (`VG-02`; requires `OD-02` and `OD-08` closed).
3. Implement typed configuration and fail-fast validation.
4. Implement error model/API envelope.
5. Implement structured tracing/correlation.
6. Implement graceful HTTP server lifecycle.
7. Implement PostgreSQL pool/work classes.
8. Implement tenant transaction/RLS wrapper.
9. Implement migration runner/checksum/locking.
10. Implement explicit module registry.
11. Implement module DAG/capability gates.
12. Implement route/table/job ownership gates.
13. Implement audit foundation.
14. Implement authentication/session compatibility.
15. Implement central authorization chokepoint.
16. Implement RBAC.
17. Implement dynamic/bounded ABAC.
18. Implement SoD.
19. Implement idempotency.
20. Implement transactional event outbox.
21. Implement PostgreSQL job worker.
22. Implement object-storage and integration ports.
23. Build AWCMS parity harness.
24. Port/implement modules vertically in dependency order.
25. Validate every module against contract/security/RLS/parity tests.
26. Benchmark representative flows.
27. Deploy production-like staging.
28. Run UAT and failure/recovery testing.
29. Execute controlled production migration waves.
30. Complete production validation and observability review.

---

## 52. Vertical module implementation

Do not implement all models first, then all repositories, then all handlers.

Implement a complete vertical slice:

```text
HTTP request
   ↓
validation
   ↓
authentication
   ↓
authorization
   ↓
domain logic
   ↓
SQL transaction
   ↓
audit/outbox
   ↓
response
   ↓
contract/security tests
```

This exposes integration and security mistakes early.

A module is not “ported” merely because its tables and endpoints compile.

---

## 53. Module Definition of Done

A migrated or new module is complete only when applicable requirements pass:

- route/API behavior;
- OpenAPI contract;
- authentication;
- RBAC/ABAC/SoD;
- RLS;
- database constraints;
- table ownership;
- audit;
- idempotency;
- events;
- jobs;
- privacy/lifecycle descriptors;
- concurrency/failure behavior;
- migrations;
- AWCMS compatibility/parity where required;
- observability;
- documentation;
- E2E/UAT evidence;
- rollback procedure.

---

## 54. Go/No-Go criteria

Production cutover is **No-Go** when any of the following remains unresolved:

- tenant-isolation failure;
- authentication/session incompatibility for migration scope;
- privilege escalation;
- destructive migration uncertainty;
- failed backup/restore test;
- non-idempotent high-risk operation;
- event/job double-processing risk;
- blocking contract regression;
- critical security regression;
- rollback path unverified;
- required monitoring absent.

Go requires:

- CI green;
- migration tests green;
- RLS/authz tests green;
- compatibility tests green for committed scope;
- staging/UAT pass;
- security review pass;
- recovery point and rollback runbook ready;
- production configuration/secrets validated;
- monitoring/alerting ready.

---

## 55. Production validation

After every migration wave validate:

- login/session/MFA flows;
- tenant isolation, including malicious cross-tenant tests;
- permission/ABAC decisions;
- critical writes;
- audit records;
- domain events;
- job processing;
- external integration health;
- backup/recovery signals;
- API consumers;
- error/latency rates;
- database pool/lock health.

Do not mark a wave accepted solely because HTTP health returns 200.

---

## 56. Monitoring baseline

Monitor at least:

### Application

- availability;
- request latency;
- 4xx/5xx anomalies;
- panic count;
- rate-limit/load-shed counts;
- active/in-flight requests.

### PostgreSQL

- connections/pool wait;
- query latency;
- lock wait/deadlocks;
- transaction duration;
- RLS/permission errors;
- replication/backup health as applicable.

### Authorization/security

- authentication failures;
- MFA/step-up anomalies;
- ABAC denies;
- cross-tenant denial attempts;
- privileged operations;
- credential/session revocations.

### Events/jobs

- queue depth;
- lag;
- retry rate;
- dead letters;
- replay operations;
- worker throughput.

### Rust runtime/application

- panics;
- blocking-task pressure;
- Tokio task/resource symptoms;
- memory growth;
- CPU saturation;
- open connections.

### Business

- successful critical workflows;
- provider failures;
- publication/payment/order/etc. KPIs according to installed modules.

---

## 57. Architecture decision summary

The authoritative summary is the **[decision register in Appendix C](#appendix-c--decision-register)**, which carries, for each decision, its status, rationale, consequences, governing section and verification gate.

This section previously restated those decisions as a second table. That duplicate has been removed rather than maintained: two summaries of the same decision set drift apart, and [§2.1](#21-documentation-drift-warning) is the document's own argument against exactly that. Consult Appendix C, and [§1](#1-executive-decision) for the one-screen view.

---

## 58. Final recommendation

**[R]** The proposed Rust backend is internally coherent and, on the reasoning recorded in [§5](#5-http-framework-decision-axum), [§6](#6-database-access-decision-sqlx) and [§8](#8-modular-monolith-decision), appropriate for AWBMS.

Stated precisely, so that the strength of this conclusion is not overread: the architecture is **sound as a design**, and **unvalidated as an implementation**. No AWBMS code, test, benchmark or deployment exists ([§0.1](#01-repository-state-verified)), and the AWCMS baseline it is compatible with has not yet been machine-inventoried (`VG-01`). This is the appropriate confidence level for a pre-Blueprint architecture validation; it is not a technical readiness statement.

The preferred architecture, as one picture:

```text
AWBMS
│
├── Rust 2024
├── Axum + Tower
├── Tokio
├── PostgreSQL + SQLx
│
├── Modular Monolith
│   ├── explicit module registry
│   ├── vertical domain boundaries
│   ├── one-table-one-writer ownership
│   └── microservice-extractable seams
│
├── Security
│   ├── opaque sessions
│   ├── RBAC
│   ├── ABAC
│   ├── SoD
│   ├── FORCE RLS
│   ├── audit
│   └── default deny
│
├── Reliability
│   ├── idempotency
│   ├── transactional outbox
│   ├── PostgreSQL job queue
│   ├── retries/backoff/dead-letter
│   ├── backpressure
│   └── recovery/rollback
│
├── Contracts
│   ├── OpenAPI
│   ├── AsyncAPI
│   └── frozen AWCMS compatibility fixtures
│
└── Runtime binaries
    ├── awbms-server
    ├── awbms-worker
    └── awbms-cli
```

The project is therefore **approved to proceed to Stage 1: AWBMS Master Blueprint**, subject to re-validating upstream AWCMS/AWCMS-Astro state at the start of each major migration/specification stage.

---

> **How to read the registers.** Up to v0.1.0 these appendices described what the registers *should* contain rather than containing it — which is the drift pattern [§2.1](#21-documentation-drift-warning) argues against, applied by this document to itself. They are now tables. Every identifier used anywhere in this document resolves to exactly one row below, and `scripts/check-docs.sh` fails the build if it does not, or if a row here has outlived every reference to it (`VG-16`).
>
> **Owner fields read *unassigned* throughout.** That is accurate, not an oversight: no owner has accepted this validation ([Appendix F](#appendix-f--approval-record)). Assigning owners is the first Blueprint task.

## Appendix A — Recorded external claims

Claims inherited from the original validation, observed against repositories **outside** this one and therefore not verifiable from it.

**Four of the seven were confirmed at v0.3.0** by the `VG-01` freeze, which pinned `ahliweb/awcms` at `11f2e95a…` and `ahliweb/awcms-astro` at `7b753be6…` and generated machine-readable inventories under `contracts/legacy/awcms/frozen/`. Verification date, verifier and evidence digest live in the inventory artifacts, not here — the inventory is the evidence, and a second copy of its metadata would be one more thing to drift.

The confirmations are worth noting precisely because the document expected the opposite. `C-03` was flagged as the weakest claim in this register, sourced from prose that [§2.1](#21-documentation-drift-warning) says is known to lag implementation; re-derived from the migration ledger it came back exactly right at 148. Being wrong about which claims would fail is a better outcome than not having checked.

| Claim | Source | Observation | Gate | Disposition |
|---|---|---|---|---|
| `C-01` | `ahliweb/awcms` v10.1.0 @ `11f2e95a47b1328a820f976d60f978c38a067903`, 2026-08-28; `ahliweb/awcms-astro` @ `7b753be619244541b817d5d8e7d3b72cfe88d4f9` | The AWCMS state the original validation reviewed | `VG-01` | **Confirmed** (v0.3.0). The freeze pinned the same `awcms` commit, and **the recorded defect is closed**: the missing `awcms-astro` commit — which the consumer contract surface in [§39](#39-awcms--awbms-parity-harness) depends on — is now recorded in `frozen/provenance.json` |
| `C-02` | AWCMS module registry source | AWCMS contains 24 registered modules, listed in [§2](#2-source-of-truth-review) | `VG-01` | **Confirmed** (v0.3.0). `frozen/static/modules.json` contains exactly 24. Still an inventory, **not** an AWBMS v1 scope commitment — see `OD-09` |
| `C-03` | AWCMS *architecture prose* | Migrations through `sql/148`; FORCE RLS on tenant-scoped tables; separated database roles; module composition rules; OpenAPI/AsyncAPI contracts; audit/event systems; SYSTEM administration surface | `VG-01` | **Confirmed on the checkable parts** (v0.3.0), against expectation. Re-derived from the ledger: **148** migrations; **134** tables with RLS, all 134 `FORCE`; roles and grants captured in `frozen/live/`. The capability descriptions remain prose and are not individually verified |
| `C-04` | AWCMS CI configuration and gate scripts | AWCMS enforces machine-checked architectural gates, listed in [§2](#2-source-of-truth-review) | `VG-01` | **Unverified.** `VG-01` MUST record each listed gate as *enforced in CI*, *advisory*, or *absent* |
| `C-05` | Original validation narrative | Those gates were created in response to production or review failures, and therefore encode invariants a feature list does not capture | `VG-01` | **Unverified, and strategically load-bearing.** `C-05` is why AWBMS treats AWCMS as a requirements source rather than a codebase to translate (`AD-01`). If the gates prove aspirational, the AWBMS gate set in [§28](#28-maintainability-and-code-quality-gates) must be re-derived from threat modelling instead of inheritance |
| `C-06` | Original validation, dated 2026-08-20 | Rust stable `1.98.0` released; Tokio, Axum, SQLx, Tower, Reqwest, Serde, rustls and OpenTelemetry production-ready at that date | `VG-03` | **Superseded, which is the correct outcome — not "confirmed".** `rust-toolchain.toml` now pins the toolchain and `Cargo.lock` resolves the graph, so the build manifests are the source of truth and this claim has no further authority. [§49.1](#491-entry-conditions) condition 5 is satisfied by the pin existing, not by the version matching |
| `C-07` | AWCMS newsletter hardening, **no commit or date recorded** | Per-IP rate limits alone could not prevent repeated email delivery to a victim address; recipient-oriented cooldown semantics were required | `VG-01` | **Unverified.** `VG-01` MUST capture the actual rate-limit dimensions from the implementation rather than from [§32](#32-abuse-resistance) |

## Appendix B — Assumption register

Assumptions are relied upon but not established. Carrying one into implementation silently is how a design acquires a failure mode nobody chose. Each row states what breaks if the assumption is false — that column, not the assumption text, is the reason the register exists.

| ID | Assumption | Relied on by | If falsified | Validation method | Owner |
|---|---|---|---|---|---|
| `AS-01` | AWCMS runs on a TypeScript/Bun runtime | [§41](#41-worker-cutover-safety) | Little. The cutover protocol depends only on there being exactly one owner at a time, not on which runtime | Observe the AWCMS deployment | *unassigned* |
| `AS-02` | The Docker/Coolify/Traefik/Cloudflare stack is available to AWBMS and operable by whoever runs it | `AD-30`, [§43](#43-deployment-architecture) | `AD-30` is re-opened; container, routing and edge selections are re-made | Confirm with whoever operates the infrastructure | *unassigned* |
| `AS-03` | The AWBMS team can obtain AWCMS repository read access and production-like database snapshots | [§2.2](#22-required-awcms-source-inventory-vg-01), [§37.3](#373-preferred-long-term-approach) | `VG-01` cannot be discharged, so [§49.1](#491-entry-conditions) condition 1 cannot be met and the Blueprint cannot be signed off | Request access; record the outcome | *unassigned* |
| `AS-04` | AWBMS requests are dominated by database and authorization work, not framework routing | [§23](#23-performance-expectations), and the reasoning behind `AD-04` | `AD-04`'s rationale weakens materially — it explicitly discounts raw HTTP throughput on the strength of this assumption | Benchmark corpus, [§23.1](#231-required-benchmark-corpus) | *unassigned* |
| `AS-05` | The tenant is the isolation unit; no sub-tenant partition or deliberate cross-tenant sharing is required | `AD-10`, [§11.2](#112-tenant-security) | The RLS policy model changes shape rather than merely extending; `VG-06` scope grows | Product requirements, Blueprint | *unassigned* |
| `AS-06` | No availability requirement forces multi-region or active-active deployment in v1 | [§24](#24-horizontal-scalability), and indirectly `AD-05` | Reopens `AD-05`. A single authoritative primary is what most transactional guarantees here rest on | SLO definition, `OD-06` | *unassigned* |
| `AS-07` | Providers offer the capabilities the ports assume — R2 presigning, a deliverable email path, a push transport, an OIDC provider where federation is required | `AD-17`, [§19](#19-object-storage) | Individual adapters are redesigned; the port abstraction itself survives, which is the point of having it | Provider documentation and a spike per adapter | *unassigned* |
| `AS-08` | One operational team controls both implementations during coexistence, so ownership transfer can be atomic by agreement | `AD-27`, [§41](#41-worker-cutover-safety) | "Never concurrently consume" needs enforcement in the queue itself, not in a runbook | Organisational confirmation | *unassigned* |

## Appendix C — Decision register

The authoritative decision list. `AD-01`–`AD-30` were made by the original validation; `AD-31` and `AD-32` were added in v0.2.0 and cover this repository's own governance.

**Status is `Accepted` for every row.** No decision here has been superseded. A reversed decision is never edited in place — it receives a successor row, and the original's status becomes `Superseded by AD-nn`, so the reasoning history stays readable.

**No decision below is implementation-ready.** Each needs alternatives, consequences, an owner and a date recorded in the Master Blueprint or a linked ADR before code depends on it. The `Gate` column names the evidence that would move it from *reasoned* to *validated*; none has been produced.

| ID | Decision | Rationale and principal consequence | § | Gate |
|---|---|---|---|---|
| `AD-01` | AWBMS is a new Rust-native platform, not a source translation of AWCMS | AWCMS invariants are valuable; AWCMS code is not portable. Consequence: requirements must be extracted, which costs `VG-01` before migration work can begin | [§1](#1-executive-decision), [§2](#2-source-of-truth-review) | `VG-01` |
| `AD-02` | Rust 2024 Edition on a pinned stable toolchain | Pinning makes builds reproducible and upgrades deliberate. Consequence: someone must own an upgrade cadence — `OD-02` | [§4.1](#41-recommended-baseline) | `VG-03` |
| `AD-03` | Tokio as the async runtime | The ecosystem's centre of gravity; Axum, Tower, Hyper, SQLx and Reqwest all assume it. Consequence: blocking I/O on executor threads becomes a defect class needing its own lint | [§4.1](#41-recommended-baseline), [§7](#7-recommended-macro-architecture) | `VG-04` |
| `AD-04` | Axum + Tower + Tower HTTP as the HTTP foundation | Shared Tower/Hyper/Tokio service ecosystem for tracing, timeouts, concurrency limits, load shedding and future gRPC. Not chosen on throughput — Actix was not disqualified. Consequence: the choice rests on architectural fit and `AS-04`, neither measured | [§5](#5-http-framework-decision-axum) | `VG-04` |
| `AD-05` | PostgreSQL is the authoritative system of record | Changing runtime language does not justify changing the database. Consequence: RLS, advisory locks, outbox and job queues all rest on one primary — see `AS-06` | [§11](#11-postgresql-architecture) | `VG-06` |
| `AD-06` | SQLx as the primary PostgreSQL access layer | AWBMS depends on database behaviour that is part of the security architecture and must stay visible: RLS, `SET LOCAL`, advisory locks, CTEs, `SKIP LOCKED`. Consequence: no ORM convenience; more hand-written SQL to review | [§6](#6-database-access-decision-sqlx) | `VG-04` |
| `AD-07` | Modular monolith, microservice-extractable | The real bottlenecks — query design, lock contention, connection capacity, authorization complexity — are not solved by network boundaries. Consequence: module seams must be maintained by gates, since nothing physical enforces them | [§8](#8-modular-monolith-decision) | `VG-02` |
| `AD-08` | Explicit module registry, reviewed at compile time | Runtime plugin discovery makes the module set a deployment-time property, which defeats every gate: a DAG known only at startup cannot be checked in CI. Consequence: adding a module is a code change, by design | [§10](#10-module-composition-model) | `VG-02` |
| `AD-09` | One mutable table has exactly one authoritative module owner | Shared write access is how module boundaries quietly stop existing. Consequence: cross-domain writes need ports, capabilities, read models or events — more indirection, deliberately | [§10.1](#101-one-table-one-writer-principle) | `VG-02` |
| `AD-10` | RBAC + ABAC + Separation of Duties + PostgreSQL FORCE RLS | Defence in depth: application authorization, explicit tenant predicate, and RLS as backstop. Consequence: three layers to keep consistent; RLS is never an excuse to omit the predicate | [§11.2](#112-tenant-security), [§13](#13-authorization-architecture) | `VG-06`, `VG-07` |
| `AD-11` | Opaque, revocable server-side sessions for human users | Logout, admin revocation, password reset, MFA step-up and membership changes must take effect immediately, which self-contained JWTs cannot do. Consequence: a session lookup per request. JWT stays appropriate for OIDC | [§12](#12-authentication-architecture) | `VG-05` |
| `AD-12` | Argon2id for new password storage, with opportunistic rehash on login during migration | Migrates users without forcing a mass password reset. Consequence: the legacy verifier lives in the compatibility adapter until `OD-03` closes and the window ends | [§12.1](#121-password-hashing) | `VG-05` |
| `AD-13` | Transactional PostgreSQL outbox as the default domain-event mechanism | Atomicity between business state and event publication without distributed transactions. Consequence: a broker may later be an adapter behind the outbox, never a replacement. Kafka/NATS/RabbitMQ are not v1 dependencies | [§15](#15-transactional-events) | `VG-11` |
| `AD-14` | PostgreSQL-backed job queues with horizontally scalable Rust workers | Built on `FOR UPDATE SKIP LOCKED` and explicit AWBMS contracts rather than a third-party scheduler abstraction. Consequence: AWBMS owns claim, lease, retry and dead-letter semantics — more code, fully inspectable | [§16](#16-job-architecture) | `VG-11` |
| `AD-15` | No external provider I/O inside a business database transaction | Shorter locks, less connection starvation, safe retries, provider-outage isolation. Consequence: user-visible effects become asynchronous, so the UX must account for eventual completion | [§17](#17-external-io-rule) | `VG-05` |
| `AD-16` | Reqwest with rustls and explicit feature configuration | Never depend on whichever TLS backend transitive features happen to select. Consequence: outbound calls must declare timeouts, body limits, redirect and retry policy, and SSRF restrictions | [§18](#18-outbound-httptls) | `VG-13` |
| `AD-17` | Cloudflare R2 behind an AWBMS storage port | Domain logic depends on the port, not the provider. Consequence: filesystem and in-memory adapters become available for local and test use, which is most of the benefit | [§19](#19-object-storage) | `VG-05` |
| `AD-18` | Redis/Valkey optional, never an initial correctness dependency | AWBMS must remain correct after full cache eviction. Consequence: permissions, membership, session state, workflow truth, audit and financial state may never live only in cache | [§20](#20-cache-architecture) | `VG-05` |
| `AD-19` | OpenAPI and AsyncAPI contract-first | Reviewed contract files, not Rust derive macros, are the source of API truth during migration. Consequence: contracts and implementation must be kept in step by a gate rather than by generation | [§21](#21-api-contracts) | `VG-08` |
| `AD-20` | `tracing` + OpenTelemetry/OTLP | Correlation across request, job and event boundaries. Consequence: a redaction discipline is mandatory — tokens, MFA secrets, recovery codes and unnecessary PII must never be logged | [§22](#22-observability) | `VG-02` |
| `AD-21` | A stable `ApiError` envelope and client-facing error taxonomy, defined before module implementation | Error shape is a published contract under `AD-19` whether or not anyone intended it. Consequence: defining it late means retrofitting every handler already written | [§29](#29-error-model) | `VG-08` |
| `AD-22` | Pinned dependencies with supply-chain gates | `Cargo.lock` committed; `cargo audit`, `deny`, `vet`, licence allow-list, SBOM, container scan. Consequence: dependency updates become reviewed events, and `OD-08` must close before `cargo-deny` can run | [§27](#27-supply-chain-security) | `VG-13` |
| `AD-23` | Idempotency as a reusable platform capability | High-risk and retry-prone mutations must resolve to one committed business effect. Consequence: key scope, principal binding, request fingerprint, in-flight conflict and replay semantics are platform concerns, not per-module ones | [§31](#31-idempotency) | `VG-11` |
| `AD-24` | AWCMS compatibility layer and parity suite, with no source or runtime dependency on the AWCMS repository | Compatibility must be reproducible without making AWCMS a build dependency. Consequence: legacy artifacts enter only as provenance-stamped frozen fixtures | [§3](#3-repository-independence), [§36](#36-awcms-compatibility-architecture) | `VG-15` |
| `AD-25` | Migration ledger with ordering, applied-migration immutability, SHA-256 checksums and a cross-process lock | A framework default that weakens these guarantees is not acceptable. Consequence: AWBMS may need to own the migration runner rather than adopt one wholesale | [§38](#38-migration-ledger) | `VG-10` |
| `AD-26` | A first-class AWCMS↔AWBMS parity harness | "Obviously the same behaviour" is exactly the claim the harness exists to check. Consequence: it cannot be built before `VG-01` supplies fixtures and `OD-03` fixes its scope | [§39](#39-awcms--awbms-parity-harness) | `VG-05` |
| `AD-27` | Strangler migration; exactly one implementation owns production writes for a capability at a time | Avoids a single massive replacement, and avoids double-writes. Consequence: coexistence on a shared schema forces expand-contract on every migration — see `OD-01` | [§40](#40-coexistence-and-cutover) | `VG-05` |
| `AD-28` | Rollback defined and rehearsed before each production wave deploys | Rollback is not "redeploy the old image" once database state or queue ownership has changed. Consequence: a wave without a verified recovery point is not deployable | [§42](#42-rollback-model) | `VG-12` |
| `AD-29` | `#![forbid(unsafe_code)]` in AhliWeb-owned crates where practical | Removes an entire vulnerability class from first-party code by default. Consequence: any exception needs an ADR, isolation, security review, dedicated tests and fuzzing | [§26.1](#261-unsafe-code-policy) | `VG-13` |
| `AD-30` | Docker + Coolify + Traefik + Cloudflare for deployment and edge | Matches existing operational capability — see `AS-02`. Consequence: multi-stage builds, non-root runtime, no toolchain in the final image, runtime-injected secrets | [§43](#43-deployment-architecture) | `VG-02` |
| `AD-31` | The specification is versioned `vX.Y.Z` with the repository, using changesets | One version number, mechanically checked across `VERSION`, `CHANGELOG.md` and this document's header. Consequence: the standalone `Document version` field is retired; every change carries a changeset | [§0.5](#05-document-control) | `VG-16` |
| `AD-32` | Structural document invariants are enforced by an executable gate, not by review attention | v0.1.0 shipped dangling identifiers, a wrong fact count and a stale commit claim, in a document written to be rigorous. Consequence: the gate is binding about form and silent about content — reviewers keep the judgement work | [§0.5](#05-document-control) | `VG-16` |

## Appendix D — Open decision register

An open decision must not be resolved silently in prose or in implementation code. Closing one means recording a decision with its rationale, as a new `AD-` row or an ADR.

Six of the nine block Blueprint sign-off ([§49.1](#491-entry-conditions) condition 2). The other three block later stages and are listed here so they are not rediscovered as surprises.

| ID | Question | Why it is open | Decision criteria | Blocks | Owner |
|---|---|---|---|---|---|
| `OD-01` | Which migration mode: takeover of the AWCMS production database, a new AWBMS-native deployment, or deterministic migration into a native schema? | The three modes are not interchangeable. Takeover forces schema-name compatibility and expand-contract on every migration; a new deployment forgoes coexistence entirely; native migration needs a reconciliation pipeline neither other mode requires | Per deployment, weighing coexistence duration, downtime tolerance and reconciliation cost. `AD-27` is the successor decision to record | **Blueprint sign-off.** Do not begin migration engineering while all three remain open | *unassigned* |
| `OD-02` | What is the dependency and toolchain upgrade policy? | "Supported/LTS line where practical" names no cadence, no owner and no upgrade trigger, so it is not actionable | Must define pin granularity, routine upgrade cadence, and an expedited path for a RustSec advisory | CI establishment, [§51](#51-initial-implementation-sequence-after-definition-of-ready) step 2 | *unassigned* |
| `OD-03` | Which AWCMS credential formats, session shapes and legacy contracts must AWBMS accept? | Cannot be answered from this document; it depends on what `VG-01` finds in the implementation | Scope chosen against migration risk and the cost of carrying compatibility surface | **Blueprint sign-off.** Determines `AD-26` harness scope and [§36](#36-awcms-compatibility-architecture) surface | *unassigned* |
| `OD-04` | What are the acceptance thresholds for each benchmark scenario? | No threshold is defined for any metric, so "benchmark improves a defined metric" ([§48](#48-performance-validation-gates)) is unenforceable | Per scenario: an explicit SLO, or a "baseline-only, no regression beyond X%" rule. Depends on `OD-06` | **Blueprint sign-off.** Until closed, `VG-09` verifies only that measurements were taken, not that they were acceptable | *unassigned* |
| `OD-05` | How many deployments exist, and are tenants separated by deployment or only by tenant ID within one? | Determines what tenant isolation means operationally as well as in SQL | Product and commercial requirements, weighed against operational cost per deployment | **Blueprint sign-off** | *unassigned* |
| `OD-06` | What are the SLO targets per capability? | [§49.2](#492-blueprint-contents) lists SLOs as Blueprint content; none exists. `OD-04` cannot set thresholds without them | Availability, latency and durability targets per capability class | **Blueprint sign-off** | *unassigned* |
| `OD-07` | Does search stay in PostgreSQL (`tsvector`/GIN/`pg_trgm`) or move to a dedicated index? | Changes whether search is a microservice extraction candidate at all ([§8.2](#82-microservice-extraction-rule)), and changes the data lifecycle surface | Corpus size, query shape and relevance requirements, measured | First module implementation requiring search, [§51](#51-initial-implementation-sequence-after-definition-of-ready) step 24 | *unassigned* |
| `OD-08` | What licence does AWBMS carry, and what dependency licence allow-list applies? | Neither is decided. Until AWBMS carries a licence, default copyright applies and no third party has any right to use it — a decision by default | Commercial and ownership decision for the first; policy derived from it for the second | CI establishment, [§51](#51-initial-implementation-sequence-after-definition-of-ready) step 2 — `cargo-deny` needs the allow-list | *unassigned* |
| `OD-09` | What is the AWBMS v1 module scope? | The 24 AWCMS modules of `C-02` are an inventory, not a commitment. Treating them as scope would make v1 a full reimplementation by default | Product priority against migration risk, module by module | **Blueprint sign-off** | *unassigned* |

## Appendix E — Verification gate register

A gate defines the evidence that would move a claim or decision from *reasoned* to *established*. **A gate reference is not a passed gate.**

**Four pass, one is partial, eleven are open.** That changed at v0.3.0; up to v0.2.0 every gate was open.

Read the passes for what they are. `VG-03` and `VG-04` establish that the chosen stack composes as assumed. `VG-15` and `VG-16` are standing structural conditions checked on every commit — they say nothing about whether any architectural claim is true. **No gate covering AWBMS behaviour has passed.**

| ID | Gate | Evidence artifact | Executable check | Pass rule | Status |
|---|---|---|---|---|---|
| `VG-01` | AWCMS source inventory | `contracts/legacy/awcms/` — module registry, migration list with per-file SHA-256, RLS/FORCE RLS table list, roles and grants, route inventory, OpenAPI/AsyncAPI documents, gate list marked enforced/advisory/absent, authorization vectors, rate-limit dimensions, credential formats | Re-runnable generator committed to this repository, recording the AWCMS commit SHA | Every item in [§2.2](#22-required-awcms-source-inventory-vg-01) present and machine-derived, not prose | **PARTIAL.** Static and live inventories are frozen under `contracts/legacy/awcms/frozen/` with source commits, SHA-256 manifests and regeneration tooling, confirming `C-01`–`C-03`. Not PASS: the gate list of `C-04`/`C-05` is not yet marked enforced/advisory/absent per item, and authorization vectors are not yet consumed by AWBMS tests |
| `VG-02` | Repository bootstrap and CI green | Cargo workspace, `rust-toolchain.toml`, CI configuration | The [§28](#28-maintainability-and-code-quality-gates) command set, plus module DAG, route, table and job ownership checks | All gates green on a commit | **Open.** No workspace exists |
| `VG-03` | Toolchain resolution and pin | `rust-toolchain.toml`, `Cargo.lock` | `cargo --version` in CI against the pin | Toolchain resolved at bootstrap and pinned; `C-06` retired rather than carried forward | **PASS.** Exact `rust-toolchain.toml`, committed `Cargo.lock`, read-only CI with `--locked`. Evidence: `docs/architecture/evidence/VG-03-VG-04-2026-08-29.md` |
| `VG-04` | Ecosystem and sensitive-stack validation | A prototype exercising Axum/Tower, SQLx with RLS and `SKIP LOCKED`, and the transactional outbox | Prototype integration tests | The two flagged `AD-06` matrix rows re-checked against current upstream documentation, and `AD-04`/`AD-06` confirmed or revised on evidence | **PASS.** Verification spike covers request lifecycle, graceful shutdown, tenant-local SQLx transaction context and non-superuser FORCE-RLS denial against real PostgreSQL. Evidence: `evidence/VG-03-VG-04-2026-08-29.md`. Note the narrowness: it validates *composition*, not AWBMS performance or `AS-04` |
| `VG-05` | Correctness and security parity | Parity harness output over the AWCMS request corpus | `tests/parity/` | No unexplained semantic difference; normalisation limited to legitimately nondeterministic fields | **Open.** Blocked on `VG-01` and `OD-03` |
| `VG-06` | Tenant isolation and RLS evidence | RLS test results, `pg_class`/`pg_policy` introspection, role privilege assertions | `tests/rls/` | Cross-tenant access denied at all three layers ([§11.2](#112-tenant-security)); `awbms_app` confirmed `NOBYPASSRLS` and non-owner | **Open** |
| `VG-07` | Authorization chokepoint coverage | Route-to-authorization mapping; decision log samples | Static check that every protected route passes the chokepoint | No protected operation bypasses it; no undocumented exception | **Open** |
| `VG-08` | Contract conformance | OpenAPI and AsyncAPI documents plus the implemented route table | `tests/contract/` | Every implemented route specified and every specified route implemented or explicitly planned; unique `operationId`; error envelope consistent | **Open** |
| `VG-09` | Performance validation | Benchmark results over the [§23.1](#231-required-benchmark-corpus) corpus | Benchmark harness | Measurements taken on identical infrastructure, dataset and workload. **Acceptability is not assessed until `OD-04` closes** | **Open.** No benchmark designed or run |
| `VG-10` | Migration ledger integrity | Migration history table and per-file checksums | Migration runner startup check | Applied migration contents immutable by checksum; cross-process lock demonstrated | **Open** |
| `VG-11` | Idempotency and concurrency evidence | Duplicate-request, parallel-update, job-claim and event-dispatch test results | `tests/concurrency/` | One committed business effect per idempotency key; no double-processing of jobs or events | **Open** |
| `VG-12` | Recovery and rollback evidence | Rehearsal record per migration wave | Restore test against a production-like snapshot | Recovery point verified and rollback executed successfully **before** the wave deploys | **Open** |
| `VG-13` | Supply-chain evidence | `cargo audit`, `deny`, `vet` output; SBOM; container scan; licence report | [§28](#28-maintainability-and-code-quality-gates) command set | All clean or with recorded, reviewed exceptions. Blocked on `OD-08` for the licence allow-list | **Open** |
| `VG-14` | Data lifecycle and privacy coverage | Per-module lifecycle and subject-data descriptors | Coverage check over data-owning modules | Every data-owning module declares descriptors, and declarations match behaviour; legal hold demonstrably non-bypassable | **Open** |
| `VG-15` | Repository independence | Absence of `.gitmodules`; absence of AWCMS path or git dependencies in any Cargo manifest; no vendored AWCMS source | `scripts/check-docs.sh` | No coupling of any prohibited form in [§3.2](#32-prohibited-couplings) | **PASS, and now automated twice** — `scripts/check-docs.sh` plus `tools/vg15_independence.rs`. A standing condition rather than a one-off proof: it can regress on any commit, which is why it is checked on every one |
| `VG-16` | Documentation and version integrity | This document, `VERSION`, `CHANGELOG.md`, `.changeset/` | `scripts/check-docs.sh` | Version consistent across all three locations; every identifier resolves to exactly one register row; no orphaned rows; every internal anchor resolves; changesets well-formed | **Currently passing.** Verifies structure only — it cannot tell whether any claim here is true |

## Appendix F — Approval record

| Field | Value |
|---|---|
| Status | Conditionally approved to **begin** Stage 1 — Master Blueprint |
| Approver | **unassigned** |
| Approval date | **unassigned** |
| Scope of approval | Beginning Blueprint authoring. **Not** approval to begin migration engineering or implementation |

**This record is deliberately incomplete, and its incompleteness is the point.** No human has accepted this validation. The status line above describes the document's intended disposition, not an approval that occurred — and this section is not evidence of one.

Stage 1 may begin immediately; Blueprint authoring is itself the work that closes most open decisions. Stage 1 may not **complete** until the six conditions in [§49.1](#491-entry-conditions) are discharged.

| Condition | State at v0.3.0 |
|---|---|
| 1 — `VG-01` discharged, superseding `C-01`–`C-05` and `C-07` | **Partial.** `C-01`, `C-02`, `C-03` confirmed; `C-04`, `C-05`, `C-07` still unverified — the per-gate enforced/advisory/absent determination is the outstanding work |
| 2 — every Blueprint-blocking open decision closed | 0 of 6 closed |
| 3 — every assumption confirmed or its dependent decision revisited | 1 of 8. `AS-03` is discharged in practice — the inventory freeze required exactly the access it assumes |
| 4 — `VG-03` and `VG-04` discharged | **Met.** Both PASS |
| 5 — `C-06` removed rather than carried forward | **Met.** Superseded by `rust-toolchain.toml` and `Cargo.lock` |
| 6 — AWBMS v1 module scope defined (`OD-09`) | Open |

Two of six conditions are met and a third is partial. **Stage 1 remains `IN PROGRESS`**, consistent with `AWBMS-STAGE-1-GATES.md`: the Blueprint cannot be signed off while `VG-01` is partial and six Blueprint-blocking open decisions stand.

If condition 1 falsifies a material claim — for example if the `C-04`/`C-05` gates prove advisory rather than enforced — this validation MUST be revised before the Blueprint proceeds, not patched inside it.

## Appendix G — AWCMS invariant traceability

The chain each row must complete: **AWCMS invariant → source evidence → AWBMS control → automated check → owner.**

The matrix below establishes the required coverage and the AWBMS side of each chain. **The source-evidence column cannot be completed from this repository** — every entry depends on `VG-01`, which is why the column reads *pending* throughout rather than citing `C-03`, whose own accuracy is unestablished.

Completing this matrix is a Blueprint deliverable. It must not be replaced by a prose assertion that parity was achieved.

| AWCMS invariant | Source evidence | AWBMS control | Automated check | Owner |
|---|---|---|---|---|
| Tenant isolation via FORCE RLS on tenant-scoped tables | Pending `VG-01` (`pg_class`/`pg_policy` introspection) | `AD-10` — application authorization + explicit tenant predicate + FORCE RLS | `VG-06` | *unassigned* |
| Separated database roles; the application role cannot bypass RLS | Pending `VG-01` (`information_schema`) | [§11.1](#111-database-roles) — `awbms_app` is `NOSUPERUSER`, `NOBYPASSRLS`, non-owner, no DDL | `VG-06` | *unassigned* |
| Authorization passes a single chokepoint; default deny; deny overrides allow | Pending `VG-01` (authorization tests) | `AD-10`, [§13](#13-authorization-architecture) | `VG-07` | *unassigned* |
| Route ownership without collision | Pending `VG-01` (route inventory) | `AD-08` — explicit compile-time registry | `VG-02` | *unassigned* |
| One module owns each mutable table | Pending `VG-01` (gate scripts) | `AD-09` | `VG-02` | *unassigned* |
| Job ownership and environment allow-lists | Pending `VG-01` (gate scripts) | `AD-14`, [§16](#16-job-architecture) | `VG-02`, `VG-11` | *unassigned* |
| Module DAG integrity | Pending `VG-01` (registry source) | `AD-08`, [§10](#10-module-composition-model) | `VG-02` | *unassigned* |
| Migration immutability with checksum verification | Pending `VG-01` (migration ledger) | `AD-25` | `VG-10` | *unassigned* |
| Auditability of security-significant actions | Pending `VG-01` (audit implementation) | [§14](#14-audit-architecture) — actor, tenant, action, decision, correlation, reason code | `VG-07` | *unassigned* |
| Data lifecycle and subject-data coverage | Pending `VG-01` (lifecycle descriptors) | [§33](#33-data-lifecycle-and-privacy) — per-module declarations, non-bypassable legal hold | `VG-14` | *unassigned* |
| OpenAPI contract and consumer coverage | Pending `VG-01` (contract files) | `AD-19`, `AD-21` | `VG-08` | *unassigned* |
| Generated-artifact and documentation drift detection | Pending `VG-01` (CI configuration) | `AD-32` for this repository's own documents | `VG-16` | *unassigned* |
| Recipient-oriented abuse controls, not per-IP alone | Pending `VG-01` (rate-limit dimensions, per `C-07`) | [§32](#32-abuse-resistance) — resource-dimensioned limits | `VG-05` | *unassigned* |
| Idempotent high-risk mutations | Pending `VG-01` (idempotency implementation) | `AD-23` | `VG-11` | *unassigned* |

---

## References

### AhliWeb source repositories

- AWCMS: <https://github.com/ahliweb/awcms>
- AWCMS-Astro: <https://github.com/ahliweb/awcms-astro>
- AWCMS module registry: <https://github.com/ahliweb/awcms/blob/main/src/modules/index.ts>
- AWCMS architecture: <https://github.com/ahliweb/awcms/blob/main/docs/ARCHITECTURE.md>
- AWCMS performance/security standard: <https://github.com/ahliweb/awcms/blob/main/docs/awcms/standar-performa-dan-keamanan.md>

### Rust ecosystem

- Rust releases: <https://blog.rust-lang.org/releases/latest/>
- Tokio: <https://tokio.rs/>
- Axum: <https://docs.rs/axum/>
- Tower: <https://docs.rs/tower/>
- Tower HTTP: <https://docs.rs/tower-http/>
- SQLx: <https://docs.rs/sqlx/>
- Serde: <https://serde.rs/>
- Reqwest: <https://docs.rs/reqwest/>
- rustls: <https://docs.rs/rustls/>
- OpenTelemetry Rust: <https://docs.rs/opentelemetry/>
- RustSec Advisory Database: <https://github.com/RustSec/advisory-db>
- cargo-vet: <https://github.com/mozilla/cargo-vet>

### Specifications and standards

- PostgreSQL Row Security: <https://www.postgresql.org/docs/current/ddl-rowsecurity.html>
- OpenAPI Specification: <https://spec.openapis.org/oas/latest.html>
- AsyncAPI Specification: <https://www.asyncapi.com/docs/reference/specification/latest>
- OWASP ASVS: <https://owasp.org/www-project-application-security-verification-standard/>
- OWASP API Security: <https://owasp.org/www-project-api-security/>
- NIST SSDF: <https://csrc.nist.gov/Projects/ssdf>
- ISO/IEC 27001: <https://www.iso.org/standard/27001>

### Indonesian regulatory baseline

- UU No. 27 Tahun 2022 — Pelindungan Data Pribadi: <https://peraturan.bpk.go.id/Details/229798/uu-no-27-tahun-2022>
- PP No. 71 Tahun 2019 — Penyelenggaraan Sistem dan Transaksi Elektronik: <https://peraturan.bpk.go.id/Details/122030/pp-no-71-tahun-2019>
- UU No. 1 Tahun 2024 — Perubahan Kedua atas UU ITE: <https://peraturan.bpk.go.id/Details/274494/uu-no-1-tahun-2024>
