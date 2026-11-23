# VG-01 Evidence — Legacy Baseline Delta (observed upstream movement)

**Date:** 2026-11-23  
**Issue:** `#7` (P0 item 3)  
**Tool:** `tools/vg01_baseline_delta.sh` with rules `tools/vg01_baseline_delta.rules`  
**Result:** observed upstream movement DOES touch contract, schema, security and consumer-surface paths. The tool exits `2` (alert). This is a report, not a gate failure of the frozen corpus.

## 1. Baseline status

**The frozen baseline was NOT refreshed.** `contracts/legacy/awcms/frozen/` is unchanged and `provenance.json` still pins the SHAs below. The tool is read-only and refuses to write inside `contracts/legacy`.

A baseline refresh requires all of the following, as an explicit decision:

1. an explicit decision recorded in the architecture validation register;
2. a provenance update (`provenance.json`, workflow pins, manifest SHAs);
3. regenerated hashes for the static and live corpus;
4. parity review of the changed contract, schema, security and consumer surface;
5. execution of the affected tests against the refreshed corpus.

## 2. Compared SHAs

| Repository | Frozen (baseline) | Observed (upstream `main`) | Ahead by | File list |
| --- | --- | --- | ---: | --- |
| `ahliweb/awcms` | `11f2e95a47b1328a820f976d60f978c38a067903` | `da4d3b194b910f36080c1d7d6282eeca4cd7765b` | 440 | shallow `git diff --name-only` (compare API file list reaches its 300-file cap) |
| `ahliweb/awcms-astro` | `7b753be619244541b817d5d8e7d3b72cfe88d4f9` | `c7b76aa41534dab6e56a8713d0a05166dbfafbf2` | 148 | GitHub compare API |

Observed SHAs were read from `main` of each repository on 2026-11-23 and passed to the tool, so the counts are reproducible:

```sh
tools/vg01_baseline_delta.sh --out <dir> \
  --awcms-head da4d3b194b910f36080c1d7d6282eeca4cd7765b \
  --astro-head c7b76aa41534dab6e56a8713d0a05166dbfafbf2
```

## 3. Changed paths per class

| Class | AWCMS | AWCMS-Astro |
| --- | ---: | ---: |
| API/OpenAPI/AsyncAPI contract (`api-contract`) | 424 | 0 |
| migration/schema/RLS/role (`migration-schema`) | 153 | 0 |
| authorization/security-invariant (`authorization-security`) | 83 | 0 |
| AWCMS-Astro consumer surface (`astro-consumer`) | 0 | 6 |
| behaviour/parity fixture (`behaviour-parity`) | 1591 | 18 |
| CI/tooling/docs-only (`ci-docs`) | 539 | 117 |
| unclassified | 59 | 36 |
| **Total changed paths** | **2849** | **177** |

Notes:

- Unclassified paths are UI assets, styles, components, design notes and load-test files. They are intentionally left for human classification rather than guessed.
- Files pinned by the frozen corpus that changed upstream: AWCMS `src/modules/index.ts`, `docs/awcms/repo-inventory.md`, `openapi/awcms-public-api.openapi.yaml`, `asyncapi/awcms-domain-events.asyncapi.yaml`, `package.json`, `.github/workflows/ci.yml`; AWCMS-Astro `tests/kontrak-awcms.test.mjs`, `package.json`. `scripts/api-consumer-contract.ts` (AWCMS) did not change.
- A path-level classification is a triage signal, not a semantic diff. It shows where review is needed, not that compatibility is broken.

## 4. Exit codes

| Code | Meaning |
| ---: | --- |
| 0 | nothing changed, or only CI/tooling/docs |
| 1 | tool error |
| 2 | alert: contract, schema, security or consumer-surface change |
| 3 | review: only behaviour/parity or unclassified changes |

## 5. Checks run

- `shellcheck tools/vg01_baseline_delta.sh` — clean.
- Two runs with identical head SHAs produced byte-identical Markdown and JSON.
- Running with the frozen SHAs as heads produced exit `0`.
- Pointing `--out` inside `contracts/legacy` is refused with exit `1`.
- The weekly workflow `.github/workflows/vg01-baseline-delta.yml` uses `permissions: contents: read`, SHA-pinned actions and no dependency installs.
