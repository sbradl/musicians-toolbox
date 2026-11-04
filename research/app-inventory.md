# Inventory: TabFinder and Drum Practice as reference specs

Resolves #5 (map #1). The question: what do the old apps do, so we can keep or drop each feature on purpose?

Sources are the two local checkouts the inventory read. Paths below are relative to them.
- `tab-sync` = `/home/sbradl/Projects/tab-sync` @ `be0fd1a`. GitHub name `sbradl/tabfinder`; Go module `tabfinder`.
- `music-tools` = `/home/sbradl/Projects/music-tools` @ `20dab7d`.

Stave appears only where its notation or display-list work matters. We are not reimplementing Stave.

## TL;DR

- **Both repos have the same shape as our target:** a core that holds all logic, and a thin UI.
  - TabFinder: a Go core. The desktop UI uses Gio. On Android, a Compose app runs the Go `tabscan -serve` binary as a subprocess.
  - Drum Practice: a Go core built as a c-shared lib and called through JNA, with FlatBuffers view models and a Compose MP UI.
  - Both are references for the Rust↔Compose bridge decision.
- **TabFinder has no ADRs, no glossary and no Gherkin.** Its spec is `docs/test-plan.md`: about 140 numbered U-/E- cases, plus a "Differences" section.
- **Drum Practice has the richest spec:**
  - 65 `.feature` files with about 287 scenarios
  - 5 ADRs, plus 2 shared ADRs and 5 Stave ADRs
  - a 348-line `GLOSSARY.md`
  - golden view files and fuzz corpora
- **Conflicting terms to settle in our bounded contexts:**
  - Beat: Drum Practice means a metric slot; Stave and GP mean an onset.
  - Note, Tempo, Bar, Session, Rest, Mode and Display list also conflict.
  - Inside TabFinder: Score (the note-data package vs the difficulty float), plus four shapes each of Tuning, Song and Part.
