# Packaging Compose Multiplatform apps with a native Rust library

Research for [#3](https://github.com/sbradl/musicians-toolbox/issues/3) (map [#1](https://github.com/sbradl/musicians-toolbox/issues/1)). Sources checked 2026-11-04. Local references: `sbradl/music-tools` (Stave: Compose desktop + AppImage + JNA. Drum Practice: Android + Go `.so` via a Gradle task) and `sbradl/tab-sync` (release matrix).

## Recommendation (TL;DR)

| Platform | Artifact (GitHub Releases) | Rust target | Build on |
|---|---|---|---|
| Windows x64 | MSI (+ portable zip of `createDistributable`) | `x86_64-pc-windows-msvc` | `windows-latest` |
| Windows arm64 | MSI (+ zip) | `aarch64-pc-windows-msvc` | `windows-11-arm` |
| Linux x64 | AppImage + tar.gz (deb optional) | `x86_64-unknown-linux-gnu`, glibc pinned | `ubuntu-*` |
| Android | APK now, AAB built too (Play later) | `aarch64-linux-android`, `x86_64-linux-android` via cargo-ndk | `ubuntu-*` |

- Use the built-in Compose Gradle plugin (jpackage) rather than Conveyor for now. It is already what `music-tools` uses, it is free, and it needs no extra tool. Revisit Conveyor if auto-update or Linux→all-OS cross-packaging becomes worth it.
- One CI job per OS/arch, because jpackage cannot cross-build. Rust is built inside each job for its own target, so you don't need any cross-compilation tooling on desktop (no `cross` or `cargo-xwin`).
- Android: only 64-bit ABIs (`arm64-v8a`, `x86_64`), NDK r28+ and AGP ≥ 8.5.1. With these, the 16 KB page-size rule for Play is met by default.
- Desktop native lib goes in `appResourcesRootDir/<os>-<arch>/`. Load it from `compose.application.resources.dir`. Android lib goes in `jniLibs/<abi>/`, loaded by name.
- Signing: Android uses the same keystore flow as today. Windows starts unsigned (SmartScreen warning), with Microsoft Artifact Signing as the cheap upgrade path. Linux: publish SHA256SUMS.

## 1. Compose desktop packaging

**Built-in (jpackage via the Compose Gradle plugin).**
- Formats: Windows `Exe` and `Msi`. Linux `Deb` and `Rpm`. macOS `Dmg` and `Pkg`. There is no AppImage, tarball or Flatpak. ([Compose native distributions](https://kotlinlang.org/docs/multiplatform/compose-native-distribution.html))
- "Cross-compilation is currently not supported, meaning you can build the specific format using the corresponding compatible OS only." (same page)
- `createDistributable` "creates a final application image without creating an installer". The `package<Format>` tasks build installers. `*Release*` variants run ProGuard (minify and optimize on by default, obfuscation off). (same page)
- `windows { upgradeUuid = ... }` must stay constant per app so MSI upgrades work. (same page; jpackage `--win-upgrade-uuid` in [jpackage man page](https://docs.oracle.com/en/java/javase/21/docs/specs/man/jpackage.html))
- jpackage `--type` values are `app-image, exe, msi, rpm, deb, pkg, dmg`. `app-image` is the portable directory. ([jpackage](https://docs.oracle.com/en/java/javase/21/docs/specs/man/jpackage.html))

**Portable exe / AppImage / tarball.**
- Portable Windows: zip the `createReleaseDistributable` output (`<App>.exe` launcher + `app/` + `runtime/`). jpackage has no single-file exe.
- AppImage: Stave already does this. It wraps `createDistributable` in an AppDir and runs a pinned, checksummed `appimagetool` (`music-tools/.github/workflows/stave-release.yml`). Reuse that recipe.
- tar.gz of the distributable is trivial, the same approach TabFinder uses for its Go desktop build today (`tab-sync/.github/workflows/release.yml`).
- Flatpak: no plugin support. Flathub is out of scope per #1. Defer.

**Conveyor (Hydraulic).** It is the alternative if cross-building or auto-update matters. It "package[s] and deploy[s] directly for every platform you support from any Linux build agent", signs for every OS from any OS, has silent background updates and delta updates, scaffolds Compose Desktop, and "It's free for open source apps". ([Conveyor docs](https://conveyor.hydraulic.dev/latest/)) It is a proprietary tool, though only used at build time, so it adds no licence obligation to shipped code. Not needed for the first release.

## 2. Bundled JRE: size and startup

- Packaging needs JDK 17+ (jpackage). The Compose runtime needs JDK 11+. ([Compose compatibility](https://kotlinlang.org/docs/multiplatform/compose-compatibility-and-versioning.html))
- jlink bundles only the listed modules. The plugin does not detect modules. Use `suggestModules` (jdeps, "might be incomplete") or `modules(...)`. A missing module shows up as `ClassNotFoundException` at runtime. ([native distributions](https://kotlinlang.org/docs/multiplatform/compose-native-distribution.html))
- **Measured locally** on Stave's non-ProGuard Linux x64 `createDistributable` (Compose 1.12.1, Temurin 21): **146 MB total**, made up of `lib/runtime` **87 MB** (modules `java.base java.datatransfer java.xml java.prefs java.desktop java.logging jdk.crypto.ec`) and `lib/app` **58 MB**. Of the app part, `libskiko-linux-x64.so` is 29 MB and the Compose jars are about 10 MB. ProGuard shrinks the jars but not the runtime or Skiko. Installers compress this further, but this was not measured.
- **Startup time: not measured.** No primary source gives numbers. Measure cold start in the bridge spike before treating it as a constraint. Levers to try then: ProGuard release build, fewer jlink modules, AppCDS.
- The shipped runtime is OpenJDK (GPLv2 + Classpath Exception). The exception allows bundling with non-GPL code, so it doesn't violate the "no GPL in what ships" rule. Still, note it in the third-party licence file.

## 3. Loading the native library per OS

**Desktop.** Put the Rust `cdylib` under `appResourcesRootDir`:
- `common/` goes to all packages, `windows/` / `linux/` go to that OS only, and `<os>-<arch>/` (e.g. `windows-arm64`, `linux-x64`) go to that OS and arch only. ([native distributions](https://kotlinlang.org/docs/multiplatform/compose-native-distribution.html))
- At runtime use `File(System.getProperty("compose.application.resources.dir"))` (same page). Then call `System.load(dir.resolve(System.mapLibraryName("core")).path)` for JNI, or point `jna.library.path` at that dir for JNA. Stave does exactly this with `appResourcesRootDir.set(...native/desktop)` and a `linux-x64/libstave.so` subdir (`music-tools/stave/app/shared/build.gradle.kts`).
- Under plain `run`/tests the property is unset, so keep a dev fallback, e.g. a path to `target/release` (Stave builds a separate test lib).

**Android.**
- Libraries must sit at `lib/<abi>/lib<name>.so` in the APK. The package manager extracts the best ABI and the app loads by name. ([NDK ABIs](https://developer.android.com/ndk/guides/abis))
- In Gradle, either output to `src/main/jniLibs/<abi>/` (cargo-ndk `-o` does this layout) or register a generated dir with `variant.sources.jniLibs?.addGeneratedSourceDirectory(...)`. Drum Practice does the latter for its Go `.so` (`music-tools/drum-practice/app/androidApp/build.gradle.kts`). The same pattern fits cargo-ndk.

**Bridge mechanics (for the bridge ticket, not decided here).** JNA is dual LGPL-2.1+/Apache-2.0 (Apache since 4.0), so it is fine to ship under Apache. ([JNA](https://github.com/java-native-access/jna)) Stave uses `jna` on JVM and `jna@aar` on Android. JNA call overhead is higher than JNI. `java.lang.foreign` (FFM) is not available on Android's ART, so a common JVM/Android bridge means JNI or JNA. Given "performance first", benchmark per-call cost in that spike.

## 4. Building Rust for each target

Target tiers ([rustc platform support](https://doc.rust-lang.org/nightly/rustc/platform-support.html)):

| Target | Tier | Notes |
|---|---|---|
| `x86_64-pc-windows-msvc` | 1 (host tools) | Windows 10+ |
| `aarch64-pc-windows-msvc` | 1 (host tools) | ARM64 Windows |
| `x86_64-pc-windows-gnu` | 1 (host tools) | MinGW, Windows 10+ |
| `*-pc-windows-gnullvm` | 2 | MinGW, LLVM ABI |
| `x86_64-unknown-linux-gnu` | 1 | kernel 3.2+, glibc 2.17+ |
| `aarch64-linux-android`, `x86_64-linux-android` | 2 (no host tools) | full std |

- **Windows: MSVC, built natively on Windows runners.** Both arches are Tier 1, it matches the toolchain the JDK itself is built with, and it avoids shipping MinGW runtime pieces. GNU only makes sense when cross-compiling from Linux, and jpackage forces a Windows job anyway. Consider `-C target-feature=+crt-static` so the DLL doesn't need the VC++ redistributable (not verified whether the jlinked runtime already ships `vcruntime140.dll`; check in the spike).
- **Linux: glibc floor.** Compose's own floor is Ubuntu 20.04 (glibc 2.31) ([compat](https://kotlinlang.org/docs/multiplatform/compose-compatibility-and-versioning.html)). A Rust lib built on `ubuntu-24.04` would require glibc 2.39. Either build on the oldest runner (`ubuntu-22.04`, glibc 2.35) or use `cargo zigbuild --target x86_64-unknown-linux-gnu.2.31`. cargo-zigbuild sets a glibc floor via a target suffix and supports Linux and macOS targets only. MIT licence. ([cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild)) TabFinder already pins `ubuntu-24.04` for the same glibc reason.
- **Android: cargo-ndk.** "handles all the environment configuration needed", e.g. `cargo ndk -t arm64-v8a -t x86_64 -o ./jniLibs build --release`. API level via `CARGO_NDK_PLATFORM`. Apache-2.0/MIT. ([cargo-ndk](https://github.com/bbqsrc/cargo-ndk)) Rust supports the latest LTS NDK and "all API levels supported by the NDK". ([Rust Android targets](https://doc.rust-lang.org/nightly/rustc/platform-support/android.html)) Alternatively call the NDK clang directly as linker, as Drum Practice does for Go (`$NDK/toolchains/llvm/prebuilt/linux-x86_64/bin/<arch>-linux-android<api>-clang`).
- `cross` (Docker-based) is unnecessary given native per-OS jobs. Not evaluated further.

## 5. Android ABIs and Play compatibility

- NDK ABIs are `armeabi-v7a`, `arm64-v8a`, `x86`, `x86_64`. Use `abiFilters 'arm64-v8a', 'x86_64'` for 64-bit only. App Bundles or splits avoid fat-APK size. ([NDK ABIs](https://developer.android.com/ndk/guides/abis)) Drum Practice already ships exactly these two ABIs.
- **16 KB pages.** "All apps targeting Android 15 (API level 35) and higher must support 16 KB memory page sizes on 64-bit devices on Google Play. Starting February 1, 2027, if your app updates don't support 16 KB memory page sizes, you won't be able to release these updates." ([page sizes](https://developer.android.com/guide/practices/page-sizes))
  - "NDK version r28 and higher compile 16 KB-aligned by default". Older NDKs need `-Wl,-z,max-page-size=16384`. Prebuilt `.so` (i.e. our Rust output) "must also" be built that way. AGP ≥ 8.5.1 handles zip alignment. (same page)
  - CI check: `zipalign -c -P 16 -v 4 app.apk` and `llvm-objdump -p lib.so | grep LOAD` (align `2**14`). (same page) Add this as a release gate.
  - Rust links through the NDK clang, so NDK r28+ (music-tools pins 28.2) gives 16 KB alignment. Verify with the check above rather than assume.
- 32-bit ABIs: not needed (Play's 64-bit rule applies to apps that *have* 32-bit code). Dropping them keeps CI and APK small.

## 6. GitHub Actions matrix (sketch)

Runner labels for public repos: `ubuntu-24.04`/`ubuntu-22.04`, `ubuntu-24.04-arm`, `windows-latest`/`windows-2025`, **`windows-11-arm`** (also for private repos with 2 CPU). ([GitHub-hosted runners](https://docs.github.com/en/actions/reference/runners/github-hosted-runners))

```yaml
jobs:
  desktop:
    strategy:
      matrix:
        include:
          - { os: windows-latest,  rust: x86_64-pc-windows-msvc,    res: windows-x64,   task: packageReleaseMsi }
          - { os: windows-11-arm,  rust: aarch64-pc-windows-msvc,   res: windows-arm64, task: packageReleaseMsi }
          - { os: ubuntu-22.04,    rust: x86_64-unknown-linux-gnu,  res: linux-x64,     task: createReleaseDistributable } # then AppImage + tar.gz
    runs-on: ${{ matrix.os }}
    # steps: mise (JDK 21, Rust) -> cargo build --release --target $rust -> copy cdylib to native/$res -> gradlew $task -> upload
  android:
    runs-on: ubuntu-latest
    # steps: NDK r28 -> cargo ndk -t arm64-v8a -t x86_64 -o <generated jniLibs> -> assembleRelease bundleRelease -> apksigner/jarsigner -> 16KB check
  release:
    needs: [desktop, android]
    # download-artifact (merge) -> SHA256SUMS -> gh release create "<app>/vX.Y.Z"
```

- Fits #1's monorepo plan: tag trigger `<app>/v*.*.*`, path-filtered CI, the same shape as `music-tools/.github/workflows/*-release.yml` (SHA-pinned actions, `permissions: {}`, `environment: release`).
- Check that the JDK distribution has a Windows arm64 build. Temurin publishes one, but confirm `mise` resolves it on `windows-11-arm`.

## 7. Code signing basics

- **Android:** APK v2+ signing with a release keystore via `apksigner`. AAB via `jarsigner` (Play App Signing re-signs later). Exactly as in today's workflows, including TabFinder's cert-SHA-256 guard. The keystore must never change, or users can't update. That also keeps the Play path open.
- **Windows:** the Compose docs cover signing for macOS only ([native distributions](https://kotlinlang.org/docs/multiplatform/compose-native-distribution.html)). Sign `exe`/`msi`/`dll` with `signtool` after packaging. Unsigned builds work but trigger SmartScreen.
  - Microsoft **Artifact Signing** (formerly Trusted Signing) is a managed service with FIPS 140-3 L3 HSMs ([overview](https://learn.microsoft.com/en-us/azure/artifact-signing/overview)). It supports Individual identity validation and GitHub Actions integration. It requires a *paid* Azure subscription (no free/trial). It issues no EV certificates, and "SmartScreen reputation builds up automatically" over download history. ([FAQ](https://learn.microsoft.com/en-us/azure/artifact-signing/faq)) Price and eligible countries are on linked pages, not checked.
  - Conveyor can also sign Windows from Linux.
- **Linux:** no OS-level code signing. Publish `SHA256SUMS` (as TabFinder does). GPG/minisign is optional.

## 8. Minimum OS / API constraints

| | Floor imposed by | Value |
|---|---|---|
| Windows | Compose 1.12 and Rust MSVC targets | Windows 10, x64 and arm64 ([compat](https://kotlinlang.org/docs/multiplatform/compose-compatibility-and-versioning.html), [rustc](https://doc.rust-lang.org/nightly/rustc/platform-support.html)) |
| Linux | Compose 1.12 | Ubuntu 20.04 x64/arm64 (glibc 2.31); Rust lib must not raise it (see §4) |
| Android | Compose: API 21. Existing apps use `minSdk 29` | Recommend keeping **minSdk 29** (matches music-tools/Stave). Revisit only if a device demands lower |
| All | Compose | 64-bit only ("All Compose Multiplatform releases support only 64-bit platforms") |

`targetSdk` must follow Play's yearly target-API rule once on Play. Not checked here; it doesn't block GitHub Releases.

## Open items for follow-up tickets

- Cold-start time and installed size with a ProGuard release build (measure in the bridge spike).
- JNI vs JNA per-call cost (bridge ticket).
- Whether the jlinked runtime ships `vcruntime140.dll`, which decides whether `+crt-static` is needed.
- Windows signing: whether to pay for Artifact Signing before the first public Windows release.
