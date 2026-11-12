# Library

A musician's Library of Songs: finding Songs in it and reorganizing it on disk.

## Language

### Collection

**Library**:
All Songs found under one chosen folder.
_Avoid_: Collection, catalog

**Library location**:
Where a Library's folder is: a file URI on desktop, a folder grant (tree URI) on Android.
_Avoid_: Path, root

**Song**:
One piece of music in a Library, known by its Song summary.
_Avoid_: Score (Score's word), tab, tab file, file, entry

**Song summary**:
What the Library knows about a Song: title, artist, album, Tunings, Written tempos and Tags per Part.
_Avoid_: Metadata, row

**Source**:
Where a summary field came from: the Song itself, or its folder and name in the Library as a fallback.

**Unreadable**:
A Song that could not be read in any supported format.
_Avoid_: Broken, invalid

**Scan**:
Reading a Library to bring its Song summaries up to date.
_Avoid_: Index, refresh

**Rescan**:
A Scan of the open Library that the musician asks for.
_Avoid_: Refresh, reload

### Finding

**Filter**:
The criteria a musician narrows a Library by: artist, song, Tuning, Written tempo range, Tags.
_Avoid_: Query, search

### Reorganizing

**Plan**:
The Moves that would bring a Library into the `Artist/Album/Song` layout, shown before anything moves.
_Avoid_: Dry run

**Move**:
One Song's change of folder and, optionally, of name.

**Clash**:
Two Songs that would get the same name in one folder; both keep their names.

**Collision**:
A new name already taken in its folder by something unrelated; the Move gets a numbered name.

**Duplicate**:
Two Songs with identical content.
