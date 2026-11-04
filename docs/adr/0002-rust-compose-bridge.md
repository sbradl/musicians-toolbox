---
status: accepted
---

# UniFFI-only bridge between the Rust core and Compose

Every app's Compose UI calls its Rust core through **UniFFI** (Kotlin bindings over JNA), on desktop JVM and Android from one Kotlin source. FFM is out (no Android). Measurements and options: [research #2](https://github.com/sbradl/musicians-toolbox/issues/2).

- **No second mechanism until measured.** jni-rs (+ `@CriticalNative` on Android) is the per-function escape hatch, added only when a benchmark on a target device shows UniFFI is too slow for that call.
- **UniFFI records for data.** FlatBuffers in a direct ByteBuffer, or paging, per payload, only after a benchmark.
- **Sync core, no async runtime.** Long jobs run in parallel inside the core (threads/rayon), take a `CancelToken`, and Kotlin calls them on a background dispatcher. On Android, Kotlin enumerates folders via SAF and fans out per-file calls taking fds.
- **Rust→Kotlin events via UniFFI callback interfaces**, throttled and batched in Rust (~10/s). Callbacks may arrive on any thread; Kotlin only emits them into a Flow. Per-frame state (e.g. playing position) is decided with the first hot path.
- **Errors follow the contexts.** Each context owns its error types, named in its own language and kept apart from infrastructure errors (the domain never sees `io::Error`). Errors are translated at every boundary (ACL, app layer, FFI); none crosses unchanged, and internals don't leak into UI texts. Expected failures become typed Kotlin exceptions. A panic is a bug: the app logs it and fails fast. Malformed input is an expected error and never panics.
- **One `<app>-ffi` crate per app** composes the contexts it uses, owns all UniFFI types, and builds one native lib. Context crates don't depend on UniFFI.
- **UniFFI (MPL-2.0) and JNA (Apache-2.0) are used as released**: no vendoring, patching, forking or upstreaming. Limits are worked around in our own code. Both appear in the third-party notices.

Revisit on the first hot path, the first large payload, or once UniFFI's JNI backend ships; benchmark numbers go into a follow-up ADR.
