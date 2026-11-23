# AWBMS Pre-Production Evidence Specification

## Document control

| Field | Value |
|---|---|
| Status | **SPECIFICATION ONLY — no gate status changes, no code, no approval** |
| Issue | `#7` — items P2.9 (invariant gates in CI) and P2.10 (production evidence shape) |
| Written | 2026-11-23 |
| Repository state read | `main` at `59cc2c3` (see [§1](#1-purpose-scope-and-non-effects)) |
| Authority | Subordinate to the [architecture validation](AWBMS-RUST-ARCHITECTURE-VALIDATION.md), the [Master Blueprint](AWBMS-MASTER-BLUEPRINT.md) and the [Stage 1 gate register](AWBMS-STAGE-1-GATES.md). Where this document and those disagree, those win and this document is wrong |

Claim labels follow [§0.2 of the validation](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#02-claim-labels). In this document:

- **[F]** is used only for statements checked against this repository at the commit above.
- **[D]** is used only to cite a decision that already exists in [Appendix C](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#appendix-c--decision-register). Nothing here creates one.
- **[R]** marks everything this document adds: a default that a team may override by recording a decision.
- **[O]** is used only to cite an existing open decision in [Appendix D](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#appendix-d--open-decision-register), or to say that a value is an owner decision which already belongs to one.
- **[G]** cites an existing gate in [Appendix E](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#appendix-e--verification-gate-register).
- **[C]** and **[A]** are not introduced. Where this document would otherwise assert something about an external tool or standard, it says "candidate" or "to be verified by the adopting change" instead, because a `[C]` or `[A]` would need an Appendix A or B row that this change is not permitted to add ([§7](#7-proposed-register-changes-not-registered-here)).

## 1. Purpose, scope and non-effects

Issue `#7` asks for two things to be defined before real domain modules exist: the architecture invariants that must become CI gates ([§3](#3-ci-invariant-gates)), and the shape of the evidence that `VG-05`, `VG-09` and `VG-12` will eventually need, together with the surrounding production-readiness evidence ([§4](#4-production-evidence-shapes)). This document specifies both. It exists so that the first module, the first migration and the first route are not built into an architecture that cannot later be evidenced.

**What this document does not do:**

- It does not change the status of any gate. Gate status is authoritative in the [Stage 1 gate register](AWBMS-STAGE-1-GATES.md), and `VG-05`, `VG-09` and `VG-12` remain **OPEN** there. A specification of evidence is not evidence, which is the register's own rule: documentation that merely describes future evidence is not a passed gate.
- It does not pull these later gates into Stage 1 sign-off.
- It does not register any `AD-`, `OD-`, `VG-`, `C-` or `AS-` identifier. Proposals are listed as recommendations in [§7](#7-proposed-register-changes-not-registered-here).
- It does not restate decisions that live elsewhere. Where a statement already exists in the validation or the Blueprint, this document links to it.
- It does not set any performance, availability, recovery or retention number ([§4.2](#42-vg-09--performance-evidence), [§4.3](#43-vg-12--recovery-and-rollback-evidence)).
- It does not claim conformance with any standard ([§5](#5-control-framework-mapping)).

**[F]** Repository state at `59cc2c3`, checked for this document:

- one Rust crate exists, `crates/verification`, whose crate root carries `#![forbid(unsafe_code)]`;
- its tests use runtime-checked `sqlx::query*` calls; no compile-time `query!` macro use was found;
- `tests/fixtures/vg04_rls.sql` and the RLS test under `crates/verification/tests/` exist;
- `.github/workflows/verification.yml` runs `cargo fmt --check`, `clippy -D warnings`, `cargo test --locked` and a `Cargo.lock` unchanged check with `contents: read`;
- `contracts/` contains only `legacy/`; there is no `contracts/openapi/` or `contracts/asyncapi/`;
- there is no domain module, migration directory, route, worker, outbox or event code.

Consequence: almost every gate below currently has nothing to check. That is the point of defining them now — each gate's trigger is the moment it stops being vacuous.

## 2. How a gate counts as "in place"

**[R]** A CI invariant gate is *in place* only when all of the following hold. The same rule is the evidence shape for every gate in [§3](#3-ci-invariant-gates), so it is stated once.

1. It runs in required CI on every pull request that can affect its surface, with `contents: read` only, under the repository's existing pinned-action and `--locked` posture.
2. It **fails closed**: a missing manifest, an unparseable input or a tool that did not run is a failure, never a skip.
3. It has a **seeded-violation fixture** — a deliberately broken input that the gate is shown to reject — kept in the repository and run in CI. A gate that has never been seen to fail has not been shown to work.
4. It passes on the commit that introduces the surface it guards. The first line of a new surface and the gate that guards it land in the same change; this is the rule [CONTRIBUTING.md](../../CONTRIBUTING.md#when-rust-code-arrives) already states for Rust code and CI.
5. Its result is recorded as an evidence record: AWBMS commit, workflow run, toolchain, command, actual result, artifact digest, and any check not run marked **NOT EXECUTED**. This follows the format of the records already under [`evidence/`](evidence/).
6. Any exception is an entry with owner, rationale, expiry and compensating control, not a silent allow-list line.

**[R]** Mechanism preference: where a check needs repository-specific logic, an in-repo Rust checker under `tools/` is preferred, following the precedent of `tools/vg15_independence.rs` **[F]**. Third-party tools named below are **candidates for evaluation, not selections**; their capabilities are unverified here and the change that adopts one must verify it and pin its version. If a non-Rust tool is adopted (a specification linter is the likeliest), that is a recorded decision, because "Rust-native" is the project's stated direction and a CI dependency on another ecosystem should be chosen rather than drifted into.

## 3. CI invariant gates

Each gate below gives the trigger, a mechanism candidate, a pass criterion, the evidence artifact, and the owning registered gate and existing issue. Pass criteria are written so that a program can decide them.

**Note on the owning issues.** Issues `#11`–`#24` are the P0–P2 implementation backlog. **[F]** They are deferred until the cross-specification Definition of Ready (gate 9) is approved: no production feature code precedes it (see the lifecycle statement in [docs/fastplaz-concepts.md](../fastplaz-concepts.md#cross-portfolio-engineering-invariants)). Naming an issue here assigns *where the gate's implementation belongs*; it does not start that issue. The mapping is by issue title and is **[R]**; an issue's owner may re-scope it.

**Note on supply chain.** `cargo audit`, `cargo deny`, `cargo vet`, SBOM and container scanning are `VG-13` and issue `#7` item P1.6. They are deliberately not repeated here.

### 3.1 Module DAG and ownership validation

- **Trigger:** before the second module is added to the workspace. With one module the graph cannot be wrong; with two it can.
- **Backing:** `AD-08` (explicit module registry, reviewed at compile time), `VG-02`; [§14 of the Blueprint](AWBMS-MASTER-BLUEPRINT.md#14-module-admission-rules) lists what the gate must reject.
- **Mechanism candidate:** an in-repo checker that reads the workspace dependency graph from `cargo metadata` and the module manifest declared by each module, then compares the two.
- **Pass criterion:** the module graph is acyclic; every inter-module dependency is declared in the manifest and in `Cargo.toml`, and vice versa; no route, permission, job or event is declared by two modules; dependency direction follows the documented layering; a module with no manifest fails.
- **Evidence artifact:** machine-readable graph and ownership report committed or attached to the CI run, plus the seeded-violation fixture (a cycle, an undeclared dependency, a duplicated route).
- **Owner:** `#24`, with `#11` for the manifest shape.

### 3.2 One mutable table has one owning module

- **Trigger:** before the first migration that creates a business table.
- **Backing:** `AD-09`; [§10.1 of the validation](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#101-one-table-one-writer-principle). The Blueprint rejects "undeclared table writes".
- **Mechanism candidate:** a table-ownership manifest (table to owning module), compared in CI against the tables present after running the migrations against an ephemeral PostgreSQL, plus a static scan of write statements in module code for tables the module does not own. SQL parsing candidates exist in the Rust ecosystem; evaluate before adopting.
- **Pass criterion:** every table created by migrations appears in exactly one module's ownership declaration; every `INSERT`/`UPDATE`/`DELETE`/`TRUNCATE` found in module code targets a table that module owns or an explicitly declared port; global (non-tenant) tables appear in an explicit global-table registry.
- **Evidence artifact:** ownership report, the catalog dump it was compared against, seeded-violation fixture (a module writing another module's table).
- **Owner:** `#24`, with `#15` (repository ports) as the legitimate cross-module path.
- **Limit [R]:** a static scan cannot see dynamically built SQL. The criterion therefore also requires that SQL be confined to owned repository code, per the code policies in [§28](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#28-maintainability-and-code-quality-gates); construction of SQL outside it is itself a failure.

### 3.3 Migration immutability and checksum

- **Trigger:** before the first migration is applied to any database other than a disposable test database.
- **Backing:** `AD-25`, `VG-10`; [§38](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#38-migration-ledger). The validation says to verify against the gate rather than rely on a framework default.
- **Mechanism candidate:** two independent checks. (a) A CI diff against the merge base that fails if any previously merged migration file is modified, renamed or deleted. (b) A runner-startup check that recomputes SHA-256 of each applied migration and compares it with the ledger, plus a test that two concurrent runners cannot both apply.
- **Pass criterion:** zero modified or removed merged migrations; ledger checksums equal recomputed checksums; a deliberately edited applied migration makes startup fail; two concurrent runners result in exactly one applying each migration.
- **Evidence artifact:** CI log of both checks, ledger snapshot, seeded-violation fixture (an edited historical migration).
- **Owner:** no issue is dedicated to this. Closest by title are `#22` (rollback test matrix) and `#21` (quality gates). **[R]** Open a dedicated issue when the first migration directory is introduced.

### 3.4 Tenant and RLS coverage, and runtime-role checks

- **Trigger:** before the first tenant-scoped table.
- **Backing:** `AD-10`, `VG-06`; [§11.1–§11.3](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#11-postgresql-architecture). The Stage 1 spike already demonstrates one FORCE-RLS denial against a non-superuser role ([evidence record](evidence/VG-03-VG-04-2026-08-29.md)); this gate generalises that from one probe table to every table.
- **Mechanism candidate:** catalog introspection of `pg_class`, `pg_policy` and `pg_roles` after migrations, run as an integration test against real PostgreSQL, plus cross-tenant tests executed through the runtime role.
- **Pass criterion:** every tenant-scoped table has row security enabled **and** forced and at least one policy; every table lacking RLS is in the global-table registry with its forbidden privileges asserted; the application role is non-superuser, `NOBYPASSRLS`, not the owner of any business table, and has no DDL; the worker role's privileges are a strict subset of what its workload needs; a tenant-A transaction cannot read, update, delete or insert for tenant B; tenant context is transaction-local and does not leak across pooled connections.
- **Evidence artifact:** catalog report, role-attribute assertions, cross-tenant denial results, seeded-violation fixture (a table without a policy; a role with `BYPASSRLS`).
- **Owner:** `#12` (tenant context and transaction wrapper), `#22` (security/RLS test matrix).

### 3.5 Authorization chokepoint and default-deny

- **Trigger:** before the first protected route.
- **Backing:** `VG-07`; [§13](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#13-authorization-architecture); [Blueprint §13](AWBMS-MASTER-BLUEPRINT.md#13-authentication-and-authorization).
- **Mechanism candidate:** a route table built at test time from the real router, joined to each route's declared permission; a static check that no handler reaches a repository without the authorization context; decision-vector tests.
- **Pass criterion:** every route is either declared public with a recorded reason or carries a permission binding; every declared permission is bound to at least one route or action; an operation with no policy denies; a deny overrides an allow; SoD and module-enablement conflicts produce deny; a handler added without a binding fails CI.
- **Evidence artifact:** route-to-permission mapping, allow/deny vector results, seeded-violation fixture (an unbound handler).
- **Owner:** `#13`, `#22`.

### 3.6 OpenAPI and AsyncAPI validation and breaking-change diff

- **Trigger:** before the first implemented route (OpenAPI) and before the first published event (AsyncAPI).
- **Backing:** `AD-19`, `VG-08`; [§21](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#21-api-contracts) already lists the required checks. This gate is the mechanism for them, not a new list.
- **Mechanism candidate:** a specification validator and a breaking-change differ against the merge base, for each of OpenAPI and AsyncAPI, plus an in-repo checker for the AWBMS-specific semantics (route-set equality with the implemented router, unique `operationId`, error envelope per `AD-21`). Adopting any of these is subject to the tooling note in [§2](#2-how-a-gate-counts-as-in-place).
- **Pass criterion:** documents validate against their schema; implemented routes equal specified routes except those explicitly marked planned; `operationId` is unique; every error response uses the standard envelope; no change to a committed or consumed operation is breaking unless it carries an approved, versioned exception; events declare owner, version and privacy classification.
- **Evidence artifact:** validator output, diff report against the merge base, route-set comparison, seeded-violation fixture (a removed required field; an unspecified route).
- **Owner:** `#23`.

### 3.7 AWCMS-Astro consumer contract checks

- **Trigger:** before AWBMS implements any endpoint that AWCMS-Astro consumes, and before any claim of compatibility for it.
- **Backing:** `VG-01` (artifact-to-test usage mapping), `VG-05`; [Blueprint §18](AWBMS-MASTER-BLUEPRINT.md#18-awcms-astro-compatibility-priority); the consumer-contract row of the [invariant traceability matrix](AWBMS-INVARIANT-TRACEABILITY.md).
- **Mechanism candidate:** the frozen consumed-surface inventory under `contracts/legacy/awcms/` replayed against the AWBMS router, with an orphan-fixture check.
- **Pass criterion:** every consumed operation exists in the AWBMS contract or is on the approved-deviation register ([§4.1](#41-vg-05--semantic-parity-and-security-corpus)); every frozen fixture is consumed by at least one test and an orphan fixture fails CI; anonymous and machine-credential execution classes behave as recorded; tenant resolution is credential-bound and a mismatch fails closed.
- **Evidence artifact:** consumed-surface-to-test map, replay results, seeded-violation fixture.
- **Owner:** `#23` for the contract side; the fixture and baseline-delta work is `#7` P0 items 3 and 4 and `#3`, and is not duplicated here.
- **Boundary:** the check consumes frozen files. It must not read the AWCMS or AWCMS-Astro source trees at CI time, which would breach `VG-15`.

### 3.8 `cargo sqlx prepare --check` (conditional)

- **Trigger:** before the first use of a compile-time-checked SQL macro. **[F]** None exists at `59cc2c3`.
- **Backing:** `AD-06`; listed in the [§28 command set](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#28-maintainability-and-code-quality-gates).
- **Condition:** this gate is **conditional on adopting compile-time SQL**, and whether to adopt it is not decided in this repository. **[R]** Record the adoption decision when the first module needs it. If compile-time SQL is not adopted, the gate is recorded as **not applicable**, with the decision as the reason; it is not left as an unexplained gap.
- **Mechanism candidate:** `cargo sqlx prepare --check` with committed offline metadata and offline mode enabled in CI.
- **Pass criterion:** the command exits zero and produces no diff against committed metadata; metadata is regenerated in the same change as any query change; CI builds without a live database for compilation, while integration tests still use real PostgreSQL.
- **Evidence artifact:** command log and the digest of the committed metadata.
- **Owner:** `#21`.

### 3.9 Unsafe-code policy

- **Trigger:** in force now for the existing crate (**[F]** `#![forbid(unsafe_code)]` in `crates/verification`); extended to every new crate at creation.
- **Backing:** `AD-29`; [§26.1](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#261-unsafe-code-policy).
- **Mechanism candidate:** an in-repo check that each AhliWeb-owned crate root carries `#![forbid(unsafe_code)]`, or appears in an exception list whose entry points to an accepted ADR.
- **Pass criterion:** zero first-party crates without the attribute unless excepted; each exception names an ADR, the isolated module, the documented safety invariants, the dedicated tests, and a security-review record; fuzz or property tests exist where `AD-29` calls for them. Unsafe in third-party dependencies is out of scope for this gate and belongs to `VG-13`.
- **Evidence artifact:** per-crate attribute report, exception list, seeded-violation fixture (a new crate without the attribute).
- **Owner:** `#21`.

### 3.10 Idempotency, outbox, inbox and replay

- **Trigger:** before the first retry-prone or high-risk mutation (idempotency); before the first domain event (outbox); before the first consumer of an event or job (inbox and replay).
- **Backing:** `AD-13`, `AD-23`, `VG-11`; [§15](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#15-transactional-events), [§31](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#31-idempotency).
- **Mechanism candidate:** integration tests against real PostgreSQL with parallel clients; property-based tests for the key and fingerprint logic where useful.
- **Pass criterion:**
  - N parallel requests with one idempotency key commit exactly one business effect and one expected event sequence, and the others replay the stored response;
  - the same key with a different request fingerprint, or from a different principal or tenant, is rejected rather than replayed;
  - an in-flight duplicate yields the defined conflict response;
  - a business transaction that rolls back leaves no outbox row, and one that commits always has one;
  - a provider call never occurs inside the business transaction (see [`AD-15`](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#appendix-c--decision-register));
  - concurrent workers never both claim one job or dispatch one event;
  - replaying an event or job a second time produces no additional business effect.
- **Inbox [R]:** the validation specifies an outbox but no inbox. Consumer-side de-duplication is **proposed** as the mechanism that makes the last criterion true; whether it is adopted is a decision for the first consumer ([§7](#7-proposed-register-changes-not-registered-here)).
- **Evidence artifact:** `tests/concurrency/` results (the path named by `VG-11`), seeded-violation fixture (a handler that skips the key check).
- **Owner:** `#19` (transactional audit and outbox durability), `#22` (transaction and rollback matrix), `#20` (first vertical slice, the first real user of it).

### 3.11 Bounded concurrency, timeout, cancellation and pool limits

- **Trigger:** before the first externally reachable listener beyond the verification spike, before the first worker, and before the first outbound HTTP call.
- **Backing:** `AD-04`, `AD-15`, `AD-16`; [§30](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#30-backpressure-and-overload-behavior) and [§25](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#25-postgresql-connection-scaling). No registered gate names this surface; the nearest are `VG-04` (graceful shutdown, already PASS for the spike) and `VG-11`.
- **Mechanism candidate:** integration tests that deliberately saturate each bound and assert the specified degradation.
- **Pass criterion:**
  - exhausting the database pool yields a bounded, retryable error within the configured timeout, not an unbounded wait;
  - a request exceeding its timeout is cancelled, and afterwards no transaction remains open and no lock is held, observable in PostgreSQL's activity view;
  - a client disconnect cancels in-flight work in the same way;
  - a request body over the limit is rejected before it is buffered in full;
  - worker concurrency and queue claim batch size never exceed their configured caps;
  - graceful shutdown stops new claims and drains in-flight work;
  - every outbound call declares timeout, body limit, redirect and retry policy, and a check fails if one does not.
- **Evidence artifact:** saturation-test results with the configured limits recorded beside the observed behaviour, seeded-violation fixture (a handler without a timeout layer).
- **Owner:** `#18` (request identity, timeout, body-limit and tracing middleware), with `#17` for the storage adapter's outbound bounds.
- **Limit:** the gate asserts that a limit exists and holds. It does **not** choose the values, which depend on measured capacity ([§4.2](#42-vg-09--performance-evidence)).

## 4. Production evidence shapes

These sections specify what each piece of evidence must contain so that it is measurable and reproducible. They set no thresholds.

**Where numeric values come from.** **[O]** Absolute SLO values are an owner decision tied to `OD-06`, and the [Blueprint's baseline-first policy](AWBMS-MASTER-BLUEPRINT.md#11-performance-policy--od-04-decision) says they are set in the Stage 2 PRD and at the Definition of Ready (gate 9). Recovery and retention values likewise are owner and legal decisions. Wherever a value would appear below, the template carries a placeholder and an owner, never a number. No figure in this document is a target, a baseline or a result.

**[R]** Every evidence record, for every gate below, carries the common header from [§2](#2-how-a-gate-counts-as-in-place) item 5.

### 4.1 `VG-05` — semantic parity and security corpus

`VG-05` passes when the committed compatibility scope has no unexplained semantic difference ([gate register](AWBMS-STAGE-1-GATES.md#gate-register)). The harness design and the comparison dimensions are [§39 of the validation](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#39-awcms--awbms-parity-harness) (`AD-26`); this section adds the *evidence shape* only.

**Corpus case record. [R]** Each case carries:

- a case key, unique within the corpus;
- the capability it exercises and the consumer it matters to (for example AWCMS-Astro);
- provenance of the source fixture: path under `contracts/legacy/awcms/`, SHA-256, source commit;
- the request, and the expected outcome across the comparison dimensions listed in §39;
- the execution class (anonymous, machine credential, authenticated human);
- a classification: functional, security, or both.

**Security corpus. [R]** A named subset that must exist for every in-scope capability, covering at least: cross-tenant access attempts, allow/deny authorization vectors, enumeration-neutral responses, anonymous versus machine-credential behaviour, replay and duplicate submission, and tenant-resolution mismatch. A capability with no security cases is not in the passing scope; it is out of scope until it has them.

**Normalisation allow-list. [R]** The list of fields normalised before comparison is a committed, versioned file. Each entry names the field, the reason it is legitimately nondeterministic, and a reviewer. A field not on the list is compared exactly. Adding an entry is a reviewed change, not a harness option.

**Approved-deviation register. [R]** A committed file. Each row carries:

- the deviation key and the case keys it affects;
- the AWCMS behaviour (pointing at the frozen fixture) and the AWBMS behaviour;
- the reason: security correction, dropped bug-compatibility, intentional redesign, or consumer-confirmed change;
- the consumer impact, and the consumer sign-off where a consumer is affected;
- the approver, approval date, and a review or expiry date;
- the test that pins the new behaviour, so the deviation cannot widen unnoticed.

A deviation that weakens an authorization or isolation outcome is not approvable through this register; it requires a recorded decision.

**Run record. [R]** AWBMS commit, pinned AWCMS and AWCMS-Astro SHAs, fixture manifest digest, toolchain, harness version. Summary counts: cases run, passed, passed under an approved deviation, unexplained. The pass rule is that the unexplained count is zero for the committed scope. A case that could not run is reported as not run, never as passed.

**Evidence artifact:** the harness output under `tests/parity/` (the path named in Appendix E), plus the two committed files above.
**Depends on:** `VG-01`, `OD-03` (what the compatibility scope is). This section does not define the scope.

### 4.2 `VG-09` — performance evidence

`VG-09` verifies that measurements were taken on identical infrastructure, dataset and workload ([Appendix E](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#appendix-e--verification-gate-register)). The scenarios and the metric list are [§23.1](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#231-required-benchmark-corpus); the acceptance policy is [Blueprint §11](AWBMS-MASTER-BLUEPRINT.md#11-performance-policy--od-04-decision). This section is the measurement record they require. **[F]** No AWBMS benchmark has been designed or run, and none is claimed here ([§23](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#23-performance-expectations)).

**Precondition: correctness first. [R]** A benchmark run is valid only if `VG-05`, `VG-06`, `VG-07` and `VG-11` checks pass on the same commit for the scenarios being measured. A run on a commit that fails them is recorded as invalid and is not compared with anything ([§48](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#48-performance-validation-gates)).

**Environment manifest. [R]** Machine-generated, committed with the result, covering:

- host class: CPU model and core count, memory, storage type, kernel and OS, virtualisation;
- the topology: where the load generator, application and PostgreSQL run, and the network between them;
- container image digests and the pinned toolchain; build profile and flags;
- PostgreSQL version and every non-default setting, as values;
- every pool size and limit in effect: application pool, worker pool, any connection pooler, runtime worker-thread count, request concurrency limits;
- the load generator and its version.

**Dataset. [R]** Synthetic and free of personal data. Recorded: generator version and seed, per-table row counts, tenant count and the size distribution across tenants, and a digest of the loaded dataset. A dataset that cannot be regenerated bit-identically from the recorded seed is not a valid dataset.

**Workload. [R]** Per scenario: request mix, concurrency levels, warm-up period, measured duration, open- or closed-loop model, and number of repetitions. The values chosen are recorded with the result; this document does not set them.

**Measurements. [R]** Per scenario, per concurrency level: throughput; latency p50, p95 and p99 computed from recorded per-request latencies, with the method for handling coordinated omission stated; error rate by class; CPU per request; memory footprint; database connections in use and pool wait; query time and lock wait; event lag and job throughput where the scenario involves them. Run-to-run variance across repetitions is reported. A result whose variance the owner has not accepted is not a baseline.

**Query plans. [R]** For each query a scenario depends on, the actual execution plan with buffer statistics, its plan digest, and a statement-statistics snapshot. A changed plan digest between runs is reported next to the latency change.

**Comparison. [R]** Against the last approved AWBMS baseline under identical conditions, using the regression-review policy in Blueprint §11. When AWCMS is compared, it runs on identical infrastructure, dataset and workload or the comparison is not made ([§23.1](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#231-required-benchmark-corpus)).

**Placeholders (not numbers).**

| Item | Value | Owner and venue |
|---|---|---|
| Absolute latency, throughput and availability targets per capability | `<owner-set>` | owner, Stage 2 PRD and gate 9 (`OD-06`) |
| Repetition count; acceptable run-to-run variance | `<owner-set>` | owner, with the first baseline |
| Concurrency levels and measured duration | `<owner-set>` | owner, with the first baseline |

**Evidence artifact:** a run bundle holding the manifest, dataset digest, raw latency histograms, resource time series, plans and the summary, archived with a checksum and referenced from an evidence record.

### 4.3 `VG-12` — recovery and rollback evidence

`VG-12` passes when restore and routing and worker rollback complete within *approved* recovery criteria ([gate register](AWBMS-STAGE-1-GATES.md#gate-register)), rehearsed **before** the wave deploys (`AD-28`). The minimum rollback preparation is [§42](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#42-rollback-model), the worker protocol is [§41](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#41-worker-cutover-safety), and the per-wave requirements are [Blueprint §22](AWBMS-MASTER-BLUEPRINT.md#22-deployment-and-recovery). This section is the shape of the rehearsal record.

**Recovery objectives per capability. [O]** The objectives are owner decisions, tied to `OD-06`. The template is a table the Blueprint or PRD must fill:

| Capability | RTO | RPO | Basis for the value | Approver | Date |
|---|---|---|---|---|---|
| `<capability>` | `<owner-set>` | `<owner-set>` | `<owner-set>` | `<owner-set>` | `<owner-set>` |

A rehearsal is judged against the row for the capability it covers. A capability with no row cannot pass `VG-12`.

**Backup evidence. [R]** What is backed up, the schedule, the retention, where copies are stored and whether they sit in a separate failure domain, encryption and key custody, and the monitoring that detects a missed backup. The schedule is derived from the capability's RPO, not chosen independently of it.

**Restore rehearsal. [R]** Against a production-like snapshot (`AS-03` records that availability of one is unconfirmed). The record states: snapshot identity, start and end timestamps, each step with its actual duration, the **measured** restore time against RTO, and the **measured** data-loss window against RPO. Integrity checks after restore: ledger checksums ([§3.3](#33-migration-immutability-and-checksum)), row-count and reconciliation queries, roles and grants recreated, the RLS and role assertions of [§3.4](#34-tenant-and-rls-coverage-and-runtime-role-checks) re-run on the restored database, and an application smoke test.

**Rollback rehearsals. [R]** Each is a separate record, for each wave:

- **Migration rollback.** The expand-contract state of the schema ([§40.1](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#401-expand-contract-rule)), and, per migration, whether the recovery path is restore or a forward fix, with the reason. Evidence that the previous implementation still runs against the post-migration schema where coexistence applies.
- **Routing rollback.** The traffic-reversal steps and their measured duration, and a check that traffic reached the previous owner.
- **Worker and outbox ownership rollback.** The stop-claims, drain, transfer and observe sequence of §41, with evidence from claim records that no job or event was consumed by two owners at any moment, and the in-flight and queue state at hand-back.
- **Reconciliation.** After rollback, a comparison of what was written during the window against both sides, covering lost writes, duplicated writes and duplicated or missing events, with idempotency keys used to explain differences. Any unexplained difference blocks acceptance.

**Verification after rollback. [R]** The §42 post-rollback checklist executed and recorded item by item. The Go/No-Go list in [§54](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#54-gono-go-criteria) names failed backup and restore tests and an unverified rollback path as No-Go; the rehearsal record is the evidence that those conditions are clear.

### 4.4 Observability evidence

Telemetry content is already specified: [§22](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#22-observability), [§56](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#56-monitoring-baseline) and [Blueprint §21](AWBMS-MASTER-BLUEPRINT.md#21-observability) (`AD-20`). This section covers the evidence that it works.

**[R]** Required evidence:

- **Redaction test.** Canary secrets and canary personal data are pushed through request, job and event paths; the test asserts none appears in any log, trace or metric output.
- **Correlation test.** One request identifier is followed through request, outbox row, job and event, and the test asserts the correlation holds at every boundary.
- **Series presence.** A test scrapes the metrics endpoint and asserts every series named in §56 exists with its expected labels, so a renamed or dropped metric fails CI rather than silently blinding an alert.
- **Alerts as code.** Alert rules are version-controlled and each is exercised with a synthetic signal that makes it fire; every alert links to a runbook ([§4.5](#45-incident-response-evidence)).
- **Audit versus operational logs.** The two are separate stores, per the [traceability matrix](AWBMS-INVARIANT-TRACEABILITY.md); retention and access evidence is kept separately for each.
- **Required-monitoring-present check** for the Go decision ([§54](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#54-gono-go-criteria)): a record that each monitoring item required for the wave is live in the target environment.

Alert thresholds and paging policy are `<owner-set>`.

### 4.5 Incident-response evidence

The Blueprint requires "incident and vulnerability readiness" ([Blueprint §19](AWBMS-MASTER-BLUEPRINT.md#19-security-and-privacy-baseline)) without defining its evidence.

**[R]** Before the first production cutover:

- named roles for incident command, technical response and communications, and an escalation path, as `<owner-set>`;
- a severity scale and response-time expectations, `<owner-set>`;
- runbooks, at minimum for: suspected cross-tenant exposure, credential or session compromise, a failed or destructive migration, worker or event double-processing, provider outage, and a disclosed dependency vulnerability;
- at least one exercise of a runbook (tabletop or technical), recorded with date, participants, scenario, the time taken to detect and to contain as observed, and the actions arising;
- a post-incident review template and the rule that each review produces tracked actions;
- a vulnerability-intake channel and handling process, consistent with the `SECURITY.md` work in issue `#7` item P1.7.

**Notification obligations** to regulators and data subjects, including deadlines, are determined by legal review of the applicable law. This document asserts none.

### 4.6 Retention, deletion and legal-hold evidence

The descriptors every data-owning module declares are [§33](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#33-data-lifecycle-and-privacy); the gate is `VG-14`; the Blueprint's declaration list is in [Blueprint §19](AWBMS-MASTER-BLUEPRINT.md#19-security-and-privacy-baseline).

**[R]** Evidence per data-owning module:

- the descriptor file, and a coverage check that fails CI when a data-owning module has none (the `VG-14` executable check);
- a test that declarations match behaviour: data in a declared-erasable table is actually erased or anonymised by the owning module's purge path, including derived copies such as search indexes, caches and exports;
- a legal-hold test that an ordinary purge job cannot remove held data, and that the hold itself is audited;
- a data-subject export test returning the declared categories and no others, tenant-scoped;
- a restore-interaction test: after a restore from backup, erased data is not silently reintroduced — either the erasure ledger is re-applied or the gap is recorded as a known limit with its owner;
- retention periods as `<owner-set>` by the business owner after legal review, recorded per category with the basis. The audit log's retention is a separate declaration from operational logs.

### 4.7 Production-validation evidence

What to validate after every wave is [§55](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#55-production-validation); the Go and No-Go lists are [§54](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#54-gono-go-criteria). This section is the record's shape.

**[R]** Per wave, one acceptance record:

- wave identity, AWBMS commit, image digest, environment, and operator;
- each §55 check as a row: the procedure or test it ran, the actual result, timestamp, and where the raw output is stored;
- the malicious cross-tenant tests run against the target environment using synthetic tenants, with results;
- the observation window and the signals watched during it — the length is `<owner-set>`;
- the Go/No-Go checklist with each item marked met, not met or not applicable with a reason;
- the references to the rehearsal records of [§4.3](#43-vg-12--recovery-and-rollback-evidence) for this wave;
- the sign-off, by an accountable approver distinct from the operator.

An HTTP 200 from the health endpoint is not an entry in this record; §55 says a wave is not accepted on that alone.

## 5. Control framework mapping

> **No compliance claim.** Nothing in this document, and no passing gate, claims or implies certification, attestation, conformance or regulatory compliance with any standard or law named here. The standards are used as control frameworks that organise what to evidence. This follows [§35 of the validation](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#35-standards-and-regulatory-mapping) and [Blueprint §20](AWBMS-MASTER-BLUEPRINT.md#20-standards-and-regulatory-mapping), which say the same and carry the concern-level table. That table is not copied here; this section adds the other axis, from evidence family to framework.
>
> This document does not quote or paraphrase clause-level content of any standard. Clause-level mapping requires the licensed edition, and an applicable-edition and applicability review by the accountable owner. For the Indonesian instruments, applicability to a given deployment requires legal review, and compliance requires operational and organisational evidence in addition to software controls, as §35 states.

**Traceability chain. [R]** Every control that this document specifies is to be traceable as:

```text
Requirement/Risk -> Control -> Implementation -> Test -> Evidence -> SOP/Runbook -> Monitoring
```

The existing [invariant traceability matrix](AWBMS-INVARIANT-TRACEABILITY.md) covers the first five links for the AWCMS lessons. The last two links, runbook and monitoring, are what [§4.4](#44-observability-evidence) and [§4.5](#45-incident-response-evidence) add.

**Evidence family to framework. [R]** A row means "this evidence is the kind a reviewer reading that framework would look for", not that the framework is satisfied.

| Evidence family | Frameworks it is read against |
|---|---|
| Module, ownership and authorization gates ([§3.1](#31-module-dag-and-ownership-validation), [§3.2](#32-one-mutable-table-has-one-owning-module), [§3.5](#35-authorization-chokepoint-and-default-deny)) | ISO/IEC 27001 and 27002 (access control, secure development); ISO/IEC 27034 (application security); OWASP ASVS and OWASP API Security; ISO/IEC 15408 concepts (explicit security objectives with independently checkable assurance evidence) |
| Tenant and RLS gate ([§3.4](#34-tenant-and-rls-coverage-and-runtime-role-checks)) | ISO/IEC 27001 and 27002; ISO/IEC 27017 and 27018 where cloud infrastructure and personal data in the cloud are involved; ISO/IEC 27034; OWASP ASVS; ISO/IEC 15408 concepts |
| Migration, contract and consumer gates ([§3.3](#33-migration-immutability-and-checksum), [§3.6](#36-openapi-and-asyncapi-validation-and-breaking-change-diff), [§3.7](#37-awcms-astro-consumer-contract-checks)) | ISO/IEC 27001 and 27002 (change and configuration management); ISO/IEC 20000-1 (change and release management); NIST SSDF |
| Idempotency, concurrency and bounds ([§3.10](#310-idempotency-outbox-inbox-and-replay), [§3.11](#311-bounded-concurrency-timeout-cancellation-and-pool-limits)) | ISO/IEC 25010 (reliability and performance-efficiency characteristics); OWASP API Security; ISO/IEC 20000-1 |
| Unsafe policy and `sqlx` metadata ([§3.8](#38-cargo-sqlx-prepare---check-conditional), [§3.9](#39-unsafe-code-policy)) | NIST SSDF; ISO/IEC 27034; ISO/IEC 25010 (maintainability, security) |
| Parity corpus ([§4.1](#41-vg-05--semantic-parity-and-security-corpus)) | ISO/IEC 25010 (functional suitability); OWASP ASVS; ISO/IEC 15408 concepts |
| Performance evidence ([§4.2](#42-vg-09--performance-evidence)) | ISO/IEC 25010 (performance efficiency), as measurable quality attributes |
| Recovery and rollback ([§4.3](#43-vg-12--recovery-and-rollback-evidence)) | ISO 22301 (continuity, tested recovery); ISO/IEC 20000-1; ISO/IEC 27001 and 27002; NIST CSF (recover); CIS Controls |
| Observability and incident response ([§4.4](#44-observability-evidence), [§4.5](#45-incident-response-evidence)) | ISO/IEC 27001 and 27002 (logging, incident management); ISO/IEC 20000-1; NIST CSF (detect, respond); CIS Controls |
| Retention, deletion, legal hold, export ([§4.6](#46-retention-deletion-and-legal-hold-evidence)) | ISO/IEC 27701 (privacy information management); ISO/IEC 27018; ISO/IEC 27005 (risk treatment and exception ownership); UU No. 27 Tahun 2022 (PDP) |
| Production validation and Go/No-Go ([§4.7](#47-production-validation-evidence)) | ISO/IEC 20000-1; ISO/IEC 27001; NIST CSF; CIS Controls; PP No. 71 Tahun 2019 (PSTE) |
| Exceptions: owner, rationale, expiry, compensating control ([§2](#2-how-a-gate-counts-as-in-place) item 6) | ISO/IEC 27005 |
| Supply chain (`VG-13`, not specified here) | NIST SSDF; CIS Controls; OWASP |

**Indonesian instruments. [R]** UU No. 27 Tahun 2022 (PDP), PP No. 71 Tahun 2019 (PSTE) and the electronic-information law (UU ITE, recorded in the validation as amended by UU No. 1 Tahun 2024) are the baseline named in [§35](AWBMS-RUST-ARCHITECTURE-VALIDATION.md#35-standards-and-regulatory-mapping). Sector-specific rules are added when a deployment enters a regulated domain. Which obligations apply to a deployment, and what evidence they require, is a legal determination that has not been made; the rows above indicate only where the technical evidence of this document would be an input to it.

## 6. Relationship to existing work

| Existing item | Relationship |
|---|---|
| `#7` P0 items 1–5 (write-token split, gate-status drift, baseline-delta, executable `VG-01`, stacked drafts) | Not duplicated. [§3.7](#37-awcms-astro-consumer-contract-checks) depends on them |
| `#7` P1 items 6–8 (supply chain, governance files, `main` protection) | Not duplicated. `VG-13` and the `SECURITY.md` item are referenced only |
| `#11`–`#24` | Named as owners in [§3](#3-ci-invariant-gates); all deferred until gate 9 |
| [Blueprint §11](AWBMS-MASTER-BLUEPRINT.md#11-performance-policy--od-04-decision) | Supplies the regression-review policy that [§4.2](#42-vg-09--performance-evidence) links rather than restates |
| [`evidence/`](evidence/) | Format precedent for the records this document requires |

## 7. Proposed register changes (not registered here)

**[R]** The following would need a register row if adopted. None is registered by this document, and none has been allocated an identifier. Each is a recommendation for the accountable owner.

1. A gate for observability and incident-response readiness, covering [§4.4](#44-observability-evidence) and [§4.5](#45-incident-response-evidence). No registered gate covers either today.
2. A gate for the per-wave production acceptance record ([§4.7](#47-production-validation-evidence)), distinct from `VG-12`.
3. A gate for bounded concurrency, timeout and cancellation ([§3.11](#311-bounded-concurrency-timeout-cancellation-and-pool-limits)), which no registered gate names.
4. A decision on whether compile-time SQL is adopted ([§3.8](#38-cargo-sqlx-prepare---check-conditional)).
5. A decision on whether an inbox is adopted for consumer-side de-duplication ([§3.10](#310-idempotency-outbox-inbox-and-replay)).
6. A decision on whether non-Rust specification tooling is acceptable in CI ([§2](#2-how-a-gate-counts-as-in-place), [§3.6](#36-openapi-and-asyncapi-validation-and-breaking-change-diff)).
7. A statement of who owns recovery objectives and retention periods. The recovery objectives are part of `OD-06`; retention periods are not covered by any open decision and may warrant one.

## 8. Limits of this document

- It specifies evidence; it produces none. No command in it has been run, and no gate here is passed.
- Tool names are candidates. Their behaviour has not been verified against this repository.
- Every placeholder marked `<owner-set>` is an unanswered question, not a deferred detail.
- The mapping in [§5](#5-control-framework-mapping) is organisational. It is not an assessment against any standard and must not be quoted as one.
