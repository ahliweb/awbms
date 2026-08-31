# AWBMS Stage 1 — Master Blueprint

**Project:** AWBMS — AhliWeb Backend Management System  
**Repository:** `ahliweb/awbms`  
**Lifecycle Stage:** Stage 1 — Master Blueprint  
**Document status:** **DRAFT / IN PROGRESS — NOT YET APPROVED FOR STAGE 2**  
**Prepared:** 2026-08-29  
**Architecture basis:** `docs/architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md` v1.1  
**AWBMS observed HEAD:** `a646bf24c809e2d47c3cb8b9eedeeab82a619cc6`  
**AWCMS baseline:** `11f2e95a47b1328a820f976d60f978c38a067903` (`v10.1.0`)  
**AWCMS-Astro baseline:** `7b753be619244541b817d5d8e7d3b72cfe88d4f9` (`v0.4.0` package state)  

> This Blueprint begins Stage 1 as authorized by the architecture validation. It MUST NOT be treated as Stage-1 sign-off until the Stage-1 completion gates in this document are discharged.

---

## 1. Executive Decision

AWBMS SHALL be developed as an independent, Rust-native backend and evolving system of record for the AhliWeb ecosystem. It SHALL NOT be a line-by-line translation of AWCMS.

The preferred program architecture is:

- Rust 2024 Edition on a pinned stable toolchain.
- Tokio asynchronous runtime.
- Axum + Tower HTTP/service foundation.
- PostgreSQL as authoritative state.
- SQLx as the primary PostgreSQL access layer.
- Modular monolith with explicit module boundaries and future extractability.
- RBAC + ABAC + Separation of Duties + PostgreSQL `FORCE ROW LEVEL SECURITY`.
- Opaque, revocable server-side sessions for human users.
- Transactional outbox for durable domain events.
- PostgreSQL-backed jobs for foundational asynchronous work.
- OpenAPI + AsyncAPI as reviewed contracts.
- `tracing` + OpenTelemetry/OTLP for observability.
- Cloudflare R2 behind an AWBMS storage port.
- Docker/Coolify/Traefik/Cloudflare deployment baseline.
- Strangler migration from AWCMS with compatibility adapters, parity tests, staged cutover, reconciliation and rollback.
- No initial correctness dependency on Redis/Valkey or an external message broker.

AWCMS remains the behavioral, security, migration and compatibility reference until each capability has been intentionally classified and cut over. AWCMS-Astro remains a first-class consumer whose actual API usage is part of the compatibility baseline.

---

## 2. Evidence Baseline

### 2.1 AWBMS

At the observed `main` state, AWBMS still contains documentation only. There is no Cargo workspace, Rust implementation, migration directory, contract fixture directory or CI baseline. Therefore:

- all runtime architecture remains proposed, not implemented;
- no AWBMS benchmark result exists;
- no AWBMS security-control implementation evidence exists;
- no production-readiness claim is permitted.

The architecture validation itself is sound enough to start Stage 1, but its evidence metadata should be amended later to distinguish:

1. original validation baseline commit `5f8f9aa...`; and
2. current validation-document revision commit `a646bf24...`.

### 2.2 AWCMS

The pinned AWCMS baseline confirms:

- 24 registered modules;
- 148 migrations;
- 152 `awcms_*` tables;
- 134 tables with `FORCE` RLS;
- 18 RLS-free global tables by design;
- 507 test files;
- 388 route files;
- 240 ADRs.

The generated inventory and module registry are stronger evidence than prose summaries.

The AWCMS CI also executes a full quality chain plus dedicated PostgreSQL integration/RLS testing and E2E smoke testing. This materially supports the architecture-validation claim that AWCMS contains machine-enforced engineering invariants.

### 2.3 AWCMS-Astro

The pinned Astro baseline is `v0.4.0` package state at commit `7b753be...`.

AWCMS currently freezes thirteen API paths consumed by AWCMS-Astro plus two committed future paths. The consumed surface includes:

- blog posts;
- media objects;
- media public origin;
- composed site profile;
- blog terms;
- blog menus;
- blog widgets;
- site search query;
- site search suggest;
- visitor analytics collect;
- newsletter subscribe;
- newsletter confirm;
- newsletter unsubscribe.

These paths include build-time machine-credential calls and anonymous cross-origin browser calls; the security and failure behavior is not uniform and MUST NOT be normalized blindly.

### 2.4 Current Rust Ecosystem Observation

