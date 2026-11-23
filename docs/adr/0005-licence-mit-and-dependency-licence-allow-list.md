# ADR-0005 — License AWBMS under MIT and enforce a dependency licence allow-list

| Field | Value |
|---|---|
| Status | Accepted |
| Date | 2026-11-23 |
| Deciders | Repository owner (decision of 2026-11-23) |
| Supersedes | none |
| Superseded by | none |
| Related registers | `OD-08` (closed by this ADR), `AD-22`, `VG-13` |

## Context

`OD-08` in the [architecture validation](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#appendix-d--open-decision-register)
asked two questions: what licence AWBMS itself carries, and what licence
allow-list applies to its dependencies. The second blocks `cargo-deny`, and
therefore the supply-chain part of `VG-13`.

`[F]` Before this decision the repository was inconsistent: the workspace
`Cargo.toml` declared `license = "MIT"`, while the README and `docs/README.md`
said no licence had been decided and no `LICENSE` file existed. The repository
is public on GitHub, so the inconsistency was visible to third parties.

`[F]` The dependency graph pinned in `Cargo.lock` (as of 2026-11-23) resolves
under `cargo deny list` to the permissive licences MIT, Apache-2.0,
Apache-2.0 WITH LLVM-exception, BSD-3-Clause, ISC, Unicode-3.0, Zlib and
CDLA-Permissive-2.0. Some crates are dual-licensed with a copyleft alternative
(for example LGPL-2.1-or-later on `r-efi`); a permissive alternative exists for
each of them.

What is **not** known: whether a future commercial or client deployment will
need a different licence for a derived product, and whether future
dependencies will stay within the permissive set.

## Decision

AWBMS is licensed under the MIT License, recorded in the root
[`LICENSE`](../../LICENSE) file and in the workspace `license` field. Dependencies
MUST resolve, for every crate, to at least one licence on the allow-list in
[`deny.toml`](../../deny.toml); the allow-list is limited to permissive licences
and is enforced by `cargo deny check` in CI. Adding a licence to the
allow-list, or a per-crate exception, MUST follow the exception policy in
[`docs/security/supply-chain.md`](../security/supply-chain.md): named owner,
rationale, expiry and compensating control, with a changeset.

## Alternatives considered

| Alternative | Why not chosen |
|---|---|
| Leave the licence undecided | Default copyright would keep applying to a public repository, and `cargo-deny` would stay blocked. The owner chose to decide rather than default |
| Proprietary / all rights reserved | Contradicts the existing `license = "MIT"` metadata and the public repository; the owner chose MIT |
| Apache-2.0 or dual MIT/Apache-2.0 | Common in the Rust ecosystem and adds an explicit patent grant, but the owner chose MIT to match the existing metadata. Can be revisited under the trigger below |
| Allow copyleft licences in dependencies | Would impose obligations on derived products that ship AWBMS binaries; no dependency currently needs it, because every dual-licensed crate offers a permissive alternative |

## Consequences

### Accepted costs

- A dependency available only under a copyleft or unknown licence cannot be
  adopted without a recorded exception.
- MIT gives no explicit patent grant.

### Benefits

- The repository states one licence consistently, in `LICENSE`, `Cargo.toml`
  and the README.
- `cargo-deny` runs in CI, so licence drift in the dependency graph fails a
  pull request instead of being found later.

### Follow-on work

- Keep the allow-list derived from the real graph when dependencies change
  (Dependabot pull requests surface this).
- SBOM and licence reporting for release artifacts remain deferred until
  release artifacts exist, as recorded in `docs/security/supply-chain.md`.

## Verification

- `LICENSE` exists and `Cargo.toml` declares `license = "MIT"`.
- The `Supply chain (cargo-deny + RustSec)` job in
  `.github/workflows/verification.yml` runs `cargo deny --locked check`, which
  fails on any dependency licence outside the allow-list.

## Revisit when

- A dependency that AWBMS needs is available only under a licence outside the
  allow-list.
- A product or client contract requires a different licence for AWBMS or a
  derived product.
- The owner decides that an explicit patent grant (Apache-2.0) is required.
