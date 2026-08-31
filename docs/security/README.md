# Security documentation

**This directory is a placeholder.** It contains no security analysis, because
none has been performed.

Saying so explicitly matters more than it might appear. An empty `docs/security/`
directory in a repository whose architecture document discusses RLS, RBAC,
ABAC, Separation of Duties and OWASP ASVS invites the assumption that security
work has happened somewhere. It has not.

## What exists today

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
| Threat model | §50 step 3 | Master Blueprint |
| Privacy analysis / DPIA | §50 step 3 | Master Blueprint |
| RBAC + ABAC + RLS matrix | §50 step 5 | ERD, §50 step 4 |
| Security test corpus | §46 | implementation |
| Authorization vectors imported from AWCMS | §2.2 | `VG-01` |
| Dependency policy (`deny.toml`, licence allow-list) | §27, §34 | Cargo workspace |
| Incident response and disclosure process | — | an owner |
| Standards control mapping with evidence | §35 | deployment |

## Compliance status

**None claimed.** §35 lists the standards AWBMS intends to map controls to —
ISO/IEC 27001, OWASP ASVS, NIST SSDF, UU No. 27/2022 on personal data
protection, and others.

Listing a standard is not a claim of conformance to it. The validation is
explicit that compliance requires deployment, operational and organisational
evidence in addition to software controls, and no such evidence exists.

## Reporting a vulnerability

There is no code to attack yet. When there is, a disclosure process belongs
here, and its absence is tracked in the gap table above.
