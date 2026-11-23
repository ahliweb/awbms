# Supply-chain controls

This page records the supply-chain controls that are **enforced today** in CI,
the policy for exceptions, and the controls that are **deliberately deferred**.
It implements the staged baseline requested in issue #7 (P1, item 6) and is
enforced by `.github/workflows/verification.yml`, `deny.toml` and
`.github/dependabot.yml`. It does not claim that any current dependency is
compromised; it makes the dependency graph reviewable and keeps it that way.

## Enforced today

| Control | Where | What it blocks |
|---|---|---|
| Locked dependency graph | `cargo ... --locked` on every cargo step; `git diff --exit-code -- Cargo.lock` after the tests | Lockfile drift; silent resolution changes |
| Exact version pins | `[workspace.dependencies]` uses `=X.Y.Z` | Unreviewed semver-compatible upgrades |
| `cargo deny check` | `supply-chain` job, policy in `deny.toml` | See the next table |
| RustSec scan | `cargo audit --deny warnings` in the same job | Known advisories, including unsound, unmaintained and yanked |
| Scheduled re-check | `schedule` trigger in the workflow (weekly, Monday 03:17 UTC) | Advisories published after merge, with no code change |
| Update proposals | `.github/dependabot.yml` (`cargo`, `github-actions`, weekly) | Stale dependencies; the PRs are reviewed, never auto-merged |
| Pinned CI actions | Every `uses:` is a full commit SHA with a version comment; workflow `permissions: contents: read` | Mutable-tag takeover of a CI action |
| Pinned tool versions | `taiki-e/install-action` (SHA-pinned) installs `cargo-deny@0.22.0` and `cargo-audit@0.23.0` | Unpinned scanner drift |
| `#![forbid(unsafe_code)]` | Crate root and `[workspace.lints.rust]` | Unsafe code in workspace crates, unless an ADR approves a narrow exception |

### `deny.toml` policy

| Section | Setting | Rationale |
|---|---|---|
| advisories | `yanked = "deny"`, no ignores | Every advisory is a failure until fixed or excepted below |
| licenses | allow-list of `MIT`, `Apache-2.0`, `Apache-2.0 WITH LLVM-exception`, `BSD-3-Clause`, `ISC`, `Unicode-3.0`, `Zlib`, `CDLA-Permissive-2.0` | Derived from the real graph with `cargo deny list` on 2026-11-23. Copyleft alternatives of dual-licensed crates are not allowed. |
| bans | `wildcards = "deny"`, `multiple-versions = "warn"` | Duplicate versions (mostly the RustCrypto 0.10 and 0.11 generations pulled in by `sqlx`) are visible but not yet blocking |
| sources | only the crates.io index; no git or alternate registries | No dependency from an unreviewed location |

Adding a licence to the allow-list is a policy change: it needs the same
record as an exception (below) unless the licence is permissive and the PR
description says so.

## Exception policy

An exception is any of: an `advisories.ignore` entry, a `licenses.exceptions`
entry, a `bans.skip` or `bans.deny` carve-out, an `allow-git` or extra
registry, or a `cargo audit` ignore.

Each exception must be recorded **in the same pull request** that adds it, in
the table below, and mirrored in the `reason` string of the `deny.toml` entry.
A pull request that adds an exception without a row here must not be merged.

| Field | Requirement |
|---|---|
| Owner | A named maintainer accountable for removing it |
| Rationale | Why it cannot be fixed now, and why it does not affect AWBMS (reachability, not just "not used") |
| Expiry | A calendar date, at most 90 days out. The weekly run surfaces an expired exception; renewing it needs a new row and fresh rationale |
| Compensating control | What limits the exposure meanwhile (feature not compiled, input not reachable, network egress blocked, and so on) |
| Removal condition | The upstream fix or replacement that lets the exception be deleted |

Current exceptions: **none.**

| ID or crate | Owner | Rationale | Expiry | Compensating control | Removal condition |
|---|---|---|---|---|---|
| (none) | | | | | |

### Fix before excepting

The first run of `cargo deny` on this baseline found two real advisories in
transitive and direct dependencies. Both had upstream fixes, so they were fixed
rather than excepted:

- `RUSTSEC-2026-0592` in `url` 2.5.8: `Cargo.lock` moved to 2.5.9.
- `RUSTSEC-2026-0386` in `tokio` 1.53.1: the workspace pin moved to `=1.53.3`.

## Deferred

These items from issue #7 are intentionally not implemented yet because there
is nothing for them to act on. Each has a trigger.

| Control | Deferred because | Trigger to implement |
|---|---|---|
| SBOM generation (CycloneDX or SPDX) | No release artifact exists; the only crate is `publish = false` | The first tagged release that ships a binary or image |
| Container and artifact vulnerability scanning | No production image exists | The first image build in CI |
| Artifact provenance and signing | No release artifact exists | Same as SBOM |
| `cargo vet` | The graph (about 140 crates, dominated by `sqlx`, `tokio` and `axum`) will still change as the schema and module work lands, so audits would be redone | The runtime dependency set is stable after Stage-1, and a trusted audit source (for example, imported audits from other projects) is chosen |

## Operating notes

- The scheduled run executes only the `supply-chain` job. The Rust and
  PostgreSQL jobs are skipped on `schedule`.
- A scheduled failure means a new advisory (or a yanked crate) landed. Treat it
  like a failing PR check: fix by updating the dependency, or record an
  exception above.
- Dependabot PRs are ordinary PRs. They need the full CI run and a human
  review; do not enable auto-merge for them.
- The security-update alerts configured in repository settings are separate
  from this file and are not changed by it.
