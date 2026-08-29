# AWBMS Rust Backend Architecture Validation

| Field | Value |
|---|---|
| Project | AWBMS — AhliWeb Backend Management System |
| Repository | `ahliweb/awbms` |
| Document version | 1.1 |
| Status | Architecture validation — **conditionally approved** to enter Stage 1 (see [§49](#49-master-blueprint-entry-conditions) and [Appendix F](#appendix-f--approval-record)) |
| Original validation date | 2026-08-29 |
| Last revision | 2026-08-29 (rigor/traceability revision; no architectural decision reversed) |
| Evidence basis | This repository at commit `5f8f9aa` + external claims recorded by the original validation (not re-verified here) |
| Decision scope | Rust-based backend architecture, ecosystem selection, scalability, maintainability, security, AWCMS compatibility, migration strategy |
| Explicitly out of scope | Product requirements, UX, pricing/commercial strategy, org/staffing, per-module functional design, SLO target values |
| Owner / approver | *unassigned — see [Appendix F](#appendix-f--approval-record)* |
| Review cadence | Re-validate at the start of each major migration or specification stage, and whenever any gate in [Appendix E](#appendix-e--verification-gate-register) changes state |

---

## 0. How to read this document

### 0.1 Repository state (verified)

**[F]** At commit `5f8f9aa` the `ahliweb/awbms` repository contains exactly one tracked file — this document. There is no Rust code, no `Cargo.toml`, `Cargo.lock` or `rust-toolchain.toml`, no `migrations/`, no `contracts/` directory, no git submodule and no CI configuration.

Two consequences follow, and they govern the whole document:

1. **Nothing in this document describes implemented behavior.** Every architectural statement is a *proposal* for work not yet started. No benchmark has been run, no test suite exists, no dependency version has been resolved by a build.
2. **Claims about systems outside this repository cannot be verified from it.** The AWCMS facts in [§2](#2-source-of-truth-review) and the Rust release fact in [§4.1](#41-recommended-baseline) were asserted by the original validation. They are preserved verbatim as *recorded claims*, each carrying a verification gate. They are not treated as established facts.

### 0.2 Claim labels

Statements are labelled where their epistemic status matters. Unlabelled prose in [§4](#4-rust-ecosystem-validation)–[§56](#56-monitoring-baseline) is design recommendation, equivalent to **[R]**.

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

`MUST`, `MUST NOT`, `SHALL` and `MUST` in capitals carry RFC 2119 / RFC 8174 force. Lowercase "should", "must" and "may" in the discussion sections are ordinary prose and carry **no** normative force — normative force lives in the decision register ([Appendix C](#appendix-c--decision-register)) and the gate register ([Appendix E](#appendix-e--verification-gate-register)). This split is deliberate: it keeps the discussion readable while making the binding surface small enough to review and audit.

### 0.4 Validation status at a glance

| Category | Count | Where |
|---|---:|---|
| Facts verified against this repository | 3 | [§0.1](#01-repository-state-verified), [§3.1](#31-current-state-verified) |
| Recorded external claims awaiting verification | See Appendix A | [Appendix A](#appendix-a--recorded-external-claims) |
| Assumptions | See Appendix B | [Appendix B](#appendix-b--assumption-register) |
| Binding decisions | See Appendix C | [Appendix C](#appendix-c--decision-register) |
| Open decisions blocking the Blueprint | See Appendix D | [Appendix D](#appendix-d--open-decision-register) |
| Verification gates | See Appendix E | [Appendix E](#appendix-e--verification-gate-register) |

**What this validation does and does not establish.** It establishes that a coherent, internally consistent Rust architecture *can* be specified for AWBMS, and it records the trade-off reasoning for each major selection. It does **not** establish that the selected crates meet AWBMS requirements under load, that AWCMS behavior has been inventoried, or that migration is feasible on any particular schedule. Those are the open items in Appendices A, D and E.

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

> **Known contradiction — do not propagate `C-03` uncritically.** `C-03` is sourced from AWCMS prose, and [§2.1](#21-documentation-drift-warning) states that AWCMS prose is known to lag the implementation and must be outranked by the registry, migrations and executable tests. The migration count `sql/148` is therefore a *documentation* figure of unknown accuracy, not a verified schema state. `VG-01` MUST re-derive it from the migration directory and ledger table rather than from prose. The same caution applies to the capability descriptions in `C-03`; `C-02`'s module list, by contrast, was reported from the registry and is more likely to hold, but is gated identically.

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

Each artifact MUST be stored under `contracts/legacy/awcms/` with the provenance fields required by [§3](#3-repository-independence). Once frozen, the inventory — not this section — is the requirements baseline.

---

## 3. Repository independence

`ahliweb/awbms` is a separate product repository (`AD-24`).

### 3.1 Current state (verified)

**[F]** At commit `5f8f9aa` this repository has **no** dependency on AWCMS of any kind: no git submodule, no `.gitmodules`, no Cargo manifest and therefore no path or git dependency, and no vendored AWCMS source. Independence is currently a fact, not merely an intention — the gate `VG-15` exists to keep it one once a build is introduced.

**[F]** Equally, `contracts/legacy/awcms/` does not yet exist. The fixture pipeline described below is entirely prospective.

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
- large-scale indexing/search.

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

The registry must be explicit and reviewed at compile time, not populated by runtime plugin discovery.

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

A mutable table should have one authoritative module owner.

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

Use Argon2id for new AWBMS-native password storage unless a reviewed compatibility requirement dictates otherwise.

During AWCMS migration, AWBMS should support the existing password format and opportunistically rehash on successful authentication rather than forcing all users to reset passwords.

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

External provider HTTP calls should not execute while a business database transaction is being held open unless a formally reviewed algorithm requires it.

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

Use Reqwest with rustls and explicit feature configuration.

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

Any future need for unsafe Rust requires:

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

A standard `ApiError` contract should be defined before module implementation.

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

Idempotency should be a reusable platform capability for high-risk or retry-prone mutations.

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

Do not rely solely on a migration framework default if it weakens these guarantees.

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

Rollback is not merely “redeploy old image” if database state or queue ownership has changed.

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
- concurrency locks.

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
2. Establish formatting/lint/test/dependency-security CI.
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

## Appendix A — Recorded external claims

This register records claims inherited from the original validation rather than verified in the current AWBMS repository. The claims currently identified are `C-01`–`C-07`, including the claims labelled in §2 and §4.1. Each claim remains unresolved until `VG-01` re-checks it against a pinned source commit and stores the resulting inventory under `contracts/legacy/awcms/`.

**Required fields:** claim ID, source repository, source commit, observation, verification date, verifier, evidence path, and disposition (`confirmed`, `amended`, or `rejected`).

## Appendix B — Assumption register

Assumptions are not facts and must not be carried into implementation silently. The current document contains assumptions about deployment topology, incumbent runtime behavior, migration access, workload shape, tenant boundaries, availability requirements, provider capabilities, and operational ownership. Before Blueprint sign-off, each assumption must receive an owner, a validation method, a target stage, and a disposition. If an assumption is falsified, its dependent decision must be revisited.

## Appendix C — Decision register

Decision identifiers such as `AD-01`, `AD-02`, `AD-04`, `AD-06`, `AD-07`, `AD-10`, `AD-11`, `AD-13`, `AD-14`, `AD-17`, `AD-18`, `AD-19`, `AD-20`, `AD-22`, `AD-24`, `AD-27`, and `AD-30` are the binding decision references used in this document. Their authoritative register entry must contain: status, decision text, rationale, alternatives considered, consequences, owner, date, superseded-by (if any), and verification gate. No decision is implementation-ready until those fields are completed in the Master Blueprint or a linked ADR.

## Appendix D — Open decision register

The currently explicit open decisions are `OD-01` (migration mode), `OD-02` (dependency/toolchain upgrade policy), `OD-04` (performance acceptance thresholds), and `OD-09` (AWBMS v1 module scope). The Blueprint must add any missing open decisions discovered during source inventory and assign each one an owner, deadline, decision criteria, and blocking stage. An open decision must not be silently resolved in implementation code.

## Appendix E — Verification gate register

The gates currently referenced are `VG-01` (AWCMS source inventory), `VG-03` (toolchain resolution), `VG-04` (ecosystem and sensitive-stack validation), `VG-05` (correctness/security parity), `VG-09` (performance validation), `VG-12` (recovery/rollback evidence), and `VG-15` (repository independence). Each gate must define its evidence artifact, executable check where applicable, pass/fail rule, owner, and last verified commit or date. A gate reference without evidence is not a passed gate.

## Appendix F — Approval record

**Status:** conditionally approved to begin Stage 1 — Master Blueprint. **Approver:** unassigned. **Approval date:** unassigned. **Conditions:** the entry conditions in §49.1 must be discharged before Blueprint sign-off. This section is intentionally incomplete until an owner records a real approval decision; it is not evidence of approval by itself.

## Appendix G — AWCMS invariant traceability

The required traceability chain is: AWCMS invariant → source evidence → AWBMS control/decision → automated test or operational check → owner. At minimum, the matrix must cover tenant isolation, authorization chokepoints, route/table/job ownership, migration immutability, auditability, data lifecycle, contract compatibility, and generated-artifact drift. The matrix is a Blueprint deliverable and must not be replaced by a prose assertion of parity.

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
