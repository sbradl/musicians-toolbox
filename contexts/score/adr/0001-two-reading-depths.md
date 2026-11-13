---
status: accepted
---

# Tab file readers read at two depths: header or whole Score

A Tab file reader offers two depths. A **header read** returns only what a Score states before its Bars (title, artist, album, Tracks with their Tunings, the opening Written tempo) and skips the Bars. A **full read** returns the whole Score, Bars and Onsets included.

TabFinder Scans whole Libraries, so it needs the header read to stay fast; reading every note of thousands of files would break the scan budget. A later app showing the music as notation needs the full read. Both depths share format detection and limits, so a file is recognized the same way at either depth.

Only the header read gets built for TabFinder now; the full read follows when an app needs it.
