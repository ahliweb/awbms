# AWBMS — AhliWeb Backend Management System

A Rust-native backend platform, currently at the **architecture specification**
stage.

| | |
|---|---|
| Version | see [`VERSION`](VERSION) · [`CHANGELOG.md`](CHANGELOG.md) |
| Stage | pre-Blueprint · conditionally approved to begin Stage 1 |
| Repository state | Stage 1 in progress — Blueprint, frozen AWCMS inventory, and a Rust verification spike |

## What this repository is right now

**There is no AWBMS application yet.** No module, no HTTP surface, no
`migrations/`. Nothing here has been benchmarked or deployed.

What exists is Stage 1 groundwork:

- the [architecture validation](docs/architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md) —
  a decision record for a Rust backend (Axum, Tokio, PostgreSQL with SQLx, a
  modular monolith with FORCE RLS tenant isolation) plus the registers tracking
  which decisions are verified, assumed, or open;
- the [Master Blueprint](docs/architecture/AWBMS-MASTER-BLUEPRINT.md) and
  [Stage 1 gates](docs/architecture/AWBMS-STAGE-1-GATES.md);
- a **frozen AWCMS inventory** under `contracts/legacy/awcms/frozen/`, pinned to
  source commits with SHA-256 manifests (`VG-01`, partial);
- a **Rust verification spike** in `crates/verification` proving Axum, Tokio,
  SQLx and FORCE RLS compose as assumed (`VG-03`, `VG-04` — both PASS).

Four of sixteen gates pass; one is partial; eleven are open. **No gate covering
AWBMS behaviour has passed** — parity, performance and rollback are all open.
Read §0 before §1.

## Layout

```text
awbms/
├── VERSION                     # single source of truth for vX.Y.Z
├── CHANGELOG.md                # generated from changesets
├── CONTRIBUTING.md
├── .changeset/                 # pending change records
├── scripts/
│   ├── changeset-new.sh        # author a changeset
│   ├── version-bump.sh         # consume changesets, cut a release
│   └── check-docs.sh           # the executable documentation gate
├── docs/
│   ├── adr/                    # architecture decision records
│   ├── architecture/           # the validation and its registers
│   ├── security/               # security documentation (mostly pending)
│   └── versioning/             # version model and changeset guide
└── .claude/skills/             # Claude Code skills for working in this repo
```

The Cargo workspace layout that this repository will eventually adopt is
proposed in [§9 of the
validation](docs/architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#9-proposed-cargo-workspace).
Only `crates/verification` exists so far; the module and app layout is still proposed.

## Start here

| If you want to | Read |
|---|---|
| Understand the architecture | [the validation](docs/architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md), §0 first |
| Know what is decided vs. assumed | its Appendices A–E |
| Know what blocks the Blueprint | its Appendix D, and §49.1 |
| Make a change | [`CONTRIBUTING.md`](CONTRIBUTING.md) |
| Cut a release | [`docs/versioning/VERSIONING.md`](docs/versioning/VERSIONING.md) |
| See every document | [`docs/README.md`](docs/README.md) |

## Checks

```bash
scripts/check-docs.sh
```

Verifies version consistency, that every register identifier resolves, that
every internal link resolves, that the repository remains independent of AWCMS
(`VG-15`), and that pending changesets are well-formed. CI runs it on every
push and pull request.

## Relationship to AWCMS

AWBMS is a **new platform**, not a translation of
[AWCMS](https://github.com/ahliweb/awcms). It inherits AWCMS's proven security
and behavioural invariants as *requirements*, and implements them with
Rust-native mechanisms.

This repository has no source or runtime dependency on AWCMS, and must not
acquire one. Compatibility is asserted only through frozen, provenance-stamped
fixtures under `contracts/legacy/awcms/`, each stamped with the AWCMS commit it
was taken from. That directory now exists and is populated; `VG-01` is partial,
not discharged.

## Licence

Not yet determined. See [`docs/README.md`](docs/README.md#known-gaps).