- **Platform risks:**
  - TabFinder Android uses `MANAGE_EXTERNAL_STORAGE` (all-files access) so tabscan gets plain paths. It only uses SAF to pick the folder. Google Play restricts this permission, which clashes with "nothing may block a later Play Store release" (#1).
  - TuxGuitar runs as a process on desktop (Linux only in practice) and through an `ACTION_VIEW` intent plus FileProvider on Android. There is no Windows code.
  - Drum Practice audio renders ahead into a lock-free ring, with a C shim over AAudio. The Linux PipeWire shim is designed but not built, and there is no Windows audio.
- **Licensing:** no GPL in either repo's dependencies.
  - Every parser (GP3-7, GPX, TG and PTB headers) is in-house stdlib Go.
  - The music font is Gonville: copyright-waived, generator MIT. Bravura was rejected to avoid OFL.
  - TabFinder bundles OFL UI fonts.

---

## 1. TabFinder (tab-sync)

### 1.1 Feature list

**Shared Go core** (`internal/*`)
- **Formats:** sniffed from content, not the extension (`internal/tab/parse.go:24`).
  - Full parse: GP3/4/5, GP6 (`.gpx`, BCFZ/BCFS), GP7 (`.gp` zip, `Content/score.gpif`).
  - Header only: TG1/TG2 and PTB2 (`internal/tab/other.go:13`).
  - Also read: `.zip` wrapping a GP5, and `.crdownload`.
  - Text decoding: UTF-8 with a Latin-1 fallback.
- **Metadata:** Artist, Album and Title, each with a Source (`file`, or `path` as fallback). Tracks with MIDI pitches, tempos per bar, an error flag (`internal/tab/song.go`).
- **Tuning naming:**
  - Standard and drop tunings are named for 4-7 strings. 8-9 strings are always Custom.
  - Matches by name or notes, enharmonics included. "standard" alone means E Standard (`internal/tab/tuning.go`).
- **Difficulty (v1.1.0):**
  - A score of 1-10 per part. The parts are drums, bass, rhythm and lead. Rhythm vs lead is decided per bar, not per track.
  - Tags: fast, syncopated, odd meter, sweeps, double kick and others (README "Difficulty"; `internal/difficulty/`).
  - Fitted to one intermediate metal guitarist. Computed only for GP3-7.
- **Index:** `index.jsonl` with header `{"tabfinderIndex":4}`. A version mismatch forces a full rescan. Writes go to `.tmp`, then rename (`internal/finder/index.go:16`).
- **Scan:** parallel on all CPUs. About 0.8 s for 950 songs with full note reading (CHANGELOG 1.1.0).

**TabFinder desktop** (`cmd/tabfinder`, Go + Gio; released for linux-amd64 only)
- **First run:** a folder picker through `kdialog` or `zenity` (`cmd/tabfinder/library.go:76`).
- **Index and scanning:** shows the cached index at startup. Scans automatically when there is no index. Rescan button; snackbar "N tabs, M unreadable, in X s".
- **Filters:** Artist, Song, Tuning and BPM, each with a clear button. BPM takes `120`, `100-140`, `180-` or `-90`.
  - Counter "x / y"; "No tabs match" offers Clear filters.
  - Suggestions are capped at 60. Tuning suggestions are grouped by string count; picking one sets a string-count badge (`internal/finder/finder.go:302`; test-plan E-DSK-10..22).
- **Song rows:** title, "artist · album", tuning tags colored by string count, a tempo summary, "Couldn't read this file".
- **Difficulty display and sort:** a 5-bar meter per part. A Difficulty popup has a 1-10 range slider per part and chips. Sort is A-Z, easiest or hardest (E-DSK-31..34).
- **Opening:** a click opens the song in TuxGuitar (§1.5).
- **Theme:** dark or light from `gsettings color-scheme`. Fonts are bundled.
- **Install:** `install.sh` puts the binary in `~/.local/bin` and adds a menu entry.

**TabFinder Android** (`android/`, Kotlin + Compose, minSdk 29, target 36)
- **One screen with the same features as desktop** (`ui/MainScreen.kt`, `ui/Difficulty.kt`). Filters are not restored after process death.
- **No search logic in Kotlin.** Matching and row texts come from `tabscan -serve` over JSON lines on stdin/stdout (`data/Finder.kt:23`, `cmd/tabscan/serve.go:13`).
  - A dead process is restarted and its `load` replayed.
  - Timeouts: search 10 s, load 60 s, scan 10 min.
- **Searches:** `mapLatest`, so only the newest query's result applies. A cancelled search is not shown as an error (`ui/MainViewModel.kt:55,120`).

**tabscan** (`cmd/tabscan`)
- **Usage:** `tabscan [-json] [-serve] [-root] [-name] [-artist] [-tuning] [-bpm] [-drums|-bass|-rhythm|-lead range] [-tag role:tag] [-sort easiest|hardest] [paths]` (`main.go:21`).
- **Output:** TSV by default, or JSON lines with `-json`.
- **Exit codes:** 2 for a bad flag, 1 for a failed directory.
- **Filtering on parts:** a level range only matches songs that have that part.
- **`-serve` protocol:** ops `load`, `scan`, `search`. Lists come back as `[]`, never null. Search takes under 5 ms (E-CLI-09).

**tabreorg** (`cmd/tabreorg`)
- **Usage:** `tabreorg [-apply] [-rename=false] [-summary] [-skip] [-config] root`.
- **Default is a dry run:** prints the plan and writes a summary `.md`.
- **`-apply`:** moves files and writes a moves TSV plus an undo script. It aborts before moving anything if it can't write those.
- **Plan rules:**
  - The target layout is `<Artist>/<Album>/`.
  - The album name is matched loosely, using a port of Python difflib's `ratio` (`difflib.go`).
  - Optional rename to the song title. A clash (same new name in one folder) is kept as is and reported. Any other collision gets " (2)". Nothing is overwritten.
  - Idempotent.
- **Summary:** lists files without an album, unrenamed clashes, and duplicate songs found by MD5.
- **Config:** `~/.config/tabreorg/config.json` with keys `skip`, `albumAliases`, `junkAlbums`, `junkArtists`, `fileSuffixes`; values are regexes (`config.go:14`).

### 1.2 Domain terms (code names only; no glossary exists)

| Term | Meaning | Source |
|---|---|---|
| Song | A parsed file: path relative to root, format, metadata plus Source, tracks, tempos, error, parts | `internal/tab/song.go:70` |
| Format | gp3, gp4, gp5, gp6 (=`.gpx`), gp7 (=`.gp`), tg, ptb | `internal/tab/song.go:43` |
| Unreadable | Error AND no Format. A partly read file is not unreadable | `internal/tab/song.go` |
| Tuning | Name plus pitches. "Custom" shows as notes in the finder and rows | `internal/tab/tuning.go:9`, `internal/finder/finder.go:17` |
| Entry / Library | A song plus its distinct tunings / all songs sorted by artist, then title | `internal/finder/finder.go:32,54` |
| Query / Filter | Typed text / parsed criteria | `internal/finder/filter.go:16` |
| Role / Part | drums, bass, rhythm, lead / a role's tracks, score and tags; `Level()` = the rounded score | `internal/difficulty/difficulty.go:13,33` |
| Index | The cached scan, `index.jsonl` | `internal/finder/index.go` |
| openAs | A clean file name with the real-format extension, for TuxGuitar | `internal/tuxguitar/tuxguitar.go:16` |
| clash / collision | tabreorg: same song name in one folder (kept) / a name collision (gets a " (2)" suffix) | `cmd/tabreorg/plan.go` |

**Conflicts inside tab-sync**
- **Score:** the `internal/score` package (note data, 960 ticks per quarter) vs `Part.Score` (difficulty, a float). JSON carries `score` in `-json` but `level` in rows.
- **Four shapes each:** Tuning (tab, finder, rows, Kotlin), Song (`tab.Song`, `rows.Song`, Kotlin `Song`) and Part.
- **Track:** `tab.Track` holds metadata; `score.Track` holds beats.
- **Format names vs extensions:** gp6 is `.gpx`; gp7 is `.gp`.
- **Default sort:** A-Z (artist, then title) in the apps vs walk order in `tabscan` with no `-sort`.
- **Dark-theme fallback:** the README and U-DSK-06 say light, the code says dark (test-plan "Differences").

### 1.3 ADRs

None. There is no `docs/adr` and no decision doc. These decisions live only in comments and the README:
- Sniff formats from content.
- One search implementation, used by Android through the subprocess.
- All-files access instead of SAF, because tabscan needs plain paths (`android/app/src/main/AndroidManifest.xml:5-11`).
- Bump the index version when the parser changes.
- Copy misnamed files to the cache before opening; open correctly named files in place, so saves go to the original (`cmd/tabfinder/library.go:98`).
- Library-specific tabreorg rules go in config, not code.
- All test data is made up, for copyright reasons.

### 1.4 Reusable specs and corpora

- **`docs/test-plan.md`** (562 lines):
  - IDs U-TAB-01..17, U-FIN-01..11, U-CLI-01, U-SRV-01..05, U-REO-01..06, U-DSK-01..10, U-AND-01..10.
  - E-CLI-01..09, E-REO-01..07, E-DSK-01..34, E-AND-01..15.
  - All are done except E-AND-12 (device perf).
  - These are the best source for TabFinder's first-slice acceptance criteria.
- **Fixtures F1-F14:** in `internal/tab/testdata/library/`, with expectations in `internal/tab/testdata/expected.json`. Builder: `internal/tab/fixtures_test.go` (`-update` regenerates them).
  - Cover GP3-7, GPX, TG, PTB, misnamed, zipped, truncated and unsupported (GP2, PTB1) files.
  - Caveat: the builders and parsers share the same format knowledge.
- **Golden files:**
  - `cmd/tabscan/testdata/tree.{tsv,jsonl}`
  - `cmd/tabreorg/testdata/{stdout_dryrun.txt,summary_dryrun.md,summary_applied.md,undo.sh}`
- **Synthetic trees:**
  - `internal/testlib/{testlib.go,tree.go}`: case variants, non-ASCII, 4-9 strings, a messy tree.
  - Android: `android/app/src/sharedTest/.../Fixture.kt`.
- **Fuzz targets:** FuzzParseBytes, FuzzTitleFromFilename, FuzzParseBPMRange, FuzzServe.
- **Test counts:** about 234 Go tests and about 91 Android `@Test`.

### 1.5 Platform integrations

- **TuxGuitar, desktop:**
  - Runs `exec.Command("tuxguitar", path)` from PATH.
  - If the file is misnamed, it is first copied to `~/.cache/tabfinder/open/` (`cmd/tabfinder/library.go:98-117`).
  - Not found gives "TuxGuitar is not installed".
  - No per-OS code, so nothing for Windows.
- **TuxGuitar, Android:**
  - Copies the song to `cacheDir/open/<openAs>` and shares it through the FileProvider authority `${applicationId}.files`.
  - Opens it with `ACTION_VIEW`, `application/octet-stream` and `setPackage("app.tuxguitar.android.application")`, plus a `<queries>` entry in the manifest (`TuxGuitar.kt:13-45`).
  - File names are cleaned because TuxGuitar Android's path patterns fail on names with many dots (`internal/tuxguitar/tuxguitar.go:16`).
- **Android file access:**
  - `MANAGE_EXTERNAL_STORAGE` on API 30+; `READ_EXTERNAL_STORAGE` plus `requestLegacyExternalStorage` on API 29.
  - SAF `OpenDocumentTree` is used only to get a path, mapped from `primary:X` to `/storage/emulated/0/X`. Other providers are refused (`Finder.kt` `treeToPath`, `MainActivity.kt:61,81`).
  - The root is stored in SharedPreferences. `tabscan` ships as `jniLibs/arm64-v8a/libtabscan.so`.
  - **Play Store risk:** all-files access needs Google's approval, and a tab browser is unlikely to qualify. A rewrite should use SAF throughout.
- **Desktop storage:** `$XDG_CONFIG_HOME/tabfinder/config.json` and `$XDG_CACHE_HOME/tabfinder/index.jsonl`.
- **Audio:** none.
- **Licensing:**
  - Go dependencies: gioui (Unlicense/MIT), x/* (BSD), godbus (BSD-2).
  - Android: AndroidX (Apache-2.0).
  - Bundled fonts are OFL (`cmd/tabfinder/fonts/*-OFL.txt`).
  - TuxGuitar is only launched, never linked.

### 1.6 Stack, perf and CI

- **Stack:** Go 1.27, Kotlin 2.3, Compose BOM 2026.03, AGP 9.0 (`mise.toml`, `android/gradle/libs.versions.toml`).
- **Perf targets:**
  - Desktop heap < 30 MB with 2000 songs (E-DSK-30).
  - Frames < 16 ms with 1000 rows (E-DSK-26).
  - Android cold start < 1000 ms and jank < 5% (`scripts/perf-device.sh`, never run).
  - AOT `compile -m speed` at install saves about 1 s of startup.
- **CI:**
  - `test.yml`: Go race tests with offscreen EGL; Android JVM tests; an emulator matrix on API 29 and 34.
  - `release.yml`: signed APK, Linux tarball, and the CLIs for Linux, macOS and Windows.

---

## 2. Drum Practice (music-tools)

### 2.1 Feature list

The platform is Android API 29+ for v1. Linux desktop comes after v1 (`drum-practice/README.md:3`).

Nothing is released yet. `drum-practice/CHANGELOG.json` has `releases: []` and everything is under `upcoming`. "Built" below means on master, each item tied to a `.feature` folder under `drum-practice/core/acceptance/features/`.

**Built**
- **Notation:** real drum notation. Sticking letters under the snare notes, a title, a scrollable paper look, light and dark themes (`see-default-pattern/`).
- **Play-along:**
  - A click plus synthesised snare and bass drum, accent-aware.
  - The playing note lights up, with a playing mark and a follow scroll.
  - Stops on going to the background or losing audio focus (`hear-it/`).
- **Tempo:**
  - 40-240 BPM in steps of 5.
  - The tempo unit follows the time signature: a dotted quarter in compound time, an eighth in 3/8 and 7/8.
  - Tempo increase: x % every n rounds, with the running tempo shown (`tempo-increase/`; `GLOSSARY.md:66-70,249-259`).
- **Editing:** tap a beat, then use the beat editor (`edit-it/`, `time-signatures/`).
  - Note values: 1/2/3/4/6 per quarter beat; 1/3/6/9 per dotted-quarter beat.
  - Per note: accent, snare hand R/L/off/Hold, bass drum.
  - Per pattern: 1-64 bars, time signature (2/4-7/4, 3/8, 6/8, 7/8, 9/8, 12/8), a name of at most 40 characters.
  - Edits made while playing are heard from the next note.
- **Ornaments:** flam, drag, buzz and roll, offered only where they fit. Hold lengthens the previous note (ADR 0004/0005; `grace-notes/`, `holds-and-rolls/`).
- **Bar selection:** copy, or delete with a two-tap confirm (`bar-selection/`). One repeat sign with 2-8 passes (`repeat-signs/`).
- **Library:**
  - Save, auto-name "Pattern n", filter, load, delete, new pattern (`save-and-library/`).
  - About 43 factory patterns: all 40 PAS rudiments plus grooves, in groups (`drum-practice/core/model/factory_patterns.go`; `factory-patterns/`, `rudiments-and-groups/`).
  - Editing a factory pattern saves an own copy.
- **Persistence:** in the app's private folder (`drum-practice/docs/DESIGN.md:128-137`).
  - Pattern files: `patterns/<id>.json`. Only the current `patternVersion` (v5) is read; older ones are skipped, not migrated.
  - `settings.json` holds theme and zoom; `state.json` holds the hint flag.
  - Atomic writes. Android backup is on.
- **Settings and texts:**
  - Theme Auto/Light/Dark.
  - Languages de/en/fr follow the system language. Every text comes from the Go catalogue (`walking-skeleton/`; `stave/docs/adr/0001`).
  - Tablet layout with an 840 dp column.
- **Not present:** no import or export, no share, no SAF, no MIDI.
- **Store listing:** `drum-practice/store/listing/{en,de,fr}.md` and `privacy-policy.md`.
  - Promises no ads, no account, no internet, no data collected.
  - Stale: says "24 rudiments" where the code has 40.

**In progress / roadmap** (`drum-practice/ROADMAP.md`, `.scratch/*/issues`)
- **Notation zoom:** the features exist (`notation-zoom/`), but the issue is still open.
- **Scale by width.**
- **What's new sheet.**
- **Groups for own patterns.**
- **Play Store pipeline.**

**Future notes** (`docs/FUTURE-NOTES.md`)
- Share code by the rule of three.
- Stave in Full vs Viewer modes.

### 2.2 shared/

| Module | Provides | Used by |
|---|---|---|
| `shared/model` | Song tree (Song, BarHeader, Track, Bar, Beat, Note); 11,531,520 ticks per quarter | Stave only. DP's Pattern stays local (`shared/docs/adr/0001`) |
| `shared/formats` | Read-only GP3-5, GPX, GP7/8 (shared GPIF XML), `Detect`/`Read` (`formats.go:37`); byte builders for tests | Stave |
| `shared/notation` + `shared/schema/display.fbs` | Display list: Line, Curve, Rect, Glyph, Text, BeatRegion; FlatBuffers wire format | DP and Stave |
| `shared/designsystem` (Gradle) | "Music Tools Neutral" Compose theme from `tokens.json`; Button, NumberStepper, Select, SegmentedControl, SearchField, TopSheet, PlacedPopover, InlineHint, NotationArea; `gonville_smufl.otf` | DP and Stave |

The shared Go module's only dependency is flatbuffers.

Note: TabFinder and shared/formats each have their own GP parser.

### 2.3 Notation and display list (Stave included as reference)

- **One display list and one FlatBuffers schema** for both apps' notation areas (`shared/docs/adr/0002`).
  - Go places boxes with fixed metrics; Kotlin fits text into them and paints the mask (rule kept from `stave/docs/adr/0005`).
  - Stave makes one display list per system; DP makes one per pattern, scaled by zoom.
- **Layout is a pure function** of (song, track, width) (`stave/docs/DESIGN.md`).
- **Fonts:** Gonville SMuFL, built from pinned commit `21a541e` and committed. Metrics come from `drum-practice/tools/glyphmetrics` (`drum-practice/docs/adr/0002`).
  - Bravura and other OFL fonts were rejected; the UI uses platform fonts.
  - No Gonville license file is committed next to the `.otf`.
- **Reference-only projects:** TuxGuitar (LGPL), alphaTab (MPL) and PyGuitarPro (LGPL) are references only, never copied. MPL/GPL test files stay in a gitignored `local/` (`stave/docs/SPEC-1.0.0.md:414,444`).

### 2.4 Domain terms (`GLOSSARY.md`, 348 lines)

- **Music:**
  - Beat: one counted slot of a bar.
  - Beat length: 2 or 3 eighths, derived.
  - Onset (Stave): where notes start.
  - Note.
  - Ornament: plain, Flam, Drag, Buzz, Roll.
  - Hold: lengthens the previous note; stored, not drawn as a tie.
  - Grace note, Rest.
  - Bar ("measure" is to be avoided).
  - Tempo, Tempo unit.
  - Tick (Stave).
- **App:** Catalogue, System language, Session, Theme, View model, Quality gate, App version, Version code.
- **Drum Practice:**
  - Notation: Sticking, Beam/Stub/Flag.
  - Patterns: Pattern, Library, Factory pattern, Group, Auto-name, Pattern id/file, Saved state (a comparison, not a dirty flag).
  - Repeats: Round, Repeat, Repeat sign, Pass.
  - Tempo: Tempo increase, Set tempo, Running tempo.
  - Screen: Mode (practice/edit), Default pattern.
  - Editing: Display list, Beat region, Beat editor, Note value, Note cell.
  - Playback view: Playing note/system/mark, Follow position.
  - Zoom: Zoom, Default zoom, User zoom.
  - Audio: Output, Ring, Stroke seed, Limiter.
- **Stave:** Song, Track, Guitar or bass track, Bar header, Voice, Reader, Score screen, Paper, System, Tab staff, File tab, Panel, Track row.

### 2.5 Specs and corpora

- **Gherkin:**
  - 65 files with about 287 scenarios in `drum-practice/core/acceptance/features/`, run by godog at the core facade.
  - Scenarios per folder: edit-it 51, save-and-library 37, hear-it 31, time-signatures 23, walking-skeleton 22, others 9-16.
  - Stave has 15 files with about 62 scenarios (`stave/core/features/`), useful for GP opening and track switching.
- **Golden views:** 9 files in `drum-practice/app/screens/src/androidHostTest/resources/golden/`, written by `drum-practice/core/bridge/golden_view_test.go -update`.
- **Fuzz corpora:**
  - `drum-practice/core/storage/testdata/fuzz/FuzzReadPattern/`: 24 pattern-JSON edge cases.
  - `shared/formats/testdata/fuzz/`: 27.
  - `stave/core/bridge/testdata/fuzz/FuzzOpenFile`: 250.
- **Manual test plan:** `drum-practice/docs/MANUAL-TEST-PLAN.md`. Property tests: `drum-practice/core/*_property_test.go`.

### 2.6 Platform integrations

- **Audio** (`drum-practice/docs/adr/0003`):
  - A Go producer renders float32 stereo ahead into a lock-free ring with about 60 ms lead. The sample counter is the clock (`nextNoteFrame`).
  - The real-time callback never enters Go. Android uses a cgo AAudio shim (`drum-practice/core/audio/aaudio/`).
  - The playhead comes from `AAudioStream_getTimestamp`, so the highlight matches what is heard, Bluetooth included.
  - Linux: a PipeWire shim through `dlopen` is designed, not built. Windows: none.
  - oto, malgo/miniaudio and pulse were rejected. Sounds are synthesised (seeded per stroke, soft limiter), with no samples.
  - Kotlin `PlaybackGuard`/`AudioEnvironment` handle audio focus, becoming-noisy, background stop and device reopen.
- **Bridge:** Go `-buildmode=c-shared` produces `libdp.so` for arm64-v8a and x86_64, built with NDK 28. JNA 5.19 is called through a Proxy over `ViewSource`; FlatBuffers 25.2.10 runs on both sides (`stave/docs/adr/0002,0003`).
  - gomobile was rejected (no desktop target), as were JNI and FFM.
  - The per-frame call passes scalars only.
- **Files:**
  - DP uses only its private folders.
  - Stave uses SAF `OpenDocument` plus `contentResolver` (`stave/app/androidApp/.../MainActivity.kt:21,38`).
- **Licensing:** no GPL dependencies; Go deps are godog, flatbuffers and x/image. There is no LICENSE file for drum-practice or the repo root; only Stave has one (MIT/Apache).

### 2.7 Stack, perf and CI

- **Stack:** Go 1.27, Kotlin 2.4, Compose MP 1.12, AGP 9.4, detekt (`mise.toml`, `app/gradle/libs.versions.toml`).
- **Quality gates:** 100 % Go statement coverage (`scripts/coverage.sh`), depguard layer rules, gremlins mutation testing, fuzzing.
- **Perf:**
  - `scripts/perf.sh` counts instructions under cachegrind.
  - `tools/perfgate` compares 8 benchmarks to `perf/baseline.txt`: layout, engine fill, stroke, view, zoom.
- **CI:**
  - `.github/workflows/drum-practice.yml`: path-filtered; lint, test, security, perf, generate-check, emulator e2e.
  - `drum-practice-release.yml`: on tag `drum-practice/v*`, a signed APK and bundle as a GitHub release. This matches our planned tag scheme.

---

## 3. Conflicting terms across apps

| Term | Drum Practice | Stave / shared | TabFinder |
|---|---|---|---|
| Beat | A metric slot of a bar | An onset; GP's "Beat" (`shared/model.Beat`), rename pending (`GLOSSARY.md:25`) | `score` package beats = onsets |
| Note | Snare/bass position with ornament or hold | A fretted string | n/a |
| Tempo | Set by the user in the signature's tempo unit; set vs running tempo | Read from the song, in quarter BPM | Per-bar BPM list from the file; filter on any BPM |
| Bar | A pattern bar | One track's content at a bar index; "Bar header" = GP master bar | Score bars |
| Track | n/a ("exercise" and "groove" are not terms; DP says Pattern, Groove is a group name) | A GP track | Metadata (`tab.Track`) vs beats (`score.Track`) |
| Session | Mode/tempo/theme/open pattern | Open files plus handles | n/a |
| Rest | Neither drum plays | An onset with no notes | n/a |
| Mode | Practice/edit | "Stave modes" Full/Viewer (FUTURE-NOTES) | n/a |
| Display list | One per pattern | One per system | n/a |
| Score | n/a | A notated song (generic) | The note-data package, AND the difficulty float |
| Song | n/a | A GP file's song tree | Three different structs (parse, row, Kotlin) |
| Play-along | Synthesised audio (a changelog id, not a glossary term) | Silent playback | Delegated to TuxGuitar |

---

## 4. Platform integrations summary (for the "Not yet specified" list in #1)

| Integration | Today | Gap vs targets (Win/Linux/Android, Play-Store-safe) |
|---|---|---|
| Open in TuxGuitar | Linux: process from PATH. Android: intent to `app.tuxguitar.android.application` via FileProvider | No Windows path |
| Tab library access, Android | All-files access; SAF only to get a path | Play policy risk; needs SAF and `contentResolver` throughout |
| Tab library access, desktop | kdialog/zenity picker, XDG config and cache | Linux-only tools; Windows needs native dialogs and paths |
| Audio | AAudio C shim behind a render-ahead ring | Linux PipeWire not built; no Windows |
| Core↔UI bridge | Go c-shared + JNA + FlatBuffers (DP); subprocess JSON lines (TabFinder Android) | Both are prior art for the Rust↔Compose bridge |
| Notation font | Gonville SMuFL (waived/MIT) | Add a license file |

## Not checked

- The difficulty scoring formulas in `internal/difficulty/scoring.go`.
- Feature-file bodies beyond names and counts. Scenario counts come from grep.
- Stave's ROADMAP.
- godog's and godbus's licenses, beyond what go.mod states.
- TuxGuitar's own license; it is believed to be LGPL. Irrelevant while it is only launched.
