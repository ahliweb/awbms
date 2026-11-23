# AWBMS — Blockchain Business Models vs Indonesian Crypto Regulation (Evidence Matrix)

## Document control
- **Version:** v1.0.0
- **Updated:** 2026-11-23
- **Issue:** [#35](https://github.com/ahliweb/awbms/issues/35) (F0/04)
- **Status:** RESEARCH / PROPOSED only. **No legal conclusion, no licence assertion, no approval to build.**
- **Parent note:** [Blockchain business backend and crypto-asset readiness](blockchain-crypto-business-backend-readiness-2026-11-23.md). This document refines its §6 and §5; it does not repeat the parent's trust-boundary or control matrices.
- **Sibling notes:** [#36 RPC/finality spec](chain-rpc-indexer-finality-reorg-spec-2026-11-23.md), [#37 anchoring vs ledger](hash-anchoring-vs-permissioned-ledger-2026-11-23.md), [#38 custody safety boundary](crypto-custody-signing-ledger-safety-boundary-2026-11-23.md).
- **Source retrieval date:** all external sources below were retrieved on **2026-11-23** unless stated otherwise.
- **Human counsel review:** **REQUIRED and NOT performed. Not simulated.** Nothing here was reviewed by an Indonesian-qualified lawyer, OJK, BI or PPATK.
- **Claim labels:** `[F]` verified in this repository; `[C]` recorded external claim (source named); `[A]` assumption; `[R]` recommendation; `[O]` open decision / unresolved question; `[G]` verification gate (evidence that would discharge a claim). Labels here are local to this research note and create no register rows.

## 1. Purpose and boundary

[R] Separate **non-financial blockchain integration** (provenance, anchoring, read-only chain data) from **regulated crypto-asset services** before any product architecture. [F] AWBMS at `59cc2c3` is a specification and verification spike with no crypto, custody, RPC or ledger code (see parent note §2; not re-inspected beyond the parent's evidence).

[R] Classification is by **activity performed**, not by technology, by the word "non-custodial", or by the word "read-only". This is the single rule the rest of the document tests.

## 2. Citation verification log (retrieved 2026-11-23)

Status key: **VERIFIED** = the page was read directly on the date above and the stated facts appear on it. **PARTIAL** = reachable only through a secondary route (search index, regulator summary page, news). **UNVERIFIED** = not fetched, or fetch failed, or only a third party asserts it. An UNVERIFIED item must not be relied on.

| ID | Citation | Status | What was actually observed |
| --- | --- | --- | --- |
| S1 | [OJK, POJK 27/2024](https://ojk.go.id/id/regulasi/Pages/POJK-27-2024-AKD-AK.aspx) | **VERIFIED** (page) | [C] Title: Penyelenggaraan Perdagangan Aset Keuangan Digital Termasuk Aset Kripto. Enacted 10 Dec 2024, promulgated 12 Dec 2024. The page is internally inconsistent on effective date: abstract says 10 Jan 2025, metadata field reads "1/10/2025". Page does not state in-force status and lists no amendment. Full-text PDF and FAQ were linked but **not read**; article-level obligations are UNVERIFIED. |
| S2 | [OJK, POJK 23/2025](https://ojk.go.id/id/regulasi/Pages/POJK-23-2025-Perubahan-POJK-27-Tahun-2024-tentang-Penyelenggaraan-Perdagangan-Aset-Keuangan-Digital-Termasuk-Aset-Kripto.aspx) | **VERIFIED** (page) | [C] Amends POJK 27/2024. Enacted 31 Oct 2025, promulgated 10 Nov 2025, effective on promulgation. Page abstract: adds a trading mechanism for digital financial asset **derivatives**, widens operator scope; transitional rules (Bappebti derivative approvals remain valid; unlicensed parties conducting derivative transactions to move activity to a licensed trader by 9 Jan 2026). The amendment relationship **is confirmed** by S2 itself and by OJK press release SP 210 surfaced in search (press release page not opened). Article text not read. |
| S3 | [OJK, PADK 3/2026](https://ojk.go.id/id/regulasi/Pages/PADK-Nomor-3-Tahun-2026-Penyelenggaraan-Perdagangan-Aset-Keuangan-Digital-termasuk-Aset-Kripto.aspx) | **VERIFIED** (page) | [C] Peraturan Anggota Dewan Komisioner (PADK). Enacted 30 Jun 2026, effective 1 Sep 2026. Abstract: implementing rules and **reporting** procedures (periodic, risk self-assessment, incidental, clearing notifications) for operators; revokes parts of SEOJK 20/SEOJK.07/2024 in stages (1 Jul 2026, 1 Jan 2027, remainder on effect). Based on abstract/metadata only; PDF not read. Note it is a reporting-procedure instrument for operators, not a licensing instrument for third-party software vendors. |
| S4 | [FATF, Seventh Targeted Update on VA/VASPs (2026)](https://www.fatf-gafi.org/en/publications/Fatfrecommendations/targeted-updated-virtualassets-vasps-2026.html) | **PARTIAL** | Direct fetch of fatf-gafi.org returned HTTP 403 (two URL variants). The page title and a 16 Jul 2026 date appear in the search index. Content was read via [OJK APU-PPT summary of the FATF update](https://www.ojk.go.id/apu-ppt/id/berita-dan-kegiatan/info-terkini/Pages/FATF-Targeted-Update-%e2%80%93-July-2026.aspx) (dated 16 Jul 2026, says it adapts the FATF text). [C] via OJK summary: R.15 progress; remaining gaps in risk assessment, VASP licensing/registration, supervision; risks flagged: stablecoins, unhosted wallets, offshore VASPs, DeFi. Statistics quoted in secondary commentary (for example Travel Rule percentages) are **UNVERIFIED against FATF text** and are deliberately not used here. |
| S5 | [BI press release 20/4/DKom](https://www.bi.go.id/id/publikasi/ruang-media/news-release/Pages/sp_200418.aspx) | **VERIFIED** (page), **dated** | [C] Dated **13 Jan 2018** (not a 2026 instrument; the parent note calls it a "statement"). States virtual currency is not a legal payment instrument and bars payment-system operators and fintech providers from processing virtual-currency payments; cites UU 7/2011 (Mata Uang) and PBI 18/40/PBI/2016, PBI 19/12/PBI/2017. **Whether those PBIs are still in force or have been replaced is UNVERIFIED** (not checked on bi.go.id). |
| S6 | BI current position 2025-2026 (stablecoin, Rupiah Digital) | **UNVERIFIED** (news only) | Search returned only press coverage: officials say stablecoin will not be a payment instrument, Rupiah Digital is not crypto, a national stablecoin-like instrument is *planned*; a news headline mentions a stablecoin POJK "January 2027". No official BI/OJK text read. Treat as direction of travel, not rule. |
| S7 | UU 4/2023 (P2SK), PP 49/2024, handover Bappebti to OJK on 10 Jan 2025 | **PARTIAL** | S1 page cites UU 21/2011 as amended by UU 4/2023 and an effective date of 10 Jan 2025; S2 mentions Bappebti permits moving to OJK. PP 49/2024 itself and the "handover" wording on peraturan.bpk.go.id / JDIH were **not fetched: UNVERIFIED**. |
| S8 | [UU 27/2022 PDP](https://peraturan.bpk.go.id/Details/229798/uu-no-27-tahun-2022) | **VERIFIED** (page) | [C] Status "Berlaku"; enacted 17 Oct 2022; page notes judicial review Putusan MK 151/PUU-XXII/2024 reading "dan" in Pasal 53(1)(b) as "dan/atau" (not an amendment). Article-level obligations not read. |
| S9 | PP 71/2019, UU 1/2024 (ITE), PSE rules, PPATK regulations, OJK APU-PPT POJK for this sector | **UNVERIFIED** | Cited in parent note; not re-fetched here. PPATK sources were not consulted at all in this pass. |
| S10 | [Hyperledger Fabric endorsement policies](https://hyperledger-fabric.readthedocs.io/en/latest/endorsement-policies.html) | **VERIFIED** (page) | [C] Chaincode-, collection- and key-level endorsement; private data collections. Technical, not legal; used by #37. |

[G] Gate to discharge S1-S3 article-level claims: a counsel-reviewed reading of the PDF texts (not performed). [G] Gate to discharge S6/S7/S9: retrieve the primary texts from ojk.go.id, bi.go.id, peraturan.bpk.go.id or JDIH and record retrieval date and hash of the PDF in a follow-up.

## 3. Status summary of the user-requested checks

| Question in issue #35 | Result |
| --- | --- |
| POJK 27/2024 exists and its identity | **VERIFIED** (S1). Effective date inconsistent on the page itself. |
| Amended by POJK 23/2025 | **VERIFIED** (S2, page states it amends POJK 27/2024). Adds derivatives. |
| PADK 3/2026 | **VERIFIED** as a reporting-procedure instrument effective 1 Sep 2026 (S3); scope beyond abstract UNVERIFIED. |
| FATF 2026 VASP status | **PARTIAL** (S4): existence and thrust confirmed through an OJK summary; FATF page itself blocked (403). |
| BI Rupiah settlement / payment boundary | **PARTIAL**: 2018 BI release verified (S5); current instruments and 2025-2026 stance UNVERIFIED (S6). |

[A] The parent note's claim that these rules "require counsel and regulator classification before entering a regulated role" is retained as a recommendation, not a finding.

## 4. Classification matrix: service type vs obligations

How to read: a cell is a **question to put to counsel**, not an answer. "Likely relevant" means a regulator could plausibly argue it applies given the activity; it is `[A]`. Primary source column names the instrument to read; "UNVERIFIED" means it must be fetched before relying on it.

| Service / actor | Assets held or moved | Custody | Payment role | Trading role | Licensing question [O] | AML/KYC/CFT question [O] | Privacy (PDP) question [O] | Primary source status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Off-chain hash anchoring (provenance, certificates) | None | None | None | None | Probably none specific to crypto-asset regulation if no asset is created, held, traded or sold; confirm. [A] | Low, unless the anchored fact is itself a financial instruction. | High: hashes of personal data can remain personal data; no PII on chain. | S8 VERIFIED page; articles UNVERIFIED |
| Permissioned consortium ledger (non-asset records) | None or non-financial records | None | None | None | Consortium governance and contract law first; sector rules depend on the data (health, finance). [A] | Depends on data. | High: erasure/rectification versus immutability. | S8; sector rules UNVERIFIED |
| EVM read-only RPC indexer / analytics dashboard | None (reads public state) | None | None | None | Does reading public data alone create a regulated role? Counsel to confirm; do not assume exempt (see §6). | Depends on whether output is used to make or support transfers for customers, or to screen wallets for a regulated party. | Medium: wallet addresses linked to a person can be personal data. | S4 PARTIAL; local OJK AML text UNVERIFIED |
| Non-custodial wallet connector / orchestration | Customer-controlled | Customer holds keys; operator may build, route or relay transactions | Potential facilitation | Possibly, if routing orders or swaps | **Open.** Depends on actual control (can operator block, redirect, reverse, or influence a transfer?), fees, marketing. Counsel [O]. | Potentially relevant if operator is an intermediary; FATF flags unhosted wallets as a risk area (S4). | High: wallet metadata linked to identity. | S1-S3 VERIFIED pages; article-level UNVERIFIED |
| Custodial wallet / signing / treasury | Customer assets | Operator controls keys | Transfer service | Often paired with trading | Very likely inside a regulated perimeter (S1-S3 are about operators of digital financial asset trading, incl. custody-related duties) but exact classification is a legal question. [A] | Likely relevant: KYC, monitoring, Travel Rule where law applies (S4 context). | High. | UNVERIFIED at article level |
| Crypto exchange / broker / market operator | Customer assets and orders | Yes | Settlement | Yes | Within the subject of S1-S3 (licensed trader/exchange/clearing). Entering without licence: stop. | Yes, per applicable APU-PPT rules (UNVERIFIED). | High. | S1-S3 VERIFIED pages |
| Derivatives on digital financial assets | Positions, margin | Yes | Settlement | Yes | S2: derivatives trading mechanism added; exchange approval and consumer knowledge test per search/FAQ summary; counsel gate. | Yes. | High. | S2 VERIFIED page; knowledge-test detail PARTIAL |
| Asset tokenization / token sale / loyalty-token transfer | Tokens that may represent claims or be transferable | Depends | Possibly | Possibly secondary market | **Do not assume a token is outside securities, commodities, digital-financial-asset or payment rules.** Counsel and regulator classification [O]. Public offering is a separate gate. | Likely if transferable for value. | Medium to high. | UNVERIFIED (no securities-law source read) |
| Crypto as payment for domestic purchases | Crypto accepted as tender | Depends | Payment | None | S5 (2018) says virtual currency is not a legal payment instrument and bans payment operators from processing it; current instrument UNVERIFIED. Use Rupiah rails. [R] | Yes. | Medium. | S5 VERIFIED page, dated; S6 UNVERIFIED |
| New L1/L2 chain, validator, node operation | Native token (if any) | Staking/validators may hold others' assets | Potentially | Potentially | Product separate from AWBMS (parent note §8). Token and staking services need classification. | Potentially. | Low for validators alone; high for staking services with users. | UNVERIFIED |

[F] This matrix contains no facts about AWBMS behaviour; AWBMS implements none of these rows today.

## 5. Oversight bodies (who to ask)

| Body | Remit relevant here | Status of sources |
| --- | --- | --- |
| OJK | Supervision of digital financial asset (incl. crypto) trading per S1-S3 | S1-S3 VERIFIED (pages) |
| Bank Indonesia | Payment systems, Rupiah as currency, Rupiah Digital | S5 VERIFIED (2018); current UNVERIFIED |
| PPATK | AML/CFT financial intelligence and reporting | **No PPATK source consulted. UNVERIFIED.** |
| Personal data authority / PDP implementing rules | Personal data protection | S8 page VERIFIED; implementing regulation status UNVERIFIED |
| Bappebti (historic) | Previous crypto commodity regulator | S2 transitional text confirms permits move to OJK; wider handover UNVERIFIED |

[O] Which of these must be contacted, in which order, for each business model is a counsel decision.

## 6. Rejected claim: "non-custodial or read-only means exempt"

[R] AWBMS research **rejects** the proposition that a service is automatically outside financial, AML/CFT, payment or consumer regulation because it is described as non-custodial or read-only. Reasons:

1. [A] The FATF VASP concept is understood to be activity-based (not re-verified in this pass; FATF page blocked, S4). [C] What S4 (via the OJK summary) does say is that FATF reports on unhosted wallets, DeFi and the difficulty of identifying who controls an arrangement, which is consistent with regulators looking through labels to control and influence.
2. [A] An operator that can influence, route, block, delay or monetize a transfer may be treated as exercising control even if it never holds a key. Whether that is so under Indonesian law is [O] and unresolved.
3. [A] A "read-only" indexer can become part of a regulated flow if its outputs are used to credit balances, release goods, or decide payouts. #36 therefore forbids financial crediting from the spike.
4. Privacy law applies to wallet addresses and metadata linked to persons regardless of custody (S8, article-level UNVERIFIED).

[R] Absence of a rule found in this pass is not evidence of exemption; it is evidence the search was incomplete.

## 7. Decision gates: counsel and owner

Each gate requires **written human counsel opinion** (Indonesian-qualified, not simulated) and **written owner acceptance**. Until both exist the answer is **no-go**.

| Gate | Trigger activity | Questions for counsel [O] | Owner decision [O] | Default |
| --- | --- | --- | --- | --- |
| LG-1 Crypto custody | Any control of customer keys or assets | Licence category, qualified custodian or partner path, segregation, capital, insolvency treatment | Whether to partner with a licensed custodian instead | No-go. See [#38](crypto-custody-signing-ledger-safety-boundary-2026-11-23.md) |
| LG-2 Derivatives | Any derivative-like product, margin, leverage | S2 exchange approval, consumer knowledge test, tax | Whether the business accepts the regulatory perimeter at all | No-go |
| LG-3 Asset tokenization | Tokens representing claims, revenue share, invoices, real-world assets | Securities/digital financial asset/commodity classification | Funding model and investor profile | Deferred |
| LG-4 Exchange / marketplace | Matching, listing, order routing | Operator licence, listing rules, clearing | Whether to build, buy or integrate a licensed operator | No-go |
| LG-5 Payments | Accepting crypto for purchases, stablecoin settlement, wallet-to-merchant flows | S5 payment boundary, payment-system licensing, Rupiah rails | Use Rupiah only; any exception needs counsel | No-go for crypto tender |
| LG-6 Public offering / marketing | Any sale or promotion of tokens or yield | Offering rules, advertising, consumer protection | Marketing claims policy | No-go |

[G] A gate is discharged only by an artifact (signed opinion, regulator correspondence) stored outside this repository if privileged, with a pointer recorded. This document discharges none.

## 8. Business examples versus a PostgreSQL alternative

All examples are illustrative and carry no measured demand, cost or ROI. [A] for every performance or cost statement.

| # | Business example | Blockchain-style design | PostgreSQL alternative | Where blockchain might add value | When PostgreSQL is enough | Regulatory touchpoint |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Certificate / diploma authenticity | Anchor document hash on public chain | Signed audit table plus public verification endpoint | Third party verifies without trusting issuer's server uptime | Verifier already trusts issuer's domain | PDP: hashes of personal data; no crypto-asset service |
| 2 | Supply-chain provenance among independent firms | Consortium ledger with endorsement | Shared PostgreSQL service run by one neutral operator, signed events | No party will accept another as operator | One party owns the process or a neutral host is acceptable | Contract/governance; PDP |
| 3 | Merchant loyalty points across brands | Transferable token | Ledger table with double-entry | Cross-merchant transfer without a common operator | Single issuer, non-transferable points | Token classification gate LG-3 |
| 4 | Invoice financing / receivable tokenization | Token for receivable | Receivable register with signed assignments | Fractional secondary market (a regulated activity) | Internal register is enough | LG-3, LG-6 likely relevant |
| 5 | Crypto wallet portfolio dashboard | Read-only RPC indexing | Same indexer writing to PostgreSQL | None needed beyond chain read | Always (chain is the data source, DB is the product) | Whether outputs feed any transfer decision; see §6 |
| 6 | Cross-border B2B settlement with stablecoin | On-chain settlement | Bank transfer / local payment rails | Speed to non-bank counterparties | Domestic or bank-reachable counterparties | LG-5, S5, FATF unhosted-wallet risk |
| 7 | Audit trail for regulated records (health, public sector) | Permissioned ledger | PostgreSQL append-only table, signed digests, offsite WORM copy | Multiple orgs must each hold the log | One custodian with external notarized digest | PDP erasure versus immutability |

See [#37](hash-anchoring-vs-permissioned-ledger-2026-11-23.md) for the criteria-based comparison of examples 1, 2 and 7.

## 9. Data protection stance

[R] No PII, AML/KYC identity, patient or client data, wallet-to-person mapping, or plaintext contract on chain. [R] Wallet metadata and KYC artefacts stay in PostgreSQL under FORCE RLS in a product repository, not in any research fixture. [C] S8 confirms the PDP law is in force; article-level obligations, cross-border transfer rules and retention are UNVERIFIED.

## 10. Unresolved legal questions (open list)

1. [O] Does any AWBMS-derived product that only reads chain data and sends alerts become an intermediary under S1-S3 or the AML rules? (UNVERIFIED)
2. [O] Is a non-custodial orchestration product a regulated "operator" if it routes, relays or monetizes transfers?
3. [O] Current status of PBI 18/40/PBI/2016 and PBI 19/12/PBI/2017, and any 2025-2026 BI regulation on stablecoin or crypto payment. (S5, S6)
4. [O] Which POJK and SEOJK on AML/CFT for this sector apply, and PPATK reporting obligations. (S9)
5. [O] Whether tokenized business claims are securities, digital financial assets or neither. (LG-3)
6. [O] Cross-border data and hosting rules for node providers outside Indonesia.
7. [O] Tax treatment (not researched).
8. [O] Shariah-review requirement if marketed as such (retained from parent note).

## 11. Risk and rollback

- Risk: regulations change (S2 and S3 appeared within eight months of S1); mitigation is dated evidence and refusal to hold funds.
- Risk: a reader treats this as legal advice; mitigation is the explicit statements above.
- **Rollback:** mark the concept **deferred**; delete or supersede this note. Nothing else depends on it.

## 12. Not performed

Counsel review; reading of PDF article texts; PPATK, BI and JDIH primary retrieval beyond §2; regulator consultation; tax analysis; any implementation.

## Acceptance criteria trace

| Issue #35 acceptance criterion | Where addressed | Status |
| --- | --- | --- |
| Matrix of service/actor/assets/custody/payment/trading vs licensing/AML/KYC/data privacy with primary sources | §4 matrix, §5, source status in §2 | Done as research; article-level sources UNVERIFIED and flagged |
| Confirm POJK 27/2024 amended by 23/2025 and PADK 3/2026; FATF 2026 VASP status; BI Rupiah/payment boundary | §2 (S1-S6), §3 | POJK 27/23, PADK 3/2026 VERIFIED (pages); FATF PARTIAL; BI current position UNVERIFIED |
| Explicit legal-counsel and owner decision gates for custody, derivatives, tokenization, exchange, payments, public offering | §7 (LG-1 to LG-6) | Done; no gate discharged |
| At least 5 business examples each compared with a PostgreSQL alternative | §8 (7 examples) | Done |
| No unsupported claim that non-custodial or read-only exempts the business | §6, §4 rows, §10 | Done; claim explicitly rejected |
| DoD: dated evidence matrix, unresolved legal questions, no automatic approval; counsel review not simulated | Document control, §10, §12 | Done |

## Changelog
- **v1.0.0 (2026-11-23):** Initial evidence matrix for issue #35.
