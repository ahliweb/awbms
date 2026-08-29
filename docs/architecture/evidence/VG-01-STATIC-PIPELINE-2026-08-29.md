# VG-01 Legacy Evidence Pipeline — 2026-08-29

**Gate:** `VG-01`  
**Status:** PARTIAL  
**Issue:** `#3`

This evidence record documents executable static-source and controlled-live-database evidence work for `VG-01`. It does not mark the gate PASS.

## Pinned inputs

- AWCMS: `11f2e95a47b1328a820f976d60f978c38a067903`
- AWCMS-Astro: `7b753be619244541b817d5d8e7d3b72cfe88d4f9`

The static generator fails closed if either checkout HEAD differs from the pinned SHA.

## Executable static-source controls

`tools/vg01_inventory.rs`:

1. verifies both Git SHAs before reading evidence;
2. derives the AWCMS module inventory from `src/modules/index.ts` imports and requires exactly 24 registered base modules;
3. enumerates `sql/*.sql`, requires exactly 148 migrations, and records SHA-256 per migration;
4. derives AWCMS-Astro consumed/committed API paths from the AWCMS consumer-contract source and requires 13 consumed plus 2 committed paths;
5. preserves the security/execution distinction between build-time machine reads, anonymous browser reads, telemetry writes, newsletter security-sensitive writes, and future committed contracts;
6. freezes source copies of the module registry, generated repository inventory, OpenAPI, AsyncAPI, consumer-contract source, AWCMS CI/package metadata, and AWCMS-Astro contract/package metadata;
7. emits a deterministic manifest containing SHA-256 for every generated/frozen file;
8. intentionally records the resulting status as `partial-static-source-evidence`.

CI runs the generator twice from separately checked-out pinned repositories and recursively diffs both outputs. This proves deterministic generation and detects source-count drift without granting the job repository write permission.

## Controlled live PostgreSQL evidence

`tools/vg01_live_db.sh` deliberately uses the pinned AWCMS repository's real `bun run db:migrate` / `scripts/db-migrate.ts` runner rather than implementing a second SQL apply loop.

The job follows the security posture documented by AWCMS's own integration harness:

1. create a fresh ephemeral database with a purpose-built owner role;
2. permit that owner to be SUPERUSER only while the historical migration set runs, because migration 019 configures a custom PostgreSQL GUC on `awcms_app`;
3. immediately demote the owner to `NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE`;
4. inspect the resulting live PostgreSQL catalogs;
5. fail closed unless the pinned baseline produces 148 ledger entries, 152 AWCMS business tables, 134 RLS-enabled tables, 134 FORCE-RLS tables, and 18 non-FORCE/global tables;
6. fail if the owner, `awcms_app`, or `awcms_worker` has SUPERUSER/BYPASSRLS after demotion;
7. fail if `awcms_app` or `awcms_worker` owns a business table;
8. export deterministic evidence for roles, table ownership/RLS flags, policies, table grants, routine grants, and migration checksums.

The workflow uploads the live evidence as a read-only CI artifact; it does not grant GitHub contents write permission.

## VG-15 independence

`tools/vg15_independence.rs` prevents direct Git/Cargo/path/runtime coupling to AWCMS and rejects source/build/deployment symlinks outside documentation/contracts. Pinned AWCMS checkouts exist only under the CI-only `.legacy/` evidence path and are excluded from runtime dependency scanning.

## Dependency/supply-chain posture

The static generator and independence gate use only the Rust standard library plus runner-provided `git`/`sha256sum`. The live database check uses the pinned AWCMS lockfile and the AWCMS migration runner. No AWBMS Cargo dependency or lockfile update is introduced.

## Why VG-01 remains PARTIAL

Even after the live catalog job is green, `VG-01` remains PARTIAL until the reviewed evidence is frozen and consumed by AWBMS checks. Remaining work:

- executable/frozen authorization vectors;
- representative request/response/behavior parity fixtures;
- checked-in static and live evidence under `contracts/legacy/awcms/`;
- artifact-to-test/orphan-fixture enforcement;
- reconciliation of any static/live contradiction into the Master Blueprint.

A green CI artifact is evidence, but it is not yet the immutable compatibility corpus required for Stage-1 sign-off.
