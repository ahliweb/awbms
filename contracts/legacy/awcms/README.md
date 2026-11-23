# AWCMS / AWCMS-Astro Legacy Evidence Provenance

**Gate:** `VG-01`  
**Status:** **PARTIAL — provenance initialized; frozen artifact package incomplete**

This directory is the only permitted repository location for imported AWCMS/AWCMS-Astro compatibility evidence. Imported evidence is a versioned fixture, **not** a source or runtime dependency.

## Pinned source baselines

| Source | Commit | Observed state | Role |
|---|---|---|---|
| `ahliweb/awcms` | `11f2e95a47b1328a820f976d60f978c38a067903` | release `v10.1.0` | behavior, schema, auth/RLS, API/event, gates, migration reference |
| `ahliweb/awcms-astro` | `7b753be619244541b817d5d8e7d3b72cfe88d4f9` | package `v0.4.0` | concrete public consumer and cross-repo contract reference |

## Verified static observations at these baselines

From the generated AWCMS repository inventory:

- 24 registered modules;
- 148 migrations;
- 152 `awcms_*` tables;
- 134 tables with `FORCE` RLS;
- 18 RLS-free global tables by design;
- 507 test files;
- 388 route files;
- 240 ADRs.

The AWCMS module registry independently lists the same 24 base modules.

The AWCMS consumer-contract source records thirteen paths consumed by AWCMS-Astro and two committed future paths. The consumed set includes build-time machine-authenticated reads, anonymous cross-origin reads, anonymous telemetry writes, and anonymous newsletter writes; these classes have different security invariants.

## Required provenance fields per imported artifact

Every frozen artifact MUST record:

- source repository;
- source commit SHA;
- import/generation date;
- SHA-256 digest;
- source path or generation command;
- contract/schema version where available;
- semantic purpose;
- AWBMS check/test consuming it;
- disposition when superseded.

## Directory shape

```text
contracts/legacy/awcms/
├── README.md
├── manifest.json            # VG-01 artifact register
├── fixture-usage.json       # every frozen file → the check that consumes it; pendingArtifacts
├── authorization/
│   └── vectors.json         # authorization vectors: prose plus structured, cited fields
└── frozen/
    ├── provenance.json      # pinned AWCMS / AWCMS-Astro SHAs, toolchain
    ├── static/              # deterministic source inventory + pinned source copies
    │   ├── modules.json, migrations.json, astro-consumer.json
    │   ├── gates.json             # generated; frozen by the next vg01-freeze run
    │   ├── routes.json            # generated; frozen by the next vg01-freeze run
    │   ├── behaviour-fixtures.json  # generated; frozen by the next vg01-freeze run
    │   ├── rate-limits.json       # generated; frozen by the next vg01-freeze run
    │   ├── credential-formats.json  # generated; frozen by the next vg01-freeze run
    │   └── source/                # pinned copies (OpenAPI, AsyncAPI, CI, package.json, ...)
    │       ├── tests/awcms/       # cited AWCMS test files (generated; frozen with the inventories above)
    │       └── awcms-impl/        # implementation files that rate-limits.json and credential-formats.json cite, at their repository paths (generated; next freeze run)
    └── live/                # controlled live PostgreSQL RLS / role introspection
```

Frozen files are published only through the `vg01-freeze` workflow
(generation with a read-only token, publication by pull request), and
`tools/vg01_verify_frozen.sh` fails CI on any orphan fixture or stale source SHA.

### Pending artifacts

`tools/vg01_inventory.rs` generates `rate-limits.json`, `credential-formats.json`
and the implementation files they cite (`source/awcms-impl/`), and
`tools/vg01_check_corpus.rs` validates them (`check:rate-limits`,
`check:credential-formats`), but they are not committed until the `vg01-freeze`
workflow runs. Until then `fixture-usage.json` lists them in `pendingArtifacts`:
an absent pending artifact is a note, a present one must validate and must be
mapped, and the freeze workflow's own verification therefore already enforces
them. **The freeze pull request empties `pendingArtifacts`** (after which
absence is a failure). `verification.yml` also checks, on every pull request,
that the checker accepts the freshly generated output
(`tools/vg01_check_generated.sh`). The gate, route and behaviour inventories and
their test copies were frozen by an earlier run and are no longer pending.

### Rate limits and credential formats

Both artifacts are derived from the pinned AWCMS implementation, never from
prose, and every fact in them cites `path`, `line` and an `anchor` that is a
substring of that line; the cited files are copied to `source/awcms-impl/` so
the checker re-resolves each citation inside the corpus.

- `rate-limits.json`: every call of `checkSharedRateLimit` / `checkAuthRateLimit`
  under `src/` (38 call lines, 40 call sites once the newsletter preflight
  wrapper is expanded at its three callers), with route and handler, bucket key
  template, key dimensions and where an attacker-chosen value could enter,
  threshold and window setting names with their defaults, the parser each one
  goes through (and what an invalid value does), the limiter module's own
  behaviour (fixed window, Redis `INCR`/`PEXPIRE` or in-process `Map`, fail-open
  on Redis failure, `X-Forwarded-For` trust), and the non-limiter controls that
  bound abuse: per-account password lockout, per-factor MFA lockout, the
  newsletter per-recipient cooldown and the Turnstile call sites. It also
  states what the source cannot establish (deployment configuration, edge
  limits).
- `credential-formats.json`: password hashing (library, algorithm, parameters,
  where verified and written), the generator and at-rest hash format of each
  bearer token kind with the database `CHECK` that pins it, the encrypted
  secrets, and an inventory of every credential-like column of the migrations
  classified as hash / encrypted / env-var-name / plaintext-candidate. Only
  formats and parameters are recorded; no hash, key, secret or fixture
  credential is copied, and the checker fails on a PHC string that carries a
  salt or digest or on any digest-like hex run.

**`C-07` verdict recorded by the generator: confirmed.** `POST
/api/v1/newsletter/subscribe` is limited per client IP only (`subscribe.ts:139`,
5 requests per 300 s by default), and a separate recipient-oriented cooldown is
enforced inside the upsert (`subscriber-directory.ts:153-179`): a confirmation
e-mail is re-issued to an existing `(tenant_id, email_normalized)` row only if
`confirmation_sent_at` is `NULL` or older than `NEWSLETTER_CONFIRMATION_COOLDOWN_SEC`
(default 900 s), and a refused re-send gets the same neutral response as every
other branch. The checker recomputes the verdict from the recorded limiter
dimensions and the cited SQL.

## Status and what is NOT proven

`VG-01` is **PASS** (2026-11-23,
[record](../../../docs/architecture/evidence/VG-01-PASS-2026-11-23.md)): all ten inventory items of
validation §2.2 are frozen with provenance and digests and are consumed by
checks. That makes this corpus the requirements baseline; it does **not** prove
parity. Still missing, and owned by `VG-05`:

- request/response body fixtures (the pinned source has none to freeze);
- a harness that executes the authorization vectors against an AWBMS runtime.

Static repository inventory is not a substitute for live database evidence where the gate requires live state.

## Independence rule

Prohibited:

- git submodule to AWCMS;
- Cargo path/git dependency on AWCMS;
- production filesystem coupling;
- runtime import of AWCMS source;
- CI that requires an unfrozen mutable AWCMS checkout to pass normal AWBMS tests.

Allowed:

- explicit inventory-generation tooling that reads a pinned external source;
- reviewed frozen fixtures committed here;
- controlled parity harnesses using separately deployed/checked-out AWCMS instances.

Once imported, ordinary AWBMS compatibility tests should consume the frozen fixtures so historical parity is reproducible.
