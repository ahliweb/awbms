# ADR-0003 — Documentation correctness is an executable gate

| Field | Value |
|---|---|
| Status | Accepted |
| Date | 2026-08-31 |
| Deciders | *unassigned — see the approval record in the architecture validation, Appendix F* |
| Supersedes | none |
| Superseded by | none |
| Related registers | `AD-32`, `VG-15`, `VG-16` |

## Context

The architecture validation contains an argument against itself. §2.1 records
that AWCMS documentation drifted from AWCMS implementation, and concludes:

> The current implementation registry, migrations, contracts, executable tests,
> and generated inventories SHALL outrank prose during AWBMS requirements
> extraction.

It then adds: *"This document is subject to the same rule it imposes."*

**[F]** Before this decision it was not. The v0.1.0 document contained, in one
2,357-line file:

- `AD-03` and `AD-05` referenced in §1 with no register entry anywhere;
- `AS-01` cited in §41 with no assumption register to define it;
- a fact count of "3" where four `[F]` statements existed;
- a claim of "exactly one tracked file" pinned to a commit that was no longer
  `HEAD`;
- registers described in prose as things that *must* contain fields, rather
  than tables that *did*.

Every one of these is the failure mode the document itself names. None of them
is a hard error to make; all of them are trivially detectable by a machine and
essentially undetectable by a human reading 2,357 lines for the fourth time.

A rule enforced by reviewer attention is enforced until the reviewer is tired.

## Decision

Documentation invariants that can be checked mechanically MUST be checked
mechanically, by `scripts/check-docs.sh`, and that check MUST run in CI on
every push and pull request.

The gate currently enforces nine invariants:

| # | Invariant |
|---|---|
| 1 | `VERSION` is a bare `X.Y.Z` |
| 2 | `CHANGELOG.md` has a released section for `VERSION` |
| 3 | The validation header states the same version |
| 4 | Every `AD-`/`OD-`/`VG-`/`C-`/`AS-` identifier referenced has a register row, **and** every register row is referenced somewhere else |
| 5 | Every internal `(#anchor)` link resolves to a real heading in the same file |
| 6 | No two headings in a file produce the same anchor slug |
| 7 | `VG-15`: no AWCMS submodule, and no Cargo manifest with a path or git dependency on AWCMS |
| 8 | Every pending changeset is well-formed |
| 9 | Every script is executable, and passes `shellcheck -S error` where available |

Invariant 4 is bidirectional deliberately. A referenced-but-undefined
identifier is a dangling pointer; a defined-but-unreferenced one is a register
row that has outlived whatever cited it, which is how a register silently
becomes fiction.

The gate is **advisory about content and binding about form**. It cannot tell
whether `AD-04` is a good decision; it can tell that `AD-04` exists, is
referenced, and is defined exactly once. That is a narrow guarantee, and
narrowness is the point — a check that tries to assess quality produces
warnings people learn to ignore.

## Alternatives considered

| Alternative | Why not chosen |
|---|---|
| Careful review | Already tried. It produced the six defects listed above, in a document explicitly written to be rigorous, revised twice. Attention does not scale to cross-referencing 2,357 lines. |
| A markdown linter (`markdownlint`, `remark`) | Checks style — heading levels, list markers, line length. It has no concept of a decision register, a claim identifier, or repository independence, which is where every one of the actual defects lived. Not mutually exclusive; simply not sufficient, and it would need a Node toolchain to deliver the part that was never the problem. |
| A link checker (`lychee`, `markdown-link-check`) | Solves invariant 5 and nothing else, and reaches the network to do it, making CI dependent on external site availability. Invariant 5 is a ten-line `awk` function against local headings. |
| Generate the document from structured data (YAML registers → Markdown) | The strongest alternative: it makes drift structurally impossible rather than merely detected. Rejected for now because it requires a template toolchain and makes the specification unreadable in its source form, which matters while the document is still being argued over rather than maintained. Worth revisiting once the registers stabilise at Blueprint sign-off. |
| Fix the defects and move on | Fixes this instance. Guarantees recurrence, since nothing changed about the conditions that produced it. |

## Consequences

### Accepted costs

- CI can fail on a documentation-only change, which some contributors will
  experience as friction on a "just a typo" pull request.
- The gate encodes assumptions about document structure — notably that a
  register row begins `| \`AD-01\` |`. Restructuring a register means updating
  the checker, and someone will eventually work around it rather than update it.
- The anchor slug algorithm reimplements GitHub's, and will diverge from it if
  GitHub changes. Divergence produces false failures, which are the expensive
  kind.
- False confidence: a green gate says the document is *internally consistent*,
  not that it is *true*. `C-01`–`C-07` can all be wrong with the gate passing.

### Benefits

- The document's central rule now applies to the document.
- Register integrity is checked on every commit rather than during whichever
  revision someone happens to be thorough about.
- `VG-15` (repository independence) becomes continuously verified instead of a
  claim someone re-reads occasionally.
- New contributors get their structural mistakes corrected by a script in
  seconds, rather than by a reviewer in days.

### Follow-on work

- `VG-01` — the AWCMS source inventory — needs its own generator script, which
  this gate should then verify the freshness of. That work is blocked on access
  to the AWCMS repository and is not attempted here.
- Extend the gate to contract conformance (`VG-08`) once `contracts/` exists.

## Verification

```bash
scripts/check-docs.sh          # 0 on success, 1 with a per-check FAIL line
scripts/check-docs.sh --quiet  # failures only; used by CI
```

CI: [`.github/workflows/docs.yml`](../../.github/workflows/docs.yml).

## Revisit when

Any one of:

- the registers stabilise at Blueprint sign-off, making generation from
  structured data cheaper than checking prose;
- the gate produces a false failure that blocks a correct change — a false
  positive in a required check is a defect in the check, not in the change;
- a Rust workspace exists, at which point these invariants may be better
  expressed as a `cargo xtask` alongside the module DAG and route ownership
  gates of §28.