For verification context on 2026-08-29:

- Rust stable 1.98.0 was released 2026-08-20.
- Axum current docs.rs release observed: 0.8.9.
- Tokio current docs.rs release observed: 1.53.1.
- SQLx current docs.rs release observed: 0.9.0.

These are observations, not Blueprint-pinned versions. The actual AWBMS versions SHALL be authoritative only in `rust-toolchain.toml`, `Cargo.toml` and `Cargo.lock`.

---

## 3. Product Mission

AWBMS exists to become the secure, reliable, maintainable, high-performance backend foundation for AhliWeb multi-ecosystem products, business systems and integrations.

It is intended to provide:

1. a stable backend system of record;
2. reusable identity, authorization, tenant and audit foundations;
3. domain-module composition without uncontrolled coupling;
4. safe multi-tenant PostgreSQL data ownership;
5. contract-first APIs and events;
6. durable jobs/events/integrations;
7. controlled coexistence with AWCMS during migration;
8. measurable operational quality, security and recovery;
9. an architecture that can serve ERP, back-office, SaaS, public platforms and domain-specific products without becoming an unstructured monolith.

---

## 4. Non-Goals

AWBMS v1 SHALL NOT:

- rewrite every AWCMS module merely because Rust is preferred;
- reproduce obsolete AWCMS implementation details when behavior can be simplified safely;
- become microservice-first;
- depend on Kafka/NATS/RabbitMQ for foundational correctness;
- depend on Redis/Valkey for authorization, tenant membership, critical sessions or accounting truth;
- rename all AWCMS schema objects during coexistence merely for aesthetics;
- introduce permanent dual-write between AWCMS and AWBMS;
- bypass AWCMS-Astro compatibility without an explicit consumer migration;
- make performance claims without controlled benchmarks;
- claim ISO, NIST, OWASP or Indonesian regulatory compliance without evidence beyond source code;
- mix Stage-1 architecture decisions with unreviewed product requirements that belong in Stage 2.

---

## 5. Architectural Principles

1. **Evidence before inheritance.** Implemented code, migrations, executable tests and contracts outrank prose.
2. **Native Rust design, behavioral compatibility.** Preserve required invariants; redesign implementation.
3. **Default deny.** Missing authorization/policy context denies access.
4. **Tenant isolation is defense in depth.** App authorization + explicit tenant predicate + `FORCE RLS`.
5. **One mutable table, one owning module.**
6. **External I/O outside ordinary business DB transactions.**
7. **Contract first.** OpenAPI/AsyncAPI and consumer fixtures are reviewed assets.
8. **Async does not mean unbounded.** Concurrency, pools, bodies, queues and provider calls are bounded.
9. **Correct after cache eviction.** Cache is optimization, not truth.
10. **Migrate vertically.** A capability is moved with data, auth, API, audit, jobs, tests and rollback—not merely its handlers.
11. **No silent compatibility break.**
12. **No performance optimization that weakens correctness or security.**
13. **Repository independence.** AWCMS is an external reference and fixture source, never a runtime/source dependency.

---

## 6. System Context

```text
                            Cloudflare
                   DNS / CDN / WAF / TLS edge
                              |
                           Traefik
                              |
                +-------------+-------------+
                |                           |
           AWBMS server              Public/API consumers
          Rust / Axum / Tower       AWCMS-Astro, mobile,
                |                   integrations, admin UI
                |
      +---------+----------+
      |         |          |
  identity   modules    contracts
      |         |          |
      +---------+----------+
                |
              SQLx
                |
           PostgreSQL
   RBAC/ABAC/SoD + FORCE RLS
                |
       +--------+---------+
       |                  |
 transactional outbox   job queue
       |                  |
       +--------+---------+
                |
           AWBMS worker
                |
       +--------+---------+
       |        |         |
      R2      email   external APIs
```

Initial binary boundaries:

- `awbms-server`
- `awbms-worker`
- `awbms-cli`

They share a workspace but SHALL have distinct runtime privileges.

---

## 7. Target Workspace Shape

```text
awbms/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── deny.toml
├── apps/
│   ├── server/
│   ├── worker/
│   └── cli/
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
├── migrations/
├── contracts/
│   ├── openapi/
│   ├── asyncapi/
│   └── legacy/awcms/
├── tests/
│   ├── contract/
│   ├── integration/
│   ├── parity/
│   ├── rls/
│   ├── security/
│   ├── concurrency/
│   └── e2e/
├── docs/
│   ├── adr/
│   ├── architecture/
│   ├── security/
│   └── operations/
└── ops/
    ├── Dockerfile.production
    ├── compose/
    └── coolify/
```

