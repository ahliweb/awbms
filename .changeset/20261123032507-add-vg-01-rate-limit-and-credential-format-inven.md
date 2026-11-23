---
bump: minor
type: added
component: contracts
---

Add VG-01 rate-limit and credential-format inventories: the generator emits rate-limits.json, credential-formats.json and the cited AWCMS implementation files (source/awcms-impl/) with source-line citations, check:rate-limits and check:credential-formats validate them (gated by pendingArtifacts until the next vg01-freeze run), and the C-07 finding is recorded from the code (confirmed)
