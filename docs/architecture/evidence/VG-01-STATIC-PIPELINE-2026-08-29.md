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

`tools/vg01_inventory.rs` verifies both SHAs, derives the 24-module registry and 148-migration set, records SHA-256 per migration, derives the 13 consumed + 2 committed AWCMS-Astro paths with execution/security classes, freezes source candidates, and emits a deterministic SHA-256 manifest. CI runs it twice and recursively diffs the outputs.

## Controlled live PostgreSQL evidence

`tools/vg01_live_db.sh` uses the pinned AWCMS repository's real `bun run db:migrate` / `scripts/db-migrate.ts` runner rather than implementing a second migration engine. It then uses AWCMS's own `deriveTableRlsStates(loadMigrations())` code and compares that source-derived end-state name-for-name and RLS-state-for-RLS-state against PostgreSQL catalogs.

The job follows AWCMS's integration-harness security posture:

1. create a fresh ephemeral database with a purpose-built owner role;
2. permit that owner to be SUPERUSER only while the historical migration set runs, because migration 019 configures a custom PostgreSQL GUC on `awcms_app`;
3. immediately demote the owner to `NOSUPERUSER NOBYPASSRLS NOCREATEDB NOCREATEROLE`;
4. inspect PostgreSQL roles, table ownership, RLS/FORCE-RLS state, policies, table/routine grants, and the migration checksum ledger;
5. fail if runtime roles own business tables or if owner/app/worker retains SUPERUSER/BYPASSRLS.

### Resolved table-count scope discrepancy

The generated AWCMS inventory says 152 `awcms_*` tables, 134 FORCE-RLS tables, and 18 non-FORCE/global tables. The first live introspection incorrectly excluded `awcms_schema_migrations`, which made the result appear as 151 tables and 17 non-FORCE tables while all 134 RLS/FORCE-RLS tables matched.

This was an introspection-scope defect, not an AWCMS schema defect. `sql/001_awcms_foundation_schema.sql` declares `awcms_schema_migrations`, and `scripts/db-migrate.ts` also pre-creates that same ledger before applying migration 001. Therefore:

- **all `awcms_*` scope:** 152 tables = 134 FORCE-RLS + 18 non-FORCE, including the migration ledger;
- **business-table scope:** 151 tables = 134 FORCE-RLS + 17 non-FORCE, excluding only `awcms_schema_migrations`;
- the migration ledger contains 148 applied migration records at the pinned baseline.

The gate now records both scopes explicitly and compares the full 152-table source set against the full live 152-table set. This avoids mislabeling the migration ledger as a business/global domain table while preserving parity with AWCMS's generated repository inventory.

## VG-15 independence

`tools/vg15_independence.rs` prevents direct Git/Cargo/path/runtime coupling to AWCMS and rejects source/build/deployment symlinks outside documentation/contracts. Pinned AWCMS checkouts exist only under the CI-only `.legacy/` evidence path and are excluded from runtime dependency scanning.

## Dependency/supply-chain posture

The static generator and independence gate use only the Rust standard library plus runner-provided `git`/`sha256sum`. The live database check uses the pinned AWCMS lockfile and AWCMS migration runner. No AWBMS Cargo dependency or lockfile update is introduced.

## Why VG-01 remains PARTIAL

Even after the static and live catalog jobs are green, `VG-01` remains PARTIAL until the reviewed evidence is frozen and consumed by AWBMS checks. Remaining work:

- executable/frozen authorization vectors;
- representative request/response/behavior parity fixtures;
- checked-in static and live evidence under `contracts/legacy/awcms/`;
- artifact-to-test/orphan-fixture enforcement;
- reconciliation of any later static/live contradiction into the Master Blueprint.

A green CI artifact is evidence, but it is not yet the immutable compatibility corpus required for Stage-1 sign-off.