No global `controllers/`, `models/`, `repositories/`, `services/` architecture is recommended. Domain locality is preferred.

---

## 8. AWBMS v1 Scope — OD-09 Decision

### 8.1 v1 definition

AWBMS v1 is defined as:

> **The Rust-native platform kernel plus the minimum content/public-consumer compatibility wave required to prove production-grade tenancy, identity, authorization, contracts, migration and cutover against a real AWCMS-Astro consumer.**

This deliberately avoids both extremes:

- a tiny hello-world kernel with no real compatibility proof; and
- a full 24-module rewrite before value can ship.

### 8.2 v1 authoritative scope

The following capabilities are in the intended AWBMS v1 migration/program scope:

| AWCMS capability | AWBMS treatment | Reason |
|---|---|---|
| `tenant_admin` | Redesign | foundational tenancy, lifecycle and provisioning |
| `profile_identity` | Redesign/Port behavior | identity/profile compatibility |
| `identity_access` | Redesign | auth, session, machine credentials, RBAC/ABAC/SoD |
| `module_management` | Redesign | explicit Rust module registry/composition |
| `logging` | Redesign | audit + operational logging separation |
| `domain_event_runtime` | Redesign | transactional outbox/event semantics |
| `data_lifecycle` | Redesign | privacy, retention, legal hold |
| `email` | Port behavior behind provider port | required by identity/newsletter and future workflows |
| `media_library` | Redesign/Port contract | AWCMS-Astro consumer dependency |
| `blog_content` | Port behavior, redesign implementation | main content compatibility vertical |
| `tenant_domain` | Port behavior, harden | host/origin tenant resolution |
| `seo_distribution` | Port required behavior | site profile/discovery dependency |
| `site_search` | Port contract/behavior | AWCMS-Astro browser consumer |
| `newsletter` | Port with abuse controls | consumed write surface; security-sensitive |
| `site_profile` | Port contract/behavior | AWCMS-Astro build dependency |
| `visitor_analytics` | Redesign privacy-first | consumed anonymous browser surface |

### 8.3 Temporarily remain in AWCMS

These capabilities remain in AWCMS until a later wave or a product need justifies migration:

- `sync_storage`
- `workflow`
- `reporting`
- `theming`
- `form_drafts`
- `comments`
- `idn_admin_regions`
- `push_delivery`

“Remain temporarily” is not equivalent to “retire.” Each requires a later Reuse / Port / Redesign / Replace / Retire decision backed by consumers and requirements.

### 8.4 v1 delivery waves

**Wave 0 — Evidence and foundation**
- frozen AWCMS/Astro inventory;
- workspace/toolchain/CI;
- DB role/RLS test harness;
- module registry;
- error, config, telemetry.

**Wave 1 — Tenancy and identity**
- tenant;
- profile/principal;
- sessions;
- machine credentials;
- RBAC/ABAC/SoD;
- audit.

**Wave 2 — Durable runtime**
- idempotency;
- outbox/events;
- jobs;
- email provider port;
- data lifecycle.

**Wave 3 — AWCMS-Astro content compatibility**
- tenant domain;
- media;
- blog;
- SEO/site profile;
- search;
- analytics;
- newsletter.

**Wave 4 — Shadow/parity and cutover**
- frozen corpus;
- replay against isolated DBs;
- shadow reads;
- consumer contract verification;
- staged routing;
- rollback rehearsal.

---

## 9. Migration Strategy — OD-01 Decision

### Decision

The default migration mode is:

> **Deterministic migration from AWCMS into an AWBMS-native PostgreSQL schema/database, executed capability-by-capability with a strangler cutover and compatibility adapters.**

This is the long-term default because AWBMS is an independent product and should not inherit AWCMS schema naming and runtime constraints permanently.

### Rules

1. AWCMS remains production owner until a capability passes its cutover gate.
2. Shadow reads are permitted.
3. Production writes SHALL NOT be blindly dual-written into both systems.
4. Captured/sanitized writes may be replayed into isolated AWBMS databases for parity.
5. Each job queue has one active production owner.
6. Data transfer is deterministic, versioned, resumable and reconciliation-driven.
7. Every wave has a tested rollback.
8. Shared-schema takeover is an exception requiring an ADR, not the default.

