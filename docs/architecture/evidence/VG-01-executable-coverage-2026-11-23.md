# VG-01 Executable Coverage — 2026-11-23

**Gate:** `VG-01`  
**Status:** PARTIAL (unchanged by this note; the gate register is edited only by the orchestrator)  
**Issue:** `#3` (also `#7`, P0 item 4)

This note audits the acceptance criteria of `#3` against `main` at `59cc2c3`
plus the checks added in the same change, and states whether `VG-01` can move to
PASS. It adds no frozen artifact and regenerates nothing.

## What was added

| Artifact | Purpose |
|---|---|
| `contracts/legacy/awcms/fixture-usage.json` | Machine-readable map from every file under `frozen/` and `authorization/vectors.json` to the consumers that read it, with a `strength` per consumer (`content`, `structural`, `integrity`). |
| `tools/vg01_check_corpus.rs` | std-only Rust checker (`rustc --edition 2024 -D warnings`, no Cargo dependency) that reads the fixtures and fails on contradiction, orphan fixtures, dangling consumers and stale source SHAs. |
| `tools/vg01_verify_frozen.sh` | Now compiles and runs the checker last. CI already runs this script in the step "Verify frozen VG-01 evidence when present", so no workflow edit is needed. |

## Checks executed against the frozen corpus

| Check | What it reads | What would make it fail |
|---|---|---|
| `stale-source-sha` | `frozen/provenance.json`, both workflows, `tools/vg01_inventory.rs`, every fixture that records a source SHA | any SHA that disagrees with `provenance.json`; an AWCMS or AWCMS-Astro checkout not pinned to a SHA |
| `modules-registry` | frozen registry source (`baseModules` array and imports), `modules.json`, generated inventory | module count or names differ between registry source, `modules.json` and the inventory (one alias, `workflow` = `workflow-approval`, is explicit) |
| `migrations` | `migrations.json`, inventory migration table, live migration ledger | non-contiguous numbering, duplicate or malformed SHA-256, list or order differs from inventory or ledger |
| `tables-rls` | inventory table section, source and live TSV, `tables-rls.csv`, `policies.csv` | any table or RLS/FORCE state differs between source, live and catalog; RLS table without a policy; policy count mismatch; counts other than 152/134/134 |
| `static-vs-live` | the three drift files, live table names, summary | any non-empty drift file; the 152 vs 151 scope difference not being exactly the migration ledger table |
| `live-roles` | `roles.csv`, table/routine grants, table owners | SUPERUSER/BYPASSRLS/CREATEROLE/CREATEDB role; more than one table owner; runtime role owning tables; grant to unknown role or table |
| `live-summary` | `summary.json`, hand-maintained `manifest.json` summaries | any summary figure that differs from the figure derived from the underlying files |
| `route-test-counts` | inventory route and test tables, frozen OpenAPI | surface or directory sums differ from the summary rows; OpenAPI `/api/v1` path count differs from the `/api/v1/**` route-file count (308) |
| `astro-consumer` | `astro-consumer.json`, frozen consumer-contract source | consumed/committed path sets differ from the source; class distribution other than 7 / 2 / 1 / 3 |
| `openapi-security` | `astro-consumer.json`, frozen OpenAPI | an anonymous browser class whose operations are not all `security: []`; a machine-authenticated or committed path that is |
| `vectors` | `authorization/vectors.json`, live fixtures | duplicate or incomplete vector; `DB-RLS-001` count or migration count differing from live/inventory fixtures |
| `manifest-digests` | both `manifest.sha256`, static `manifest.json`, files on disk | a file on disk without a digest, a digest without a file, or `manifest.json` disagreeing with `manifest.sha256` |
| `source-headers` | OpenAPI/AsyncAPI headers, package versions, frozen CI, consumer test | OpenAPI not 3.1.0, AsyncAPI not 3.0.0, package versions not 10.1.0 / 0.4.0, lost `bun run check`, lost tenant-binding tests |
| `usage-map` | `fixture-usage.json`, tree under `frozen/` | file with no consumer; mapped file that does not exist; unknown consumer id; consumer whose file or marker does not exist; tool check that did not run |

Result with the current corpus: 31 mapped artifacts, 17 consumers, no
integrity-only artifact (every artifact has at least one structural or content
consumer).

## Authorization vectors: do executable vectors exist?

`authorization/vectors.json` exists and holds 14 vectors. Each has `id`,
`source`, `class`, `input` and `expected`, and the last two are **prose
sentences**, not machine-executable inputs. There is no parity harness in
AWBMS that runs them. 12 of the 14 cite test files in the AWCMS-Astro (5) and
AWCMS (7) repositories that are not part of the frozen corpus (the frozen
`awcms-astro-contract.test.mjs` is a different, AWCMS-side contract test); the
other 2 cite controlled database introspection. The checker
validates their structure and binds `DB-RLS-001` to the live fixtures; it does
not execute them.

## Acceptance criteria of `#3`

