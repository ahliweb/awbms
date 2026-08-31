# ADR-0002 — Semantic versioning with `vX.Y.Z` and changesets

| Field | Value |
|---|---|
| Status | Accepted |
| Date | 2026-08-31 |
| Deciders | *unassigned — see the approval record in the architecture validation, Appendix F* |
| Supersedes | none |
| Superseded by | none |
| Related registers | `AD-22`, `AD-31` |

## Context

**[F]** Before this decision the repository had no version number, no tags, and
no changelog. Its single document carried a private `Document version 1.1`
field maintained by hand.

That is workable for one document and one author, and fails immediately
afterwards. Three specific problems were already visible:

1. **No stable reference.** The document's own header cited its evidence basis
   as commit `5f8f9aa`, but the document had since been revised at `a646bf2`.
   A commit SHA cannot be written into the commit that contains it, so a
   hand-maintained SHA in a document header is guaranteed to lag by at least
   one revision. Readers had no citable identifier for "the version of AWBMS I
   read".
2. **A second version number.** `Document version 1.1` versioned one file
   inside a repository with no version of its own. Two independently maintained
   version numbers for one artifact is the drift pattern the validation's §2.1
   explicitly warns against.
3. **No record of change.** The distinction between "this release corrected a
   typo" and "this release reversed a binding decision" existed only in commit
   messages, where nobody consuming the specification would look.

AWBMS will eventually be a Rust workspace publishing HTTP and event contracts,
where versioning is not optional. Establishing the scheme now, while the
repository is small, costs a day; retrofitting it across a workspace, a
changelog history and a set of published contracts costs considerably more.

## Decision

AWBMS adopts **Semantic Versioning 2.0.0 in `vX.Y.Z` form**, with the single
source of truth in the root `VERSION` file and releases marked by annotated
`v`-prefixed git tags.

Because the repository is currently a specification rather than software, the
bump levels are defined by *what a reader must re-check*, not by API surface:
MAJOR reverses a binding decision or breaks a published contract, MINOR is
additive, PATCH corrects without changing any conclusion. The full policy,
including the `0.x` rule and the reservation of `1.0.0` for the first accepted
production cutover, is [`docs/versioning/VERSIONING.md`](../versioning/VERSIONING.md).

Change is recorded through **changesets**: a Markdown file with `bump`, `type`
and `component` frontmatter, written by the author of the change and consumed
at release time by `scripts/version-bump.sh` into `CHANGELOG.md`.

The implementation is **dependency-free POSIX shell** in `scripts/`. The
repository has no Node, Bun or Cargo toolchain, and the validation's `AD-02`
pins toolchain decisions to the Blueprint stage — so the versioning system must
not be the thing that forces a runtime into the repository ahead of that
decision.

## Alternatives considered

| Alternative | Why not chosen |
|---|---|
| Date versioning (`2026.08.31`) | Communicates recency, not compatibility. A reader cannot tell from `2026.09.02` whether a binding decision was reversed. The one question a version must answer, it does not answer. |
| The `@changesets/cli` npm package | The mature, obvious choice — and it requires Node plus a `package.json` in a repository whose entire subject is a Rust backend. It would also make a JavaScript toolchain a de facto dependency of reading the specification. Revisit once a workspace exists (see below). |
| `cargo-release` / `cargo-smart-release` | Requires the Cargo workspace that `AD-02` has not yet authorised. Genuinely the right answer later; unusable now. |
| Generate the changelog from Conventional Commits | Puts the changelog's quality at the mercy of commit hygiene, and offers no place to explain *why* a change matters — a commit subject line is written for reviewers, not for consumers of a release. |
| No versioning until code exists | The specification is the deliverable during this stage. "Which version of the architecture did you approve?" must have an answer before Blueprint sign-off, not after. |

## Consequences

### Accepted costs

- Bespoke shell tooling that the team must maintain, in place of a widely
  supported package. Roughly 400 lines across three scripts.
- Every change carries changeset authoring overhead, including trivial ones.
- Bump level is a judgement call. Two reviewers will sometimes disagree about
  `minor` versus `patch`, and the policy's rule of thumb — *would a reader who
  acted on the previous version now act differently?* — will not settle every
  case.
- Shell scripts are less portable than a packaged tool. They assume `bash`,
  `awk`, `sed` and GNU-ish `date`.

### Benefits

- A citable identifier for any state of the specification.
- One version number, mechanically enforced across `VERSION`, `CHANGELOG.md`
  and the validation header, so the drift §2.1 warns about cannot occur
  silently.
- The reason for a change is captured while it is still known.
- No runtime dependency is imposed on the repository ahead of `AD-02`.

### Follow-on work

- When the Cargo workspace is bootstrapped (`AD-02`, §51 step 1), reconcile
  `VERSION` with workspace crate versions. Crates may version independently;
  `VERSION` then denotes the platform release, and `scripts/check-docs.sh`
  gains a check that the two are consistent.
- Once contracts are published under `contracts/`, extend the MAJOR rule to
  cover contract compatibility and give it a verification gate.

## Verification

```bash
scripts/check-docs.sh   # asserts VERSION == CHANGELOG heading == doc header
scripts/version-bump.sh # previews the next version without writing anything
```

## Revisit when

Any one of:

- the Cargo workspace exists and `cargo-release` can do this natively;
- the shell scripts exceed roughly 600 lines or need a second maintainer;
- the repository acquires a Node toolchain for another reason, at which point
  `@changesets/cli` costs nothing extra.