### Data migration pattern

```text
freeze source version
      ↓
extract/version
      ↓
transform deterministically
      ↓
load AWBMS-native schema
      ↓
reconcile counts/checksums/domain invariants
      ↓
shadow compare
      ↓
cut over one capability
      ↓
observe
      ↓
accept or rollback
```

---

## 10. Toolchain and Dependency Policy — OD-02 Decision

### Policy

- Rust toolchain SHALL be pinned to an exact stable release in `rust-toolchain.toml`.
- `Cargo.lock` SHALL be committed.
- Direct dependencies SHALL be intentionally constrained in `Cargo.toml`.
- Runtime/build dependencies SHALL be reviewed for source, license, maintenance and security.
- Routine dependency review: monthly.
- Broader upgrade review: at least quarterly and before each major release train.
- RustSec/high-severity supply-chain findings: triage within 24 hours.
- Exploitable critical/high findings: expedited remediation target within 72 hours unless a documented compensating control and risk acceptance exists.
- Any runtime/toolchain update must pass compilation, unit/integration/RLS/security/contract/concurrency tests and representative benchmarks.
- Versions are not duplicated as normative values in architecture prose.

The current Rust 1.98.0 observation is therefore evidence for `VG-03`, not the permanent normative version.

---

## 11. Performance Policy — OD-04 Decision

Because no AWBMS benchmark corpus exists yet, Stage 1 closes OD-04 using a **baseline-first no-regression policy**, not invented production SLO claims.

Before any production cutover:

1. `VG-09` SHALL establish a reproducible baseline on fixed infrastructure/data.
2. Correctness, authorization and RLS must pass before performance is considered.
3. Compared with the last approved AWBMS baseline under identical conditions:
   - p95 latency regression >10% requires explicit review;
   - p99 latency regression >10% requires explicit review;
   - throughput regression >10% requires explicit review;
   - CPU/request regression >15% requires explicit review;
   - memory regression >15% requires explicit review;
   - DB connection/pool-wait regression that threatens capacity is blocking regardless of percentage.
4. Product-facing absolute SLO values SHALL be set in Stage 2 PRD and Stage 9 DoR when actual workload and service tier are defined.

Required scenarios:

- authenticated tenant GET;
- RBAC/ABAC mutation;
- write + audit + outbox;
- search;
- worker dispatch;
- media metadata;
- reporting/query representative of migrated modules.

---

## 12. Database and Tenant Architecture

### Roles

Minimum native roles:

- `awbms_migrator`
- `awbms_app`
- `awbms_worker`
- `awbms_operator`

Application/worker roles SHALL be:

- non-superuser;
- `NOBYPASSRLS`;
- not schema/table owner;
- no ordinary DDL;
- minimum DML only.

### Tenant request flow

```text
BEGIN
  ↓
set transaction-local tenant context
  ↓
authenticate/resolve principal
  ↓
authorize
  ↓
execute owned-domain SQL with explicit tenant predicate
  ↓
write audit
  ↓
write outbox when applicable
  ↓
COMMIT
```

Tenant repositories SHALL use a tenant-bound transaction abstraction rather than unrestricted pool access.

RLS tests MUST include malicious cross-tenant requests and direct SQL attempts through the actual application role.

---

## 13. Authentication and Authorization

### Human sessions

Default human session model:

- cryptographically random opaque token;
- raw token returned once;
- only verification material/hash persisted;
- revocable;
- expiry bounded;
- session assurance tracked;
- password reset/security events can revoke sessions;
- MFA/step-up supported.

### Machine credentials

- tenant-bound;
- scope-limited;
- purpose-specific;
- revocable;
- expiry bounded;
- optional network restrictions for high-risk writes;
- raw secret shown once.

### Authorization chokepoint

All protected actions pass one central policy service:

```text
principal
→ assurance
→ tenant membership
→ module enabled
→ declared permission
→ RBAC
→ ABAC
→ business scope
→ SoD
→ resource/domain rule
→ ALLOW/DENY
→ decision audit
```

No generic `is_admin` bypass is allowed.

---

## 14. Module Admission Rules

Every AWBMS module must declare:

- unique key;
- version;
- type/kind;
- dependencies;
- capabilities provided/required;
- permissions;
- routes;
- owned mutable tables;
- events;
- jobs;
- privacy/data categories;
- retention/lifecycle behavior;
- subject-data behavior;
- configuration;
- observability dimensions.

