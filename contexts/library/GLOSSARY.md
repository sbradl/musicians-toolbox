# Library

A musician's collection of tab files: finding songs in it and reorganizing it on disk.

## Language

### Collection

**Library**:
All Songs found under one chosen folder.
_Avoid_: Collection, catalog

**Library location**:
Where a Library's folder is: a path on desktop, a folder grant (tree URI) on Android.
_Avoid_: Path, root

**Song**:
One Tab file in a Library, known by its Song summary.
_Avoid_: Score (Score's word), tab, entry

**Song summary**:
What the Library knows about a Song: title, artist, album, Tunings, Written tempos and Tags per Part.
_Avoid_: Metadata, row

**Source**:
Where a summary field came from: the file itself, or its folder path and file name as a fallback.

**Unreadable**:
A Song whose Tab file could not be read as any File format.
_Avoid_: Broken, invalid

**Scan**:
Reading a Library's files to bring its Song summaries up to date.
_Avoid_: Index, refresh

### Finding

**Filter**:
The criteria a musician narrows a Library by: artist, song, Tuning, Written tempo range, Tags.
_Avoid_: Query, search

### Reorganizing

**Plan**:
The Moves that would bring a Library into the `Artist/Album/Song` layout, shown before anything moves.
_Avoid_: Dry run

**Move**:
One Tab file's change of folder and, optionally, of name.

**Clash**:
Two Songs that would get the same name in one folder; both keep their names.

**Collision**:
A new name already taken in its folder by an unrelated file; the Move gets a numbered name.

**Duplicate**:
Two Tab files with identical content.
