# Glossary Map

The domain of tools and apps for musicians, cut into bounded contexts by subdomain. Apps are compositions of contexts. A word may mean different things in different contexts; each glossary is authoritative only for its own context. Why the cut looks like this: [ADR 0001](./docs/adr/0001-bounded-contexts-and-context-map.md).

## Contexts

- [Music kernel](./docs/contexts/music-kernel/GLOSSARY.md): the few musical values every context agrees on
- [Score](./docs/contexts/score/GLOSSARY.md): songs as written in tab files, read from foreign file formats
- [Library](./contexts/library/GLOSSARY.md): a musician's collection of tab files: finding and reorganizing songs
- [Analysis](./docs/contexts/analysis/GLOSSARY.md): facts derived from a score, such as which techniques a part uses
- [Drum Pattern](./docs/contexts/drum-pattern/GLOSSARY.md): writing and practicing snare and bass drum patterns
- [Engraving](./docs/contexts/engraving/GLOSSARY.md): laying out music notation for drawing
- [Audio](./docs/contexts/audio/GLOSSARY.md): turning sound events into sound

## Relationships

- **Music kernel ↔ all**: shared kernel. Score, Library, Analysis, Drum Pattern and Engraving share its values by agreement; a change considers every consumer.
- **Score → Analysis**: customer/supplier. Score is upstream; Analysis reads a Score to derive Parts and Tags.
- **Score → Library**: customer/supplier. Score is upstream; Library takes only what a Song summary needs.
- **Analysis → Library**: customer/supplier. Library filters on an Analysis's Tags per Part.
- **Drum Pattern → Engraving**: Engraving is an open host with a published language; Drum Pattern translates a Pattern into it. Score will do the same for a tab player.
- **Drum Pattern → Audio**: Audio is an open host; Drum Pattern translates into Sound events. Audio knows nothing of bars or tempo.
- **Foreign file formats → Score**: an anti-corruption layer owned by Score translates every tab file format into Score's language.

## Compositions

- **TabFinder** and its CLIs: Library + Analysis + Score.
- **Drum Practice**: Drum Pattern + Engraving + Audio.