Architecture gates must reject:

- cyclic module dependencies;
- route collisions;
- undeclared table writes;
- undeclared jobs;
- undeclared permissions;
- missing lifecycle metadata for data-owning modules;
- contract drift;
- direct cross-domain mutations.

---

## 15. Events, Jobs and Idempotency

### Events

Use PostgreSQL transactional outbox.

Required event metadata:

- event name;
- schema/version;
- tenant where applicable;
- aggregate/ordering key;
- occurrence time;
- correlation/causation identifiers;
- privacy classification;
- retry/dead-letter state.

### Jobs

PostgreSQL-backed jobs SHALL define:

- registry;
- ownership;
- schedule;
- claim/lease;
- retry/backoff;
- idempotency;
- tenant context;
- failure/dead-letter;
- execution history;
- trace correlation.

### Idempotency

High-risk/retryable mutations SHALL bind the idempotency record to:

- tenant;
- principal/client;
- operation;
- key;
- request fingerprint;
- transaction outcome;
- replayable response metadata;
- expiry.

---

## 16. Storage and External Integrations

Domain code depends on ports.

Object storage capabilities:

- put;
- stat;
- get;
- delete;
- presign;
- checksum/metadata verification.

Cloudflare R2 is an adapter, not a domain dependency.

Outbound HTTP SHALL define:

- connect timeout;
- request timeout;
- body/response limits;
- retry based on idempotency;
- redirect policy;
- SSRF controls for attacker-influenced destinations;
- correlation IDs;
- provider error redaction;
- circuit breaking where evidence justifies it.

Ordinary external provider calls SHALL execute after business commit through outbox/jobs rather than inside open DB transactions.

---

## 17. Contract and Compatibility Architecture

### Contract sources

- `contracts/openapi/`
- `contracts/asyncapi/`
- `contracts/legacy/awcms/`

### AWCMS fixture provenance

Every imported fixture must record:

- source repo;
- source SHA;
- generation/import date;
- SHA-256;
- contract/schema version;
- consuming AWBMS parity test.

### Compatibility adapter

```text
legacy AWCMS request/response
          ↓
     compat-awcms
          ↓
AWBMS application/domain
```

The adapter may translate:

- headers/cookies;
- machine credential formats;
- response shapes;
- legacy error codes;
- identifiers;
- versions.

New domain rules SHALL NOT live in the compatibility layer.

---

## 18. AWCMS-Astro Compatibility Priority

The current consumed surface must become a frozen Stage-1 fixture. Particular attention is required because it contains at least five materially different execution/security classes:

| Class | Example | Important invariant |
|---|---|---|
| Build-time authenticated read | blog posts | read-only machine credential + tenant binding |
| Build-time media resolution | media objects | unresolved IDs are explicit, not silently dropped |
| Anonymous browser read | site search | cross-origin tenant resolution, no credential |
| Anonymous browser telemetry write | analytics collect | neutral/limited response and abuse controls |
| Anonymous browser mail-triggering write | newsletter subscribe | recipient cooldown + anti-enumeration + origin binding |

This distinction must survive the Rust migration.

---

## 19. Security and Privacy Baseline

Mandatory controls include:

- least privilege;
- separate DB roles;
- default-deny authz;
- `FORCE RLS`;
- tenant-bound transactions;
- secure session lifecycle;
- MFA/step-up architecture;
- safe SQL binding;
- input/body limits;
- rate/abuse controls;
- idempotency/replay defenses;
- audit and decision logging;
- secret/PII redaction;
- data lifecycle;
- legal hold;
- backup/restore evidence;
- dependency governance;
- security regression tests;
- SSRF protections;
- secure headers/TLS configuration;
- container non-root and minimal runtime;
- incident and vulnerability readiness.

### Privacy-by-design declarations

Each data-owning module declares:

- purpose;
- categories;
- PII/sensitivity;
- source;
- tenant/global scope;
- retention;
- archive;
- erasure/anonymization;
- legal hold;
- export/access request behavior.

---

## 20. Standards and Regulatory Mapping

This is a control-alignment baseline, not a compliance certification.

