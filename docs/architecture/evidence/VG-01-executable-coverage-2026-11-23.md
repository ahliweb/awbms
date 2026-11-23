# VG-01 Executable Coverage — 2026-11-23

**Gate:** `VG-01`  
**Status:** PARTIAL (unchanged by this note; the gate register is edited only by the orchestrator)  
**Issue:** `#3` (also `#7`, P0 item 4)

This note audits the acceptance criteria of `#3` against `main` at `59cc2c3`
plus the checks added in the same change, and states whether `VG-01` can move to
PASS. It adds no frozen artifact and regenerates nothing.

## What was added

| Artifact | Purpose |
|---|---|
| `contracts/legacy/awcms/fixture-usage.json` | Machine-readable map from every file under `frozen/` and `authorization/vectors.json` to the consumers that read it, with a `strength` per consumer (`content`, `structural`, `integrity`). |
| `tools/vg01_check_corpus.rs` | std-only Rust checker (`rustc --edition 2024 -D warnings`, no Cargo dependency) that reads the fixtures and fails on contradiction, orphan fixtures, dangling consumers and stale source SHAs. |
| `tools/vg01_verify_frozen.sh` | Now compiles and runs the checker last. CI already runs this script in the step "Verify frozen VG-01 evidence when present", so no workflow edit is needed. |

## Checks executed against the frozen corpus

| Check | What it reads | What would make it fail |
|---|---|---|
| `stale-source-sha` | `frozen/provenance.json`, both workflows, `tools/vg01_inventory.rs`, every fixture that records a source SHA | any SHA that disagrees with `provenance.json`; an AWCMS or AWCMS-Astro checkout not pinned to a SHA |
| `modules-registry` | frozen registry source (`baseModules` array and imports), `modules.json`, generated inventory | module count or names differ between registry source, `modules.json` and the inventory (one alias, `workflow` = `workflow-approval`, is explicit) |
| `migrations` | `migrations.json`, inventory migration table, live migration ledger | non-contiguous numbering, duplicate or malformed SHA-256, list or order differs from inventory or ledger |
| `tables-rls` | inventory table section, source and live TSV, `tables-rls.csv`, `policies.csv` | any table or RLS/FORCE state differs between source, live and catalog; RLS table without a policy; policy count mismatch; counts other than 152/134/134 |
| `static-vs-live` | the three drift files, live table names, summary | any non-empty drift file; the 152 vs 151 scope difference not being exactly the migration ledger table |
| `live-roles` | `roles.csv`, table/routine grants, table owners | SUPERUSER/BYPASSRLS/CREATEROLE/CREATEDB role; more than one table owner; runtime role owning tables; grant to unknown role or table |
| `live-summary` | `summary.json`, hand-maintained `manifest.json` summaries | any summary figure that differs from the figure derived from the underlying files |
| `route-test-counts` | inventory route and test tables, frozen OpenAPI | surface or directory sums differ from the summary rows; OpenAPI `/api/v1` path count differs from the `/api/v1/**` route-file count (308) |
| `astro-consumer` | `astro-consumer.json`, frozen consumer-contract source | consumed/committed path sets differ from the source; class distribution other than 7 / 2 / 1 / 3 |
| `openapi-security` | `astro-consumer.json`, frozen OpenAPI | an anonymous browser class whose operations are not all `security: []`; a machine-authenticated or committed path that is |
| `vectors` | `authorization/vectors.json`, live fixtures | duplicate or incomplete vector; `DB-RLS-001` count or migration count differing from live/inventory fixtures |
| `manifest-digests` | both `manifest.sha256`, static `manifest.json`, files on disk | a file on disk without a digest, a digest without a file, or `manifest.json` disagreeing with `manifest.sha256` |
| `source-headers` | OpenAPI/AsyncAPI headers, package versions, frozen CI, consumer test | OpenAPI not 3.1.0, AsyncAPI not 3.0.0, package versions not 10.1.0 / 0.4.0, lost `bun run check`, lost tenant-binding tests |
| `usage-map` | `fixture-usage.json`, tree under `frozen/` | file with no consumer; mapped file that does not exist; unknown consumer id; consumer whose file or marker does not exist; tool check that did not run |

