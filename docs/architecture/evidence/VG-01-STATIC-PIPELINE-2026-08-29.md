# VG-01 Static Legacy Inventory Pipeline — 2026-08-29

**Gate:** `VG-01`  
**Status:** PARTIAL  
**Issue:** `#3`

This evidence record documents the first executable part of `VG-01`. It does not mark the gate PASS.

## Pinned inputs

- AWCMS: `11f2e95a47b1328a820f976d60f978c38a067903`
- AWCMS-Astro: `7b753be619244541b817d5d8e7d3b72cfe88d4f9`

The generator fails closed if either checkout HEAD differs from the pinned SHA.

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

## Dependency/supply-chain posture

The generator uses only the Rust standard library plus the runner's `git` and `sha256sum` executables. It does not add a Cargo dependency and does not alter the committed Rust dependency graph.

## Why VG-01 remains PARTIAL

Static repository evidence is not equivalent to a controlled live PostgreSQL state. The following remain required before `VG-01` may become PASS:

- controlled PostgreSQL RLS/policy introspection for the pinned AWCMS baseline;
- controlled PostgreSQL roles/grants/ownership introspection;
- executable/frozen authorization vectors;
- representative response/behavior parity fixtures;
- checked-in frozen output plus an orphan-fixture/usage check that proves each fixture is consumed by an AWBMS check/test.

The next work item is therefore to establish a controlled AWCMS PostgreSQL fixture and then freeze the generator output only after its static results and live DB evidence agree.
