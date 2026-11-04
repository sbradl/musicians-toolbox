# Rust core <-> Kotlin (Compose Multiplatform: JVM desktop + Android) bridge options

Researched 2026-11-04. Primary sources linked inline; full list at the end. "Unverified" means no primary source was found.

## Question

How can shared Kotlin code (one API, used by JVM desktop on Windows/Linux and by Android) call a Rust core? What does each option cost in per-call overhead, bulk data, callbacks/streams, async, threading, errors, memory ownership, tooling and licence? Performance decides. macOS/iOS are not targets, but nothing should rule them out.

## TL;DR + recommendation

- **Desktop JVM and Android both run JNI, so any JNI- or JNA-based binding works on both from one Kotlin source set.** FFM (Panama) is desktop only. ART has no `java.lang.foreign`; the only port found is the third-party [PanamaPort](https://github.com/vova7878/PanamaPort).
- **UniFFI is the strongest generator, but its stable Kotlin backend is JNA-based and serializes non-primitive arguments into `RustBuffer`.** It switched to JNA *direct mapping* in v0.30.0 ([CHANGELOG](https://raw.githubusercontent.com/mozilla/uniffi-rs/main/CHANGELOG.md)). Since v0.32.0 (2026-06-30), `&[u8]` arguments are zero-copy (Kotlin passes a direct `ByteBuffer`). An unreleased **experimental JNI Kotlin generator** (`uniffi-bindgen-kotlin-jni`) claims "large improvements in performance" but publishes no numbers and says "use at your own risk".
- **Gobley (the KMP/Gradle layer for UniFFI) is MPL-2.0, stuck on UniFFI 0.29.5, and has not released since 0.3.7 (2025-10-08).** Its last commits are from 2026-10 ([commits](https://github.com/gobley/gobley/commits/main)). Because desktop and Android are both JVM, plain upstream UniFFI Kotlin plus cargo-ndk plus a small Gradle task is enough; Gobley is optional.
- **Recommendation: a hybrid.**
  1. **General API (view models, commands, errors, async, callbacks): UniFFI** with upstream Kotlin bindings, shared between `jvmMain` and `androidMain` (for example a `jvmCommon` intermediate source set).
  2. **Bulk view data: FlatBuffers** (same as the Go predecessor). Kotlin allocates a direct `ByteBuffer` and passes it as `&mut [u8]` / `[ByMutRef] bytes`. That is unreleased upstream; until it ships, use `&[u8]` plus a Rust-side copy, or the raw path below. Rust writes into it, Kotlin reads in place, and Rust never hands out memory.
  3. **Per-frame hot path (scalar beat query): one hand-written `extern "system"` JNI function** via [jni-rs 0.22](https://docs.rs/jni/latest/jni/) that takes and returns only primitives. On Android, annotate it `@CriticalNative` and register it with `RegisterNatives`. Android docs say such a call costs "almost as cheap as a non-inline call in C/C++" on Android 12+ ([JNI tips](https://developer.android.com/training/articles/perf-jni)). On desktop it is a plain JNI static native.
  4. Re-evaluate once UniFFI's JNI generator is released: it could absorb step 3.
- **Licences are fine for shipping.** JNA can be taken under Apache-2.0. The generated UniFFI code and runtime are MPL-2.0 (file-level copyleft, not GPL; see open questions). jextract is GPL-2.0, but it is a build tool and only matters if FFM is used. cbindgen is MPL-2.0, also a build tool.

## Options

### 1. UniFFI (mozilla/uniffi-rs), current v0.32.1 (2026-09-08)

- **How it works:** proc-macros or UDL in Rust generate a C ABI plus Kotlin bindings. The stable Kotlin backend loads the cdylib through JNA, using direct mapping since [v0.30.0, 2025-10-08](https://raw.githubusercontent.com/mozilla/uniffi-rs/main/CHANGELOG.md). The same Kotlin file runs on desktop JVM and Android.
- **Per-call cost:** primitives (ints, floats, bool) pass as raw values. Strings, bytes, records, enums, options, sequences and maps are serialized into a `RustBuffer` ([lifting/lowering](https://mozilla.github.io/uniffi-rs/latest/internals/lifting_and_lowering.html); the format is "designed mainly for simplicity"). Every call also passes a `RustCallStatus` out-parameter ([rust_calls](https://raw.githubusercontent.com/mozilla/uniffi-rs/main/docs/manual/src/internals/rust_calls.md)).
  - No official ns figures exist.
  - Third parties: [BoltFFI's README](https://docs.rs/crate/boltffi/latest/source/README.md) reports a UniFFI noop at 1,416 ns. That is a competitor's number, the language is unspecified, and it is unverified. [tweedegolf/java-interop](https://github.com/tweedegolf/java-interop) says UniFFI Kotlin bindings "seem slow compared to the other bindings", probably because of JNA.
- **Bulk data:** `Vec<u8>` returns are copied via `RustBuffer`. Since v0.32.0, `[ByRef] bytes` / `&[u8]` arguments travel as `ForeignBytes` (pointer and length), zero-copy, from a **direct** `ByteBuffer`. Unreleased: zero-copy `&mut [u8]` (`[ByMutRef] bytes`) for sync functions; this does not work in async functions ([CHANGELOG](https://raw.githubusercontent.com/mozilla/uniffi-rs/main/CHANGELOG.md)).
- **Callbacks/streams:** foreign traits (callback interfaces are "soft deprecated"). Kotlin implementations must be thread-safe because Rust may call them from any thread. A callback error that doesn't fit the trait's `Result` panics unless the error type implements `From<UnexpectedUniFFICallbackError>` ([foreign traits](https://raw.githubusercontent.com/mozilla/uniffi-rs/main/docs/manual/src/foreign_traits.md)). The callback ABI was reworked for speed in v0.24.
- **Async:** Rust `async fn` becomes Kotlin `suspend fun`. The foreign side drives polling and no Rust event loop is required; `async_runtime = "tokio"` is optional. There is **no built-in cancellation** ([futures](https://mozilla.github.io/uniffi-rs/latest/futures.html)).
- **Threading:** callbacks into Kotlin go through JNA, which attaches native threads (see JNA).
- **Errors:** `Result<T, E>` with `#[uniffi::Error]` becomes a typed Kotlin exception. Panics are caught with `catch_unwind` and surface as an internal exception (code 2). Not caught when `panic = "abort"` ([rust_calls](https://raw.githubusercontent.com/mozilla/uniffi-rs/main/docs/manual/src/internals/rust_calls.md), [errors](https://mozilla.github.io/uniffi-rs/latest/types/errors.html)).
- **Memory:** objects are `Arc::into_raw` handles with generated clone/free functions ([object refs](https://raw.githubusercontent.com/mozilla/uniffi-rs/main/docs/manual/src/internals/object_references.md)). Kotlin wrappers are `AutoCloseable` and need `close()`/`use {}`, with a Cleaner as backstop. Config `android_cleaner` uses Android's `SystemCleaner` ([lifetimes](https://mozilla.github.io/uniffi-rs/latest/kotlin/lifetimes.html), [config](https://mozilla.github.io/uniffi-rs/latest/kotlin/configuration.html)).
- **Tooling:** `uniffi-bindgen generate --library` produces the bindings. There is no official Gradle plugin, so a Gradle task runs cargo/cargo-ndk.
- **Licence:** MPL-2.0. Generated code embeds UniFFI templates; check the header (unverified whether generated output carries MPL).

### 1a. Gobley (formerly trixnity uniffi-kotlin-multiplatform-bindings)

- **Status:** the trixnity repo now returns 404. Gobley says the original is "no longer maintained" and continues it as a fork ([gobley README](https://github.com/gobley/gobley)).
- **What it adds:** Gradle plugins `dev.gobley.cargo`, `dev.gobley.uniffi`, `dev.gobley.rust` ([docs](https://gobley.dev/docs/gradle-plugins)) that build Rust per Kotlin target. Targets are Android, Kotlin/JVM and Kotlin/Native; WASM is not supported.
  - Cross-compile tips: Zig as linker for Linux; NDK r28+ for 16 KB pages; JNA 5.17 or newer required on Android ([tips](https://gobley.dev/docs/cross-compilation-tips)).
  - Windows-from-Linux cross builds are not documented.
- **Versions:** latest is 0.3.7 (Maven/crates, 2025-10-08) on UniFFI 0.29.x; it moved to 0.29.5 in 2026-01 ([crates.io](https://crates.io/api/v1/crates/gobley-uniffi-bindgen), [commits](https://github.com/gobley/gobley/commits/main)). It therefore **lacks UniFFI 0.30 direct mapping (it has its own option) and 0.32 zero-copy bytes.**
- **Licence:** MPL-2.0.
- **Verdict:** only worth it if Kotlin/Native (iOS) becomes a target. For JVM plus Android, upstream UniFFI is newer and faster.

### 2. Hand-written JNI (jni-rs crate, v0.22.4, 2026-03-16, MIT OR Apache-2.0)

- **Per-call cost:** the lowest of the generic options. Desktop: JNI about 57 ns versus FFM about 50 ns versus JNA about 1,350 ns for one native call (OpenJDK 19, Windows, i7-10610U; [zakgof/java-native-benchmark](https://github.com/zakgof/java-native-benchmark), 2023). Android: `@FastNative`/`@CriticalNative` cut the transition further. `@CriticalNative` allows only primitive arguments, needs `RegisterNatives` on Android 8–11, and must not run long because it blocks the GC ([JNI tips](https://developer.android.com/training/articles/perf-jni)).
- **Bulk data:** `NewDirectByteBuffer` / `GetDirectBufferAddress` give zero-copy shared memory. `Get/Set<Type>ArrayRegion` copies with no pinning. Android advice: "If there's no clear winner, use a direct byte buffer" ([JNI tips](https://developer.android.com/training/articles/perf-jni)).
- **Callbacks:** cache the `JavaVM`, global-ref the class and method IDs, then call methods. Native threads must `AttachCurrentThread` and detach before they exit. `FindClass` on an attached native thread uses the system class loader, so look up and cache classes in `JNI_OnLoad` ([JNI tips](https://developer.android.com/training/articles/perf-jni)).
  - jni-rs 0.22 changes the API: `EnvUnowned`, `with_env` with an `AttachGuard`, and `bind_java_type`/`native_method`/`jni_mangle` macros that reduce glue ([docs.rs](https://docs.rs/jni/latest/jni/)).
- **Async:** do-it-yourself. Typically Rust completes a Kotlin `CompletableDeferred` through a callback, or Kotlin wraps a blocking call in `withContext(Dispatchers.IO)`.
- **Errors:** `with_env` wraps the closure in `catch_unwind`. An `ErrorPolicy` such as `ThrowRuntimeExAndDefault` turns an error or panic into a Java exception ([docs.rs](https://docs.rs/jni/latest/jni/)).
- **Memory:** fully manual: local versus global refs, and handles stored as `long` (`Box::into_raw`).
- **Tooling:** cargo-ndk 4.1.2 (Apache-2.0 OR MIT) for Android ABIs. Desktop builds per target triple. No codegen for the Kotlin side beyond the jni-rs macros.
- **Cost:** high glue effort per function, and signature mistakes crash at runtime (CheckJNI helps on Android).

### 3. JNA (v5.19.1; licence LGPL-2.1+ **or** Apache-2.0)

- **Licence:** "You can freely decide which license you want to apply". Apache-2.0 applies from 4.0 onward ([LICENSE](https://raw.githubusercontent.com/java-native-access/jna/master/LICENSE)), so choose Apache-2.0 and no LGPL obligation applies.
- **Per-call cost:**
  - JNA FAQ: interface mapping is "~10X" slower than custom JNI, and direct mapping offers "performance near that of custom JNI" ([FAQ](https://raw.githubusercontent.com/java-native-access/jna/master/www/FrequentlyAskedQuestions.md)). The FAQ's "hundreds of microseconds" wording is dated.
  - Measured in 2023: about 1.35 µs per call for both interface and direct mapping on Windows ([zakgof](https://github.com/zakgof/java-native-benchmark)). Direct mapping was *not* faster in that benchmark.
  - Direct-mapping limits: no varargs; no arrays of Pointer/Structure/String as arguments ([DirectMapping](https://github.com/java-native-access/jna/blob/master/www/DirectMapping.md)).
- **Bulk data:** `Memory`/`Pointer` or a direct `ByteBuffer` passed as a pointer.
- **Callbacks:** JNA `Callback` interfaces. JNA attaches native threads and, by default, detaches threads it attached itself after each callback. `CallbackThreadInitializer(daemon, detach=false)` keeps them attached for repeated callbacks ([javadoc](https://java-native-access.github.io/jna/5.17.0/javadoc/com/sun/jna/CallbackThreadInitializer.html)).
- **Errors/async/memory:** all hand-made (C ABI conventions).
- **Tooling:** Android needs the `jna` AAR (per-ABI `libjnidispatch.so`), desktop needs the jar. One Kotlin source works for both.
- This is exactly the predecessor's stack, and also what UniFFI generates on top of.

### 4. Java FFM / Panama (desktop only)

- **Status:** final in **JDK 22** ([JEP 454](https://openjdk.org/jeps/454)); preview in 19–21 (JEP 424/434/442). **JDK 25 is an LTS (GA 2025-09-16)** ([JDK 25](https://openjdk.org/projects/jdk/25/)), so the predecessor's "needs JDK 22+" objection is now just "needs JDK 25 LTS". Compose Desktop bundles its own runtime, so this is feasible (but see Android).
- **Restricted methods** warn unless the app runs with `--enable-native-access=ALL-UNNAMED` or has the manifest attribute; this may become an error in future releases ([JEP 454](https://openjdk.org/jeps/454)).
- **Per-call cost:** the JEP's goal is overhead "comparable to, if not better than, JNI". Measured about 50 ns versus JNI about 57 ns ([zakgof](https://github.com/zakgof/java-native-benchmark)). `Linker.Option.critical(...)` trims short calls further; that claim is from the JEP API, and no numbers were found (unverified).
- **Bulk data:** `MemorySegment` with an `Arena` (deterministic free). Heap segments can be passed with `critical(true)`.
- **Upcalls:** `Linker.upcallStub` (threads attach automatically; unverified detail).
- **jextract:** generates bindings from a C header (for example one produced by cbindgen). The `master` branch tracks the latest JDK, a `jdk21` branch exists, binaries ship at jdk.java.net/jextract, and it is **GPL-2.0** but used only at build time ([jextract](https://github.com/openjdk/jextract)).
- **Android:** not available in ART. The only route found is the third-party [PanamaPort](https://github.com/vova7878/PanamaPort) (`com.v7878.foreign`, API 26+); unvetted.
- **Verdict:** breaks the "one shared Kotlin API" requirement, because Android would need a second binding. Only worth considering as a desktop-only hot-path swap behind an `expect`/`actual`.

### 5. Plain C ABI + cbindgen (v0.29.4, MPL-2.0, build-time only)

- **What it is:** a hand-designed `extern "C"` Rust API; cbindgen emits the `.h` file. Kotlin then needs a caller: JNA (option 3), FFM via jextract (option 4), or a thin JNI shim. So this is an *ABI design choice*, not a separate transport, and it is what the Go predecessor does (`c-shared`).
- **Costs:** you design ownership yourself (caller-allocated buffers, explicit free functions) and error convention (status code plus out-buffer). Panics must be stopped at every `extern "C"` function with `catch_unwind`, or `panic = "abort"`.
- **Benefits:** simplest to reason about, portable to any language or macOS/iOS later, and the best fit for "Kotlin owns the buffer".

### Others seen (not deeply evaluated)

- **BoltFFI** (MIT, v0.32.1): claims "up to 1000x faster than UniFFI". Its KMP (JVM plus Android) support is experimental and limited to sync, infallible free functions ([README](https://docs.rs/crate/boltffi/latest/source/README.md)). Too young; watch it.

## Comparison table

| | UniFFI (JNA backend) | Gobley | jni-rs (hand JNI) | JNA (hand C ABI) | FFM/Panama | C ABI + cbindgen |
|---|---|---|---|---|---|---|
| One API desktop + Android | yes | yes (+K/N) | yes | yes | **no** (desktop only) | depends on caller |
| Per-call overhead | JNA direct plus `RustBuffer` serialization for non-primitives; µs range (third-party, unverified) | same as UniFFI 0.29 | about 57 ns desktop; `@CriticalNative` near C call on Android 12+ | about 1.35 µs (2023 bench); direct mapping claimed near JNI | about 50 ns | = caller |
| Bulk data | `RustBuffer` copy; zero-copy `&[u8]` in (0.32); `&mut [u8]` unreleased | `RustBuffer` only (0.29) | direct `ByteBuffer`, zero-copy | Pointer / direct BB | `MemorySegment`, zero-copy | caller-allocated buffer |
| Callbacks/streams | foreign traits, generated | generated | manual, cache IDs in `JNI_OnLoad` | JNA `Callback` | upcall stubs | fn pointers |
| Async | `suspend fun`, no cancellation | yes (0.29 semantics) | manual | manual | manual | manual |
| Threading | JNA attaches | JNA attaches | manual attach/detach; class loader trap | auto attach; `CallbackThreadInitializer` | auto (unverified) | n/a |
| Errors/panics | typed exceptions; panic becomes internal exception | same | `ErrorPolicy` throws; `catch_unwind` | manual codes | manual codes | manual `catch_unwind` |
| Memory ownership | `Arc` handles, `AutoCloseable` + Cleaner | same | manual refs/handles | manual | `Arena` | manual |
| Tooling maturity | high (Mozilla/Firefox Android); no Gradle plugin | Gradle plugins, stale releases | mature crate, 0.22 API churn | very mature | JDK 22+/25 LTS, jextract EA | mature |
| Licence (shipped) | MPL-2.0 runtime/generated | MPL-2.0 | MIT/Apache | Apache-2.0 (choose) | JDK (GPL+CPE, already in JRE) | n/a (build tool MPL) |

## Bulk-data serialization comparison (Rust writer, Kotlin JVM/Android reader)

| Format | Kotlin read in place? | Rust side | Licence | Notes |
|---|---|---|---|---|
| **FlatBuffers** | yes, from `ByteBuffer`; Kotlin uses the Java runtime and runs on JVM/Android only, not K/N or JS ([docs](https://flatbuffers.dev/languages/kotlin/)) | `flatbuffers` 25.12.19; custom `Allocator` via `FlatBufferBuilder::new_in`. Allocators must be able to grow (`grow_downwards`), so writing into a fixed Kotlin buffer needs a "too small, retry" protocol ([docs.rs](https://docs.rs/flatbuffers/latest/flatbuffers/trait.Allocator.html)) | Apache-2.0 (unverified here) | **What the predecessor uses: one `.fbs` file, schema evolution.** |
| Cap'n Proto | yes (zero-copy design); capnproto-java is pure Java ([repo](https://github.com/capnproto/capnproto-java)); Android status unverified | `capnp` crate | MIT ([LICENSE](https://raw.githubusercontent.com/capnproto/capnproto-java/master/LICENSE)) | Fewer users on Kotlin; maintenance cadence unverified. |
| protobuf / prost | no, parse into objects (allocations) | `prost` | prost Apache-2.0, protobuf-java BSD-3 (unverified here) | Fine for commands/config; worst for per-frame. |
| bincode | no Kotlin reader | — | MIT | **Unmaintained**: v3.0.0 (2025-12-16) is a compile-error stub; "development on bincode has ceased" ([README](https://docs.rs/crate/bincode/3.0.0/source/README.md)). Rejected. |
| postcard | no Kotlin reader (hand-write a decoder) | `postcard` | MIT/Apache (unverified here) | Compact, but needs a Kotlin decoder. |
| UniFFI `RustBuffer` | no, generated code decodes into Kotlin objects | built in | MPL-2.0 | Convenient for small records. Copies plus allocation per call. |
| Raw direct `ByteBuffer` + `#[repr(C)]` structs | yes (manual offsets) | `bytemuck`-style | — | Fastest, but there is no schema and endianness/layout are hand-maintained. |

## Comparison with the Go bridge (music-tools)

| Aspect | Go (c-shared + JNA + FlatBuffers) | Rust equivalent |
|---|---|---|
| Binding | Hand JNA, one Kotlin source | UniFFI generates JNA-direct Kotlin (less glue, typed errors/async/objects); or keep a hand C ABI + JNA as an exact port |
| Per-call | JNA, µs range | Same with UniFFI/JNA. Better with a JNI hot path (tens of ns desktop; `@CriticalNative` Android) |
| View models | FlatBuffers written by Go into a Kotlin-owned buffer | Same `.fbs` file; `flatbuffers` crate. Pass the Kotlin direct BB as `&mut [u8]` (UniFFI unreleased) or via the raw C/JNI path. The Rust allocator must handle "buffer too small". |
| Per-frame beat | Scalar JNA call | Scalar `@CriticalNative`/JNI static, roughly 20x cheaper on desktop per the zakgof numbers (JNI about 57 ns vs JNA about 1,350 ns) |
| Runtime weight | Go runtime + GC inside the process; cgo call cost (not measured here) | No runtime/GC; panics need `catch_unwind` at the boundary |
| FFM rejection reasons | Not on Android; JDK 22+ | Not on Android still holds; the JDK reason is weaker now (JDK 25 LTS) |
| JNI rejection reason (glue) | Valid for Go | Reduced: jni-rs 0.22 macros + `catch_unwind` policies; glue limited to 1–3 hot functions |

## Open questions / prototype

1. **Benchmark on target hardware** (desktop Windows/Linux, a mid-range arm64 Android phone), using JMH plus an Android microbenchmark: UniFFI 0.32 noop, a UniFFI record round-trip, JNA direct noop, a jni-rs static, `@CriticalNative`, and FFM `critical`. The zakgof numbers are from 2023, Windows only, JDK 19.
2. **UniFFI's experimental JNI Kotlin generator:** when will it release, what do its benchmarks show, and does it keep one source for Android and desktop? Track the uniffi-rs CHANGELOG "Unreleased" section.
3. **When does `[ByMutRef] bytes` (`&mut [u8]`) ship?** Until then, how should Rust write FlatBuffers into a Kotlin-owned buffer through UniFFI?
4. **FlatBuffers in Rust into a fixed external buffer:** write an `Allocator` that errors when full, plus a retry-with-bigger-buffer protocol; or have Rust build into its own Vec and `memcpy` into the Kotlin buffer (one copy, simple).
5. **MPL-2.0 in shipped generated code (UniFFI, Gobley):** MPL is file-level copyleft and compatible with an MIT/Apache app (we must ship the source of unmodified MPL files, which is trivial for generated sources). Confirm against the project's licence policy.
6. **Building Windows from Linux** (`x86_64-pc-windows-gnu` vs `cargo-xwin` for msvc) is not documented by Gobley; CI on native runners may be simpler. Android: cargo-ndk 4.1.2, NDK r28+ for 16 KB page size.
7. **Coroutines:** UniFFI has no cancellation. Decide whether long jobs (analysis, rendering) take an explicit cancel handle.
8. **`panic = "abort"` vs `"unwind"`:** UniFFI and jni-rs only catch panics with unwind.

## Sources

- UniFFI changelog: https://raw.githubusercontent.com/mozilla/uniffi-rs/main/CHANGELOG.md
- UniFFI lifting/lowering: https://mozilla.github.io/uniffi-rs/latest/internals/lifting_and_lowering.html
- UniFFI RustCallStatus/panics: https://raw.githubusercontent.com/mozilla/uniffi-rs/main/docs/manual/src/internals/rust_calls.md
- UniFFI object references: https://raw.githubusercontent.com/mozilla/uniffi-rs/main/docs/manual/src/internals/object_references.md
- UniFFI futures: https://mozilla.github.io/uniffi-rs/latest/futures.html
- UniFFI errors: https://mozilla.github.io/uniffi-rs/latest/types/errors.html
- UniFFI foreign traits: https://raw.githubusercontent.com/mozilla/uniffi-rs/main/docs/manual/src/foreign_traits.md
- UniFFI callback interfaces: https://raw.githubusercontent.com/mozilla/uniffi-rs/main/docs/manual/src/types/callback_interfaces.md
- UniFFI Kotlin lifetimes: https://mozilla.github.io/uniffi-rs/latest/kotlin/lifetimes.html
- UniFFI Kotlin config: https://mozilla.github.io/uniffi-rs/latest/kotlin/configuration.html
- Gobley: https://github.com/gobley/gobley, https://gobley.dev/docs/gradle-plugins, https://gobley.dev/docs/cross-compilation-tips, https://github.com/gobley/gobley/commits/main, https://crates.io/api/v1/crates/gobley-uniffi-bindgen, https://repo1.maven.org/maven2/dev/gobley/uniffi/dev.gobley.uniffi.gradle.plugin/maven-metadata.xml
- jni-rs: https://docs.rs/jni/latest/jni/, https://crates.io/api/v1/crates/jni
- Android JNI tips: https://developer.android.com/training/articles/perf-jni
- Android ABIs: https://developer.android.com/ndk/guides/abis
- JNA: https://github.com/java-native-access/jna, LICENSE https://raw.githubusercontent.com/java-native-access/jna/master/LICENSE, FAQ https://raw.githubusercontent.com/java-native-access/jna/master/www/FrequentlyAskedQuestions.md, Direct mapping https://github.com/java-native-access/jna/blob/master/www/DirectMapping.md, CallbackThreadInitializer https://java-native-access.github.io/jna/5.17.0/javadoc/com/sun/jna/CallbackThreadInitializer.html
- JEP 454: https://openjdk.org/jeps/454 ; JDK 25: https://openjdk.org/projects/jdk/25/ ; jextract: https://github.com/openjdk/jextract
- PanamaPort (Android FFM port, third-party): https://github.com/vova7878/PanamaPort
- Benchmark (JNI/JNA/FFM): https://github.com/zakgof/java-native-benchmark
- tweedegolf java-interop: https://github.com/tweedegolf/java-interop
- BoltFFI: https://docs.rs/crate/boltffi/latest/source/README.md
- cargo-ndk: https://crates.io/api/v1/crates/cargo-ndk ; cbindgen: https://crates.io/api/v1/crates/cbindgen
- FlatBuffers Kotlin: https://flatbuffers.dev/languages/kotlin/ ; Rust Allocator: https://docs.rs/flatbuffers/latest/flatbuffers/trait.Allocator.html
- Cap'n Proto Java: https://github.com/capnproto/capnproto-java ; licence https://raw.githubusercontent.com/capnproto/capnproto-java/master/LICENSE
- bincode status: https://docs.rs/crate/bincode/3.0.0/source/README.md
