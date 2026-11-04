# Rust crates for Guitar Pro, TuxGuitar and Power Tab parsing

Research for [#4](https://github.com/sbradl/musicians-toolbox/issues/4), under map [#1](https://github.com/sbradl/musicians-toolbox/issues/1). Constraints from the map: no GPL in shipped code, performance first. Researched 2026-11-04.

## Answer

- **Guitar Pro 3-5, GPX (GP6) and .gp (GP7/8):** two usable, permissively licensed library crates exist: `guitarpro` (MIT) and `acorde-io` with feature `gp` (MIT OR Apache-2.0). Both read full notation. Both are young and maintained by one person. `acorde-io` defends much better against malformed input. Neither offers a fast metadata-only path.
- **.gtp (Guitar Pro 1/2):** no crate supports it. tab-sync rejects it too ("Guitar Pro 2 files are not supported").
- **TuxGuitar (.tg) and Power Tab (.ptb):** no Rust crate exists. Searches of crates.io and GitHub (Rust repos) found nothing. The only open-source readers are TuxGuitar itself (LGPL) and Power Tab Editor (GPL-3.0), so we can't ship code from either.
- **Recommendation:** write our own reader crate, with tab-sync and music-tools as the spec. Use `acorde-io` (and optionally `guitarpro`) only as a dev-dependency, to check our output against theirs in tests. TG and PTB have to be written in-house anyway, and the work is small: TabFinder reads only their headers. Details in [Recommendation](#recommendation).

## Crates found

Search: crates.io API queries "guitar pro", guitarpro, tuxguitar, "power tab", powertab, gp5, gpx, "guitar tab" ([crates.io API](https://crates.io/api/v1/crates?q=guitar%20pro)). I downloaded each crate's published source from crates.io and read it. I did not compile or run any of it (see [Not verified](#not-verified)).

| Crate | Version (date) | License | Formats read | Depth | Maintenance |
|---|---|---|---|---|---|
| [`guitarpro`](https://crates.io/crates/guitarpro) ([codeberg.org/slundi/scorelib](https://codeberg.org/slundi/scorelib)) | 0.4.5 (2026-11-03) | MIT | GP3, GP4, GP5, GPX, GP7/8, plus MusicXML and MSCZ; also writes them | Full notation | Created 2022, active (4 releases Jul-Nov 2026), 1 author, 0 stars on Codeberg, ~1.9k recent downloads |
| [`acorde-io`](https://crates.io/crates/acorde-io) ([github.com/kent-tokyo/acorde](https://github.com/kent-tokyo/acorde)), feature `gp` | 1.3.9 (2026-11-02) | MIT OR Apache-2.0 | GP3-5, GPX, GP7/8; import only | Full notation, mapped into `acorde-core::Score` | Repo created 2026-07-26, 2 contributors, 4 stars, very frequent releases |
| [`ruxguitar`](https://crates.io/crates/ruxguitar) ([github.com/agourlay/ruxguitar](https://github.com/agourlay/ruxguitar)) | 0.14.0 (2026-11-01) | Apache-2.0 | GP3, GP4, GP5, GPX, GP7 | Full notation | Active, 215 stars. **A binary crate (`[[bin]]` only, no `[lib]`)** that pulls in iced, cpal, tokio and rustysynth. We could vendor its ~6.1k-line `src/parser` with attribution, but we can't add it as a dependency. |
| [`score-tab`](https://crates.io/crates/score-tab) (same author as `guitarpro`) | 0.2.0 (2025-08-31) | MIT | Claims GP3-GP and MuseScore | — | One release, 9 recent downloads, README roadmap is a placeholder. Looks like an earlier attempt that `guitarpro` replaced. |
| [`gpx_reader`](https://crates.io/crates/gpx_reader) | 0.0.2 (2014-11-21) | MIT/BSD-2-Clause | GPX container only | Container | Dead. Pre-1.0 Rust (`#![feature(phase)]`), won't compile. |
| [`tuitar-core`](https://crates.io/crates/tuitar-core) | 0.1.0 | MIT OR Apache-2.0 | — | — | Not a parser. Its `build.rs` uses the `guitarpro` crate. |

Nothing for TuxGuitar or Power Tab came up. GitHub search for Rust repos matching "tuxguitar" returned only an MCP bridge and an unrelated editor. "powertab" and "power tab" returned nothing.

### `guitarpro` 0.4.5: details

- API: `Song::read_gp3/read_gp4/read_gp5/read_gpx/read_gp(&mut self, &[u8]) -> GpResult<()>` (`src/model/legacy/song.rs`). **No format detection**: the caller picks the reader. tab-sync shows that renamed files (a `.gp3` that is really GP5, `.crdownload`, zip wrappers) are common, so it sniffs the content instead (`tab-sync/internal/tab/parse.go`, `parseBytes`).
- It is a 40k-line crate with no cargo features. MusicXML, MSCZ, MIDI, chord/scale/fingering analysis and writers are always compiled in. Dependencies: `encoding_rs`, `fraction`, `quick-xml`, `serde`, `thiserror`, `zip` (`Cargo.toml`).
- The published crate has no LICENSE file. Only `license = "MIT"` in `Cargo.toml`.
- Robustness, from reading the code:
  - Good: binary primitives check bounds and return `GpError::UnexpectedEof` (`src/io/primitive.rs`). Measure and track counts are clamped to 100,000 (`song.rs:166-167`). Almost no `unwrap`/`expect` in parsing paths: two `expect`s in `key_signature.rs:273-274`, on fraction math.
  - **Risk:** the GPX decompressor calls `Vec::with_capacity(expected_len)` with the length declared in the file, up to 2^31-1 (`src/io/gpx.rs`, `decompress_bcfz`). A 12-byte malformed `.gpx` can ask for a ~2 GiB allocation. If the allocator refuses, the process aborts, and a Rust abort can't be caught. This matters most on Android. music-tools caps the same step at 32 MiB (`gpx.go`, `maxDecompressed`).
  - **Risk:** `.gp` reading calls `read_to_string` on `Content/score.gpif` with no size cap (zip bomb). It also always reads any embedded `Content/Assets/*.mp3` into memory (`read_gp_with_audio`). That costs a lot when scanning a library.
  - "Fuzz testing for binary parsers" is still unchecked in `Roadmap.md`.
- Tests: `audit_report.txt` lists 244 files read OK: 74 .gp, 73 .gpx, 51 .gp5, 33 .gp4, 13 .gp3, 18 .mxl. The rest are MusicXML files marked SKIP. The fixture files themselves are not in the published crate.
- The crate ships a `CLAUDE.md`, so it was written with AI help. Releases come fast and the API is pre-1.0 (the README example still says `guitarpro = "0.1.0"`).

### `acorde-io` 1.3.9 (`gp` feature): details

- API: `parse_gp(&[u8]) -> Result<Score, Error>` and `parse_gp_with_report`. It **detects the format from the bytes** (GP3-5 family, `BCFZ`/`BCFS`, zip) (`src/gp/mod.rs`). The GP code is about 4.1k lines: `gp/mod.rs`, `gp/gp5.rs`, `gp/gpx.rs`.
- Robustness: explicit limits on everything: `MAX_GP_COMPRESSED` 64 MiB, `MAX_GP_ENTRIES` 4096, `MAX_GPIF_BYTES` 64 MiB, `MAX_GPIF_ELEMENTS` 4M, `MAX_GPIF_DEPTH` 128, `MAX_GP_MEASURES` 10,000, GPX `MAX_DECOMPRESSED` 64 MiB, checked both before and during decompression (`src/gp/mod.rs:25-30`, `src/gp/gpx.rs`). Content outside its model becomes a typed diagnostic instead of being dropped silently.
- Trade-offs: its output is a notation model (`acorde-core::Score`: parts, staves, beams via `compute_beams`), not a tab/library model. For TabFinder's scan (metadata, tracks, tunings, tempos, notes for difficulty) it does more work than needed. Our types would still need converting from it. The project is three months old and has one main author.

## Licenses of the non-Rust references

These are references only. None can ship.

- TuxGuitar: "released under the GNU Lesser General Public License" ([helge17/tuxguitar README](https://github.com/helge17/tuxguitar)). Fine to read for format knowledge; don't copy code.
- Power Tab Editor: GPL-3.0 ([powertab/powertabeditor](https://github.com/powertab/powertabeditor)). Same.
- alphaTab: MPL-2.0 ([CoderLine/alphaTab LICENSE](https://github.com/CoderLine/alphaTab)). File-level copyleft, so copying its files would bring MPL with them. Fine to read.
- PyGuitarPro: LGPL-3.0 ([Perlence/PyGuitarPro](https://github.com/Perlence/PyGuitarPro)).
- ruxguitar (Apache-2.0) and `guitarpro`/`acorde-io` (MIT, MIT/Apache) are the only sources we could legally copy from, with attribution.

## Reference implementations (Go)

### tab-sync: `internal/tab`, `internal/tabfiles`

Package doc: "reads guitar tab files (Guitar Pro 3-7, TuxGuitar, Power Tab) and extracts artist, album, title, tempo, instruments and tunings" (`internal/tab/song.go`).

| Format | Coverage | Source |
|---|---|---|
| Sniffing | By content, not extension: zip, `BCFZ`/`BCFS`, `ptab`, "GUITAR" in the header, TuxGuitar magic. Unwraps plain zip archives that hold one tab. | `parse.go` `parseBytes`, `parseZip` |
| GP3/4/5 | Header, tracks, tunings, **full note data** (into `score.Score`, for difficulty), tempo changes from mix tables | `gp.go` (244 lines), `gpbeats.go` (329) |
| GPX (GP6) | BCFZ decompression, BCFS sector filesystem, then GPIF | `gpif.go` (582), `gpifxml.go` (351) |
| GP7/8 | Zip, then `Content/score.gpif` | `parse.go`, `gpif*.go` |
| TuxGuitar 1 (.tg) | **Header only**: title, artist, album (UTF-16BE length-prefixed strings) | `other.go` `parseTG` |
| TuxGuitar 2 (zip + `content.xml`) | Title, artist, album, channels, tempo, tracks with string tunings | `other.go` `tg2Doc`, `parseTG2` |
| Power Tab (.ptb) v3+ | **Header only**: title, artist, album (MFC CString). Rejects PT 1.0 and Unicode strings. | `other.go` `parsePTB`, `mfcString` |
| .gtp (GP1/2) | Rejected with an error | `testdata/expected.json` |

The parsers are about 1.8k lines of Go (`parse.go`, `gp.go`, `gpbeats.go`, `gpif.go`, `gpifxml.go`, `other.go`).

**Test corpus:** synthetic only. `internal/tabfiles` (about 1.1k lines) builds GP3, TuxGuitar 1/2, Power Tab, zip and GPIF files as bytes, "so parsing and end-to-end tests need no copyrighted fixtures" (`tabfiles.go`). `internal/tab/testdata/library` holds 14 small files (126 B to 20 KB). They cover gp3/gp4/gp5/gpx/gp/gtp/ptb/tg/zip/`.crdownload`, plus edge cases: a misnamed gp3, a truncated gp5 ("Cut Short"), PT 1.0, GP2. Expected results are in `testdata/expected.json`. There are 49 `Test*` functions plus `FuzzParseBytes` (`parse_test.go:472`).

### music-tools: `shared/formats`

| Package | Coverage | Lines (non-test) |
|---|---|---|
| `formats` | Detects GP3-5, clipboard, GPX, GP7 | 92 |
| `gp5` | GP3, GP4, GP5 (5.00/5.10) full notation into `model.Song` (bars, beats, notes, ticks) | 555 |
| `gpx` | BCFZ/BCFS with `maxDecompressed = 32 << 20` | 190 |
| `gpif` | GPIF XML decode, references, tracks, songs, parallel decode | 1,583 |
| `gp7` | Zip, then GPIF | 65 |
| `gp5bytes`, `gpxbytes`, `gp7bytes` | Builders that write synthetic files for tests | ~1,500 |

There is no TuxGuitar and no Power Tab support. **Test corpus:** synthetic builders, plus a Go fuzz corpus of about 30 seeds (`testdata/fuzz/FuzzReadingAnyBytesReturnsASongOrAnError`). Fuzz targets exist for `formats`, `gp5`, `gpx`, `gpif` and `gp7`. There are property tests (`property_test.go`) and limit tests (`gpif/limits_test.go`, `gpif/large_test.go`): 71 `Test*` functions.

Across both repos there are about 5.4k lines of Go tests.

## Recommendation

**Write our own reader crate. Don't depend on the crates above in shipped code.**

1. **TG and PTB leave no choice.** No crate exists, and TabFinder needs exactly what tab-sync reads: TG1/PTB headers and TG2 XML. That is about 150 lines of Go, so a small port.
2. **Performance.** TabFinder scans whole libraries. A reader built for that purpose can: sniff by content; skip RSE, page setup, lyrics and embedded audio; avoid deserializing the GPIF DOM where streaming `quick-xml` events are enough; and return only metadata, tracks, tunings, tempos and the notes needed for difficulty. Neither crate allows that. `guitarpro` also always decompresses embedded MP3s.
3. **Robustness on Android.** Uncaught allocation aborts (the GPX case in `guitarpro`) are unacceptable. `acorde-io` sets the bar we should copy: hard caps on input, decompressed size, zip entries, XML depth and element count, and measure count.
4. **Dependency risk.** Both candidates are pre-maturity projects led by one person, with weekly API churn.
5. **Effort.** Expect roughly 2-3k lines of Rust readers, plus about 1.5k lines of synthetic fixture builders (which keep the "no copyrighted fixtures" rule), plus tests. That rough figure comes from the Go line counts above: the formats are already understood and specified in our own code. Use `cargo-fuzz` targets in place of the Go fuzz targets.
6. **Make use of the crates in tests.** Add `acorde-io` (feature `gp`, MIT/Apache) as a **dev-dependency** and compare against it: our track count, tunings and tempos should match its output on the fixture corpus and on a private local library. ruxguitar's parser (Apache-2.0) is the best readable cross-check for GP3-5 edge cases.

If the timeline forces a shortcut, use `acorde-io` behind our own trait for GP3-8 as the fallback, not `guitarpro`. Its limits make it the safer choice. The price is a conversion from `Score` and extra notation work per file.

## Not verified

- I did not compile or run any crate. Running downloaded code was not permitted in this session. The robustness claims come from reading source. No fuzzing or benchmarks were run.
- I did not test format completeness against real-world GP files. The `guitarpro` coverage figures are its own `audit_report.txt`.
- Whether GP 4.06 "L" files and GP 5.2 load in `acorde-io` was not checked.
