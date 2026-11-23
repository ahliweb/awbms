# AWBMS Stage 2 — User Research Input

| Field | Value |
|---|---|
| Status | Draft — research input for Stage 2 PRD |
| Date | 2026-11-23 |
| Method | desk research of existing usage artifacts; no interviews conducted |
| Lifecycle step | Stage 2, step 2 "User Research + PRD" (tracking issue [#61](https://github.com/ahliweb/awbms/issues/61)) |
| Consumer | The Stage 2 PRD traces requirements to the persona IDs `P-01` to `P-07` below |
| Claim labels | `[F]` fact (verified, source cited) · `[C]` claim from a source not independently verified · `[A]` assumption · `[O]` open question |

This document is research input, not a requirements document. It does not
restate Blueprint decisions, and nothing in it is a requirement. Where
evidence is missing it says so.

## Contents

- [1. Scope, method and evidence base](#1-scope-limits-and-evidence-base)
- [2. Usage scale evidence](#2-usage-scale-evidence)
- [3. Usage contexts](#3-usage-contexts)
- [4. Personas and jobs-to-be-done](#4-personas-and-jobs-to-be-done)
- [5. Recurring pain themes from issues](#5-recurring-pain-themes-from-issues)
- [6. Untested assumptions](#6-untested-assumptions)
- [7. Research gaps and recommended validation plan](#7-research-gaps-and-recommended-validation-plan)
- [8. Source index](#8-source-index)

## 1. Scope, limits and evidence base

**Honest framing.** AWBMS is a backend. Its users are reached mostly through
consumers (AWCMS-Astro sites, AWCMS admin UI, integrations). No interview,
survey, observation session or analytics export was available to this
research **[F]** (method is desk research only). The evidence is therefore
what existing artifacts reveal about *how the predecessor surface is built
to be used*, plus what maintainers wrote down when it hurt. Artifacts show
designed use and reported failure; they do not show who the people are, how
many there are, or what they value. Every persona below inherits that limit.

Confidence scale used per persona:

- **evidenced** — at least one artifact shows real usage or a real incident
  for this actor;
- **partially evidenced** — the actor is implied by built interfaces or by
  design documents, but no usage or incident data shows real use;
- **assumption only** — nothing in the sources supports the persona beyond
  its definition.

### 1.1 Sources and the baseline each one is pinned to

| Ref | Source | Pinned to | Notes |
|---|---|---|---|
| S1 | Frozen VG-01 corpus, `contracts/legacy/awcms/` and [VG-01 PASS record](../architecture/evidence/VG-01-PASS-2026-11-23.md) | AWCMS `11f2e95a47b1328a820f976d60f978c38a067903` (`v10.1.0`), AWCMS-Astro `7b753be619244541b817d5d8e7d3b72cfe88d4f9` | Frozen and CI-checked **[F]** (PASS record). Cited by default. |
| S2 | AWCMS source checkout (read-only) | HEAD `6ab60abc2cc35a3243e1211185c773f25828a7f9` (`v10.12.0` plus 44 commits, dated 2026-11-23) | **Newer than the frozen baseline [F]** (`git describe`). Cited only where stated as S2; baseline-sensitive counts always come from S1. |
| S3 | AWCMS-Astro checkout (read-only) | HEAD `c7b76aa41534dab6e56a8713d0a05166dbfafbf2` (2026-11-20) | **Newer than the frozen Astro baseline [F]**. Tests and `src/lib/awcms` cited as S3. |
| S4 | One downstream site derived from AWCMS-Astro, found on disk | not pinned | Used only to establish that and how the surface is consumed. Not named: the repository is not public. No content copied. |
| S5 | GitHub issues, `ahliweb/awcms` (200 most recent of all states), `ahliweb/awcms-astro` (30), `ahliweb/awbms` (26) | read 2026-11-23 | Support-ticket proxy. The awcms list is truncated at 200 **[F]**, so older issues are reachable only through targeted search. |
| S6 | Blueprint and validation | Stage 1 sign-off | Context only; not restated. |

Not accessed: production telemetry, access logs, CDN statistics, tenant
databases, support mailboxes, any interview material **[F]** (none provided).

## 2. Usage scale evidence

**Result: no production usage-volume evidence found.** No tenant count, site
count, request rate, active-user count or data-volume figure from a running
production deployment appears in S1 to S6. The numbers below describe
*built surface* and *test or reference corpora*, not real traffic. They must
not be copied into the PRD as capacity inputs.

| Datum | Value | Kind | Source |
|---|---|---|---|
| Route files / handlers in AWCMS at baseline | 388 / 483 | built surface | `contracts/legacy/awcms/frozen/static/routes.json:4` **[F]** |
| Handlers by auth class | session 362, public 63, admin 50, machine-credential 6, indeterminate 2 | built surface | same, `summary.auth_class` **[F]** |
| Modules at baseline / in S2 checkout | 24 / 40 directories | built surface | `frozen/static/modules.json:3` **[F]**; `ls src/modules` in S2 **[F]** |
| Tables / RLS-enforced / policies at baseline | 152 / 134 / 143 | built surface (catalog of the frozen live schema) | `frozen/live/summary.json:10,18` **[F]** |
| Consumed public endpoints by the Astro reader | 13 consumed, 2 committed-future | built surface | `frozen/static/astro-consumer.json:4-21` **[F]** |
| Largest news archive known | 25,029 legacy articles in one archive to be turned into a tenant | reference corpus, not live traffic | awcms-astro [#75](https://github.com/ahliweb/awcms-astro/issues/75), awcms ADR-0114 **[C]** (ADR text read in S2; not frozen) |
| Build ceiling in the Astro reader | 50 per page x 400 pages = 20,000 posts, then the build throws | code limit | awcms-astro [#75](https://github.com/ahliweb/awcms-astro/issues/75) **[C]** |
| Uncached public page throughput, laptop test | clean at 300 views/s per 2-CPU app; 5xx 18.8% at 400/s | synthetic local load test, no CDN, tmpfs database | awcms [#1454](https://github.com/ahliweb/awcms/issues/1454) **[C]** (issue self-labels it LOCAL) |
| Earlier family load test | about 620 rps via CDN, 452 rps at origin, limit set by the test client | synthetic, July 2026, cited second-hand | awcms ADR-0161 text in S2 **[C]** |
| Deployed environments | production and staging, one host | deployment fact, no volume | awcms `docs/PROJECT_STATE.md` in S2 **[C]** |

Further statements:

- **No tenant or site count exists in any source [F].** The frozen live
  summary counts schema objects only; `runtime_owned_table_count` and role
  counts say nothing about tenants (`frozen/live/summary.json`).
- **Consumer count is not established [O].** One downstream site derived from
  AWCMS-Astro was found on disk and it calls the content, media, search,
  newsletter, analytics and site-profile endpoints (S4, endpoint paths only).
  Other repositories on the same disk use other AWCMS-family shells; whether
  they are production consumers of the public API was not determined. The
  PRD must not state a consumer count.
- **Concurrent users per tenant: no evidence.** The AWCMS capacity decision
  itself says the admin target must be set "at a stated number of concurrent
  users per tenant" and none is stated (ADR-0161 text, S2) **[C]**. See `UA-02`.
- **Data volume:** the only volume-like number is the 25,029-article archive
  above, a legacy-site reference, not a measured tenant.

## 3. Usage contexts

Each context states what the artifacts show. Contexts are the places where
requests to the backend originate; they are not personas.

### 3.1 Anonymous browser on a public site (build-time static, runtime calls)

- The Astro reader builds a static site from authenticated, machine-credential
  reads (`/blog/posts`, `/blog/terms`, `/blog/menus`, `/blog/widgets`,
  `/media/objects`, `/media/public-origin`, `/site-profile/composed`) **[F]**
  (`astro-consumer.json:5-14` execution class `build-time-machine-authenticated-read`).
- A visitor's own browser then calls five endpoints cross-origin with no
  credential: `/site-search/query`, `/site-search/suggest` (reads),
  `/analytics/collect` (telemetry write), and the three newsletter writes **[F]**
  (`astro-consumer.json`, classes `browser-anonymous-cross-origin-*`).
- The reader code deliberately has no retry on build calls; a failed build
  that ships nothing is preferred to a build that ships a site with sections
  silently empty (`src/lib/awcms/client.ts:119-126` in S3) **[F]**. This is
  a stated operator preference about failure mode, not a measured need.

### 3.2 Newsletter lifecycle on a public site

- Subscribe is anonymous, rate-limited per client IP, answers the same
  neutral body for new, active, suppressed or malformed addresses, and uses
  double opt-in; unsubscribe takes a token and nothing else (awcms-astro
  [#79](https://github.com/ahliweb/awcms-astro/issues/79)) **[C]**.
- The frozen rate-limit inventory confirms the limiter has one dimension
  (`client_ip`) and a per-recipient cooldown is enforced separately inside the
  upsert (`frozen/static/rate-limits.json:2948`, claim `C-07`, verdict
  `confirmed`) **[F]**.

### 3.3 Tenant staff in the admin UI

- AWCMS ships an admin surface of 50 admin-class handlers and a large
  session-authenticated API behind it; blog-content alone owns 94 handlers
  (9 admin, 70 session, 15 public) and tenant-admin 16 (`routes.json`,
  recomputed from `routes[]` by owner and auth class) **[F]**.
- Role vocabulary in AWCMS docs is a planned ERP/retail catalogue (Owner,
  Admin, Finance Approver, Payroll Admin, Auditor, ...) that the document
  itself labels "(planned)" (`docs/awcms/17_default_seed_rbac_abac.md:80`
  in S2) **[C]**. No editor-specific role was found in the sources read, so the
  mapping from "editor" to a concrete role is not evidenced **[O]**.

### 3.4 Operator and recovery work

- Production and staging run on one host with separate database and
  secrets; the maintainers record a trap in which a superuser database user
  makes RLS inert (`docs/PROJECT_STATE.md` in S2) **[C]**.
- Migrations are a 148-file ordered set with per-file checksums and a
  ledger; the runner is `scripts/db-migrate.ts` (`frozen/live/summary.json:5`)
  **[F]**. Migration immutability had no CI gate at the baseline (VG-01 PASS
  record, claim `C-04` verdict) **[F]**.
- Backup and restore drill assurance was a tracked work item (awcms
  [#812](https://github.com/ahliweb/awcms/issues/812)), and drills for
  newer sensitive domains were still open (awcms
  [#968](https://github.com/ahliweb/awcms/issues/968)) **[C]**.
- The production-preflight runbook states that its own orchestrator does not
  exist and is a specification (`docs/awcms/production-preflight-runbook.md:5-12`
  in S2) **[C]**. So the operator workflow evidence is a design document, not
  an observed practice.

### 3.5 Machine and integration clients

- Machine credentials bind the tenant: a valid credential with no asserted
  tenant resolves to the credential's tenant (`ASTRO-TENANT-001` in
  `contracts/legacy/awcms/authorization/vectors.json`, citing the Astro
  contract test "token mesin menentukan tenant tanpa variabel lain") **[F]**.
- The Astro reader is configured by an API URL, a token and optionally a
  tenant code, plus a timeout (`src/lib/awcms/client.ts`, `tenant.ts` in S3)
  **[F]**.
- Machine-credential handlers are few: six, of which five are the sync
  protocol and one is session-handoff redeem (`routes.json`, filtered on
  `auth_class`) **[F]**. Machine access to content is otherwise carried by the
  consumed reads above.
- Native-client access (OIDC with PKCE, rotating refresh token,
  device-bound session) was a requested but unbuilt capability at the time of
  reading (awcms [#1406](https://github.com/ahliweb/awcms/issues/1406)) **[C]**.

### 3.6 Privacy and audit handling

- AWCMS documents an access/export and maker-checker erasure flow per table
  descriptor, and states restriction/objection is only approximated by
  deactivating a user and revoking sessions
  (`docs/awcms/privacy-analysis.md:103-123` in S2) **[C]**.
- The data-lifecycle module covers retention, archive, purge and legal hold
  (`docs/awcms/data-lifecycle.md` in S2) **[C]**.
- Audit gaps were reported as defects: audit table not append-only for the
  application role (awcms [#1341](https://github.com/ahliweb/awcms/issues/1341)),
  login without audit trail (awcms [#145](https://github.com/ahliweb/awcms/issues/145))
  **[C]**.

## 4. Personas and jobs-to-be-done

Persona IDs are fixed by the PRD author and are validated here, not renamed.
Jobs use the form "when (situation), the persona wants (outcome)". None of
the jobs was confirmed with a person **[A]** as a job statement; the evidence
column says what supports the situation, not the wording.

### 4.1 Summary table

| ID | Persona | Confidence | Evidence in one line |
|---|---|---|---|
| `P-01` | Public site visitor | evidenced (as designed traffic) | Five browser-anonymous endpoints are consumed by the Astro reader; no visitor data |
| `P-02` | Newsletter subscriber | evidenced (as designed flow) | Three anonymous endpoints with frozen limiter inventory and an abuse-driven cooldown; no subscriber data |
| `P-03` | Tenant content editor | partially evidenced | Admin and session blog APIs exist; one reported publish failure; no role or editor data |
| `P-04` | Tenant administrator | partially evidenced | Tenant-admin, tenant-domain, site-profile routes exist; no usage or role data |
| `P-05` | Platform operator / SRE | evidenced (strongest of the human personas) | Migration ledger, deploy and DR tickets, load-test and pool-saturation findings written by operators |
| `P-06` | Integration client / consumer developer | evidenced | Frozen consumer map, contract test, many consumer-authored issues |
| `P-07` | Security & privacy officer / data subject handling | partially evidenced | Documented data-subject flows and audit defects; no request volumes or officer activity |
| `P-08` | Derived-application / module developer (added) | partially evidenced | Blueprint scope and AWCMS derived-app tickets; see section 4.9 |

### 4.2 `P-01` Public site visitor

- **Confidence:** evidenced as designed traffic; the person is not
  evidenced **[F]** (frozen consumer map). Nothing says who visitors are,
  where they connect from or on what devices **[O]**.
- **Usage context:** section 3.1. Reaches AWBMS only through an AWCMS-Astro
  site; no credential; cross-origin browser calls to search and analytics.
- **Evidence:** `frozen/static/astro-consumer.json:4-14` and routes
  classified as 63 public handlers. The Astro repository carries a public
  reading-preferences (accessibility) panel and a fix for accessibility
  defects found by an automated checker, which shows accessibility is a
  stated product concern for visitor-facing pages (awcms-astro
  [#174](https://github.com/ahliweb/awcms-astro/issues/174),
  [#179](https://github.com/ahliweb/awcms-astro/issues/179)) **[C]**.
- **Jobs:**
  - `J-01` (`P-01`): when reading a site, find an article by searching or
    typing ahead and get a fast answer. Basis: search and suggest consumed
    anonymously; a prefix-match search-as-you-type was shipped (awcms
    [#1305](https://github.com/ahliweb/awcms/issues/1305)) **[C]**.
  - `J-02` (`P-01`): have pages load quickly and stay available while many
    others read. Basis: public-page saturation finding (awcms
    [#1454](https://github.com/ahliweb/awcms/issues/1454)) and the stated
    goal that public pages be CDN-cacheable (awcms
    [#1284](https://github.com/ahliweb/awcms/issues/1284)) **[C]**. The visitor's own
    expectation is **[A]**.
  - `J-03` (`P-01`): browse without being tracked beyond what is necessary.
    Basis: the beacon endpoint is anonymous and documented with a privacy-first
    stance and a UU PDP mapping (`docs/awcms/visitor-analytics.md` in S2)
    **[C]**. Visitor attitudes to analytics are **[A]**.
- **Contradiction to note:** `visitor-analytics.md` says the module has no
  code in AWCMS, while the frozen route inventory has 14 visitor-analytics
  handlers and the Astro reader consumes the collect endpoint **[F]**. The
  document is stale; the frozen inventory prevails for the baseline.

### 4.3 `P-02` Newsletter subscriber

- **Confidence:** evidenced as designed flow. No subscriber counts, no
  abuse incident count, no bounce data.
- **Usage context:** section 3.2. Subscribe, confirm by link, unsubscribe by
  token only. The person never authenticates.
- **Evidence:** consumed classes in `astro-consumer.json:13-15`; limiter sites
  RL-031 to RL-036 and the cooldown control in `rate-limits.json:2750`;
  design of neutral responses and double opt-in in awcms-astro
  [#79](https://github.com/ahliweb/awcms-astro/issues/79) **[C]**. The
  rate-limit claim that per-IP limits alone could not stop repeated mail to a
  victim address is verified as design reasoning, not as an observed attack
  **[F]** (claim `C-07`, `rate-limits.json:2948`).
- **Jobs:**
  - `J-04` (`P-02`): subscribe with an email address and be certain the
    mailbox owner consented. Basis: double opt-in, `consent_at` set only at
    confirmation **[C]**.
  - `J-05` (`P-02`): unsubscribe in one step without proving identity. Basis:
    token-only unsubscribe **[C]**.
  - `J-06` (`P-02`): not be flooded with mail triggered by someone else.
    Basis: recipient cooldown and the opt-out propagation work (awcms
    [#1478](https://github.com/ahliweb/awcms/issues/1478)) **[C]**. This
    job belongs equally to a third party whose address is abused; that person
    never interacts, so the job is **[A]** as a user need.

### 4.4 `P-03` Tenant content editor

- **Confidence:** partially evidenced. Interfaces for authoring exist; no
  data on editors, their numbers or their tools.
- **Usage context:** section 3.3. Creates, edits and publishes posts and
  media through admin routes and session-authenticated APIs.
- **Evidence:** 94 blog-content handlers and 13 media handlers at the baseline
  (`routes.json` summary and owner counts) **[F]**. Reader-side defects show
  the authoring-to-publish chain as seen from outside: an article written in
  the CMS's own editor was never published because the tab was read from a
  sidecar the CMS could not write (awcms-astro
  [#73](https://github.com/ahliweb/awcms-astro/issues/73)); bodies were
  rendered as unstyled prose until a structured body was read (awcms-astro
  [#74](https://github.com/ahliweb/awcms-astro/issues/74)) **[C]**. A cache-on
  condition made blog publishing fail with a 500 (`docs/PROJECT_STATE.md` in
  S2, tenant GUC) **[C]**.
- **Jobs:**
  - `J-07` (`P-03`): publish an article and see it on the public site
    correctly formatted, with its SEO title, section and image. Basis: the
    three reader issues above and seoTitle consumption (awcms-astro
    [#193](https://github.com/ahliweb/awcms-astro/issues/193)) **[C]**.
  - `J-08` (`P-03`): manage media (upload, reuse) for posts without
    breaking links. Basis: media objects and public-origin endpoints are
    consumed **[F]**. How editors use media is **[A]**.
- **Not evidenced:** editor count per tenant, session length, draft volume,
  concurrent editing, review or approval practice **[O]** (`UA-03`).

### 4.5 `P-04` Tenant administrator

- **Confidence:** partially evidenced. The surface exists; usage is not
  shown.
- **Usage context:** section 3.3. Users, roles, domains, site profile.
- **Evidence:** 16 tenant-admin, 8 tenant-domain and 4 site-profile handlers
  (`routes.json` owner counts) **[F]**. The tenant-domain module resolves
  host to tenant and verifies domains; a DNS reconciliation job exists with
  no default serving target "because guessing means" a wrong record
  (`docs/PROJECT_STATE.md` in S2) **[C]**. Defects touching this persona:
  the login flow had no enrolment step so a user without a second factor was
  locked out when tenant MFA policy required one (awcms
  [#1425](https://github.com/ahliweb/awcms/issues/1425), closed) **[C]**;
  a password change in one tenant could overwrite a credential shared across
  tenants (awcms [#1035](https://github.com/ahliweb/awcms/issues/1035),
  closed) **[C]**.
- **Jobs:**
  - `J-09` (`P-04`): invite, deactivate and re-role tenant users without
    locking people out or leaving stale access. Basis: #1425, #1035 **[C]**.
  - `J-10` (`P-04`): attach a custom domain to the tenant's site and know
    when it is live. Basis: tenant-domain module and DNS reconciliation **[C]**.
    How often this occurs is **[A]**.
- **Not evidenced:** whether tenant administrators are the same people as
  site owners, tenant sizes, number of admins per tenant **[O]**.

### 4.6 `P-05` Platform operator / SRE

- **Confidence:** evidenced. This persona leaves the most artifacts because
  it is the maintainer's own role.
- **Usage context:** section 3.4.
- **Evidence:** migration set and ledger, `frozen/live/summary.json:5` and
  `frozen/live/migration-ledger.csv` **[F]**; 82 gates in CI of which 71
  enforced, 4 advisory, 7 absent (VG-01 PASS record, `gates.json`) **[F]**;
  recurring operator-written findings in section 5 (pool saturation, limiter
  per process, proxy host detection #1366, provider-outage observability
  #1412, circuit-breaker probe #1409, backup assurance #812). A repeated
  operator rule: do not default what cannot be guessed (DNS target) and
  refuse to run with an unsafe configuration (rate limiter without Redis
  across instances, awcms [#1285](https://github.com/ahliweb/awcms/issues/1285)) **[C]**.
- **Jobs:**
  - `J-11` (`P-05`): deploy a change and migrate the database with a
    rehearsed, reversible path, and know before cutover whether tenants
    isolate correctly. Basis: migration ledger, RLS catalog checks,
    preflight runbook (a specification) **[F]/[C]**.
  - `J-12` (`P-05`): see load, queue age and failures in time to act, and
    shed load deliberately rather than fall over. Basis: pool-saturation
    finding with CPU at 35-46% (#1454), unwired latency metric (awcms
    [#1287](https://github.com/ahliweb/awcms/issues/1287)), provider-outage
    observability (#1412) **[C]**.
  - `J-13` (`P-05`): restore service and data after a failure with a measured
    recovery time. Basis: backup assurance (#812) and the restore-drill
    scenario set that records `restoreRtoSeconds` and `restoreRpoSeconds`
    (`docs/awcms/resilience-dr-verification.md:115-131` in S2) **[C]**. RTO and
    RPO values the business wants are **[O]**.
- **Not evidenced:** the size of the operator team, on-call practice, incident
  frequency, actual downtime, actual restore times from production **[O]**.

### 4.7 `P-06` Integration client / consumer developer

- **Confidence:** evidenced. Contract, tests and issue stream are
  consumer-authored.
- **Usage context:** sections 3.1 and 3.5.
- **Evidence:** 13 consumed endpoints classified by execution class
  (`astro-consumer.json`) **[F]**; 14 structured authorization vectors, some
  from the consumer's own contract test (`authorization/vectors.json`) **[F]**;
  16 mocked-fetch exchanges in that test and no recorded live responses and
  no body examples in the OpenAPI at the baseline (`frozen/static/behaviour-fixtures.json`
  classes) **[F]**. The consumer reports its own pain: build throws above
  20,000 posts (#75), cannot issue redirects (#77), cannot reach newsletter
  (#79), twelve accepted backend decisions absorbed nowhere (awcms-astro
  [#76](https://github.com/ahliweb/awcms-astro/issues/76)) **[C]**. Contract
  hygiene work: example responses (awcms
  [#1391](https://github.com/ahliweb/awcms/issues/1391)), declaring
  `429 RATE_LIMITED` (awcms [#1385](https://github.com/ahliweb/awcms/issues/1385)),
  two-way mock and contract tests (awcms
  [#1379](https://github.com/ahliweb/awcms/issues/1379)) **[C]**.
- **Jobs:**
  - `J-14` (`P-06`): build a site from the backend and fail loudly rather than
    publish partial content. Basis: client has no retry, build throws on
    truncation (`client.ts:119-126`, #75) **[F]/[C]**.
  - `J-15` (`P-06`): know the contract (shapes, errors, rate-limit responses)
    without reading backend source. Basis: missing examples and recorded
    responses at the baseline, #1391, #1385 **[F]/[C]**.
  - `J-16` (`P-06`): have the tenant derived from the credential and be
    unable to read another tenant's data. Basis: `ASTRO-TENANT-001`;
    credentials are machine-bound **[F]**.
- **Not evidenced:** number of consumers and integrations in production,
  their build frequency, who rotates credentials and how **[O]** (`UA-05`).

### 4.8 `P-07` Security & privacy officer / data subject handling

- **Confidence:** partially evidenced. Controls and defects are documented;
  no officer, request log or regulator contact is evidenced.
- **Usage context:** section 3.6.
- **Evidence:** documented access/export and erasure flows
  (`privacy-analysis.md:103-123` in S2) **[C]**; data-lifecycle module with
  legal hold **[C]**; in the live catalog 134 of 152 tables carry FORCE RLS
  (`frozen/live/summary.json:10-16`) **[F]**. Security issues form the
  largest class in the recent awcms list (section 5.1). Sampled examples:
  worker or setup roles able to read the password hash column (awcms
  [#1078](https://github.com/ahliweb/awcms/issues/1078),
  [#1082](https://github.com/ahliweb/awcms/issues/1082)); audit table not
  append-only (#1341); a cross-tenant credential overwrite (#1035); a
  server-side-request-forgery guard forwarding credentials on redirect (awcms
  [#1242](https://github.com/ahliweb/awcms/issues/1242)) **[C]**.
- **Jobs:**
  - `J-17` (`P-07`): reconstruct who did what and when for a tenant, from an
    audit record that cannot be edited. Basis: #1341, #145 **[C]**.
  - `J-18` (`P-07`): fulfil an access, export or erasure request for one
    person across all tables that hold their data, and prove completeness.
    Basis: the descriptor-per-table design that paid down a 139-table debt
    before exposing the endpoint (`privacy-analysis.md:103-123`) **[C]**.
    Request volumes and deadlines in practice are **[A]**.
  - `J-19` (`P-07`): enforce retention and legal hold so data is neither
    kept past need nor purged under hold. Basis: data-lifecycle module **[C]**.
    The retention periods actually required are **[O]** (`UA-08`).
- **Not evidenced:** that a dedicated privacy officer exists, how UU PDP
  requests arrive, how many occurred **[O]**.

### 4.9 `P-08` Derived-application / module developer (added)

- **Why added:** the Blueprint mission names ERP, back-office and SaaS
  products and module composition (Blueprint section 3, context only). The
  AWCMS side carries tickets for derived applications and module admission
  (awcms [#187](https://github.com/ahliweb/awcms/issues/187), the module
  descriptor work in #1470 and #1471), and the module registry has grown
  from 24 modules at baseline to 40 directories in the newer checkout **[F]**
  (S1 vs S2). The persona is not on the PRD author's list; it is proposed,
  not renamed into an existing ID.
- **Confidence:** partially evidenced. Module growth and descriptor tickets
  show developers adding modules; no developer was observed.
- **Jobs:**
  - `J-20` (`P-08`): add a domain module that inherits tenant, authorization
    and audit behaviour without re-implementing them or weakening isolation.
    Basis: descriptor tickets #1470, #1471 and tenant-GUC allow-list issue
    **[C]**. The PRD decides whether this persona is in scope for v1 **[O]**.

### 4.10 Evidence against or about the fixed personas

- No evidence contradicts any of `P-01` to `P-07` **[F]**.
- Evidence **narrows** `P-03` and `P-04`: no source distinguishes editors
  from administrators; AWCMS documents list ERP-style roles, not a CMS
  editor role (3.3). The two may be one human in small tenants **[A]**.
- Evidence **widens** `P-02`: the person abused by a forged subscription
  is a different actor from the willing subscriber (`J-06`).
- A fifth context is visible but has no persona: an authenticated end user
  of a native or omnichannel product (requested in awcms
  [#1406](https://github.com/ahliweb/awcms/issues/1406) and omnichannel
  tickets such as #1164). Because it has one ticket and no usage, it is
  recorded as an open question, not a persona **[O]**.

## 5. Recurring pain themes from issues

Basis: 200 most recent awcms issues, plus all 30 awcms-astro issues and all
26 awbms issues, read as titles with selected bodies **[F]**. Titles describe
what maintainers decided to track; they are a biased proxy for user pain. No
issue is a customer-submitted support ticket as far as the sources show
**[F]** (all read issues are maintainer- or agent-authored backlog items;
issue authorship was not analysed per issue).

| Theme | Issues | What it says | Personas |
|---|---|---|---|
| Credential and secret exposure inside the database | awcms #1078, #1082, #1089, #1098 | Worker and setup roles could read password hashes; push auth secret needed encryption at rest; key rotation follow-ups | `P-07`, `P-05` |
| Tenant isolation edge cases | awcms #1035, #1279, #1347, #1046 | Cross-tenant password overwrite; definer reads and views that bypass RLS; search_path shadowing | `P-07`, `P-04` |
| Audit integrity | awcms #1341, #1346, #145 | Audit not append-only for the app role; login not audited | `P-07` |
| Abuse of anonymous write surface | awcms #1478, #1161; frozen `C-07`; awcms-astro #79 | Newsletter opt-out and suppression propagation; limiter has only a per-IP dimension; per-recipient cooldown compensates | `P-02`, `P-06` |
| Rate limiter correctness | awcms #1285, #1385; frozen `rate-limits.json:157-170` | In-process fallback multiplies limits across instances; failure is fail-open by decision; 4 sites parse numbers without validation | `P-05`, `P-06` |
| Performance under public load | awcms #1454, #1321, #1433, #1284, #1287 | Pool-bound saturation at about 300 views/s while CPU idle; search plan is a sequential scan under FORCE RLS; latency metric unwired | `P-01`, `P-05` |
| Outage handling and recovery | awcms #1412, #1409, #1432, #812, #968 | Provider outage observability, half-open breaker race, upload retries burning on local faults, backup assurance and restore drills for sensitive domains | `P-05` |
| Proxy and deployment correctness | awcms #1366 | Host and origin resolve to localhost behind a proxy when allowed domains are unset | `P-05`, `P-01` |
| Consumer build limits and missing surface | awcms-astro #75, #77, #79, #76, #73, #74 | 20,000-post ceiling vs a 25,029-article archive; no redirects; no newsletter UI; backend decisions unabsorbed; unpublished or unstyled articles | `P-06`, `P-03`, `P-02` |
| Contract clarity | awcms #1391, #1385, #1379; awcms-astro #194 | Examples, `429` declaration, two-way tests, fixture-package consumption | `P-06` |
| Auth lockout and enrolment | awcms #1425, #1406 | Users locked out under MFA policy; native client flow missing | `P-04`, `P-06` |

Notes on reading this table:

- The security class dominates the recent list because the list was
  truncated at 200 and the recent window is a security-hardening period
  **[A]**; it is not a measured incident rate.
- Outage rows (#1412, #1409, #1432) are design and test work on failure
  handling, not records of production outages **[F]** (titles and bodies
  read). **No production outage record was found [F].**
- Newsletter "abuse" is evidenced as a designed defence and a claim
  confirmation, not as an observed attack **[F]**.
- AWBMS issues (#11 to #24) are future implementation items; they are not
  evidence of user pain.

## 6. Untested assumptions

Each item is something the PRD will need and no source supports. All are
**[A]** until validated. They are not findings and must not enter the PRD as
facts.

| ID | Assumption | Why the PRD needs it | Related |
|---|---|---|---|
| `UA-01` | Number of tenants and sites AWBMS must serve at v1 and at 12 months | Capacity, isolation test breadth, cost | section 2 |
| `UA-02` | Concurrent authenticated users per tenant (staff) | Admin latency target requires a stated user count (ADR-0161) | `P-03`, `P-04` |
| `UA-03` | Editors per tenant, posts published per day, media uploaded per day | Write throughput, storage growth, queue sizing | `P-03` |
| `UA-04` | Public read rate per site and the achievable cache hit ratio | Origin rps depends on hit ratio, not on the application alone | `P-01` |
| `UA-05` | Number of consumers and integrations that will call AWBMS directly, and how often they build | Contract freeze scope, compatibility test matrix | `P-06` |
| `UA-06` | Newsletter list sizes and send frequency | Email job capacity and abuse limits | `P-02` |
| `UA-07` | A tenant administrator and an editor are separate people with separate permission needs | Role catalogue for v1 | `P-03`, `P-04` |
| `UA-08` | Retention periods and UU PDP request deadlines that actually bind tenant data | Lifecycle defaults, legal hold use | `P-07` |
| `UA-09` | Acceptable downtime and data loss from the tenant's point of view | RTO and RPO values; the PRD marks these as owner decisions | `P-05` |
| `UA-10` | Size and skill of the operator team | Runbook granularity, automation priority | `P-05` |
| `UA-11` | Visitors accept anonymous analytics and prefer search latency over richer results | Beacon retention and search scope | `P-01` |
| `UA-12` | A tenant's largest archive will not exceed the 25,029-article reference | Pagination and build-time limits | `P-06` |
| `UA-13` | Machine credentials are provisioned and rotated by operators, not tenant administrators | Credential lifecycle ownership | `P-06`, `P-05` |
| `UA-14` | Derived-application developers are a v1 user group | Module-admission surface in scope | `P-08` |

## 7. Research gaps and recommended validation plan

These are recommendations. None has been done.

### 7.1 Gaps, in priority order

1. **No production telemetry (scale, traffic, failures).** The most
   consequential gap: capacity assumptions, SLO values and migration
   success criteria cannot be grounded without it (`UA-01`, `UA-02`,
   `UA-04`).
2. **No person has been heard.** Editors, tenant administrators and
   subscribers are described only by interface shape; jobs `J-07` to `J-10`
   have no first-hand confirmation (`UA-03`, `UA-07`).
3. **No incident or support-ticket record from operations.** The issue stream
   is a maintainer backlog; real production incidents, restore times and
   downtime are unknown (`UA-09`, `UA-10`).
4. Consumer inventory is unknown: who else calls the API and how (`UA-05`).
5. Legal obligation for retention and data-subject requests is unconfirmed
   by counsel or a privacy officer (`UA-08`).

### 7.2 Recommended validation plan

| Who or what | Method | Question it answers | Gap |
|---|---|---|---|
| The platform operator(s) of the current AWCMS deployment | Structured interview plus read-only export of deploy, restore and incident history | Real incidents, downtime, restore time, team size | 1, 3 |
| Request logs and CDN statistics of production, aggregated and without personal data | Telemetry export: requests per endpoint per day, p50/p95/p99, cache hit ratio, 4xx/5xx, peak hour | Read rate, hit ratio, abuse and error rates | 1 |
| Database aggregates: tenants, users per tenant, posts, media objects, subscribers, audit rows, table sizes | Read-only counts, no row content | Tenant count and sizes, volume growth | 1 |
| Two to five tenant editors and administrators | Contextual observation of one publish and one user-management task each, then a short interview | Real jobs, error recovery, role split | 2 |
| Developers of each AWCMS-Astro site | Interview plus review of their build logs | Consumers, build frequency, credential handling, the 20,000-post ceiling in practice | 4 |
| Newsletter data, aggregated | Counts of subscribe, confirm, unsubscribe and bounce per tenant | List sizes, confirmation rate, abuse | 2 |
| A privacy or legal reviewer for the tenants' sector | Review of retention rules and data-subject request handling | Binding retention periods and deadlines | 5 |
| Support channel of the operator organisation, if any | Tag and count requests for 30 days | Real support tickets, replacing the issue proxy | 3 |

### 7.3 Telemetry to start collecting now (if not already present)

- Per-endpoint request count, latency histogram and status by tenant and
  by client class (anonymous, session, machine credential).
- Cache hit ratio at the edge and origin requests per second.
- Rate-limiter refusals per bucket, and limiter backend (shared or process).
- Database pool queue depth and rejections.
- Job and queue age (email, sync, purge).
- Backup age, restore-drill duration, migration duration per release.
- Aggregate counts of tenants, users, posts, media objects and subscribers.

## 8. Source index

- Frozen corpus: `contracts/legacy/awcms/frozen/static/astro-consumer.json`,
  `routes.json`, `modules.json`, `rate-limits.json`, `behaviour-fixtures.json`,
  `contracts/legacy/awcms/frozen/live/summary.json`,
  `contracts/legacy/awcms/authorization/vectors.json`, and the
  [VG-01 PASS record](../architecture/evidence/VG-01-PASS-2026-11-23.md).
- AWCMS (public repository `ahliweb/awcms`), checkout newer than the
  frozen baseline (see section 1.1): `docs/PROJECT_STATE.md`,
  `docs/awcms/privacy-analysis.md`, `docs/awcms/data-lifecycle.md`,
  `docs/awcms/visitor-analytics.md`, `docs/awcms/resilience-dr-verification.md`,
  `docs/awcms/production-preflight-runbook.md`,
  `docs/awcms/17_default_seed_rbac_abac.md`, `docs/adr/0114-*`, `docs/adr/0161-*`.
- AWCMS-Astro (public repository `ahliweb/awcms-astro`):
  `tests/kontrak-awcms.test.mjs`, `src/lib/awcms/client.ts`, `tenant.ts`.
- GitHub issues in `ahliweb/awcms`, `ahliweb/awcms-astro`, `ahliweb/awbms`
  as cited inline, read 2026-11-23.
- Out of this document on purpose: the Blueprint's decisions, which are not
  restated; see [Master Blueprint](../architecture/AWBMS-MASTER-BLUEPRINT.md)
  sections 3, 6, 8, 11, 18, 19 and 31 for context.
