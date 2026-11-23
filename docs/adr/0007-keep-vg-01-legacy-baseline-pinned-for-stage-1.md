# ADR-0007 — Keep the VG-01 legacy baseline pinned for Stage 1

| Field | Value |
|---|---|
| Status | Accepted |
| Date | 2026-11-23 |
| Deciders | Repository owner (instruction of 2026-11-23 delegating the refresh decision; decision taken by the orchestrating agent and recorded here) |
| Supersedes | none |
| Superseded by | none |
| Related registers | `VG-01`, `VG-05`, `C-01`–`C-05`, `C-07` |

## Context

`[F]` The frozen VG-01 corpus under `contracts/legacy/awcms/frozen/` is pinned
to AWCMS `11f2e95a47b1328a820f976d60f978c38a067903` (`v10.1.0`) and
AWCMS-Astro `7b753be619244541b817d5d8e7d3b72cfe88d4f9`.

`[F]` The baseline-delta report of 2026-11-23
([`VG-01-baseline-delta-2026-11-23.md`](../architecture/evidence/VG-01-baseline-delta-2026-11-23.md))
shows upstream moved 440 commits (AWCMS) and 148 commits (AWCMS-Astro) past
the pin, including 424 API-contract, 153 migration/schema, 83
authorization/security and 6 Astro-consumer paths.

`[F]` `VG-01` is PARTIAL. The missing evidence (gate classification, route
ownership inventory, behaviour fixtures, executable vectors, per-file migration
hash check) is missing because the generator does not yet produce it, not
because the pin is old.

What is **not** known: which upstream AWCMS release AWBMS will finally cut over
from. AWCMS keeps changing while AWBMS is in Stage 1.

## Decision

The VG-01 baseline stays pinned at the SHAs above for the whole of Stage 1.
The missing VG-01 evidence is produced by extending the generator and running
the `vg01-freeze` workflow **at the same SHAs**. The baseline-delta check keeps
reporting upstream movement weekly; no automatic refresh happens. A refresh is
a separate decision, recorded in a successor ADR, and needs a provenance
update, regenerated hashes, a parity review and the affected tests.

## Alternatives considered

| Alternative | Why not chosen |
|---|---|
| Refresh to current AWCMS `main` now | Invalidates every existing frozen artifact and its review while adding no missing evidence. The target keeps moving (hundreds of contract paths), so a refresh now would likely be followed by another before cutover |
| Refresh to the latest AWCMS release tag now | Same cost as above. A tag is a better refresh target, so it is the one named in the revisit trigger |
| Unpin and track `main` continuously | Removes the reproducibility that `VG-01` exists to provide; contradicts the frozen-fixture rule in validation §3 |

## Consequences

### Accepted costs

- Stage 1 evidence describes AWCMS as of `v10.1.0`, which is increasingly far
  from current AWCMS. Requirements derived from it may need re-checking at
  refresh time.
- The weekly baseline-delta job keeps warning until a refresh happens.

### Benefits

- Existing evidence stays valid, and new evidence is directly comparable with
  it.
- `VG-01` can reach PASS on a stable target.

### Follow-on work

- Before `VG-05` parity-corpus work begins, decide the refresh: choose the
  AWCMS release tag to freeze, regenerate the corpus through `vg01-freeze`,
  and review the delta classes reported by `tools/vg01_baseline_delta.sh`.

## Verification

- `contracts/legacy/awcms/frozen/provenance.json` and the pins in
  `.github/workflows/verification.yml` and `vg01-freeze.yml` show the SHAs
  above; the corpus checker fails on any mismatch (stale-source-SHA check).

## Revisit when

- `VG-05` parity-corpus work is about to begin, or Stage 2 is entered —
  whichever comes first.
- The baseline-delta report shows a change in the migration-schema or
  authorization-security class that touches a capability in Blueprint §8.2 and
  that the owner considers material.
- AWCMS publishes a release that the owner designates as the cutover source.
