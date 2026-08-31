# AWBMS documentation

Every document in this repository, what it is for, and how far it can be
trusted.

## Index

### Architecture

| Document | Status | Purpose |
|---|---|---|
| [AWBMS Rust Architecture Validation](architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md) | Conditionally approved | The binding architecture decisions and their registers. Start at §0. |
| [Architecture index](architecture/README.md) | Current | Navigation and reading order for the above. |

### Decisions

| Document | Status |
|---|---|
| [ADR index](adr/README.md) | Current |
| [ADR-0001 — Record architecture decisions](adr/0001-record-architecture-decisions.md) | Accepted |
| [ADR-0002 — Semantic versioning with `vX.Y.Z` and changesets](adr/0002-semantic-versioning-and-changesets.md) | Accepted |
| [ADR-0003 — Documentation correctness is an executable gate](adr/0003-executable-documentation-gate.md) | Accepted |
| [ADR template](adr/TEMPLATE.md) | — |

### Versioning

| Document | Purpose |
|---|---|
| [Versioning policy](versioning/VERSIONING.md) | The `vX.Y.Z` model, bump rules, release procedure |
| [Writing changesets](versioning/CHANGESETS.md) | When a changeset is required and how to write one |

### Security

| Document | Status |
|---|---|
| [Security documentation index](security/README.md) | Placeholder — see its own gap list |

## How to read a document here

Claims are labelled. The vocabulary is defined in [§0.2 of the
validation](architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#02-claim-labels)
and used across all documents:

| Label | Trust it because | Do not trust it for |
|---|---|---|
| `[F]` | It was checked against this repository at a stated commit | Anything about AWCMS or the outside world |
| `[C]` | Someone recorded it | Anything, until its gate is discharged |
| `[A]` | The design depends on it | It has not been established |
| `[D]` | It is binding | It may still be wrong; check the gate |
| `[R]` | It is a sensible default | A team may override it with a recorded decision |
| `[O]` | It is deliberately unresolved | Do not resolve it in passing |
| `[G]` | It names the evidence that would settle the matter | A gate reference is not a passed gate |

The most important thing to understand about this documentation set: of sixteen
verification gates, **four pass, one is partial, and eleven are open.**

- `VG-03`, `VG-04` — the Rust stack composes as assumed. Real evidence, narrow scope.
- `VG-15`, `VG-16` — standing structural conditions checked on every commit. They
  say nothing about whether an architectural claim is true.
- `VG-01` — **partial**. The AWCMS inventory is frozen and confirms `C-01`–`C-03`;
  `C-04`, `C-05` and `C-07` remain unverified.

**No gate covering AWBMS behaviour has passed.** Parity, performance and rollback
are all open, and no AWBMS module exists to test.

## Known gaps

These are absences, recorded so that nobody mistakes them for oversights.

| Gap | Blocked on | Tracked as |
|---|---|---|
| AWCMS inventory frozen but `VG-01` still **partial** — `C-04`/`C-05` gates not yet marked enforced/advisory/absent | per-gate determination | `VG-01` |
| No threat model, privacy analysis, ERD, RBAC/ABAC matrix, or SLOs | Master Blueprint, §50 steps 2–7 | §49.2, `OD-06` |
| No AWBMS application module — the workspace holds only `crates/verification` | Blueprint sign-off, §51 step 10 | `VG-02` |
| No performance baseline for AWBMS or AWCMS | benchmark corpus, §23.1 | `VG-09`, `OD-04` |
| No licence file, and no dependency licence allow-list | an ownership decision nobody has recorded | `OD-08` |
| No approver named on the architecture validation | an owner accepting the conditional approval | Appendix F |

The licence gap is worth stating plainly: this repository currently carries no
licence, which means default copyright applies and no third party has any right
to use it. That is a decision to make, not a file to forget.

## Documents that do not exist yet

The [validation's §9](architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#9-proposed-cargo-workspace)
proposes `docs/adr/`, `docs/architecture/` and `docs/security/`. The first two
now exist. `docs/security/` is a placeholder.

The [engineering lifecycle in
§50](architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#50-mandatory-engineering-lifecycle)
lists eighteen stages. Stage 1 — the Master Blueprint — has not begun. Its
required contents are in §49.2, and its entry conditions in §49.1.
