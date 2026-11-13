---
status: accepted
---

# Repo layout by Ports & Adapters, one Cargo workspace and one Gradle build

```
contexts/<ctx>/                 domain of one bounded context (+ its ports, ADR 0001), crate mt-<ctx>
apps/<app>/
  application/                  use cases composing contexts + driven ports they need
  presentation/                 App, presenters, ViewModels, navigation, Fluent texts (ADR 0003); no UniFFI
  adapters/ffi/                 driving adapter for Compose (UniFFI mapping only, ADR 0002) + driven adapters forwarding ports to Kotlin
  adapters/cli/                 CLIs (e.g. tabscan, tabreorg) on application (+ texts from presentation)
  adapters/<name>/              driven adapters in Rust implementing application ports (e.g. fs: desktop Library folder), wired by ffi
  ui/shared/                    KMP module: Compose rendering + the Kotlin driven ports both platforms implement (e.g. FolderPicker) and the glue that runs them
  ui/desktop/, ui/android/      entry points + Kotlin driven adapters (SAF, FileProvider, launching a tab editor)
  CHANGELOG.json
adapters/<name>/                driven adapters shared by several apps, once a second app needs one
ui-shared/                      Compose modules shared by apps
build-logic/                    Gradle convention plugins
```

- **Dependencies point inward**: `ui → ffi → presentation → application → contexts`; Rust driven adapters depend only on application and are wired in by ffi. Contexts know no apps.
- **One Cargo workspace** at the root: shared `[workspace.dependencies]` and `[workspace.lints]`. Packages are named `mt-<name>` (folders without prefix), all `publish = false`, version `0.0.0`. CLI binaries keep product names.
- **One Gradle build** at the root with a version catalog and convention plugins. Gradle drives Cargo through our own task (cargo / cargo-ndk + uniffi-bindgen); UniFFI bindings are generated into `build/`, never committed. Compose per app is split into `ui/shared`, `ui/desktop`, `ui/android` (AGP 9 keeps the Android application plugin out of KMP modules).
- **Toolchain pinned in `mise.toml` wherever possible** (Rust with targets and components, JDK, cargo-ndk, Android SDK/NDK). Exceptions: the Gradle wrapper (IDE default, checksummed) and uniffi-bindgen as a workspace crate (must match the runtime version).
- **mise tasks are thin wrappers that CI calls too** (provisional): `check`, `fmt`, `test`, `lint`, `run:<app>`, `android:<app>`, `package:<app>`.
- **CI**: `check` over the whole repo on every push; packaging per app, path-filtered and deliberately coarse (any context or shared build file triggers all apps).
- **Release**: the tag `<app>/vX.Y.Z` (set via the commit skill with changelog rollover) triggers that app's release workflow; notes come from its `CHANGELOG.json`. The tag is the only version source (`0.0.0-dev` locally); Android `versionCode = major*1_000_000 + minor*1_000 + patch`.
- **Signing**: one Android release keystore per app, kept as GitHub secrets with an offline backup, later the Play upload key. Windows and Linux builds are unsigned for now; every release ships SHA256 checksums.
- **Linux floor**: glibc 2.31 (`checkGlibcFloor`), so Ubuntu 20.04, Debian 11 and newer. Desktop integration goes through xdg portals and works with the portal versions those distros ship: a portal method newer than that gets a fallback (dark/light reads `Settings.ReadOne`, from xdg-desktop-portal 1.17.1, and falls back to `Read`). A desktop whose portal lacks a setting gets the plain default (light). The display scale is the one exception: no portal publishes it, so TabFinder reads the X11 settings Java sees (`Gdk/UnscaledDPI`, `Gdk/WindowScalingFactor`) at start and adds what Java's own scale misses.
- **IDs**: Android applicationId and Kotlin root `io.github.sbradl.<app>`; ui-shared `io.github.sbradl.mt.ui`.
- **Context docs move next to the code** (`contexts/<ctx>/GLOSSARY.md`, `adr/`) with the first commit of that context's crate; system-wide ADRs stay in `docs/adr/`.
