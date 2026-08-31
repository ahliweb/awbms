# Changelog

All notable changes to AWBMS are recorded here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows the AWBMS `vX.Y.Z` model defined in
[`docs/versioning/VERSIONING.md`](docs/versioning/VERSIONING.md).

Entries are generated from changesets by `scripts/version-bump.sh --apply`.
Do not hand-edit a released section; add a changeset instead.

## [Unreleased]

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
