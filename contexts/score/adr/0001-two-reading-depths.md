---
status: accepted
---

# Tab file readers read at two depths: header or whole Score

A Tab file reader offers two depths. A **header read** returns only what a Score states before its Bars (title, artist, album, Tracks with their Tunings, the opening Written tempo) and skips the Bars. A **full read** returns the whole Score, Bars and Onsets included.

TabFinder Scans whole Libraries, so it needs the header read to stay fast; reading every note of thousands of files would break the scan budget. A later app showing the music as notation needs the full read. Both depths share format detection and limits, so a file is recognized the same way at either depth.

Only the header read gets built for TabFinder now; the full read follows when an app needs it.

## Written tempos in Guitar Pro 3–5 (#44)

A header read returns every Written tempo, the opening one and its changes, so TabFinder can show a Song's BPM range. Guitar Pro 6–8 state all tempos in the Score XML before the Tracks. Guitar Pro 3–5 state only the opening tempo in the header; the changes sit inside the Bars, beside the notes, and the format has no way to jump past a Bar. So for these formats the header read walks every Bar, skipping each note without keeping it; the maintainer chose this over showing only the opening tempo. The scan budget (`bench:tabfinder`) still holds with a realistic Tab file. A Bar the walk cannot follow costs only the tempo changes, never the rest of the Score header; likewise a tempo no reader can make sense of is left out, in any format. Only 1 to 1000 beats per minute count as a Written tempo, so a walk that has lost its step cannot show a made-up tempo.