Result with the current corpus: 31 mapped artifacts, 17 consumers, no
integrity-only artifact (every artifact has at least one structural or content
consumer).

## Authorization vectors: do executable vectors exist?

`authorization/vectors.json` exists and holds 14 vectors. Each has `id`,
`source`, `class`, `input` and `expected`, and the last two are **prose
sentences**, not machine-executable inputs. There is no parity harness in
AWBMS that runs them. 12 of the 14 cite test files in the AWCMS-Astro (5) and
AWCMS (7) repositories that are not part of the frozen corpus (the frozen
`awcms-astro-contract.test.mjs` is a different, AWCMS-side contract test); the
other 2 cite controlled database introspection. The checker
validates their structure and binds `DB-RLS-001` to the live fixtures; it does
not execute them.

## Acceptance criteria of `#3`

| # | Criterion | Verdict | Evidence |
|---|---|---|---|
| 1 | Inventory generation is re-runnable from pinned source inputs | MET | `tools/vg01_inventory.rs` verifies both checkout HEADs and fails closed; `verification.yml` builds it, runs it twice and `diff -ru`s the outputs. Not re-executed in this change (needs the pinned checkouts). |
| 2 | Every imported artifact has SHA-256 provenance | MET | Both `manifest.sha256` files verified by `sha256sum -c`; `check:manifest-digests` now also proves the digest list equals the files on disk. `authorization/vectors.json` is authored, not imported, and has no digest (finding below). |
| 3 | AWCMS and AWCMS-Astro SHAs are both recorded | MET | `provenance.json`, `static/manifest.json`, `manifest.json`, `vectors.json`, fixtures; `check:stale-source-sha` ties all of them to the workflow pins. |
| 4 | 24-module claim derived from registry source, not prose | MET | `check:modules-registry` parses the frozen `baseModules` array and imports (24) and compares to `modules.json` and the generated inventory. |
| 5 | Migration inventory derived from files/ledger semantics, not prose | MET, with a limit | List, numbering and per-file SHA-256 shape are cross-checked against the inventory and the live ledger (148). Per-file SHA-256 against the real SQL cannot be recomputed without the AWCMS checkout; that remains the generator's CI job. Ledger checksums (`sha256:...`) differ from the file SHA-256 values and the algorithm is not documented in the corpus, so they are format-checked only. |
| 6 | RLS/role evidence distinguishes source declaration from live introspection | MET | Separate `source-*` and `live-*` fixtures, empty drift files, `check:tables-rls`, `check:static-vs-live`, `check:live-roles`. The 152 vs 151 difference is asserted to be exactly the ledger table. |
| 7 | Consumer-contract fixture captures execution/security class distinctions | MET | `astro-consumer.json` has four consumed classes (7/2/1/3) plus the committed class; `check:openapi-security` shows the anonymous classes correspond to `security: []` in the frozen OpenAPI and the machine-authenticated class does not. |
| 8 | Gate inventory identifies enforced vs advisory vs absent checks | NOT MET | Only raw `awcms-ci.yml` and `awcms-package.json` are frozen. `manifest.json` records `source-frozen-pending-structured-inventory`. Producing it needs a read of the pinned AWCMS source and goes through the `vg01-freeze` publication workflow. |
| 9 | Authorization vectors are executable/frozen inputs for future parity tests | NOT MET | Frozen, but prose-only and with no harness (see section above). Machine-readable inputs and a Rust consumer need design and the cited test sources. |
| 10 | `VG-01` changed to PASS only after all required artifacts/checks exist | NOT MET | Criteria 8 and 9 are open, plus the two artifacts below. |

Required artifacts from the gate's 14-item set that are still absent:

