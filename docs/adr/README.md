# Architecture Decision Records

An ADR records one decision, the forces that produced it, and what it costs.
It is written when the decision is made and never rewritten afterwards — a
decision that changes gets a **new** ADR that supersedes the old one, so the
reasoning history stays readable.

## Numbering

`NNNN-kebab-case-title.md`, allocated sequentially from `0001`. Numbers are
never reused, and an ADR is never deleted.

## Status vocabulary

| Status | Meaning |
|---|---|
| `Proposed` | written, not yet accepted |
| `Accepted` | binding |
| `Superseded by ADR-NNNN` | no longer binding; the successor says why |
| `Deprecated` | no longer binding, with no successor |
| `Rejected` | considered and declined; kept because the reasoning is useful |

## Relationship to the `AD-` register

The [architecture validation](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md)
carries a decision register (Appendix C) using identifiers `AD-01`…`AD-32`.
Those two systems are **not** duplicates, and keeping them distinct is
deliberate:

| | `AD-` register | ADR |
|---|---|---|
| Scope | the pre-Blueprint architecture selections made by the validation | any decision, at any time, by anyone |
| Form | one table row: status, rationale, consequence, gate | one document: full context and alternatives |
| Lifecycle | frozen when the Master Blueprint is signed off | continuous |

An `AD-` entry may be *expanded* into an ADR when its context outgrows a table
row. When that happens the register row gains an `ADR-NNNN` reference and the
ADR becomes the authoritative text. Until then the register row is
authoritative and no ADR should restate it — §2.1 of the validation is the
project's own argument against maintaining the same statement in two places.

## Index

| ADR | Title | Status |
|---|---|---|
| [0001](0001-record-architecture-decisions.md) | Record architecture decisions | Accepted |
| [0002](0002-semantic-versioning-and-changesets.md) | Semantic versioning with `vX.Y.Z` and changesets | Accepted |
| [0003](0003-executable-documentation-gate.md) | Documentation correctness is an executable gate | Accepted |

## Writing one

Copy [`TEMPLATE.md`](TEMPLATE.md), fill every section, and add a `minor` /
`added` / `adr` changeset. Add the row to the index above in the same change.
