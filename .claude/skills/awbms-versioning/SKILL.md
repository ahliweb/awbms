---
name: awbms-versioning
description: Version and release AWBMS using the vX.Y.Z model — author changesets, choose the right bump level, cut a release with scripts/version-bump.sh, and keep VERSION, CHANGELOG.md and the architecture validation header in agreement. Use when making any change that needs recording, when preparing a release or tag, or when asked what version something is or should become.
---

# AWBMS — Versioning and changesets

Policy: `docs/versioning/VERSIONING.md` · Authoring: `docs/versioning/CHANGESETS.md`
· Decision: `docs/adr/0002-semantic-versioning-and-changesets.md`

## The model in four lines

- One version, `X.Y.Z`, in the root `VERSION` file. No `v` prefix in the file.
- Git tags are annotated and `v`-prefixed: `v0.2.0`.
- Every change ships with a changeset in `.changeset/`.
- `scripts/check-docs.sh` fails if `VERSION`, `CHANGELOG.md` and the
  architecture validation header disagree.

## Choosing a bump level

AWBMS is a specification, not yet software. Bump levels are defined by **what a
reader has to re-check**, not by API surface.

| Level | When |
|---|---|
| `major` | A binding decision in Appendix C is **reversed or superseded**; a published contract breaks; a gate's pass rule tightens so prior evidence no longer discharges it |
| `minor` | Anything additive — a new decision, gate, assumption, section, ADR, skill, or script; a gate moving from specified to implemented |
| `patch` | Correction that changes no conclusion — typo, broken link, wrong count, completing a blank register field |

**The test:** *would someone who read the previous version and acted on it now
act differently?* If yes, it is not a patch.

Clarifying a decision is `patch`. Reversing it is `major`. Adding a second
decision beside it is `minor`.

## The `0.x` rule — read this before promising a version number

While `MAJOR` is `0`, a `major` changeset bumps **MINOR**. `version-bump.sh`
applies this automatically and prints a note explaining it.

`1.0.0` is **reserved for the first production cutover accepted under §54
Go/No-Go**, and is declared deliberately:

```bash
scripts/version-bump.sh --set 1.0.0 --apply
```

Never let a routine release reach `1.0.0`. If someone asks for the version
after a breaking change during `0.x`, the answer is a MINOR bump, not `1.0.0`.

## Authoring a changeset

```bash
scripts/changeset-new.sh \
  --bump minor \
  --type added \
  --component architecture-validation \
  --summary "Add VG-17 covering consumer contract coverage"
```

`bump` and `type` are **independent**. `type` only chooses the Keep a Changelog
section. A `security` change is often a `patch`; a `removed` item is a `major`
once the platform is `1.x`.

| `type` | Section it lands in |
|---|---|
| `added` `changed` `deprecated` `removed` `fixed` `security` | the same-named heading |

`component` is a closed list in `.changeset/config.json`:
`architecture-validation`, `adr`, `governance`, `versioning`, `skills`,
`scripts`, `ci`, `contracts`, `workspace`.

### Writing the summary

It becomes the changelog line verbatim, read by someone deciding whether the
release affects them.

- Imperative mood, no trailing period (the script rejects one).
- Name the thing, not the file: `Correct the VG-04 owner`, not `Update docs`.
- Cite register IDs — `AD-04`, `OD-02`, `VG-01`. They survive renumbering and
  `check-docs.sh` verifies they resolve.

Poor: `Update docs` · `Fixed a bug` · `Various improvements to §4`

### Granularity

One changeset per logical change — not per commit, not per file. Six typo fixes
in one document is one `patch`. A typo fix *plus* a new gate is two: different
bump levels, different changelog sections.

## Cutting a release

```bash
scripts/version-bump.sh              # 1. preview — writes nothing
scripts/version-bump.sh --apply      # 2. rewrite VERSION + CHANGELOG, consume changesets
scripts/check-docs.sh                # 3. verify
git add -A && git commit -m "chore(release): v0.3.0"
git tag -a v0.3.0 -m "v0.3.0"
```

Always preview first. The preview prints the computed version, the highest bump
found, any `0.x` note, and the exact changelog section that will be inserted.

## Rules that are not negotiable

1. `VERSION` is edited **only** by `version-bump.sh`. A hand edit is a review
   defect — it silently desynchronises the changelog and the doc header.
2. A released `CHANGELOG.md` section is never rewritten. Corrections land as a
   new entry in a later release.
3. A tag is never moved or deleted once pushed.
4. A release commit contains the bump and nothing else.
5. `check-docs.sh` must pass before tagging.

## Troubleshooting

| Symptom | Cause | Fix |
|---|---|---|
| `CHANGELOG.md has no '## [X.Y.Z]' section` | `VERSION` was hand-edited | Revert `VERSION`, re-run `version-bump.sh --apply` |
| `architecture validation header does not state vX.Y.Z` | Header row was reformatted, so the script's `sed` no longer matches | Restore the row to exactly `\| Specification version \| vX.Y.Z \|` |
| `No changesets — nothing to release` | Changesets were already consumed | Nothing to do, or author one |
| `body is empty; a changeset needs a summary line` | The file has frontmatter but no text | Add the summary line below the closing `---` |
| CI: "changes documentation but adds no changeset" | Expected — add one | `scripts/changeset-new.sh …` |

## When the Cargo workspace arrives

`VERSION` becomes the *platform* version; workspace crates may version
independently. Reconciling the two is follow-on work recorded in ADR-0002, and
`check-docs.sh` should gain a check that they agree. Until the workspace exists
(`AD-02`, §51 step 1), do not add `package.json`, `Cargo.toml`, or any tool
that requires a runtime — keeping this system dependency-free is the reason it
is written in shell.
