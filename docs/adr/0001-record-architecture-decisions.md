# ADR-0001 — Record architecture decisions

| Field | Value |
|---|---|
| Status | Accepted |
| Date | 2026-08-31 |
| Deciders | *unassigned — see the approval record in the architecture validation, Appendix F* |
| Supersedes | none |
| Superseded by | none |
| Related registers | `AD-01` |

## Context

AWBMS begins as a specification. Its first artifact — the [architecture
validation](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md) — makes
roughly thirty architectural selections that will bind implementation work for
years, and it makes them **before any code exists**, on qualitative reasoning
rather than measurement.

That combination is exactly where undocumented decisions rot. A selection made
on 2026-08-29 for a reason that was obvious that week becomes, eighteen months
later, an unexplained constraint that somebody either cargo-cults or discards —
both wrong, and indistinguishable without the reasoning.

The validation already anticipates this in §2.1: AWCMS prose drifted from AWCMS
implementation, and the document imposes the rule that registries and
executable checks outrank prose. It then applies that rule to itself.

**[F]** At the point this ADR was written, the repository had no ADR practice
and no decision documents outside the validation's own registers.

## Decision

AWBMS records every architecturally significant decision as a numbered ADR in
`docs/adr/`, using [`TEMPLATE.md`](TEMPLATE.md). A decision is architecturally
significant when reversing it later would require changing code, contracts,
schema or operational procedure that other people have already built on.

ADRs are immutable once accepted. A decision that changes is recorded as a new
ADR that supersedes its predecessor, and the predecessor's status is updated to
point forward — that status line is the only permitted edit to an accepted ADR.

The `AD-` register in the validation's Appendix C is **not** replaced by this
practice. The two coexist under the division of labour set out in
[`README.md`](README.md): the register holds the pre-Blueprint selections as
table rows; ADRs hold decisions whose context does not fit a row, and every
decision made after the validation. No statement lives in both.

## Alternatives considered

| Alternative | Why not chosen |
|---|---|
| Register rows only, no ADRs | A table row holds a rationale sentence, not the alternatives, the forces, or the accepted costs. `AD-04` (Axum over Actix) already needed a full section of prose to be defensible — that is an ADR wearing a different hat. |
| ADRs only, migrate the `AD-` register into them | Thirty-two ADRs written retroactively, in one batch, by someone reconstructing reasoning they did not produce. That manufactures false confidence, and it would orphan every `AD-nn` cross-reference in a 2,300-line document. |
| Decisions in commit messages and pull request descriptions | Not discoverable, not indexed, and lost if the forge is migrated. A decision nobody can find has not been recorded. |
| A wiki | Same drift problem as prose, with no review gate and no version history tied to the code. |

## Consequences

### Accepted costs

- Every significant decision costs a document, not a sentence. Some decisions
  will be under-recorded because of that friction.
- Two decision systems exist during the specification phase. The boundary
  between them is a judgement call and will occasionally be got wrong.
- Superseded ADRs accumulate and must not be deleted, so the directory grows
  monotonically and readers must check status before trusting content.

### Benefits

- A decision's reasoning survives the departure of the person who made it.
- Alternatives are recorded, so a later reviewer can tell the difference
  between "considered and rejected" and "never thought of".
- Each ADR's **Revisit when** section converts a decision from permanent into
  conditional, which is what most architectural decisions actually are.

### Follow-on work

- The Master Blueprint must decide whether the `AD-` register freezes at
  sign-off or migrates into ADRs. That is a Blueprint deliverable, not a
  decision this ADR makes.

## Verification

```bash
scripts/check-docs.sh
```

The gate confirms that every `AD-`, `OD-` and `VG-` identifier referenced in
the validation resolves to a register row, and that every internal link in
every document resolves to a real heading. It does not — and cannot — verify
that a decision was worth recording.

## Revisit when

The Master Blueprint is signed off and the `AD-` register freezes. At that
point one decision system remains, and the division of labour described here
becomes obsolete rather than merely unused.