| AWBMS concern | Relevant references |
|---|---|
| Security management and control governance | ISO/IEC 27001, ISO/IEC 27002 |
| Risk identification/treatment | ISO/IEC 27005 |
| Cloud shared-responsibility controls | ISO/IEC 27017 |
| Protection of PII in cloud services | ISO/IEC 27018 |
| Application security lifecycle | ISO/IEC 27034 |
| Privacy information management | ISO/IEC 27701 |
| IT service management | ISO/IEC 20000-1 |
| Business continuity/recovery | ISO 22301 |
| Security target/assurance concepts | ISO/IEC 15408 |
| Product/software quality | ISO/IEC 25010 |
| Web/application/API verification | OWASP ASVS, OWASP Top 10, OWASP API Security |
| Secure software development | NIST SSDF |
| Cybersecurity governance | NIST CSF |
| Operational hardening | CIS Controls |
| Indonesian personal data | UU No. 27 Tahun 2022 (PDP) |
| Indonesian electronic systems | PP No. 71 Tahun 2019 (PSTE) |
| Indonesian electronic-information legal baseline | UU No. 1 Tahun 2024 (Second Amendment to ITE Law) |

Sector-specific regulation SHALL be added when an AWBMS deployment enters healthcare, finance, government or another regulated domain.

---

## 21. Observability

Required telemetry:

### Requests
- request ID;
- correlation ID;
- trace ID;
- tenant ID where safe;
- module;
- operation;
- latency;
- status/error code.

### Database
- active/idle connections;
- pool wait;
- query duration;
- transaction duration;
- lock wait/deadlocks.

### Auth/security
- auth failures;
- revocations;
- MFA/step-up;
- policy denies;
- cross-tenant denial attempts;
- privileged operations.

### Jobs/events
- queue depth;
- lag;
- retries;
- dead letters;
- replay;
- throughput.

### Runtime
- panics;
- CPU/memory;
- task pressure;
- blocking work;
- open connections.

Secrets and unnecessary PII are forbidden in telemetry.

---

## 22. Deployment and Recovery

Baseline:

- Docker multi-stage build;
- minimal runtime image;
- non-root;
- runtime secret injection;
- health/readiness separation;
- graceful shutdown;
- resource limits;
- SBOM;
- image scanning;
- immutable digest promotion;
- Coolify/Traefik orchestration;
- Cloudflare edge controls.

Every cutover wave must have:

1. recovery point;
2. deployable previous owner;
3. migration compatibility state;
4. routing reversal;
5. worker ownership reversal;
6. reconciliation procedure;
7. post-rollback verification.

---

## 23. Five Architecture Alternatives Compared

| Approach | Compatibility risk | Operational complexity | Long-term cleanliness | Migration safety | Decision |
|---|---:|---:|---:|---:|---|
| Line-by-line AWCMS→Rust rewrite | High | Medium | Low | Low | Reject |
| Big-bang replacement | Very high | Medium | Medium | Very low | Reject |
| Microservices-first rewrite | High | Very high | Medium | Low | Reject initially |
| Shared-DB permanent coexistence | Medium | High | Low | Medium | Exception only |
| Modular-monolith + strangler + native DB | Lowest controllable | Medium | High | High | **Preferred** |

The preferred option is not the smallest amount of engineering, but it best preserves testability, rollback, independence and gradual value delivery.

---

## 24. Practical Implementation Examples

### Example 1 — Cross-tenant access

A principal from tenant A requests tenant B data.

Required outcome:

- central authz denies;
- repository query includes tenant A;
- RLS denies tenant B row even if the application predicate is defective;
- decision is safely audited;
- regression test proves denial with the real app DB role.

### Example 2 — Newsletter abuse

An attacker repeatedly submits a victim email.

Required controls:

- rate limits by requester;
- cooldown by recipient/resource;
- origin/tenant resolution;
- neutral 200 behavior to prevent enumeration;
- refused repeats do not invalidate a previously-issued confirmation token;
- provider send occurs asynchronously.

### Example 3 — Retried invoice/order mutation

Client sends POST, times out and retries.

Required outcome:

- same tenant+principal+idempotency key+request fingerprint;
- one domain mutation;
- one stable resource identity;
- one expected event sequence;
- response can be replayed safely.

### Example 4 — Worker cutover

AWCMS email worker is replaced by AWBMS.

Procedure:

1. stop AWCMS from claiming new jobs;
2. drain in-flight jobs;
3. reconcile queue;
4. activate AWBMS worker;
5. observe retry/dead-letter/throughput;
6. accept or reverse ownership.

Never run both owners on the same non-idempotent workload by assumption.

### Example 5 — R2 outage

