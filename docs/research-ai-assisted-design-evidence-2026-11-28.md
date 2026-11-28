# AI-assisted design and specification evidence — backend contract and parity evidence

- Version: 0.1.0
- Date: 2026-11-28
- Status: research proposal only
- Related issue: #23
- Evidence snapshot: main AGENTS.md blob prefix 5f2e322579f7 (not commit SHA).

## Research evidence
Rapid evidence review, not a systematic review or vendor trial. [NgodingPakeAI](https://ngodingpakeai.com/) presents planning and codebase synchronization; [DesainPakeAI](https://desainpakeai.com/updates) describes contextual design feedback. Feature claims are not causal evidence. [Cui et al. 2026](https://doi.org/10.1287/mnsc.2025.00535) pooled 4,867 developers across three field experiments, reporting 26.08% more completed tasks (SE 10.3%); [METR 2025](https://metr.org/blog/2025-07-10-early-2025-ai-experienced-os-dev-study/) randomized 246 tasks with 16 experienced developers and reported 19% longer time; [2026 update](https://metr.org/blog/2026-02-24-uplift-update/) revises experiment design. Distinct outcomes and populations; no pooled effect, local savings or performance claim. [Perry et al.](https://arxiv.org/abs/2211.03622) found security and confidence risks in a bounded study. [ISO 12207:2026](https://www.iso.org/standard/90219.html), [ISO 9241-210:2019](https://www.iso.org/standard/77520.html), [ISO 29148:2018](https://www.iso.org/standard/72089.html), [Spec Kit](https://github.com/github/spec-kit), [DTCG 2025.10](https://www.designtokens.org/tr/2025.10/format/) support lifecycle, human-centred design, requirements and interoperability.

## Scope and pilot
Independent Rust backend; no transfer of authority, replatforming or production migration without approved parity, benchmarks, recovery and ADR.
Inventory existing API/event contracts at a pinned SHA; reconstruct observed vs approved vs inferred/unknown/proposed behavior; one synthetic contract-generated mock for a consumer and negative authorization/transaction tests. DESIGN.md/token N/A where no UI. Compare existing contract tests and do not create a parallel contract system.

## Acceptance and evidence
Reuse existing ADRs, requirements and canonical artifacts; trace requirement/AC → UI if applicable → data/permission/operation → test. Pin actual source SHA before pilot; label observed, approved, inferred, unknown and proposed. Cross-spec conflicts return to source and applicable Gate 9. Test with synthetic fixtures only; visual baselines, keyboard and manual plus automated [WCAG 2.2](https://www.w3.org/TR/WCAG22/) where UI applies; otherwise N/A with reason. Measure human+agent time including review/rework, cost, defects/security and task success against matched baseline. No fabricated CI/UAT results.

## Safety and decision
Concept-only independent adoption; no copying proprietary code/assets, no unapproved vendor dependency or context egress. [NIST SSDF](https://csrc.nist.gov/projects/ssdf/publications) 1.1 final; 1.2 draft. [Indonesia PDP](https://peraturan.bpk.go.id/Details/229798/uu-no-27-tahun-2022) and [copyright](https://peraturan.bpk.go.id/Details/38690/uu-no-28-tahun-2014) apply as relevant; no certification claim. Issue #23 owns deeper inspection, approval and validation. No implementation, architecture or gate approval. Documentation index and applicable gates require review before merge.
