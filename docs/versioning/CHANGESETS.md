# Writing changesets

A changeset is the record of *why* a version moved, written by the person who
moved it, at the moment they knew the answer. Reconstructing that from
`git log` at release time is guesswork; this system exists so nobody has to.

Policy: [`VERSIONING.md`](VERSIONING.md). Directory: [`.changeset/`](../../.changeset).

## When one is required

| Change | Changeset? |
|---|---|
| Any edit to a document under `docs/` that a reader would act on | yes |
| A new or amended register entry (`AD-`, `OD-`, `VG-`, `C-`, `AS-`) | yes |
| A new ADR, skill, or script | yes |
| A gate moving from specified to implemented | yes |
| Fixing a typo that changes no meaning | yes — `patch` / `fixed` |
| Reformatting whitespace with no textual change | no |
| The release commit itself | no |

When in doubt, write one. An extra `patch` entry costs a line in the changelog;
a missing entry costs a reader who cannot tell what changed.

## Creating one

```bash
scripts/changeset-new.sh \
  --bump minor \
  --type added \
  --component architecture-validation \
  --summary "Add VG-17 covering consumer contract coverage"
```

Then open the generated file and write the body.

## Anatomy

```markdown
---
bump: minor
type: added
component: architecture-validation
---

Add VG-17 covering consumer contract coverage

§21.1 requires consumer-contract coverage but no gate discharged it —
VG-08 checks conformance between specified and implemented routes, which
is a different claim. VG-17 names the evidence artifact and an owner.
```

### `bump` — how far the version moves

`major` · `minor` · `patch`, per [VERSIONING.md §2](VERSIONING.md#2-what-the-numbers-mean).
While the platform is `0.x`, a `major` is applied as a MINOR bump.

### `type` — which changelog section it lands in

Keep a Changelog vocabulary:

| Type | Use for |
|---|---|
| `added` | something that did not exist before |
| `changed` | existing behaviour or guidance that now differs |
| `deprecated` | still present, scheduled for removal, replacement named |
| `removed` | gone in this release |
| `fixed` | something that was wrong and is now right |
| `security` | a control, advisory response, or hardening change |

`bump` and `type` are independent. A `security` change is often a `patch`; a
`removed` item is a `major` once the platform is `1.x`.

### `component` — which part of the system

`architecture-validation` · `adr` · `governance` · `versioning` · `skills` ·
`scripts` · `ci` · `contracts` · `workspace`

The list is closed and lives in [`.changeset/config.json`](../../.changeset/config.json).
Adding a component is itself a `minor` / `changed` / `versioning` change.

## Writing the summary line

The summary becomes the changelog entry verbatim. It is read by someone
deciding whether this release affects them.

- Imperative mood: "Add", "Correct", "Remove" — not "Added" or "Adds".
- No trailing period. `changeset-new.sh` rejects one.
- Name the thing, not the file: "Correct the VG-04 owner", not "Update the doc".
- Reference register IDs. `AD-04`, `OD-02`, `VG-01` are the stable handles that
  survive section renumbering, and `scripts/check-docs.sh` verifies they
  resolve.

**Good**

```
Add VG-17 covering consumer contract coverage
Correct AD-03 and AD-05, which were referenced in §1 but never registered
Remove C-06; toolchain versions now live only in rust-toolchain.toml
```

**Poor**

```
Update docs.                      → which docs, and what changed?
Fixed a bug                       → past tense, and names nothing
Various improvements to §4        → a reader cannot tell if it affects them
```

## Body

Optional, and worth writing when the summary cannot carry the reason. Answer
what the summary leaves open: what was wrong, why it matters, what a reader
should now do differently. State the register IDs the change touches.

## Multiple changes

One changeset per logical change, not per commit and not per file. A pull
request that fixes six typos in one document is one `patch` changeset. A pull
request that fixes a typo *and* adds a verification gate is two — they release
under different bump levels and different changelog sections.

## Validation

`scripts/check-docs.sh` rejects a changeset that:

- does not begin with `---` frontmatter;
- is missing `bump`, `type` or `component`;
- carries a value outside the allowed vocabulary.

`scripts/version-bump.sh` additionally rejects one with an empty body, because
an entry with no summary would produce a blank changelog line.
