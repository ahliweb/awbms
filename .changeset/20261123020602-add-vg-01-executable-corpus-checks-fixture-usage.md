---
bump: minor
type: added
component: contracts
---

Add VG-01 executable corpus checks, fixture-usage map and coverage evidence note

Add a std-only Rust corpus checker, a fixture-usage map (orphan and dangling-consumer detection), stale-source-SHA detection and a per-criterion MET/NOT MET evidence note for VG-01 (#3). VG-01 stays PARTIAL: gate inventory, route ownership/auth inventory, behavior fixtures and executable authorization vectors are still absent.
