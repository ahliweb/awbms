---
bump: minor
type: added
component: contracts
---

Extend the VG-01 generator and checker with gate, route and fixture inventories, structured vectors and a migration SQL check

Add `gates.json`, `routes.json`, `behaviour-fixtures.json` and cited test copies to the `tools/vg01_inventory.rs` output, matching validators in `tools/vg01_check_corpus.rs` (gated by `pendingArtifacts` in `fixture-usage.json` until the `vg01-freeze` run), structured fields on every authorization vector, `tools/vg01_verify_migrations.sh` (per-file migration SHA-256 against the pinned SQL, run in `verification.yml`) and `tools/vg01_check_generated.sh`. Nothing is frozen by this change and VG-01 stays PARTIAL: request/response bodies and a vector-executing harness are still absent.
