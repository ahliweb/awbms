# ADR-0006 — Accept the Master Blueprint dispositions of OD-01, OD-02, OD-04 and OD-09

| Field | Value |
|---|---|
| Status | Accepted |
| Date | 2026-11-23 |
| Deciders | Repository owner (instruction of 2026-11-23 to close the register entries the Blueprint already decides) |
| Supersedes | none |
| Superseded by | none |
| Related registers | `OD-01`, `OD-02`, `OD-04`, `OD-09` (closed by this ADR); successor to `AD-27` for the migration mode; `AD-02`, `VG-03`, `VG-09`, `OD-05`, `OD-06` |

## Context

`[F]` The [Master Blueprint](../architecture/AWBMS-MASTER-BLUEPRINT.md) contains
four sections headed as decisions: §8 (v1 scope, `OD-09`), §9 (migration mode,
`OD-01`), §10 (toolchain and dependency policy, `OD-02`) and §11 (performance
policy, `OD-04`). [Appendix D of the validation](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#appendix-d--open-decision-register)
still listed all four as open with owner *unassigned*. The two documents
therefore disagreed, and the validation's own rule is that an open decision is
closed by a recorded decision with rationale, not in passing.

`[F]` Each Blueprint section meets the decision criteria that Appendix D set
for it:

| Entry | Appendix D criterion | Blueprint disposition |
|---|---|---|
| `OD-01` | Choose a migration mode, weighing coexistence, downtime and reconciliation; record as a successor to `AD-27` | [§9](../architecture/AWBMS-MASTER-BLUEPRINT.md#9-migration-strategy--od-01-decision): deterministic migration into an AWBMS-native schema, capability by capability, strangler cutover, no blind dual-write, shared-schema takeover only by exception ADR |
| `OD-02` | Pin granularity, routine upgrade cadence, expedited RustSec path | [§10](../architecture/AWBMS-MASTER-BLUEPRINT.md#10-toolchain-and-dependency-policy--od-02-decision): exact stable toolchain pin, committed lockfile, monthly routine review, quarterly upgrade review, 24 h triage and 72 h remediation target for critical/high RustSec findings |
| `OD-04` | Per scenario, an explicit SLO or a "baseline-only, no regression beyond X%" rule | [§11](../architecture/AWBMS-MASTER-BLUEPRINT.md#11-performance-policy--od-04-decision): baseline-first no-regression thresholds per metric and a fixed scenario list; absolute SLOs deferred to `OD-06` |
| `OD-09` | Product priority against migration risk, module by module | [§8](../architecture/AWBMS-MASTER-BLUEPRINT.md#8-awbms-v1-scope--od-09-decision): 16 capabilities in v1 with a treatment each, 8 remaining in AWCMS, delivery waves 0–4 |

What is **not** known: the number of deployments and their tenant separation
(`OD-05`), absolute SLO targets (`OD-06`), and the accepted legacy credential
and contract formats (`OD-03`). This ADR does not decide them.

## Decision

The dispositions in Master Blueprint §8, §9, §10 and §11 are accepted as the
decisions that close `OD-09`, `OD-01`, `OD-02` and `OD-04`. The Blueprint
sections remain the single normative statement of each policy; this ADR
records the acceptance and does not restate them. For `OD-01`, the accepted
mode is the default for every deployment; a deployment that needs a different
mode (for example shared-schema takeover) MUST record it in its own ADR, as
Blueprint §9 rule 8 already requires. `OD-04` is closed with relative
(no-regression) thresholds only; acceptance against absolute targets stays
open under `OD-06`.

## Alternatives considered

| Alternative | Why not chosen |
|---|---|
| Keep the four entries open until Stage 1 sign-off | Leaves the validation contradicting the Blueprint, and keeps work that depends on the decisions (CI policy, migration engineering design, v1 scope) formally blocked although the content is settled |
| Close them by editing Appendix D only | Violates the rule that an open decision is closed by a recorded decision with rationale |
| Restate each policy in this ADR | Would create a second normative copy that can drift; the Blueprint is kept as the only statement |
| Decide `OD-01` per deployment now | `OD-05` (how many deployments exist) is still open; a default with an exception path is the decision available today |

## Consequences

### Accepted costs

- Changing any of the four policies now requires a successor ADR, not an edit
  to the Blueprint alone.
- `VG-09` acceptability is judged only relative to an approved baseline until
  `OD-06` sets absolute targets.

### Benefits

- Appendix D, §49.1 condition 6 and Appendix F agree with the Blueprint.
- Three of the six Blueprint-blocking decisions are closed; what still blocks
  sign-off is visible: `OD-03`, `OD-05`, `OD-06`, and `VG-01`.

### Follow-on work

- Close `OD-03`, `OD-05` and `OD-06` before Stage 1 sign-off (owner).
- Keep the dependency review cadence of Blueprint §10 operational; Dependabot
  and the weekly RustSec job (`docs/security/supply-chain.md`) already run more
  often than the monthly minimum.

## Verification

- Appendix D shows the four entries closed with a link to this ADR, and
  `scripts/check-docs.sh` passes (every register identifier and link resolves).
- Appendix F condition 2 reads 3 of 6 closed and condition 6 reads met.

## Revisit when

- `OD-05` establishes more than one deployment with different coexistence or
  downtime constraints (re-examine `OD-01` per deployment).
- `OD-06` sets absolute SLOs (re-examine `OD-04` thresholds).
- A capability outside the Blueprint §8.2 list is proposed for v1, or one on
  it is dropped (re-examine `OD-09`).
- A RustSec finding misses the §10 remediation target (re-examine `OD-02`).
