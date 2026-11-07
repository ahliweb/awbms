---
bump: patch
type: changed
component: dependencies
---

Run `cargo update` within semver (54 crates, Rust toolchain 1.98.0 unchanged) and bump `actions/upload-artifact` to v7.0.2. PostgreSQL 18.6 was already pinned. The Bun 1.3.14 pin is intentionally kept: it belongs to the frozen VG-01 provenance (`contracts/legacy/awcms/frozen/provenance.json`), not to the AWBMS toolchain.
