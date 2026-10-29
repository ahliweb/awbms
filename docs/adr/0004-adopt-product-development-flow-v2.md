# ADR-0004 — Adopt product development flow v2.0.0

| Field | Value |
|---|---|
| Status | Accepted |
| Date | 2026-10-30 |
| Deciders | Repository owner (decision of 2026-10-30) |
| Supersedes | none (revises the lifecycle list in validation §50, which is discussion text, not an Appendix C decision) |
| Superseded by | none |
| Related registers | none (no `AD-`, `OD-` or `VG-` entry changes) |

## Context

Validation §50 lists an eighteen-step lifecycle with UX/UI at step 8, after the
ERD, the RBAC/ABAC/RLS matrix and the OpenAPI contract, and treats step 11
(implementation) as one undivided step.

The owner's cross-portfolio flow, maintained in `ahliweb/personal-coding`
(`docs/alur-pengembangan-produk.md`, decision log entry **DL-033**), was
revised to **v2.0.0** on 2026-10-30:

- UX/UI is visualised and usability-tested as a prototype **before** the data
  model and the contract, so what is built is what users need rather than what
  is convenient to store.
- Step 11 is split per slice into 11.1 frontend (against a mock generated from
  the contract), 11.2 backend, 11.3 integration.
- Definition of Ready (gate 9) gains a blocking defect: a screen with no data
  source in the contract.

`[F]` AWBMS is a separate backend project. It is **not** part of the AWCMS
family, and the portfolio flow document does not list it among the repositories
it applies to. `[F]` The adoption below is therefore an owner decision for this
repository, not something inherited from a family rule.

`[A]` The prototype and user-research steps assume there will be human users of
some AWBMS-facing screen. AWBMS today has no frontend and no PRD; whether a
given slice has a screen of its own is decided per slice at step 4.

## Decision

AWBMS adopts product development flow v2.0.0 for its lifecycle. Validation §50
is rewritten to the v2 step list; §51 and §52 state that module slices are
delivered 11.1 → 11.2 → 11.3. Authorisation remains enforced in the backend and
in PostgreSQL, never frontend-only. Gate 9 (Definition of Ready) and gate 15
(Go/No-Go) are unchanged and remain human decisions. Where AWBMS has no
frontend, 11.1 is performed by the consuming repository against the contract
mock; this ADR creates no frontend component.

Step-number translation (citations written before this ADR use the old column):

| Old (≤ v0.3.0) | New | Step |
|---|---|---|
| 2 | 2 | User Research + PRD |
| 4 | 5 | ERD + Data Dictionary |
| 5 | 6 | RBAC + ABAC + RLS Matrix |
| 6 | 7 | Domain / Verification Algorithm Spec |
| 7 | 8 | OpenAPI + AsyncAPI + mock |
| 8 | 4 | UX/UI visualisation + validated prototype |
| 11 | 11 (11.1, 11.2, 11.3) | Implementation |
| 1, 3, 9, 10, 12–18 | same | unchanged |

The `§51 step N` numbering (the implementation sequence) is a different list
and is **not** renumbered.

## Alternatives considered

| Alternative | Why not chosen |
|---|---|
| Keep the v1 lifecycle in §50 | Leaves AWBMS on a flow the owner has retired; step 8 would put UX after the contract. |
| Reference the portfolio document instead of restating the list | The validation argues (§2.1) against stating one fact in two places without a mechanical check, but it also must stay readable offline; the list is short and this ADR records its source and date. |
| Renumber §51 as well | §51 item numbers are cited by `OD-02`, `OD-07`, `OD-08` and ADR-0002; renumbering buys nothing. |

## Consequences

### Accepted costs

- Citations of "§50 step N" written before this ADR now point at a different
  step; the table above is the only translation, and nothing checks it
  mechanically.
- The list is a copy of an external document and can drift from it.

### Benefits

- UX findings reach the data model and contract before they are fixed.
- A defined order for slices (mock-first) lets consumer work such as the
  AWCMS-Astro compatibility wave proceed without a running backend.

### Follow-on work

- When the Stage 2 PRD is written, include user research inputs (Blueprint
  §31 lists it).
- Per slice, decide at step 4 whether a screen exists; if none, record that.

## Verification

`grep -n "UX/UI" docs/architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md`
shows the item at step 4; `scripts/check-docs.sh` passes; `docs/security/README.md`
cites RBAC/ABAC/RLS as step 6.

## Revisit when

`ahliweb/personal-coding` publishes a flow version after v2.0.0, or AWBMS
acquires its first frontend of its own.
