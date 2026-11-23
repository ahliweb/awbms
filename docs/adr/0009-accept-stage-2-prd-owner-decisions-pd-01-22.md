# ADR-0009 — Accept the Stage 2 PRD and its owner decisions PD-01 to PD-22

| Field | Value |
|---|---|
| Status | Accepted |
| Date | 2026-11-23 |
| Deciders | Repository owner. Recorded approval, 2026-11-23: *"Ya, setuju semua dg riset sesuai konsep rencana"* ("Yes, I agree to all of it, in line with the research and the planned concept"), given in reply to the list of owner decisions `PD-01`–`PD-22` in the draft PRD and the question whether to accept every proposal as written. The proposals were drafted by the orchestrating agent's sub-agents from the cited evidence, checked by the orchestrator, and accepted by that statement |
| Supersedes | none |
| Superseded by | none |
| Related registers | `OD-06` (values refined, not reopened); `OD-07` (owner assigned, still open); `OD-03`; `VG-05`, `VG-09`, `VG-12`, `VG-14` |

## Context

`[F]` Lifecycle v2 step 2 (User Research + PRD,
[ADR-0004](0004-adopt-product-development-flow-v2.md)) produced the
[AWBMS Stage 2 PRD](../product/AWBMS-PRD.md) and its
[user research input](../product/AWBMS-USER-RESEARCH.md), merged as drafts on
2026-11-23 (issue #61). The PRD collects every value it could not take from an
existing decision in §13 as owner decisions `PD-01`–`PD-22`, each with a
proposed value, rationale and the step it blocks.

The evidence base, and its limits:

- `[F]` Thresholds inherited from AWCMS (`PD-08` lockout 5 attempts / 15 min;
  `PD-10` newsletter 5 per 300 s and 900 s recipient cooldown) were checked by
  the orchestrator against the frozen `rate-limits.json` (`VG-01`).
- `[D]` Availability and latency starting values are those of
  [ADR-0008](0008-close-od-03-05-06-resolve-assumptions-sign-off-stage-1.md)
  (`OD-06`).
- `[A]` The research found **no production telemetry**: no tenant count, request
  rate or data volume. Every capacity, RTO and RPO value is therefore a
  sizing assumption, not a measurement.
- `[O]` Retention periods (`PD-11`) and the data-subject request deadline
  (`PD-12`) were proposed pending legal review; no counsel has reviewed them.

## Decision

The owner accepts the Stage 2 PRD and every proposed value in its §13 exactly as
written. In particular:

| Decision | Accepted value |
|---|---|
| `PD-01` Recovery | RTO / RPO: T1 60 min / 5 min; T2 120 min / 5 min (analytics events 60 min); T3 240 min / 15 min. A declared disaster event is excluded from the 99.9 % availability SLO; object-store content relies on provider durability |
| `PD-02` Async and error targets | Async processing 99.5 %; event dispatch lag p95 ≤ 30 s; email hand-off ≤ 5 min; 5xx ≤ 0.1 / 0.5 / 1 % for T1 / T2 / T3 |
| `PD-03` Screens | Decided per slice at step 4; contract-only until then |
| `PD-04` Latency tightening | Session and authorisation decision p95 ≤ 300 ms; search suggestions ≤ 300 ms; telemetry accept ≤ 300 ms; 500 / 900 ms stay the outer bounds |
| `PD-05` Capacity | The S1 and S2 table of PRD §8, including 25 concurrent authenticated users per tenant at S1 and 100 at S2 |
| `PD-06`–`PD-10` Identity and abuse | Sessions idle 30 min, absolute 12 h, step-up 10 min; revocation 5 s (sessions, credentials) and 60 s (tenant suspension); lockout 5 / 15 min; machine credentials ≤ 365 days, legacy credentials rejected 30 days after the wave 3 cutover; newsletter and telemetry limits as proposed |
| `PD-11`, `PD-12` Privacy | Analytics raw 90 days, operational logs 30 days, audit 2 years; data-subject requests answered within 14 days. **Adopted as working values; legal review remains a follow-on obligation, and a counsel finding that requires a change supersedes them** |
| `PD-13`, `PD-14` Migration | Shadow ≥ 14 days or 100,000 comparable requests (later of the two), ≥ 99.9 % equal, 100 % of differences explained, 0 authorisation differences; 7-day observation window and the rollback triggers of PRD §9 |
| `PD-15` Analytics identifiers | No raw network address; no persistent identifier across sites |
| `PD-16` Search store | `OD-07` is assigned to the repository owner; the S1 corpus size and query shape are measured before the first search implementation. `OD-07` itself stays open |
| `PD-17`–`PD-21` | Tenant provisioning ≤ 5 min and the initial SoD pair (role-grant request versus approval); 8 retry attempts then dead-letter; anonymous request body ≤ 16 KiB; quarterly restore rehearsal; search freshness and unpublish propagation ≤ 60 s |
| `PD-22` Persona `P-08` | Derived-application / module developers are not a v1 user group; module admission stays internal (Blueprint §14) |

Accepting the PRD closes lifecycle step 2. It authorises step 3 (threat model
and privacy analysis) and the later specification steps; it does **not**
authorise implementation, which still waits for gate 9.

## Alternatives considered

| Alternative | Why not chosen |
|---|---|
| Review a subset (`PD-01`, `PD-05`, `PD-11`/`PD-12`) before accepting | Offered to the owner; the owner chose to accept all proposals in one decision |
| Leave capacity and recovery values open until measured | Steps 3–9 need numbers to design against; the values are explicitly assumptions and are replaced when `VG-09` and `VG-12` produce evidence |
| Count disaster events against the SLO | With one primary database at S1, a single restore (60 min) exceeds the 43.2-minute monthly budget of 99.9 %; the SLO would be breached by design |

## Consequences

### Accepted costs

- Capacity, RTO and RPO values rest on no AWBMS measurement; they may prove
  wrong in either direction.
- Excluding declared disasters from the SLO weakens the availability promise for
  that class of event; it must be stated wherever the SLO is published.
- Retention and the subject-request deadline are in force before legal review.

### Benefits

- Every `REQ-` now has a decided threshold; steps 3–8 can design and specify
  tests against fixed numbers.
- `OD-07` has an owner, so it can no longer stall unnoticed.

### Follow-on work

- Legal review of `PD-11` and `PD-12` (and of the privacy analysis at step 3).
- Validate the research's untested assumptions (`UA-` rows) when the first
  tenants and telemetry exist; revise `PD-05` from `VG-09` results.
- Measure the S1 search corpus for `OD-07`.

## Verification

- The [PRD](../product/AWBMS-PRD.md) header states *Accepted* and cites this ADR;
  its §13 states every `PD-` row is accepted.
- The [validation](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#appendix-d--open-decision-register)
  Appendix D row for `OD-07` names the owner.
- `scripts/check-docs.sh` passes.

## Revisit when

- `VG-09` or `VG-12` evidence contradicts a capacity, latency or recovery value.
- Counsel advises a different retention period or subject-request deadline.
- A derived product needs to build modules on AWBMS (`PD-22`).
- A tenant needs a service tier the three tiers do not describe.
