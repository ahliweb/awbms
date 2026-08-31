# Changesets

Every change that a reader of `CHANGELOG.md` would care about ships with a
changeset. A changeset is a small Markdown file in this directory that records
**what changed and how it moves the version**, written at the time of the
change rather than reconstructed at release time from `git log`.

Full policy: [`docs/versioning/VERSIONING.md`](../docs/versioning/VERSIONING.md).
Authoring guide: [`docs/versioning/CHANGESETS.md`](../docs/versioning/CHANGESETS.md).

## Create one

```bash
scripts/changeset-new.sh \
  --bump minor \
  --type added \
  --component skills \
  --summary "Add the awbms-versioning skill"
```

This writes `.changeset/<timestamp>-<slug>.md`. Edit the body to add detail.

## Format

```markdown
---
bump: minor
type: added
component: skills
---

One-line summary, imperative mood, no trailing period.

Optional body paragraphs. Reference registers by ID (`AD-04`, `VG-01`,
`OD-02`) so the change is traceable to the decision it implements.
```

| Field | Required | Allowed values |
|---|---|---|
| `bump` | yes | `major`, `minor`, `patch` |
| `type` | yes | `added`, `changed`, `deprecated`, `removed`, `fixed`, `security` |
| `component` | yes | see `components` in [`config.json`](config.json) |

`bump` drives the version number. `type` drives which *Keep a Changelog*
section the entry lands in. They are independent: a `security` fix can be a
`patch`, and a `removed` item is usually `major` once the platform is 1.x.

## Release

```bash
scripts/version-bump.sh          # preview
scripts/version-bump.sh --apply  # rewrite VERSION + CHANGELOG, consume changesets
```

The release run deletes every consumed changeset file. Only `README.md` and
`config.json` are permanent residents of this directory.
