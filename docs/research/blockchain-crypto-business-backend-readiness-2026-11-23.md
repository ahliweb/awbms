# AWBMS — Blockchain Business Backend and Crypto-Asset Readiness Research

## Document control
- **Version:** v1.0.0
- **Updated:** 2026-11-23
- **Status:** RESEARCH / PROPOSED only; no approval to handle crypto assets, hold keys, launch a chain, operate an exchange or deploy a node.
- **Evidence snapshot:** AWBMS `main` at `e6bb5638e0628418079b1c1d6fcf0a123c5bb4cc`; primary official standards/regulator documentation reviewed 2026-11-23.
- **Document Changeset:** Assess AWBMS as an independent Rust-native backend for business blockchain integration and regulated crypto-asset infrastructure, with separate custody/legal gates.
- **Boundary:** no product implementation before its 18-step lifecycle/DoR; no transfer of AWBMS ownership, existing decisions or gates.

## 1. Executive recommendation

**[R] Prefer an off-chain authoritative business backend on AWBMS/Linux/PostgreSQL, optionally connected to external permissioned or public chains through typed adapters.** Do not build a new chain, validator, trading engine, wallet custodian or token launch inside AWBMS's core Stage 1.

[**F, repository**] AWBMS is still specification + verification spike, not a production backend. `Cargo.toml` specifies Rust 2024/1.98.0, Tokio/Axum/Tower/SQLx, PostgreSQL and a narrow FORCE RLS test. No crypto custody, on-chain RPC adapters, blockchain consensus, chain indexer, smart contracts, wallet infrastructure or domain-specific financial ledger were found in inspected implementation paths/search. Business logic, network security, accounting, audits and recovery remain planned.

**Do not conflate these products:**

| Product type | Technical scope | Regulatory/business risk | Recommendation |
| --- | --- | --- | --- |
| Internal hash anchoring / document provenance | Hash + timestamp metadata, optional public on-chain proof | Low-to-moderate; document privacy and immutable-hash rules | First candidate research |
| Multi-organization permissioned consortium | Identity, endorsement, shared ledger + permissioned node network | Governance, contractual, operational | Conditional PoC if multiple independent writers |
| EVM public-chain **read-only** indexer | External RPC, chain state, receipts, finality/reorg handling | Availability/data integrity; privacy | Good adapter PoC |
| Enterprise wallet connector, **non-custodial** | User/device signs outside backend; backend orchestrates only authorized requests | Fraud, transaction reversal, AML/legal role classification | Later, with threat model |
| Custodial wallet / signing / treasury | Key generation/storage, HSM/MPC, withdrawal, settlement | High: asset loss, consumer liability, licensing | Defer; dedicated regulated provider/product |
| Crypto exchange / broker / market operator | Matching engine, custody/settlement, compliance/reporting | Very high; OJK licensing and oversight | Not Stage-1 AWBMS |
| Layer-1/L2 blockchain, full node or validator | Consensus/networking, state sync, governance, slashing/upgrade | Very high complexity and capital/ops | Separate product & architecture, not AWBMS core |

**[R]** A backend with a PostgreSQL append-only audit log or hash chain is *not* a decentralized blockchain. A blockchain should be justified only when distinct parties require a distributed trust/consensus model that cannot be satisfied by simpler databases plus signed evidence.

## 2. Why the current Rust stack is useful but insufficient

