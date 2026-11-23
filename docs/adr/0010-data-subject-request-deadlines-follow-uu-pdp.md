# ADR-0010 — Data-subject request deadlines follow UU PDP (supersedes PD-12 of ADR-0009)

| Field | Value |
|---|---|
| Status | Accepted |
| Date | 2026-11-23 |
| Deciders | Repository owner. Recorded approval, 2026-11-23: *"Ya"* ("Yes"), given in reply to the proposal to revise `PD-12` by a new ADR so that rights with a statutory period follow UU PDP (3 × 24 hours) and the 14-day target applies only to requests without one |
| Supersedes | [ADR-0009](0009-accept-stage-2-prd-owner-decisions-pd-01-22.md), **`PD-12` only**; every other decision of ADR-0009 stays in force |
| Superseded by | none |
| Related registers | `VG-14`; PRD `REQ-043`, `REQ-085`, `PD-12`; privacy analysis `OQP-02`, `DPR-15`, `PPRQ-04` |

## Context

`[D]` [ADR-0009](0009-accept-stage-2-prd-owner-decisions-pd-01-22.md) accepted
`PD-12`: verified data-subject export or erasure within **14 days**, pending legal
advice. The step-3 [privacy analysis](../security/AWBMS-PRIVACY-ANALYSIS.md) then
flagged that UU No. 27/2022 (Pelindungan Data Pribadi) may set shorter periods
(`OQP-02`).

`[C]` The orchestrator read the text of UU 27/2022 on 2026-11-23 at
[pasal.id](https://pasal.id/akn/id/act/uu/2022/27), an aggregator that names
peraturan.go.id as its source; the official PDF was not compared line by line.
The text sets **"paling lambat 3 x 24 (tiga kali dua puluh empat) jam terhitung
sejak Pengendali Data Pribadi menerima permintaan"** for:

| Article | Controller duty |
|---|---|
| Art. 30 (1) | Update and/or correct personal data (right of Art. 6) |
| Art. 32 (2) | Give access to, and a copy of, the data and its processing record (right of Art. 7) |
| Art. 40 (2) | Stop processing after consent is withdrawn (right of Art. 9) |
| Art. 41 (1) | Delay and/or restrict processing, partly or wholly (right of Art. 11) |

`[C]` The text states **no period** for ending processing and deleting or
destroying data (Art. 8 with Arts. 43–44), for portability (Art. 13) or for
objection to automated decisions (Art. 10, details deferred to a government
regulation). Art. 14 requires requests to be made as a recorded application
(*permohonan tercatat*). Art. 46 (1) sets 3 × 24 hours for written breach
notification to the data subject and the authority, without defining the event
that starts the clock.

What is **not** known: whether an implementing regulation or the authority sets
periods for the rights without one, how the exceptions of Art. 15 apply (the
page truncated that article), and the reading of counsel. Under AWBMS's
multi-tenant model the duty falls on the tenant as controller; AWBMS must let the
tenant meet it.

## Decision

`PD-12` is replaced by:

1. **Statutory rights — 72 hours.** For correction or update (Art. 30), access
   and copy (Art. 32), stopping processing after withdrawal of consent
   (Art. 40), and delaying or restricting processing (Art. 41), the system
   SHALL let the tenant complete the action within **3 × 24 hours of receipt of
   the recorded request**. Identity verification of the requester happens inside
   that window, not before the clock starts.
2. **Rights without a statutory period — 14 days.** Erasure or destruction,
   portability and objection keep the **14-day** internal target of the former
   `PD-12`.
3. **Shorter rule wins.** If counsel, an implementing regulation or the
   authority sets a shorter period for any right, that period applies without a
   further ADR; a longer one needs a new ADR.

Breach notification (Art. 46) is recorded here as a statutory fact; the
processor-to-controller notice time and incident severity scale remain part of
`PD-20` follow-on work.

## Alternatives considered

| Alternative | Why not chosen |
|---|---|
| Keep 14 days for every request | Conflicts with the 3 × 24-hour duties of Arts. 30, 32, 40 and 41 as read |
| 72 hours for every request | Stricter than the text requires for erasure and portability, which are the most expensive to complete across derived copies; can be adopted later if counsel advises |
| Wait for counsel before changing anything | Leaves an accepted value that appears to breach the Act in force; the shorter-rule-wins clause keeps counsel's later reading binding |

## Consequences

### Accepted costs

- Access exports and corrections must be achievable within 72 hours, including
  identity verification, which favours self-service flows and constrains the
  step-7 algorithm and step-8 contract.
- The legal reading rests on an aggregator copy of the text, not counsel.

### Benefits

- The accepted values no longer conflict with the operative text as read.
- The request clock has one source of truth per right, which `PPRQ-04` can
  implement.

### Follow-on work

- Counsel review of this reading, of Art. 15 exceptions and of any implementing
  regulation (`OQP-02` stays open for that review only).
- Fold the per-right clock into the PRD revision that adopts `PPRQ-03` and
  `PPRQ-04`.

## Verification

- PRD `REQ-043` states 14 days for erasure, portability and objection; PRD
  `REQ-085` states 72 hours for the four statutory rights; PRD §13 `PD-12` cites
  this ADR.
- ADR-0009 header names this ADR as superseding `PD-12` only.
- `scripts/check-docs.sh` passes.

## Revisit when

- Counsel, an implementing regulation or the authority sets or interprets a
  period differently.
- UU 27/2022 is amended.
