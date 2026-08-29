# AWBMS Stage 1 — AWCMS Invariant Traceability

**Status:** initial traceability baseline  
**Source baseline:** AWCMS `11f2e95a47b1328a820f976d60f978c38a067903`; AWCMS-Astro `7b753be619244541b817d5d8e7d3b72cfe88d4f9`.

This matrix turns lessons from AWCMS into explicit AWBMS control objectives. It does **not** claim the AWBMS controls are implemented yet.

| AWCMS invariant / lesson | Source evidence class | AWBMS control / decision | Required automated evidence | Operational evidence |
|---|---|---|---|---|
| Tenant-scoped rows cannot cross tenant boundaries | migrations, generated RLS inventory, integration tests | explicit tenant predicate + transaction-local tenant context + `FORCE RLS` | malicious cross-tenant integration tests under `awbms_app` | RLS/permission anomaly monitoring |
| App role must not bypass RLS or own business tables | DB-role separation design/tests | separate `migrator/app/worker/operator` roles; `NOBYPASSRLS` | grants/ownership integration test | periodic privilege drift check |
| Protected operations pass an authorization chokepoint | AWCMS access gate scripts/tests | one central authorization service; default deny | bypass-scanner/architecture test + decision-vector tests | privileged-action/deny monitoring |
| Declared permissions must be enforced | permission-enforcement gate | module permission declaration + policy binding | declaration-to-route/action coverage test | policy-deny anomaly review |
| RBAC alone is insufficient | AWCMS ABAC/business scope/SoD implementation | RBAC + ABAC + business scope + SoD | allow/deny vectors, conflict tests | policy decision audit |
| Module dependency graph stays acyclic | module DAG gate | explicit compile/review-time module registry | DAG validation | release gate |
| Routes have one owner | module route gate | route ownership registry | collision/undeclared route test | contract inventory |
| Mutable tables have one authoritative writer | table-write ownership gate | one-table-one-writer | SQL/write ownership scanner or repository architecture test | migration review |
| Jobs have explicit owners and env contracts | job registry/env allow-list gates | job registry + one active runtime owner | undeclared-job/env tests | queue-owner/runbook checks |
| Migration history is immutable | migration tooling/gates | ordered migration ledger + SHA-256 + lock | applied migration checksum test | deploy preflight |
| External provider I/O must not hold ordinary business transactions | AWCMS incident/ADR lessons | outbox/job after commit | integration test proving provider call occurs outside mutation tx | DB transaction-duration monitoring |
| Consumer contracts cannot drift silently | AWCMS consumer-contract gate + Astro contract tests | frozen OpenAPI/compat fixtures + parity tests | additive/backward compatibility checks | consumer-version inventory |
| AWCMS-Astro tenant identity must agree with machine credential | Astro tenant resolution tests | credential-bound tenant + optional asserted tenant equality | mismatch must fail closed | build/deploy diagnostics |
| Anonymous browser API cannot reuse build-time auth assumptions | Astro consumed-path behavior | endpoint class-specific CORS/origin/auth rules | browser/CORS/security tests by surface class | origin/rejection metrics |
| Newsletter sender limiting does not protect the recipient | AWCMS newsletter abuse fix | multi-dimensional abuse controls incl. recipient/resource cooldown | concurrent recipient-abuse tests | delivery complaint/bounce/security metrics |
| Enumeration-sensitive responses remain neutral | auth/newsletter behavior | stable neutral response contracts | response indistinguishability tests | abuse monitoring |
| Refused newsletter repeat must not invalidate valid prior token | AWCMS newsletter integration tests | conditional atomic mutation, no token rotation on refused repeat | concurrency + token-validity regression test | delivery/confirmation anomaly review |
| Audit logs differ from operational logs | logging architecture | dedicated security/business audit model | audit field/redaction tests | immutable/retention monitoring |
| Data lifecycle needs module-owned semantics | lifecycle registries/gates | module data classification + retention/purge/legal hold declaration | registry coverage + legal-hold regression tests | purge/archive/legal-hold evidence |
| Global tables are exceptional, not an escape from tenancy | generated RLS inventory + global privilege list | explicit global-table registry with forbidden privileges | no-business-table escape test | privilege drift review |
| Generated inventories can drift from code | repo/project/script inventory gates | AWBMS generated artifact checks | regeneration-diff gate | CI |
| Documentation claims can outlive implementation | AWCMS drift incidents | code/contracts/tests outrank prose; provenance required | doc/source inventory checks where meaningful | architecture review |
| Release assumptions can be wrong even when final state looks correct | AWCMS release workflow incidents | explicit release invariants and post-action verification | workflow policy tests | release audit |
| Cache must not become authorization truth | architecture decision | PostgreSQL authoritative state | cache-eviction correctness tests | cache health independent of auth correctness |
| Retried high-risk mutations must not duplicate state | idempotency/domain-event lessons | tenant/principal/request-bound idempotency | concurrent duplicate-request tests | duplicate detection metrics |
| Worker migration must have one production owner | migration architecture | explicit drain/handover/cutback protocol | isolated handover test | queue ownership dashboard/runbook |

## Minimum Stage-1 Traceability Closure

Before Stage-1 sign-off:

- each row above must have a concrete AWBMS owner or owning future module;
- `VG-01` must link the source evidence path for each inherited invariant;
- each Blueprint security invariant must identify its planned test class;
- any invariant rejected as obsolete must carry rationale rather than disappearing.
