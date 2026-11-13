# Score

A song as written in a tab file: its tracks, bars and notes. Every foreign tab file format is translated into this language at Score's boundary.

## Language

**Score**:
One song as written in one tab file.
_Avoid_: Song (Library's word), tab, file

**Tab file**:
A file holding one Score in a foreign format such as Guitar Pro, TuxGuitar or Power Tab.
_Avoid_: Tab

**File format**:
The foreign format a Tab file is written in, recognized by its content, not its name.
_Avoid_: Extension, type

**Score header**:
What a Tab file states before its Bars: title, artist, album, Tracks with their Tunings, the opening Written tempo. Read alone when the Bars aren't needed (ADR 0001).
_Avoid_: Metadata

**Track**:
One instrument's part of a Score, with its Tuning when it has strings.
_Avoid_: Instrument, channel

**Bar**:
One Track's content between two bar lines.
_Avoid_: Measure

**Onset**:
A point in a Bar where notes start, with its Duration; an Onset without notes is a rest.
_Avoid_: Beat (the foreign formats' word)

**Written tempo**:
The tempo the Score states, which may change from bar to bar.
_Avoid_: Tempo, BPM, playback tempo
