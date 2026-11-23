# AWBMS — Fuchsia Rust SDK Policy and Locked-Dependency Portability Matrix

## Document control
- **Version:** v1.0.0
- **Updated:** 2026-11-23
- **Status:** RESEARCH / PROPOSED. Not an ADR, verification gate, or production authorisation. Issue [#33](https://github.com/ahliweb/awbms/issues/33).
- **Parent note:** [Fuchsia/Zircon interoperability and Rust portability readiness](fuchsia-zircon-awbms-portability-readiness-2026-11-23.md). This note does not repeat its architecture options, trust model, use cases or legal list; it only discharges the "F0 / research only" and "Feasibility gate" steps of its §7 for the locked dependency graph.
- **Sibling:** [Fuchsia FEMU HTTPS spike plan](fuchsia-femu-https-spike-plan-2026-11-23.md) (issue [#34](https://github.com/ahliweb/awbms/issues/34)), which is blocked on the verdict in §9 below.
- **Evidence snapshot:** AWBMS `main` commit `59cc2c3ac5705cb8e50476249aad82026569de57`; external documentation retrieved **2026-11-23** (§7).
- **Scope:** source-provenance matrix only. No Cargo, workspace, API or CI change. No Fuchsia SDK installation, cross-compile, FEMU boot or runtime test was executed.

## 1. Verdict in one paragraph

**[R] NO-GO for any native Fuchsia Rust work on AWBMS code until a written, current Fuchsia statement of external (out-of-tree) Rust support exists and the owner records authorisation.** The rustc `*-unknown-fuchsia` targets are Tier 2 **[C]**, but the Fuchsia project's own language policy says Rust is "not supported for end-developers" **[C]**, and its IDK page says Fuchsia "does not support public usage of the Fuchsia IDK" **[C]**. These are different layers of support and the gap is the central finding (§4). At source level, the transport and TLS stack AWBMS locks (Tokio, Mio, socket2, libc, getrandom, ring) contains explicit Fuchsia handling **[F]** (§6), which makes a build attempt plausible but proves nothing about linking, packaging or runtime. No Fuchsia SDK or runtime dependency enters the AWBMS core (§8).

## 2. Evidence labels

- **[F]** Verified in this repository at the commit above, or in the unpacked source of a crate whose SHA-256 equals the `checksum` recorded in `Cargo.lock` (§5.2).
- **[C]** External claim from the documents in §7; not an AWBMS fact. Re-verify at each PoC.
- **[A]** Assumption, not established.
- **[R]** Recommendation. **[O]** Open decision. **[G]** Evidence that would discharge a claim.

This note registers no new `C-`, `AS-`, `OD-` or `VG-` identifier in the architecture validation; the labels are local to this document.

## 3. Exact toolchain and target versions

| Item | Value | Label | Source |
| --- | --- | --- | --- |
| Rust toolchain | channel `1.98.0`, profile `minimal`, components `clippy`, `rustfmt` | [F] | `rust-toolchain.toml` |
| Edition / MSRV | edition 2024, `rust-version = "1.98.0"`, resolver 3 | [F] | `Cargo.toml` |
| Fuchsia targets in the pinned toolchain | **none installed**: `profile = "minimal"` and no `targets = [...]` key | [F] | `rust-toolchain.toml` |
| Workspace members | one: `awbms-verification` (`publish = false`) | [F] | `Cargo.toml` |
| Direct dependencies (exact pins) | axum `=0.8.9`, sqlx `=0.9.0` (no default features; `runtime-tokio`, `tls-rustls`, `postgres`), tokio `=1.53.1` (`macros`, `rt-multi-thread`, `signal`, `net`, `sync`, `time`), tower `=0.5.3` (`util`), tower-http `=0.7.0` (`limit`, `request-id`, `timeout`) | [F] | `Cargo.toml` |
| Lock file | `Cargo.lock` format `version = 4`, 185 packages, all 185 reachable from `awbms-verification` | [F] | `Cargo.lock`, graph walk in §5.1 |
| Official Fuchsia rustc targets | `aarch64-unknown-fuchsia`, `x86_64-unknown-fuchsia` are Tier 2; a `riscv64gc-unknown-fuchsia` triple appears only in the source-build instructions with no tier stated | [C] | rustc book, §7 |
| Fuchsia SDK version needed | page says "a recent copy of the Fuchsia SDK"; compiler test suite minimum `20.20240412.3.1`; clang 14+ for source builds | [C] | rustc book, §7 |

Consequences **[F]**: a plain `rustup target add x86_64-unknown-fuchsia` is a deliberate deviation from the pinned toolchain profile and must happen only in a throwaway PoC checkout (§10), never in the repository's `rust-toolchain.toml`.

## 4. Mismatch: rustc Tier 2 versus external Rust authoring policy

| Layer | What it says | Label | What it does **not** say |
| --- | --- | --- | --- |
| rustc platform support | Targets are Tier 2, support `std`, binaries are ELF, built by cross-compiling against "a recent copy of the Fuchsia SDK"; the page includes cargo, packaging (`ffx package build`) and emulator (`ffx emu start`, `ffx component run`) steps | [C] | Does not state that the Fuchsia project supports external Rust developers or that SDK APIs are stable. The page does not define host-tool support. |
| Fuchsia Programming Language Policy | "Rust is approved for use throughout the Fuchsia Platform Source Tree" except the kernel; for end-developers (SDK) the table says Rust is "Not supported"; "None of our current end-developers use Rust."; the policy is an allowlist | [C] | The fetched page shows no publication or revision date. |
| Fuchsia IDK page | "Fuchsia does not support public usage of the Fuchsia IDK."; "not suitable for immediate consumption"; "APIs in the IDK are subject to change at any time without notice." | [C] | Lists no supported languages. |
| Fuchsia SDK page | "The APIs in the SDK are subject to change without notice." | [C] | The fetched text names no languages, host OS list or licence. |
| Fuchsia Rust SDK sample repo | README: "Rust is not yet supported in the Fuchsia SDK." Latest commit shown as "3 years, 10 months ago" (relative, not a calendar date) | [C] | May be stale; it is a repository banner, not a policy. |
| Fuchsia F31 release notes (2026-07-22) | Many Rust entries (FIDL Rust wire types, `fuchsia-component` structured config, `fx create driver --lang rust`, Rust storage-host) and some "out-of-tree" entries, none of which is Rust-specific for out-of-tree | [C] | Shows Rust is heavily used **in-tree**; shows no change to the end-developer policy. |

**Reading [R]:** Tier 2 means the Rust project builds and tests `std` for the triple; it is a statement about the compiler, not about Fuchsia's commitment to out-of-tree Rust authors. The AWBMS decision rule is therefore: **external Rust authoring is unsupported until Fuchsia says otherwise in a source that is current and dated.** The policy page may lag source-level work (as the parent note already warns), but a lagging policy is not a licence to assume support; and a Tier 2 target must never be cited in an AWBMS document as "Fuchsia supports Rust apps".

Allowed fallbacks if the native-Rust route stays blocked **[R]**: (a) a supported-language (C++) adapter in a separate repository, (b) a Linux gateway speaking the neutral HTTPS contract, (c) an upstream-source-tree-only research spike under separate licence and governance approval. None enters AWBMS.

## 5. Method and limits

### 5.1 Dependency graph

**Deviation from the issue's wording [F]:** the issue asks for read-only `cargo metadata --locked --format-version 1`. The agent environment that produced this note has **no Rust toolchain installed** (`cargo` not found), so that command was **NOT EXECUTED**. Substitute method: parse `Cargo.lock` (TOML) and walk `dependencies` from `awbms-verification`; this yields the full resolved package set (185 packages, 185 reachable) but **not** per-target or per-feature activation, which only `cargo metadata` / `cargo tree --target` can show. Re-run both in the PoC (§10) and diff against this note.

### 5.2 Crate source inspection

For each crate in the matrix, the `.crate` tarball of the **exact locked version** was downloaded from the crates.io static distribution host, its SHA-256 compared with the `checksum` in `Cargo.lock` (all 35 downloaded tarballs matched **[F]**), unpacked into a throwaway directory outside the repository, and searched for `fuchsia` and `target_os = "fuchsia"`. No crate was compiled or run. "Source evidence" below means a match in that text, nothing more.

### 5.3 Disposition rules

- **supported** — the crate source contains explicit Fuchsia handling for the capability AWBMS uses. This is source evidence, **not** a compile or run result.
- **unsupported** — the source shows Fuchsia is excluded, stubbed, or the functionality AWBMS needs is absent.
- **unverified** — no Fuchsia-specific code found (portable by construction at best) or the evidence is mixed. This is the default; absence of a `cfg` is not evidence of support.

Categories: **Build** = compiles/links for the Fuchsia triple (needs SDK sysroot); **Run** = behaves correctly in a Fuchsia process (needs FEMU/device); **Package** = ships as a Fuchsia component package (CML manifest, capabilities, library availability).

## 6. Per-crate disposition matrix

Versions are those in `Cargo.lock` **[F]**. "Fuchsia evidence" is from §5.2.

| Crate (locked version) | Role for AWBMS | Fuchsia evidence in source | Build | Run | Package |
| --- | --- | --- | --- | --- | --- |
| tokio 1.53.1 | direct; runtime, `net`, `signal`, `time` | 45 `target_os = "fuchsia"` occurrences, all in `net` socket-option gating (`tcp/socket.rs`, `udp.rs`); runtime I/O driver delegates to mio | supported | unverified (`signal` feature: see signal-hook-registry) | unverified (needs no extra library beyond libc) |
| mio 1.2.4 | tokio's I/O driver | `selector/poll.rs` (poll(2)) chosen for fuchsia; `eventfd` waker chosen for fuchsia; unix `pipe`, `tcp` gating | supported | unverified (poll selector is the non-default, less-exercised path; eventfd availability on the Fuchsia libc is [A]) | unverified |
| socket2 0.6.5 | tokio/hyper-util sockets | 66 occurrences; `x86_64-unknown-fuchsia` listed in its docs.rs target list | supported | unverified | unverified |
| libc 0.2.191 | FFI bindings | `fuchsia/` module with `aarch64.rs`, `x86_64.rs`, `riscv64.rs`; `new/fuchsia/` | supported | n/a (declarations only) | n/a |
| getrandom 0.2.17 | via `rand`/`ring` older paths | `fuchsia.rs` calls `zx_cprng_draw` with `#[link(name = "zircon")]` | supported (link needs SDK `zircon` library) | unverified | unverified: ties the binary to libzircon from the SDK sysroot |
| getrandom 0.4.3 | via `rand` 0.10 / `uuid` | `backends/fuchsia.rs`, same `zx_cprng_draw` + `#[link(name = "zircon")]` | supported (same link caveat) | unverified | unverified (same) |
| ring 0.17.14 | rustls crypto provider (selected by sqlx features, §6.1) | `build.rs` lists `fuchsia` in `LINUX_ABI`; `rand.rs` uses getrandom on fuchsia; `cpu/arm/fuchsia.rs` for aarch64 | unverified (C/asm via `cc`; needs a Fuchsia-targeting C compiler from the SDK; source suggests the asm path is expected) | unverified | unverified |
| cc 1.11.1 | build-time helper for ring | Target table has `aarch64`/`x86_64`/`riscv64gc-unknown-fuchsia` | supported (host tool) | n/a | n/a |
| rustls 0.23.46 | TLS client for sqlx | no OS-specific code found | unverified | unverified | unverified (trust-anchor source is webpki-roots, no OS store) |
| rustls-webpki 0.103.16, rustls-pki-types 1.15.1, webpki-roots 1.0.11, untrusted 0.9.0 | certificate validation, bundled roots | none | unverified | unverified (wall-clock time via std; FEMU clock behaviour untested) | unverified |
| hyper 1.12.0 | HTTP server under axum | none | unverified | unverified | unverified |
| hyper-util 0.1.21 | server/runtime glue | fuchsia cfg only in `client/legacy/connect/http.rs` (socket options), which the verification server path does not use | unverified (server path) | unverified | unverified |
| axum 0.8.9, tower 0.5.3, tower-http 0.7.0, tower-layer/service | HTTP framework and middleware | none | unverified | unverified | unverified |
| sqlx 0.9.0, sqlx-core 0.9.0, sqlx-postgres 0.9.0 | PostgreSQL client with tenant transaction | none; `net/socket` gates unix sockets on `cfg(unix)` | unverified | **out of PoC scope**: requires a reachable PostgreSQL; the spike must not touch a database | unverified |
| signal-hook-registry 1.4.8 | tokio `signal` feature | none; uses `libc` `sigaction` on any non-Windows target | unverified | unverified (Fuchsia libc signal semantics are [A]; graceful shutdown must not rely on POSIX signals there) | unverified |
| parking_lot_core 0.9.12 | lock internals | none; generic `cfg(unix)` thread parker | unverified | unverified | unverified |
| cpufeatures 0.2.17 / 0.3.1 | CPU feature detection for SHA-2/ChaCha | none (arch-level `cfg` only) | unverified | unverified | unverified |
| whoami 2.1.3 | sqlx default username | Fuchsia only as a platform-name enum; user/host lookup falls through to a stub implementation | supported (compiles to stub) | **unsupported** for real username/hostname lookup; supply an explicit user in config | n/a |
| etcetera 0.11.0 | sqlx config dirs | declared only under `cfg(windows)` | not built on Fuchsia | n/a | n/a |
| rand 0.10.3, chacha20 0.10.2, uuid 1.29.1, once_cell 1.21.4, crossbeam-utils 0.8.23, tokio-stream 0.1.19, cmov 0.5.4 | misc | none | unverified | unverified | unverified |
| windows-*, wasm-bindgen*, js-sys, wasi 0.11.1, redox_syscall 0.5.18 | target-gated transitive crates | not applicable | not expected to be built for Fuchsia ([A] from `cfg` gating; confirm with `cargo tree --target`) | n/a | n/a |

Remaining locked packages (pure-Rust utilities such as `bytes`, `http`, `serde*`, `memchr`, `idna`/`icu_*`, `syn`/`quote`/`proc-macro2`, `thiserror`, `tracing*`, `sha2`, `hkdf`, `hmac`, `digest`, `futures-*`) were **not individually inspected**. Disposition: **unverified**. Proc-macro and build-script crates run on the host, not the Fuchsia target.

### 6.1 TLS and runtime feature choice **[F]**

`Cargo.toml` enables sqlx `tls-rustls`; `Cargo.lock` contains `ring`, `rustls`, `rustls-webpki`, `webpki-roots` and **no** `aws-lc-rs` or `rustls-native-certs`, so the resolved TLS provider is `ring` with bundled roots. Implications **[R]**: (a) the build-side risk concentrates in `ring`'s C/asm compilation against the Fuchsia sysroot; (b) bundled roots avoid an OS trust store but make root rotation a release matter; (c) `aws-lc-rs` would add a larger C/CMake surface and must not be introduced for this research. Note that the verification server speaks plain HTTP on loopback in tests **[F]**; TLS on Fuchsia would be exercised only by the spike's mock HTTPS client.

### 6.2 What the matrix does and does not show

- **[F]** The crates on the build-critical path (tokio, mio, socket2, libc, both getrandom versions, ring, cc, and the hyper-util client path) carry explicit Fuchsia code.
- **[F]** No crate in the HTTP framework layer (hyper, axum, tower, tower-http) or the database layer (sqlx*) contains Fuchsia-specific code; that is expected for portable crates and is **not** evidence of support.
- **[A]** Nothing here shows the graph compiles, links, or runs. Linker flags, `-Lnative=` SDK paths, libzircon/libc availability, and `panic` strategy are untested.
- A realistic native client would **not** reuse the whole verification graph: axum/hyper server stack and sqlx are server concerns. A minimal HTTPS client component needs far fewer crates (§10, candidate list **proposed, not adopted**).

### 6.3 Licence and provenance

- **[C]** Fuchsia source `LICENSE` text matches BSD 2-Clause, "Copyright 2019 The Fuchsia Authors" (retrieved 2026-11-23). SDK/IDK licence and redistribution terms were **not** found in the fetched pages: **[O]** record them before any SDK use.
- **[O]** The AWBMS workspace declares `license = "MIT"` while the repository licence is undecided (per the parent note, owned by issue #7). Redistribution of any Fuchsia-linked artifact is undecided.
- **[R]** Run `cargo deny`/`cargo audit` style licence and advisory checks in the PoC against the locked graph (not executed here).
- **[R]** No unreviewed native FFI: `ring` (C/asm), `getrandom` (`zx_cprng_draw` via libzircon) and `libc` are the only native boundaries identified; any new FFI crate for Fuchsia (FIDL bindings, `fuchsia-zircon`) requires separate review.

## 7. Sources and retrieval log

All retrieved **2026-11-23** by web fetch/search; the fetch tool summarises pages, so quotes are as returned and must be re-read in the original before they are relied on.

- [rustc book — aarch64-unknown-fuchsia and x86_64-unknown-fuchsia](https://doc.rust-lang.org/rustc/platform-support/fuchsia.html) — Tier 2, `std`, SDK need, cargo/packaging/emulator steps. [C]
- [Fuchsia Programming Language Policy](https://fuchsia.dev/fuchsia-src/contribute/governance/policy/programming_languages) — Rust platform-tree vs end-developers; no date shown. [C]
- [Fuchsia IDK](https://fuchsia.dev/fuchsia-src/development/idk) — no public usage support; APIs may change. [C]
- [Fuchsia SDK](https://fuchsia.dev/fuchsia-src/development/sdk) — SDK definition; API-change caution. [C]
- [Fuchsia release notes F31](https://fuchsia.dev/whats-new/release-notes/f31) — dated July 22, 2026; Rust in-tree activity; only the first 100000 of 104326 characters were read. [C]
- [Fuchsia SDK sample, getting-started-rust](https://fuchsia.googlesource.com/sdk-samples/getting-started-rust/) — README banner. [C]
- [Fuchsia `LICENSE`](https://fuchsia.googlesource.com/fuchsia/+/refs/heads/main/LICENSE) — BSD-style text, Fuchsia Authors. [C]
- [Fuchsia FEMU](https://fuchsia.dev/fuchsia-src/development/build/emulator) and [component capabilities](https://fuchsia.dev/fuchsia-src/concepts/components/v2/capabilities) — used in the sibling spike plan.
- AWBMS files at the snapshot commit: `rust-toolchain.toml`, `Cargo.toml`, `Cargo.lock`, `crates/verification/Cargo.toml`.
- Crate sources: the 35 locked-version tarballs inspected (§5.2), checksum-matched to `Cargo.lock`.

One URL (`fuchsia.dev/fuchsia-src/get-started/sdk`) returned 404 and a web search found no 2026 statement that specifically covers Rust for out-of-tree SDK users; that absence is itself recorded as **unresolved [O]**, not as proof either way.

## 8. Boundary: nothing enters the AWBMS core

- **[R]** Fuchsia SDK, IDK, FIDL bindings, `fuchsia-*` crates and Fuchsia toolchains are **not** dependencies of the AWBMS workspace, of any crate under `crates/`, of CI, or of `rust-toolchain.toml`. Existing `VG-15` already forbids source or runtime coupling to external systems of this kind for AWCMS; the same discipline is applied here.
- **[R]** Any PoC lives in a separate throwaway repository or directory with its own toolchain file, no AWBMS source, no secrets, no database access.
- **[R]** Any later porting recommendation needs measured exit criteria, at minimum: reproducible package build from a pinned SDK version; capability-denial test passing; mock-HTTPS validation passing; Linux-control comparison of p95/p99, RSS, startup and recovery with raw numbers; recorded SDK licence and support statement; recorded owner authorisation. No threshold values are proposed here because none have been measured.

## 9. Go / no-go

| Question | Answer | Label |
| --- | --- | --- |
| Does the locked graph have source-level Fuchsia handling on its build-critical path? | Largely yes (§6.2) | [F] |
| Has it been compiled, linked, packaged or run for Fuchsia? | No | [F] |
| Is external Rust authoring supported by Fuchsia policy? | Not per the policy page and IDK page retrieved today | [C] |
| Is the SDK licence/support statement recorded? | No | [O] |
| **Native Rust PoC** | **NO-GO** until the two [O] items above are closed and the owner authorises | [R] |
| **Research-only next step** | Allowed: documentation refresh, and a source-only re-verification when Fuchsia publishes a dated statement | [R] |

## 10. Reproducible commands for a FUTURE isolated PoC — NOT EXECUTED

**Everything in this section is NOT EXECUTED. It is a procedure, not a result. No output, exit code or timing has been observed. Do not paste these into AWBMS CI or the AWBMS workspace.** Prerequisites: PoC authorisation recorded (§9), an isolated disposable directory outside the AWBMS checkout, a Fuchsia SDK obtained and licensed per §6.3, no network credentials.

1. Record the environment manifest (host OS and kernel, `rustc -Vv`, `cargo -V`, SDK version string, clang version, `ffx` version, the date, and the AWBMS commit whose `Cargo.lock` is copied).
2. Copy only `Cargo.lock`, `Cargo.toml` and `rust-toolchain.toml` into the isolated directory; do not copy AWBMS source.
3. Read-only graph facts, host target:
   ```bash
   cargo metadata --locked --format-version 1 > metadata.json 2> metadata.stderr; echo "exit=$?" > metadata.status
   cargo tree --locked -e features,no-dev > tree-host.txt
   ```
4. Read-only graph facts, Fuchsia target (shows which target-gated crates activate):
   ```bash
   cargo tree --locked --target x86_64-unknown-fuchsia -e features,no-dev > tree-fuchsia.txt
   ```
5. Tiny crate check (pure-Rust, no AWBMS code), then the candidate client stack. Candidate crates are **proposed, not adopted**: `rustls` with the `ring` provider, `webpki-roots`, a small HTTP/1.1 client, `getrandom`. Compare each choice against §6.
   ```bash
   rustup target add x86_64-unknown-fuchsia --toolchain 1.98.0
   # .cargo/config.toml per the rustc Fuchsia page: -Lnative=<SDK_PATH>/arch/x64/lib and -Lnative=<SDK_PATH>/arch/x64/sysroot/lib
   cargo build --locked --target x86_64-unknown-fuchsia 2> build.stderr; echo "exit=$?" > build.status
   ```
6. Record stderr and exit status for every command, including failures. A failed link on `ring` or `getrandom` is a valid, valuable result.
7. Package and launch steps (`ffx package build`, `ffx repository publish`, `ffx emu start`, `ffx component run`) belong to the sibling [spike plan](fuchsia-femu-https-spike-plan-2026-11-23.md) and require its own approval.
8. Supply-chain checks: licence and advisory scan of the locked graph; list every crate with a `build.rs` or native code.

Success is never inferred from the build alone; a green build proves compile and link only.

## 11. Residual risks

- SDK policy, GN/Bazel packaging and Fuchsia toolchains move quickly; this note decays. **Fail closed** if external Rust use is unsupported at the time of any PoC.
- The fetch tool summarises pages; exact policy wording must be re-read at the source.
- The graph walk cannot show feature or target activation; `cargo metadata` was not run.
- Only 35 crates were source-inspected; the rest are unverified, not supported.
- Rollback: withdraw this proposal; there are no product effects.

## Acceptance criteria trace

| Issue #33 criterion | Satisfied by |
| --- | --- |
| Exact source/toolchain/target versions and external SDK policy evidenced by official citations | §3 (toolchain, pins, snapshot commit, targets), §4 (policy layers), §7 (sources and retrieval date) |
| Per-crate disposition supported/unsupported/unverified with build-vs-run vs package categories | §5.3 (rules, categories), §6 (matrix), §6.1 (TLS/runtime features) |
| Mismatch between rustc Tier 2 and Fuchsia external Rust authoring policy explicitly captured | §1, §4 |
| No Fuchsia SDK/source runtime dependency in AWBMS core; porting recommendation has measured exit criteria | §8, §9 |
| Reproducible commands for a future isolated PoC without presenting unexecuted commands as results | §10 (marked NOT EXECUTED), §5.1 (what was and was not run) |

## Changelog
- **v1.0.0 (2026-11-23):** Initial SDK-policy and locked-dependency portability matrix for issue #33.