- Rust memory-safety and static types are favorable for decoding on-chain events, typed payloads, validation and concurrency; they do **not** prove funds safety or smart contract correctness.
- Axum/Tokio can become authenticated API and asynchronous workers *after* AWBMS module/quality gates pass; no current service endpoint is a crypto product.
- PostgreSQL with FORCE RLS is suitable for customer/business metadata, balances **only if a separate correct accounting model is designed**. RLS alone cannot prove double-entry consistency, nonnegative balances, consensus or fiat/crypto reconciliation.
- SQLx/Postgres and outbox support reliable state orchestration but transactions cannot atomically commit with a public blockchain. Design saga/settlement and compensation, no claim of distributed ACID across chains.
- OpenAPI/AsyncAPI (already proposed by existing [issue #23](https://github.com/ahliweb/awbms/issues/23)) should specify typed requests and outbox transitions; avoid a duplicate generic contract issue.
- Hardware signing, HSM/MPC, wallet recovery, exchange licensing, fraud controls and cross-chain consensus are **not implemented** by `crates/verification`.

Evidence basis: [AWBMS README](../../README.md), [architecture validation](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md), [Stage-1 gates](../architecture/AWBMS-STAGE-1-GATES.md).

## 3. Trust-boundary architecture

~~~mermaid
flowchart TD
  U[Business clients / ERP / ARKA persona] --> G[Authorized product gateway: Rust AWBMS candidate]
  G --> DB[PostgreSQL system of record; FORCE RLS; domain ledger]
  G --> AUD[Signed audit/provenance and transactional outbox]
  AUD --> PORT[Versioned blockchain adapter port]
  PORT --> NODE[Independent permissioned/public node or approved provider RPC]
  NODE --> CHAIN[Blockchain network: consensus/finality]
  PORT --> OBS[Chain observer/indexer + reconciliation worker]
  OBS --> DB
  G --> APPR[Human approval, AML/legal gating for regulated use]
  APPR -. optional separate custody operator .-> HSM[Licensed/custody provider + HSM/MPC service]
~~~

Summary: AWBMS is an off-chain control plane candidate, not a consensus engine. Chain nodes, customer asset custody and signing remain independent, qualified trust domains. A derived business product owns its backend, database, migrations, domain, backup, secrets and UAT; a template/reference repo is never used as common production backend.

## 4. On-chain state is not the database transaction

Mandatory distinctions before any user-facing service:
- **Transaction ID/intent**: local immutable ID, actor, consent and business scope.
- **Proposed**: request validated but not signed, broadcast or settled.
- **Signed**: proof of authorization for exact chain ID, destination, amount, fee limit, nonce, contract call; signing service is externally controlled.
- **Submitted**: node/provider accepted a raw transaction; *not* on-chain finality.
- **Included**: block observed, with chain ID, block hash/height, tx hash, log index.
- **Finalized**: confirmation policy satisfied on that chain/rollup, with reorg/finality rules and provider-independent verification.
- **Reorged/failed/expired**: reverse or compensate *off-chain projection* safely; no double credit/debit.
- **Reconciled**: node data and authoritative account/transaction records agree; exceptions queued for human review.

Operational invariants:
1. Chain-specific idempotency, replay protection, `chainId`, `nonce`, signed-payload digest, sequence and concurrency lock; sign exactly the reviewed payload. Address validation must include network/token and anti-address-poisoning policies.
2. A `200 OK` from RPC, broadcast receipt, or event `observed` is never evidence of finality. Two independent RPC sources where justified; bounded reorg window and deterministic reindexing.
3. Integer base units only; no binary floating point for money/coins. Decimal representation must have asset-specific scale, rounding, fee/currency rule and explicit handling of dust.
4. Distinguish financial accounting ledger from blockchain ledger and operational audit log. Double-entry, segregation, reconciliation, reserve/treasury controls need independent domain specification and audit.
5. No secret seed phrase, private key, PIN, wallet recovery material or signing key is delivered to an AI agent, browser bundle, log, prompt, CI or ordinary business database.
6. All mutating operations have default-deny RBAC+ABAC, separation of duties, step-up for high-risk action, signed approval binding, spending limits and kill switch; worker uses least privilege.
7. User consent and applicable law determine what goes on chain; on-chain immutability conflicts with rights to erase/rectify sensitive data. Put **no PII, patient data, client secret, or plaintext contracts on chain**; consider that hashes of personal data can still create privacy risk when linkable.
8. Public blockchain fees, chain congestion, rollup downtime, provider outage, block reorganizations and bridge exploits must be failure-tested.

## 5. Five detailed business use cases

| Use case | Where blockchain could help | When PostgreSQL alone is preferable | Policy |
| --- | --- | --- | --- |
| Supply-chain provenance for retailers | Independent supplier/carrier/buyer attestations and verifiable events | One company owns all records and audit access | Start signed off-chain records; hash anchor optionally |
| Document/certificate authenticity | Public timestamp/proof hash, third-party verifier | Internal records with centralized authority | Never post private document/PII |
| Consortium inter-company workflow | Multiple independent writers with governance/endorsement | All counterparties trust an existing service provider | Permissioned PoC only after governance ADR |
| Product loyalty/rewards | Transferability across independent merchants under justified rules | Conventional points/ledger under a single merchant | Analyze classification; don't assume token exempt from regulation |
| Asset/tokenization for business financing | Conditional representation/settlement with legally recognized claims | Simple business asset register or invoice database | **Defer** until counsel + OJK and relevant authority classification |
| Crypto analytics/portfolio dashboard | Public chain read-only ownership/transaction state | Pure internal holdings data only | Read-only PoC, no trading/withdrawal |

There is no measured revenue, ROI, scale or usability evidence for any case. Test value against a simpler non-blockchain implementation before funding production work.

## 6. Indonesia regulatory boundary as of 2026-11-23

The legal classification depends on the actual activity, not on using Rust, blockchain or calling an app "non-custodial".

- **Regulatory migration:** UU 4/2023 P2SK and PP 49/2024 transferred oversight from Bappebti to OJK/BI, with handover on 10 January 2025. Confirm exact OJK/BI remit for each business activity.
- **Crypto-asset trading:** [POJK 27/2024](https://ojk.go.id/id/regulasi/Pages/POJK-27-2024-AKD-AK.aspx) as amended by [POJK 23/2025](https://ojk.go.id/id/regulasi/Pages/POJK-23-2025-Perubahan-POJK-27-Tahun-2024-tentang-Penyelenggaraan-Perdagangan-Aset-Keuangan-Digital-Termasuk-Aset-Kripto.aspx), related SEOJK, and [PADK 3/2026](https://ojk.go.id/id/regulasi/Pages/PADK-Nomor-3-Tahun-2026-Penyelenggaraan-Perdagangan-Aset-Keuangan-Digital-termasuk-Aset-Kripto.aspx) where applicable. Licensing, segregation, capital, reporting and operational rules require counsel and regulator classification before entering a regulated role.
- **Payment prohibition:** [Bank Indonesia](https://www.bi.go.id/id/publikasi/ruang-media/news-release/Pages/sp_200418.aspx) identifies crypto/virtual currencies as not legal tender for payment in Indonesia, citing UU 7/2011 on Mata Uang. Do not implement merchant settlement in crypto as if permitted; use lawful Rupiah payment rails for domestic purchases.
- **AML/CFT:** [OJK's cyber guidelines](https://ojk.go.id/id/Publikasi/Roadmap-dan-Pedoman/ITSK/Pages/Pedoman-Keamanan-Siber-Penyelenggara-Perdagangan-Aset-Keuangan-Digital.aspx), POJK/SEOJK APU-PPT as applicable, and [FATF R.15/2026 virtual assets/VASPs report](https://www.fatf-gafi.org/en/publications/Fatfrecommendations/targeted-updated-virtualassets-vasps-2026.html) for Travel Rule/risk and licensing. Do not present Travel Rule as blanket obligation for every read-only indexer.
- **Privacy and electronic services:** UU 27/2022 PDP, PP 71/2019, UU ITE as amended by UU 1/2024, plus PSE applicability. Consider tax, consumer protection, cross-border data transfer and financial-sector rules based on product classification.
- **Crypto-specific religious/ethical review**, if the business markets a shariah-conforming product, requires independent qualified review of asset structure, ownership, riba/gharar/maysir and applicable fatwa; code design does not imply shariah compliance.

**No legal conclusion or operational permission is granted by this research.** Obtain external counsel, applicable licenses, and written owner acceptance as required.

## 7. Security and reliability control matrix

| Threat | Required design control | Reproducible evidence |
| --- | --- | --- |
| Key compromise/insider signing | External HSM/MPC/custodian, segregation, quorum, key lifecycle | Dummy keys only: negative access, rotation and revoke drills |
| Fraudulent tx / approval phishing | Typed EIP-712-style payload or chain-native equivalent, allowlists, exact digest, human review | Unauthorized chain, destination, nonce or limits rejected |
| Event reorg/double counting | Confirmed vs final settlement states, deterministic indexer and reconciliation | Synthetic reorg and duplicate events do not double-post |
| RPC spoofing/outage/rate limit | Endpoint pinning/TLS, multi-provider optional, timeouts/backoff/circuit breaker | Failover, stale head, inconsistent chain/network IDs |
| Cross-tenant funds/data | Backend policy, actor binding, per-service non-superuser DB roles, FORCE RLS | Negative tenant tests and audit evidence |
| Smart contract exploit | Independent security audit, static checks, formal analysis if warranted, allowlisted contract versions | Testnet simulations, audit trail and upgrade/pausing plan |
| Supply-chain/bridge hacks | SBOM, signed artifacts, dependency review and bridge threat model | Lockfile provenance, exploit simulation, immutable logs |
| Irrecoverable/incorrect settlement | Double-entry accounting, reconciliation, two-person release, recovery/rollback | Ledger balance and reconcile invariants, restore drills |
| On-chain privacy leak | Hash-only anchoring where necessary, PII minimization, unlinkability assessment | Automated PII fixture scans and legal privacy review |

Standards: ISO/IEC 27001/27002/27005, 27017/27018, 27034, 27701, ISO 22301, ISO/IEC 20000-1, ISO/IEC 15408; OWASP ASVS/API Security/Smart Contract security, NIST SSDF/CSF, FATF R.15 where applicable. This is a control candidate list, not proof of certification.

## 8. Technical integration options and comparative readiness

| Option | Source/protocol | Fit to AWBMS | Status |
| --- | --- | --- | --- |
| Signed event/hash anchor | PostgreSQL + off-chain signing; optional chain RPC | Narrow reusable application port | Most viable research |
| Permissioned Fabric | [Peers/orderer, endorsement policies](https://hyperledger-fabric.readthedocs.io/en/latest/peers/peers.html) | External fabric node/gateway, separate organization PKI | PoC after consortium governance |
| EVM read-only chain adapter | [Ethereum JSON-RPC](https://ethereum.org/developers/docs/apis/json-rpc/) | Separate Rust RPC worker, normalized events | PoC candidate |
| EVM tx submission | EVM execution client + owner-signed transactions | Async settlement state machine and fee/nonce service | High-risk, later |
| Custodial exchange backend | Custody, matching, treasury, trading/compliance | Far beyond current AWBMS | Defer |
| New L1/validator | Consensus and p2p chain | Different project than ERP/business backend | Not recommended |

A normal API response from a blockchain node cannot replace business rules, financial books or data ownership. Node operators retain their own infrastructure, resource budgeting, backups and monitoring. Never make AWBMS production release depend on an experimental Fuchsia kernel/SDK at this stage.

## 9. Roadmap and test plan (research first, gate-driven)

**Phase 0 — classification before code:** choose exact business persona and use case, legal/financial activity, custody/non-custody, parties, on-chain data, privacy, tax, settlement, availability, keys, cross-border, and ROI. Output: ADR *proposal*, threat model, regulator/counsel checklist; no accepted decision implied.

**Phase 1 — read-only synthetic chain bridge:** mock Ethereum JSON-RPC/Hyperledger gateway, pinned chain IDs, event normalization, cursor checkpoint, idempotency and deterministic replay. Prove unauthorized tenant denied and replay/reorg does not duplicate business event. Outside production backend workspace unless Gate 9 permits.

**Phase 2 — provenance pilot:** hash a synthetic document, store minimal metadata, optionally anchor a proof on public testnet; independently verify off-chain, redaction and erasure requirements.

**Phase 3 — ledger/accounting design:** if business case needs asset movements, specify double-entry postings, reconciliation, scale, custody separation, cross-chain finality and recovery before code.

**Phase 4 — regulated capabilities:** only after explicit legal classification, licensing/partner due diligence, HSM/MPC threat review, external security audit, human UAT and relevant release gates. **No live crypto deposits, withdrawals, staking, matching engine, token sale or customer funds during Stage 1.**

Suggested acceptance tests:
1. RPC wrong-chain and stale head rejected.
2. Two identical event notifications produce one logical credit/posting.
3. Synthetic reorg retracts provisional state without double-counting.
4. Signing service cannot sign a payload that differs from the approved hash.
5. Tenant A cannot read wallet/transaction metadata of tenant B.
6. Private-key/seed phrase fixtures never appear in logs/prompts/errors.
7. Node-provider outage degrades to bounded retry/read-only with alerts.
8. Backup/restore rebuilds indexer and reconciles against independent network state.

**Boundaries and ownership:** Stage 1 #1, documentation/evidence #7, security/quality #21/#22, contract-first #23. Do not duplicate these foundation issues; crypto-specific design belongs to the consumer product after Blueprint and Gate 9, not to generic AWBMS before readiness.

## Related follow-on research issues

- [#32](https://github.com/ahliweb/awbms/issues/32): documentation and review.
- [#35](https://github.com/ahliweb/awbms/issues/35): Indonesian regulatory classification, business and crypto scope.
- [#36](https://github.com/ahliweb/awbms/issues/36): synthetic read-only RPC/finality/reorg spec and spike plan.
- [#37](https://github.com/ahliweb/awbms/issues/37): business provenance/hash anchoring versus permissioned chain.
- [#38](https://github.com/ahliweb/awbms/issues/38): custody, signing, financial ledger and operational-risk boundary.

## 10. References and status of evidence

- [AWBMS snapshot at inspected commit](https://github.com/ahliweb/awbms/tree/e6bb5638e0628418079b1c1d6fcf0a123c5bb4cc)
- [Ethereum node architecture](https://ethereum.org/developers/docs/nodes-and-clients/node-architecture/), [Ethereum JSON-RPC](https://ethereum.org/developers/docs/apis/json-rpc/)
- [Hyperledger Fabric ordering](https://hyperledger-fabric.readthedocs.io/en/latest/orderer/ordering_service.html), [peers](https://hyperledger-fabric.readthedocs.io/en/latest/peers/peers.html)
- [OJK POJK 27/2024](https://ojk.go.id/id/regulasi/Pages/POJK-27-2024-AKD-AK.aspx), [POJK 23/2025](https://ojk.go.id/id/regulasi/Pages/POJK-23-2025-Perubahan-POJK-27-Tahun-2024-tentang-Penyelenggaraan-Perdagangan-Aset-Keuangan-Digital-Termasuk-Aset-Kripto.aspx), [PADK 3/2026](https://ojk.go.id/id/regulasi/Pages/PADK-Nomor-3-Tahun-2026-Penyelenggaraan-Perdagangan-Aset-Keuangan-Digital-termasuk-Aset-Kripto.aspx)
- [OJK crypto-sector cybersecurity guideline](https://ojk.go.id/id/Publikasi/Roadmap-dan-Pedoman/ITSK/Pages/Pedoman-Keamanan-Siber-Penyelenggara-Perdagangan-Aset-Keuangan-Digital.aspx), [BI crypto payment statement](https://www.bi.go.id/id/publikasi/ruang-media/news-release/Pages/sp_200418.aspx), [FATF 2026 virtual assets report](https://www.fatf-gafi.org/en/publications/Fatfrecommendations/targeted-updated-virtualassets-vasps-2026.html)
- [Indonesia UU 27/2022](https://peraturan.bpk.go.id/Details/229798/uu-no-27-tahun-2022), [PP 71/2019](https://peraturan.bpk.go.id/Details/122030/pp-no-71-tahun-2019), [UU 1/2024](https://peraturan.bpk.go.id/Details/274494/uu-no-1-tahun-2024)

**Verification performed:** GitHub file/code/issue audit and official primary-source review on 2026-11-23. **Not performed:** blockchain node deployment, Rust RPC integration, smart-contract tests, security audit, custody/KYC legal review, performance benchmarking, crypto transactions or user UAT.

## Changelog
- **v1.0.0 (2026-11-23):** Initial blockchain-business and crypto-asset feasibility/constraint research.
