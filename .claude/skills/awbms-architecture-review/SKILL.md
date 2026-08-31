---
name: awbms-architecture-review
description: Evaluate an AWBMS design proposal, dependency choice, or implementation plan against the binding decisions (AD-01…AD-32), open decisions (OD-), and verification gates (VG-) in the architecture validation. Use when someone proposes a crate, a schema, an endpoint, a module boundary, a caching or queueing approach, a migration step, or asks "does this fit the AWBMS architecture" / "can we use X".
---

# AWBMS — Architecture review

Review a proposal against what AWBMS has already decided, before it becomes
code that is expensive to unwind.

Source of truth: `docs/architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md`,
Appendix C (decisions), Appendix D (open), Appendix E (gates).

## First, the thing reviewers get wrong

**The architecture is unvalidated.** No AWBMS code, test, benchmark or
deployment exists. Every AWCMS fact is a `[C]` claim nobody has re-checked. No
verification gate has been discharged.

So a proposal can conflict with a decision **and still be right** — the
decision may be the thing that is wrong. Your job is to surface the conflict
and route it to a decision, not to reject the proposal because a table says so.
What is never acceptable is a conflict that goes unrecorded.

## The binding decisions

| Area | ID |
|---|---|
| New platform, not a translation of AWCMS | `AD-01` |
| Rust 2024, pinned stable toolchain | `AD-02` |
| Tokio async runtime | `AD-03` |
| Axum + Tower HTTP foundation | `AD-04` |
| PostgreSQL is the authoritative system of record | `AD-05` |
| SQLx as the primary data-access layer | `AD-06` |
| Modular monolith, microservice-extractable | `AD-07` |
| Explicit compile-time module registry | `AD-08` |
| One-table-one-writer ownership | `AD-09` |
| RBAC + ABAC + SoD + FORCE RLS | `AD-10` |
| Opaque server-side sessions for humans | `AD-11` |
| Argon2id, opportunistic rehash on login | `AD-12` |
| Transactional PostgreSQL outbox | `AD-13` |
| PostgreSQL-backed job queues | `AD-14` |
| No external provider I/O inside business transactions | `AD-15` |
| Reqwest + rustls, explicit features | `AD-16` |
| Storage port, not provider coupling | `AD-17` |
| Redis/Valkey optional, never a correctness dependency | `AD-18` |
| OpenAPI + AsyncAPI contract-first | `AD-19` |
| `tracing` + OpenTelemetry/OTLP | `AD-20` |
| Stable error taxonomy and `ApiError` envelope | `AD-21` |
| Pinned dependencies, supply-chain gates | `AD-22` |
| Idempotency as a platform capability | `AD-23` |
| AWCMS compatibility layer, no repo dependency | `AD-24` |
| Migration ledger with checksums and locking | `AD-25` |
| Parity harness against frozen fixtures | `AD-26` |
| Strangler migration, single writer per capability | `AD-27` |
| Rollback defined before every wave | `AD-28` |
| `#![forbid(unsafe_code)]` by default | `AD-29` |
| Docker + Coolify + Traefik + Cloudflare | `AD-30` |
| `vX.Y.Z` versioning with changesets | `AD-31` |
| Executable documentation gate | `AD-32` |

Read the register row before citing one. The row carries the rationale and the
gate; this table is only an index.

## Review procedure

### 1. Identify which decisions the proposal touches

Search the validation for the concern, not the technology. A proposal to "add
Redis for sessions" touches `AD-11` (sessions), `AD-18` (Redis optional) and
`AD-05` (PostgreSQL authoritative) — three decisions, only one of which
mentions Redis.

### 2. Classify the proposal

| Verdict | Meaning | Action |
|---|---|---|
| **Consistent** | Implements a decision as written | Approve; note the ID |
| **Fills a gap** | The decisions are silent | Approve, and record a new decision — silence is not permission |
| **Closes an open decision** | Resolves an `OD-` | Approve *only* with the decision text written; an `OD-` must not be resolved implicitly in code |
| **Conflicts** | Contradicts a binding decision | Do not silently accept. Either revise, or write a successor decision with rationale |
| **Discharges a gate** | Produces evidence for a `VG-` | Verify the artifact actually exists; a claim is not evidence |

