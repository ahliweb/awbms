# AWBMS Privacy Analysis

| Field | Value |
|---|---|
| Status | **Draft — awaiting human privacy/legal review.** Prepared by an AI agent. It gives no legal conclusion and is not legal advice |
| Date | 2026-11-23 |
| Lifecycle step | Step 3 — Threat Model awal + Privacy Analysis (lifecycle v2, [ADR-0004](../adr/0004-adopt-product-development-flow-v2.md)). This document is the **privacy half**; the STRIDE threat model is a separate document |
| Inputs | [AWBMS PRD](../product/AWBMS-PRD.md) (accepted by the owner: every proposed value `PD-01`…`PD-22` accepted as proposed on 2026-11-23); [AWBMS User Research](../product/AWBMS-USER-RESEARCH.md); [Master Blueprint](../architecture/AWBMS-MASTER-BLUEPRINT.md) §8, §12, §16, §19, §20; [Architecture Validation](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md) registers; the frozen AWCMS corpus under `contracts/legacy/awcms/frozen/` |
| Legal-basis scope | UU No. 27 Tahun 2022 tentang Pelindungan Data Pribadi ("UU PDP"); PP No. 71 Tahun 2019 tentang Penyelenggaraan Sistem dan Transaksi Elektronik ("PP PSTE") where relevant. Control alignment only — **no claim of regulatory compliance** (Blueprint §20; PRD §11) |
| Citation rule | Article numbers are given only where the author is reasonably sure, and **every** one is marked `[O] verify with counsel`: the primary text was not consulted in this session. Where unsure, the obligation is described without a number and flagged |
| Implementation | **Blocked until gate 9.** This document specifies and analyses; it authorises no code and decides nothing that Stage 1 or the PRD already decided |

## Contents

