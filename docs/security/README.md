# Security documentation

**Security analysis here is draft specification.** The initial threat model
and privacy analysis (lifecycle step 3) exist as drafts awaiting human security
and legal review; no control is implemented or reviewed yet.

Saying so explicitly matters more than it might appear. An empty `docs/security/`
directory in a repository whose architecture document discusses RLS, RBAC,
ABAC, Separation of Duties and OWASP ASVS invites the assumption that security
work has happened somewhere. It has not.

## What exists today

| Document | Status |
|---|---|
| [Initial threat model](AWBMS-THREAT-MODEL.md) — DFD, trust boundaries, STRIDE `THR-001`…`THR-063`, proposed requirements `PRQ-` | Draft — awaiting human security review |
| [Privacy analysis](AWBMS-PRIVACY-ANALYSIS.md) — personal-data inventory, classification, retention, DPIA risks `DPR-`, proposed requirements `PPRQ-` | Draft — awaiting privacy/legal review; not legal advice |
| [Supply-chain policy](supply-chain.md) | Current |


Security *architecture* — controls proposed, not implemented or reviewed — is
in the architecture validation:

| Topic | Section |
|---|---|
| Authentication, sessions, password hashing, machine credentials | [§12](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#12-authentication-architecture) |
| Authorization pipeline, default deny, deny-overrides | [§13](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#13-authorization-architecture) |
| Audit architecture | [§14](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#14-audit-architecture) |
| Tenant isolation: application predicate + FORCE RLS | [§11.2](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#112-tenant-security) |
| Outbound HTTP, TLS, SSRF controls | [§18](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#18-outbound-httptls) |
| Rust security posture and the unsafe-code policy | [§26](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#26-rust-security-posture) |
| Supply-chain controls | [§27](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#27-supply-chain-security) |
| Abuse resistance and rate-limit dimensions | [§32](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#32-abuse-resistance) |
| Data lifecycle, privacy, legal hold | [§33](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#33-data-lifecycle-and-privacy) |
| Standards and regulatory mapping | [§35](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#35-standards-and-regulatory-mapping) |
| Worked security scenarios | [§45](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#45-security-examples) |

## What does not exist

| Missing | Required by | Blocked on |
|---|---|---|
| Human review of the threat model and privacy analysis | §50 step 3 | a security reviewer and counsel |
| RBAC + ABAC + RLS matrix | §50 step 6 | ERD, §50 step 5 |
| Security test corpus | §46 | implementation |
| Executable authorization parity harness (vectors are frozen as prose in `contracts/legacy/awcms/authorization/`) | §2.2 | `VG-01`, `VG-05` |
| Incident response process (vulnerability disclosure is in [`SECURITY.md`](../../SECURITY.md)) | — | an owner |
| Standards control mapping with evidence | §35 | deployment |

## Compliance status

**None claimed.** §35 lists the standards AWBMS intends to map controls to —
ISO/IEC 27001, OWASP ASVS, NIST SSDF, UU No. 27/2022 on personal data
protection, and others.

Listing a standard is not a claim of conformance to it. The validation is
explicit that compliance requires deployment, operational and organisational
evidence in addition to software controls, and no such evidence exists.

## Reporting a vulnerability

Report vulnerabilities privately as described in [`SECURITY.md`](../../SECURITY.md).

## Supply chain

The dependency policy is in force: `deny.toml` (licence allow-list set by
[ADR-0005](../adr/0005-licence-mit-and-dependency-licence-allow-list.md),
advisories, bans, sources) is enforced by `cargo deny` and `cargo audit` in CI.
Policy and exception handling: [`supply-chain.md`](supply-chain.md).
