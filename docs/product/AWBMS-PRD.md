# AWBMS Stage 2 — Product Requirements Document

| Field | Value |
|---|---|
| Status | **Accepted 2026-11-23** — every `PD-` value accepted by the owner ([ADR-0009](../adr/0009-accept-stage-2-prd-owner-decisions-pd-01-22.md)); lifecycle step 2 closed |
| Date | 2026-11-23 |
| Lifecycle stage | Stage 2 — User Research + PRD (lifecycle v2, step 2; [ADR-0004](../adr/0004-adopt-product-development-flow-v2.md)) |
| Master Blueprint | [AWBMS Master Blueprint](../architecture/AWBMS-MASTER-BLUEPRINT.md) (Stage 1, signed off) |
| Architecture validation | [AWBMS Rust Architecture Validation](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md) (registers in Appendices A–E) |
| Stage 1 sign-off | [ADR-0008](../adr/0008-close-od-03-05-06-resolve-assumptions-sign-off-stage-1.md) |
| User research | [AWBMS User Research](AWBMS-USER-RESEARCH.md) (personas, usage context, jobs-to-be-done) |
| Implementation | **Blocked until gate 9.** This document specifies; it authorises no code |

## Contents

1. [Purpose and scope](#1-purpose-and-scope)
2. [v1 release boundary](#2-v1-release-boundary)
3. [Stakeholders and personas](#3-stakeholders-and-personas)
4. [Business capabilities](#4-business-capabilities)
5. [Requirements](#5-requirements)
6. [Service tiers and product SLO targets](#6-service-tiers-and-product-slo-targets)
7. [Recovery objectives per tier](#7-recovery-objectives-per-tier)
8. [Capacity assumptions](#8-capacity-assumptions)
9. [Migration success criteria and v1 acceptance](#9-migration-success-criteria-and-v1-acceptance)
10. [Metrics and instrumentation plan](#10-metrics-and-instrumentation-plan)
11. [Won't have](#11-wont-have)
12. [Traceability](#12-traceability)
13. [Owner decisions](#13-owner-decisions)

## 0. How to read this document

Claim labels are those of [validation §0.2](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#02-claim-labels). In this PRD:

- **[D]** cites a decision that already exists (`AD-`, `OD-`, or an ADR). Nothing here re-decides one.
- **[F]** is used only for a fact checked against a frozen artifact in this repository.
- **[A]** marks an assumption. Every capacity figure in [§8](#8-capacity-assumptions) is **[A]**: no AWBMS workload has been measured.
- **[R]** marks a requirement or value this PRD proposes.
- **[O]** marks an open question. Each one is collected in [§13](#13-owner-decisions) as a `PD-` row.
- **[G]** cites a verification gate. A gate reference is not a passed gate.

A value written **(owner)** was proposed in the draft and is now an owner decision: every `PD-` row was accepted as written on 2026-11-23 ([ADR-0009](../adr/0009-accept-stage-2-prd-owner-decisions-pd-01-22.md)). The label is kept so that each owner-set value stays traceable to its `PD-` row. Values taken from [ADR-0008](../adr/0008-close-od-03-05-06-resolve-assumptions-sign-off-stage-1.md) are initial decisions (`OD-06`), not measurements; they came from internal traffic-readiness research (DL-049).

Requirement rules applied to every `REQ-` row: one sentence, "The system SHALL …"; says what, not how; testable; traceable to existing register IDs. Priority uses MoSCoW. A threshold with no decision behind it is marked (owner).

## 1. Purpose and scope

AWBMS is the independent, Rust-native backend and evolving system of record for the AhliWeb ecosystem ([Blueprint §1](../architecture/AWBMS-MASTER-BLUEPRINT.md#1-executive-decision), [§3](../architecture/AWBMS-MASTER-BLUEPRINT.md#3-product-mission)). This PRD turns the signed-off architecture into product requirements, as [Blueprint §31](../architecture/AWBMS-MASTER-BLUEPRINT.md#31-recommended-stage-transition) requires of Stage 2: personas, business capabilities, prioritised requirements, service tiers, product SLO targets, recovery objectives, capacity assumptions, migration success criteria and acceptance criteria.

In scope: the sixteen v1 capabilities and five delivery waves of [Blueprint §8](../architecture/AWBMS-MASTER-BLUEPRINT.md#8-awbms-v1-scope--od-09-decision), their non-functional properties, and the migration from AWCMS to AWBMS with the AWCMS-Astro consumed surface as the compatibility contract ([OD-03](../adr/0008-close-od-03-05-06-resolve-assumptions-sign-off-stage-1.md), `VG-05`).

Not in scope: architecture or technology selection (settled in Stage 1; this PRD names no library), screen design (step 4), data model (step 5), authorisation matrix (step 6), API contracts (step 8). AWBMS has no frontend today; whether a slice has a screen of its own is decided per slice at step 4 ([ADR-0004](../adr/0004-adopt-product-development-flow-v2.md), `PD-03`).

## 2. v1 release boundary

The v1 definition is [Blueprint §8.1](../architecture/AWBMS-MASTER-BLUEPRINT.md#8-awbms-v1-scope--od-09-decision) (`OD-09`, closed by ADR-0006): the Rust-native platform kernel plus the minimum content and public-consumer compatibility wave needed to prove tenancy, identity, authorisation, contracts, migration and cutover against a real AWCMS-Astro consumer.

| Boundary | Content | Authority |
|---|---|---|
| **In v1** | The sixteen capabilities `CAP-01`–`CAP-16` of [§4](#4-business-capabilities), delivered in waves 0–4 | Blueprint §8.2, §8.4 |
| **Remains in AWCMS** | `sync_storage`, `workflow`, `reporting`, `theming`, `form_drafts`, `comments`, `idn_admin_regions`, `push_delivery`. "Remains" is not "retire": each needs a later Reuse / Port / Redesign / Replace / Retire decision backed by consumers | Blueprint §8.3, `AD-S1-04` |
| **Out of v1** | Everything in [§11](#11-wont-have) | This PRD |

Compatibility boundary [D]: only the AWCMS-Astro consumed surface (13 consumed paths, 2 committed future paths) carries a compatibility guarantee. Other AWCMS endpoints carry none ([ADR-0008](../adr/0008-close-od-03-05-06-resolve-assumptions-sign-off-stage-1.md), `OD-03`).

Deployment boundary [D]: one shared multi-tenant deployment per environment; a dedicated deployment is an exception recorded in its own ADR (`OD-05`).

## 3. Stakeholders and personas

Persona IDs are fixed and shared with the research document. **Evidence, usage context and jobs-to-be-done for every persona are in [AWBMS User Research](AWBMS-USER-RESEARCH.md); this PRD does not duplicate them.**

| ID | Persona | Relationship to AWBMS | Primary requirement areas |
|---|---|---|---|
| `P-01` | Public site visitor (via AWCMS-Astro) | Never calls AWBMS knowingly; reaches search, analytics and newsletter through the consumer site | [§5.9](#59-search-seo-and-site-profile), [§5.10](#510-visitor-analytics-and-newsletter) |
| `P-02` | Newsletter subscriber | Anonymous; subscribes, confirms, unsubscribes by email link | [§5.10](#510-visitor-analytics-and-newsletter), [§5.6](#56-data-lifecycle-and-privacy) |
| `P-03` | Tenant content editor | Creates and publishes posts and media within one tenant | [§5.8](#58-media-and-blog-content) |
| `P-04` | Tenant administrator | Manages users, roles, modules, domains and credentials of one tenant | [§5.1](#51-tenancy-and-isolation), [§5.2](#52-identity-and-credentials), [§5.3](#53-authorisation-and-audit) |
| `P-05` | Platform operator / SRE | Provisions tenants, runs cutover, watches health, recovers | [§5.5](#55-durable-runtime), [§5.11](#511-availability-recovery-and-observability), [§5.12](#512-migration-and-cutover) |
| `P-06` | Integration client / consumer developer (machine credentials, AWCMS-Astro) | Build-time authenticated reads and anonymous browser calls | [§5.7](#57-tenant-domain-and-consumer-compatibility), [§5.8](#58-media-and-blog-content) |
| `P-07` | Security and privacy officer / data-subject handling | Audits access, handles subject requests, owns retention | [§5.3](#53-authorisation-and-audit), [§5.6](#56-data-lifecycle-and-privacy) |
| `P-08` | Derived-application / module developer (candidate, added by the research) | Adds a domain module that must inherit tenant, authorisation and audit guarantees | No `REQ-` traced yet; inclusion in v1 is `PD-22` |

## 4. Business capabilities

The sixteen capabilities are those of [Blueprint §8.2](../architecture/AWBMS-MASTER-BLUEPRINT.md#8-awbms-v1-scope--od-09-decision). "Build wave" is where the capability is delivered ([Blueprint §8.4](../architecture/AWBMS-MASTER-BLUEPRINT.md#8-awbms-v1-scope--od-09-decision)); **wave 4 applies to every capability** (shadow, parity, staged cutover, rollback). Tier is defined in [§6](#6-service-tiers-and-product-slo-targets).

| ID | AWCMS capability | AWBMS treatment | Build wave | Tier | Main personas |
|---|---|---|---|---|---|
| `CAP-01` | `tenant_admin` | Redesign | 1 | T1 | P-04, P-05 |
| `CAP-02` | `profile_identity` | Redesign / port behaviour | 1 | T1 | P-03, P-04 |
| `CAP-03` | `identity_access` | Redesign | 1 | T1 | P-04, P-06, P-07 |
| `CAP-04` | `module_management` | Redesign | 0 (registry), 1 | T3 | P-04, P-05 |
| `CAP-05` | `logging` (audit and operational logging, separate) | Redesign | 0 (telemetry), 1 (audit) | T1 audit write, T3 query | P-05, P-07 |
| `CAP-06` | `domain_event_runtime` (outbox, jobs, idempotency) | Redesign | 2 | T3 | P-05 |
| `CAP-07` | `data_lifecycle` | Redesign | 2 | T3 | P-07, P-02 |
| `CAP-08` | `email` | Port behaviour behind provider port | 2 | T3 | P-02, P-05 |
| `CAP-09` | `tenant_domain` | Port behaviour, harden | 3 | T1 | P-06, P-04 |
| `CAP-10` | `media_library` | Redesign / port contract | 3 | T2 | P-03, P-06 |
| `CAP-11` | `blog_content` | Port behaviour, redesign implementation | 3 | T2 | P-03, P-06, P-01 |
| `CAP-12` | `seo_distribution` | Port required behaviour | 3 | T2 | P-06, P-01 |
| `CAP-13` | `site_search` | Port contract and behaviour | 3 | T2 | P-01, P-06 |
| `CAP-14` | `newsletter` | Port with abuse controls | 3 | T2 (request), T3 (delivery) | P-02, P-01 |
| `CAP-15` | `site_profile` | Port contract and behaviour | 3 | T2 | P-06, P-04 |
| `CAP-16` | `visitor_analytics` | Redesign, privacy-first | 3 | T2 | P-01, P-06 |

Cross-cutting requirements that belong to no single capability carry the code `PLT` (platform) in the requirement tables.

## 5. Requirements

Each group has a requirements table and an acceptance table. Columns: **P** persona(s); **CAP** capability (`PLT` = platform-wide); **Pri** MoSCoW; **Measure** is metric IDs from [§10](#10-metrics-and-instrumentation-plan) (metric ← source); **Derived from** existing `AD-`, `OD-`, `VG-`, `C-`, `AS-` IDs. Acceptance scenarios are Given / When / Then with concrete values; security-relevant requirements include a negative or boundary scenario.

### 5.1 Tenancy and isolation

| ID | Requirement | P | CAP | Pri | Measure | Derived from |
|---|---|---|---|---|---|---|
| `REQ-001` | The system SHALL prevent a principal bound to one tenant from reading or changing the data of any other tenant, even when one enforcement layer is defective. | P-04, P-07 | CAP-01 | Must | MET-01 | `AD-10`, `AS-05`, `OD-05`, `VG-06` |
| `REQ-002` | The system SHALL serve all tenants of an environment from one shared deployment, requiring a dedicated deployment only through a recorded exception. | P-05 | CAP-01 | Must | MET-32 (deployment inventory) | `OD-05`, `AS-05` |
| `REQ-003` | The system SHALL let a platform operator provision a tenant with its initial administrator and enabled module set within 5 minutes (owner). | P-05 | CAP-01 | Must | MET-31 | `AD-10`, `AD-08` |
| `REQ-004` | The system SHALL deny every principal of a suspended tenant within 60 seconds (owner) of the suspension and SHALL restore access on reactivation without data loss. | P-05, P-04 | CAP-01 | Must | MET-07 | `AD-10`, `AD-11` |

| ID | Acceptance scenario |
|---|---|
| `REQ-001` | **Given** tenants `acme` and `beta` each own 10 posts, **when** a session of `acme` requests a post identifier owned by `beta` through the API, **then** the response is a denial with the stable error envelope, no `beta` field appears, and a cross-tenant denial event is audited. **Negative:** with the application-level tenant predicate removed in a test build, the same direct query through the application database role returns 0 rows. |
| `REQ-002` | **Given** the production inventory, **when** the deployment list is read, **then** it shows exactly one AWBMS application deployment for the production environment and any dedicated deployment cites an ADR. |
| `REQ-003` | **Given** an operator with the provisioning permission, **when** a tenant `gamma` with administrator `admin@gamma.example` and modules `blog_content`, `media_library` is provisioned, **then** the administrator can sign in and only those two modules respond, within 300 s. **Boundary:** a duplicate tenant key `gamma` is refused and creates no partial tenant. |
| `REQ-004` | **Given** 3 active sessions and 1 machine credential of tenant `acme`, **when** the operator suspends `acme`, **then** all four are denied within 60 s; **when** reactivated, **then** the same data is readable and sessions that were revoked stay revoked. |

### 5.2 Identity and credentials

| ID | Requirement | P | CAP | Pri | Measure | Derived from |
|---|---|---|---|---|---|---|
| `REQ-005` | The system SHALL authenticate human users by password and issue a revocable session whose raw token is disclosed once and never stored in recoverable form. | P-03, P-04 | CAP-03 | Must | MET-08 | `AD-11` |
| `REQ-006` | The system SHALL stop honouring a session within 5 seconds (owner) of logout, administrator revocation, password reset or removal of the user's tenant membership. | P-04, P-07 | CAP-03 | Must | MET-07 | `AD-11`, `VG-05` |
| `REQ-007` | The system SHALL expire a session after 30 minutes of inactivity or 12 hours of age, whichever is first (owner). | P-03, P-04 | CAP-03 | Must | MET-08 | `AD-11` |
| `REQ-008` | The system SHALL require step-up authentication, within the last 10 minutes (owner), for each operation the permission catalogue marks as privileged, and SHALL support a time-based second factor. | P-04 | CAP-03 | Must | MET-08 | `AD-10`, `AD-11` |
| `REQ-009` | The system SHALL lock an account for 15 minutes after 5 consecutive failed sign-ins (owner), independently of the client address. | P-04, P-07 | CAP-03 | Must | MET-08 | `OD-03`, `AD-11` |
| `REQ-010` | The system SHALL verify a user's password stored in the AWCMS password format and SHALL store it in the AWBMS format after the next successful sign-in. | P-03, P-04 | CAP-03 | Must | MET-08 (rehash rate) | `AD-12`, `OD-03` |
| `REQ-011` | The system SHALL require re-authentication of every user at the cutover of identity and SHALL NOT accept an AWCMS session token afterwards. | P-03, P-04 | CAP-03 | Must | MET-16 | `OD-03`, `AD-27` |
| `REQ-012` | The system SHALL issue machine credentials that are bound to one tenant, limited to declared scopes, bounded in lifetime to at most 365 days (owner), shown once and revocable within 5 seconds (owner). | P-04, P-06 | CAP-03 | Must | MET-07 | `AD-10`, `AD-11` |
| `REQ-013` | The system SHALL accept AWCMS-format machine credentials from AWCMS-Astro build-time reads during coexistence only, and SHALL reject them from a sunset date set per wave, proposed as 30 days after the wave 3 cutover (owner). | P-06, P-05 | CAP-03 | Must | MET-14 | `OD-03`, `AD-24` |
| `REQ-014` | The system SHALL reject, without returning data, a request whose asserted tenant differs from the tenant bound to the presented machine credential. | P-06 | CAP-03 | Must | MET-01 | `AD-10`, `VG-05` |
| `REQ-015` | The system SHALL let a tenant user view and update their own profile and SHALL NOT let them read another user's profile without a granted permission. | P-03, P-04 | CAP-02 | Should | MET-05 | `AD-10` |

| ID | Acceptance scenario |
|---|---|
| `REQ-005` | **Given** user `ani` with a valid password, **when** she signs in, **then** one session is created and the raw token is returned exactly once; **negative:** a database dump of session storage cannot be used to authenticate. |
| `REQ-006` | **Given** an active session `S1`, **when** the administrator revokes it at T0, **then** a request with `S1` at T0+5 s returns 401. **Boundary:** a request at T0+0 s that is already in flight completes, a new one is denied. |
| `REQ-007` | **Given** a session idle for 31 min, **when** it is used, **then** 401. **Given** an active session of age 12 h 1 min, **then** 401 regardless of activity. |
| `REQ-008` | **Given** a session authenticated 11 min ago and the privileged operation "grant administrator role", **when** invoked, **then** a step-up challenge is returned and no grant occurs; **after** step-up it succeeds. |
| `REQ-009` | **Given** 4 failed sign-ins from IP A and 1 from IP B for `ani`, **when** the fifth fails, **then** the account is locked for 15 min and the correct password is refused during that time without revealing the lock reason beyond the stable error. |
| `REQ-010` | **Given** a migrated user with an AWCMS Argon2id hash (m = 65536 KiB, t = 2, p = 1), **when** the correct password is entered, **then** sign-in succeeds and the stored hash is rewritten if it differs from current parameters. **Negative:** a plaintext or unrecognised hash format is refused. |
| `REQ-011` | **Given** an AWCMS session token captured before the identity cutover, **when** presented to AWBMS afterwards, **then** 401 and the user is prompted to sign in again. |
| `REQ-012` | **Given** a credential issued for tenant `acme` with scope `blog:read` and expiry 90 days, **when** it is used on day 91, **then** it is denied; **when** revoked on day 10, **then** denied within 5 s; the secret is not retrievable after issuance. |
| `REQ-013` | **Given** an `awcmsm_` credential and the build-time read path, **when** used before the sunset date, **then** accepted; **after** the sunset date, **then** rejected; **negative:** used on an anonymous browser path or on a write path, **then** rejected at any date. |
| `REQ-014` | **Given** a credential bound to `acme`, **when** a request asserts tenant `beta`, **then** a denial with no content is returned and the mismatch is audited; **when** it asserts `acme`, **then** it proceeds. |
| `REQ-015` | **Given** user `ani` without `profile:read-others`, **when** she requests `budi`'s profile, **then** denied; her own profile update with a 120-character display name succeeds. |

### 5.3 Authorisation and audit

| ID | Requirement | P | CAP | Pri | Measure | Derived from |
|---|---|---|---|---|---|---|
| `REQ-016` | The system SHALL authorise every protected operation through one central decision that denies when any required context is missing. | P-04, P-07 | CAP-03 | Must | MET-05 | `AD-10`, `VG-07` |
| `REQ-017` | The system SHALL let a tenant administrator grant and revoke roles and business-scope assignments within their own tenant and SHALL NOT provide an unconditional administrator bypass. | P-04 | CAP-03 | Must | MET-05 | `AD-10`, `VG-07` |
| `REQ-018` | The system SHALL refuse a single principal performing two actions that the separation-of-duties catalogue declares conflicting, such as approving their own privilege grant. | P-04, P-07 | CAP-03 | Must | MET-05 | `AD-10`, `VG-07` |
| `REQ-019` | The system SHALL deny every operation of a module that is not enabled for the requesting tenant. | P-04 | CAP-04 | Must | MET-05 | `AD-08`, `AD-10` |
| `REQ-020` | The system SHALL record, in the same atomic outcome as the change, an audit record naming actor, tenant, operation, target, outcome and correlation identifier for every security-relevant or privileged operation. | P-07, P-05 | CAP-05 | Must | MET-06 | `AD-10`, `AD-20` |
| `REQ-021` | The system SHALL prevent any principal, including a tenant administrator, from altering or deleting an audit record within its retention period. | P-07 | CAP-05 | Must | MET-06 | `AD-10` |
| `REQ-022` | The system SHALL audit every authorisation denial, including attempted cross-tenant access, without recording secrets. | P-07, P-05 | CAP-05 | Must | MET-01, MET-05 | `AD-10`, `AD-20` |
| `REQ-023` | The system SHALL keep audit records and operational logs in separate stores with separate access and retention. | P-07 | CAP-05 | Must | MET-06 | `AD-20` |
| `REQ-024` | The system SHALL let a tenant administrator and the privacy officer search the audit records of their own tenant by actor, time range and operation, with p95 latency of at most 500 ms. | P-04, P-07 | CAP-05 | Should | MET-03 | `OD-06`, `AD-10` |
| `REQ-025` | The system SHALL admit a module only when it declares owner, dependencies, permissions, routes, owned tables, events, jobs and data categories, and SHALL reject cyclic dependencies and route collisions. | P-05 | CAP-04 | Must | MET-28 | `AD-07`, `AD-08`, `AD-09`, `VG-02` |
| `REQ-026` | The system SHALL let a tenant administrator enable and disable the modules available to their tenant, with an audit record per change. | P-04 | CAP-04 | Should | MET-06 | `AD-08` |

| ID | Acceptance scenario |
|---|---|
| `REQ-016` | **Given** a route registered without a policy binding, **when** CI runs, **then** the build fails; **given** a request with a valid session but no resolvable tenant membership, **then** denied. |
| `REQ-017` | **Given** administrator `adm` of `acme`, **when** granting role `editor` to `ani` in `acme`, **then** it succeeds and is audited; **when** granting a role in `beta`, **then** denied. **Negative:** a role named `admin` carries no permission it was not assigned. |
| `REQ-018` | **Given** the rule "grant requester and grant approver must differ", **when** `adm` requests and approves the same grant, **then** the approval is refused and audited. |
| `REQ-019` | **Given** module `newsletter` disabled for `acme`, **when** its subscribe route is called with `acme`'s origin, **then** denied; enabling it makes the same call succeed. |
| `REQ-020` | **Given** a role grant, **when** the transaction is forced to fail after the audit write, **then** neither the grant nor the audit record exists; on success both exist with the same correlation identifier. |
| `REQ-021` | **Given** audit record `A1` of tenant `acme`, **when** the tenant administrator or the application role attempts to update or delete it, **then** the attempt fails and is itself audited. |
| `REQ-022` | **Given** `ani` of `acme` requests a `beta` resource, **when** denied, **then** one denial record exists with actor, target tenant and no token. |
| `REQ-023` | **Given** a retention change to operational logs, **then** audit retention is unchanged; an operational-log reader role cannot read audit records. |
| `REQ-024` | **Given** 1,000,000 audit rows in `acme` (assumption), **when** the officer searches actor `ani` over 7 days, **then** results return at p95 ≤ 500 ms and contain no row of `beta`. |
| `REQ-025` | **Given** a candidate module that depends on itself through two others, **when** admission runs, **then** it is rejected with the cycle named; a duplicate route `GET /api/v1/blog/posts` is rejected. |
| `REQ-026` | **Given** `adm` disables `site_search` for `acme`, **then** its routes deny within 60 s and one audit record exists. |

### 5.4 Cross-cutting security

| ID | Requirement | P | CAP | Pri | Measure | Derived from |
|---|---|---|---|---|---|---|
| `REQ-027` | The system SHALL keep secrets, credentials and unnecessary personal data out of every log, trace and metric. | P-07, P-05 | PLT | Must | MET-20 | `AD-20` |
| `REQ-028` | The system SHALL bound the size and processing time of every request and SHALL bound anonymous request bodies to 16 KiB (owner). | P-05 | PLT | Must | MET-04, MET-29 | `AD-04`, `VG-09` |
| `REQ-029` | The system SHALL answer overload with an explicit retryable response and SHALL NOT let latency grow without bound. | P-05, P-06 | PLT | Must | MET-29, MET-03 | `AD-04`, `VG-09` |
| `REQ-030` | The system SHALL keep authorisation, tenant membership and session validity correct after the loss of any cache. | P-05, P-07 | PLT | Must | MET-05 | `AD-18`, `AD-05` |
| `REQ-031` | The system SHALL keep no known critical or high-severity vulnerability in released dependencies unremediated beyond 72 hours (Blueprint §10) and SHALL publish a software bill of materials with each release. | P-05, P-07 | PLT | Must | MET-21 | `AD-22`, `OD-02`, `VG-13` |
| `REQ-032` | The system SHALL carry no runtime, build-time or source dependency on the AWCMS repository. | P-05 | PLT | Must | MET-28 | `AD-24`, `VG-15` |

| ID | Acceptance scenario |
|---|---|
| `REQ-027` | **Given** canary password `Canary-Pw-1`, token `awbms_canary_tok` and email `canary@example.test` pushed through sign-in, subscribe and job paths, **when** all log, trace and metric outputs are searched, **then** 0 matches. |
| `REQ-028` | **Given** an anonymous POST of 17 KiB to a public endpoint, **then** it is refused before processing with a stable error; a 16 KiB body is processed. |
| `REQ-029` | **Given** the database pool is exhausted in a test, **when** 200 requests arrive, **then** excess requests receive a retryable overload response within 1 s rather than waiting beyond the 900 ms write target. |
| `REQ-030` | **Given** a full cache eviction, **when** a revoked session is presented, **then** denied; permissions equal their pre-eviction values. |
| `REQ-031` | **Given** a new critical advisory against a dependency in the release, **then** an issue is triaged within 24 h and remediated or risk-accepted within 72 h; a release without an SBOM fails the release gate. |
| `REQ-032` | **Given** the repository, **when** the independence check runs, **then** no submodule, path or git dependency on AWCMS exists. |

### 5.5 Durable runtime

| ID | Requirement | P | CAP | Pri | Measure | Derived from |
|---|---|---|---|---|---|---|
| `REQ-033` | The system SHALL commit a domain event if and only if the business change it describes commits. | P-05 | CAP-06 | Must | MET-10, MET-09 | `AD-13`, `VG-11` |
| `REQ-034` | The system SHALL deliver each committed event at least once, deduplicable by a stable identifier, with p95 dispatch lag of at most 30 seconds (owner). | P-05 | CAP-06 | Must | MET-10 | `AD-13`, `VG-11` |
| `REQ-035` | The system SHALL move an event or job that fails after 8 attempts (owner) to a dead-letter state visible to operators and SHALL allow an audited replay. | P-05 | CAP-06 | Must | MET-11 | `AD-13`, `AD-14` |
| `REQ-036` | The system SHALL ensure that each job is processed by exactly one worker at a time and is not processed twice after a worker failure. | P-05 | CAP-06 | Must | MET-11 | `AD-14`, `VG-11` |
| `REQ-037` | The system SHALL NOT perform an external provider call while a business transaction is open. | P-05 | CAP-06 | Must | MET-13 | `AD-15`, `VG-05` |
| `REQ-038` | The system SHALL turn a retried high-risk mutation with the same tenant, principal, key and request content into exactly one committed effect and an identical response. | P-06, P-05 | CAP-06 | Must | MET-09 | `AD-23`, `VG-11` |
| `REQ-039` | The system SHALL refuse reuse of an idempotency key with different request content and SHALL expire stored keys after 24 hours (owner). | P-06 | CAP-06 | Must | MET-09 | `AD-23`, `VG-11` |

| ID | Acceptance scenario |
|---|---|
| `REQ-033` | **Given** a post publish whose transaction rolls back after the event write, **then** no event is visible; on commit both exist. |
| `REQ-034` | **Given** 1,000 committed events, **when** the dispatcher runs normally, **then** each is delivered at least once and p95 lag ≤ 30 s; a duplicate delivery carries the same event identifier. |
| `REQ-035` | **Given** a job whose provider call fails permanently, **then** after attempt 8 it is dead-lettered, an operator sees it, and replay creates one audit record. |
| `REQ-036` | **Given** 4 workers and 10,000 jobs with one worker killed mid-run, **then** every job completes exactly once in effect and 0 jobs are claimed by two workers at once. |
| `REQ-037` | **Given** the provider is delayed 30 s, **when** a subscribe request commits, **then** the request returns within the write target and database transaction time stays below 100 ms (assumption). |
| `REQ-038` | **Given** key `K1` and body `B`, **when** sent twice (the second after a timeout), **then** one resource with one identifier and one event sequence exist and both responses are identical. **Negative:** 50 parallel sends yield one effect. |
| `REQ-039` | **Given** key `K1` used with body `B`, **when** reused with body `B'`, **then** a conflict error and no effect; after 24 h the key can be reused. |

### 5.6 Data lifecycle and privacy

| ID | Requirement | P | CAP | Pri | Measure | Derived from |
|---|---|---|---|---|---|---|
| `REQ-040` | The system SHALL refuse to admit a data-owning module that has not declared its data categories, purpose, retention, erasure behaviour, legal-hold behaviour and subject-export behaviour. | P-07 | CAP-07 | Must | MET-19, MET-28 | `VG-14`, `AD-08` |
| `REQ-041` | The system SHALL remove or anonymise data when its declared retention period ends, including derived copies, unless a legal hold applies. | P-07 | CAP-07 | Must | MET-19 | `VG-14` |
| `REQ-042` | The system SHALL NOT allow any retention or purge process to remove data under legal hold, and SHALL audit each hold's creation and release. | P-07 | CAP-07 | Must | MET-19 | `VG-14` |
| `REQ-043` | The system SHALL complete a verified data-subject export or erasure within 14 days (owner) of a request, scoped to one tenant and returning only declared categories. | P-07, P-02 | CAP-07 | Must | MET-19 | `VG-14` |
| `REQ-044` | The system SHALL NOT reintroduce erased personal data when state is restored from backup. | P-07 | CAP-07 | Should | MET-19 | `VG-14`, `VG-12` |

| ID | Acceptance scenario |
|---|---|
| `REQ-040` | **Given** a module with owned tables and no lifecycle descriptor, **when** admission runs, **then** it fails and names the missing descriptors. |
| `REQ-041` | **Given** analytics events older than the declared period (90 days, owner), **when** the purge runs, **then** they and their derived aggregates containing identifiers are gone; the run is recorded. |
| `REQ-042` | **Given** legal hold `H1` on subscriber `s@example.test`, **when** the ordinary purge runs after the retention end, **then** the data remains and the hold is audited; **after** release it is purged. |
| `REQ-043` | **Given** a request from `s@example.test` for tenant `acme`, **when** processed, **then** within 14 days an export contains only declared categories for `acme`, or the erasure leaves no identifiable row, excluding held data. |
| `REQ-044` | **Given** a restore to a point before an erasure, **then** the erasure ledger is re-applied before the system serves traffic, or the gap is recorded with an owner. |

### 5.7 Tenant domain and consumer compatibility

| ID | Requirement | P | CAP | Pri | Measure | Derived from |
|---|---|---|---|---|---|---|
| `REQ-045` | The system SHALL resolve the tenant of a browser request from its verified origin or host and SHALL return no tenant data when the origin is unknown or unverified. | P-06, P-01 | CAP-09 | Must | MET-01, MET-14 | `AD-10`, `AD-24`, `VG-05` |
| `REQ-046` | The system SHALL let a tenant administrator register a custom domain and activate it only after proof of control. | P-04 | CAP-09 | Should | MET-05 | `AD-10` |
| `REQ-047` | The system SHALL serve each of the 13 AWCMS-Astro consumed paths with request and response behaviour that shows no unexplained difference from the frozen consumer fixture, in all five execution classes. | P-06, P-01 | PLT | Must | MET-14, MET-15 | `OD-03`, `AD-24`, `AD-26`, `VG-05` |
| `REQ-048` | The system SHALL reject any change to the consumed or committed contract that breaks the frozen consumer fixture unless the old shape is retained under a compatibility window. | P-06 | PLT | Must | MET-14 | `AD-19`, `VG-08` |
| `REQ-049` | The system SHALL answer every client-facing error with one stable, documented error envelope. | P-06 | PLT | Must | MET-04, MET-14 | `AD-21`, `VG-08` |
| `REQ-050` | The system SHALL provide every implemented route in the reviewed API contract and SHALL provide no implemented route absent from it. | P-06 | PLT | Must | MET-28 | `AD-19`, `VG-08` |
| `REQ-051` | The system SHALL provide the two committed future consumer paths, machine-credential management and session inspection, as specified contracts before any consumer depends on them. | P-06, P-04 | CAP-03 | Could | MET-14 | `AD-19`, `OD-03` |

| ID | Acceptance scenario |
|---|---|
| `REQ-045` | **Given** tenant `acme` registered with origin `https://acme.example`, **when** search is called from `https://evil.example`, **then** no `acme` data and no existence signal; from `https://acme.example`, results for `acme` only. |
| `REQ-046` | **Given** `adm` registers `news.acme.example`, **then** it stays inactive until a control proof is verified; an unverified domain resolves to no tenant. |
| `REQ-047` | **Given** the frozen fixture for `/api/v1/blog/posts`, **when** the parity run replays it, **then** 0 unexplained differences; the same for each of the 13 paths. **Negative:** the anonymous classes reject a credential-based call shape and the build-time classes reject anonymous calls. |
| `REQ-048` | **Given** a change removing field `slug` from the posts response, **then** CI fails on the consumer-fixture comparison. |
| `REQ-049` | **Given** a forced validation failure on any route, **then** the body matches the envelope schema and exposes no internal detail. |
| `REQ-050` | **Given** a new handler without a contract entry, **then** the contract check fails; a contract entry marked "planned" is allowed. |
| `REQ-051` | **Given** the two committed paths, **then** a contract document exists with fixtures, and the contract check passes. |

### 5.8 Media and blog content

| ID | Requirement | P | CAP | Pri | Measure | Derived from |
|---|---|---|---|---|---|---|
| `REQ-052` | The system SHALL return, for a media-object request, every resolvable object and an explicit unresolved marker for each unresolved identifier, never dropping one silently. | P-06 | CAP-10 | Must | MET-25, MET-14 | `AD-17`, `VG-05` |
| `REQ-053` | The system SHALL let an editor register media with verified checksum and metadata and SHALL commit that registration while the object store is unavailable, showing a pending or failed state. | P-03 | CAP-10 | Must | MET-25 | `AD-15`, `AD-17`, `AS-07` |
| `REQ-054` | The system SHALL expose the public origin of a tenant's media to authorised consumers. | P-06 | CAP-10 | Must | MET-14 | `AD-17` |
| `REQ-055` | The system SHALL let an editor create, edit, publish and unpublish blog posts of their own tenant, and SHALL record each change in the audit log. | P-03 | CAP-11 | Must | MET-27, MET-06 | `AD-10` |
| `REQ-056` | The system SHALL return only published posts, terms, menus and widgets of the credential-bound tenant to a consumer read, in the shape of the frozen fixture. | P-06, P-01 | CAP-11 | Must | MET-14, MET-26 | `AD-10`, `VG-05` |
| `REQ-057` | The system SHALL make an unpublished post unavailable to every consumer read within 60 seconds (owner) of unpublishing. | P-03, P-06 | CAP-11 | Must | MET-24 (freshness), MET-14 | `AD-18`, `VG-05` |

| ID | Acceptance scenario |
|---|---|
| `REQ-052` | **Given** a request for media ids `m1`, `m2`, `m3` where `m3` does not exist, **then** `m1` and `m2` are returned and `m3` is reported unresolved with its id. |
| `REQ-053` | **Given** the object store is down, **when** an editor registers `photo.jpg` (SHA-256 supplied), **then** the registration commits, status is pending, and a repair job completes after recovery without holding a database lock during the timeout. **Negative:** a checksum mismatch is rejected. |
| `REQ-054` | **Given** tenant `acme`, **when** the origin is requested with its credential, **then** its own public origin is returned, never another tenant's. |
| `REQ-055` | **Given** editor `ani`, **when** she publishes post `p1`, **then** it becomes readable to consumers and one audit record exists; an editor without `blog:publish` is denied. |
| `REQ-056` | **Given** 10 published and 3 draft posts in `acme`, **when** a consumer lists posts, **then** exactly 10 are returned; **negative:** a credential of `beta` receives 0 `acme` posts. |
| `REQ-057` | **Given** `p1` unpublished at T0, **then** at T0+60 s consumer reads and search exclude it. |

### 5.9 Search, SEO and site profile

| ID | Requirement | P | CAP | Pri | Measure | Derived from |
|---|---|---|---|---|---|---|
| `REQ-058` | The system SHALL answer an anonymous cross-origin search query for the tenant resolved from the origin with published content only, at p95 latency of at most 500 ms. | P-01, P-06 | CAP-13 | Must | MET-24, MET-03 | `OD-06`, `OD-07`, `AD-10` |
| `REQ-059` | The system SHALL answer an anonymous search-suggestion request for the same tenant, at p95 latency of at most 300 ms (owner). | P-01, P-06 | CAP-13 | Must | MET-24 | `OD-06`, `OD-07` |
| `REQ-060` | The system SHALL return the composed site profile and the tenant's search-engine and discovery metadata in the shape the consumer fixture records. | P-06 | CAP-15 | Must | MET-14, MET-26 | `AD-24`, `VG-05` |
| `REQ-061` | The system SHALL provide the SEO and discovery data the consumed surface depends on, for the tenant resolved from the request. | P-06, P-01 | CAP-12 | Must | MET-14 | `AD-24`, `VG-05` |

| ID | Acceptance scenario |
|---|---|
| `REQ-058` | **Given** the origin `https://acme.example` and 5,000 indexed posts (assumption), **when** querying "tarif", **then** only published `acme` posts return at p95 ≤ 500 ms. **Negative:** an unknown origin returns an empty neutral result and no other tenant's text. |
| `REQ-059` | **Given** prefix "tar", **then** at most 10 suggestions return at p95 ≤ 300 ms (owner). |
| `REQ-060` | **Given** the frozen fixture for `/api/v1/site-profile/composed`, **then** parity shows 0 unexplained differences; a credential-less call is rejected. |
| `REQ-061` | **Given** tenant `acme` with SEO defaults, **then** the response carries them and no other tenant's. |

### 5.10 Visitor analytics and newsletter

| ID | Requirement | P | CAP | Pri | Measure | Derived from |
|---|---|---|---|---|---|---|
| `REQ-062` | The system SHALL accept anonymous visitor telemetry and answer with a neutral, minimal response regardless of whether the event was stored. | P-01, P-06 | CAP-16 | Must | MET-23 | `AD-10`, `VG-05` |
| `REQ-063` | The system SHALL NOT store a direct visitor identifier such as a raw network address (owner). | P-01, P-07 | CAP-16 | Must | MET-23 | `AD-10`, `VG-14` |
| `REQ-064` | The system SHALL limit telemetry acceptance per client and per tenant and SHALL drop excess events with the neutral response. | P-05, P-06 | CAP-16 | Must | MET-23 | `AD-04`, `VG-05` |
| `REQ-065` | The system SHALL respond identically to a newsletter subscription whether or not the address is already subscribed, and SHALL send no more than one confirmation message per recipient per 15 minutes (owner). | P-02 | CAP-14 | Must | MET-22 | `C-07`, `AD-15` |
| `REQ-066` | The system SHALL limit subscription attempts to 5 per client per 5 minutes (owner). | P-02, P-05 | CAP-14 | Must | MET-22 | `C-07`, `AD-04` |
| `REQ-067` | The system SHALL NOT invalidate a previously issued confirmation token when a repeat subscription is refused. | P-02 | CAP-14 | Must | MET-22 | `C-07`, `VG-11` |
| `REQ-068` | The system SHALL accept a newsletter write from a browser only from an origin registered to the resolved tenant. | P-02, P-06 | CAP-14 | Must | MET-22, MET-01 | `AD-10`, `VG-05` |
| `REQ-069` | The system SHALL honour a confirmation or unsubscribe link issued before cutover for its remaining validity and SHALL stop all mailing to an unsubscribed address within 60 seconds (owner). | P-02 | CAP-14 | Must | MET-22 | `OD-03`, `AD-15` |
| `REQ-070` | The system SHALL send newsletter confirmation messages asynchronously after commit through the email capability. | P-02 | CAP-08 | Must | MET-12 | `AD-15`, `AD-17`, `AS-07` |
| `REQ-071` | The system SHALL stop sending to an address after a permanent delivery failure or a complaint. | P-02, P-05 | CAP-08 | Should | MET-12 | `AD-17` |

| ID | Acceptance scenario |
|---|---|
| `REQ-062` | **Given** valid and malformed telemetry payloads, **then** both return the same status and body shape; only the valid one is stored. |
| `REQ-063` | **Given** an event from `203.0.113.7`, **then** no stored row or log contains that address. |
| `REQ-064` | **Given** 200 events from one client in 10 s against a limit of 100 (owner), **then** the first 100 are stored and the rest return the neutral response without storage. |
| `REQ-065` | **Given** `v@example.test` subscribed at T0, **when** subscribing again at T0+60 s, **then** the response is byte-equivalent to a first-time response and one message exists; at T0+16 min a second message is allowed. |
| `REQ-066` | **Given** 6 attempts from one client in 300 s, **then** the sixth is limited with a retry hint. **Boundary:** the 5th succeeds. |
| `REQ-067` | **Given** token `T1` issued at T0, **when** a refused repeat occurs at T0+60 s, **then** `T1` still confirms. |
| `REQ-068` | **Given** the origin `https://evil.example`, **then** the subscribe is rejected and sends no message to the victim address. |
| `REQ-069` | **Given** a confirmation link emailed 2 days before cutover with 7 days of validity, **then** it works after cutover; **after** unsubscribe, no mail is queued after T0+60 s. |
| `REQ-070` | **Given** the provider times out for 30 s, **then** the request has already returned within the write target and retry is bounded. |
| `REQ-071` | **Given** a hard-bounce notification, **then** the address receives no further message. |

### 5.11 Availability, recovery and observability

| ID | Requirement | P | CAP | Pri | Measure | Derived from |
|---|---|---|---|---|---|---|
| `REQ-072` | The system SHALL be available at least 99.9 percent of each calendar month per production deployment. | P-05, P-06 | PLT | Must | MET-02 | `OD-06`, `AS-06` |
| `REQ-073` | The system SHALL serve authenticated reads at p95 ≤ 500 ms and writes at p95 ≤ 900 ms at the concurrent-user count of [§8](#8-capacity-assumptions). | P-03, P-04 | PLT | Must | MET-03 | `OD-06`, `OD-04` |
| `REQ-074` | The system SHALL show no p95 or p99 latency or throughput regression above 10 percent, nor CPU or memory regression above 15 percent, against the last approved baseline without recorded review. | P-05 | PLT | Must | MET-30 | `OD-04`, `VG-09` |
| `REQ-075` | The system SHALL restore each tier within its RTO and to its RPO, as proven by a rehearsal on a production-like snapshot before each wave deploys. | P-05 | PLT | Must | MET-17, MET-18 | `AD-28`, `VG-12`, `AS-03` |
| `REQ-076` | The system SHALL emit request, database, authentication, job and event telemetry with one correlation identifier traceable from request to event to job. | P-05 | PLT | Must | MET-13, MET-11 | `AD-20` |
| `REQ-077` | The system SHALL alert on each service objective breach with a link to a runbook, and SHALL separate liveness from readiness. | P-05 | PLT | Should | MET-02, MET-04 | `AD-20`, `AD-30` |

| ID | Acceptance scenario |
|---|---|
| `REQ-072` | **Given** a 30-day month, **then** unplanned downtime budget is 43.2 minutes; the monthly report from the external probe shows ≥ 99.9 percent. |
| `REQ-073` | **Given** 25 concurrent users per tenant (assumption) under the Blueprint §11 scenarios, **then** read p95 ≤ 500 ms and write p95 ≤ 900 ms over a 30-minute run. |
| `REQ-074` | **Given** baseline p95 = 400 ms, **when** a candidate shows 450 ms (+12.5 percent), **then** the gate demands recorded review. |
| `REQ-075` | **Given** a snapshot, **when** the rehearsal restores T1 in 52 min against RTO 60 min (owner) with a measured data-loss window of 3 min against RPO 5 min (owner), **then** the rehearsal passes; any larger value fails the wave. |
| `REQ-076` | **Given** request `R1`, **then** the same correlation identifier appears in its outbox row, job and resulting event. |
| `REQ-077` | **Given** a synthetic availability drop, **then** an alert fires and links to a runbook; a readiness failure removes the instance from traffic while liveness stays true. |

### 5.12 Migration and cutover

| ID | Requirement | P | CAP | Pri | Measure | Derived from |
|---|---|---|---|---|---|---|
| `REQ-078` | The system SHALL migrate AWCMS data in a deterministic, versioned, resumable process whose reconciliation of counts, checksums and domain invariants shows zero unexplained differences. | P-05 | PLT | Must | MET-16 | `OD-01`, `AD-25`, `VG-10` |
| `REQ-079` | The system SHALL have exactly one implementation owning production writes for a capability at any time and SHALL NOT write the same data blindly to both systems. | P-05 | PLT | Must | MET-32 | `AD-27`, `OD-01` |
| `REQ-080` | The system SHALL run shadow reads for each capability, with the parity thresholds of [§9](#9-migration-success-criteria-and-v1-acceptance), before that capability's cutover. | P-05, P-06 | PLT | Must | MET-15, MET-14 | `AD-26`, `VG-05` |
| `REQ-081` | The system SHALL complete a rehearsed rollback of each wave, covering data, routing and worker ownership, before the wave deploys. | P-05 | PLT | Must | MET-17 | `AD-28`, `VG-12` |
| `REQ-082` | The system SHALL transfer a job or event queue to a new owner only after the old owner has stopped claiming and drained, and SHALL record that no item was processed by both owners. | P-05 | CAP-06 | Must | MET-11, MET-32 | `AD-27`, `VG-12` |
| `REQ-083` | The system SHALL provide operators with an audited command-line surface for provisioning, ownership transfer and dead-letter replay. | P-05 | PLT | Should | MET-31 | `AD-07` |
| `REQ-084` | The system SHALL migrate encrypted AWCMS secrets by re-encryption under AWBMS key management and SHALL NOT accept the legacy encryption format at runtime. | P-05, P-07 | CAP-03 | Must | MET-16 | `OD-03` |

| ID | Acceptance scenario |
|---|---|
| `REQ-078` | **Given** a frozen AWCMS snapshot, **when** the migration runs twice, **then** outputs are identical and resumable after a kill at 50 percent; reconciliation difference count = 0, every exception listed with a cause. |
| `REQ-079` | **Given** capability `blog_content` is AWBMS-owned, **when** a write is sent to AWCMS, **then** it is refused or routed, never applied to both. |
| `REQ-080` | **Given** 14 days of replay (owner), **then** parity for the capability is ≥ 99.9 percent of comparable responses with every difference explained (see [§9](#9-migration-success-criteria-and-v1-acceptance)). |
| `REQ-081` | **Given** a wave deployed in staging, **when** rollback is executed, **then** routing returns to AWCMS within the rollback target, no write is lost or duplicated, and the post-rollback checklist passes. |
| `REQ-082` | **Given** the AWCMS email worker, **when** ownership moves, **then** claim records show a stop time before the AWBMS first claim and zero overlap. |
| `REQ-083` | **Given** an operator replays a dead-letter item by command, **then** one audit record names the operator. |
| `REQ-084` | **Given** an AWCMS-format encrypted secret, **when** migrated, **then** the AWBMS store holds only re-encrypted data and a runtime read of the legacy format fails. |

## 6. Service tiers and product SLO targets

Tiers group capabilities by consequence of failure. The initial values are those of [ADR-0008](../adr/0008-close-od-03-05-06-resolve-assumptions-sign-off-stage-1.md) (`OD-06`) and apply to all tiers unless a tighter or tier-specific value is proposed. Staging is S1 first, S2 only after S1 is proven.

| Tier | Meaning | Capabilities |
|---|---|---|
| **T1** | Critical authentication and tenant path: everything else depends on it | `CAP-01`, `CAP-02`, `CAP-03`, `CAP-09`, audit write of `CAP-05` |
| **T2** | Consumer read and anonymous-write path of AWCMS-Astro | `CAP-10`, `CAP-11`, `CAP-12`, `CAP-13`, `CAP-14` (request), `CAP-15`, `CAP-16` |
| **T3** | Asynchronous and back-office | `CAP-04`, `CAP-05` (query), `CAP-06`, `CAP-07`, `CAP-08`, `CAP-14` (delivery) |

| Target | T1 | T2 | T3 | Status |
|---|---|---|---|---|
| Availability, production, per deployment per month | 99.9 % | 99.9 % | 99.9 % for the API; async processing 99.5 % (owner) | 99.9 % is [D] `OD-06`; 99.95 % only after replicas and HA exist; T3 async value is (owner) `PD-02` |
| Authenticated read p95 | ≤ 500 ms; session and authorisation decision ≤ 300 ms (owner) | ≤ 500 ms; search query ≤ 500 ms; suggestions ≤ 300 ms (owner) | ≤ 500 ms | 500 ms is [D] `OD-06`; tighter values (owner) `PD-04` |
| Authenticated write p95 | ≤ 900 ms | ≤ 900 ms; telemetry accept ≤ 300 ms (owner) | ≤ 900 ms | 900 ms is [D] `OD-06` |
| Asynchronous lag p95 | n/a | n/a | event dispatch ≤ 30 s; email provider hand-off ≤ 5 min (owner) | (owner) `PD-02` |
| Error rate (5xx), 30-day | ≤ 0.1 % (owner) | ≤ 0.5 % (owner) | ≤ 1 % (owner) | (owner) `PD-02` |
| Regression | Blueprint §11 thresholds against the last approved baseline | same | same | [D] `OD-04`, `VG-09` |

Rules: latency targets hold at the concurrent-user count of [§8](#8-capacity-assumptions), measured at origin with edge-cache hits excluded [A]. A target stated only as requests per second, without cache-hit ratio and p95, is not used ([ADR-0008](../adr/0008-close-od-03-05-06-resolve-assumptions-sign-off-stage-1.md)). Absolute values are targets, not measurements, until `VG-09` produces a baseline.

## 7. Recovery objectives per tier

Placeholders in [AWBMS-PRE-PRODUCTION-EVIDENCE.md §4.3](../architecture/AWBMS-PRE-PRODUCTION-EVIDENCE.md#43-vg-12--recovery-and-rollback-evidence) are filled here as proposals. A capability with no row cannot pass `VG-12`; rows below are per tier, each capability inheriting its tier's row unless listed.

| Tier | RTO (owner) | RPO (owner) | Basis |
|---|---|---|---|
| T1 | 60 minutes | 5 minutes | Every other capability depends on identity and tenant data; one primary database on S1 ([AS-06](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#appendix-b--assumption-register), no HA) means recovery is a restore |
| T2 | 120 minutes | 5 minutes (analytics events 60 minutes) | Public sites keep serving from edge cache while AWBMS is down [A]; build-time reads can be retried |
| T3 | 240 minutes | 15 minutes | Outbox and jobs are retried after recovery; delay is tolerable |

Notes. All tiers share one PostgreSQL primary [D], so the achievable RPO is that of the database, and T3's looser value only permits a later restore order. The media object content in the object store has the provider's durability and is outside the database RPO (`PD-01`). Derived data, such as search indexes, is rebuilt rather than restored. A full restore of T1 at 60 minutes exceeds the 43.2 minute monthly budget of a 99.9 percent target, so a disaster month would breach the SLO; whether a disaster event is excluded from the SLO is `PD-01`. Backups must be derived from the RPO, kept in a separate failure domain, and rehearsed on a production-like snapshot ([AS-03](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#appendix-b--assumption-register) confirmed access).

## 8. Capacity assumptions

**Every figure below is an assumption [A] proposed for the owner (owner), not a measurement.** The only load datum available is one AWCMS test (620 requests per second at p95 405 ms) from internal traffic-readiness research (DL-049), which does not describe AWBMS. Public traffic is assumed edge-cached; the S1 → S2 sequence is [D] `OD-06`.

| Dimension | S1 [A] (owner) | S2 [A] (owner) | Rationale |
|---|---|---|---|
| Tenants per environment | ≤ 10 at launch, ≤ 25 by end of S1 | ≤ 100 | Small portfolio; per-tenant sharding is S3 and out of scope (`OD-05`) |
| Sites per tenant | 1–3 | 1–5 | One public Astro site typical |
| Concurrent authenticated users per tenant | 25 (peak 50) | 100 (peak 200) | Editors and administrators only; visitors are anonymous |
| Authenticated API rate, whole deployment, peak | 30 requests/s | 150 requests/s | Back-office plus build-time reads |
| Anonymous browser endpoints at origin, peak | 100 requests/s sustained, 300 burst for 5 min | 300 sustained, 900 burst | Search, suggestions, telemetry, newsletter; edge cache absorbs the rest |
| Build-time consumer burst | 10 builds/hour, each ≤ 500 requests over ≤ 2 min | 40 builds/hour | Consumer fetch of posts, terms, menus, widgets, media, profile |
| Published posts per tenant | ≤ 10,000 | ≤ 50,000 | Drives search sizing (`OD-07`) |
| Media objects per tenant | ≤ 50,000 | ≤ 250,000 | |
| Newsletter subscribers per tenant | ≤ 50,000 | ≤ 250,000 | |
| Audit rows per tenant per year | ≤ 5,000,000 | ≤ 20,000,000 | Drives retention and partitioning later |
| Database size, production | ≤ 100 GB | ≤ 500 GB | Backup and restore time basis for RTO |

Capacity SLOs are claimed only after a distributed load test confirms them at these figures; no capacity claim is made before then.

## 9. Migration success criteria and v1 acceptance

Method is [Blueprint §9](../architecture/AWBMS-MASTER-BLUEPRINT.md#9-migration-strategy--od-01-decision) (`OD-01`): deterministic migration into an AWBMS-native schema, one capability at a time, strangler cutover, rollback tested first. Evidence shapes are [VG-05 and VG-12 in the pre-production evidence specification](../architecture/AWBMS-PRE-PRODUCTION-EVIDENCE.md#41-vg-05--semantic-parity-and-security-corpus). Numeric thresholds below are proposals (owner), `PD-13`, `PD-14`.

| Wave | Success criteria (all must hold) |
|---|---|
| 0 Evidence and foundation | Inventory frozen (`VG-01` passed 2026-11-23); workspace and CI green on one commit (`VG-02`); role and RLS harness runs against a non-superuser role (`VG-06` harness); module registry gate passes; telemetry redaction canary leaks 0 |
| 1 Tenancy and identity | Cross-tenant tests 100 % denied at the three layers (`VG-06`); every protected route passes the chokepoint (`VG-07`); legacy password and machine-credential acceptance proven per `OD-03`; audit write atomic with change; users re-authenticate at cutover |
| 2 Durable runtime | Duplicate-request, parallel-update, job-claim and event-dispatch tests show one effect and zero double processing (`VG-11`); no provider call inside a transaction; data-lifecycle coverage 100 % of data-owning modules (`VG-14`); legal-hold bypass test fails to bypass |
| 3 Content compatibility | All 13 consumed paths replay with 0 unexplained differences (`VG-05`); five execution classes keep their distinct security behaviour; newsletter abuse tests pass; media unresolved identifiers explicit |
| 4 Shadow, parity and cutover | See cutover and rollback criteria below |

Shadow and parity thresholds (owner): shadow reads run for at least 14 days or 100,000 comparable requests per capability, whichever is later; at least 99.9 % of comparable responses are equal after documented normalisation of nondeterministic fields; 100 % of differences are explained; 0 authorisation or tenant-isolation differences are acceptable.

Cutover criteria (a capability is cut over only when all hold; any failing item is No-Go per [validation §54](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#54-gono-go-criteria)): parity thresholds met; rollback rehearsed with measured duration; restore rehearsed against RTO and RPO of its tier; monitoring and alerts live; legacy credential sunset date set; queue ownership transfer procedure rehearsed; accountable approver distinct from the operator signs.

Observation and rollback (owner): observation window of 7 days after each cutover; automatic rollback trigger, or No-Go to proceed, if any of: any cross-tenant exposure; 5xx rate above 2 % for 10 minutes; p95 above the SLO by 50 % for 30 minutes; reconciliation shows an unexplained lost or duplicated write; double-processing of a job or event. Rollback is complete only after the post-rollback checklist passes ([validation §42](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#42-rollback-model)). A wave is never accepted on an HTTP 200 from the health endpoint alone.

**Overall v1 acceptance.** v1 is accepted when: all `Must` requirements pass their acceptance scenarios; all sixteen capabilities are AWBMS-owned in production or have a recorded, owner-approved exception; the AWCMS-Astro consumer runs against AWBMS for its 13 consumed paths with no consumer regression; the SLOs of [§6](#6-service-tiers-and-product-slo-targets) are met over a 30-day window after the last cutover; `VG-05`, `VG-06`, `VG-07`, `VG-08`, `VG-09`, `VG-10`, `VG-11`, `VG-12`, `VG-13` and `VG-14` are PASS with committed evidence; no open Go/No-Go item; and the owner has closed every `PD-` row that blocks acceptance.

## 10. Metrics and instrumentation plan

Instrumentation is defined now so that step 8 and step 11 build it, not retrofit it. Required telemetry content follows [Blueprint §21](../architecture/AWBMS-MASTER-BLUEPRINT.md#21-observability) (`AD-20`). Metrics must carry no secret or unnecessary personal data.

| ID | Metric | Kind | Measured where |
|---|---|---|---|
| `MET-01` | Cross-tenant test pass rate (target 100 %) and production cross-tenant denial events | Operational, security | CI `tests/rls`; audit and policy-decision telemetry |
| `MET-02` | Availability per deployment per month | Operational | External synthetic probe, uptime monitor |
| `MET-03` | Latency p50, p95, p99 by route class and tier | Operational | Request telemetry histograms; load test (`VG-09`) |
| `MET-04` | Error rate by class (4xx, 5xx, overload) | Operational | Request telemetry |
| `MET-05` | Policy decisions allow and deny by operation; routes outside the chokepoint (target 0) | Security | Decision telemetry; static route-to-policy check (`VG-07`) |
| `MET-06` | Audit completeness: privileged operations with an audit record ÷ privileged operations (target 100 %) | Security | Reconciliation job over audit store and operation counters |
| `MET-07` | Revocation-to-deny latency for sessions, credentials, suspended tenants | Security | Synthetic probe; test suite |
| `MET-08` | Sign-in failures, lockouts, step-up events, password rehash count | Security | Authentication telemetry |
| `MET-09` | Idempotent replays, key conflicts, duplicate-effect count (target 0) | Operational | Idempotency store; `tests/concurrency` |
| `MET-10` | Outbox dispatch lag p95 and age of oldest undispatched event | Operational | Outbox telemetry |
| `MET-11` | Job queue depth, age, retries, dead-letter count, double-claim count (target 0) | Operational | Job telemetry; claim records |
| `MET-12` | Email hand-off latency p95, bounce rate, complaint rate | Product, operational | Email capability telemetry; provider callbacks |
| `MET-13` | Database pool wait, connections, transaction duration, lock wait | Operational | Database telemetry |
| `MET-14` | Consumer-contract verification pass and unexplained parity differences (target 0) | Compatibility | `tests/contract`, `tests/parity` |
| `MET-15` | Shadow-read parity rate and difference classes | Migration | Shadow comparison log |
| `MET-16` | Reconciliation difference count per migration batch (target 0 unexplained) | Migration | Migration ledger and reconciliation report |
| `MET-17` | Rollback duration, restore time versus RTO, data-loss window versus RPO | Recovery | `VG-12` rehearsal record |
| `MET-18` | Age of last successful backup and archive | Recovery | Backup monitor |
| `MET-19` | Records past retention, legal-hold violations (target 0), time to complete subject requests, lifecycle coverage | Privacy | Lifecycle job telemetry; `VG-14` coverage check |
| `MET-20` | Redaction canary leak count (target 0) | Security | CI canary test; log scan |
| `MET-21` | Age of open critical and high dependency findings; SBOM present per release | Supply chain | Dependency scanners; release gate (`VG-13`) |
| `MET-22` | Newsletter subscribe, confirm and unsubscribe counts; cooldown and rate-limit refusals; complaint rate; response-indistinguishability test pass | Product, security | Newsletter telemetry; security tests |
| `MET-23` | Telemetry events accepted, dropped, rate-limited; stored-identifier audit (target 0 direct identifiers) | Product, privacy | Analytics telemetry; schema audit |
| `MET-24` | Search latency, zero-result rate, index freshness lag | Product, operational | Search telemetry |
| `MET-25` | Media pending and failed counts; unresolved-identifier counts | Operational | Media telemetry |
| `MET-26` | Consumer build-time fetch success rate (target ≥ 99.9 %) | Product | Synthetic consumer build; consumer build logs |
| `MET-27` | Editor task time to publish a post; first-attempt success | Product | UAT and prototype sessions (see research document) |
| `MET-28` | Architecture-gate results: module DAG, route collisions, ownership, contract drift, independence | Governance | CI (`VG-02`, `VG-08`, `VG-15`) |
| `MET-29` | Overload responses, queue saturation, shed requests | Operational | Request and runtime telemetry |
| `MET-30` | Regression deltas against baseline for the §23.1 scenarios | Performance | `VG-09` run bundle |
| `MET-31` | Tenant provisioning duration | Product, operational | Operator audit timestamps |
| `MET-32` | Production owner per capability and queue; deployments per environment | Migration | Ownership register and deployment inventory |

## 11. Won't have

MoSCoW **Won't** for v1. Each item may return only through a new decision.

| Item | Reason |
|---|---|
| Migration of the eight capabilities that remain in AWCMS (`sync_storage`, `workflow`, `reporting`, `theming`, `form_drafts`, `comments`, `idn_admin_regions`, `push_delivery`) | `OD-09`, `AD-S1-04` |
| Compatibility guarantees for AWCMS endpoints outside the consumed surface | `OD-03` |
| Acceptance of AWCMS human session tokens or other legacy token namespaces after cutover | `OD-03` |
| Permanent dual-write between AWCMS and AWBMS | `AD-27`, Blueprint §4 |
| Per-tenant database sharding and a dedicated deployment per tenant by default | `OD-05` |
| Multi-region or active-active deployment; availability above 99.9 % before replicas and HA | `AS-06`, `OD-06` |
| Message broker, Redis or Valkey as a correctness dependency | `AD-13`, `AD-18` |
| Microservice split of any module | `AD-07` |
| Absolute performance claims without a controlled benchmark | Blueprint §4, `VG-09` |
| Claims of ISO, NIST, OWASP or Indonesian regulatory compliance | Blueprint §4; control alignment only |
| A new AWBMS-owned administrative or editor user interface inside v1, unless step 4 shows a slice needs one | ADR-0004, `PD-03` |
| Blockchain, custody or ledger features | Out of the v1 capability list; the repository's research notes are not requirements |

## 12. Traceability

Each `REQ-` row is the anchor that later steps reference. Steps 3 to 11 add derived identifiers: `UI-` (step 4), `TBL-` (step 5), `PERM-` (step 6), `API-` and `EVT-` (step 8), `THR-` (step 3), `TST-` (steps 7 to 11). This PRD defines none of them. The matrix below is the empty placeholder that those steps fill, one row per `REQ-`.

| REQ | CAP | UI- | TBL- | PERM- | API- | EVT- | THR- | TST- |
|---|---|---|---|---|---|---|---|---|
| `REQ-nnn` | `CAP-nn` | | | | | | | |

Every `Must` requirement must reach at least one `TST-` before gate 9; a requirement with no test is a Definition-of-Ready defect. Business goal to capability to acceptance criteria is the chain of [Blueprint WP-4](../architecture/AWBMS-MASTER-BLUEPRINT.md#29-immediate-stage-1-work-packages).

## 13. Owner decisions

Every (owner) value in this document is collected here. **All rows accepted as proposed on 2026-11-23** ([ADR-0009](../adr/0009-accept-stage-2-prd-owner-decisions-pd-01-22.md)); the "Proposed value" column is the decided value. `PD-11` and `PD-12` remain subject to legal review, and `PD-16` assigns `OD-07` an owner without closing it. "Blocks" names the later step that cannot be finished without it.

| ID | Question | Proposed value | Rationale | Blocks |
|---|---|---|---|---|
| `PD-01` | RTO and RPO per tier; does a disaster event count against the 99.9 % SLO; object-store content recovery | T1 60 min / 5 min; T2 120 min / 5 min (analytics 60 min); T3 240 min / 15 min; disaster excluded from the SLO | One primary database, no HA at S1; a 60 min restore exceeds the 43.2 min monthly budget | `VG-12` rehearsal criteria; gate 9 |
| `PD-02` | T3 async availability, lag and per-tier error-rate targets | Async 99.5 %; dispatch lag p95 ≤ 30 s; email hand-off ≤ 5 min; 5xx ≤ 0.1 / 0.5 / 1 % (T1/T2/T3) | Async work tolerates delay; API stays 99.9 % | Alert design; gate 9 |
| `PD-03` | Does v1 deliver any AWBMS-owned screen for P-03 and P-04, or only contracts consumed by existing admin surfaces | Decide per slice at step 4; assume contract-only until then | AWBMS has no frontend; ADR-0004 | Step 4 prototype scope; research validation |
| `PD-04` | Per-capability latency tightening | Session and authorisation decision p95 ≤ 300 ms; suggestions ≤ 300 ms; telemetry accept ≤ 300 ms | Decision cost is paid by every request; keep 500 / 900 ms as outer bounds | `VG-09` scenario targets |
| `PD-05` | Capacity assumptions S1 and S2 ([§8](#8-capacity-assumptions)) | As tabled, including 25 concurrent users per tenant at S1 | No AWBMS workload measured; sizing for the portfolio | Load-test design; gate 9 |
| `PD-06` | Session lifetimes and step-up window | Idle 30 min, absolute 12 h, step-up 10 min | Common practice; no AWCMS value re-derived here | Step 7 session algorithm |
| `PD-07` | Revocation propagation limit for sessions, credentials, suspended tenants | 5 s sessions and credentials; 60 s tenant suspension | Server-side sessions allow immediacy; tenant state may be read less often | Step 7; `VG-05` |
| `PD-08` | Lockout parameters | 5 failed attempts, 15 min lock, independent of client address | Matches the frozen AWCMS defaults [F] `rate-limits.json`; confirm they remain right | Step 7 |
| `PD-09` | Machine credential maximum lifetime; legacy credential sunset | 365 days; legacy rejected 30 days after wave 3 cutover | `OD-03` requires a per-wave date | Wave 3 migration plan |
| `PD-10` | Newsletter abuse limits | 5 attempts per client per 300 s; one confirmation per recipient per 900 s; telemetry limit 100 events per client per 10 s | First two are frozen AWCMS defaults [F] (`C-07` record); third is new | Abuse-control spec (step 7) |
| `PD-11` | Retention periods per data category, including audit, operational logs, analytics, suppression lists; legal review | Analytics raw 90 days; operational logs 30 days; audit 2 years; confirm after legal review | Retention is a business and legal choice | `VG-14` descriptors |
| `PD-12` | Data-subject request deadline | 14 days | Internal target pending legal advice on statutory duties | `VG-14`; privacy analysis (step 3) |
| `PD-13` | Shadow and parity thresholds | 14 days or 100,000 comparable requests; ≥ 99.9 % equal; 100 % explained; 0 authorisation differences | Strict on security, tolerant of nondeterministic fields | Wave 4 plan; `VG-05` |
| `PD-14` | Observation window and rollback triggers per cutover | 7 days; triggers per [§9](#9-migration-success-criteria-and-v1-acceptance) | Short enough to be actionable, long enough for weekly cycles | `VG-12`; wave 4 |
| `PD-15` | Which visitor identifiers analytics may store; is a coarse one acceptable | No raw network address; no persistent identifier across sites | Privacy-first redesign (Blueprint §8.2) | Privacy analysis; `TBL-` design |
| `PD-16` | `OD-07` (search store) is unassigned and open | Assign an owner; measure corpus size and query shape at S1 | PRD fixes behaviour, not technology | First search implementation |
| `PD-17` | Tenant provisioning time target; the separation-of-duties catalogue (which duty pairs conflict) | 5 min; start with role-grant request versus approval | Operator effort; SoD must be listed to be testable | Step 6 matrix |
| `PD-18` | Event and job retry limit and dead-letter policy | 8 attempts with backoff | Bounded retry, visible failure | Step 7 |
| `PD-19` | Request limits | Anonymous body ≤ 16 KiB; authenticated body and timeouts decided at step 8 | Bound abuse surface | Step 8 contract |
| `PD-20` | Restore rehearsal cadence and incident severity scale with response times | Quarterly rehearsal; severity scale and times by the owner | Pre-production evidence leaves both `<owner-set>` | `VG-12`; incident readiness |
| `PD-21` | Search freshness and unpublish propagation | 60 s | Unpublished content must not linger | `REQ-057`; search design |
| `PD-22` | Is `P-08` (derived-application / module developer) a v1 user group | Not in v1: module admission stays internal (Blueprint §14); revisit when a derived product needs AWBMS | Research rates `P-08` partially evidenced (`UA-14`); adding it would widen the v1 surface | Step 4 and step 8 scope |

Escalated, not decided: `PD-01`, `PD-03` and `PD-16` touch decisions that Stage 1 left open or that depend on later steps. No binding decision of Stage 1 is changed by this document. Observation for the owner: ADR-0008 states the SLO as per deployment while a shared deployment serves every tenant; a per-tenant SLO would need separate measurement, which this PRD does not propose.
