---
status: accepted (to be rechecked once the first code exists)
---

# Bounded contexts cut by subdomain, not by app

The repo hosts several separately released apps, with more to come, so contexts are cut by subdomain and apps are compositions of them (see `GLOSSARY-MAP.md`). The non-obvious cuts:

- **Drum Pattern and Score are separate models.** A Pattern counts metric slots; a Score is onset-based. Any translation between them goes through an explicit layer.
- **Tab file formats are an anti-corruption layer inside Score**, not a context: they have no domain behaviour beyond translating into Score.
- **Analysis is its own context**, a customer of Score and supplier to Tab Library, so derived facts (Tags per Part now, a rating later) can evolve and be reused without touching either.
- **Reorganizing a collection lives inside Tab Library**: same collection, same Song summaries.
- **Engraving and Audio are open hosts** with published languages; callers translate in, so neither knows Patterns, Scores, bars or tempo.
- **Music kernel** is a deliberately tiny shared kernel of values (Pitch, Duration, Time signature, Tuning).
- **Platform concerns are not contexts**: file access, opening a song in a tab editor, storage, UI texts and audio devices are adapters behind ports owned by the context that needs them.

Open: whether playback speed (playback tempo, tempo increase, loops, playing position) becomes a shared Playback context between the music models and Audio.
