# Smart School: high-concurrency CBT and online-only provider readiness

**Date:** 2026-11-28 | **Status:** Research / proposed consumer workload, not an accepted ADR or an implementation authorization.  
**Evidence:** AWBMS main `264ba70dd57e5318ce0bfac5836edadc930a3177`, README, PRD §§6–8, Blueprint §11, pre-production evidence §4.2, ADR-0008 and ADR-0004. AWBMS is pre-production; only the verification spike exists. VG-09 remains OPEN.

## 1. Capacity gap and scope

PRD S1/S2 assume 25/100 concurrent authenticated users per tenant (peaks 50/200) and 30/150 authenticated API requests/s per deployment. The reported AWCMS 620 RPS at p95 405 ms is historical AWCMS evidence only, not AWBMS or CBT evidence. CBT is not admitted into AWBMS v1 by this note. Propose a separate school-edge workload profile and owner-approved ADR/PRD delta; do not alter existing approved capacity figures silently.

Assume 10,000 simultaneous students total across grades 1–12 (SD 6, SMP 3, SMA 3), ten class groups per grade = 120 groups, mean 83.3 students/group. This is arithmetic planning, not an assumed normal class size; confirm actual enrolment, group count and per-campus split. Model both one 10,000-student origin (worst-case qualification) and distributed school origins (capacity per largest school plus shared gateway aggregate). A WAN outage must not move authority to a second writable database.

## 2. Reproducible offered-load model

All rates are HTTP requests at the school origin, not concurrent sessions, not SQL TPS. Assume distinct calls, no CDN cache for protected exam data. 10,000 students, 90–120 min exam, randomized client timers. Conservative steady mix:

| Class | Assumed frequency | RPS |
|---|---:|---:|
| Durable answer batch (including autosave; do not double count) | 1 / 20 s / student | 500 |
| Question page/read | 1 / 30 s / student | 333.33 |
| Heartbeat/integrity signal | 1 / 30 s / student | 333.33 |
| Connectivity capability status | 1 / 60 s / student | 166.67 |
| Proctor, admin and other API traffic | fixed allowance | 100 |
| Total | | 1,433.33 |

30% reserve => 1,863.33 RPS; proposed steady target 2,000 RPS. Login/preflight/start adds 3 requests per student in 10 s => 3,000 incremental RPS, combined 4,433.33; reserve => 5,763.33; round to 6,000 RPS burst. Submit/status adds 2 requests/student in 10 s => 2,000 incremental; combined + reserve = 4,463.33; target 5,000 RPS. Reconnect 3 requests/student/10 s uses start envelope; simultaneous one-second retry is adversarial overload, not an accepted normal operating pattern. Test 10,000 RPS overload for 10 s; bounded rejection and recovery, not a throughput promise. At 10 KB mean response payload, 6,000 RPS ~480 Mbps payload only; media/downloads, TLS, Wi-Fi and retransmits separate. Preload authorized exam media on LAN with embargo, no answer keys. Re-estimate using captured route mix, payload distributions and think time.

## 3. Proposed acceptance profile (requires owner approval)

10,000 concurrent authenticated student sessions; 2,000 offered RPS for 120 min; 6,000 offered RPS for 60 s startup/reconnect; 5,000 for 60 s submit. At target rates, read p95 <=500 ms and write p95 <=900 ms (existing PRD thresholds), proposed p99 read <=1,000 ms / write <=2,000 ms, 5xx/timeout <=0.1%, zero dropped load-generator iterations, no unexpected 429, zero cross-tenant disclosure, zero lost acknowledged answers, zero duplicate finalization, no false success, no answer revision regression. Report successful achieved RPS, exact status distribution, p50/p95/p99 by route, cache hit ratio and origin rate. Stress separately: explicit 429/503 + retry-after, bounded queues; no corruption; drain backlog after load returns within 120 s proposed. Three repeat runs, production-like data and hardware manifest, warm/cold cases, no real student data. Error metrics must not hide dropped requests, refused calls or retries.

## 4. Backend engineering requirements

Reuse the approved Rust/Axum/Tokio/Tower/SQLx/PostgreSQL single-writer architecture as design, not implemented capabilities. Extend PRD for school edge and examination; align identity, tenant-bound transaction, FORCE RLS, RBAC/ABAC/SoD, audit and transactional outbox. Scoped attempt ownership, immutable exam revision, server deadlines, unique attempt and idempotency key/body binding, optimistic version/sequence, durable ACK after commit, atomic finalization, late/reordered/replayed answers rejected deterministically. No external calls inside DB transactions. Protect bank vs practice data; never ship answer keys, no shared public cache. Batch answer writes and bounded payloads; jitter heartbeat/retry; reserve capacity for answer/finalize; backpressure, timeouts, admission control, separate bounded pool/worker budget for non-exam work and auth; do not bypass auth/RLS/durability to pass load tests. Cache immutable authorized content locally, not permissions across tenants. Use indexed tenant/exam/attempt keys; inspect EXPLAIN (ANALYZE, BUFFERS) in synthetic DB, pool wait, locks, WAL and checkpoint I/O. Tune measured SQLx pools rather than 10k DB connections; if measured DB occupancy is 20 ms/request at 2,000 RPS, mean concurrent DB work is about 40 (not a sizing guarantee). HA/standby requires single-writer fencing; no automatic cloud promotion. Prioritize local exam traffic over media, backups, reconciliation and analytics; hourly backup still scheduled, test I/O budget. Restore drills and RPO/RTO reviewed for T1; acknowledged writes survive process crash with durable PostgreSQL config, not guaranteed under total storage/site loss.

