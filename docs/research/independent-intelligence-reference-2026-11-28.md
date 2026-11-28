# AWBMS — Independent Intelligence Capability Research and IP Guardrails

## Document control

- **Version:** v1.3.0
- **Updated:** 2026-11-28
- **Status:** RESEARCH / PROPOSED. Not an ADR, verification gate or requirement. No product code, no approved dependency, no Gate 9 or Gate 15 decision.
- **Tracker:** [#70](https://github.com/ahliweb/awbms/issues/70) (stays open until its acceptance criteria pass).
- **Evidence snapshot:** AWBMS `main` commit `264ba70dd57e5318ce0bfac5836edadc930a3177`; blobs `AGENTS.md` `5f2e322579f7` and `README.md` `24d00f6b6cde` on that commit. A full audit of code, migrations, issue status and CI against this note has **not** been done.
- **Scope:** documentation only. No dependency, CI or contract change. No test was run for this document.

All capability IDs (`IND-01`…`IND-06`), controls, thresholds and benchmark ideas below are **proposed and provisional**. They are not requirements and not rules in force.

## 1. Origin, evidence and intellectual property

This research was prompted by the public description of a commercial product family in the area of data fusion and security assessment. That description is vendor self-report: it was not independently tested or certified, and this note makes no assessment of it. The note uses only general functional concepts and open standards.

Guardrails (proposed):

- No code, assets, API, text, rules, templates, indicators, datasets, brand, SDK, UI or structure of any third-party product; no reverse engineering; no use of a third-party product as an oracle; no unauthorised testing of targets.
- Original requirements, specification, design and implementation only, written from public standards and general concepts.
- The Indonesian [copyright law](https://peraturan.bpk.go.id/Details/38690/uu-no-28-tahun-2014) is the starting point for separating protected expression from ideas and methods; the [WIPO Lex entry](https://www.wipo.int/wipolex/en/legislation/details/15600) lists it. Which articles apply was **not verified at takeover**, so no article number is stated. Patent, trademark, trade secret and contract review are separately required.
- No dependency is introduced. Any proposed dependency, model or data source needs version and digest, SPDX licence, commercial and hosted-service rights, notice, transitive SBOM, owner approval and an exit plan before use.
- Do not reproduce the text of licensed standards. Record author provenance and attestation, a source register, an independent requirement author and reviewer, similarity and dependency checks. This is not a formal clean-room claim.

## 2. Repository disposition (proposed)

Research only. A Rust/Axum/Tokio/SQLx/PostgreSQL specification and verification spike may later be planned; this note is not a product application and makes no production-readiness claim. AWCMS remains the approved baseline and there is no transfer of authority.

## 3. Controls, validation and evidence (proposed)

- Benchmark methodology against an equivalent AWCMS workload: p95/p99 latency, throughput, memory, RLS negative cases, durability, failure recovery and cost. Baselines and thresholds are hypotheses until measured.
- Trace requirement and threat → control → versioned contract → reproducible synthetic test → evidence → procedure and monitoring.
- Least privilege; default-deny RBAC/ABAC and FORCE RLS where applicable; no client-supplied tenant trust; no secrets or personal data in logs; audited approvals and recovery.
- No live vendor, customer or public target is tested. Verify purpose, retention, lawful basis and controller/processor roles. No cross-tenant sharing without explicit scoped grants.
- No new product code, graph database, blockchain or migration before the applicable gates.

## 4. Proposed capabilities

Provisional identifiers, not registered in the architecture validation:

| ID | Capability |
| --- | --- |
| `IND-01` | TLS posture observation |
| `IND-02` | Host metadata observation |
| `IND-03` | Normalised event and correlation |
| `IND-04` | Integrity evidence |
| `IND-05` | Mobile posture |
| `IND-06` | Risk, evidence and cases |

No full scanner, SIEM, XDR or forensic parity is intended. Observed TLS 1.2 is not a compliance verdict; unknown is never pass. Use approved TLS runtime APIs, not new cryptography. Explicit, informed, non-coerced consent of the data or device owner remains required for any observation.

### Proposed guardrails

- Explicit asset scope and authorisation, expiry, revocation and recheck; tenant isolation.
- Deny unauthorised destinations, redirects, SSRF and private targets; bounded job budgets and a stop switch.
- No intrusive or destructive scans, arbitrary templates or shell, or credential harvesting. Synthetic fixtures only.
- Restricted evidence with provenance, hash, redaction, retention and deletion; finding lifecycle, risk acceptance and retest; incident notification only with human approval.
- No secrets or private findings in public artifacts or issue trackers. AI output is a draft only, with no self-approval; prompt-injection, egress and tenant-negative tests.
- Severity scores are not risk. Store source, time and version for every score; stale and unknown are distinct from zero; a human decides with asset impact, exposure and controls. No proprietary or uncalibrated production scoring formula. Validate coverage, efficiency, precision, recall and calibration on time-separated authorised or synthetic data against a severity-only baseline.

### Local work (proposed)

Research the workload design only: compare an equivalent AWCMS security finding and evidence ingestion and a bounded event workload using synthetic observation and event records, bounded windows and deduplication; measure throughput, p95/p99, isolation, resource use, RPO/RTO and cost. No scanner, SIEM or agent runtime and no cryptographic library in AWBMS; no product code, dependency admission or authority transfer. A Rust spike would not establish readiness.

### Reference spike (outside this repository)

A local offline spike (Node v22.16.0, no packages, no network, synthetic observations) was reported by the drafting session as passing 19 of 19 synthetic unit tests for scope, TLS posture decision, event counts, digest comparison and consent. It was **not run in this repository and not re-run at takeover**, and it is not evidence of implementation, of Bun compatibility or of any TLS, device, database or CI behaviour.

## 5. Candidate open-source licence register

Licences were **observed on the default branch of each upstream repository on 2026-11-28; this is not a pinned release** and not an approval. No tool is installed, linked or approved.

| Tool | Observed licence | Note |
| --- | --- | --- |
| [ZAP](https://github.com/zaproxy/zaproxy/blob/main/LICENSE) | Apache-2.0 | Unapproved candidate |
| [Nuclei](https://github.com/projectdiscovery/nuclei/blob/dev/LICENSE.md) | MIT | Unapproved candidate |
| [Nuclei templates](https://github.com/projectdiscovery/nuclei-templates/blob/main/LICENSE.md) | MIT | Separate provenance, signature and safety review |
| [testssl.sh](https://github.com/testssl/testssl.sh) | GPL-2.0 | Not used in this initiative (see below) |
| [Wazuh](https://github.com/wazuh/wazuh/blob/main/README.md) | Engine AGPL-3.0-or-later; selected shared modules GPL-2.0-only OR AGPL-3.0-or-later; remainder GPLv2 | Not used in this initiative |
| [MVT](https://github.com/mvt-project/mvt/blob/main/LICENSE) | MVT License 1.1, adapted from MPL 2.0, with a consensual-use restriction (Sec. 3.0) that is a condition of the licence grant (Sec. 2.7) | Not used in this initiative |

Proposed approach: build original bounded capabilities from public standards rather than install, link, port, translate or copy these tools or their rules, templates or indicators. This is a design preference, not avoidance of lawful obligations; GPL and AGPL permit commercial use under their terms. Any later adoption needs pinned tag and digest, SBOM, notices, commercial and hosted-service rights, feed and plugin terms, integration and exit plan; running a tool in a separate process does not remove its obligations.

## 6. Standards and professional references

Status of each reference at takeover is in §8.

- RFC [9846](https://www.rfc-editor.org/info/rfc9846/) (July 2026) obsoletes RFC 8446. [BCP 195](https://www.rfc-editor.org/info/bcp195/) comprises RFC 8996, 9325 and 9852; RFC 9325 is also updated by RFC [10015](https://www.rfc-editor.org/rfc/rfc10015.html), which is outside BCP 195. [RFC 9525](https://www.rfc-editor.org/rfc/rfc9525.html) covers service identity. [NIST SP 800-52 Rev. 2](https://csrc.nist.gov/pubs/sp/800/52/r2/final) (August 2019, revision planned).
- Logging and monitoring: [NIST SP 800-92](https://csrc.nist.gov/pubs/sp/800/92/final) (final 2006; a Rev. 1 draft is in progress), [SP 800-137](https://csrc.nist.gov/pubs/sp/800/137/final) (September 2011), [RFC 5424](https://www.rfc-editor.org/info/rfc5424/), the [OpenTelemetry logs data model](https://opentelemetry.io/docs/specs/otel/logs/data-model/) (reference only).
- [NIST SP 800-53](https://csrc.nist.gov/pubs/sp/800/53/r5/upd1/final) Rev. 5, Release 5.2.0 (27 August 2025) for AU-6, SI-7 and SI-4 as mapping targets.
- Mobile and forensics scope: [SP 800-124 Rev. 2](https://csrc.nist.gov/pubs/sp/800/124/r2/final) (May 2023); [SP 800-101 Rev. 1](https://csrc.nist.gov/pubs/sp/800/101/r1/final) (May 2014) as a forensic scope boundary, not acquisition instructions; [SP 800-115](https://csrc.nist.gov/pubs/sp/800/115/final) (September 2008).
- Incident response and software supply: [SP 800-61 Rev. 3](https://csrc.nist.gov/pubs/sp/800/61/r3/final) (final April 2025); [SSDF SP 800-218](https://csrc.nist.gov/pubs/sp/800/218/final) v1.1 (final February 2022) and its [Rev. 1 initial public draft](https://csrc.nist.gov/pubs/sp/800/218/r1/ipd) (17 December 2025); [SP 800-218A](https://csrc.nist.gov/pubs/sp/800/218/a/final) (final July 2024); [CSF 2.0](https://www.nist.gov/cyberframework); [NIST AI RMF 1.0](https://doi.org/10.6028/NIST.AI.100-1) (not verified at takeover).
- [OASIS STIX 2.1](https://docs.oasis-open.org/cti/stix/v2.1/os/stix-v2.1-os.html) (OASIS Standard, 10 June 2021); [W3C PROV-O](https://www.w3.org/TR/prov-o/) (Recommendation, 30 April 2013); [OWASP ASVS](https://owasp.org/projects/asvs) 5.0.0 (stable; release date not stated on the page); [WCAG 2.2](https://www.w3.org/TR/WCAG22/) (Recommendation, 12 December 2024).
- Vulnerability prioritisation: [CVSS v4.0 FAQ](https://www.first.org/cvss/faq) ("base scores are not risk"); [EPSS](https://www.first.org/epss/) (probability of exploitation in the wild in the next 30 days, not probability of asset compromise) and its [research page](https://www.first.org/epss/research); CISA [Known Exploited Vulnerabilities catalog](https://www.cisa.gov/known-exploited-vulnerabilities-catalog) and [SSVC](https://www.cisa.gov/ssvc).
- Academic: [Lewis et al., NeurIPS 2020, retrieval-augmented generation](https://papers.nips.cc/paper/2020/hash/6b493230205f780e1bc26945df7481e5-Abstract.html). **Not verified at takeover:** [Hall and Llinas, data fusion](https://doi.org/10.1109/5.554205), [Denning, intrusion detection](https://doi.org/10.1109/TSE.1987.232894), [Jacobs et al., prediction horizon of exploit scoring](https://doi.org/10.1145/3436242). No year or finding from these three is relied on here.
- ISO catalog **(not verified at takeover; edition years are not stated)**: ISO/IEC 27001, 27002, 27005, 27017, 27018, 27034-1, 27035-1 and -2, 27036-3, 27037, 27701, 15408-1, 20000-1, 42001, 23894, 25010 and ISO 22301, as candidate control and evidence mapping targets. Catalog entry: [ISO/IEC 27000 family](https://www.iso.org/standard/iso-iec-27000-family). No certification, compliance or clause audit is claimed.
- Indonesian law pages that exist at takeover: [UU PDP](https://peraturan.bpk.go.id/Details/229798/uu-no-27-tahun-2022), [PP 71/2019](https://jdih.komdigi.go.id/produk_hukum/view/id/695/t/common), [Perpres 82/2022](https://peraturan.bpk.go.id/Details/211029/perpres-no-82-tahun-2022). **Not verified at takeover:** the text of UU PDP Article 46 and any notification period; an unattributed claim about a 2026 regulation (PP 33/2026) is unverified and must not be hard-coded. Any deadline must be configurable and legally reviewed; see ADR-0010 for how this repository treats data-subject deadlines. Review PSE, lawful basis, DPIA, transfer, sector rules and the electronic information and transactions law separately.

## 7. Next gate

Follow the 18-step product development flow adopted by ADR-0004. Discovery and UX, privacy and threat model, data and authorisation, algorithm and contract specifications come before Gate 9; only the research issue is authorised now. Repository-specific CI, index, changeset and project-board coordination are validated during review. Consumer UAT and Gate 15 come before any production use. Rollback is removal of this document (no runtime effect); future features need tested recovery, rollback and monitoring. No CI or test execution is claimed for this document.

## 8. Reference verification at takeover (2026-11-28)

References were opened read-only on 2026-11-28; a page that could not be opened is not treated as correct.

- **Supported as stated:** RFC 9846, BCP 195 and RFC 9852, RFC 9525, SP 800-61r3, SP 800-218 v1.1, its Rev. 1 draft and 800-218A, SP 800-92 and its draft status, SP 800-124r2, SP 800-52r2, SP 800-137, SP 800-115, SP 800-101r1, ASVS 5.0.0, EPSS, CVSS v4.0 FAQ, STIX 2.1, PROV-O, Lewis et al., WCAG 2.2, RFC 5424, OpenTelemetry logs data model, CISA KEV and SSVC, CSF 2.0 (page exists, no date stated), the licence pages in §5, and the existence of the three Indonesian law pages and the WIPO Lex entry.
- **Corrected from the draft:**
  - RFC 10015 updates RFC 9325 but is not a member of BCP 195.
  - SP 800-53 is cited as Rev. 5, Release 5.2.0 (27 August 2025); the page describes Rev. 5 and names 5.2.0 as the latest release.
  - Licence statements are marked as observed on the default branch, not a pinned release.
  - All statements about a commercial vendor's prices, service hours, product catalogue and tooling were removed because they were either not reproducible from the pages or are not needed for this note.
  - The synthetic-test claim is restated as a spike outside this repository.
  - Calling a vendor's own claims "verified" was removed; nothing about vendor claims is asserted here.
- **Not verified at takeover:** the ISO catalog and all edition years (access refused), UU PDP Article 46 (text only in PDF), copyright law article numbers, Hall and Llinas, Denning, Jacobs et al. (DOI redirects gave no content), NIST AI RMF 1.0 (content not confirmed), and the "ISO 27000 family" catalog page (not opened).
- No reference was added.

## Changelog

### 2026-11-28 - v1.3.0

- Takeover: adopted the repository's document-control layout, replaced vendor-specific content with a neutral origin statement, corrected the TLS and SP 800-53 references, marked unverifiable references and the external spike, labelled every item proposed, added §8, the index entry and a changeset.

### 2026-11-28 - v1.2.0

- Independent reconstruction replacing unapproved components; proposed local acceptance.

### 2026-11-28 - v1.1.0

- Follow-up research, local acceptance and licence register.

### 2026-11-28 - v1.0.0

- Initial research, local ownership and controls.
