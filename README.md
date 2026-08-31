# AWBMS — AhliWeb Backend Management System

A Rust-native backend platform, currently at the **architecture specification**
stage.

| | |
|---|---|
| Version | see [`VERSION`](VERSION) · [`CHANGELOG.md`](CHANGELOG.md) |
| Stage | pre-Blueprint · conditionally approved to begin Stage 1 |
| Repository state | specification and governance only — **no Rust code yet** |

## What this repository is right now

**It contains no Rust code.** There is no `Cargo.toml`, no `Cargo.lock`, no
`rust-toolchain.toml`, no `migrations/` and no `contracts/`. Nothing here has
been built, benchmarked or deployed.

What it does contain is the [architecture
validation](docs/architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md): a
decision record for a Rust backend — Axum, Tokio, PostgreSQL with SQLx, a
modular monolith with FORCE RLS tenant isolation — together with the registers
that track which of those decisions are verified, assumed, or still open.

The document is deliberately explicit that it is a *design*, not an
*implementation*. Every external claim it inherits carries a verification gate,
and none of those gates has been discharged. Read §0 before §1.

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
It does not exist yet.

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
fixtures under `contracts/legacy/awcms/` — a directory that does not exist yet,
and whose creation is the outstanding `VG-01` gate.

## Licence

Not yet determined. See [`docs/README.md`](docs/README.md#known-gaps).