- **Route inventory with owner/auth classification (item 7).** Only counts exist
  (308 `/api/v1/**`, 50 `/admin/**`, 30 public). The OpenAPI gives per-path
  `security` data for `/api/v1` but no owning module, and `manifest.json` says
  `pending-structured-inventory`.
- **Representative compatibility fixtures (item 13).** `manifest.json` says
  `pending`; nothing in the corpus is a request/response or behavior fixture.

Issue `#3` test/evidence list: generator determinism (CI diff), SHA-256
verification, stale-source-SHA detection, orphan fixture detection and
inventory count consistency are now covered. "Manifest schema/required fields"
is only partly covered (digest manifests and the usage map are validated; the
hand-written `manifest.json` has no schema beyond the summary cross-check).

## Can VG-01 move to PASS?

No. The pass rule requires every required artifact to exist and be consumed by a
deterministic check or parity test. This change closes the "artifact-to-test
usage mapping" and static/live consistency items, but the structured gate
inventory, the route ownership/auth inventory, representative behavior
fixtures and executable authorization vectors do not exist. None of them can be
produced without reading AWCMS, so they must be added through the `vg01-freeze`
workflow or a separate reviewed import, not here.

## Verification performed

- `bash tools/vg01_verify_frozen.sh contracts/legacy/awcms` passes (14 checks
  plus the existing `sha256sum -c` and grep assertions).
- Negative runs: an unmapped dummy file under `frozen/static/` fails
  `check:usage-map` (and `check:manifest-digests`); a mapped consumer id that
  does not exist fails; a mapped marker that is not in the consumer file fails;
  a workflow checkout pinned to a different SHA fails `check:stale-source-sha`.
  Each was reverted afterwards.
- `rustc --edition 2024 -D warnings --test tools/vg01_check_corpus.rs` unit
  tests pass (JSON, CSV and SHA-shape helpers).
- `scripts/check-docs.sh` passes.

## Findings outside this change

- `contracts/legacy/awcms/README.md` still lists as "NOT yet proven" several
  items that now exist (generator, per-file migration SHA-256, live
  introspection, OpenAPI/AsyncAPI copies, usage checks), and its planned
  directory shape does not match `frozen/`.
- `contracts/legacy/awcms/manifest.json` has `status: partial-frozen-evidence`
  and says the artifact usage check was implemented; that is only true from
  this change onward.
- The checker's unit tests are not run in CI; adding them needs a
  `verification.yml` edit.
- Regenerating the frozen corpus through `vg01-freeze` will fail
  `check:usage-map` if the set of frozen files changes until
  `fixture-usage.json` is updated. That is intended.

## Follow-up: generator and checker extension for the five open criteria

A later change extends `tools/vg01_inventory.rs` and `tools/vg01_check_corpus.rs`
for the five criteria this note marks NOT MET (and the migration limit under
criterion 5). It commits no generated corpus: the new frozen files come from the
next `vg01-freeze` run, and the baseline is **not** refreshed (pins stay
`11f2e95a47b1328a820f976d60f978c38a067903` and
`7b753be619244541b817d5d8e7d3b72cfe88d4f9`; the baseline-delta note records why).
Verdict columns describe the state after that freeze run; nothing here is PASS.

| Issue `#3` item | Added | After the freeze run |
|---|---|---|
| Gate inventory enforced / advisory / absent | `gates.json`: 82 entries (71 enforced, 4 advisory, 7 absent) from AWCMS `.github/workflows/*.yml`, `package.json` scripts, `scripts/` and docs; every entry cites its source. Validated by `check:gates` against the frozen `package.json` and `ci.yml`. | MET, with limits below |
| Route inventory with owner and auth class | `routes.json`: 388 route files (308 / 50 / 30, equal to the frozen inventory), 483 handlers, owner from module `api.routes`, `PLATFORM_ROUTES`, navigation and `CORE_NAV_ENTRIES`, auth class from code markers. Validated by `check:routes`, including a recomputation of OpenAPI agreement from the frozen OpenAPI. | MET, with limits below |
| Executable authorization vectors | `authorization/vectors.json` keeps ids and prose and gains a `structured` block per vector (actor, credential class, tenant relation, operations, decision, status, test kind, cited test and name). Validated immediately by `check:vectors`; citations into the new frozen test copies resolve after the freeze. | Machine-readable and cited: MET. Executed by a harness: NOT MET |
| Representative request/response fixtures | `behaviour-fixtures.json`: 399 OpenAPI operation contracts (security, request schema, response statuses), 8 test-asserted exchanges (method, path, literal status, error code, cited test line), and an explicit list of classes with no source. | NOT MET for bodies; operation-level contracts only |
| Migration SHA-256 against the SQL | `tools/vg01_verify_migrations.sh`, run in the Rust job of `verification.yml` against the pinned checkout (both the frozen and the freshly generated `migrations.json`). | MET once merged (does not need the freeze) |

