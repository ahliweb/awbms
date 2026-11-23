# Final-main verification record: VG-03, VG-04, VG-15

**Date:** 2026-11-23  
**Issue:** `#7` (P0 items 2 and 5)

The stacked pull requests `#2`, `#5` and `#6` were merged, so a green stacked branch is no longer the evidence of record. This file records the verification run on `main` after the merges, as the machine-readable register `docs/architecture/gates.json` requires for every PASS gate.

## Run of record

- Branch: `main` (push event)
- Commit: `59cc2c3ac5705cb8e50476249aad82026569de57`
- Workflow: `AWBMS verification`, run `45097211873`
- Job `VG-03 / VG-04 Rust + PostgreSQL verification` (`137439576505`): success

Steps observed as successful in that job: `Verify VG-15 repository independence`, `Clippy`, `Unit and PostgreSQL integration tests`, `Lockfile remains unchanged`. Retrieve with `gh run view 45097211873 --json jobs`.

## Per-gate disposition

- `VG-03`: the final-main run builds with the committed `Cargo.lock` and leaves it unchanged. Status unchanged (PASS); the earlier PR-branch evidence in `VG-03-VG-04-2026-08-29.md` is now backed by a post-merge main run.
- `VG-04`: the PostgreSQL integration tests (`crates/verification/tests/rls.rs`) passed on main. Status unchanged (PASS), with the narrowness stated in the architecture validation.
- `VG-15`: the pass rule (no AWCMS source/runtime dependency, submodule, path dependency or production filesystem coupling) is checked by `tools/vg15_independence.rs`, run in CI by the step named above, and independently by `scripts/check-docs.sh`. The step passed on main. The gates document previously described this gate as lacking automation although the check already existed; it is recorded as PASS here. It is a standing condition and can regress on any commit.

## Not covered

`VG-01` is not touched by this record and keeps its own evidence files.