### 3. Run the invariant checks

These are the ones that produce security or correctness failures, drawn from
§10–§33. Check every proposal against them:

- **Tenant isolation** — application authorization **and** an explicit tenant
  predicate **and** FORCE RLS. All three. RLS is a backstop, not a substitute
  for the predicate (§11.2).
- **Authorization chokepoint** — one service, default deny, deny overrides
  allow, missing policy data fails closed. No scattered `is_admin` (§13).
- **Table ownership** — one module owns each mutable table. Cross-module
  mutation goes through ports, capabilities, read models or events (§10.1).
- **Transaction order** — `BEGIN` → tenant context → authorization → mutation →
  audit → outbox → `COMMIT` (§11.3).
- **No provider I/O in a business transaction** — enqueue, commit, then let a
  worker call out (§17).
- **Cache is never authorization truth** — the system stays correct after full
  cache eviction (§20).
- **Idempotency** for retry-prone or high-risk mutations (§31).
- **Single runtime owner** for any job or queue during migration (§16, §41).
- **Rate limits target the resource abused**, not only the requester — the
  newsletter case (`C-07`) is why (§32).
- **Privacy declared at module admission** — categories, retention, erasure,
  legal hold (§33).

### 4. Check the gate status before trusting a rationale

`AD-04` (Axum) and `AD-06` (SQLx) rest on reasoning, not measurement, and are
gated by `VG-04` — which is open. A proposal that challenges them on
*measured* grounds is stronger than the decision it challenges. Say so.

## Common proposals, and the fast answer

| Proposal | Answer |
|---|---|
| "Use Redis for sessions / permissions / rate limits" | Sessions and permissions: no — `AD-05`, `AD-11`, `AD-18`. Distributed rate limiting: allowed *with measured need*, and correctness must survive eviction |
| "Add an ORM / SeaORM / Diesel" | Conflicts with `AD-06`. §6.3 lists the PostgreSQL behaviour that must stay visible: RLS, `SET LOCAL`, advisory locks, `SKIP LOCKED`, CTEs, JSONB |
| "Add Kafka / NATS / RabbitMQ" | Not required for v1 (`AD-13`). A broker may be an adapter *behind* the outbox, never a replacement for it |
| "Derive OpenAPI from Rust macros" | Conflicts with `AD-19` during migration — contracts are reviewed artifacts, not generated ones (§21.1) |
| "Extract this into a microservice" | Requires evidence against the §8.2 criteria. Extractable, not extracted |
| "Add PgBouncer" | Only on measured connection pressure (§25), and prepared-statement behaviour must be integration-tested first |
| "Use JWTs for user sessions" | Conflicts with `AD-11`. Revocation, step-up and membership changes must take effect immediately. JWT is fine for OIDC |
| "Copy this code from AWCMS" | Conflicts with `AD-01` and `AD-24`. AWCMS is a requirements source; artifacts enter only as frozen fixtures under `contracts/legacy/awcms/` (`VG-15`) |
| "Skip the parity test, behaviour is obviously the same" | `AD-26`. "Obviously the same" is the claim the harness exists to check |
| "We benchmarked X and it is faster" | Check `VG-09` and `OD-04` — no acceptance thresholds exist yet, and correctness/security parity precedes performance (§23, §48) |

## Output format

State, in this order:

1. **Verdict** — one of the five in step 2.
2. **Decisions touched** — IDs, with the register row's position.
3. **Invariants at risk** — from step 3, or "none identified".
4. **Required follow-up** — the new decision to record, the `OD-` to close, the
   gate evidence to produce, or the changeset to add.

Be specific about what you did *not* check. A review that does not state its
own limits is the same failure the validation warns about in §0.