A media-related business change commits while R2 is unavailable.

Correct design:

- business transaction commits valid metadata/intention;
- outbox/job records upload/repair work;
- worker retries with bounded policy;
- unrelated DB locks are not held during R2 timeout;
- user-facing status reflects pending/failed media state.

### Example 6 — AWCMS-Astro contract change

AWBMS changes the blog response shape.

Required sequence:

1. compare against frozen AWCMS-Astro consumer fixture;
2. reject breaking change unless versioned/adapter-backed;
3. update consumer in coordinated PR/release;
4. preserve old shape during compatibility window;
5. remove only after usage evidence confirms retirement.

---

## 25. Historical Lessons Applied

### International engineering history

Large platform rewrites repeatedly fail when teams combine language migration, data-model redesign, service decomposition and product change in one big-bang release. The Blueprint therefore separates runtime modernization from capability cutover and uses strangler-style migration.

Rust's evolution also supports this approach: memory safety and modern async tooling improve implementation safety, but they do not solve authorization, tenancy, privacy, SQL logic or operational recovery. Those controls remain explicit.

### AhliWeb/AWCMS lessons

AWCMS demonstrates that many high-value controls arose because ordinary code review did not catch drift or abuse classes. Examples include:

- architecture-gate drift;
- consumer-contract drift;
- tenant resolution mismatch;
- newsletter recipient abuse;
- release workflow assumptions;
- migration/RLS invariants.

AWBMS should inherit the *lesson and control objective*, not necessarily the original TypeScript implementation.

---

## 26. Stage-1 Gate Register

| Gate | Current state | Required evidence before Stage-1 sign-off |
|---|---|---|
| `VG-01` AWCMS source inventory | **PARTIAL** | frozen machine-generated AWCMS + Astro artifacts under `contracts/legacy/awcms/`, provenance, hashes, live DB RLS/roles where required, auth vectors |
| `VG-03` toolchain resolution | **PARTIAL** | actual `rust-toolchain.toml`, build evidence |
| `VG-04` ecosystem validation | **PARTIAL** | Axum/SQLx/Tokio/PostgreSQL spike + tests; sensitive behavior verified |
| `VG-05` parity | NOT STARTED | golden request/DB/audit/event comparison |
| `VG-09` performance | NOT STARTED | reproducible baseline corpus |
| `VG-12` recovery | NOT STARTED | restore/rollback rehearsal |
| `VG-15` repo independence | CURRENTLY TRUE / needs automation | CI check against source/runtime AWCMS coupling |

---

## 27. Stage-1 Assumption Register

| Assumption | Validation method | Sign-off requirement |
|---|---|---|
| PostgreSQL remains primary DB | confirm product/ops constraints | owner approval |
| Coolify/Traefik/Cloudflare remain default deployment | ops review | documented deployment profile |
| AWCMS DB can be exported/snapshotted safely | staging migration rehearsal | successful deterministic restore/migration |
| Separate DB roles are operationally supportable | staging role test | RLS integration pass |
| AWCMS-Astro remains a required consumer during migration | consumer inventory | pinned fixture + ownership |
| No module requires microservice isolation at v1 | workload/threat review | no evidence requiring extraction |
| R2/provider APIs can be abstracted behind ports | adapter spike | integration test |
| Product SLOs are not yet fixed | Stage-2 PRD | explicit PRD values or tier policy |

Any falsified assumption reopens dependent decisions.

---

## 28. Stage-1 Completion Criteria

Stage 1 may be signed off only when:

1. `VG-01` is discharged.
2. All Blueprint-blocking open decisions are closed.
3. All assumptions are confirmed or dependent decisions are revised.
4. `VG-03` is discharged.
5. `VG-04` is discharged.
6. Exact dependency/toolchain versions live in manifests, not duplicated as architecture truth.
7. v1 scope is accepted.
8. decision register has owner/date/status/consequence/gate fields.
9. AWCMS invariant → AWBMS control → test/evidence traceability exists.
10. explicit approval record identifies the accountable approver.

Until then, status remains **Stage 1 — In Progress**.

---

## 29. Immediate Stage-1 Work Packages

### WP-1 — Freeze source inventory
Create re-runnable tooling to import:

- AWCMS module registry;
- migration list + SHA-256;
- generated table/RLS inventory;
- roles/grants from a controlled DB;
- route/auth inventory;
- OpenAPI;
- AsyncAPI;
- gate inventory;
- authorization vectors;
- AWCMS-Astro consumed/committed paths.