1. [Scope, method and limits](#1-scope-method-and-limits)
2. [Legal reference frame](#2-legal-reference-frame)
3. [Evidence base from the frozen legacy corpus](#3-evidence-base-from-the-frozen-legacy-corpus)
4. [Personal-data inventory](#4-personal-data-inventory)
5. [Data classification scheme](#5-data-classification-scheme)
6. [Retention schedule](#6-retention-schedule)
7. [Data protection impact assessment](#7-data-protection-impact-assessment)
8. [Data-subject rights procedure](#8-data-subject-rights-procedure)
9. [Gaps, proposed requirements and open questions](#9-gaps-proposed-requirements-and-open-questions)

## 1. Scope, method and limits

**Claim labels** are those of [validation §0.2](../architecture/AWBMS-RUST-ARCHITECTURE-VALIDATION.md#02-claim-labels): **[F]** fact checked against a frozen artefact in this repository, **[C]** recorded external claim, **[A]** assumption, **[D]** existing decision (`AD-`, `OD-`, ADR), **[R]** recommendation of this document, **[O]** open question, **[G]** gate reference (not a passed gate). `[O] verify with counsel` marks anything a lawyer must confirm.

In scope: the sixteen v1 capabilities `CAP-01`…`CAP-16` ([PRD §4](../product/AWBMS-PRD.md)), the migration of their data from AWCMS (wave 4), backups, and the processors they depend on.

Out of scope: legal conclusions; screen design; the column-level data model (step 5, `TBL-`); the authorisation matrix (step 6); API contracts (step 8); the STRIDE threat model.

**What this document is not.** It is not a record of processing activities, a privacy notice, a data-processing agreement, or a finished DPIA. It is the input from which counsel and the privacy officer (`P-07`) can produce those.

**Method.** (1) Read the PRD requirements that touch personal data. (2) Inventory what the AWCMS capabilities hold today from table and column names in the frozen corpus, because AWBMS migrates that data ([PRD §9](../product/AWBMS-PRD.md)). (3) Classify, set retention, assess risk, trace measures to existing IDs, and record what the PRD does not yet require as `PPRQ-` rows.

**Epistemic limit [F].** The frozen corpus contains **table names for all 152 AWCMS tables and column definitions for only 16 frozen SQL files** (identity, sessions, MFA, OIDC, tenant domain, comments, password reset, machine credentials, handoff, push, invitations, principals, tenant selection, delegated access, theming, newsletter). **No row data exists in the corpus and none is invented here.** For every other table the inventory states what the table name suggests and marks the column content `[O]`. The frozen files are as first created; later migrations that altered those tables were not frozen `[O]`.

## 2. Legal reference frame

This section is a map for counsel, not an interpretation. All article numbers: `[O] verify with counsel`.

| Topic | Understanding of the obligation | Citation (unverified) | Used in |
|---|---|---|---|
| General versus specific personal data | UU PDP distinguishes general personal data from specific personal data (understood to include health, biometric, genetic, criminal-record, children's and personal financial data) with stricter treatment for the latter | UU PDP Art. 4 `[O] verify with counsel` | [§4](#4-personal-data-inventory) |
| Lawful bases | Processing needs a lawful basis; the Act lists explicit consent, contract performance, legal obligation, protection of vital interests, public-interest task and legitimate interest | UU PDP Art. 20 `[O] verify with counsel` | [§4](#4-personal-data-inventory) |
| Data-subject rights | Information, correction, access and copy, ending processing or erasure, withdrawal of consent, objection to automated decisions, delay or restriction, redress, and portability | UU PDP Arts. 5–14 `[O] verify with counsel` (individual numbers not asserted) | [§8](#8-data-subject-rights-procedure) |
| Response periods for rights requests | The Act (and any implementing regulation) is understood to set response periods for some requests. **They may be shorter than the 14 days of `PD-12`.** `[C]` Secondary legal commentary checked by the orchestrator on 2026-11-23 describes **3 × 24 hours** for several rights — stopping processing after consent is withdrawn, correcting or updating data, giving access, and suspending or restricting processing ([Hukumonline](https://www.hukumonline.com/klinik/a/kewajiban-penting-dalam-uu-pdp-dan-sanksinya-jika-melanggar-lt633abd26570aa)) — not a single period for every request; the primary text was not read | `[O] verify with counsel` | DPR-15, `OQP-02` |
| Records of processing | A controller is understood to be required to keep records of its processing | UU PDP Art. 31 `[O] verify with counsel` | [§4](#4-personal-data-inventory) |
| Impact assessment | A data-protection impact assessment is understood to be required for processing with high risk to data subjects | UU PDP Art. 34 `[O] verify with counsel` | [§7](#7-data-protection-impact-assessment) |
| Breach notification | The Act is understood to require written notification of a data-protection failure to the data subject and to the supervisory authority within **3 × 24 hours** of discovery. The author is fairly, not fully, confident | UU PDP Art. 46 `[O] verify with counsel`; whether the 3 × 24 hours runs from discovery by the controller or by the processor is `[O]` | DPR-17 |
| Processor duties | A processor acts on the controller's instructions and carries its own security and notification duties; the contract content is prescribed | Article numbers not asserted `[O]` | DPR-18, DPR-20 |
| Cross-border transfer | Transfer out of Indonesia is understood to require protection in the destination that is equivalent or adequate, or binding safeguards, or consent | UU PDP Art. 56 `[O] verify with counsel` | DPR-18 |
| Data protection officer | Appointment is understood to be required for certain kinds of processing (public service, large-scale monitoring, large-scale specific data) | UU PDP Art. 53 `[O] verify with counsel` | `OQP-08` |
| Electronic system operator duties | PP PSTE sets duties for electronic system operators: security, personal-data principles, deletion of information no longer relevant, incident reporting, and registration of private operators. Numbers not asserted | `[O] verify with counsel` | DPR-17, `OQP-11` |
| Implementing regulation and authority | Whether the government regulation implementing UU PDP and the supervisory authority exist, and what they require, was not checked (author knowledge ends mid-2026) | `[O] verify with counsel` | `OQP-02`, `OQP-03` |

## 3. Evidence base from the frozen legacy corpus

All of this is **[F]** and sourced from `contracts/legacy/awcms/frozen/`. It shows what AWCMS holds, which is what migration must carry or leave behind.

| Fact | Evidence |
|---|---|
| 152 AWCMS tables, 134 with RLS, all 134 `FORCE` | `live/summary.json`, `live/tables-rls.csv` |
| Password storage is Argon2id (`m` = 65536 KiB, `t` = 2, `p` = 1), in `awcms_identities.password_hash` and `awcms_principals.password_hash`; no plaintext-password column exists; no rehash-on-login code exists | `static/credential-formats.json` |
| 26 credential-like columns: 19 hash, 3 encrypted, 1 environment-variable name, 3 plaintext candidates (`awcms_oidc_auth_requests.nonce`, `.code_verifier`; `awcms_push_subscriptions.auth_secret`) | same |
| Encrypted secrets: MFA seed (`secret_ciphertext`, AES-256-GCM, key from an environment variable) in `awcms_identity_mfa_factors` and `awcms_principal_mfa_factors`; OIDC `client_secret_ciphertext` in `awcms_auth_providers` | same |
| `awcms_sessions` as created holds `token_hash`, `issued_at`, `expires_at`, `revoked_at` and **no network-address or user-agent column** | `sql/004` |
| `awcms_identities` holds `login_identifier`, `failed_login_count`, `locked_until`, `last_login_at` | `sql/004` |
| `awcms_newsletter_subscribers` holds `email` (as typed), `email_normalized`, `status`, `source`, `locale`, `confirmation_token_hash`, `confirmation_sent_at`, `confirmed_at`, `unsubscribe_token_hash`, `consent_at`, **`consent_ip_hash`**, `unsubscribed_at`, `suppressed_at`, `suppression_reason` | `sql/139` |
| Rate limits for analytics and newsletter are keyed `analytics-collect:${client_ip}`, `newsletter:subscribe:${client_ip}`, `newsletter:confirm:${client_ip}`, `newsletter:unsubscribe:${client_ip}`; the 15-minute recipient cooldown is separate (`C-07`) | `static/rate-limits.json` |
| `awcms_invitations` holds `login_identifier`, `display_name`, `invited_by_tenant_user_id`, `revoke_reason` (free text) | `sql/106` |
| `awcms_external_identities` holds `issuer` and `subject` | `sql/025` |
| `awcms_machine_credentials` holds `name`, `token_hash`, `allowed_permission_keys`, `last_used_at`, and staff references `created_by_tenant_user_id`, `revoked_by_tenant_user_id` | `sql/082` |
| Comments (capability remaining in AWCMS) store `author_display_name`, `author_email_hash`, `author_email_masked`, `author_ip_hash`, `user_agent_hash`, and `subscriber_email_encrypted`; abuse events store `ip_hash` | `sql/066` |
| Push subscriptions (remaining in AWCMS) store the raw `endpoint`, key material, `user_agent_summary` | `sql/093` |
| Tables named for personal-data-bearing capabilities whose columns are **not** frozen: `awcms_profiles`, `awcms_profile_identifiers`, `awcms_profile_entity_links`, `awcms_principal_preferences`, `awcms_visit_events`, `awcms_visitor_sessions`, `awcms_visitor_daily_rollups`, `awcms_site_search_query_log`, `awcms_seo_not_found_observations`, `awcms_audit_events`, `awcms_abac_decision_logs`, `awcms_email_messages`, `awcms_email_delivery_attempts`, `awcms_email_suppression_list`, `awcms_subject_requests`, `awcms_data_lifecycle_legal_holds`, `awcms_registration_requests`, `awcms_site_profile` | `live/tables-rls.csv` |

**Observations.** (1) AWCMS already hashes network addresses for newsletter consent and comments. A hash of a network address is still pseudonymous, not anonymous, because the input space is small `[O]`; whether it is acceptable is a counsel question (`OQP-06`). (2) AWCMS has a `awcms_subject_requests` table and a lifecycle module, so a subject-request design exists to port or redesign (`CAP-07`). (3) Whether `awcms_profile_identifiers` can hold a national identity number or similar is unknown `[O]` and must be read from the live schema before step 5 (`OQP-10`).

## 4. Personal-data inventory

### 4.1 Inventory: what, whose, from where, why

Personas are those of the research document. Table names are **[F]** evidence of the AWCMS location; the AWBMS location is decided at step 5 (`TBL-`).

| ID | CAP | Data categories (legacy evidence) | Data subjects | Source | Purpose |
|---|---|---|---|---|---|
| `PDI-01` | `CAP-01` | Tenant owner and administrator contact and identifiers; tenant, partner and status-change records (`awcms_tenants`, `awcms_tenant_settings`, `awcms_partners`, `awcms_partner_managed_tenants`, `awcms_tenant_status_transitions`, `awcms_setup_state`); columns not frozen `[O]` | `P-04`, `P-05` | Operator provisioning (`REQ-003`) | Provision, suspend and reactivate tenants (`REQ-004`) |
| `PDI-02` | `CAP-02`, `CAP-03` | Staff identity and profile: login identifier, normalised email, display name, profile, profile identifiers and entity links, preferences, membership, groups, role and business-scope assignments (`awcms_identities`, `awcms_principals`, `awcms_profiles`, `awcms_profile_identifiers`, `awcms_profile_entity_links`, `awcms_principal_preferences`, `awcms_tenant_users`, `awcms_user_groups`, `awcms_user_group_members`, `awcms_access_assignments`, `awcms_business_scope_assignments`) | `P-03`, `P-04`, `P-05` | Administrator creation or invitation; self-update (`REQ-015`) | Authenticate and authorise staff; show who did what |
| `PDI-03` | `CAP-03` | Authentication secrets bound to a person: password hash, MFA seed ciphertext, recovery-code hashes, token hashes for sessions, resets, invitations, handoff and challenges (26 credential-like columns, §3) | `P-03`, `P-04`, `P-05` | The person (password, MFA enrolment); the system (tokens) | Authenticate; step-up (`REQ-008`) |
| `PDI-04` | `CAP-03` | Session and sign-in state: session issue, expiry and revocation times; failed-attempt count, lock time, last sign-in time (`awcms_sessions`, `awcms_identities`, `awcms_mfa_challenges`, `awcms_session_handoff_codes`) | `P-03`, `P-04` | Generated at sign-in | Session lifecycle, lockout (`REQ-006`, `REQ-007`, `REQ-009`) |
| `PDI-05` | `CAP-03` | Federated identity links: issuer and subject; provider configuration (`awcms_external_identities`, `awcms_auth_providers`, `awcms_oidc_auth_requests`) | `P-03`, `P-04` | Identity provider | Single sign-on where a tenant enables it |
| `PDI-06` | `CAP-03`, `CAP-08` | Invitation, reset and registration flow data: invited identifier and display name, inviter, revoke reason (free text), reset tokens, registration requests, delegated-access grants (`awcms_invitations`, `awcms_password_reset_tokens`, `awcms_registration_requests`, `awcms_delegated_access_grants`) | `P-03`, `P-04` | Administrator; the invitee | Onboard and recover accounts |
| `PDI-07` | `CAP-03` | Machine credentials: name, token hash, scopes, last-used time, creating and revoking staff reference (`awcms_machine_credentials`, `awcms_bff_clients`) — personal data only through staff references | `P-06`, `P-04`, `P-05` | Operator or administrator | Build-time authenticated reads (`REQ-012`) |
| `PDI-08` | `CAP-05` | Audit records: actor, tenant, operation, target, outcome, correlation identifier (`REQ-020`); legacy `awcms_audit_events`, `awcms_access_policy_events`, `awcms_business_scope_assignment_events`; targets may be subjects | `P-03`, `P-04`, `P-05`, `P-02` | Written with each privileged change | Accountability, investigation (`REQ-020`–`REQ-024`) |
| `PDI-09` | `CAP-05` | Authorisation-decision records and operational telemetry: request, correlation and trace identifiers, tenant, operation, status; risk of incidental client address, query string or free text (`awcms_abac_decision_logs`, `awcms_sod_conflict_evaluations`; AWBMS logs, traces, metrics per `AD-20`) | `P-01`–`P-05` | Generated per request | Operations, security, SLOs (`REQ-076`, `REQ-077`) |
| `PDI-10` | `CAP-06` | Event, outbox, job and idempotency payloads: whatever a producing module places in them, for example a recipient address for a confirmation job (`awcms_domain_events`, `awcms_domain_event_deliveries`, `awcms_domain_event_consumer_effects`, `awcms_domain_event_replays`, `awcms_module_jobs`, `awcms_idempotency_keys`) | `P-02`, `P-03`, `P-04` | Producing modules | Reliable delivery (`REQ-033`–`REQ-039`) |
| `PDI-11` | `CAP-07` | Lifecycle ledgers: subject requests (requester, type, status), legal holds (subject reference, reason), run records, archive manifests, and the AWBMS erasure ledger (`REQ-044`) (`awcms_subject_requests`, `awcms_data_lifecycle_legal_holds`, `awcms_data_lifecycle_runs`, `awcms_data_lifecycle_archive_manifests`, `awcms_data_lifecycle_cursors`) | `P-02`, `P-07`, `P-03`, `P-04` | Requester; privacy officer | Fulfil rights, retention, hold (`REQ-041`–`REQ-044`) |
| `PDI-12` | `CAP-08` | Email: recipient address, rendered message content, delivery attempts, provider message identifiers, bounces, complaints, suppression entries (`awcms_email_messages`, `awcms_email_delivery_attempts`, `awcms_email_templates`, `awcms_email_suppression_list`) | `P-02`, `P-03`, `P-04` | Producing modules; provider callbacks | Deliver messages (`REQ-070`, `REQ-071`) |
| `PDI-13` | `CAP-09` | Tenant domain records: hostname, verification method and record, status, check times (`awcms_tenant_domains`); organisation data, personal only if a hostname names a person | `P-04`, `P-06` | Administrator | Tenant resolution (`REQ-045`, `REQ-046`) |
| `PDI-14` | `CAP-10` | Media: object metadata, checksum, uploader reference, and **content** that may depict or record identifiable people, plus embedded metadata such as location `[A]` (`awcms_news_media_objects`, `awcms_media_library_tenant_state`; content in the object store) | `P-03`, third parties in media | Editor upload | Publish media (`REQ-052`–`REQ-054`) |
| `PDI-15` | `CAP-11` | Blog content, revisions, author attribution, institutions, menus, widgets; article text may name third parties (`awcms_blog_posts`, `awcms_blog_revisions`, `awcms_blog_institutions`, `awcms_blog_post_institutions`, `awcms_blog_terms`, `awcms_blog_menus`, `awcms_blog_widgets`) | `P-03`, named third parties | Editor | Publish content (`REQ-055`–`REQ-057`) |
| `PDI-16` | `CAP-12` | Not-found observations: requested paths, counts; paths may carry personal data in the query string `[O]` (`awcms_seo_not_found_observations`, `awcms_seo_redirects`) | `P-01` | Browser request | Repair broken links and redirects |
| `PDI-17` | `CAP-13` | Site search: copies of published content (derived copy), indexing runs, and **visitor query text** (`awcms_site_search_documents`, `awcms_site_search_query_log`, `awcms_site_search_index_runs`, `awcms_site_search_index_failures`) | `P-01` | Anonymous query | Search and zero-result analysis (`MET-24`) |
| `PDI-18` | `CAP-15` | Site profile: organisation name, contact email, phone, address, social links (`awcms_site_profile`); personal when the organisation is an individual | `P-04`, owners of sites | Administrator | Compose public profile (`REQ-060`) |
| `PDI-19` | `CAP-14` | Newsletter subscriber: email as typed and normalised, status, source, locale, double-opt-in tokens (hashed), consent time, `consent_ip_hash`, unsubscribe and suppression times, suppression reason (`awcms_newsletter_subscribers`) | `P-02` | The subscriber, or a third party typing someone else's address | Deliver the newsletter; prove consent; honour opt-out (`REQ-065`–`REQ-071`) |
| `PDI-20` | `CAP-16` | Visitor telemetry: events, sessions, daily roll-ups (`awcms_visit_events`, `awcms_visitor_sessions`, `awcms_visitor_daily_rollups`); columns not frozen `[O]`; AWBMS may not store a raw network address (`REQ-063`) and, per `PD-15`, no persistent identifier across sites | `P-01` | Anonymous browser beacon | Aggregate audience measurement (`REQ-062`–`REQ-064`) |
| `PDI-21` | `PLT` | Abuse-control counters keyed by client address or normalised recipient (legacy keys `${client_ip}`; §3) | `P-01`, `P-02` | Network address of the request | Rate limiting (`REQ-064`, `REQ-066`, `REQ-009`) |
| `PDI-22` | `PLT` | Migration staging copies, reconciliation extracts, and production-like snapshots used for rehearsal (`REQ-078`, `AS-03`, `VG-10`, `VG-12`) — a copy of every category above | All | AWCMS production | Migrate, reconcile, rehearse restore |
| `PDI-23` | `PLT` | Backups and archives of every category above (`REQ-075`, `VG-12`) | All | AWBMS database and object store | Recovery |

Capabilities with no dedicated row: `CAP-04` (module registry) and `CAP-06`'s non-payload data hold no personal data expected `[O]` (module settings could hold contact fields; the module descriptor of `REQ-040` must say). AWCMS capabilities that **remain in AWCMS** (comments, push delivery, form drafts, and the others of [PRD §2](../product/AWBMS-PRD.md)) hold personal data (§3) that this analysis does not inventory further; see DPR-13.

### 4.2 Inventory: basis, sensitivity, location, transfer

*Lawful-basis candidates* are inputs for counsel, not conclusions. The tenant is the likely controller `[O]` (DPR-20); "controller" below means the tenant.

| ID | Lawful-basis candidate | Specific personal data (UU PDP)? | Storage location | Cross-border exposure | Classification (§5) |
|---|---|---|---|---|---|
| `PDI-01` | Contract (tenant service agreement); legitimate interest | No evidence | PostgreSQL: tenant and partner families; backups | Backups location `[O]` | Restricted-personal |
| `PDI-02` | Contract or legitimate interest of the tenant in managing its staff | No evidence in frozen columns; `awcms_profile_identifiers` unknown `[O]` | PostgreSQL: identity, profile, access families; backups; audit copies by reference | Backups `[O]` | Restricted-personal |
| `PDI-03` | Contract; legal duty of security (PP PSTE) `[O]` | No (a TOTP seed is not a biometric) | PostgreSQL identity family; key material in AWBMS key management (`REQ-084`) | Key store location `[O]` | Secret-credential |
| `PDI-04` | Legitimate interest (security); contract | No | PostgreSQL; cache only as a derived copy (`AD-18`) | Backups `[O]` | Restricted-personal |
| `PDI-05` | Contract; consent at the identity provider | No | PostgreSQL identity family | The external identity provider is its own controller `[O]` | Restricted-personal (links); Secret-credential (provider secret) |
| `PDI-06` | Contract; legitimate interest | Free-text `revoke_reason` could hold any content; none expected | PostgreSQL; email provider for the message | Email provider `[O]` | Restricted-personal |
| `PDI-07` | Legitimate interest (integration security) | No | PostgreSQL; hash only | None beyond backups | Secret-credential |
| `PDI-08` | Legal obligation or legitimate interest (accountability) `[O]`; retention basis is `PD-11` | None expected; free text must be prohibited (`PPRQ-11`) | PostgreSQL audit store, separate from logs (`REQ-023`); backups | Backups `[O]` | Restricted-personal |
| `PDI-09` | Legitimate interest (security, reliability) | None expected; redaction required (`REQ-027`) | Log and trace store; metric store | **Observability backend location `[O]`** | Internal, with Restricted-personal when incidental (must be redacted) |
| `PDI-10` | Same as the producing purpose | Depends on payload | PostgreSQL outbox and job tables | Backups `[O]` | Restricted-personal (payloads) |
| `PDI-11` | Legal obligation (to answer requests) `[O]` | Held-subject reference only; reason free text must not hold specific data | PostgreSQL lifecycle family; ledger copy that survives restore | Backups `[O]` | Restricted-personal |
| `PDI-12` | Same as the sending purpose (consent for newsletter) | Message body could include any content; none expected | PostgreSQL; **email provider** | **Email provider (processor) `[O]`** | Restricted-personal |
| `PDI-13` | Legitimate interest | No | PostgreSQL | None beyond DNS lookups `[O]` | Internal |
| `PDI-14` | Legitimate interest or consent, depending on subject; image or voice of identifiable people can be personal data | **Possible by content** (biometric if used for identification; children's images) — not evidenced | Object store (**Cloudflare R2**, `AD-17`); PostgreSQL metadata | **Cloudflare R2 region and jurisdiction `[O]`** | Public once published; Confidential before; Restricted-personal when it depicts people |
| `PDI-15` | Contract (editor); legitimate interest, or journalistic and expression bases for third-party content `[O]` | Possible by content | PostgreSQL | CDN edge cache `[O]` | Public once published; Confidential before |
| `PDI-16` | Legitimate interest | No | PostgreSQL | None beyond backups | Internal; Restricted-personal if paths carry personal data |
| `PDI-17` | Legitimate interest; query text could be personal data typed by the visitor | Possible by content (a visitor can type anything) | PostgreSQL search store (`OD-07` open) | Backups `[O]` | Public (documents); Restricted-personal (query log) |
| `PDI-18` | Consent or legitimate interest of the site owner | No | PostgreSQL | CDN edge cache `[O]` | Public once published |
| `PDI-19` | **Explicit consent** (double opt-in), with a legal-obligation basis for retaining an opt-out record `[O]` | No evidence; subscribers could be minors `[O]` (`OQP-12`) | PostgreSQL newsletter family; **email provider** | **Email provider `[O]`** | Restricted-personal |
| `PDI-20` | Consent or legitimate interest; undecided `[O]` (`OQP-05`); tenant's notice to visitors sits outside AWBMS | No | PostgreSQL analytics family | **Cloudflare edge sees the client address in transit `[O]`** | Restricted-personal until proven non-identifying; roll-ups without identifiers Internal |
| `PDI-21` | Legitimate interest (abuse prevention) | No | Shared counter store, ephemeral (`AD-18` allows no correctness dependency) | Edge `[O]` | Restricted-personal |
| `PDI-22` | Compatible-purpose continuation of the original basis `[O]` (`OQP-09`) | As the source data | Migration workspace; rehearsal environment | Depends on where staging runs `[O]` | Highest classification of any data it copies |
| `PDI-23` | Continuity of the original basis | As the source data | Backup store in a separate failure domain (`REQ-075`) | **Backup location `[O]`** | Highest classification of any data it copies |

**Finding [F]/[R].** No specific personal data is evidenced in the frozen corpus. Specific data could still enter through free text (`revoke_reason`, `suppression_reason`, search queries, comment bodies, article text) and through media content. The control is a prohibition and a descriptor declaration (`PPRQ-20`, `PPRQ-21`), not an assumption that none exists.

## 5. Data classification scheme

Five levels. A column or file takes the **highest** level of any datum it can hold. Per-column classification is completed at step 5 (`TBL-`); the levels below are the vocabulary and the `PDI-` rows above are the starting point.

| Level | Meaning | Encryption in transit | Encryption at rest `[R]` | Logging and telemetry | Masking | Access |
|---|---|---|---|---|---|---|
| **Public** | Published to anonymous readers | TLS | Not required | Allowed | None | Anyone, through a published route |
| **Internal** | Non-personal operational or tenant-business metadata | TLS | Storage-level | Allowed without identifiers | None | Operators and tenant staff by permission |
| **Confidential** | Tenant business data not published (drafts, settings, unpublished media) | TLS | Storage-level | Identifiers only, never content | Content omitted in telemetry | Tenant staff by permission; operators by audited access |
| **Restricted-personal** | Personal data of identifiable people | TLS | Storage-level for the database, backups and object store | **Prohibited** in logs, traces and metrics (`REQ-027`); canary-tested (`MET-20`) | Mask in operator views (`j***@e***` style); show full value only where the task needs it | Least privilege, tenant-bound (`REQ-001`); every read of bulk personal data audited `[R]` |
| **Secret-credential** | Anything that grants access if disclosed or recoverable | TLS | One-way hash where recovery is never needed (`AD-11`, `AD-12`); authenticated encryption under AWBMS key management where it must be recovered (MFA seed; provider secrets; `REQ-084`) | **Prohibited** everywhere, including error messages | Never displayed after issuance (`REQ-005`, `REQ-012`) | No human read path; verification code only |

**Overlay: specific personal data** (UU PDP Art. 4 `[O] verify with counsel`). Data in any level that is health, biometric, genetic, criminal-record, children's or personal financial data takes Restricted-personal handling **plus**: purpose declared in the module descriptor, a DPIA before admission (`PPRQ-20`), and no use in analytics. No v1 capability is expected to declare it.

**Rules common to all levels.** Tenant-bound access (`REQ-001`, `AD-10`); no unnecessary personal data in logs, traces or metrics (`REQ-027`, `AD-20`); non-production environments never receive production personal data unmasked (`PPRQ-12`); classification travels with derived copies (search index, roll-ups, event payloads, backups), because `REQ-041` extends erasure to derived copies.

Classification of each `PDI-` row is in the last column of [§4.2](#42-inventory-basis-sensitivity-location-transfer).

## 6. Retention schedule

Periods marked **PD-11** are the owner's accepted values of 2026-11-23 and remain **subject to legal review** (`OQP-07`). Periods marked **(owner)** are **gaps**: `PD-11` did not cover the category, and the value is this document's proposal `[R]`, not a decision. `PD-12`: a verified subject export or erasure within 14 days (`REQ-043`), pending legal advice (`OQP-02`).

Legend. **Hold**: `H1` = a legal hold suspends disposal and is audited (`REQ-042`); `H2` = the datum is itself the hold record. **Backup**: `B1` = erasure must not return on restore, so the erasure ledger is re-applied before serving traffic or the gap is recorded with an owner (`REQ-044`); `B2` = no copy outside the live system; `B3` = the datum is the ledger and must outlive the oldest backup that can contain what it erased.

| Category | PDI | Period | Basis | Trigger | Disposal | Hold / Backup |
|---|---|---|---|---|---|---|
| Visitor analytics, raw events | `PDI-20` | **90 days** | PD-11 (`REQ-041` scenario) | Event time | Delete, including derived aggregates that contain identifiers | H1 / B1 |
| Visitor analytics, aggregates with no identifier | `PDI-20` | 24 months | (owner) gap | Roll-up date | Delete | H1 / B1 |
| Operational logs, traces | `PDI-09` | **30 days** | PD-11 | Log time | Delete in the log store | H1 not expected / B2 (not backed up) |
| Audit records | `PDI-08` | **2 years** | PD-11 | Record time | Delete the row; **never edited** (`REQ-021`); actor identity severed by removing the mapping (`PPRQ-11`) | H1 / B1 |
| Authorisation-decision records | `PDI-09` | 30 days | (owner) gap; aligns with logs | Decision time | Delete | H1 / B1 |
| Newsletter subscriber, active | `PDI-19` | While subscribed | (owner) gap | Unsubscribe, erasure request, tenant offboarding | Delete identifying fields; keep suppression entry (next row) | H1 / B1 |
| Newsletter subscriber, unconfirmed | `PDI-19` | 7 days from the confirmation message | (owner) gap; matches the 7-day example of `REQ-069` | Confirmation expiry | Delete the row | H1 / B1 |
| Newsletter opt-out and suppression entry | `PDI-19`, `PDI-12` | 3 years after unsubscribe, bounce or complaint | (owner) gap; counsel to set | Unsubscribe, hard bounce, complaint | Keep only a keyed hash of the normalised address; then delete | H1 / B1, and B3 for the entry |
| Consent proof (time, text version, source) | `PDI-19` | Subscription life plus 12 months | (owner) gap; counsel to set | End of subscription | Delete; the network-address hash is not retained (`PPRQ-06`) | H1 / B1 |
| Email delivery attempts and message bodies | `PDI-12` | 30 days (attempts); bodies not retained beyond send | (owner) gap | Send time | Delete | H1 / B1 |
| Staff account and profile | `PDI-02` | While a membership exists; 30 days after removal | (owner) gap | Membership removal | Delete or anonymise; audit rows keep only the opaque identifier | H1 / B1 |
| Authentication secrets | `PDI-03` | While the factor or credential exists | (owner) gap | Factor removal, account removal | Delete | H1 not applicable / B1 |
| Sessions, lockout, reset and invitation tokens | `PDI-04`, `PDI-06` | Expiry plus 30 days | (owner) gap | Expiry, use, revocation | Delete | H1 not applicable / B2 |
| Federated identity links | `PDI-05` | While the account exists | (owner) gap | Account removal or unlink | Delete | H1 / B1 |
| Machine credentials | `PDI-07` | Expiry or revocation plus 90 days (credential lifetime ≤ 365 days, `REQ-012`) | (owner) gap | Expiry, revocation | Delete the hash and metadata; the audit record remains | H1 / B1 |
| Idempotency keys | `PDI-10` | **24 hours** | PRD `REQ-039` (owner) | Key creation | Delete | H1 not applicable / B2 |
| Dispatched events, completed jobs | `PDI-10` | 30 days | (owner) gap; aligns with logs | Dispatch or completion | Delete payload | H1 / B1 |
| Dead-letter items | `PDI-10` | 30 days after resolution or replay | (owner) gap | Resolution | Delete payload | H1 / B1 |
| Subject-request records | `PDI-11` | 2 years after closure | (owner) gap; evidence of fulfilment | Closure | Delete | H1 / B1 |
| Erasure ledger (what was erased, key hash, date) | `PDI-11` | Oldest backup retention plus 30 days | (owner) gap | Erasure time | Delete when no backup can still hold the erased data | H1 / **B3** |
| Legal-hold records | `PDI-11` | Until release, then 2 years | (owner) gap; mirrors audit | Release | Delete | H2 |
| Search query log | `PDI-17` | 30 days raw text | (owner) gap | Query time | Delete; keep only counts | H1 / B1 |
| Search index copy | `PDI-17` | Until the source is unpublished or erased; unpublished content excluded within 60 seconds (`REQ-057`) | PRD `REQ-057` | Unpublish, erasure | Rebuild or delete (derived data is rebuilt, not restored — [PRD §7](../product/AWBMS-PRD.md)) | H1 / B2 |
| Not-found observations | `PDI-16` | 30 days | (owner) gap | Observation time | Delete | H1 / B1 |
| Blog content and revisions | `PDI-15` | While the tenant keeps it; deleted content purged within 30 days | (owner) gap | Deletion by editor | Delete including revisions | H1 / B1 |
| Media objects | `PDI-14` | As blog content; object-store deletion within 30 days | (owner) gap | Deletion by editor | Delete object and metadata | H1 / B1 |
| Site profile, tenant domain | `PDI-13`, `PDI-18` | While the tenant exists | (owner) gap | Tenant offboarding | Delete | H1 / B1 |
| Tenant records | `PDI-01` | Tenant lifetime; after offboarding a 30-day export window, then deletion within 90 days | (owner) gap | Offboarding | Delete | H1 / B1 |
| Abuse-control counters | `PDI-21` | No longer than the limiter window (seconds to minutes); never logged | (owner) gap | Window end | Expire | H1 not applicable / B2 |
| Migration staging, reconciliation extracts, rehearsal snapshots | `PDI-22` | Destroyed no later than 30 days after the wave's observation window (7 days, `PD-14`) closes | (owner) gap | Wave acceptance | Destroy and record | H1 / B2 |
| Database and object-store backups | `PDI-23` | 35 days for database backups | (owner) gap; `PD-01` fixes RPO, not backup retention | Backup time | Expire | H1 applies to what the backup preserves / B3 interplay |
| AWCMS source rows after cutover | `PDI-22` | Until the legacy decommission decision | (owner) gap | Capability cutover (`AD-27`) | Delete under a recorded AWCMS decision | H1 / outside AWBMS |

**Rules.** Each retention period is a maximum for ordinary purge. A shorter period applies when the purpose ends sooner, when consent is withdrawn, or when an erasure request is granted. The purge job is `CAP-07`; a module without a descriptor is not admitted (`REQ-040`). The audit period of 2 years and the 5,000,000 rows per tenant per year of [PRD §8](../product/AWBMS-PRD.md) set the audit store size `[A]`.

## 7. Data protection impact assessment

Prepared as structured input for counsel. Whether UU PDP requires a formal DPIA for each activity is `[O]` (Art. 34, `[O] verify with counsel`). **Residual** assumes the measures are implemented and verified as specified; today **none is implemented**, so the pre-implementation position is the *Inherent* column. Scale: L = low, M = medium, H = high. *Likelihood* and *Severity* are the author's judgement `[A]`.

### 7.1 Processing description

| Activity | What happens | Who decides purposes | Systems and processors |
|---|---|---|---|
| A. Newsletter | An anonymous visitor submits an address from a tenant site; AWBMS stores a pending row, sends one confirmation by email after commit, and on confirmation marks it active; unsubscribe by link; bounces and complaints suppress (`REQ-065`–`REQ-071`) | Tenant (likely controller) `[O]` | AWBMS; email provider; edge network |
| B. Visitor analytics | A browser beacon from a tenant site posts an event; AWBMS answers neutrally, stores aggregate-capable data, rate-limits per client and tenant (`REQ-062`–`REQ-064`) | Tenant `[O]` | AWBMS; Cloudflare edge |
| C. Identity and profile | Staff sign in, hold sessions, enrol MFA, update profiles; machine credentials serve builds (`REQ-005`–`REQ-015`) | Tenant for its staff; operator for operator staff | AWBMS |
| D. Audit and logging | Privileged operations and denials are recorded in an immutable audit store; operational telemetry is kept separately (`REQ-020`–`REQ-024`, `REQ-027`) | Operator (security) and tenant (accountability) | AWBMS; observability backend |
| E. Legacy migration | AWCMS data of the capabilities in scope is migrated by a deterministic, resumable process; passwords are verified in the legacy format then rehashed; encrypted secrets are re-encrypted; sessions are not carried (`REQ-010`, `REQ-011`, `REQ-078`, `REQ-084`; `OD-03`) | Operator, on the controller's behalf | AWCMS source; migration workspace; production-like snapshots (`AS-03`) |
| F. Subject-rights handling | Requests for export or erasure are processed within 14 days, scoped to one tenant, excluding held data (`REQ-043`) | Tenant | AWBMS lifecycle capability |
| G. Backup and recovery | Backups in a separate failure domain, rehearsed on production-like snapshots (`REQ-075`) | Operator | Backup store |

### 7.2 Necessity and proportionality

| Activity | Necessity | Minimisation already required | Proportionality concern |
|---|---|---|---|
| A | An address is the minimum needed to send mail; double opt-in is needed to prove the owner of the address consented | Neutral responses (`REQ-065`); hashed tokens; no provider call inside a transaction (`AD-15`) | Storing a hashed network address as consent proof goes beyond the minimum unless counsel finds it necessary (DPR-03) |
| B | Aggregate counts of audience are a legitimate site-owner need; identifiers are not | No raw network address (`REQ-063`); no cross-site identifier (`PD-15`); 90 days (`PD-11`) | The allowed field set is not yet defined, so re-identification through combination is open (DPR-05) |
| C | Staff cannot be authorised without an identifier and credentials | Revocable opaque sessions (`AD-11`); Argon2id (`AD-12`) | Profile fields beyond the need are decided at step 5 (DPR-09) |
| D | Accountability and investigation require recording actors | No secrets in audit (`REQ-022`); separate stores (`REQ-023`) | Immutability conflicts with erasure (DPR-10) |
| E | Migration is required to continue service; passwords cannot be converted without the user | Sessions not carried; legacy format not accepted at runtime (`REQ-084`) | Snapshots copy all categories, and staging could outlive the need (DPR-12) |
| F | Rights must be honoured | Declared categories only (`REQ-043`) | Identity verification is not specified (DPR-15) |
| G | Recovery requires copies | Separate failure domain | Erasure may return on restore (DPR-21) |

### 7.3 Risks to data subjects

Measures cite existing IDs only. "PPRQ" names a **proposed** measure (not yet a requirement).

| ID | Activity | Risk to the data subject | Likelihood | Severity | Inherent | Measures (existing IDs; proposed) | Residual |
|---|---|---|---|---|---|---|---|
| `DPR-01` | A | A third party subscribes someone else's address and the owner receives unwanted mail | M | M | M | `REQ-065`, `REQ-066`, `REQ-068`, `C-07`, `AD-04`, `VG-05`; double opt-in; one confirmation per recipient per 15 minutes | L |
| `DPR-02` | A | An attacker learns whether a person is a subscriber (membership disclosure) from response differences | M | M | M | `REQ-065` byte-equivalent responses, `MET-22` test, `VG-05` | L |
| `DPR-03` | A | Consent cannot be shown, or its proof stores a hashed network address that is pseudonymous, not anonymous | M | M | M | Consent time recorded in legacy (`consent_at`); no AWBMS requirement exists; `PPRQ-06`, `OQP-06` | M |
| `DPR-04` | A | Mail continues after unsubscribe, or an erased address is re-added | L | M | L | `REQ-069` (60 seconds), `REQ-071`; `PPRQ-07` | L |
| `DPR-05` | B | A visitor is re-identified from the combination of stored fields even without a network address | M | M | M | `REQ-063`, `PD-15`, `VG-14`; `PPRQ-09` (field allow-list) | M until step 5 fixes the fields |
| `DPR-06` | B, PLT | The client address is processed transiently by the edge, the limiter and request handling, so "never stored" is hard to prove | H | L | M | `REQ-063`, `REQ-064`, `REQ-027`; `PPRQ-08` (limiter counters ephemeral); `OQP-13` | M |
| `DPR-07` | B | Visitors are measured without a notice or choice, because the tenant's site, not AWBMS, shows notices | M | M | M | None in AWBMS; `PPRQ-10`; `OQP-05` | M |
| `DPR-08` | C | Account takeover exposes the personal data the account can read | M | H | H | `REQ-005`–`REQ-009`, `REQ-012`, `REQ-016`, `AD-11`, `AD-12`, `VG-07` | L |
| `DPR-09` | C | A profile is readable by another user, or holds more than needed | L | M | L | `REQ-015`, `REQ-016`, `REQ-001`, `VG-06`; column minimisation at step 5 | L |
| `DPR-10` | D | Audit rows hold personal data beyond need, and immutability blocks erasure | M | M | M | `REQ-020`–`REQ-023`, `REQ-027`; `PPRQ-11` (opaque actor identifier, mapping removal) | M |
| `DPR-11` | D | Logs, traces or metrics capture personal data or secrets and are copied to a third-party backend | M | H | H | `REQ-027`, `AD-20`, `MET-20` canary; 30-day retention (`PD-11`); `PPRQ-01` label-driven redaction | L |
| `DPR-12` | E | Production personal data and credential material are exposed in migration workspaces or in production-like snapshots used for rehearsal, and copies outlive the need | M | H | H | `REQ-084`, `REQ-010`, `REQ-011`, `REQ-078`, `OD-03`, `VG-10`, `VG-12`, `AS-03`; `PPRQ-12`, `PPRQ-13` | M |
| `DPR-13` | E | During coexistence the same person's data sits in both AWCMS and AWBMS, and an erasure or correction reaches only one; data of the eight capabilities that remain in AWCMS (for example comments and push subscriptions) is outside AWBMS control | M | M | M | `REQ-079`, `AD-27`; `PPRQ-13`, `PPRQ-17`; `OQP-09` | M |
| `DPR-14` | E | A re-encryption or rehash step leaves a legacy-format secret or plaintext candidate (`nonce`, `code_verifier`, push `auth_secret`) readable | L | H | M | `REQ-084`, `REQ-010`, `OD-03`, `VG-10`; the three plaintext candidates of §3 must be excluded or expired, not migrated | L |
| `DPR-15` | F | A request is answered late, or to an impersonator who then receives another person's data | M | H | H | `REQ-043`, `VG-14`, `REQ-001`; `PPRQ-03` (verification), `PPRQ-04` (clock); `OQP-02` | M |
| `DPR-16` | F | Erasure misses a derived copy (search index, roll-up, event payload, object-store file, email provider) | M | M | M | `REQ-040`, `REQ-041`, `VG-14` coverage; `PPRQ-01`, `PPRQ-14` | L |
| `DPR-17` | PLT | A breach is not detected, or notification misses the 3 × 24 hour duty (`[O] verify with counsel`) because no severity scale or breach record exists | M | H | H | `REQ-022`, `REQ-076`, `REQ-077`; `PD-20` (severity scale, owner-set); `PPRQ-15`; `OQP-03` | M |
| `DPR-18` | PLT | Processors and sub-processors (email provider, Cloudflare for edge and R2, observability backend, backup store) process data abroad or change without notice | M | H | H | `AD-17`, `AD-30`, `AS-07`, `AD-22`; `PPRQ-14`; `OQP-04` | M until counsel clears |
| `DPR-19` | PLT | A cross-tenant leak, or an operator reads tenant personal data without need | L | H | M | `REQ-001`, `REQ-014`, `AD-10`, `AS-05`, `VG-06`; `PPRQ-16` (justified operator access) | L |
| `DPR-20` | PLT | Controller and processor roles are unclear, so nobody owns requests, notices or breach notifications | M | M | M | `OQP-01`; tenant as controller and operator as processor is the working assumption `[O]` | M until counsel decides |
| `DPR-21` | G | A restore re-introduces erased personal data (`REQ-044` is only a "Should") | M | M | M | `REQ-044`, `VG-14`, `VG-12`; `PPRQ-02` | L |
| `DPR-22` | B | Query text or not-found paths capture personal data a visitor typed | M | L | L | `REQ-027`; 30 days; `PPRQ-19` | L |
| `DPR-23` | PLT | Uploaded media discloses location or identity of people through embedded metadata or content | M | M | M | `REQ-054`; `PPRQ-18` | L |
| `DPR-24` | PLT | Specific or children's data enters through free text or media, with none of the added safeguards | L | H | M | `REQ-040`, `VG-14`; `PPRQ-20`, `PPRQ-21` | L |

**Highest residual risks** (after specified measures): `DPR-03`, `DPR-05`, `DPR-06`, `DPR-07`, `DPR-10`, `DPR-12`, `DPR-13`, `DPR-15`, `DPR-17`, `DPR-18`, `DPR-20`. Of these, `DPR-15`, `DPR-17`, `DPR-18` and `DPR-20` depend on a legal answer, not on engineering.

### 7.4 Specific topics

**Newsletter (consent proof, double opt-in, unsubscribe, third-party addresses).** Double opt-in answers `DPR-01` and gives consent evidence. `REQ-065`–`REQ-069` cover cooldown, indistinguishable responses, token survival and origin binding. Missing: what a consent record contains and whether a hashed address belongs in it (`PPRQ-06`), a minimal suppression entry (`PPRQ-07`), and minor subscribers (`OQP-12`).

**Visitor analytics (`PD-15`).** `REQ-063` and `PD-15` forbid a raw network address and a persistent cross-site identifier. They do not say what **is** allowed. Fingerprinting by user agent, language, screen size, path and time is the main residual risk. `PPRQ-09` proposes an allow-list; `PPRQ-08` clarifies that limiter counters keyed by address are transient and are not "storage".

**Identity and profile.** Covered by the authentication requirements; the privacy delta is column minimisation at step 5 and the unknown content of `awcms_profile_identifiers` (`OQP-10`).

**Audit logs containing personal data.** `REQ-021` forbids altering audit rows; erasure law may require removing personal data. The proposed resolution is structural: audit rows carry an opaque actor identifier and no free text, and erasure deletes the mapping from identifier to person (`PPRQ-11`). Whether 2 years is a legal minimum or a choice is `OQP-07`.

**Legacy migration (`OD-03`, re-encryption).** The AWCMS password format is Argon2id with fixed parameters and no rehash-on-login exists in AWCMS [F]; `REQ-010` adds the rehash. Sessions and legacy tokens are not carried across (`REQ-011`); newsletter links issued before cutover stay valid for their remaining life (`REQ-069`). Encrypted secrets are re-encrypted (`REQ-084`). The three plaintext candidates are short-lived or key-material rows and should be expired rather than migrated `[R]`. Rehearsals on production-like snapshots (`AS-03`, `VG-12`) are themselves processing of production personal data and need production-grade controls or masking (`PPRQ-12`). Whether continuing the same purposes in a new system needs notice is `OQP-09`.

**Data-subject requests within 14 days.** `REQ-043` is the requirement; `PD-12` is accepted as an internal target. The statutory period may be shorter (§2, `OQP-02`). Identity verification and a request clock are missing (`PPRQ-03`, `PPRQ-04`).

**Breach notification.** UU PDP is understood to impose a 3 × 24 hour written notification duty (`[O] verify with counsel`, §2). AWBMS has audit and alerting requirements but no breach record, no per-tenant affected-subject export and no severity scale (`PD-20` is owner-set and unfilled). If AWBMS is a processor, its notice to the controller must be fast enough for the controller to meet its own period `[O]`.

**Processors and sub-processors.** From the architecture: Cloudflare (edge, DNS, R2 object store; `AD-17`, `AD-30`), an email provider (`AS-07`, unnamed), an observability backend (`AD-20`, unnamed), and a backup store (unnamed). Their regions, contracts and sub-processors are not in any document `[O]`. A processor register is `PPRQ-14`.

**Multi-tenant roles.** Working assumption: each tenant is a controller for the personal data of its staff, subscribers and visitors; the AWBMS operator is a processor acting on the tenant's instructions; for operator staff and for platform security data the operator is a controller `[O]`; joint-controller situations are possible for shared abuse-prevention data `[O]` (`OQP-01`). The shared deployment (`OD-05`) makes tenant isolation (`REQ-001`) the primary technical control for this split.

### 7.5 DPIA conclusion

This analysis recommends, **without legal finality**, that counsel and the privacy officer decide on formal impact assessments for:

1. **Legacy migration of AWCMS personal data** (activity E) — **required in practice before wave 1**: it copies every category, handles credentials and uses production-like snapshots.
2. **Visitor analytics** (activity B) — **recommended before `CAP-16` build**: anonymous, large-scale in number of visitors, and the field set is undecided.
3. **Newsletter** (activity A) — **recommended before `CAP-14` build**: consent evidence, third-party addresses and possible minors.
4. **Processors and cross-border transfer** (activity G and `DPR-18`) — **recommended before any production data reaches a processor**.
5. **Identity, audit and subject-rights handling** (activities C, D, F) — **recommended as one combined assessment** at step 5, once the columns are known.

Whether any of these is legally mandatory under UU PDP Art. 34 `[O] verify with counsel` is for counsel.

## 8. Data-subject rights procedure

Outline only. The tenant, as likely controller, decides; the operator executes (`OQP-01`). Rights are listed as the Act is understood to define them (Arts. 5–14, `[O] verify with counsel`).

| Right | Understood basis | PRD requirement | Capability | What AWBMS must do | Gap |
|---|---|---|---|---|---|
| Access | UU PDP `[O]` | `REQ-043`, `REQ-040` | `CAP-07` | Verified export of declared categories for one tenant | Verification, format, clock (`PPRQ-03`, `PPRQ-04`) |
| Correction | UU PDP `[O]` | `REQ-015` (own profile) | `CAP-02`, `CAP-07` | Staff self-service; controller-initiated correction for others | No requirement for subscribers or other subjects (`PPRQ-04`) |
| Deletion / ending processing | UU PDP `[O]` | `REQ-041`, `REQ-043`, `REQ-044` | `CAP-07` | Erase or anonymise declared categories, including derived copies, excluding held data | Backup interaction is only "Should" (`PPRQ-02`) |
| Restriction (delay) | UU PDP `[O]` | None | `CAP-07` | Mark data restricted: stored, not used | `PPRQ-05` |
| Portability | UU PDP `[O]` | `REQ-043` (export) | `CAP-07` | Export in a structured, machine-readable form for data the subject provided | Format not specified (`PPRQ-04`) |
| Objection / withdraw consent | UU PDP `[O]` | `REQ-069` (unsubscribe) | `CAP-14`, `CAP-16` | Stop mailing within 60 seconds; stop analytics for a visitor who objects | No visitor objection path for analytics (`PPRQ-10`) |
| Information | UU PDP `[O]` | None | Tenant sites | Provide the tenant what it needs to write a notice | `PPRQ-10` |

**Procedure steps.**

| Step | Action | Role | Existing anchor |
|---|---|---|---|
| 1 | Receive the request in the tenant's channel; record it and start the clock | `P-04`, `P-07` | `REQ-043` |
| 2 | Verify the requester controls the identifier concerned (confirmation link to the address; staff by session) without confirming that data exists | `P-07` | `PPRQ-03` |
| 3 | Resolve the tenant and scope to it (a person in two tenants is two requests) | `P-07` | `REQ-043`, `REQ-001` |
| 4 | Enumerate every location from the module descriptors, including derived copies and processors | `P-05` | `REQ-040`, `VG-14` |
| 5 | Check legal holds; held data is excluded and the hold is audited | `P-07` | `REQ-042` |
| 6 | Execute export, correction, restriction or erasure; record in the erasure ledger | `P-05` | `REQ-041`, `REQ-044` |
| 7 | Verify completeness and respond within the deadline | `P-07` | `REQ-043`, `MET-19` |
| 8 | Audit the request and its outcome without personal data in the record | `P-07` | `REQ-020`, `PPRQ-11` |

## 9. Gaps, proposed requirements and open questions

### 9.1 Proposed new requirements

For a later PRD revision. Each is a proposal (`[R]`), phrased as the PRD phrases requirements; none is decided, none changes a binding decision. "Resolves" cites the risk or gap.

| ID | Proposed requirement | Resolves | Extends |
|---|---|---|---|
| `PPRQ-01` | The system SHALL require every column and stored file of a data-owning module to carry a classification level of [§5](#5-data-classification-scheme), and SHALL drive log, trace and metric redaction from that level. | `DPR-11`, `DPR-16` | `REQ-027`, `REQ-040` |
| `PPRQ-02` | The system SHALL NOT serve traffic after a restore until the erasure ledger has been re-applied, and the ledger SHALL outlive the oldest backup that can contain erased data (raising `REQ-044` from Should to Must). | `DPR-21` | `REQ-044` |
| `PPRQ-03` | The system SHALL verify that a data-subject requester controls the identifier concerned before releasing or erasing data, without revealing whether data exists. | `DPR-15` | `REQ-043` |
| `PPRQ-04` | The system SHALL track a clock per data-subject request, with a deadline configurable per tenant (initially 14 days, `PD-12`) and a defined export format. | `DPR-15` | `REQ-043`, `PD-12` |
| `PPRQ-05` | The system SHALL let a controller mark a subject's data as restricted so that it is retained but excluded from mailing, analytics and processing other than storage. | Rights gap | `REQ-043` |
| `PPRQ-06` | The system SHALL record newsletter consent as time, tenant, consent-text version and source, and SHALL NOT store a network address or its hash unless the owner decides it is necessary. | `DPR-03` | `REQ-065` |
| `PPRQ-07` | The system SHALL retain an unsubscribed, bounced or erased address only as a keyed hash in a suppression entry with a declared retention. | `DPR-04` | `REQ-069`, `REQ-071` |
| `PPRQ-08` | The system SHALL keep any limiter counter that is keyed by a client address only for the length of the limiter window and SHALL NOT log or export it. | `DPR-06` | `REQ-063`, `REQ-064` |
| `PPRQ-09` | The system SHALL store from an analytics event only the fields of a declared allow-list, reject events with other fields, and SHALL NOT combine fields into a persistent visitor identifier. | `DPR-05` | `REQ-063`, `PD-15` |
| `PPRQ-10` | The system SHALL expose to each tenant a description of its analytics and newsletter processing, and a way to honour a visitor's objection, so the tenant site can inform visitors. | `DPR-07` | `REQ-062` |
| `PPRQ-11` | The system SHALL refer to a natural person in an audit record only by an opaque identifier and SHALL carry no free-text personal data in it, so that erasure removes the identifier-to-person mapping instead of altering a record. | `DPR-10` | `REQ-020`, `REQ-021` |
| `PPRQ-12` | The system SHALL NOT place unmasked production personal data in a non-production environment, and a rehearsal on a production-like snapshot SHALL run under production controls and be destroyed afterwards. | `DPR-12` | `REQ-075`, `AS-03` |
| `PPRQ-13` | The system SHALL destroy migration staging data and reconciliation extracts within a declared period after wave acceptance, and reconciliation reports SHALL contain counts and checksums only. | `DPR-12`, `DPR-13` | `REQ-078` |
| `PPRQ-14` | The system SHALL maintain a register of processors and sub-processors, their regions, data categories and agreement status, and SHALL NOT enable a new processor for a tenant before the register is updated. | `DPR-16`, `DPR-18` | `AD-17`, `AD-30` |
| `PPRQ-15` | The system SHALL keep an incident record with detection time, categories, tenants affected and notification decision, and SHALL export the affected subjects of a tenant on request. | `DPR-17` | `REQ-077` |
| `PPRQ-16` | The system SHALL require a recorded justification and an audit record for each operator read of tenant personal data and SHALL grant no standing access. | `DPR-19` | `REQ-017`, `REQ-020` |
| `PPRQ-17` | The system SHALL provide tenant offboarding with an export window, deletion within a declared period, and a defined treatment of backups. | `DPR-13` | `REQ-041` |
| `PPRQ-18` | The system SHALL strip or flag location metadata embedded in uploaded images unless the tenant opts out. | `DPR-23` | `REQ-053` |
| `PPRQ-19` | The system SHALL keep raw search-query text and not-found paths no longer than a declared period and SHALL NOT associate them with a client address. | `DPR-22` | `REQ-027` |
| `PPRQ-20` | The system SHALL require each module descriptor to state whether the module can hold specific or children's personal data, and SHALL require a completed impact assessment before admitting a module that can. | `DPR-24` | `REQ-040` |
| `PPRQ-21` | The system SHALL restrict operator-facing free-text fields that describe a person (reasons, notes) to a coded value where practical and SHALL redact them from logs. | `DPR-24` | `REQ-027` |

### 9.2 Open questions for owner and counsel

| ID | Question | For | Blocks |
|---|---|---|---|
| `OQP-01` | Are tenants controllers and the operator a processor, with the operator a controller for its own staff and platform security data? Are there joint-controller cases? | Counsel | Contracts, request routing, breach notice |
| `OQP-02` | What response periods do UU PDP and any implementing regulation set for each right, and is 14 days (`PD-12`) compatible? | Counsel | `REQ-043` value |
| `OQP-03` | Exact breach-notification duty: period, recipients, content, whether it runs from the processor's discovery, and the supervisory authority's current status | Counsel | `PPRQ-15`, `PD-20` |
| `OQP-04` | What conditions apply to Cloudflare, R2, the email provider, the observability backend and the backup store, and can regions be pinned? | Counsel, owner | Processor choice, `PPRQ-14` |
| `OQP-05` | What lawful basis fits visitor analytics, and does a tenant need a notice or consent banner? Who is responsible for it? | Counsel | `PPRQ-10`, `CAP-16` |
| `OQP-06` | Is a hashed network address retained as consent proof necessary and acceptable, or is it personal data to avoid? | Counsel | `PPRQ-06` |
| `OQP-07` | Are 2 years (audit), 90 days (analytics) and 30 days (logs) legally sufficient or required, and what do the gap proposals of [§6](#6-retention-schedule) need? | Counsel | `PD-11` confirmation |
| `OQP-08` | Does the operator or any tenant need a data protection officer, and is a formal impact assessment mandatory for the activities of [§7.5](#75-dpia-conclusion)? | Counsel | Governance |
| `OQP-09` | Does moving data from AWCMS to AWBMS need a notice, a new basis or a record, and who is controller for data in both systems during coexistence? | Counsel, owner | Wave 1 plan |
| `OQP-10` | What do `awcms_profiles`, `awcms_profile_identifiers`, `awcms_visit_events`, `awcms_visitor_sessions`, `awcms_site_search_query_log`, `awcms_audit_events`, `awcms_email_messages` and `awcms_subject_requests` contain, column by column? | Owner (live schema) | Step 5 |
| `OQP-11` | Is the operator registered as a private electronic system operator, and what does PP PSTE require of it for incidents and data location? | Counsel | Go-live |
| `OQP-12` | Can newsletter subscribers be minors, and does that change consent? | Counsel, owner | `CAP-14` |
| `OQP-13` | Is a client address held in a short-lived limiter counter "storage" under `REQ-063` and "processing" under UU PDP? | Owner, counsel | `PPRQ-08` |
| `OQP-14` | When, relative to gate 9, must the assessments of [§7.5](#75-dpia-conclusion) be complete? | Owner | Plan |

### 9.3 Findings for the owner

1. `REQ-044` (no return of erased data on restore) is priority **Should** while erasure is a legal right; `PPRQ-02` proposes **Must**.
2. `PD-11` covers analytics, operational logs and audit; [§6](#6-retention-schedule) lists the many other categories that need an owner value, among them backups, the erasure ledger and migration staging.
3. `PD-12` (14 days) may exceed a statutory period (`OQP-02`).
4. `REQ-063` forbids storing a raw network address but the limiters need one transiently; the boundary is undefined (`PPRQ-08`).
5. Production-like snapshots for rehearsal (`VG-12`, `AS-03`) are production personal data; no requirement governs their handling (`PPRQ-12`).
6. The analytics field set is undecided, so `PD-15` cannot yet be verified (`PPRQ-09`).
7. The frozen corpus holds column definitions from only 16 SQL files (of 148 migrations); column-level evidence for profiles, analytics and audit needs the live schema (`OQP-10`).
