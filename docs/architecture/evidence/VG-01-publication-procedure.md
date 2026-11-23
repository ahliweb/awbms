# VG-01 Evidence Publication Procedure

**Gate:** `VG-01`
**Issue:** `#7` (P0.1, part 1)
**Workflow:** `.github/workflows/vg01-freeze.yml`

This document describes how the frozen legacy evidence under `contracts/legacy/awcms/frozen/` is regenerated and published. It does not change any gate status.

## Trust boundaries

| Job | Token permissions | Runs |
| --- | --- | --- |
| `generate` | `contents: read`, plus `id-token: write` and `attestations: write` for artifact attestation | Pinned AWCMS dependency install, Rust compilation, migrations and introspection against a throwaway PostgreSQL service, determinism checks |
| `publish` | `contents: write`, `pull-requests: write` | Artifact download and verification, file copy, `git`, `gh`. No dependency install, build, migration, or repository script |

The workflow is `workflow_dispatch` only. `publish` additionally requires `github.ref == refs/heads/main` and the `vg01-publish` GitHub Environment.

## Procedure

1. Run the `Freeze VG-01 legacy evidence` workflow manually from `main`.
2. `generate` checks out the pinned AWCMS and AWCMS-Astro commits without persistent credentials, regenerates static and live evidence, verifies determinism, assembles `frozen/` (`static/`, `live/`, `provenance.json`), runs `tools/vg01_verify_frozen.sh`, writes `frozen.sha256` (a sha256 manifest of the whole tree), uploads the artifact `vg01-frozen-evidence` and attests `frozen.sha256`.
3. A reviewer approves the `vg01-publish` Environment deployment.
4. `publish` downloads the artifact, verifies the attestation (`gh attestation verify`), re-checks every digest in `frozen.sha256`, rejects non-regular entries and any file missing from or extra to the manifest, then replaces only `contracts/legacy/awcms/frozen/`.
5. `publish` fails closed if `git status --porcelain` lists any path outside `contracts/legacy/awcms/frozen/`. Otherwise it commits to a new branch `ci/vg01-freeze-<run>-<attempt>` and opens a pull request against `main`. It never pushes to `main`.
6. A human reviews and merges the pull request through the normal review path. If the corpus already matches, no branch or pull request is created.

## One-time repository setup

These are repository settings, not part of this change:

- Create the Environment `vg01-publish` with required reviewers.
- Enable "Allow GitHub Actions to create and approve pull requests" so `gh pr create` works with `GITHUB_TOKEN`.

Without them the `publish` job waits indefinitely or fails at PR creation; the evidence in `main` is unaffected.

## Regression check

`scripts/check-workflow-permissions.sh` runs in the `Documentation gate` job. It fails if any job holding `contents: write` (declared on the job or inherited from workflow-level `permissions`) installs dependencies, builds, runs migrations, or executes repository scripts or local actions. It is a heuristic text check over `run:` steps, not a substitute for review of new workflow steps.

## Rollback

Revert the change to `.github/workflows/vg01-freeze.yml`. Frozen evidence already merged is unaffected.
