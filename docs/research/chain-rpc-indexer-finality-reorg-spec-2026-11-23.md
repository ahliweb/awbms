# AWBMS — Read-Only EVM RPC Indexer: Finality and Reorg Spike Specification

## Document control
- **Version:** v1.0.0
- **Updated:** 2026-11-23
- **Issue:** [#36](https://github.com/ahliweb/awbms/issues/36) (F0/05)
- **Status:** SPEC / PROPOSED for a future isolated proof of concept. **No code, no runtime module, no change to accepted OpenAPI/AsyncAPI or migrations.**
- **Parent note:** [Blockchain business backend and crypto-asset readiness](blockchain-crypto-business-backend-readiness-2026-11-23.md) (§4 state distinctions and §9 Phase 1 are the origin; this document specifies them, it does not repeat the trust-boundary diagram).
- **Siblings:** [#35 regulation](blockchain-business-models-indonesia-regulation-2026-11-23.md), [#37 anchoring vs ledger](hash-anchoring-vs-permissioned-ledger-2026-11-23.md), [#38 custody boundary](crypto-custody-signing-ledger-safety-boundary-2026-11-23.md).
- **Source retrieval date:** 2026-11-23.
- **Claim labels:** `[F]` repository fact; `[C]` external claim with source; `[A]` assumption; `[R]` recommendation; `[O]` open decision; `[G]` verification gate.
- **Not executed:** every test in §10 is **NOT EXECUTED**. No node, mock server, or Rust code was run.

## 1. Scope

[R] A deterministic, read-only integration of an Ethereum-compatible JSON-RPC endpoint that can detect duplicate events, wrong chain, reorgs and stale node data, **without ever financially crediting a customer**. Testnet or synthetic data only. No private keys. No transfer submission (`eth_sendRawTransaction` is out of scope and MUST be blocked at the port). Hyperledger Fabric is comparison-only (see #37).

[F] At `59cc2c3` AWBMS contains no RPC adapter, indexer, outbox implementation or API for this (see parent §2). Everything below is a proposal.

[R] Coordination with [#23](https://github.com/ahliweb/awbms/issues/23) (generic contract-first foundation): the contracts here are **proposals** inside this document and MUST NOT be committed to the accepted OpenAPI/AsyncAPI files from this issue. Any promotion goes through #23's process.

## 2. Primary sources and what they support

| Source (retrieved 2026-11-23) | Status | Supports |
| --- | --- | --- |
| [Ethereum JSON-RPC API](https://ethereum.org/developers/docs/apis/json-rpc/) | Read directly | [C] Block parameter accepts `latest` ("latest proposed block"), `safe` ("latest safe head block"), `finalized` ("latest finalized block"), `pending`, `earliest` or a hex number. `eth_chainId` "returns the chain ID used for signing replay-protected transactions". `eth_getLogs` accepts `fromBlock`, `toBlock`, `address`, `topics`, `blockHash`; `fromBlock`/`toBlock` default to `latest`. |
| [Ethereum Gasper / finality](https://ethereum.org/developers/docs/consensus-mechanisms/pos/gasper/) | Read directly | [C] A block is finalized after being justified and then having another justified block built on it; finalized blocks cannot be reverted unless an attacker destroys at least one third of total stake; justified blocks are unlikely but not impossible to revert. The page states no time-to-finality figure and no definition of "safe head". |
| L2/rollup, other EVM chains finality | **UNVERIFIED** | Not fetched. Policies for them in §4 are `[O]`. |
| Exact client behaviour of `safe` on non-Ethereum chains | **UNVERIFIED** | The block tags are part of the Ethereum API; other chains may not implement them, or may map them differently. |

[A] The meaning of `safe` is taken only as the page words it. Any stronger interpretation (for example "justified checkpoint") is an assumption to be verified against the execution API specification and the client in use.

## 3. Data states

[R] States are per `(chainId, blockHash, txHash, logIndex)` observation and are **monotonic only forward except via an explicit `reorged` transition**.

| State | Definition | Credit allowed | Source of truth |
| --- | --- | --- | --- |
| `observed` | Log seen at head (`latest`) from a provider; not yet checked against other data | **Never** | Provider response (untrusted) |
| `included` | Log belongs to a block whose hash and parent chain were verified against stored headers and the pinned `chainId` | **Never** | Local header store |
| `safe` | Block number is at or below the provider's `safe` block and the block hash still matches | Only if network policy says so (§4); default **no** | Provider `safe` tag + local re-check |
| `finalized` | Block number is at or below the provider's `finalized` block and hash still matches (and, where policy requires, a second independent provider agrees) | Eligible for a **business projection**, not automatically a financial credit | Provider `finalized` tag + second source |
| `reconciled` | Finalized event matches the authoritative PostgreSQL record (expected deposit or intent) or an operator-reviewed exception is closed | The only state from which accounting may be triggered, and only by a separate ledger spec (#38) | PostgreSQL + human review |
| `reorged` | Previously `observed`, `included` or `safe` event whose block hash is no longer canonical | Retract projection; no financial effect ever occurred | Local header store |
| `rejected` | Failed a fail-closed rule (§7) | None | Audit log |

[R] A broadcast receipt, an `eth_getTransactionReceipt` result, or event `observed` is never evidence of irreversibility (parent note §4 and issue risk text).

## 4. Network-specific finality policy (proposal)

[R] Policy is a **configuration record per `chainId`**, not code. Default is deny: an unlisted chain is rejected.

| Network class | `included` | `safe` | `finalized` | Notes |
| --- | --- | --- | --- | --- |
| Ethereum mainnet or its public testnets | Block hash verified, depth N_inc | Use provider `safe` tag | Use provider `finalized` tag; require second provider agreement for any business projection | [C] per §2 sources. Mainnet is not used in this spike: **testnet or synthetic only**. |
| L2 / rollups (optimistic or ZK) | Sequencer-confirmed block | `[O]` | `[O]` finality on L1 data availability and challenge or proof windows differ by rollup | **UNVERIFIED**; no assumption that L2 `finalized` equals L1 finality. |
| Other EVM-compatible L1 | Per chain | `[O]` tag may not exist | `[O]` fixed-depth confirmations are weaker than a protocol finality gadget | **UNVERIFIED**; document the chain's own finality rule before whitelisting. |
| Permissioned EVM (e.g. private consortium) | Per consensus rules | `[O]` | `[O]` | Out of scope except as a fixture. |

[R] Depth values (N_inc, any confirmation count) are `[A]` unvalidated defaults to be set by the product owner with evidence; this spec does not choose numbers.

## 5. Idempotency key and cursor

[R] **Event identity:** `event_key = (chainId, blockHash, txHash, logIndex)`. Block number alone is never part of identity because a reorg can reuse it. The same `event_key` seen twice is the same event.

[R] **Logical projection key** (for business meaning): `(chainId, txHash, logIndex)` bound to a single canonical `blockHash` at a time; when the canonical hash changes, the old `event_key` becomes `reorged` and the new one is a new observation of the same logical event only if `txHash` and `logIndex` match the new block's receipt and log data.

[R] **Cursor** per `(tenant_scope_of_source, chainId, source_id)`: `{blockNumber, blockHash, parentHash}` of the last fully processed block, stored with a monotonically increasing `cursor_version`. A cursor advance is a single PostgreSQL transaction together with its projection writes and outbox rows. On startup the indexer verifies the stored `blockHash` is still canonical at `blockNumber`; if not, it walks back to the common ancestor within the bounded window (§6).

[R] **Duplicate handling:** the projection table has a unique constraint on `event_key`; the second insert is a no-op that records a duplicate counter, so **two identical notifications yield one logical projection**.

## 6. Bounded reindex and reorg handling

1. [R] Maintain a header ring buffer of the last `W` blocks (`W` is a configured bound `[A]`).
2. [R] On every new head, verify `parentHash` continuity with the stored head. A mismatch means reorg: walk back to the common ancestor, mark events in abandoned blocks `reorged`, emit compensating outbox messages (`event.retracted`), then re-scan forward.
3. [R] If the common ancestor is deeper than `W`, **stop and fail closed**: mark the source `halted`, raise an alert, require operator action. The indexer never silently rewrites history deeper than its bound.
4. [R] `eth_getLogs` is requested in block-hash-addressed or bounded-range queries (`blockHash` filter is supported per §2), with a maximum range and maximum result count; an over-large response is rejected rather than truncated.
5. [R] Replay from a checkpoint is deterministic: the same fixture sequence always produces the same projection and outbox rows (golden-file check).

## 7. Fail-closed rules

| Condition | Required behaviour | Test ID |
| --- | --- | --- |
| `eth_chainId` differs from pinned value, at startup or in any later response | Reject the provider, halt source, alert; never ingest | F-01 |
| Head inconsistent: `parentHash` break not explained by a valid reorg, block number going backwards beyond `W`, header hash not matching recomputed content, or two providers disagreeing on a finalized block | Halt, no projection changes beyond retraction already proven | F-02 |
| Timeout | Bounded retry with jitter and circuit breaker; while open, status `degraded`, no new events ingested, read API marks data stale | F-03 |
| Rate limit (HTTP 429 or provider error code) | Backoff honouring limits; circuit breaker; no fallback to an unpinned provider | F-04 |
| Malicious response: oversized body, deeply nested JSON, wrong types, hex of illegal length or leading zeros in quantities, duplicate JSON keys, `logIndex` collisions inside one block, address checksum or topic length mismatch, log whose `blockHash` differs from the requested | Parser with hard byte/depth/array bounds rejects and records `rejected`; no partial ingestion of that response | F-05 |
| Provider returns `finalized` number lower than a previously stored finalized number | Treat as inconsistent provider; halt | F-06 |
| Plain-HTTP endpoint, certificate failure, or endpoint not on allowlist | Refuse to connect (TLS required, endpoint pinned) | F-07 |
| Method not on read-only allowlist (anything that sends or signs) | Port rejects before the network call | F-08 |

[R] The allowlist of RPC methods in the spike is limited to read calls (for example `eth_chainId`, `eth_blockNumber`, `eth_getBlockByNumber`, `eth_getBlockByHash`, `eth_getLogs`, `eth_getTransactionReceipt`). No private key material exists in this component at all.

## 8. Typed event normalization (fixture schema proposal)

[R] Fixtures are JSON files under a future isolated PoC repository, not in AWBMS contracts. Proposed shape (synthetic values only):

~~~json
{
  "fixture_version": 1,
  "chain": { "chainId": "0x539", "policy": "testnet-synthetic" },
  "blocks": [
    { "number": "0x10", "hash": "0xaaaa...", "parentHash": "0x0f0f...", "logs": [
      { "txHash": "0xt1...", "logIndex": "0x0", "address": "0xSYNTH-CONTRACT", "topics": ["0xTRANSFER-SIG", "0xFROM", "0xTO"], "data": "0x..." }
    ] }
  ],
  "tags": { "latest": "0x10", "safe": "0x0e", "finalized": "0x0c" },
  "script": [ "advance_head", "duplicate_notification", "reorg_to:alt_branch_1" ],
  "expect": { "projections": 1, "retractions": 0 }
}
~~~

[R] The normalized internal event has: `event_key`, `chainId`, `blockNumber`, `blockHash`, `txHash`, `logIndex`, `contract`, `event_type` (from an allowlist of ABI signatures), integer base-unit amounts as decimal strings (never floating point), `state`, `observed_at`, `source_id`. Unknown event signatures are stored as `unclassified` and never projected.

## 9. Outbox and read-only API contracts (proposals)

[A] Neither contract exists and neither modifies the accepted OpenAPI/AsyncAPI. They are inputs to #23.

**Idempotent outbox message** (proposed AsyncAPI-style subject `chain.event.state-changed`):

| Field | Meaning |
| --- | --- |
| `message_id` | Deterministic: hash of `event_key` + `state` + `cursor_version`; same transition produces the same id |
| `event_key` | As §5 |
| `from_state`, `to_state` | Allowed transitions only, see §3 |
| `tenant_scope` | Tenant of the subscription, never inferred from chain data |
| `occurred_at` | Indexer clock; not authoritative for finality |

[R] Delivery is at-least-once; consumers dedupe on `message_id`. Outbox rows are written in the same PostgreSQL transaction as the projection change. PostgreSQL cannot commit atomically with a chain; the outbox exists precisely because of that (parent note §2).

**Read-only API** (proposed, not accepted): `GET /chain-events?chainId=&state=&cursor=` and `GET /chain-sources/{id}/status`. Responses carry `state`, `finality_policy_id`, `data_age_seconds`, and a `stale` flag. There is **no** write endpoint and no endpoint that accepts a transaction or signature. Pagination cursor is opaque and tenant-bound.

## 10. Authority, tenancy and RLS requirements

- [R] **PostgreSQL is the sole authority** for business state, tenant mapping and any later accounting. Chain data is evidence; it never overrides the database, and a node answer alone never changes a balance.
- [R] Every table holding projections, cursors, outbox and sources has a `tenant_id` (or an explicit global scope marker for public-chain data) with `ENABLE` and `FORCE ROW LEVEL SECURITY`, and the application connects as a non-superuser, non-owner role.
- [F] The repository's existing verification spike exercises FORCE RLS in a narrow test (parent note §1); it does not cover these tables.

Negative tests required (all **NOT EXECUTED**):

| ID | Case | Expected |
| --- | --- | --- |
| R-01 | Tenant A role selects tenant B projections | Zero rows |
| R-02 | Tenant A inserts a projection with tenant B id | Rejected by policy |
| R-03 | Table owner role without BYPASSRLS reads other tenant with FORCE RLS | Zero rows (FORCE applies to owner) |
| R-04 | Session with no tenant context set | Zero rows, not an error leak |
| R-05 | Outbox consumer for tenant A sees tenant B message | Never |
| R-06 | Read API cursor from tenant A used by tenant B | Rejected |
| R-07 | Indexer worker role attempts to update cursor of another source it does not own | Rejected |

## 11. Mock-node test matrix (NOT EXECUTED)

| ID | Scenario | Mock behaviour | Expected result | Status |
| --- | --- | --- | --- | --- |
| T-01 | Happy path | Linear chain, tags advance | Events progress observed -> included -> safe -> finalized | NOT EXECUTED |
| T-02 | Duplicate delivery | Same log delivered twice, also via two polls | One projection, duplicate counter 1 | NOT EXECUTED |
| T-03 | Synthetic reorg depth 1 | Head replaced by alt branch containing the event | Event `reorged`, retraction outbox, re-observed if present in new branch | NOT EXECUTED |
| T-04 | Reorg deeper than `W` | Alt branch diverges below window | Source halted, alert | NOT EXECUTED |
| T-05 | Reorg after `safe` but before `finalized` | Mock violates safe | Retraction; incident recorded as provider inconsistency | NOT EXECUTED |
| T-06 | Wrong chainId | `0x1` instead of `0x539` | F-01 | NOT EXECUTED |
| T-07 | Inconsistent head | `parentHash` mismatch | F-02 | NOT EXECUTED |
| T-08 | Finalized goes backwards | Lower finalized number | F-06 | NOT EXECUTED |
| T-09 | Timeout | No response | F-03 | NOT EXECUTED |
| T-10 | Rate limit | 429 then recovery | F-04, resume without gap or duplicate | NOT EXECUTED |
| T-11 | Malicious payloads | Oversized, nested, wrong types, hostile hex | F-05 | NOT EXECUTED |
| T-12 | Node outage and restart | Kill mock mid-block | Cursor resumes, no loss, no double projection | NOT EXECUTED |
| T-13 | Replay determinism | Run same script twice | Byte-identical projection and outbox rows | NOT EXECUTED |
| T-14 | Unknown event signature | Unlisted topic | `unclassified`, no projection | NOT EXECUTED |
| T-15 | Disallowed method | Request to a send method | F-08 | NOT EXECUTED |
| T-16 | Amount parsing | Value above 2^256-1 or non-integer string | Rejected; no float anywhere | NOT EXECUTED |

[G] Gate: this matrix is executed in an isolated repository after the product Stage-1 and definition-of-ready authorization; results recorded with commit hash.

## 12. Security and privacy notes

[R] Provider isolation: each provider is a separate source with its own credentials held in the secret store; TLS required; endpoints pinned; circuit breakers; parser resource bounds (bytes, depth, array length, time); anti-DoS on the read API; no private keys; synthetic or testnet only; no PII in fixtures or logs. Wallet addresses are treated as potentially personal data (see #35).

## 13. Risks and rollback

- [A] Finality semantics vary across L1, L2 and clients; the table in §4 leaves L2 policy open on purpose.
- [R] Two independent providers raise cost and operational burden; whether justified is an owner decision `[O]`.
- **Rollback:** delete this document; nothing is deployed.

## Acceptance criteria trace

| Issue #36 acceptance criterion | Section | Status |
| --- | --- | --- |
| Distinguish observed, included, safe/finalized, reconciled states and network-specific policies | §2, §3, §4 | Specified; L2 policy `[O]`/UNVERIFIED |
| Duplicate messages yield one projection; synthetic reorg rolls back a provisional event | §5, §6, tests T-02, T-03 | Specified; NOT EXECUTED |
| Wrong chainId, inconsistent head, timeout/rate limit, malicious RPC fail closed | §7, tests T-06 to T-11 | Specified; NOT EXECUTED |
| Idempotent outbox and independent read-only API contracts, no runtime module yet | §9, §1 | Proposal only; no accepted contract changed |
| Off-chain PostgreSQL authority and cross-tenant FORCE RLS negative tests | §10 (R-01 to R-07) | Specified; NOT EXECUTED |
| Fixture/event schema proposal; no change to accepted OpenAPI/AsyncAPI or migration | §8, §9 | Done as proposal |
| Mock node test matrix, fake reorg, duplicate, outage; NOT EXECUTED | §11 | Done; all NOT EXECUTED |

## Changelog
- **v1.0.0 (2026-11-23):** Initial spike specification for issue #36.