### How each classification is derived

- **Gates.** Statuses come from files, never prose. `enforced`: reached, directly or through `bun run` chains (the `check` script expands to 63 distinct scripts), by a workflow job that triggers on `pull_request` or a branch push. `advisory`: a gate-like script that only release or manual workflows run, or that no workflow runs. `absent`: a doc names a check (`bun run X` or a script file) that does not exist at the pinned commit. Not derivable from source: whether branch protection makes a job *required* (a repository setting), and invariants that no document names as a command.
- **Routes.** Session-free body endpoints come from AWCMS's own `SESSION_FREE_BODY_ENDPOINTS` registry; `machine-credential` from `verifySyncHeaders` and `defineClientCredentialTenantRoute`; `session` from `authorizeInTransaction`, the tenant-route factories, or a hand-written 401 `AUTH_REQUIRED` path; `admin` from the `/admin` middleware prefix. 392 of 399 OpenAPI-documented handlers agree with the frozen OpenAPI `security`; the 5 that differ carry the registry's reason (enrolment token, locale cookie, retired 410 endpoints); 2 stay `indeterminate` (optional-session reads: `GET /api/v1/comments`, `GET /api/v1/auth/sso/{providerKey}/start`); 4 `OPTIONS` preflights are not in the OpenAPI.
- **Fixtures.** The bundled OpenAPI holds 5 scalar property `example:` values and 0 `examples:` blocks, and the pinned tree holds no recorded traffic, so those classes are listed as not available rather than invented. The AWCMS-Astro contract test (16 `fetch` mocks) is available by reference as an already-frozen file.

### Verification performed