## 5. Connectivity and provider guard

Connectivity state must distinguish client-to-school API, school WAN, gateway/tunnel, and provider/channel capability. Browser navigator.onLine or native link flags are hints only. School backend runs bounded, coalesced, read-only probes with TTL, timestamps, hysteresis, timeout and circuit breaker. Unknown/stale = fail closed for new external payment actions; no provider call for every student poll. UI displays status on every persona (Astro and Lynx), explicit online/local-only/provider-degraded/reconnecting/API-disconnected, last checked, accessible text; finance actions show specific reason and retry manually. Backend independently rechecks policy at command admission and worker dispatch; no new DOKU checkout/QR/VA/top-up/refund/disbursement or queued auto-debit while offline; never auto-submit a previously refused user request after reconnect. Local bill creation/read and separately authorized cash recording remain available. An already initiated payment may complete independently while school is offline: keep unknown/pending verification; never declare failed or paid merely from network state or screenshot. Verify signed provider callback per product contract; durable relay inbox, unique provider event/reference, tenant resolution, amount/currency/order matching, replay protection, status inquiry/reconciliation before retry, idempotent ledger posting and audit. Provider adapter outside DB transaction, single authoritative ledger at school. DOKU product-specific sandbox verification and legal review remain required. Proposed APIs/events are not existing AWBMS routes.

## 6. Evidence plan and backlog boundaries

- Step 2: consumer workload admission / PRD delta, school-edge ADR (OD-05 exception), owner accepts capacity and SLOs; confirm actual class/tenant topology. Step 3: threat/DPIA incl. children, bank leakage, account/session abuse, financial fraud and denial of service; linked existing issue #63.
- Steps 4–8: validated UX and connectivity matrix, ERD/data dictionary, RBAC/ABAC/FORCE RLS, attempt/payment state machines, OpenAPI/AsyncAPI + mock; use existing #11–#24 as dependencies, not duplicate foundation.
- After Gate 9: separate atomic slices for (a) bounded middleware, budgets, load observability (#18, #12, #15); (b) attempt write/finalize and security (#20, #22); (c) connectivity capability and provider state/relay (#19, #23); (d) benchmark harness and report for VG-09; (e) backup/WAN/rollback recovery evidence VG-12. No product code before Gate 9.
- Benchmark using k6 open-model constant/ramping arrival rate and 10k stateful identities, independent distributed generators; report dropped iterations, avoid coordinated omission, verify DB state after every run. Run 500/1k/2k sustained, 6k start/reconnect, 5k submit, 10k stress, 120-min soak; include simultaneous backup, loss of WAN/VPS/Cloudflare, provider outage, latency, process restart, DB contention, tenant abuse and retry storms. Separate HTTP API, network/Wi-Fi and database microbenchmark (pgbench custom SQL) results. Test direct LAN, public via VPS, distributed sites; do not use public production DOKU or real students. Use CI smoke subset; full scale on isolated staging/pre-production, archive commit, workload seed, config, hardware, raw metrics, reconciliation queries and signed decision. VG-09/VG-12 remain OPEN; Gate 15 No-Go without evidence.
- Proposed hardware experiment: one origin 16–32 physical/vCPU class, 64–128 GB RAM, mirrored enterprise NVMe, separate backup target, 10 GbE server uplink; compare 8/16/32-core configurations, multiple app replicas only if measured; scale Wi-Fi/VLAN per classroom/site and measure actual client concurrency. This is a test configuration, not procurement sizing.
- Privacy/continuity: synthetic load data; tenant isolation in metrics/logs and backups; no question/answer content in logs; align UU PDP, ISO/IEC 27001/27002/27005/27034/27701, ISO 22301, ISO/IEC 20000-1, ISO/IEC 25010; no certification claim.

## 7. Research references

- DOKU Checkout: https://docs.doku.com/accept-payments/integration-tools/doku-checkout
- DOKU payment notification: https://docs.doku.com/get-started/manage-business/set-up-integration/webhook-payment-notification
- DOKU transaction status: https://docs.doku.com/get-started/manage-business/manage-reports/transaction-report
- MDN Navigator.onLine limitations: https://developer.mozilla.org/en-US/docs/Web/API/Navigator/onLine
- k6 open/closed models: https://grafana.com/docs/k6/latest/using-k6/scenarios/concepts/open-vs-closed/
- k6 thresholds: https://grafana.com/docs/k6/latest/using-k6/thresholds/
- k6 dropped iterations: https://grafana.com/docs/k6/latest/using-k6/scenarios/concepts/dropped-iterations/
- PostgreSQL 18 pgbench: https://www.postgresql.org/docs/18/pgbench.html
- Cloudflare Tunnel observability: https://developers.cloudflare.com/tunnel/observability/

All numeric CBT values are assumptions/proposed gates. No benchmark or deployment was executed by this research.