# ADR-0008 — Close OD-03, OD-05 and OD-06, resolve the assumption register, and sign off Stage 1

| Field | Value |
|---|---|
| Status | Accepted |
| Date | 2026-11-23 |
| Deciders | Repository owner. Recorded approval, 2026-11-23: *"Ya aku setuju semua sesuai dg riset sebelumnya"* ("Yes, I agree to all of it, in line with the earlier research"), given in reply to the list of remaining Stage-1 conditions (`OD-03`, `OD-05`, `OD-06`, the assumption register, and approval of the validation). The dispositions below were drafted by the orchestrating agent from the cited evidence and accepted by that statement |
| Supersedes | none |
| Superseded by | none |
| Related registers | `OD-03`, `OD-05`, `OD-06` (closed); `AS-01`–`AS-08` (resolved); Appendix F (approval); `AD-04`, `AD-05`, `AD-10`, `AD-17`, `AD-26`, `AD-27`, `AD-30`; `VG-05`, `VG-09`, `VG-12` |

## Context

`[F]` After [ADR-0006](0006-accept-blueprint-dispositions-od-01-02-04-09.md)
and the `VG-01` PASS ([record](../architecture/evidence/VG-01-PASS-2026-11-23.md)),
four of the six conditions in [validation §49.1](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#491-entry-conditions)
were met. Open were condition 2 (Blueprint-blocking decisions `OD-03`,
`OD-05`, `OD-06`) and condition 3 (assumptions `AS-01`–`AS-08`, one
confirmed), plus the approval record in Appendix F.

The evidence available for each:

- `OD-03`: the frozen inventories `frozen/static/credential-formats.json`,
  `astro-consumer.json` and `routes.json` (`VG-01`).
- `OD-05`: Master Blueprint §12 (multi-tenant PostgreSQL, tenant-bound
  transactions, FORCE RLS) and the internal cross-portfolio traffic-readiness
  research (decision-log entry DL-049), whose scale ladder places per-tenant
  sharding at stage S3 and dedicated hosts for critical applications at S1.
- `OD-06`: the same research (owner decision DL-049: staged S1 → S2 scale with
  measurable NFRs; its proposed testable NFRs), and Master Blueprint §11
  (baseline-first no-regression; absolute product SLOs confirmed at the Stage 2
  PRD).
- Assumptions: the frozen corpus, the research's measured AWCMS load test, and
  the owner's confirmation of operating facts.

What is **not** known: real AWBMS workload, the number of tenants per
deployment at launch, and provider behaviour under AWBMS adapters. None of the
values below is a measured AWBMS result.

## Decision

### `OD-03` — legacy formats AWBMS accepts

AWBMS carries the smallest compatibility surface that keeps users and the
AWCMS-Astro consumer working through cutover:

| Legacy item (from `credential-formats.json`) | Disposition |
|---|---|
| Passwords: Argon2id PHC strings (v19, m = 65536 KiB, t = 2, p = 1) | **Accepted as stored.** Verified by AWBMS; rehashed to AWBMS parameters on next successful login if they differ. No plaintext or other hash format is accepted |
| Human sessions (`sha256:` verifier of an opaque token) | **Not carried.** Sessions of a capability end at its cutover; users re-authenticate. Avoids importing a second session format |
| Machine credentials (`awcmsm_…`, stored `mc-sha256:`) | **Accepted during coexistence** through the compatibility adapter, for the consumer classes that use them (AWCMS-Astro build-time reads). AWBMS issues its own format; legacy credentials are rotated and stop being accepted at a date set in each wave's migration plan |
| Newsletter confirmation/unsubscribe tokens | **Accepted for their remaining validity**, so links already e-mailed keep working |
| Other token namespaces (`pt-`, `dg-`, `ho-`, `bff-sha256`) | **Not carried**; they expire with the AWCMS capability |
| Encrypted secrets (MFA TOTP, SSO client secrets; AES-256-GCM `v1:iv:tag:ct`) | **Migrated, not accepted at runtime**: decrypted and re-encrypted under AWBMS key management during deterministic migration |
| Plaintext candidates (OIDC `nonce`/`code_verifier`; push `auth_secret`) | OIDC values are ephemeral and **not migrated**; push `auth_secret` is **migrated encrypted** |
| Legacy API contracts | The **AWCMS-Astro consumed surface** (`astro-consumer.json`, five execution classes, Master Blueprint §18) is the compatibility contract and the scope of the `VG-05` parity corpus. Other AWCMS endpoints carry no compatibility guarantee |

### `OD-05` — deployments and tenant separation

v1 runs as **one shared multi-tenant deployment per environment** (staging and
production) per product installation. Tenants are separated **by tenant ID
within one PostgreSQL database**, enforced by FORCE RLS and tenant-bound
transactions (`AD-10`, Blueprint §12). A **dedicated deployment** for a tenant
is an exception, recorded in its own ADR, for regulated or `critical`-class
tenants (for example health data) or contractual isolation. Per-tenant
sharding is the S3 scale path and is out of scope for v1.

### `OD-06` — SLO targets

Initial SLO policy, staged S1 → S2, to be confirmed or tightened per capability
in the Stage 2 PRD and fixed at gate 9:

| Target | Initial value | Measured by |
|---|---|---|
| Availability, production | 99.9 % monthly per deployment; 99.95 % only after replicas and HA exist | uptime monitoring |
| Authenticated API latency | p95 read ≤ 500 ms, p95 write ≤ 900 ms at the concurrent-user count the PRD sets per tenant | load test of the Blueprint §11 scenarios |
| Capacity | stage S1 first (edge-cached public traffic, pooled connections); S2 (horizontal app tier, read replicas) only after S1 is proven | distributed load test; no capacity claim without one |
| Regression | Blueprint §11 thresholds against the last approved baseline | `VG-09` |
| Recovery | RTO/RPO set per capability in the Stage 2 PRD; until then the [pre-production evidence specification](../architecture/AWBMS-PRE-PRODUCTION-EVIDENCE.md) placeholders apply | `VG-12` rehearsal |

Targets expressed only as absolute requests per second, without cache hit ratio
and p95, are not used.

### Assumption register

| ID | Resolution | Basis |
|---|---|---|
| `AS-01` | **Confirmed** | frozen `awcms-package.json` and CI: TypeScript on Bun |
| `AS-02` | **Confirmed** by owner | the owner operates Docker/Coolify/Traefik/Cloudflare across the portfolio |
| `AS-03` | **Confirmed** | `VG-01` PASS used exactly this access |
| `AS-04` | **Accepted, dependent decision revisited and retained**; to be measured by `VG-09` | the only AWCMS HTTP load test (internal research DL-049) showed 620 rps at p95 405 ms with application CPU at 1.4 %, i.e. not framework-bound; `AD-04` rationale holds |
| `AS-05` | **Confirmed** | `OD-05` above: the tenant is the isolation unit |
| `AS-06` | **Confirmed** | `OD-06` above: 99.9 % on a single authoritative primary; HA only later |
| `AS-07` | **Accepted, dependent decision revisited and retained**; verified per adapter when each adapter is built (e.g. issue #17) | if a provider lacks a capability only that adapter changes; the port abstraction (`AD-17`) survives by design |
| `AS-08` | **Confirmed** by owner | one operator (the owner) controls AWCMS and AWBMS during coexistence |

### Stage 1 sign-off

With the above, all six §49.1 conditions are met. The owner's statement is
recorded as approval of the architecture validation and the Master Blueprint,
and **Stage 1 is signed off**. The approval covers entering **Stage 2 (PRD)**.
It is **not** approval of implementation (gate 9) or of migration engineering;
issues #11–#20 and #22–#24 stay blocked until gate 9.

## Alternatives considered

| Alternative | Why not chosen |
|---|---|
| Carry every AWCMS credential and session format | Maximises compatibility surface and parity scope for little user benefit; re-login at cutover is a bounded cost |
| Accept no legacy password hashes (force reset) | Every user would need a reset e-mail at cutover; Argon2id hashes are verifiable as-is, so this adds risk without security gain |
| Deployment per tenant by default | Multiplies operational cost before any tenant requires it; kept as an exception path |
| Leave SLOs to Stage 2 entirely | Leaves `OD-06` and `AS-06` open and Stage 1 unsignable; the research already gives testable initial values that the PRD can tighten |
| Higher availability (99.95 %+) for v1 | Requires replicas/HA that `AD-05` and the S1 stage do not include; would reopen `AS-06` |

## Consequences

### Accepted costs

- Users re-authenticate when each capability cuts over.
- Legacy machine credentials must be rotated within each wave.
- SLO values are initial and may change in the PRD; until measured they are
  targets, not facts.

### Benefits

- All Blueprint-blocking decisions are closed; Stage 2 can start.
- The `VG-05` parity scope is bounded to the AWCMS-Astro consumed surface.

### Follow-on work

- Stage 2 PRD: per-capability SLO confirmation, RTO/RPO, concurrent-user
  counts, service tiers (owner).
- Gate 9 berita acara before any issue in #11–#24 starts.
- Cut a release recording the Stage 1 milestone (`scripts/version-bump.sh`).

## Verification

- Appendix D shows no Blueprint-blocking open decision; Appendix B shows each
  assumption resolved; Appendix F names the approver and date;
  `scripts/check-docs.sh` passes.
- The Stage 1 gate register states Stage 1 signed off.

## Revisit when

- The Stage 2 PRD sets a value that conflicts with the initial SLO table.
- A tenant requires dedicated deployment (record the exception ADR).
- A cutover wave finds a legacy format not listed above.
- `VG-09` falsifies `AS-04`, or an adapter spike falsifies `AS-07`.
