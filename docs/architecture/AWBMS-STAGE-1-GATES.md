# AWBMS Stage 1 — Verification Gate Register

**Status:** Stage 1 in progress  
**Issue:** `#1`  
**Purpose:** authoritative Stage-1 evidence register derived from the architecture validation and Master Blueprint.

A gate is **PASS** only when its named evidence exists and its pass/fail rule can be independently checked. Documentation that merely describes future evidence is not a passed gate.

## Gate Register

| Gate | Objective | Current status | Required evidence | Pass rule |
|---|---|---|---|---|
| `VG-01` | Freeze AWCMS/AWCMS-Astro source-of-truth inputs | **PARTIAL** | provenance manifest, machine-generated inventories, contracts, authorization vectors, live DB RLS/role evidence where required | all required artifacts exist under `contracts/legacy/awcms/`, include source SHA + digest, and are consumed by checks/tests |
| `VG-03` | Resolve/pin Rust toolchain | **OPEN** | `rust-toolchain.toml`, build output, CI evidence | exact stable toolchain builds the verification workspace in CI |
| `VG-04` | Validate Axum/Tokio/SQLx/PostgreSQL fit | **OPEN** | minimal verification spike + PostgreSQL integration tests | tenant transaction context, non-superuser RLS denial, request lifecycle, SQLx behavior, timeouts/shutdown all pass |
| `VG-05` | Correctness/security parity | **OPEN** | golden request/response/DB/auth/audit/event corpus | committed compatibility scope has no unexplained semantic diff |
| `VG-09` | Establish performance evidence | **OPEN** | reproducible benchmark corpus + environment manifest | measurements are repeatable and satisfy approved baseline/no-regression policy |
| `VG-12` | Prove recovery/rollback | **OPEN** | backup/restore + cutover rollback rehearsal | restore and routing/worker rollback complete within approved recovery criteria |
| `VG-15` | Preserve repository independence | **CURRENTLY TRUE / UNAUTOMATED** | dependency/source/runtime coupling check | no AWCMS source/runtime dependency, submodule, path dependency, or production filesystem coupling |

## VG-01 Required Artifact Set

The final `VG-01` package MUST include:

1. `provenance.md` / machine-readable manifest.
2. AWCMS module registry inventory.
3. AWCMS migration inventory with per-file SHA-256.
4. AWCMS schema/table inventory.
5. RLS and `FORCE ROW LEVEL SECURITY` evidence.
6. DB roles/grants evidence from a controlled PostgreSQL instance.
7. Route inventory with owner/auth classification.
8. OpenAPI snapshot.
9. AsyncAPI snapshot.
10. Architectural gate inventory, classified as enforced/advisory/absent.
11. Authorization vectors.
12. AWCMS-Astro consumed/committed API surface.
13. Representative compatibility fixtures needed by parity tests.
14. Artifact-to-test usage mapping, so frozen fixtures cannot become orphaned.

## Known Pinned Baselines

- AWBMS validation revision observed: `a646bf24c809e2d47c3cb8b9eedeeab82a619cc6`
- AWCMS: `11f2e95a47b1328a820f976d60f978c38a067903` (`v10.1.0`)
- AWCMS-Astro: `7b753be619244541b817d5d8e7d3b72cfe88d4f9` (`v0.4.0` package state)

These baselines are starting evidence only. A future inventory refresh MUST intentionally record a new source SHA rather than silently replacing the old one.

## Gate Sequencing

```text
VG-01 source inventory
       |
       +----> Stage-1 decisions validated against reality
       |
VG-03 toolchain pin
       |
VG-04 verification spike
       |
       +----> Stage 1 may be signed off
                 |
                 v
             Stage 2 PRD

Later implementation/migration:
VG-05 parity
VG-09 performance
VG-12 rollback
VG-15 independence (continuous)
```

## Stage-1 Sign-Off Rule

Stage 1 MUST remain `IN PROGRESS` while any Blueprint-blocking assumption is unresolved or while `VG-01`, `VG-03`, or `VG-04` is not PASS.

No later implementation result retroactively substitutes for missing Stage-1 evidence; findings must update the Blueprint and decision register explicitly.
