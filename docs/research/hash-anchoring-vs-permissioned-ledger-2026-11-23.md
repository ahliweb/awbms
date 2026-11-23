# AWBMS — Business Hash Anchoring vs Permissioned Ledger (Alternatives Analysis)

## Document control
- **Version:** v1.0.0
- **Updated:** 2026-11-23
- **Issue:** [#37](https://github.com/ahliweb/awbms/issues/37) (F0/06)
- **Status:** RESEARCH / PROPOSED only. Not a production implementation issue; no consensus code; no approval to run a ledger node.
- **Parent note:** [Blockchain business backend and crypto-asset readiness](blockchain-crypto-business-backend-readiness-2026-11-23.md) (its §1 table and §8 option table are the starting point; not repeated).
- **Siblings:** [#35 regulation](blockchain-business-models-indonesia-regulation-2026-11-23.md), [#36 RPC/finality spec](chain-rpc-indexer-finality-reorg-spec-2026-11-23.md), [#38 custody boundary](crypto-custody-signing-ledger-safety-boundary-2026-11-23.md).
- **Source retrieval date:** 2026-11-23.
- **Claim labels:** `[F]` repository fact; `[C]` external claim with source; `[A]` assumption; `[R]` recommendation; `[O]` open decision; `[G]` verification gate.
- **Not executed:** no test in this document was run. No PoC exists.

## 1. Question

[R] Does a **signed PostgreSQL audit trail plus optional hash anchoring** meet real multi-party trust needs more efficiently than a permissioned consortium blockchain? Default answer under the evidence available: **yes, unless the trigger in §8 is met.** [A] This default is an assumption; no measured business case exists.

[F] AWBMS at `59cc2c3` contains no audit-signing, anchoring or ledger code. [A] The existing parent note's claim that an append-only PostgreSQL log is not a decentralized blockchain is retained: the options below differ mainly in **who must be trusted not to rewrite history**.

## 2. Options compared

| Code | Option | One-line description |
| --- | --- | --- |
| O-A | Signed PostgreSQL audit / WORM | Append-only table, per-row or batch signatures, hash chain, offsite write-once copy. Single operator. |
| O-B | Notarized hash anchor | O-A plus periodic Merkle root published to an external timestamp or notary service (could be a trusted third party, a transparency log, or a chain). |
| O-C | Hyperledger Fabric | Peers, ordering service, endorsement policies, channels, private data collections. |
| O-D | Public EVM metadata proof | O-A plus Merkle root written to a public EVM chain via a transaction. |

## 3. Criteria comparison

Ratings are qualitative and `[A]`. No benchmark was run. Where a source is cited it supports only the mechanism, not the rating.

| Criterion | O-A signed PostgreSQL | O-B notarized anchor | O-C Fabric | O-D public EVM proof |
| --- | --- | --- | --- | --- |
| Security (tamper evidence vs the operator) | Weak against a malicious operator holding signing key and DB; strong against outsiders. Needs external copy | Good: operator cannot backdate beyond last anchor | Strong against any single org if endorsement spans several; weak if one org controls most peers or orderer | Strong for timestamp and integrity of the root; says nothing about truth of content |
| Data governance | Operator-defined, simplest erasure story (delete source rows; keep only hash chain with care) | Same plus anchor immutability of the root only | Channel membership, per-org MSP identity, private data collections limit visibility; governance must be contractually defined | Fully public metadata; no access control on the anchor |
| Throughput and latency | Database speed; synchronous | Anchor cadence (minutes to hours) is a policy choice | Consensus and endorsement round trips add latency; throughput depends on topology and chaincode `[A]` | Bounded by chain block time and finality (see #36 for finality caveats) |
| Cost | Lowest | Low plus anchor fee or notary fee | Highest: multiple orgs run peers, an ordering service, CAs, upgrades | Per-anchor transaction fee, variable; plus key management for the anchoring account |
| Recovery | Backups, PITR, offsite copy; operator-owned | Same; anchors survive loss of operator | Requires every org to keep ledger copies; consortium recovery runbook; hard if an org disappears | Chain survives; local re-derivation from DB needed to produce proofs |
| Participant onboarding | Account creation | Account creation plus verifier tooling | Per-org CA, MSP, peer, channel join, policy updates: slowest | Verifier needs only chain access; onboarding of writers is a key |
| Legal and evidence value | Depends on court acceptance of operator records and signatures; `[O]` counsel | Stronger timestamp evidence; legal weight of anchors in Indonesia `[O]` UNVERIFIED | Shared record signed by multiple orgs may carry more evidentiary weight; `[O]` UNVERIFIED | Public timestamp; legal weight `[O]` UNVERIFIED |
| Maintenance | Existing stack | Existing stack plus anchor job | Specialist skills, upgrades, chaincode lifecycle | Chain RPC dependency, key custody for anchor account, fee monitoring |

[C] For O-C the mechanism claims (endorsement policies at chaincode, collection and key level; private data collections) are from the [Fabric endorsement policies documentation](https://hyperledger-fabric.readthedocs.io/en/latest/endorsement-policies.html) (read 2026-11-23). [C] Fabric ordering and peer roles are as described in the pages linked from the parent note's references (not re-read here).

[R] Reading the table: the **number of writers who do not trust each other** is the variable that decides between O-A/O-B and O-C. Everything else is secondary.

## 4. Synthetic off-chain hash proof design (no PII, nothing on chain)

[R] Design for a research PoC; all inputs synthetic.

1. **Canonical document bytes.** Define a canonicalization (for example a fixed JSON canonical form or the PDF bytes as stored) so the same logical document always hashes the same.
2. **Per-document leaf:** `leaf = H( 0x00 || domain_tag || tenant_salt || doc_digest )`, where `doc_digest = H(canonical_bytes)`.
   - `tenant_salt` is a high-entropy random value per tenant and per document kept **only in the off-chain database**. It prevents dictionary and linkage attacks on low-entropy documents (a hash of a short, guessable document is brute-forceable).
   - `domain_tag` binds the hash to this purpose and schema version, preventing cross-protocol reuse.
3. **Batch Merkle tree** over leaves with a distinct interior-node prefix (`0x01`) to avoid second-preimage ambiguity between leaves and nodes; the **root** and `batch_id`, `count`, `schema_version`, `issuer_key_id`, `timestamp` form the anchor record.
4. **Issuer signature** over the anchor record with an issuer key held in the secret store (never in code or logs).
5. **Anchor:** publish only the signed root (32 bytes) plus `batch_id` to the chosen anchor (notary service, transparency log or public EVM chain). **No document, no filename, no person identifier, no tenant name, no per-document hash goes on chain.**
6. **Proof package** handed to a verifier: the document (shared by the holder, off-chain), `tenant_salt` for that document, Merkle path, anchor record, issuer signature, and anchor location. The salt is disclosed only to the verifier the holder chooses.
7. **Independent verification:** recompute `doc_digest`, leaf, path to root, check issuer signature against a published key, check the root against the anchor, check the issuer key was valid at that time (key transparency or revocation list).

[R] Privacy properties to preserve: unlinkability across documents (distinct salts), no batch-size leakage beyond a policy (pad or batch by time window), and erasure by deleting the salt and source (the root remains but cannot be matched to anything: this is a design claim `[A]` to be reviewed by counsel under the PDP law, see [#35](blockchain-business-models-indonesia-regulation-2026-11-23.md)).

[R] Synthetic fixture only: random 1 KiB byte strings labelled `SYNTHETIC-DOC-0001`...; no real names, numbers or documents.

## 5. Multi-organization assumptions (for any option beyond O-A)

All `[A]` unless stated; each must be turned into a signed consortium agreement `[O]`.

| Topic | Assumption to validate |
| --- | --- |
| Agreement | A written consortium agreement exists before any node is deployed: purpose, data classes, liability, dispute process, fees, change control. Technology does not substitute for it. |
| Identity | Every participant is a legal entity with a verified identity; keys are bound to organizational identity (for Fabric, a per-org certificate authority and MSP), not to individuals only. |
| Endorsement | An explicit policy states which organizations must endorse which transaction types (for example 2 of 3 named orgs, matching the Fabric mechanism read in §3). No single org can endorse alone for records that affect others. |
| Ordering | Who runs the ordering service is a governance decision; if one party runs it alone, that party is a single trust point. `[O]` |
| Key management | Per-org rotation schedule, compromise procedure, and revocation propagation time are agreed; no shared keys. |
| Data visibility | Channels or private collections decided per data class; default is least visibility. |
| Participant exit | Exit procedure: revoke identity, archive the exiting org's view, retain evidentiary copies for the agreed retention period, define what happens to records only that org endorsed. A departing org must not be able to destroy the quorum. `[O]` |
| Admission | New-member onboarding requires a policy change (endorsement and channel membership) approved by the existing members. |
| Termination | Wind-down plan: who keeps the final ledger, where read-only archives live, for how long. |

## 6. What an immutable ledger cannot solve

| Limitation | Why | Mitigation to specify |
| --- | --- | --- |
| Erroneous source facts | Immutability preserves whatever was written, including wrong data ("garbage in, immutable garbage") | Source validation, attestation by accountable parties, correction records that reference the original |
| Compromised keys | A stolen signing key produces valid, permanent, false entries | Key rotation, revocation list, anomaly detection, hardware-backed keys, compensating entries and an incident process |
| Data erasure | Conflicts with rights to erase or rectify personal data | Keep personal data off chain; salted hashes; delete salts and sources; counsel review `[O]` |
| Contractual disputes | A ledger records that something was written, not who has legal right to the asset or whether a contract is enforceable | Contract law, arbitration clauses, legal evidence rules |
| Ownership rights | A record is not title | Registry or legal recognition outside the ledger |
| Oracle truth | The ledger cannot verify real-world events | Attestors, multi-party sign-off |
| Governance capture | A majority of endorsers can write false history for the consortium | Policy design, external audit |

[R] The parent note's statement that on-chain immutability conflicts with erasure rights is retained; this table is its expansion, not a second statement of the policy.

## 7. Proposed independent verification and negative tests (NOT EXECUTED)

| ID | Scenario | Expected |
| --- | --- | --- |
| V-01 | Independent verifier recomputes proof from document, salt, path and anchor | Accept |
| V-02 | Falsified certificate (one byte changed) | Reject at leaf or path |
| V-03 | Valid proof with substituted issuer key not in the published key set | Reject |
| V-04 | Key compromise: entries signed after the recorded revocation time | Reject and flag; entries before revocation accepted |
| V-05 | Unauthorized organization submits an entry (Fabric-like) | Rejected by endorsement policy |
| V-06 | Source-data correction | New correcting entry references the old; old remains; verifier shows lineage |
| V-07 | Salt deleted (erasure request) | Proof can no longer be produced; no personal data remains in anchor |
| V-08 | Leaf vs interior node confusion (second preimage attempt) | Reject due to domain prefixes |
| V-09 | Anchor reorg (public chain option) | Re-anchor after finality per [#36](chain-rpc-indexer-finality-reorg-spec-2026-11-23.md) policy |
| V-10 | Participant exit | Retained copies verify; remaining quorum functions |

All rows **NOT EXECUTED**; no tests are claimed.

## 8. Measurable business trigger and no-go

[R] **Go to a permissioned-ledger PoC only if all of the following are evidenced** (thresholds `[A]` to be set by the owner with data, not by this note):

1. At least **N independent organizations** (N at least 3 as a starting hypothesis) each need to write records.
2. **No existing party is accepted by the others as neutral operator** of a shared database, shown by a documented refusal or failed negotiation.
3. A neutral third-party notary or transparency log (O-B) was evaluated and rejected for a stated, evidenced reason.
4. The expected benefit exceeds the operating cost over a stated period: estimated cost of disputes or reconciliation that a shared ledger removes versus the cost of running peers, orderers and CAs in all organizations.
5. Counsel confirms the data class permits shared storage and the PDP position is acceptable.

[R] **No-go (stay on O-A or O-B)** when any of: a single organization owns the records; all counterparties already trust one provider; the data is personal and erasure or rectification is required; the only motivation is "innovation" or marketing; the expected consortium is fewer than three writers; or the metric in item 4 is not measurable.

[R] **Optional PoC proposal** (research only): implement §4 against synthetic documents in an isolated repository with a notary stub and a Fabric test network in containers; measure proof generation and verification time, anchor lag and operational steps. Do not claim results until measured.

## 9. Risk register (summary)

| ID | Risk | Mitigation |
| --- | --- | --- |
| K-1 | Hash of guessable data is linkable | Salts per document, batching, no per-document hashes on chain |
| K-2 | Operator rewrites DB history before anchoring | Short anchor cadence, offsite copy, third-party notary |
| K-3 | Consortium dissolves; ledger orphaned | Exit and archive clauses |
| K-4 | Anchoring key compromise | Hardware-backed key, rotation, revocation list, key transparency |
| K-5 | False sense of legal certainty | Counsel opinion on evidence value `[O]` |
| K-6 | Public-chain fee or outage blocks anchoring | Queue anchors; anchoring is not on the critical path |
| K-7 | Fabric operating complexity exceeds benefit | Trigger in §8; revisit annually |

**Rollback:** if no business case, delete this note's PoC proposal and record the concept as deferred. Nothing is deployed.

## Acceptance criteria trace

| Issue #37 acceptance criterion | Section | Status |
| --- | --- | --- |
| Compare at least five criteria (security, data governance, throughput/latency, cost, recovery, onboarding, legal/evidence value, maintenance) | §3 (all eight) | Done; ratings `[A]`, not measured |
| Sample off-chain synthetic document hash proof, no PII or source document on chain | §4 | Done as design; not implemented |
| Multi-organization agreement/endorsement/identity and participant exit assumptions | §5 | Done as assumptions |
| What immutable chain cannot solve: erroneous source facts, compromised keys, data erasure, contractual disputes | §6 | Done |
| Measurable business trigger and no-go when DB/audit suffices | §8 | Done; thresholds are `[A]` |
| Proposed independent verification, falsified certificate, key compromise, unauthorized org, source corrections; do not claim tests run | §7 | Proposed; NOT EXECUTED |
| DoD: comparison with trigger, risk register, optional PoC proposal | §8, §9 | Done |

## Changelog
- **v1.0.0 (2026-11-23):** Initial alternatives analysis for issue #37.
