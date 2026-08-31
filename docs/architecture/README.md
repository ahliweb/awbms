# Architecture documentation

## Documents

| Document | What it settles |
|---|---|
| [AWBMS Rust Architecture Validation](AWBMS-RUST-ARCHITECTURE-VALIDATION.md) | Whether a coherent Rust architecture can be specified for AWBMS, and which selections it makes |

## Reading order

The document is 58 sections and eight appendices. Read it in this order rather
than front to back.

1. **§0 — How to read this document.** Not optional. It defines the claim
   labels and states what the document does and does not establish. Reading §1
   without §0 produces a confident misreading.
2. **§1 — Executive decision.** The one-screen summary of every binding
   selection.
3. **Appendix C — Decision register.** The authoritative form of §1, with
   rationale, consequence and gate per decision.
4. **Appendix D — Open decisions.** What is *not* decided. Four of these block
   Blueprint sign-off.
5. **Appendix E — Verification gates.** What evidence would make any of this
   more than a proposal. None has been produced.
6. Then the sections that matter for your work, via the map below.

## Section map

| Concern | Sections |
|---|---|
| Reading the document, epistemic status | §0 |
| AWCMS as a requirements source | §2, §2.1, §2.2, Appendix G |
| Repository independence | §3 |
| Stack selection and rationale | §4, §5, §6 |
| Macro architecture and workspace layout | §7, §9 |
| Modular monolith and module composition | §8, §10 |
| PostgreSQL, tenancy, RLS | §11 |
| Authentication, authorization, audit | §12, §13, §14 |
| Events, jobs, external I/O | §15, §16, §17, §18 |
| Storage, cache, contracts, observability | §19, §20, §21, §22 |
| Performance and scalability | §23, §24, §25, §48 |
| Security posture and supply chain | §26, §27, §45 |
| Quality gates and error model | §28, §29 |
| Overload, idempotency, abuse resistance | §30, §31, §32 |
| Privacy, licensing, standards | §33, §34, §35 |
| AWCMS compatibility and migration | §36, §37, §38, §39, §40, §41, §42 |
| Deployment and health | §43, §44 |
| Testing | §46, §47 |
| Blueprint entry, lifecycle, sequencing | §49, §50, §51, §52, §53 |
| Cutover and production validation | §54, §55, §56 |
| Registers | Appendices A–G |

## What is binding

Normative force lives in **Appendix C** (decisions) and **Appendix E** (gates),
plus capitalised RFC 2119 keywords in the body. Lowercase "should" and "must"
in the discussion sections carry none — see §0.3.

This split is deliberate. It keeps 2,000 lines of discussion readable while
keeping the auditable surface small enough to actually audit.

## What this does not contain

No threat model, ERD, data dictionary, RBAC/ABAC/RLS matrix, API contract, SLO
target, or implementation plan. Those are Master Blueprint deliverables listed
in §49.2, and the Blueprint has not begun.

No measurement of any kind. §23 is explicit that no benchmark has been designed
or run, and that no number in it may be quoted as a result.

## Adding to this directory

New architecture documents need an entry in the table above and in
[`docs/README.md`](../README.md). A decision belongs in an
[ADR](../adr/README.md) or the `AD-` register — see
[`docs/adr/README.md`](../adr/README.md) for which — not in a new prose
document that restates one.
