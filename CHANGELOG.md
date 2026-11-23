# Changelog

All notable changes to AWBMS are recorded here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows the AWBMS `vX.Y.Z` model defined in
[`docs/versioning/VERSIONING.md`](docs/versioning/VERSIONING.md).

Entries are generated from changesets by `scripts/version-bump.sh --apply`.
Do not hand-edit a released section; add a changeset instead.

## [Unreleased]

## [0.4.0] — 2026-11-23

### Added

- **governance** — Add orchestrator-agent standard section and pelaksana agent definitions
- **adr** — Add ADR-0004 adopting product development flow v2.0.0
- **architecture-validation** — Add pre-production evidence specification for CI invariant gates and VG-05, VG-09, VG-12 evidence shapes (issue 7 P2.9, P2.10)
- **contracts** — Add VG-01 executable corpus checks, fixture-usage map and coverage evidence note
- **contracts** — Extend the VG-01 generator and checker with gate, route and fixture inventories, structured vectors and a migration SQL check
- **contracts** — Add VG-01 rate-limit and credential-format inventories: the generator emits rate-limits.json, credential-formats.json and the cited AWCMS implementation files (source/awcms-impl/) with source-line citations, check:rate-limits and check:credential-formats validate them (gated by pendingArtifacts until the next vg01-freeze run), and the C-07 finding is recorded from the code (confirmed)
- **scripts** — Add a read-only VG-01 legacy-baseline delta tool, its classification rules and a weekly workflow that report (never refresh) upstream movement from the frozen AWCMS and AWCMS-Astro SHAs, plus the 2026-11-23 evidence. Part of #7.
- **docs** — Add Fuchsia Rust SDK policy and locked-dependency portability matrix (issue #33) and a not-run FEMU HTTPS spike plan (issue #34); research only, no code, Cargo or gate changes.
- **governance** — Add root governance files: MIT LICENSE, SECURITY.md with private vulnerability reporting and a NOT PRODUCTION READY status, CODEOWNERS for high-risk paths, and a pull request template. The licence is recorded as MIT, matching Cargo.toml; the dependency licence allow-list remains open (OD-08).
- **adr** — Add ADR-0005 (MIT licence and dependency licence allow-list), closing OD-08; reconcile post-merge documentation: single gate-status source in the Blueprint, VG-01 register entry and contracts README, security index, architecture and docs indexes, and pin the docs workflow checkout
- **adr** — Add ADR-0006 accepting the Master Blueprint dispositions that close OD-01, OD-02, OD-04 and OD-09, and ADR-0007 keeping the VG-01 legacy baseline pinned for Stage 1; update Appendix D, §§ 4/23/37 open-decision notes, Appendix F and indexes
- **contracts** — Freeze the VG-01 gate inventory, route inventory, behaviour fixtures and cited AWCMS tests at the pinned baseline (vg01-freeze run 45102539207) and make them required by the corpus checks
- **contracts** — Freeze the VG-01 rate-limit dimensions, credential storage formats and cited AWCMS implementation files at the pinned baseline (vg01-freeze run 45106458867) and make them required by the corpus checks
- **adr** — Add ADR-0008 closing OD-03, OD-05 and OD-06, resolving AS-01–AS-08 and recording the owner's Stage 1 sign-off; mark the architecture validation and Master Blueprint approved for Stage 2 (not for implementation)
- **docs** — Add four non-binding blockchain research documents: Indonesian regulation evidence matrix with verified and unverified citations (#35), read-only EVM RPC finality and reorg spike specification (#36), hash anchoring versus permissioned ledger comparison (#37), and crypto custody, signing and ledger safety boundary (#38). Research only; no code, no legal conclusion, counsel review not performed.
- **architecture-validation** — Add evidence-based Fuchsia/Zircon and blockchain/crypto backend research notes

### Changed

- **architecture-validation** — Adopt product development flow v2.0.0 in the §50 lifecycle (UX/UI at step 4, slices 11.1 to 11.3)
- **governance** — Neutralise references to the owner's internal cross-portfolio standard in AGENTS.md and the agent definitions
- **dependencies** — Run `cargo update` within semver (54 crates, Rust toolchain 1.98.0 unchanged) and bump `actions/upload-artifact` to v7.0.2. PostgreSQL 18.6 was already pinned. The Bun 1.3.14 pin is intentionally kept: it belongs to the frozen VG-01 provenance (`contracts/legacy/awcms/frozen/provenance.json`), not to the AWBMS toolchain.
- **architecture-validation** — Document concept-only IDE/build diagnostic reference while preserving Rust backend Stage-1 verification and AWBMS independence. Issue #30.
- **architecture-validation** — Add machine-readable gate register `docs/architecture/gates.json` and a `check-docs.sh` section that fails on table/register drift, PASS gates without evidence or a recorded commit/run, and gates described as lacking automation while their executable check exists. Reconcile `VG-15` wording (PASS) and record the final-main run for `VG-03`/`VG-04`/`VG-15`. Issue #7.
- **ci** — Split VG-01 evidence generation (read-only token, attested artifact) from write publication (environment-gated, no install/build, pull request only) and add a workflow permission regression check. Issue #7, part 1.
- **versioning** — Add the docs and dependencies changeset components already used by pending changesets
- **architecture-validation** — Mark VG-01 PASS: all ten validation §2.2 inventory items are frozen at the pinned baseline and consumed by checks; confirm C-04 (with the migration-immutability exception), C-05 and C-07; fill the Appendix G source-evidence column; carry body fixtures and vector execution to VG-05

### Fixed

- **docs** — Neutralise the remaining reference to the owner's internal standard in ADR-0004

### Security

- **ci** — Add Rust quality gates (check, rustdoc with warnings denied, doc tests, workspace lints) and a supply-chain baseline (cargo-deny, cargo audit, Dependabot, exception policy). Bump tokio to 1.53.3 and url to 2.5.9 for RUSTSEC-2026-0386 and RUSTSEC-2026-0592. Issues #21 and #7.

## [0.3.0] — 2026-08-31

### Added

- **architecture-validation** — Add the Stage 1 Master Blueprint, gate register and invariant traceability
- **workspace** — Bootstrap the Cargo workspace and verification spike (VG-03, VG-04 PASS)
- **contracts** — Freeze the AWCMS source inventory under contracts/legacy/awcms (VG-01 partial)

### Changed

- **architecture-validation** — Confirm C-01, C-02 and C-03 against the frozen inventory; supersede C-06

### Fixed

- **architecture-validation** — Correct §0.1 and §3.1, which the Stage 1 merge falsified
- **scripts** — Exclude checksum-pinned legacy fixtures from documentation link checks

## [0.2.0] — 2026-08-31

### Added

- **versioning** — `vX.Y.Z` semantic version model with a `VERSION` file, `v`-prefixed git tags, and a documented bump policy for a specification-stage repository
- **versioning** — dependency-free changeset system in `.changeset/`, with `scripts/changeset-new.sh` to author entries and `scripts/version-bump.sh` to consume them into this changelog
- **scripts** — `scripts/check-docs.sh`, the executable documentation gate: version consistency, register-identifier resolution, internal anchor resolution, `VG-15` repository independence, and changeset well-formedness
- **ci** — GitHub Actions workflow running the documentation gate on every push and pull request
- **adr** — ADR directory, template, and `ADR-0001`–`ADR-0003` recording the ADR practice, the versioning model, and the ratification of the Rust stack
- **governance** — `README.md`, `CONTRIBUTING.md`, `docs/README.md`, `docs/architecture/README.md`, and `docs/security/README.md`
- **skills** — three Claude Code skills: `awbms-docs-governance`, `awbms-versioning`, and `awbms-architecture-review`
- **architecture-validation** — Appendices A–G populated as real register tables (external claims, assumptions, decisions, open decisions, verification gates, approval, invariant traceability), replacing prose descriptions of what the registers should contain

### Fixed

- **architecture-validation** — `AD-03` (Tokio) and `AD-05` (PostgreSQL) were referenced in §1 but absent from the decision register
- **architecture-validation** — `AS-01` was cited in §41 with no assumption register to define it
- **architecture-validation** — §0.3 listed `MUST` twice and omitted `SHOULD`, `SHOULD NOT` and `MAY` from the RFC 2119 keyword set
- **architecture-validation** — §0.4 reported 3 verified facts and cited two sections; there are 4, and §23 was missing from the list
- **architecture-validation** — §0.2 scoped unlabelled prose to §4–§56, silently excluding §57 and §58
- **architecture-validation** — §2.2 omitted the rate-limit dimension inventory item that §32 requires `VG-01` to capture
- **architecture-validation** — identifier sequences `AD-`, `OD-` and `VG-` contained unexplained numbering gaps; every allocated identifier is now either defined or explicitly recorded as reserved
- **architecture-validation** — §0.1 and §3.1 asserted a single-file repository at a stale commit; both now describe the current tree and are re-derived by `scripts/check-docs.sh` rather than hand-maintained

### Changed

- **architecture-validation** — the standalone "Document version 1.1" field is retired in favour of a single repository-wide `Specification version`, mechanically kept equal to `VERSION`

## [0.1.0] — 2026-08-29

### Added

- **architecture-validation** — initial AWBMS Rust backend architecture validation, conditionally approving entry to Stage 1 (Master Blueprint)

[Unreleased]: https://github.com/ahliweb/awbms/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/ahliweb/awbms/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ahliweb/awbms/releases/tag/v0.1.0
