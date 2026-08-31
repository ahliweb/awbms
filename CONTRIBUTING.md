# Contributing to AWBMS

AWBMS is at the specification stage. Almost every change right now is a change
to a document, and the rules below reflect that.

## Before you change anything

Read [§0 of the architecture
validation](docs/architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#0-how-to-read-this-document).
It defines the claim labels — `[F]`, `[C]`, `[A]`, `[D]`, `[R]`, `[O]`, `[G]` —
that the whole project uses to separate what is known from what is assumed.
Writing an unlabelled assertion where a `[C]` belongs is the most common defect
in this repository, and it is the one that does real damage: a claim that loses
its label becomes a fact nobody re-checks.

## The loop

```bash
# 1. Make your change.

# 2. Record it.
scripts/changeset-new.sh --bump patch --type fixed \
  --component architecture-validation \
  --summary "Correct the VG-04 owner field"

# 3. Check it.
scripts/check-docs.sh

# 4. Commit.
git commit -m "docs(architecture): correct the VG-04 owner field"
```

Step 2 is not optional. See [`docs/versioning/CHANGESETS.md`](docs/versioning/CHANGESETS.md)
for when a changeset is required and how to choose the bump level.

## Rules

### Every claim carries its epistemic status

- `[F]` only for something verified **against this repository**, at a stated
  commit. Not "verified against my memory of AWCMS".
- `[C]` for anything asserted about an external system. It needs a register
  entry in Appendix A and a verification gate.
- `[A]` for anything relied upon but not established. It needs a register entry
  in Appendix B and a falsification consequence.
- Never promote a `[C]` to an `[F]` without the evidence artifact existing in
  the repository.

### Identifiers are the stable handles

`AD-04`, `OD-02`, `VG-01` survive section renumbering; "§6.2" does not. Cite
identifiers in changesets, commit messages and pull requests.

If you introduce a new identifier, add its register row in the same change.
`scripts/check-docs.sh` fails on a dangling reference **and** on an orphaned
register row.

### Do not state anything twice

The validation's §2.1 is the project's argument against duplicated statements:
two copies of the same fact drift, and the reader cannot tell which is current.
It applies to this repository's own documents.

If you find yourself copying a table, a version number or a decision into a
second place, either link to the first or add a check that keeps them equal.
The version number is the worked example — `VERSION`, `CHANGELOG.md` and the
validation header all state it, and `check-docs.sh` enforces that they agree.

### Decisions

- A decision that binds future work gets an ADR. See
  [`docs/adr/README.md`](docs/adr/README.md).
- An accepted ADR is never rewritten. Supersede it with a new one.
- An open decision (`OD-nn`) must not be quietly resolved in prose or in code.
  Close it explicitly, and record the closure as a decision.

### Repository independence

AWBMS MUST NOT gain a source or runtime dependency on AWCMS — no submodule, no
path dependency, no git dependency, no vendored source. This is `VG-15` and
`scripts/check-docs.sh` enforces it.

Legacy artifacts enter only as frozen fixtures under `contracts/legacy/awcms/`,
each stamped with source repository, source commit, import date, SHA-256, and
the parity test that consumes it.

### Shell scripts

- `bash`, `set -euo pipefail`, no non-POSIX dependencies beyond `awk`/`sed`.
- Must pass `shellcheck -S error`.
- Must be executable (`chmod +x`).

## Commit messages

Conventional Commits, with the register identifier in the subject where one
applies:

```text
docs(architecture): populate Appendix C with AD-03 and AD-05
feat(scripts): add repository independence check to check-docs.sh
fix(versioning): correct the 0.x major-bump rule in version-bump.sh
chore(release): v0.3.0
```

The changelog is generated from changesets, not from commit messages, so
commits are written for reviewers rather than for consumers.

## Pull requests

State which register identifiers the change touches, and whether any `[C]`,
`[A]` or `[O]` entry changes disposition. A pull request that closes an open
decision needs the decision text, not just the code.

`scripts/check-docs.sh` must pass. It runs in CI, but running it locally first
is faster than a round trip.

## When Rust code arrives

The gates in [§28 of the
validation](docs/architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#28-maintainability-and-code-quality-gates)
become mandatory at that point — `cargo fmt --check`, `clippy -D warnings`,
tests, `cargo sqlx prepare --check`, `cargo audit`, `cargo deny`, plus the
project-specific module DAG, route ownership and RLS checks. Do not add code
ahead of that CI; the first Rust commit and the CI that guards it belong in the
same change.
