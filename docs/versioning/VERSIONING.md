# AWBMS versioning policy

| Field | Value |
|---|---|
| Status | Binding — ratified as [`ADR-0002`](../adr/0002-semantic-versioning-and-changesets.md) |
| Scheme | `vX.Y.Z`, Semantic Versioning 2.0.0 adapted for a specification-stage repository |
| Source of truth | the [`VERSION`](../../VERSION) file at the repository root |
| Enforced by | `scripts/check-docs.sh` |

## 1. One version, one file

The repository has exactly one version number. It lives in `VERSION` as a bare
`X.Y.Z` string with no `v` prefix and no trailing metadata.

Every other place the version appears is *derived*, never authored:

| Location | Form | Written by |
|---|---|---|
| `VERSION` | `0.2.0` | `scripts/version-bump.sh` |
| Git tag | `v0.2.0` | the releaser, after the bump commit |
| `CHANGELOG.md` heading | `## [0.2.0] — YYYY-MM-DD` | `scripts/version-bump.sh` |
| Architecture validation header | `\| Specification version \| v0.2.0 \|` | `scripts/version-bump.sh` |

`scripts/check-docs.sh` fails the build if any of these disagree. This is the
same rule the architecture validation imposes on itself in §2.1: a fact stated
in two places without a mechanical check between them is a drift source.

> **Why not a separate document version?** The original validation carried its
> own `Document version 1.1` field alongside no repository version at all.
> Two independently maintained version numbers for one artifact is precisely
> the pattern §2.1 warns against, so the document-local field was retired in
> v0.2.0.

## 2. What the numbers mean

AWBMS is currently a specification, not a running system. The bump rules are
therefore stated in terms of **what a reader has to re-check** when the version
moves — which is the property a version number is actually for.

### MAJOR — `X`

Bump when a consumer of this repository must revisit work they already
completed:

- a binding decision in Appendix C is **reversed or superseded** by a successor
  decision (not merely clarified);
- a published contract under `contracts/` changes incompatibly;
- a verification gate's pass/fail rule is tightened such that previously
  discharged evidence no longer discharges it;
- once code exists: any breaking change to a public HTTP, event, CLI or
  database-schema interface.

### MINOR — `Y`

Bump for additive change that leaves existing conclusions standing:

- a new decision, open decision, assumption, claim or verification gate;
- a new section, appendix, ADR, skill or script;
- a new module, endpoint or capability, once code exists;
- a gate moving from *specified* to *implemented*.

### PATCH — `Z`

Bump for correction and clarification that changes no decision:

- typo, formatting, broken link, wrong cross-reference;
- completing register fields that were left blank;
- rewording that a reasonable reviewer would agree preserves meaning;
- correcting a count, date or commit reference.

> **The test to apply:** if someone who read the previous version and acted on
> it would now act differently, it is not a PATCH.

## 3. The `0.x` phase

While `MAJOR` is `0`, AWBMS is pre-Blueprint and nothing in it is stable.

- A `major` changeset during `0.x` bumps **MINOR**, not MAJOR.
  `scripts/version-bump.sh` applies this automatically and prints a note.
- `1.0.0` is **reserved for the first production cutover accepted under §54
  Go/No-Go**. It is declared deliberately, never reached by accident:

  ```bash
  scripts/version-bump.sh --set 1.0.0 --apply
  ```

Planned milestones, for orientation only — these are not commitments:

| Version | Meaning |
|---|---|
| `0.1.0` | initial architecture validation |
| `0.2.0` | governance, versioning and documentation gates |
| `0.3.0` | `VG-01` discharged — frozen AWCMS source inventory exists |
| `0.4.0` | Master Blueprint signed off; every Blueprint-blocking `OD-` closed |
| `0.5.0`+ | Cargo workspace bootstrapped; modules implemented vertically |
| `1.0.0` | first production cutover accepted |

## 4. Release procedure

```bash
# 1. Every change carries a changeset (see docs/versioning/CHANGESETS.md)
scripts/changeset-new.sh --bump patch --type fixed \
  --component architecture-validation --summary "Correct the VG-04 owner"

# 2. Preview the release
scripts/version-bump.sh

# 3. Apply it — rewrites VERSION and CHANGELOG.md, consumes the changesets
scripts/version-bump.sh --apply

# 4. Verify, commit, tag
scripts/check-docs.sh
git add -A
git commit -m "chore(release): v0.3.0"
git tag -a v0.3.0 -m "v0.3.0"
```

Tags are annotated, never lightweight, so the release carries an author and a
date independent of the commit it points at.

## 5. Rules

1. `VERSION` is edited **only** by `scripts/version-bump.sh`. Hand edits are a
   review defect.
2. A released `CHANGELOG.md` section is never rewritten. Corrections land as a
   new entry in a later release.
3. A tag is never moved or deleted once pushed.
4. A release commit contains the version bump and nothing else.
5. `scripts/check-docs.sh` must pass before a tag is created.

## 6. Relationship to AWCMS versions

AWBMS versions are independent of AWCMS versions. `AWCMS v10.1.0` and a future
`AWBMS v1.0.0` share no numbering relationship, and no AWBMS version implies
compatibility with any AWCMS version.

Compatibility is asserted only by the frozen fixtures under
`contracts/legacy/awcms/`, each recording the AWCMS commit it was taken from
(§3.2). That provenance, not the version number, is what makes a compatibility
claim checkable.