- Generator run twice against the pinned checkouts: `diff -ru` empty (deterministic); the pre-existing static files are byte-identical to the committed corpus.
- Checker run on the real generated tree assembled as a post-freeze corpus (`tools/vg01_check_generated.sh`): 17 checks pass, `pendingArtifacts` emptied; also passes with the artifacts present and still listed pending (the freeze job's state).
- Corrupted copies are detected by the intended validator: a route reclassified, an unknown owner, a deleted handler, an enforced gate marked advisory, a command differing from the frozen `package.json`, an unasserted status, a deleted test copy, a missing cited test name, an invalid decision, an operation path absent from the OpenAPI, a missing artifact with an empty pending list, an unmapped extra file.
- `bash tools/vg01_verify_frozen.sh contracts/legacy/awcms` still passes against the committed corpus (new artifacts reported as pending notes).
- `tools/vg01_verify_migrations.sh` passes for 148 files, and fails on a changed digest and on an extra migration.

### Findings outside this change

- The AWCMS `ci.yml` comment says `bun run check` covers "all 52 gates"; the script chain expands to 63 distinct scripts.
- Docs name checks that do not exist: `config:docs:check` ("part of `bun run check`"), `database:capacity:check`, `check:lockfile`, `extension:check`, `i18n:parity:check`, `i18n:pot:check`, `performance:query-plan:check`.
- Once the freeze lands, `bash tools/vg01_verify_frozen.sh` should list the three new JSON files in its `required` array; that edit belongs with the freeze pull request.

## Follow-up: rate-limit and credential-format inventories

A further change extends `tools/vg01_inventory.rs` and `tools/vg01_check_corpus.rs`
for the two items of the validation's required inventory that no frozen
artifact covers: "rate-limit and abuse-control dimensions per endpoint class"
(superseding claim `C-07`) and "password and machine-credential storage
formats in use". It commits no generated corpus; the files come from the next
`vg01-freeze` run, `fixture-usage.json` lists them in `pendingArtifacts`, the
baseline is not refreshed (pins unchanged) and no gate changes state.

| Artifact (frozen by the next run) | Content | Validator |
|---|---|---|
| `frozen/static/rate-limits.json` | 38 limiter call lines, 40 call sites (the newsletter preflight wrapper expanded at its three callers), 53 buckets in 39 endpoint classes; bucket key template and dimensions; threshold and window setting names, defaults and parsers; limiter-module behaviour; password lockout, MFA factor lockout, newsletter recipient cooldown and Turnstile call sites; the `C-07` finding | `check:rate-limits` |
| `frozen/static/credential-formats.json` | password hashing; 12 bearer-token kinds with generator, hash format and database `CHECK`; 2 encrypted-secret kinds; 26 credential-like migration columns classified by storage | `check:credential-formats` |
| `frozen/static/source/awcms-impl/` | 84 implementation files (33 route handlers, 35 limiter, configuration, credential and module files, 16 migrations) at their repository paths, so every cited line resolves inside the corpus | both |

Every fact cites `path`, `line` and an `anchor` that must be a substring of that
line; the generator refuses to emit a fact otherwise, and the checker
re-resolves all of them in the frozen copies.

### `C-07`: what the code does

Verdict: **confirmed** at the pinned SHA. The claim says per-IP limits alone
could not stop repeated mail to a victim address and recipient-oriented
cooldown semantics were required.

- `POST /api/v1/newsletter/subscribe` is limited per client IP only: key
  `newsletter:subscribe:${clientIp}` (`src/pages/api/v1/newsletter/subscribe.ts:139`),
  `NEWSLETTER_RATE_LIMIT_MAX` default 5 per `NEWSLETTER_RATE_LIMIT_WINDOW_SEC`
  default 300. No limiter site in the tree has a recipient dimension.
- A separate recipient-oriented cooldown is enforced in the upsert itself
  (`src/modules/newsletter/application/subscriber-directory.ts:153-179`): on
  `ON CONFLICT (tenant_id, email_normalized) DO UPDATE` a confirmation is
  re-issued only if `confirmation_sent_at IS NULL` or older than
  `make_interval(secs => confirmationCooldownSeconds)`. The value is
  `NEWSLETTER_CONFIRMATION_COOLDOWN_SEC`, default 900, parsed by
  `parsePositiveIntSetting` (`subscribe.ts:89`, passed at `subscribe.ts:201`),
  and the conflict key is the unique index in `sql/139` line 101.
- A refused re-send returns the same neutral body as every other branch
  (`subscribe.ts:96`), so the cooldown is not an enumeration oracle.
- The cooldown covers the confirmation e-mail of `subscribe` only. The
  `confirm` and `unsubscribe` endpoints are per-IP only. `POST
  /api/v1/auth/password/forgot` is limited by IP, tenant header and scope with
  no recipient dimension, and a word search of `password-reset.ts` for
  cooldown, throttle, resend and recently finds nothing. That is not proof that
  no other control exists there; the artifact states it as a search result.

The checker recomputes the verdict from the recorded limiter dimensions and the
cited SQL lines (predicate inside the `ON CONFLICT` statement, setting name and
default). It does not edit the `C-07` register row, which belongs to the
orchestrator.

### What the inventories show

- Rate-limit settings reach the limiter through five parsers or literals. Four public auth
  endpoints (`password/forgot`, `password/reset`, `register`, `sso/{providerKey}/start`)
  use a bare `Number(process.env.X ?? default)` with no validation, so a
  non-numeric value yields `NaN` (and an empty one `0`); the others fall back to
  the default on an invalid value. `AUTH_SOURCE_RATE_LIMIT_*` uses its own
  `parseInt` ceiling parser.
- `checkAuthRateLimit` always checks a shared per-client-IP bucket first, then a
  per-route bucket keyed by client IP, tenant and scope. In the 13 tenant-keyed
  sites the tenant value is read from the request (`x-awcms-tenant-id`, a
  cookie, a query parameter or the body) before any lookup, so only the source
  bucket cannot be chosen by the caller.
- The shared limiter fails open when Redis is configured but a command fails;
  without Redis it counts in a per-process `Map` (50 000 bucket cap).
- Passwords: Argon2id through `Bun.password` with no options, so the
  parameters are the runtime defaults, stated in a source comment and visible
  in the PHC prefix of a constant dummy hash: v19, m 65536 KiB, t 2, p 1. They
  are stored in `awcms_identities.password_hash` (`NOT NULL`) and
  `awcms_principals.password_hash` (nullable until a first login promotes it).
  No code path reads another hash format and none rehashes on login.
- Session tokens are 32 random bytes in base64url; only `sha256:<hex>` is
  stored (`awcms_sessions.token_hash`, no database `CHECK`). Machine credentials
  are `awcmsm_<tenant uuid hex>_<43 chars>`, stored as `mc-sha256:<hex>` with a
  database `CHECK` that matches the prefix the code writes. Six hash namespaces
  that route a bearer are pairwise distinct. The newsletter token hash has no
  prefix at all.
- Three columns are plaintext candidates: `awcms_oidc_auth_requests.nonce` and
  `code_verifier` (the migration says PLAINTEXT and why) and
  `awcms_push_subscriptions.auth_secret` (Web Push key material, with no comment
  saying it is intentionally unencrypted).

### Verification performed for this follow-up

- Generator run twice against the pinned checkouts: `diff -ru` empty. The 20
  committed static files (excluding `manifest.json` and `manifest.sha256`,
  which change because they list the new files) are byte-identical to the
  generator output.
- `tools/vg01_check_generated.sh` on the generated tree (post-freeze assembly,
  `pendingArtifacts` emptied): 19 checks pass; also passes with the artifacts
  present and still listed pending (the freeze job state).
- `bash tools/vg01_verify_frozen.sh contracts/legacy/awcms` passes against the
  committed corpus, reporting both artifacts as pending notes.
- Corrupted copies are rejected by the intended validator: a changed default, a
  flipped `C-07` verdict, a deleted cited handler, a dropped call site, a
  reclassified dimension, a moved line, the cooldown setting repointed at the
  per-IP variable, the cooldown predicate edited out of the frozen SQL, a
  changed Argon2 cost, a hash prefix that disagrees with the database `CHECK`,
  a dropped column, a PHC string carrying salt and digest, a digest-like hex
  run, the PLAINTEXT comment edited out of a migration, a changed cipher, an
  uncited extra file in the frozen tree, a fact citing the wrong line, and an
  absent artifact with an empty pending list.
- Unit tests of both tools pass (`rustc --edition 2024 -D warnings --test`).

### Findings outside this change

- `contracts/legacy/awcms/manifest.json` still marks `routes-auth-ownership`,
  `architectural-gates`, `representative-behavior-fixtures` and the vector
  citations as pending the freeze although the gate, route and behaviour
  artifacts are frozen; that register is not corrected here.
- The frozen `login-policy.ts` copy contains the constant dummy Argon2id hash
  that AWCMS publishes on purpose (its comment says it guards no account). It
  is a byte copy of the source and is not repeated in either artifact; a secret
  scanner pointed at the corpus may still flag it.
- `bash tools/vg01_verify_frozen.sh` should list `rate-limits.json` and
  `credential-formats.json` in its `required` array once they are frozen; that
  edit belongs with the freeze pull request.
- `config:validate` (named in the AWCMS source comments) was not read here, so
  whether it rejects a malformed value of the four unvalidated settings at
  startup is not known.
