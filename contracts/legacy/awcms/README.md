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
    │   └── source/                # pinned copies (OpenAPI, AsyncAPI, CI, package.json, ...)
    │       └── tests/awcms/       # cited AWCMS test files (generated; same freeze run)
    └── live/                # controlled live PostgreSQL RLS / role introspection
```

Frozen files are published only through the `vg01-freeze` workflow
(generation with a read-only token, publication by pull request), and
`tools/vg01_verify_frozen.sh` fails CI on any orphan fixture or stale source SHA.

### Pending artifacts

`tools/vg01_inventory.rs` already generates `gates.json`, `routes.json`,
`behaviour-fixtures.json` and the cited test copies, and
`tools/vg01_check_corpus.rs` already validates them, but they are not committed
until the `vg01-freeze` workflow runs. Until then `fixture-usage.json` lists
them in `pendingArtifacts`: an absent pending artifact is a note, a present one
must validate and must be mapped, and the freeze workflow's own verification
therefore already enforces them. **The freeze pull request empties
`pendingArtifacts`** (after which absence is a failure). `verification.yml`
also checks, on every pull request, that the checker accepts the freshly
generated output (`tools/vg01_check_generated.sh`).

## What will be proven once the freeze pull request lands

This README does not discharge `VG-01`; the gate is **PARTIAL** in the
[Stage 1 gate register](../../../docs/architecture/AWBMS-STAGE-1-GATES.md). The
per-criterion state is in
[`VG-01-executable-coverage-2026-11-23.md`](../../../docs/architecture/evidence/VG-01-executable-coverage-2026-11-23.md).

Closed by the generator and checker extension (frozen by the next `vg01-freeze` run):

- an AWCMS gate inventory classified as enforced / advisory / absent, each entry citing its source (`gates.json`);
- a complete route inventory with owning module and auth classification, counts reconciled with the repo inventory and the frozen OpenAPI (`routes.json`);
- structured authorization vectors (actor, credential, tenant relation, operation, expected decision/status, cited test) whose citations resolve to frozen test copies;
- per-file migration SHA-256 recomputed from the SQL on every pull request (`tools/vg01_verify_migrations.sh`; effective as soon as the change merges, independent of the freeze).

Still missing:

- request/response body fixtures: the pinned source holds no body examples and no recorded responses, so `behaviour-fixtures.json` carries operation-level contracts and test-asserted status/error-code exchanges only;
- a parity harness that executes the structured vectors (AWBMS has no HTTP runtime to run them against, and several cited tests are static-source assertions rather than requests).

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
