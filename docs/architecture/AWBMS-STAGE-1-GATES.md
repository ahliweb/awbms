# AWBMS Stage 1 — Verification Gate Register

**Status:** Stage 1 signed off 2026-11-23 ([ADR-0008](../adr/0008-close-od-03-05-06-resolve-assumptions-sign-off-stage-1.md))  
**Issue:** `#1`  
**Purpose:** authoritative Stage-1 evidence register derived from the architecture validation and Master Blueprint.

The status of each gate is also recorded in the machine-readable register [`gates.json`](gates.json), together with its evidence paths, executable check and, for a PASS gate, the AWBMS commit and workflow run that established it. `scripts/check-docs.sh` fails when the table below and `gates.json` disagree, when a PASS gate lacks evidence or a recorded commit/run, or when a gate with a present executable check is described as lacking automation. Final-main run record: [`MAIN-VERIFICATION-2026-11-23.md`](evidence/MAIN-VERIFICATION-2026-11-23.md).

A gate is **PASS** only when its named evidence exists and its pass/fail rule can be independently checked. Documentation that merely describes future evidence is not a passed gate.

## Gate Register

| Gate | Objective | Current status | Required evidence | Pass rule |
|---|---|---|---|---|
| `VG-01` | Freeze AWCMS/AWCMS-Astro source-of-truth inputs | **PASS** | provenance manifest, machine-generated inventories, contracts, authorization vectors, live DB RLS/role evidence where required | all required artifacts exist under `contracts/legacy/awcms/`, include source SHA + digest, and are consumed by checks/tests |
| `VG-03` | Resolve/pin Rust toolchain | **PASS** | exact `rust-toolchain.toml`, committed `Cargo.lock`, read-only CI with `--locked` | evidence recorded in `evidence/VG-03-VG-04-2026-08-29.md`; final CI verifies the committed graph and leaves the lockfile unchanged |
| `VG-04` | Validate Axum/Tokio/SQLx/PostgreSQL fit | **PASS** | minimal verification spike + real PostgreSQL integration tests | evidence recorded in `evidence/VG-03-VG-04-2026-08-29.md`; request lifecycle, graceful shutdown, tenant-local SQLx transaction context and non-superuser FORCE-RLS denial pass |
| `VG-05` | Correctness/security parity | **OPEN** | golden request/response/DB/auth/audit/event corpus | committed compatibility scope has no unexplained semantic diff |
| `VG-09` | Establish performance evidence | **OPEN** | reproducible benchmark corpus + environment manifest | measurements are repeatable and satisfy approved baseline/no-regression policy |
| `VG-12` | Prove recovery/rollback | **OPEN** | backup/restore + cutover rollback rehearsal | restore and routing/worker rollback complete within approved recovery criteria |
| `VG-15` | Preserve repository independence | **PASS** | dependency/source/runtime coupling check (`tools/vg15_independence.rs`, run in CI; plus `scripts/check-docs.sh`) | no AWCMS source/runtime dependency, submodule, path dependency, or production filesystem coupling |

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
VG-01 source inventory (PASS)
       |
       +----> Stage-1 decisions validated against reality
       |
VG-03 toolchain pin (PASS)
       |
VG-04 verification spike (PASS)
       |
       +----> Stage 1 signed off 2026-11-23 (open decisions, assumptions, approval: ADR-0006, ADR-0008)
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

`VG-01`, `VG-03` and `VG-04` are PASS, all Blueprint-blocking open decisions are closed (ADR-0006, ADR-0008), every assumption is resolved, and the owner's approval is recorded. **Stage 1 is signed off (2026-11-23).** Stage 2 (PRD) is next; implementation still waits for gate 9.

No later implementation result retroactively substitutes for missing Stage-1 evidence; findings must update the Blueprint and decision register explicitly.
