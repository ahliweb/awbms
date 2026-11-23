# AWBMS — Fuchsia/Zircon Interoperability and Rust Portability Readiness

## Document control
- **Version:** v1.0.0
- **Updated:** 2026-11-23
- **Status:** RESEARCH / PROPOSED, not an ADR, verification gate or production authorisation.
- **Evidence snapshot:** AWBMS \`main\` commit \`e6bb5638e0628418079b1c1d6fcf0a123c5bb4cc\`; public upstream references checked 2026-11-23.
- **Document Changeset:** Record alternatives, ecosystem constraints, boundary security, reproducible proofs and decisions needed to connect future AWBMS products with Fuchsia/Zircon devices.
- **Scope:** Stage 1 architectural research only. No application runtime, device integration, source import or deployment.

## 1. Recommendation and certainty

**[R] Keep AWBMS a Rust-native Linux/server backend with PostgreSQL as the sole authoritative business state.** Integrate devices through a neutral versioned HTTPS/event boundary, independent of the device OS. Explore pure-Rust shared algorithms or a minimal native Fuchsia component only in separately gated research. **Do not port AWBMS to Zircon kernel.**

Evidence labels below:
- **[F] Verified AWBMS:** inspected \`Cargo.toml\`, \`rust-toolchain.toml\`, \`README.md\`, \`crates/verification/src/lib.rs\`, \`crates/verification/tests/rls.rs\`, Stage 1 evidence and gate register at snapshot above.
- **[C] External technical claim:** primary Rust/Fuchsia documentation linked in §9; verify upstream versions and policy at each PoC. These are not AWBMS local facts.
- **[A] Assumption:** a simple Fuchsia client can reach an AWBMS-style HTTPS interface when Fuchsia SDK/runtime/network capability is provisioned; **not tested**.
- **[O] Open decision:** whether a product has a real hardware need for Fuchsia, what deploy/publishing route is permitted, component/toolchain versions, software license and runbook owner.
- **[G] Verification required:** cross-target compilation, real packaging and FEMU launch, capability deny tests, HTTP exchange, baseline comparison and recovery.
- **DEFERRED:** native AWBMS service/process migration, local PostgreSQL and Docker on Fuchsia, Starnix reliance, production robot safety controls.

This document does **not** register new formal \`C-/A-/VG-/OD-\` identifiers in the architecture validation. Existing binding decisions/gates are unchanged; a new gate or reversal requires the repository's formal review/ADR mechanism.

## 2. Actual AWBMS snapshot

| Item | Evidence | Meaning for Fuchsia |
| --- | --- | --- |
| Rust 2024; \`rust-toolchain.toml\` pins 1.98.0 | \`Cargo.toml\`, toolchain file | A compiler version is pinned, not a Fuchsia build |
| Axum 0.8.9, Tokio 1.53.1, SQLx 0.9.0, Tower 0.5.3, tower-http 0.7.0 | \`Cargo.toml\` | Host Linux verification only; crate + transitive feature support not tested on Fuchsia |
| \`crates/verification\` | \`lib.rs\` and \`tests/rls.rs\` | Minimal route, graceful shutdown, SQLx tenant transaction; not a production module |
| PostgreSQL FORCE RLS, non-BYPASSRLS role tests | Stage 1 evidence \`VG-03-VG-04-2026-08-29.md\` | DB security evidence limited to PostgreSQL testing on Linux |
| \`VG-01\` partial; \`VG-03\`, \`VG-04\` pass; \`VG-05\`, \`VG-09\`, \`VG-12\` open | Stage 1 register | No production readiness/parity/performance/rollback evidence |
| \`VG-15\` status wording conflicts between Stage 1 table and architecture summary | Stage 1 register vs validation | Existing issue #7 owns reconciliation, not this research |
| Workspace \`license = "MIT"\` but README indicates license undecided, no LICENSE file found | Cargo.toml, README, docs/README | Treat redistribution rights as undecided until formal owner decision; existing #7 governance |
| No Fuchsia-specific code, FIDL, cross-target CI or packaging found in checked paths/search | GitHub source/search snapshot | **ABSENT IN CHECKED SCOPE**, not a universal claim about forks |

AWBMS remains independent of AWCMS source/runtime under existing \`VG-15\`; Fuchsia must not become a new mandatory runtime/dependency of the core workspace. Do not silently widen the platform's purpose to IoT infrastructure.

## 3. Important nuance: Rust target ≠ supported external Fuchsia Rust SDK

1. **[C]** Rust documents \`x86_64-unknown-fuchsia\` and \`aarch64-unknown-fuchsia\` as **Tier 2**, with build/packaging instructions.
2. **[C]** Fuchsia's published [Programming Language Policy](https://fuchsia.dev/fuchsia-src/contribute/governance/policy/programming_languages) says Rust is used within the Fuchsia Platform Source Tree but *"Rust is not supported for end-developers"*. The policy may lag source-level development; do not infer an approved stable external Rust SDK from the rustc target alone.
3. **[C]** Zircon kernel and Fuchsia user-space are different environments. Kernel integration implies separate restricted tooling and capability interfaces; AWBMS HTTP/backend code does not belong in the kernel.
4. **[C]** Starnix is a Linux UAPI compatibility layer for selected programs; does not establish compatibility with Bun, PostgreSQL, Tokio or Docker.
5. **[C]** Fuchsia device support is specific to boards/drivers, not all ARM64/x86-64 systems. FEMU proves emulator behaviour, not physical peripherals.
6. **[A]** No assumed ABI or library parity for \`tokio::net\`, \`tokio::signal\`, TLS/rustls, \`sqlx-postgres\`, DNS, filesystem, entropy or network permissions. Test independently.

**Consequence:** before a native PoC, read current Fuchsia end-developer packaging and policy. If external Rust authoring is not viable under the chosen SDK, record **BLOCKED** and use a non-Fuchsia adapter or an isolated upstream-source-tree-only research spike, with separate license/governance approval. Do not work around policy via an unsupported production build.

## 4. Architecture options (trade-offs, not decisions)

| Option | Fit/security | Performance/operability | Recommendation |
| --- | --- | --- | --- |
| A. AWBMS Linux + Fuchsia HTTPS client | Compatible service boundary; tenant/device identity server-side | Network retries needed; no Fuchsia SDK in core | **Preferred** |
| B. Pure Rust domain/DTO crate shared with edge | Portable in principle if no OS/DB dependency | Requires semver & no leak of backend secrets | Conditional future reuse |
| C. Native Fuchsia component + FIDL adapter | Strong component capability model | Build/deploy tooling/policy/driver unknown | Research spike only |
| D. Starnix Linux binary | Could reduce rewrite in narrow cases | ABI and support unknown; operational complexity | Defer |
| E. Full AWBMS on Fuchsia | Large porting surface, threatens current DB/ops guarantees | No evidence of parity, backup or rollback | Reject for current roadmap |

Accessibility/SEO: backend adapters have no user UI/SEO impact; any downstream UI must be evaluated in its consumer app. Business cost: choose the smallest strategy that proves a product requirement; record hardware and CI toolchain costs before approving work.

## 5. Proposed boundary and trust model

~~~mermaid
flowchart TD
  P[Business or ARKA client] --> G[Authorized backend product API: AWBMS candidate]
  G --> DB[PostgreSQL + FORCE RLS: authoritative ledger/business data]
  G --> B[Versioned device integration port: HTTPS/events]
  B --> X[Linux gateway]
  B -. research-only .-> F[Fuchsia Rust component + FIDL locally]
  X --> MCU[MCU / RTOS sensor]
  X --> ROS[ROS 2 robot task controller]
  ROS --> SAFE[Independent safety interlock, emergency stop]
~~~

**Diagram summary:** Fuchsia is a bounded endpoint of a future product contract; Linux remains the backend and PostgreSQL authority. AWCMS-One remains a **template**, never a shared production backend for derived apps; AWBMS remains a separate strategic platform and is not promoted automatically.

Security invariants for any future product:
- Device principal is distinct from human actor. Never trust the body-provided \`tenant_id\`, device ID or capabilities without a server-side credential-to-resource binding.
- Every command carries authorization, resource scope, expiry, nonce/idempotency, correlation, bounded retry, and evidence of outcome. Use TLS/mTLS or equivalent certified channel; revoke credentials and enforce at server.
- No raw DB credentials, private personal context, agent system secrets, or provider keys on devices. No open PostgreSQL port to edge network.
- Capability manifests are default-deny; test absent FIDL/handle grants, not merely positive execution.
- A reply \`accepted\` from gateway is not proof of completed action.
- Actuator and robot emergency stop are enforced locally, not by agent/cloud. No safety claims based on Zircon deadline scheduling alone.

## 6. Six implementation/use-case candidates

| Case | Backend responsibility | Fuchsia role | Evidence required |
| --- | --- | --- | --- |
| Business smart hub | Identity, configuration, audit, events | Local device hub | Capability restriction + secure commissioning |
| Cold-chain warehouse | Immutable-ish audit trail and telemetries in PostgreSQL | Read-only sensor gateway | Offline buffer/replay/kalibrasi |
| Smart office | Asset ownership, notifications | Interactive panel/sensor aggregator | Privacy/tenant negative tests |
| ARKA home assistant | Owner-approved task routing | Speech/IoT interface candidate | No owner content on unbound speaker |
| Logistics robotics | Typed task dispatch and completion evidence | Diagnostics sidecar candidate | Local estop, fail-safe task TTL |
| IoT research device | Synthetic heartbeat | Rust/FIDL emulator component | Reproducible FEMU launch and restart |

None of these is implemented in AWBMS. A real device product must own its application repo, backend/data, UAT, deployment and privacy classification.

## 7. PoC plan, staged and isolated

- **F0 / research only:** audit \`Cargo.lock\` for target-specific crates, native linking, \`cfg(target_os)\`, TLS, syscalls and runtime requirements. Save pinned source SHA + target/toolchain versions + license notes. Report supported/unsupported/unverified separately.
- **Feasibility gate:** confirm the current Fuchsia external Rust development policy, SDK availability, and legitimate target build/publish path. If unavailable → stop the native-Rust route.
- **Isolated spike** (only with scope/PoC approval): compile a tiny no-secrets pure Rust component targeting official Fuchsia triple; separately build as package, launch on FEMU with synthetic data. Do not add Fuchsia SDK to AWBMS production workspace.
- **Integration** (later): mock HTTPS call with device identity and denied/allowed cases; local capability routing, audit and kill switch. Never require live AWBMS product endpoint during Stage 1.
- **Comparison:** Linux control with equivalent synthetic workload; record start-up, CPU/RAM, p95/p99, reconnect, offline survival, update and recovery. Emulator results must not be described as hardware results.
- **Go/no-go:** explicit ADR once results, product demand, legal/license constraints, support plan and cost are established. Do not mark existing AWBMS stage gates PASS from these PoCs.

## 8. Legal and standards applicability

- Indonesia: UU 27/2022 PDP, UU ITE as amended by UU 1/2024, PP 71/2019; check radio/device certification and sector safety obligations if a hardware product is sold.
- Standards for relevant products: ISO/IEC 27001, 27002, 27005, 27017/27018 for cloud, 27034 for app security, 27701 for PII management, ISO/IEC 20000-1, ISO 22301, ISO/IEC 15408, ISO/IEC 30141/27400 for IoT, IEC 62443 for industrial, ISO 10218 or ISO 13482 if applicable to robots, OWASP ASVS/API Security, NIST SSDF. Editions and applicability must be verified before asserting compliance/certification.
- Traceability: product goal → acceptance criteria → threat/control → actor/auth/device data → contract → test → evidence → SOP → monitoring.

## 9. Source list and reproducibility

### AWBMS snapshot
- [Repository source](https://github.com/ahliweb/awbms/tree/e6bb5638e0628418079b1c1d6fcf0a123c5bb4cc)
- [Stage 1 verification evidence](https://github.com/ahliweb/awbms/blob/main/docs/architecture/evidence/VG-03-VG-04-2026-08-29.md)
- [Stage 1 gate register](https://github.com/ahliweb/awbms/blob/main/docs/architecture/AWBMS-STAGE-1-GATES.md)
- Existing #1 (Stage 1), #7 (drift/supply-chain), #21 (Rust quality gates), #23 (contract-first).

### Public upstream
- [Rust compiler: Fuchsia target support, Tier 2](https://doc.rust-lang.org/rustc/platform-support/fuchsia.html)
- [Fuchsia language policy](https://fuchsia.dev/fuchsia-src/contribute/governance/policy/programming_languages) — note explicit external Rust support caveat.
- [Fuchsia source](https://fuchsia.googlesource.com/fuchsia/), [hardware](https://fuchsia.dev/fuchsia-src/development/hardware), [FEMU](https://fuchsia.dev/fuchsia-src/development/build/emulator), [Starnix](https://fuchsia.dev/fuchsia-src/concepts/starnix).
- [Zircon kernel](https://fuchsia.googlesource.com/fuchsia/+/refs/heads/main/zircon/kernel/), [Fuchsia component capabilities](https://fuchsia.dev/fuchsia-src/concepts/components/v2/capabilities).
- [ROS 2](https://docs.ros.org/).

**Test record:** GitHub source and official documentation review only. No new build, Fuchsia SDK installation, FEMU boot, cross-compile, live database, device or runtime test was executed for this report. Gate 9 and Gate 15 are **not** passed by this report.

## Changelog
- **v1.0.0 (2026-11-23):** Initial evidence-first Fuchsia/Zircon interoperability assessment and bounded PoC criteria.