| # | Criterion | Verdict | Evidence |
|---|---|---|---|
| 1 | Inventory generation is re-runnable from pinned source inputs | MET | `tools/vg01_inventory.rs` verifies both checkout HEADs and fails closed; `verification.yml` builds it, runs it twice and `diff -ru`s the outputs. Not re-executed in this change (needs the pinned checkouts). |
| 2 | Every imported artifact has SHA-256 provenance | MET | Both `manifest.sha256` files verified by `sha256sum -c`; `check:manifest-digests` now also proves the digest list equals the files on disk. `authorization/vectors.json` is authored, not imported, and has no digest (finding below). |
| 3 | AWCMS and AWCMS-Astro SHAs are both recorded | MET | `provenance.json`, `static/manifest.json`, `manifest.json`, `vectors.json`, fixtures; `check:stale-source-sha` ties all of them to the workflow pins. |
| 4 | 24-module claim derived from registry source, not prose | MET | `check:modules-registry` parses the frozen `baseModules` array and imports (24) and compares to `modules.json` and the generated inventory. |
| 5 | Migration inventory derived from files/ledger semantics, not prose | MET, with a limit | List, numbering and per-file SHA-256 shape are cross-checked against the inventory and the live ledger (148). Per-file SHA-256 against the real SQL cannot be recomputed without the AWCMS checkout; that remains the generator's CI job. Ledger checksums (`sha256:...`) differ from the file SHA-256 values and the algorithm is not documented in the corpus, so they are format-checked only. |
| 6 | RLS/role evidence distinguishes source declaration from live introspection | MET | Separate `source-*` and `live-*` fixtures, empty drift files, `check:tables-rls`, `check:static-vs-live`, `check:live-roles`. The 152 vs 151 difference is asserted to be exactly the ledger table. |
| 7 | Consumer-contract fixture captures execution/security class distinctions | MET | `astro-consumer.json` has four consumed classes (7/2/1/3) plus the committed class; `check:openapi-security` shows the anonymous classes correspond to `security: []` in the frozen OpenAPI and the machine-authenticated class does not. |
| 8 | Gate inventory identifies enforced vs advisory vs absent checks | NOT MET | Only raw `awcms-ci.yml` and `awcms-package.json` are frozen. `manifest.json` records `source-frozen-pending-structured-inventory`. Producing it needs a read of the pinned AWCMS source and goes through the `vg01-freeze` publication workflow. |
| 9 | Authorization vectors are executable/frozen inputs for future parity tests | NOT MET | Frozen, but prose-only and with no harness (see section above). Machine-readable inputs and a Rust consumer need design and the cited test sources. |
| 10 | `VG-01` changed to PASS only after all required artifacts/checks exist | NOT MET | Criteria 8 and 9 are open, plus the two artifacts below. |

Required artifacts from the gate's 14-item set that are still absent:

- **Route inventory with owner/auth classification (item 7).** Only counts exist
  (308 `/api/v1/**`, 50 `/admin/**`, 30 public). The OpenAPI gives per-path
  `security` data for `/api/v1` but no owning module, and `manifest.json` says
  `pending-structured-inventory`.
- **Representative compatibility fixtures (item 13).** `manifest.json` says
  `pending`; nothing in the corpus is a request/response or behavior fixture.

Issue `#3` test/evidence list: generator determinism (CI diff), SHA-256
verification, stale-source-SHA detection, orphan fixture detection and
inventory count consistency are now covered. "Manifest schema/required fields"
is only partly covered (digest manifests and the usage map are validated; the
hand-written `manifest.json` has no schema beyond the summary cross-check).

## Can VG-01 move to PASS?

No. The pass rule requires every required artifact to exist and be consumed by a
deterministic check or parity test. This change closes the "artifact-to-test
usage mapping" and static/live consistency items, but the structured gate
inventory, the route ownership/auth inventory, representative behavior
fixtures and executable authorization vectors do not exist. None of them can be
produced without reading AWCMS, so they must be added through the `vg01-freeze`
workflow or a separate reviewed import, not here.

## Verification performed

- `bash tools/vg01_verify_frozen.sh contracts/legacy/awcms` passes (14 checks
  plus the existing `sha256sum -c` and grep assertions).
- Negative runs: an unmapped dummy file under `frozen/static/` fails
  `check:usage-map` (and `check:manifest-digests`); a mapped consumer id that
  does not exist fails; a mapped marker that is not in the consumer file fails;
  a workflow checkout pinned to a different SHA fails `check:stale-source-sha`.
  Each was reverted afterwards.
- `rustc --edition 2024 -D warnings --test tools/vg01_check_corpus.rs` unit
  tests pass (JSON, CSV and SHA-shape helpers).
- `scripts/check-docs.sh` passes.

## Findings outside this change

- `contracts/legacy/awcms/README.md` still lists as "NOT yet proven" several
  items that now exist (generator, per-file migration SHA-256, live
  introspection, OpenAPI/AsyncAPI copies, usage checks), and its planned
  directory shape does not match `frozen/`.
- `contracts/legacy/awcms/manifest.json` has `status: partial-frozen-evidence`
  and says the artifact usage check was implemented; that is only true from
  this change onward.
- The checker's unit tests are not run in CI; adding them needs a
  `verification.yml` edit.
- Regenerating the frozen corpus through `vg01-freeze` will fail
  `check:usage-map` if the set of frozen files changes until
  `fixture-usage.json` is updated. That is intended.
