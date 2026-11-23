# Security Policy

## Status

**AWBMS is NOT PRODUCTION READY.** It is at the architecture-specification
stage. No production deployment exists, no AWBMS application module exists,
and no AWBMS behaviour gate has passed. See the
[README](README.md) and [`docs/README.md`](docs/README.md#known-gaps).

Do not deploy AWBMS, or connect it to real personal or business data, on the
basis of anything in this repository.

## Reporting a vulnerability

Please **do not** open a public issue, pull request or discussion for a
suspected vulnerability.

Report it privately through GitHub's private vulnerability reporting for this
repository:

<https://github.com/ahliweb/awbms/security/advisories/new>

Include, where you can:

- the affected file, document, workflow or commit;
- a description of the issue and its likely impact;
- steps or evidence to reproduce it;
- any suggested mitigation.

Maintainers acknowledge reports on a best-effort basis. There is no response
time commitment and no bug bounty while the project is pre-release.

## Supported versions

AWBMS is pre-release. Only the `main` branch is supported. No released version
receives security fixes, because no production deployment exists to be
patched. Tagged versions (`vX.Y.Z`, see [`VERSION`](VERSION)) are
documentation and specification milestones, not supported releases.

## Scope

In scope, once the relevant code exists:

- the AWBMS application and its crates;
- database migrations, PostgreSQL roles and row-level security (FORCE RLS)
  policies, including tenant-isolation defects;
- authentication, authorisation and audit logic;
- CI/CD workflows and release automation under `.github/workflows/`;
- dependency and supply-chain configuration (`Cargo.toml`, `Cargo.lock`,
  `deny.toml`, `rust-toolchain.toml`);
- the frozen legacy fixtures under `contracts/legacy/` and the tooling in
  `tools/` and `scripts/`, where they could mislead a compatibility or
  provenance decision.

Today, the security-relevant content is documentation. A defect in the
architecture or security specification is in scope when it would lead to an
insecure implementation, for example a control that is claimed but cannot be
enforced.

Out of scope:

- findings against AWCMS, AWCMS-Astro or any other repository, which should be
  reported to their own maintainers;
- denial-of-service testing against any host, and any test that touches
  systems you do not own;
- social engineering of maintainers;
- reports that a placeholder document is incomplete, unless the placeholder
  misstates its own status.

## Safe harbour

Good-faith research that follows this policy, avoids privacy harm and data
destruction, and gives maintainers a reasonable opportunity to fix the issue
before disclosure is welcome.
