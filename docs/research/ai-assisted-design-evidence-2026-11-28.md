# AWBMS — AI-Assisted Design and Specification Evidence for Backend Contract and Parity Evidence

## Document control

- **Version:** v0.2.0
- **Updated:** 2026-11-28
- **Status:** RESEARCH / PROPOSED. Not an ADR, verification gate, requirement or implementation approval.
- **Related issue:** [#23](https://github.com/ahliweb/awbms/issues/23) (stays open; owns deeper inspection, approval and validation).
- **Evidence snapshot:** AWBMS `main` commit `264ba70dd57e5318ce0bfac5836edadc930a3177`; `AGENTS.md` blob `5f2e322579f7` on that commit. External pages were read on 2026-11-28.
- **Scope:** documentation only. No code, dependency, CI, contract or gate change. No test was run for this document.

Every design item, check, identifier and project-board mapping below is **proposed**. Nothing here is a rule in force, a gate decision or permission to implement.

## 1. Research evidence

This is a rapid evidence review, not a systematic review or a vendor trial. Reference status was checked at takeover (see §6).

- Two public pages of commercial AI planning and design tools were read: [planning tool](https://ngodingpakeai.com/) and [design tool](https://desainpakeai.com/updates). The first lists four features as navigation labels without describing them, and marks its design feature as coming soon. The second lists release entries dated 30 and 31 August and 1 to 23 September 2026. These are feature claims, not causal evidence, and they are not an endorsement or a dependency.
- [METR 2025](https://metr.org/blog/2025-07-10-early-2025-ai-experienced-os-dev-study/): 16 experienced developers completing 246 issues; reported 19% longer completion time with AI tools allowed. The page marks the result as not current. The [2026-02-24 update](https://metr.org/blog/2026-02-24-uplift-update/) describes a changed study design and selection effects; that this prevents a stable effect estimate is this document's own inference, not a quotation.
- [Perry et al.](https://arxiv.org/abs/2211.03622): participants with an AI assistant wrote significantly less secure code and were more likely to believe their code was secure, in a bounded study.
- [Cui et al.](https://doi.org/10.1287/mnsc.2025.00535): **not verified at takeover** (publisher page refused access). Year, sample size and effect sizes are deliberately not stated here.
- [GitHub Spec Kit](https://github.com/github/spec-kit) (clarification and read-only cross-artifact analysis steps) and the [Design Tokens Community Group format 2025.10](https://www.designtokens.org/tr/2025.10/format/) (a Community Group report of 28 October 2025, not a W3C Recommendation) were read for interoperability ideas.
- Standards: [ISO 12207](https://www.iso.org/standard/90219.html), [ISO 9241-210](https://www.iso.org/standard/77520.html) and [ISO 29148](https://www.iso.org/standard/72089.html): **not verified at takeover**; edition years are not stated. [WCAG 2.2](https://www.w3.org/TR/WCAG22/) is a W3C Recommendation of 12 December 2024. [NIST SSDF](https://csrc.nist.gov/projects/ssdf/publications): version 1.1 final (2022); version 1.2 is a draft released 17 December 2025.

Distinct outcomes and populations are not pooled. No local time saving, cost saving or performance claim is made.

## 2. Scope and pilot (proposed)

- Independent Rust backend. No transfer of authority, replatforming or production migration without approved parity, benchmarks, recovery evidence and an ADR.
- Inventory existing API and event contracts at a pinned commit. Classify behaviour as observed, approved, inferred, unknown or proposed.
- One synthetic, contract-generated mock for one consumer, plus negative authorization and transaction tests. Design-token and `DESIGN.md` artifacts are N/A where there is no UI.
- Compare with existing contract tests; do not create a parallel contract system.

## 3. Acceptance and evidence (proposed)

- Reuse existing ADRs, requirements and canonical artifacts. Trace requirement or acceptance criterion → UI (if applicable) → data, permission and operation → test.
- Pin the actual source commit before any pilot. Cross-spec conflicts return to the source artifact and the applicable Gate 9.
- Synthetic fixtures only. Where a UI exists: visual baselines, keyboard checks and manual plus automated WCAG 2.2 checks; otherwise N/A with a reason.
- Measure human plus agent time including review and rework, cost, defects, security findings and task success against a matched baseline. No fabricated CI or UAT result.

## 4. Safety and decision

- Concept-only adoption: no copying of proprietary code or assets, no unapproved vendor dependency, no context egress to third-party tools.
- Indonesian [PDP law](https://peraturan.bpk.go.id/Details/229798/uu-no-27-tahun-2022) and [copyright law](https://peraturan.bpk.go.id/Details/38690/uu-no-28-tahun-2014) apply as relevant. No certification or compliance claim.
- Issue #23 owns deeper inspection, approval and validation. This document gives no implementation, architecture or gate approval.

## 5. GitHub Projects coordination (proposed)

The research request asked for the steps of the product development flow to be coordinated through GitHub Projects (v2), integrated with the existing project management and not a second board. This is a direction noted in that request, not a recorded decision.

- Steps 1 to 8: track research and specification deliverables with dependency, reviewer, acceptance criteria and evidence URL. No implementation issue before the applicable Gate 9. Step 9: the gate tracker references a recorded decision and is never inferred from a board stage.
- Steps 10 to 12: one canonical issue per S/M unit, native parent/sub-issue and blocked-by relations where supported, milestone, priority and assignee preserved; the PR references the issue and CI evidence. Steps 13 to 15: staging and UAT only if applicable, release-readiness evidence and decision. Steps 16 to 18: deployment, validation, monitoring and follow-up.
- Proposed status mapping on an existing project: Rencana = backlog, Siap = applicable definition of ready, Berjalan = claimed, Menunggu = dependency, review or approval, Selesai = verified outcome, Batal = cancelled with reason. The lifecycle step is separate from the stage. A closed-unmerged PR, a duplicate or not-planned issue, or missing evidence is never marked Selesai automatically.
- Discover the live project owner, number, access and field IDs; add existing issues idempotently with a dry-run diff and readback; keep one canonical issue; no shared draft cards or confidential context. The board is coordination only, never merge or deploy authority. No project membership was checked for this document.

References: [Planning and tracking with Projects](https://docs.github.com/en/issues/planning-and-tracking-with-projects), [Using the API to manage Projects](https://docs.github.com/en/issues/planning-and-tracking-with-projects/automating-your-project/using-the-api-to-manage-projects), [Creating issue dependencies](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/creating-issue-dependencies), [About issue statuses](https://docs.github.com/en/issues/tracking-your-work-with-issues/learning-about-issues/about-issue-statuses). All four pages exist at takeover; their content was not audited.

## 6. Reference verification at takeover (2026-11-28)

The pages were opened read-only on 2026-11-28; a page that could not be opened is not treated as correct.

- **Supported:** the two tool pages (as described in §1), METR 2025 and its update, Perry et al., Spec Kit, Design Tokens 2025.10, SSDF 1.1 and 1.2 draft, WCAG 2.2, the PDP and copyright law pages (existence), and the four GitHub documentation pages (existence).
- **Corrected from the draft:** METR counted 246 *issues*, not tasks, and its page marks the result as not current; the Perry confidence statement is a paraphrase of "more likely to believe their code was secure".
- **Not verified at takeover:** Cui et al. (publisher refused access; year, sample size and effect sizes removed), ISO 12207 (catalog refused access), ISO 9241-210 and ISO 29148 (not opened; edition years removed).
- No reference was added. Four sources are marked not verified at takeover (Cui et al. and three ISO standards); all other references were opened and support the statements made about them.

## Changelog

### 2026-11-28 - v0.2.0

- Takeover: moved under `docs/research/`, adopted the repository's document-control layout, removed unverifiable figures, corrected METR and Perry wording, labelled every item proposed, added §6, added the index entry and a changeset.

### 2026-11-28 - v0.1.0

- Initial research draft (single session, unreviewed).
