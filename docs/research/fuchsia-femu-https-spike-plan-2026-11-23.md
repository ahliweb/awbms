# AWBMS — Fuchsia FEMU Capability and HTTPS Edge Spike Plan

## Document control
- **Version:** v1.0.0
- **Updated:** 2026-11-23
- **Status:** SPIKE PLAN, RESEARCH / PROPOSED. **This document is not authorisation to run anything.** It is not an ADR, verification gate or production authorisation. Issue [#34](https://github.com/ahliweb/awbms/issues/34).
- **Parent note:** [Fuchsia/Zircon interoperability and Rust portability readiness](fuchsia-zircon-awbms-portability-readiness-2026-11-23.md) (§7 staged PoC). Architecture options, trust model and use cases are not repeated here.
- **Blocked on:** [Fuchsia Rust SDK policy and locked-dependency portability matrix](fuchsia-rust-sdk-dependency-portability-2026-11-23.md) (issue [#33](https://github.com/ahliweb/awbms/issues/33)); its verdict (NO-GO for native Rust until Fuchsia states external Rust support and the owner authorises) gates §3 and §12 of this plan.
- **Evidence snapshot:** AWBMS `main` commit `59cc2c3ac5705cb8e50476249aad82026569de57`; Fuchsia documentation retrieved 2026-11-23.
- **Execution status:** **NOT RUN.** No SDK installed, no FEMU booted, no component built, no measurement taken. No result in this document is observed.

## 1. Purpose and non-goals

**Purpose [R]:** specify a limited, isolated spike that evaluates Fuchsia user-space component lifecycle, capability (FIDL) routing, and a synthetic HTTPS telemetry client against a mock server. **Non-goals:** porting AWBMS, benchmarking AWBMS, proving driver or hardware support, or touching any business data.

Labels: **[F]** verified in this repository at the snapshot; **[C]** external claim (re-verify at execution); **[A]** assumption; **[R]** recommendation; **[O]** open decision; **[G]** gate evidence required.

## 2. Scope fence (hard exclusions)

The spike MUST contain none of the following **[R]**; any breach aborts it.

| Excluded | Rule |
| --- | --- |
| AWBMS backend code | No file from `crates/` or any AWBMS module is copied, linked or vendored. Only the published, versioned synthetic message contract in §6 is shared. |
| Keys and secrets | Only freshly generated, single-use, synthetic test credentials and a throwaway test CA, created inside the sandbox, never reused elsewhere, destroyed at teardown. |
| Personal data | Payloads are generated constants (e.g. `device-synth-01`); no names, addresses, location, or real identifiers. |
| Database | No PostgreSQL client, no connection string, no storage of results other than evidence files. |
| Robot or physical actuation | No actuator, GPIO, motor, camera or microphone capability is requested or routed. |
| Production host | The mock server runs only on a disposable host-side loopback or a private emulator bridge; no production URL, DNS name or credential appears. |
| AWBMS workspace changes | No Cargo, CI, toolchain or workspace edit; the spike lives in a separate disposable directory or repository. |

## 3. Preconditions and gate chain

All must be recorded in writing before execution; none is satisfied today **[F]**.

1. **#33 verdict resolved:** either Fuchsia publishes a dated statement that external (out-of-tree) Rust authoring is supported, **or** the owner approves a fallback in §4.2.
2. **Fuchsia SDK licensing and support recorded:** licence terms for the SDK/IDK in use, redistribution position, and the support channel. The fetched IDK page states Fuchsia "does not support public usage of the Fuchsia IDK" and that its APIs "are subject to change at any time without notice" **[C]**; the fetched SDK page gives no licence text **[C]**. Status: **unrecorded [O]**.
3. **Owner PoC authorisation recorded** with named executor, host, time box, and teardown date. Status: **absent [O]**.
4. **No bypass of the Stage 1 gates:** the spike's results never mark an existing AWBMS gate PASS, and no implementation is made ahead of Gate 9 (per the parent note).
5. **Environment manifest template ready** (§10).

## 4. Sandbox and artefact design

### 4.1 Emulator sandbox

- **[C]** FEMU is launched with `ffx emu start`; the networking flag `--net` accepts `tap`, `user` (SLiRP port maps), `none`, `auto`, and `auto` is the default, which means networking is **on** unless explicitly disabled. The emulator page also warns about unsupported host CPUs and says ARM64 emulator support is "very limited and not recommended" and the generic `arm64` board does not run in the emulator.
- **[R]** Start with `--net none` for the capability and lifecycle phases (Phases A to C below). Enable `--net user` only for the HTTPS phase (Phase D), after the consent record in §3, with a single port map to the host mock server. This implements "no network until consented mock".
- **[R]** Use an x64 product bundle; record the exact SDK version, product name and checksum in the manifest rather than "latest".
- **[R]** The emulator runs in a disposable user account/VM/container with no mounted AWBMS tree, no cloud credentials and no SSH agent forwarding.

### 4.2 Component under test: minimal synthetic client

**Route A (only if #33 clears external Rust):** a single Rust component, built per the rustc Fuchsia target documentation, containing only: a timer loop that emits a synthetic heartbeat, a TLS client using a candidate pure-Rust stack from the #33 note (proposed, not adopted), and structured logging to the component's own log sink. No AWBMS crate, no `sqlx`, no server stack.

**Route B (if external Rust stays blocked):** a supported-language alternative selected by the owner, for example a C++ component using the SDK's supported toolchain, implementing the **identical** behaviours and the identical synthetic contract. The spike then measures the supported route, and the Linux control (§9) compares like with like. Route B results say nothing about Rust.

**Route C (not chosen by this plan):** building inside the Fuchsia source tree under the platform's own policy. It needs separate licence and governance approval and is out of scope **[O]**.

Behaviours to implement identically in A or B:
- Heartbeat every N seconds (N chosen and recorded before the run; no default is assumed here).
- `POST` of a versioned synthetic event (§6) over HTTPS to the mock server, with an idempotency key and a bounded retry with backoff.
- A deliberate attempt to use one capability that the manifest does not grant (§5.2).
- Clean exit on a stop request; ability to be restarted by the framework.

### 4.3 Packaging and launch **[C]**

The rustc page describes this sequence: create package metadata and a `.manifest` mapping paths, write a CML component manifest, compile it with `cmc compile`, `ffx package build`, `ffx repository create` and `publish`, start the emulator, register the repository, then `ffx component run`. Rerunning may need `--recreate`. These steps are executed only after §3 and are recorded verbatim in the evidence manifest. Nothing here has been run **[F]**.

## 5. Capability model tests

### 5.1 Default-deny manifest

- **[C]** Fuchsia components interact with the system and other components only through routed capabilities; each capability must have a valid route from its source (capabilities page).
- **[R]** The CML manifest requests the minimum set and is committed as evidence: the component's own log sink, the network socket provider **only in Phase D**, and the clock if the chosen runtime needs it. Everything else is absent: no storage, no device, no `fuchsia.net.*` beyond what Phase D lists, no `fuchsia.sys2` administration, no debug protocols.
- **[R]** The manifest is reviewed against a checklist before each phase; every `use` line has a one-sentence justification in the manifest comments or the evidence manifest.

### 5.2 Independent negative test: absent FIDL capability

- **[R]** A second, independent test component (separate package and manifest, not an optional mode of the main component) attempts to connect to a protocol that is **not** routed to it. It must be written and reviewed separately from the main component so that a bug in one cannot hide a failure in the other.
- **Expected result:** connection to the unrouted protocol fails and the failure is logged, and the same protocol connects successfully in a **positive control** component whose manifest does route it. **[A]** The exact runtime error form (for example a closed channel) is not documented on the fetched capabilities page, so the pass rule is "connection unusable and no data obtained", with the observed error recorded rather than pre-asserted.
- Pass rule: negative test denied **and** positive control allowed **and** denial persists after a framework restart. A test that only shows the main component working is not a negative test.
- Also test: attempt to `offer` an ungranted capability to a child in the realm; expected to fail routing.

## 6. Versioned synthetic interface (contract only)

Defined for the spike; it is **not** a new AWBMS API and creates no migration or production coupling **[F]**.

- `POST /spike/v0/events` with JSON: `schema` (`"spike.v0"`), `device_id` (synthetic), `event_id` (UUID chosen by the client), `seq` (monotonic integer), `ts_client`, `kind` (`heartbeat` or `reading`), `value` (synthetic number).
- Response: `202` with `{"status":"accepted","event_id":...}` for a new event; `200` with `{"status":"duplicate"}` for an `event_id` already seen; `401` for an unknown or revoked credential; `409` for an out-of-window replay; `5xx` and delays injected by the mock.
- **[R]** Remember (from the parent note) that `accepted` is not proof of completion; the mock records a durable receipt and the test asserts on the receipt, not on the HTTP status alone.
- The mock server lives host-side in the disposable directory, is configured by a test fixture, and stores state only in memory and its evidence log.

## 7. Network and credential test matrix

All cases run with the same component; the mock server fault-injects. Each case has an ID, an input, an expected observation and a pass rule, and every result row in the evidence manifest cites the case ID.

| ID | Case | Setup | Expected observation | Pass rule |
| --- | --- | --- | --- | --- |
| H1 | Valid TLS | Mock presents a certificate chaining to the throwaway test CA the component trusts | Event accepted; receipt recorded | Receipt count equals sent count |
| H2 | Unknown CA | Mock presents a certificate from a different CA | TLS handshake fails; no HTTP request reaches the mock | Zero requests logged by mock; component logs verification failure; no retry downgrades to plaintext |
| H3 | Hostname mismatch | Certificate valid for another name | Handshake fails | Same as H2 |
| H4 | Expired certificate | Certificate with a past validity window | Handshake fails | Same as H2. **[A]** Depends on the emulator clock being correct, so the manifest records the guest clock source and value. |
| H5 | Plaintext refusal | Client pointed at an `http://` URL | Component refuses | No request sent |
| C1 | Valid machine credential | Fresh synthetic credential bound to `device-synth-01` | Accepted | Receipt recorded |
| C2 | Revoked machine credential | Credential valid in C1 is revoked at the mock mid-run | Next request returns `401`; component stops sending events with that credential and logs the state; it does not retry the same credential in a tight loop | Mock logs `401`; no event receipts after revocation; bounded retry count observed |
| C3 | Stale credential | Credential past its expiry | `401` | Same as C2 |
| C4 | Credential bound to another resource | Credential for `device-synth-02` used with `device-synth-01` payload | Rejected | No receipt; server binds credential to resource, not to a body field |
| D1 | Duplicate event | Same `event_id` sent twice | Second answered `duplicate`; one receipt | Receipt count is 1 |
| D2 | Duplicate after restart | Component restarted between the two sends | Same as D1 | Same as D1 |
| R1 | Replay of an old event | Previously accepted request replayed after the replay window | `409` | No second receipt |
| R2 | Sequence regression | `seq` lower than the last accepted for the device | Rejected or flagged | No receipt of the stale event; flagged in log |
| T1 | Response timeout | Mock delays beyond the client timeout | Client times out, retries with the same `event_id` | Eventually one receipt; no more than the documented retry bound |
| T2 | Slow connect | Mock accepts the TCP connection and stalls TLS | Client gives up within the connect timeout | Time-to-give-up recorded |
| P1 | Network partition | Port map removed or mock stopped for a window, then restored | Component buffers or drops per its policy; reconnects when the mock returns | Behaviour matches the documented policy; recovery time recorded; no duplicate receipts |
| P2 | Partition during send | Cut after request sent, before response | Retry with the same `event_id` | One receipt |
| L1 | Component crash | Component killed | Framework restarts per manifest policy, or does not if none configured | Observed restart behaviour matches the manifest and is recorded; time to first successful event after restart recorded |
| L2 | Orderly stop | Stop request | Clean exit | Exit status and last log line recorded |

Fail closed **[R]**: any case in which an event is accepted without valid TLS and a valid credential is a spike failure and aborts the run.

## 8. Phases and order

| Phase | Network | Content | Exit condition |
| --- | --- | --- | --- |
| A | none | Package build, FEMU start and stop, component run, clean stop (L2), crash and restart (L1) | Manifest and logs captured |
| B | none | Default-deny review, negative test and positive control (§5) | Pass rule of §5.2 |
| C | none | Idempotency and replay logic against an in-process stub (no sockets), if Route A/B permits | Unit-level D1, R1, R2 |
| D | consented `user` mode, single port map to the mock | H, C, D, R, T, P cases | Matrix complete |
| E | Linux control | Same component logic and mock on Linux, same cases (§9) | Metrics collected |
| F | teardown | Destroy emulator, credentials, CA and caches; retain approved evidence only | Teardown record |

Phases may stop early; a failed phase ends the spike unless the owner records a continuation.

## 9. Linux control and metrics

**Purpose:** a like-for-like baseline. The same synthetic behaviour (heartbeat, `POST`, retry policy, same mock server, same fault injections) runs as a Linux process.

**Measured quantities, all with raw samples kept in the evidence manifest (no guessed numbers, no target values proposed here):**

| Metric | Definition | Method requirement |
| --- | --- | --- |
| Request latency p95 and p99 | Client-observed time from request start to final response, per case H1/D1 with enough samples that the percentile is meaningful; the sample count is recorded | Same mock, same payload, same pacing on both platforms; record clock source and resolution |
| Resident memory (RSS) | Peak and steady-state resident memory of the component | Platform-appropriate probe recorded; Fuchsia and Linux definitions differ and the difference is written down |
| Startup time | From launch request to first successful event | Cold start and warm restart reported separately |
| Recovery time | From fault removal (P1, L1) to first successful event | Same fault injected on both platforms |
| Binary and package size | Bytes of the built artefact plus required runtime libraries | Recorded as measured |
| SDK and maintenance cost | Effort and constraints, not a score: SDK acquisition and licence steps, toolchain versions pinned, hours spent per phase (actual), number of documented workarounds, breaking changes encountered between SDK versions, support channel used and response | Reported as a log and prose; no composite number |

Rules **[R]**: report distributions, not single runs; state the sample count and run count; state emulator limitations; label every Fuchsia number "emulator" and never present it as hardware evidence; do not compare numbers across different hosts. If a metric cannot be measured, record **not measured** and why.

## 10. Evidence manifest

One directory per run, never inside the AWBMS tree, retained only as approved synthetic evidence. Contents:

- `manifest.md`: run ID, date, executor, authorisation reference (§3), host OS and kernel, CPU model, `rustc -Vv` and `cargo -V` (Route A) or compiler versions (Route B), SDK version and checksum, product bundle name and checksum, `ffx` version, FEMU flags (including `--net`), guest clock source, and the AWBMS commit the plan was read at.
- `commands.log`: every command with timestamp, stdout, stderr, **exit status** (including failures).
- `components/`: each CML manifest and its review checklist.
- `cases/<ID>.md`: for each §7 case, input, expected, observed, pass or fail, links to log excerpts.
- `mock-receipts.jsonl`: the mock's durable receipt log (synthetic only).
- `metrics/raw/`: raw samples for §9.
- `teardown.md`: what was destroyed and when.
- `sha256sums.txt`: checksums of every file above, so later edits are detectable.

Redaction rule **[R]**: before retention, scan the directory for anything resembling a key, token, host name outside the sandbox or personal data; the credentials in the run are synthetic and destroyed at teardown, so none should remain.

## 11. Risks and rollback

- **[C]** Emulator behaviour does not prove driver or hardware support; no hardware claim may be made from this spike.
- **[C]** SDK and IDK APIs may change without notice; pin versions, and treat any result as valid only for the pinned SDK.
- **[A]** The runtime error for an unrouted capability is not documented in the fetched page; the plan records the observed form rather than assuming it.
- **[A]** FEMU host constraints (CPU support, `tap` versus `user` networking) may block some hosts; record the blocking reason as a result.
- Mock and test CA mistakes are the likeliest cause of a false pass; H2 to H5 exist to catch them.
- **Rollback:** dispose of the sandbox, emulator images, packages, test CA and credentials; retain only the approved synthetic evidence manifest. No AWBMS repository state needs reverting because none is changed.

## 12. Go / no-go

| Condition | Required before native work (any Fuchsia build, SDK install or FEMU run) | Status today |
| --- | --- | --- |
| #33 verdict resolved: external Rust support stated by Fuchsia, or owner-approved Route B | Yes | **Open: NO-GO** |
| Fuchsia SDK licensing and support recorded | Yes | **Not recorded** |
| Owner PoC authorisation recorded with executor, host, time box | Yes | **Not recorded** |
| Scope fence of §2 accepted | Yes | Proposed |
| Environment manifest template ready | Yes | Specified in §10 |

**[R] Decision today: NO-GO for execution and for any native Fuchsia work.** The block lifts only when every row above is recorded in writing; this document does not lift it. A GO would still authorise only the spike in this plan, not a port, not a dependency in AWBMS, and not a Stage 1 gate PASS.

Post-spike go/no-go for any further work requires a formal ADR once measured results, product demand, legal constraints and support plan exist, as the parent note states.

## Acceptance criteria trace

| Issue #34 criterion | Satisfied by |
| --- | --- |
| PoC scope contains no AWBMS backend code, keys, personal data, DB access, robot actuation or production host | §2 |
| Independent negative test for absent FIDL capability; HTTPS server validation and revoked machine credential tests specified | §5.2 (negative test and positive control), §7 cases H1 to H5, C1 to C4 |
| Synthetic event duplicate/replay/timeout/partition cases and log/evidence manifest defined | §6, §7 cases D, R, T, P, L, §10 |
| Linux control comparison with measured p95/p99, RSS, startup, recovery and SDK/maintenance cost; no guessed scores | §9 |
| Go/no-go blocks native work until SDK licensing/support and owner PoC authorisation are recorded | §3, §12 |
| Blocked-on relation to #33 (issue section 3) | Document control, §3, §12 |

## Changelog
- **v1.0.0 (2026-11-23):** Initial spike plan for issue #34; not run, not authorised.
