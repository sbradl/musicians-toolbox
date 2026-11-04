---
status: accepted
---

# Rust owns view state; Compose renders and forwards intents

The UI stays as thin as possible: the Rust core owns all view state and logic (Elm/Crux-like), Compose renders what it gets and turns user input into intents. Builds on the bridge in ADR 0002.

- **ViewModel** means an immutable, complete value describing one screen, produced in Rust and crossing as a UniFFI record. Strings arrive formatted, actions arrive already enabled or disabled. No androidx ViewModels.
- **One `App` object per app**: `dispatch(intent)`, `onViewModel(AppViewModel)`, `onEffect(effect)`. Push only: intents return nothing, Rust coalesces, Kotlin pipes the callback into a StateFlow. The AppViewModel holds the shell (navigation, global status) plus the visible screen only; screens further back stay in Rust.
- **Navigation lives in Rust**: the stack is part of the AppViewModel, system back is the intent `Back`.
- **Compose keeps pure UI mechanics**: scroll position, text cursor/IME, focus, animations and screen transitions, gesture recognition. Rust drives them only through effects (e.g. `ScrollTo`).
- **Effects are not state**: delivered once via `onEffect`, in emission order with ViewModels. Platform answers (picked folder, opened file) come back as intents; the platform side is an adapter behind a port (ADR 0001).
- **Input**: widgets send semantic intents; text fields send the whole text (`FilterChanged(text)`), not keystrokes. Keyboard shortcuts are platform-dependent or absent, so Kotlin maps them to semantic intents; Rust only sees actions.
- **UI texts and i18n live in Rust** (Fluent; en, de, fr), including plurals, number and duration formatting, error messages and accessibility descriptions. Kotlin reports the locale as an intent. App name and manifest texts stay platform resources.
- **Visuals live in Compose**: colours, typography, icons, layout per window size. Rust sends only semantic facts (`severity`, `isSelected`) and the theme setting.
- **Notation** is laid out by Engraving into a display list in abstract units, which Compose draws on a `Canvas`; zoom and scroll stay in Compose. Format and fonts are decided with Drum Practice; the display list is the first FlatBuffers candidate (ADR 0002).
- **Size budget**: full lists in the ViewModel until measured. Budget: intent → rendered under 16 ms with 10k songs on a low-end Android device, measured with benchmarks and platform profilers (no logging, ADR 0005). Over budget → windowing (`totalCount` + rows of the visible range reported coarsely by Compose).
- **Android process death is ignored**: the app restarts on its start screen. Real data (library index, settings, grants) is persisted separately.
