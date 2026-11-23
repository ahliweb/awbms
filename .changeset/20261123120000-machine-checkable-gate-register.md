---
bump: patch
type: changed
component: architecture-validation
---

Add machine-readable gate register `docs/architecture/gates.json` and a `check-docs.sh` section that fails on table/register drift, PASS gates without evidence or a recorded commit/run, and gates described as lacking automation while their executable check exists. Reconcile `VG-15` wording (PASS) and record the final-main run for `VG-03`/`VG-04`/`VG-15`. Issue #7.
