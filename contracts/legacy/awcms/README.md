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

## Planned directory shape

```text
contracts/legacy/awcms/
├── README.md
├── manifest.json
├── modules/
├── migrations/
├── database/
│   ├── tables-rls.json
│   └── roles-grants.json
├── routes/
├── openapi/
├── asyncapi/
├── authorization/
├── gates/
├── astro-consumer/
└── fixtures/
```

## What is NOT yet proven

This README does not discharge `VG-01`. In particular, AWBMS does not yet contain:

- the re-runnable inventory generator;
- per-file migration SHA-256 inventory;
- live PostgreSQL `pg_class` / `pg_policy` RLS introspection;
- live role/grant introspection;
- complete route/auth ownership inventory;
- frozen OpenAPI/AsyncAPI copies;
- authorization vectors;
- the full AWCMS-Astro consumer fixture;
- artifact-to-test usage checks.

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
