# AWBMS — Crypto Custody, Signing and Ledger Safety Boundary (Reasons Not to Launch)

## Document control
- **Version:** v1.0.0
- **Updated:** 2026-11-23
- **Issue:** [#38](https://github.com/ahliweb/awbms/issues/38) (F0/07)
- **Status:** RESEARCH / PROPOSED only. A paper threat model. **No licence is asserted; no approval to sign, hold, deposit, withdraw, trade, stake or handle customer funds.** Merging this document is not implementation authorization.
- **Parent note:** [Blockchain business backend and crypto-asset readiness](blockchain-crypto-business-backend-readiness-2026-11-23.md) (its §4 invariants 1-8 and §7 matrix are the baseline; this document extends them and does not restate them).
- **Siblings:** [#35 regulation](blockchain-business-models-indonesia-regulation-2026-11-23.md), [#36 RPC/finality spec](chain-rpc-indexer-finality-reorg-spec-2026-11-23.md), [#37 anchoring vs ledger](hash-anchoring-vs-permissioned-ledger-2026-11-23.md).
- **Source retrieval date:** 2026-11-23 (regulatory sources as logged in #35).
- **Counsel and independent review:** REQUIRED and **NOT performed, not simulated**. The "signed" register required by the issue's definition of done is **not signed**; no one has signed it.
- **Claim labels:** `[F]` repository fact; `[C]` external claim with source; `[A]` assumption; `[R]` recommendation; `[O]` open decision; `[G]` verification gate.
- **Not executed:** no dummy-key drill, no test, no HSM or MPC evaluation was performed.

## 1. Position

[R] AWBMS should **not** launch custody, signing, deposits, withdrawals, trading, staking, a matching engine or any client-fund handling. This document lists the reasons and the conditions under which the question could be reopened. [F] AWBMS at `59cc2c3` has no signing code, key store, ledger or custody workflow; the foundation items it would depend on (secret baseline #14, security/RLS testing #22, supply chain #7, Stage 1 #1) are separate issues. [A] Their status is not re-verified here.

## 2. Reasons not to launch (summary)

1. [A] Loss is irreversible: a lost or stolen key is a permanent loss; there is no chargeback.
2. [A] Key compromise by an insider is the dominant risk and cannot be prevented by application code alone.
3. [O] The licensing perimeter for custody is unresolved and counsel review has not occurred (see [#35](blockchain-business-models-indonesia-regulation-2026-11-23.md), gate LG-1).
4. [A] There is no financial ledger, accounting, reconciliation or audit design; the accounting domain needs its own PRD, ERD, segregation of duties and contract, plus Gate 9, none of which exist.
5. [A] Insolvency, segregation of customer assets and disaster recovery have no designed or tested answer.
6. [A] AML/KYC, sanctions screening and Travel Rule duties, where legally applicable, are unimplemented and their applicability is unresolved (#35).
7. [A] Operational capacity (24x7 on-call, key ceremonies, incident response) does not exist.

## 3. Three distinct models and reconciliation invariants

[R] Three models MUST never be merged into one table or treated as interchangeable.

| Model | Purpose | Authority for | Written by | Mutability |
| --- | --- | --- | --- | --- |
| **L - Accounting ledger** (double-entry, PostgreSQL) | Who owns what, in integer base units, with postings that balance | Customer entitlements and liabilities (if ever in scope) | Domain service only, via validated postings | Append-only; corrections are new reversing postings |
| **C - Chain state** (external blockchain) | What the network says about addresses, transactions, events | Existence and ordering of on-chain transfers (subject to finality, see [#36](chain-rpc-indexer-finality-reorg-spec-2026-11-23.md)) | The network; AWBMS only reads | Not controlled by AWBMS; can reorg before finality |
| **A - Audit log** (signed, tamper-evident) | Who did what, when, with which approvals | Evidence of actions and decisions | Every actor, via an append-only service | Append-only; no ledger or chain effect |

Reconciliation invariants (all `[R]`, none implemented):

| ID | Invariant |
| --- | --- |
| I-1 | For every asset and every closed period, sum of all ledger debits equals sum of all credits (double-entry balance). |
| I-2 | For every asset, ledger-recorded custody balance for each controlled address equals the finalized on-chain balance plus explainable in-flight amounts (pending withdrawals), with each difference tied to a named exception record. |
| I-3 | No customer balance is negative; no posting reduces a balance below zero. |
| I-4 | Every ledger posting that moves value references exactly one authorized, audited intent (approval chain recorded in A); no posting without an A record. |
| I-5 | Every on-chain movement from a controlled address matches an approved and signed intent; any unmatched on-chain movement is an incident (possible key compromise). |
| I-6 | A chain event enters L only from `reconciled` state ([#36 §3](chain-rpc-indexer-finality-reorg-spec-2026-11-23.md#3-data-states)); a reorg after crediting must be impossible by construction. |
| I-7 | Integer base units only, asset-specific scale; no floating point. |
| I-8 | A is itself verifiable: hash chain continuous, signatures valid, anchored copy matches (see #37 design). |
| I-9 | Reconciliation runs on a schedule; a non-zero unexplained difference halts withdrawals automatically (kill switch). |

[A] Liability versus reserves (proof that reserves cover customer liabilities) is a further invariant for any regulated custodian and is `[O]`/UNVERIFIED as a legal requirement.

## 4. Key handling boundary

### 4.1 Hard rule

[R] **A signer key, seed phrase, mnemonic, wallet export, recovery share, PIN, or any material that can authorize a transfer MUST NEVER be present in:**

| Location | Control | Evidence required (NOT EXECUTED) |
| --- | --- | --- |
| AWBMS application process memory or code | The application has no signing code path; signing is an external service holding the key | Dependency and code scan for signing libraries; none present |
| Application, agent and worker logs, errors, traces, metrics | Secret redaction types that cannot print; log scanning | Fixture with canary dummy seed never appears in captured output |
| AI agent prompts, tool inputs and outputs, memory files | Policy that no secret is provided to an agent; canary tests; agents get no credentials with value-moving authority | Canary dummy seed in environment is not readable by agent tools |
| CI pipeline, build logs, artefacts, caches | Secret scanning in CI; no signing secret stored in CI; OIDC-style short-lived credentials only for non-signing tasks | Canary in a CI test is blocked or redacted |
| Browser bundles, local storage, cookies, front-end state | No key material ever sent to the browser; signing by user device or custodian | Static scan of bundle for canary patterns |
| Normal business database, backups, dumps, replicas | Separate secret store; the business DB stores key **identifiers** only | Schema review plus canary scan of a restored backup |
| Source repository, history, issues, PRs, docs | Secret scanning and push protection; dummy keys only in this document class | History scan |

[F] The repository contains no such secret material as far as this documentation change is concerned; the files added contain no keys, seeds or addresses (verified by inspection of these four documents at authoring). [A] Repository-wide scanning is not part of this change.

### 4.2 HSM and MPC (paper analysis only)

| Aspect | HSM | MPC / threshold signing | Notes |
| --- | --- | --- | --- |
| Key never leaves boundary | Hardware boundary | Key never reconstituted in one place | Both depend on policy engine correctness |
| Failure modes | Vendor firmware, admin credentials, physical access, backup of wrapped keys | Protocol bugs, collusion of share holders, availability of quorum | Independent review needed `[O]` |
| Recovery | Key ceremony, wrapped backup, quorum of custodians | Share rotation, resharing, share backup | Recovery drill is mandatory before any value |
| Chain support | Curve and signature scheme support varies | Scheme support varies | Check per chain `[O]` UNVERIFIED |
| Certification | Evaluation schemes exist (for example common-criteria style); not verified here | Less standardized `[A]` | No certification claim is made |

[R] External certified custody or a qualified custodian is preferred over self-built HSM/MPC for any future product (see §8).

### 4.3 Key lifecycle

[R] The following lifecycle stages each need a written procedure, a named owner, dual control and an audit record: generation (ceremony, witnessed, offline entropy), registration of public key and policy, activation, use (policy-bound signing), rotation, suspension, revocation, compromise response, backup, recovery, retirement and destruction. Recovery must not depend on a single person; loss of any one person must not lose funds, and no one person may be able to move funds.

### 4.4 Treasury and payout

[R] Wallet tiers (cold, warm, hot) with hard caps on warm and hot balances; refilling from cold needs a quorum; multi-approver payout (maker, checker, optional third approver above a threshold); approver identities and devices bound to the approval; cooling-off delay for new destinations; address allow-lists with delay on change; per-transaction, per-day and per-customer limits; a kill switch that any of several roles can trigger but only a quorum can lift.

### 4.5 Segregation

[R] Separation of duties: the person who requests cannot approve; the person who approves cannot operate the signer; the operator of the signer cannot change policy; the policy owner cannot access keys; reconciliation staff cannot post. Customer assets segregated from operator assets on-chain (separate addresses and accounts) and in the ledger. Whether segregation must follow a specific legal form (trust, bankruptcy-remote structure) is a counsel question `[O]` (#35, LG-1).

## 5. Threat model mapped to negative tests (all NOT EXECUTED)

Dummy keys and a local test chain only. No live transaction. Each threat row maps to at least one negative test.

| ID | Threat | Control | Negative test (expected result) |
| --- | --- | --- | --- |
| T-01 | Unsafe signing: signer signs a payload different from the approved intent | Sign only the digest of the approved, immutable intent; policy engine re-derives the digest | Alter one byte of destination, amount, fee or calldata after approval: signer refuses |
| T-02 | Replay of a signed transaction on the same chain | Nonce managed by one authority; signed payload bound to nonce | Resubmit an already-used signed payload: rejected; no second ledger posting |
| T-03 | Cross-chain replay / chain-id mismatch | Signed payload includes `chainId`; signer pins allowed `chainId` per key | Request signature with a wrong `chainId`: refused; send to a node reporting different `chainId`: refused (see #36 F-01) |
| T-04 | Nonce misuse: gap, reuse, concurrent signing producing stuck or conflicting transactions | Serialized per-address nonce allocation under lock; reconcile pending set | Two concurrent requests for the same address: one nonce each, no duplicate |
| T-05 | Withdrawal limit bypass by splitting, parallel requests, or race | Limits enforced in the signer policy, not only the application; atomic counters | N requests below the per-transaction limit exceeding the daily limit: later ones refused |
| T-06 | Privileged actor abuse: insider approves own request | Segregation of duties; approvals bound to distinct identities | Same identity as maker and checker: rejected; admin attempts to edit policy and sign in one session: rejected |
| T-07 | Privileged actor abuse: DB administrator edits ledger or approvals | Append-only ledger and audit, separate roles, external anchor of audit digests | Direct update on ledger row by application role: denied; tamper detected by hash-chain check |
| T-08 | Destination substitution or address poisoning | Allow-list with delay, checksum and network-aware address validation | Look-alike address and unlisted address: refused or held for review |
| T-09 | Compromised approver device or phishing | Step-up authentication, hardware-bound approval, out-of-band confirmation above threshold | Approval from unregistered device: rejected |
| T-10 | Signer isolation breach (application host compromised) | Signer on separate trust domain, authenticated mutually, no private network path from the app beyond a narrow API | Application credentials cannot request a signature outside policy; cannot read key material |
| T-11 | Key or seed leak to logs, prompts, CI, browser, DB, repo | §4.1 controls | Canary seed searches over all channels return nothing |
| T-12 | Reorg after crediting or paying | Credit only from `reconciled`; payouts only from finalized funds | Replay synthetic reorg: no ledger posting changed |
| T-13 | Double payout through retry | Idempotency key per intent; outbox | Retry a payout request: one signing, one posting |
| T-14 | RPC lies about balance or receipt | Two independent providers, finality policy | Malicious node responses: halted (see #36 F-05) |
| T-15 | Smart-contract or token risk (fee-on-transfer, upgradeable, pausable token) | Allow-list of token contracts and versions; independent audit | Unlisted token deposit ignored, not credited |
| T-16 | Insolvency or run: liabilities exceed assets | Reserve reconciliation, proof of reserves design, limits | Simulated shortfall triggers automatic halt |
| T-17 | Sanctions or AML hit on counterparty | Screening where legally applicable; hold and review | Test counterparty on a synthetic watch list: held, not paid |
| T-18 | Disaster: loss of a custodian, datacentre or HSM | Documented recovery with tested drill | Recovery drill from dummy backup restores signing capability; reconciliation passes |
| T-19 | Cross-tenant data or wallet metadata leakage | FORCE RLS and actor binding (see #36 section 10) | Tenant A cannot read tenant B wallet or approval data |
| T-20 | Supply-chain compromise of signer or SDK | Pinned, reviewed, minimal dependencies; SBOM; reproducible build | Unsigned artefact refused at deploy |
| T-21 | Agent or automation misuse | No agent has any signing or approval credential; human-in-the-loop for any value-moving action | Agent tool call to a signing endpoint: denied by design |

[G] Gate: dummy-key abuse and recovery drills are specified (§6) and must be executed in an isolated environment before any proposal to proceed.

## 6. Dummy-key abuse and recovery drill specifications (NOT EXECUTED)

1. **Abuse drill:** with a dummy signer and a local test chain, execute T-01 to T-10 and T-13, recording expected refusals and audit entries.
2. **Recovery drill:** generate a dummy key under a documented ceremony, split or wrap backups, destroy the live signer, recover from backup with the quorum, sign a test message, and verify addresses match.
3. **Compromise drill:** declare a dummy key compromised; measure time to revoke, rotate and migrate dummy balances; verify nothing signed after revocation is accepted.
4. **Reconciliation drill:** inject a deliberate one-unit mismatch in each of L, C and A; verify I-2 and I-5 detect it and I-9 halts withdrawals.
5. **Canary drill:** place a dummy seed in each forbidden location from §4.1 in a controlled test and verify detection or absence.

[R] Record date, commit hash, participants and results in an evidence artefact; none exists today.

## 7. Compliance obligations (where legally applicable)

[O] Applicability is unresolved for every item below and depends on the licensed activity (see [#35](blockchain-business-models-indonesia-regulation-2026-11-23.md)). This section lists duties to **ask counsel about**, not duties confirmed.

| Duty | Notes |
| --- | --- |
| AML/KYC customer due diligence and ongoing monitoring | Indonesian AML/CFT provisions for this sector were **not retrieved**; UNVERIFIED (#35 S9) |
| Travel Rule (originator and beneficiary information with transfers) | FATF R.15 context (#35 S4, partial). Do not assume it applies to a read-only indexer |
| Sanctions screening | Applicable lists and legal basis UNVERIFIED |
| Suspicious transaction reporting to the financial intelligence unit | PPATK sources not consulted; UNVERIFIED |
| Personal data handling for KYC | PDP law verified as in force (#35 S8); article-level duties not read |
| Record retention | Period UNVERIFIED; conflicts with erasure need counsel |
| Consumer protection and complaint handling | Not researched |

## 8. OJK classification and the counsel / qualified-custodian path (no licence assertion)

- [C] OJK instruments POJK 27/2024, amended by POJK 23/2025, and PADK 3/2026 govern operators of digital financial asset (including crypto-asset) trading; the pages were read on 2026-11-23, article text was not (#35 S1-S3).
- [O] Whether a custody-only service, a signing-as-a-service product or a treasury product falls under those instruments, another OJK category, or a Bank Indonesia payment category is **not determined here**.
- [R] Path: (1) obtain written opinion from Indonesian-qualified counsel on classification; (2) if regulated, prefer to integrate a **qualified, licensed custodian or partner** under a contract rather than self-custody, after vendor due diligence; (3) obtain regulator clarification where counsel advises; (4) obtain written owner acceptance of residual risk; (5) only then write a product-scoped PRD, ERD, SoD matrix and contract and pass Gate 9.
- **Nothing in this repository states or implies that any AWBMS-derived product is or will be licensed.**

## 9. Explicit deferrals

The following are **deferred and not authorized** by this document or its merge: generating or storing signing keys; signing any transaction; accepting deposits; executing withdrawals; trading or order matching; a matching engine; staking or validator operations; token sales or issuance; lending or yield products; handling of any client funds; any live financial transaction. A separate domain PRD/ERD/SoD/contract, Gate 9, counsel opinion, independent security review and owner sign-off are prerequisites (and may not be sufficient).

## 10. Risk, assumption and no-go register (unsigned)

| ID | Item | Type | Owner | Status |
| --- | --- | --- | --- | --- |
| X-1 | Irrecoverable key loss | Risk | Unassigned | Open |
| X-2 | Insider or privileged-actor fraud | Risk | Unassigned | Open |
| X-3 | Insolvency, segregation failure | Risk | Unassigned | Open |
| X-4 | Conflicting legal obligations (AML retention vs erasure, cross-border) | Risk | Unassigned | Open, counsel |
| X-5 | Hidden key access via logs, CI, agents, backups | Risk | Unassigned | Open; §4.1 and §6 |
| X-6 | Assumption: external certified custodian is available and licensable | Assumption | Unassigned | Unvalidated |
| X-7 | Assumption: signer policy can enforce limits independently of the app | Assumption | Unassigned | Unvalidated |
| X-8 | Regulatory classification unresolved | Open decision | Owner | Open (#35) |

**No-go until qualified independent review** (security, legal, financial audit). This register is **unsigned**; "signed" in the issue's definition of done refers to a future human sign-off.

**Product-scoped next-step proposal** (not authorization): (1) counsel classification (#35 gates); (2) paper evaluation of two qualified custodians; (3) accounting domain design for the ledger model L with an independent accountant; (4) isolated dummy-key drill environment per §6; (5) only then a go/no-go review.

## 11. Rollback

Mark the concept **deferred**; no code, schema or contract was added, so there is nothing to revert beyond this document.

## Acceptance criteria trace

| Issue #38 acceptance criterion | Section | Status |
| --- | --- | --- |
| Asset/accounting vs chain state vs audit log as three distinct models, reconciliation invariants | §3 (I-1 to I-9) | Done as specification; not implemented |
| Signer key and seed never in app/agent logs, prompt, CI, browser, normal DB or repository | §4.1, T-11, T-21, §6 drill 5 | Specified; NOT EXECUTED |
| Unsafe signing/replay/nonces, chain-id mismatch, withdrawal limits, privileged actor threats mapped to negative tests | §5 (T-01 to T-06, T-07, T-13) and §6 | Mapped; NOT EXECUTED |
| OJK classification, counsel review, qualified custodian/partner path; no assertion of licence | §8, §7 | Done; classification undetermined; no licence asserted |
| Defer signing, deposit, withdrawal, trading, staking, matching engine, client funds | §9 | Done |
| Threat model: HSM/MPC, signer isolation, key lifecycle, recovery, treasury, multi-approver payout, segregation, double-entry, AML/KYC/Travel Rule/sanctions | §4.2, §4.3, §4.4, §4.5, §3, §5, §7 | Done on paper |
| Paper threat model, dummy-key drills specified, no live transactions | §5, §6 | Done; NOT EXECUTED |
| DoD: threat/assumption/risk register and next-step proposal; merge is not implementation authorization | §10, document control | Done; register unsigned |

## Changelog
- **v1.0.0 (2026-11-23):** Initial custody, signing and ledger safety boundary for issue #38.