### WP-2 — Bootstrap verification spike
Create minimal Rust workspace and prove:

- pinned stable Rust;
- Axum route;
- Tower request/correlation ID;
- SQLx PostgreSQL connection;
- transaction-local tenant context;
- non-superuser RLS denial;
- clean shutdown;
- compile/lint/test/security tooling.

This is evidence for `VG-03`/`VG-04`, not feature implementation.

### WP-3 — Create ADR set
Minimum:

- ADR: Rust/toolchain pin and upgrade policy;
- ADR: Axum/Tower;
- ADR: SQLx/PostgreSQL;
- ADR: native DB strangler migration;
- ADR: v1 module scope;
- ADR: opaque sessions;
- ADR: authz/RLS model;
- ADR: outbox/jobs;
- ADR: compatibility fixture policy.

### WP-4 — Build traceability matrices
- AWCMS invariant → evidence → AWBMS control → test.
- AWCMS module → disposition → consumer → migration wave.
- security risk → control → test → evidence → monitoring.
- business goal → capability → acceptance criteria → implementation issue.

### WP-5 — Stage-1 review
Perform cross-document review and formally record Stage-1 approval only after gates pass.

---

## 30. Decision Register — Stage-1 Additions

| ID | Decision |
|---|---|
| `AD-S1-01` | Default migration is strangler + deterministic migration to AWBMS-native PostgreSQL |
| `AD-S1-02` | Shared AWCMS DB takeover is exception-only and requires ADR |
| `AD-S1-03` | v1 includes platform kernel + AWCMS-Astro content compatibility wave |
| `AD-S1-04` | Eight non-critical modules remain in AWCMS temporarily |
| `AD-S1-05` | Toolchain pinned exactly; routine monthly review; expedited security path |
| `AD-S1-06` | Performance acceptance starts with reproducible no-regression baseline; absolute product SLOs set later |
| `AD-S1-07` | AWCMS-Astro commit/path inventory is part of `VG-01`, not optional context |
| `AD-S1-08` | No normative crate versions are duplicated in Blueprint prose |

These decisions are proposed as Stage-1 Blueprint decisions and should be assigned owner/date when committed to the repository.

---

## 31. Recommended Stage Transition

**Current:** Stage 1 has begun.  
**Not ready:** Stage 1 sign-off.  
**Next after Stage-1 completion:** Stage 2 — AWBMS PRD.

Stage 2 should convert the architectural scope into:

- stakeholders/personas;
- business capabilities;
- prioritized user/system requirements;
- service tiers;
- product SLO targets;
- migration success criteria;
- acceptance criteria;
- functional/non-functional requirements;
- explicit v1 release boundaries.

---

## References

### AhliWeb repositories
- https://github.com/ahliweb/awbms
- https://github.com/ahliweb/awcms
- https://github.com/ahliweb/awcms-astro
- https://github.com/ahliweb/awbms/blob/main/docs/architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md
- https://github.com/ahliweb/awcms/blob/main/src/modules/index.ts
- https://github.com/ahliweb/awcms/blob/main/docs/awcms/repo-inventory.md
- https://github.com/ahliweb/awcms/blob/main/scripts/api-consumer-contract.ts
- https://github.com/ahliweb/awcms-astro/blob/main/tests/kontrak-awcms.test.mjs

### Rust ecosystem
- https://blog.rust-lang.org/2026/08/20/Rust-1.98.0/
- https://docs.rs/axum/
- https://docs.rs/tokio/
- https://docs.rs/sqlx/
- https://docs.rs/tower/
- https://docs.rs/tracing/
- https://rustsec.org/
- https://github.com/mozilla/cargo-vet

### Architecture/security
- https://www.postgresql.org/docs/current/ddl-rowsecurity.html
- https://spec.openapis.org/oas/latest.html
- https://www.asyncapi.com/docs/reference/specification/latest
- https://owasp.org/www-project-application-security-verification-standard/
- https://owasp.org/www-project-api-security/
- https://csrc.nist.gov/Projects/ssdf
- https://www.iso.org/standard/27001

### Indonesian regulatory baseline
- https://peraturan.bpk.go.id/Details/229798/uu-no-27-tahun-2022
- https://peraturan.bpk.go.id/Details/122030/pp-no-71-tahun-2019
- https://peraturan.bpk.go.id/Details/274494/uu-no-1-tahun-2024
