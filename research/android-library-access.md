# Android tab library access without all-files access

Research for [#12](https://github.com/sbradl/musicians-toolbox/issues/12), part of map [#1](https://github.com/sbradl/musicians-toolbox/issues/1).

**Question:** How can TabFinder on Android scan and open a tab library (thousands of files, nested folders) without `MANAGE_EXTERNAL_STORAGE`? How does it hand bytes to the Rust core, and how does it open a song in TuxGuitar?

## Answer

- **Use SAF.** The user picks the library folder once with `ACTION_OPEN_DOCUMENT_TREE`, and the app persists the grant. No storage permission is needed, so nothing blocks a Play release.
- **Kotlin walks the tree, Rust parses.** Kotlin walks the tree with `DocumentsContract`: one query per directory, which returns name, size and mtime for every child. Kotlin opens each changed file as a `ParcelFileDescriptor`, detaches the raw fd and passes the int to Rust. Rust adopts it with `File::from_raw_fd` and reads normally. Rust never sees paths on Android.
- **Main cost: opening each file is an IPC call.** On AOSP each open also runs a MediaStore scan. Make rescans incremental, keyed by (document ID, size, mtime), so only new or changed files are opened. The first full scan pays the cost once. Measure it on a real device (not measured here).
- **MediaStore and app-specific storage are dead ends** for a user's existing library of non-media files.
- **To open in TuxGuitar,** keep tab-sync's approach. Copy the bytes into `cacheDir` under a clean `<title>.<real-ext>` name, share the copy through `FileProvider`, and send `ACTION_VIEW` with `FLAG_GRANT_READ_URI_PERMISSION` and `setPackage("app.tuxguitar.android.application")`. TuxGuitar works out the format from the URI's last path segment, so the clean name matters.

## Current state (tab-sync reference)

`/home/sbradl/Projects/tab-sync/android`:

- The manifest declares `MANAGE_EXTERNAL_STORAGE`, plus `requestLegacyExternalStorage` for Android 10. The comment explains why: "tabscan needs plain file paths, which scoped storage doesn't give" (`app/src/main/AndroidManifest.xml`).
- The user already picks the folder with `OpenDocumentTree`. The app then converts the tree URI back into a file system path (`Finder.treeToPath`, `data/Finder.kt`) and runs `tabscan -serve` on that path. The conversion only works for `com.android.externalstorage.documents` and only reads files because of all-files access.
- To open a song, the app copies it to `cacheDir/open/<openAs>`, shares it through `FileProvider` and sends `ACTION_VIEW` with `application/octet-stream` to TuxGuitar's package (`TuxGuitar.kt`). `openAs` is the title plus the real format's extension. The originals "may be misnamed (.crdownload, .zip, wrong gp version), and TuxGuitar's Android path patterns fail on names with many dots" (`internal/tuxguitar/tuxguitar.go`).

So the folder picker and the TuxGuitar handoff survive as they are. What changes is how the files are read: through the tree URI instead of a path.

## 1. Why not `MANAGE_EXTERNAL_STORAGE`

- Play's policy lists permitted uses as "file managers, backup and restore apps, anti-virus apps, and document management apps". It recommends SAF "as the privacy-friendly option". Approval is decided case by case through a Permissions Declaration Form, so it is uncertain for a tab finder. ([Play policy: All files access](https://support.google.com/googleplay/android-developer/answer/10467955))
- Android's own guidance says: "Request the `MANAGE_EXTERNAL_STORAGE` permission only when your app can't effectively make use of the more privacy-friendly APIs, such as the Storage Access Framework or the Media Store API." ([Manage all files](https://developer.android.com/training/data-storage/manage-all-files))
- **Verdict:** possible ("on-device file search" is on the likely-allowed list), but it depends on review and is unnecessary because SAF covers the use case. Keep it out of the manifest.

## 2. SAF: `ACTION_OPEN_DOCUMENT_TREE` and persisted grants

All quotes and limits in this section come from [Access documents and other files](https://developer.android.com/training/data-storage/shared/documents-files) unless another source is linked.

- **Scope:** the app "can then access any file in the selected directory and any of its sub-directories". It needs no system permission.
- **Folders that can't be picked on Android 11+:** the root of internal storage, the root of each "reliable" SD card, the `Download` folder, and the `Android/data` and `Android/obb` folders. AOSP enforces this in `ExternalStorageProvider.shouldBlockDirectoryFromTree`, which blocks the volume root, `Download/` and `Android/`, but allows everything on removable USB storage. ([source](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/main/packages/ExternalStorageProvider/src/com/android/externalstorage/ExternalStorageProvider.java))
  - **UX consequence:** the library must live in a subfolder such as `Music/Tabs` or `Documents/Tabs`, not in `Download/` itself. A subfolder of `Download` is allowed because only the exact path is blocked. Tell users this in the empty state.
- **Persisting the grant:** call `contentResolver.takePersistableUriPermission(uri, FLAG_GRANT_READ_URI_PERMISSION)`. Without it, the grant "lasts until the user's device restarts". Caveat from the doc: the app loses access "if the associated document is moved or deleted". If the library folder is renamed, the user has to pick it again.
- **Grant limit:** 512 persisted grants per app (`MAX_PERSISTED_URI_GRANTS = 512` in [UriGrantsManagerService](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/main/services/core/java/com/android/server/uri/UriGrantsManagerService.java)). One tree grant covers the whole library, so this doesn't matter.
- **Other providers:** the picker can also return trees from other providers (SD cards, USB OTG, cloud). The tree code below works against any provider. Only `ExternalStorageProvider` document IDs carry the relative path (see section 3).

## 3. Scan performance

**Don't use `DocumentFile` to walk the tree.** `TreeDocumentFile.listFiles()` queries only `COLUMN_DOCUMENT_ID`. Every later `getName()`, `length()` or `lastModified()` call runs its own provider query through `DocumentsContractApi19`. That makes one IPC call per file per attribute. ([TreeDocumentFile.java](https://android.googlesource.com/platform/frameworks/support/+/refs/heads/androidx-main/documentfile/documentfile/src/main/java/androidx/documentfile/provider/TreeDocumentFile.java)) Android warns: "If you iterate through a large number of files within the directory that's accessed using `ACTION_OPEN_DOCUMENT_TREE`, your app's performance might be reduced." ([documents-files](https://developer.android.com/training/data-storage/shared/documents-files))

**Do this instead: one `DocumentsContract` query per directory.**

- Query `buildChildDocumentsUriUsingTree(treeUri, dirDocId)` with the projection `DOCUMENT_ID, DISPLAY_NAME, MIME_TYPE, SIZE, LAST_MODIFIED`.
- In AOSP, `FileSystemProvider.queryChildDocuments` lists the directory and fills every requested column in that single call. ([FileSystemProvider.java](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/main/core/java/com/android/internal/content/FileSystemProvider.java))
- **Listing cost:** about one IPC call per directory, not per file. Listing a few thousand files should be cheap. This is an inference from the source, not measured.
- **Free relative paths:** `ExternalStorageProvider` document IDs are `<volume>:<relative path>` (for example `primary:Music/Tabs/Artist/song.gp5`). The relative path that the core indexes by comes with each row (see `getPathFromDocId` in the source above and tab-sync's `treeToPath`). For other providers, rebuild the path from the display names during the walk.

**Opening each file costs more than listing.** `ContentResolver.openFileDescriptor(docUri, "r")` goes through `FileSystemProvider.openDocument`, which for read-only access calls `openFileForRead`. That method first runs `MediaStore.scanFile(...)` on the file, then opens it through MediaStore's `openTypedAssetFileDescriptor` so MediaProvider can decide about transcoding. ([FileSystemProvider.java](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/main/core/java/com/android/internal/content/FileSystemProvider.java)) So every open costs several IPC calls plus a MediaStore scan. That is the dominant cost of a full content scan of thousands of files. Mitigations:

1. **Incremental index:** store (document ID, size, mtime) in the core's index. On rescan, list the tree (cheap) and open only new or changed files. tab-sync already keeps an index file (`Finder.kt`, `index.path`), so this fits.
2. **Parallel opens:** opens are blocking binder calls and can run on a small pool of IO threads. Rust parses as fds arrive.
3. **Measure before optimising further.** Build a throwaway benchmark (a few thousand GP files on a mid-range phone) to get real numbers for list time and open+parse time. Treat those numbers as a release gate, like the 16 KB page-alignment gate from #3.

**Direct paths are no way around this.** The Android 11+ "direct file path" access applies to media files only. `MediaStore.Files` shows other apps' files only if they are images, video or audio ([media](https://developer.android.com/training/data-storage/shared/media)). The fast path is closed to `.gp5`, `.tg` and `.ptb` files without all-files access. That is why tab-sync needed `MANAGE_EXTERNAL_STORAGE`.

## 4. Handing bytes to the Rust core

- `ParcelFileDescriptor.detachFd()`: "Return the native fd int for this ParcelFileDescriptor and detach it from the object here. You are now responsible for closing the fd in native code." ([ParcelFileDescriptor](https://developer.android.com/reference/android/os/ParcelFileDescriptor)) Use `getFd()` instead if Kotlin keeps ownership and closes the descriptor afterwards.
- Rust side: `unsafe { std::fs::File::from_raw_fd(fd) }`. The fd "must be an owned file descriptor", and the returned `File` closes it on drop ([FromRawFd](https://doc.rust-lang.org/std/os/fd/trait.FromRawFd.html)). After that it is ordinary `Read`/`Seek`, so the core's readers stay platform-neutral: they take `impl Read + Seek` or a byte slice, never a path.
- **Recommended bridge shape:**
  - Kotlin produces a batch of `(relPath, docId, size, mtime)` from the listing.
  - The core answers with the entries it needs to (re)read.
  - Kotlin opens each one and calls `core.ingest(docId, fd)`.
  - Desktop uses the same core API with `File::open`, so only the "walk and open" layer differs per platform.
- Tab files are small, so `read_to_end` into a `Vec` followed by parsing is fine. mmap is unnecessary.
- **Rejected alternative:** reading the bytes in Kotlin and passing a `ByteArray`. It costs an extra copy across JNI and buys nothing.

## 5. MediaStore: not usable for this

- With scoped storage, `MediaStore.Files` "shows only the photos, videos, and audio files that your app has created", plus other apps' media if the app holds the read-media permission. Tab files are neither. ([media](https://developer.android.com/training/data-storage/shared/media))
- Files in `MediaStore.Downloads` that the app didn't create can only be reached through SAF ("you must use the Storage Access Framework", same page).
- On Android 13+, `READ_EXTERNAL_STORAGE` is replaced by the per-type `READ_MEDIA_IMAGES`, `READ_MEDIA_VIDEO` and `READ_MEDIA_AUDIO` permissions. None of them covers documents. ([Android 13 behavior changes](https://developer.android.com/about/versions/13/behavior-changes-13))
- Only `MANAGE_EXTERNAL_STORAGE` grants "Access to the contents of the `MediaStore.Files` table" in full ([manage-all-files](https://developer.android.com/training/data-storage/manage-all-files)).

## 6. App-specific storage: only for the app's own files

- The app's own directories (`filesDir`, `cacheDir`, `getExternalFilesDir`) need no permission. They are the right place for the index, the cache copies for TuxGuitar and settings.
- They are the wrong place for the user's library. `Android/data` can't be picked through SAF on Android 11+ (section 2), so the user can't easily put files there. The files are also deleted on uninstall.
- An "import into the app" flow would duplicate the library and break the link to the user's sync folder. Not recommended.

## 7. Opening a song in TuxGuitar

What TuxGuitar for Android accepts (helge17/tuxguitar, `master`):

- **Intent filter:** `TGActivity` accepts `VIEW` and `EDIT` with `content` and `file` schemes and `mimeType="*/*"`, plus extension path patterns for `.tg`, `.gp*`, `.ptb`, `.tef` and `.mid`. ([AndroidManifest.xml](https://github.com/helge17/tuxguitar/blob/master/android/build-scripts/tuxguitar-android/apk/src/main/AndroidManifest.xml))
- **How it handles the intent:** `TGProcessIntentAction` passes `intent.getData()` to `TGUriReadAction`. That action calls `getContentResolver().openInputStream(uri)`, buffers the whole stream and picks the reader with `TGFileFormatUtils.getFileFormatCode(uri.getLastPathSegment())`, i.e. the text after the last `.` in the last path segment. ([TGProcessIntentAction](https://github.com/helge17/tuxguitar/blob/master/android/TuxGuitar-android/src/app/tuxguitar/android/action/impl/intent/TGProcessIntentAction.java), [TGUriReadAction](https://github.com/helge17/tuxguitar/blob/master/android/TuxGuitar-android/src/app/tuxguitar/android/action/impl/storage/uri/TGUriReadAction.java), [TGFileFormatUtils](https://github.com/helge17/tuxguitar/blob/master/common/TuxGuitar-lib/src/main/java/app/tuxguitar/io/base/TGFileFormatUtils.java))
- **Consequence:** the format comes from the URI's file name, not from the MIME type or the file's content. A misnamed file opens with the wrong reader or fails.

Options:

1. **Re-grant the SAF document URI directly (works for well-named files).** An app may grant a URI permission it holds itself: `UriGrantsManagerService` checks "does the caller itself have permission to access this uri?" ([source](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/main/services/core/java/com/android/server/uri/UriGrantsManagerService.java)). A tree-derived document URI's last segment decodes to `primary:Music/Tabs/Artist/song.gp5`, which ends in the right extension.
   - **Fails for misnamed files** (`.crdownload`, wrong gp version, many dots), which tab-sync explicitly handles.
   - The last segment of a non-ExternalStorage provider's document IDs may not end in a file name at all.
   - Not tested on a device.
2. **Cache copy plus FileProvider (recommended; current tab-sync behavior).**
   - Read the bytes through the SAF fd (already open for the core, or open it again).
   - Write them to `cacheDir/open/<clean title>.<real ext>`, where the core supplies the real extension (tab-sync's `tuxguitar.FileName`).
   - Get a URI with `FileProvider.getUriForFile` and send `ACTION_VIEW` with `setPackage(TUXGUITAR)` and `FLAG_GRANT_READ_URI_PERMISSION`.
   - Works for every provider and every misnamed file. Tab files are a few KB, so the copy costs nothing noticeable.
   - Downside: TuxGuitar edits go to the copy, not the original. That matches tab-sync today.
3. **Keep the `<queries><package android:name="app.tuxguitar.android.application"/></queries>` element.** It is needed for package visibility on Android 11+ so the app can detect whether TuxGuitar is installed. It already exists in tab-sync's manifest.

## Recommendations for #1

- **Android library access = SAF tree plus a persisted read grant.** Drop `MANAGE_EXTERNAL_STORAGE`, `requestLegacyExternalStorage` and `READ_EXTERNAL_STORAGE`. This keeps Play open.
- **UI/core line on Android:** Kotlin owns enumerating the tree and opening fds; Rust owns parsing and the index. The core API takes fds or readers, never paths. Desktop implements the same "walk and open" layer with `std::fs`.
- **Make the core's index incremental by (document ID, size, mtime) from day one.** This is the main lever for SAF performance and is also cheap on desktop.
- **Open songs in TuxGuitar with the cache copy plus FileProvider,** as tab-sync does.
- **Open item: a device benchmark** of tree listing and open+parse over a real library of several thousand files, to set a performance budget.

## Not verified

- No measurements were taken. All performance statements are inferred from AOSP source (`main` branch). OEM builds of `ExternalStorageProvider` and MediaProvider may differ.
- Re-granting a SAF URI to TuxGuitar (option 1) was not tested on a device.
