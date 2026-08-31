---
name: awbms-docs-governance
description: Edit AWBMS specification documents without breaking their epistemic discipline — claim labels ([F]/[C]/[A]/[D]/[R]/[O]/[G]), the AD-/OD-/VG-/C-/AS- registers in Appendices A–G, and the no-duplicate-statement rule of §2.1. Use when writing, correcting, or reviewing anything under docs/, especially the architecture validation, or when adding a register entry, an ADR, or a claim about AWCMS.
---

# AWBMS — Documentation governance

The AWBMS specification is unusual: it is a 2,300-line architecture document
written **before any code exists**, whose central discipline is separating what
is *known* from what is *assumed*. That discipline is the document's whole
value. Editing it carelessly does not produce a slightly worse document — it
produces a document that looks authoritative while asserting things nobody
checked.

Authoritative sources:
- `docs/architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md` — §0 defines the rules
- `docs/adr/README.md` — ADR practice, and how it divides from the `AD-` register
- `CONTRIBUTING.md` — the change loop

## The one command

```bash
scripts/check-docs.sh
```

Run it before and after every documentation change. It catches dangling
register references, orphaned register rows, broken anchors, ambiguous heading
slugs, version drift, AWCMS coupling, and malformed changesets. It does **not**
catch a wrong claim label — that is on you.

## Claim labels: the rule that matters most

| Label | Means | You may write it when |
|---|---|---|
| `[F]` | Verified fact | You checked it **against this repository**, at a stated commit |
| `[C]` | Recorded external claim | Someone asserted it about a system outside this repo |
| `[A]` | Assumption | The design relies on it; it is not established |
| `[D]` | Decision | Binding until a recorded successor supersedes it |
| `[R]` | Recommendation | A default a team may override by recording a decision |
| `[O]` | Open decision | Deliberately unresolved; names the stage it blocks |
| `[G]` | Verification gate | The evidence that would discharge a claim |

**The failure mode to guard against.** A `[C]` claim that loses its label
becomes an unqualified statement, and an unqualified statement is one nobody
re-checks. Every AWCMS fact in this repository is `[C]` — the module list, the
migration count, the gate inventory, all of it. None has been verified. If you
find yourself writing "AWCMS has 24 modules" without a `[C]` and a gate
reference, stop.

**`[F]` is narrow on purpose.** It means checked against `ahliweb/awbms`.
"I read it in the AWCMS repo" is `[C]`. "I'm confident about the Rust release
schedule" is `[C]` and time-decaying — `C-06` is the worked example of why.

## Registers

Appendices A–G are tables, not prose. Every row has an ID in backticks in the
first cell, because `scripts/check-docs.sh` matches on `^| \`AD-01\` |`.

| Prefix | Register | Required fields |
|---|---|---|
| `C-nn` | A — external claims | source repo, source commit, observation, gate, disposition |
| `AS-nn` | B — assumptions | statement, relied on by, falsification consequence, validation method, stage |
| `AD-nn` | C — decisions | status, decision, rationale, consequence, section, gate |
| `OD-nn` | D — open decisions | question, why open, decision criteria, blocks, owner |
| `VG-nn` | E — gates | evidence artifact, executable check, pass rule, status |

### Adding an entry

1. Allocate the next free number **in the register**, never by guessing.
   Reserved and retired IDs are listed in each register — read them first.
2. Write the row with every field filled. `*unassigned*` is acceptable for
   owner; a blank cell is not.
3. Reference the ID from at least one body section. An unreferenced row fails
   the gate — a register row nobody cites is how a register becomes fiction.
4. Add a changeset (`--component architecture-validation`).

### Amending an entry

Decisions and gates are amended in place with the status field updated.
A **reversed** decision is not edited — it gets a successor decision, and the
original's status becomes `Superseded by AD-nn`.

## The no-duplicate-statement rule

§2.1 is the document's argument against itself: AWCMS prose drifted from AWCMS
implementation, so registries and executable checks outrank prose.

Applied here: **never state the same fact in two places without a mechanical
check keeping them equal.**

- The version number lives in `VERSION`, `CHANGELOG.md` and the validation
  header — and `check-docs.sh` asserts all three agree.
- §57 used to restate Appendix C as a second decision table. It was deleted
  rather than maintained. Do not bring it back.
- An ADR must not restate an `AD-` register row. Link to it.

When you catch yourself copying a table, either link to the original or add a
check.

## Anchors and cross-references

- Cite **identifiers** (`AD-04`, `VG-01`), not section numbers, in changesets
  and commit messages. Identifiers survive renumbering.
- Internal links use GitHub anchor slugs: lowercase, punctuation stripped,
  spaces to hyphens. An em dash leaves a double hyphen —
  `## Appendix F — Approval record` → `#appendix-f--approval-record`.
- `check-docs.sh` verifies every anchor resolves and that no two headings
  collide. Do not hand-verify; run it.

## Editing the architecture validation specifically

- **Do not renumber sections.** Cross-references throughout the document and
  in every other doc point at them.
- **Do not weaken a `[C]` into unlabelled prose** to make a sentence read
  better.
- **Do not discharge a gate in prose.** `VG-01` is discharged by an artifact
  under `contracts/legacy/awcms/` with a recorded source commit, not by a
  paragraph saying the inventory was done.
- **Do not resolve an `[O]` in passing.** Closing `OD-01` means recording a
  decision with rationale, not picking one of the three options mid-sentence.
- **§0.1 and §3.1 describe the repository's actual state.** If your change adds
  files, check whether those statements are still true.

## Adding an ADR

Copy `docs/adr/TEMPLATE.md`. Fill **every** section — an ADR with no
alternatives is a description, not a decision, and one with no *Revisit when*
trigger is a decision pretending to be permanent.

Then: add the index row in `docs/adr/README.md`, add the row in
`docs/README.md`, and add a changeset (`--component adr`).

Accepted ADRs are immutable. The only permitted edit is the status line, when
a successor supersedes it.

## Review checklist

- [ ] Every new assertion about an external system is `[C]` with a gate
- [ ] Every new `[F]` was actually checked against this repository
- [ ] Every new identifier has a register row with no blank fields
- [ ] Every new register row is referenced from a body section
- [ ] No statement was duplicated without a check keeping the copies equal
- [ ] No section was renumbered
- [ ] `scripts/check-docs.sh` passes
- [ ] A changeset exists with the right bump level
