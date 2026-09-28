# Jobs — `phiber/v2/app`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 5 | 3 |

## Todo

- [ ] Loading window: while a file loads the status zone expands to a full-width panel (progress bar across the whole bottom edge, current stage, running log with timings, Cancel) and collapses back into the status bar when done
- [ ] Resume the last session: persist `BState` (minus transient keys) on change, restore on launch, reopen the last file with Load UI off
- [ ] Text input entity (`EntityInputHandler`) for search and text fields
- [ ] Swap in evaluated meshes when `Phx` finishes modifiers (progressive open)
- [ ] Collaboration service: writes `collab.*`, applies remote `BAction`s through `apply_action`

## Done

- [x] Open through `Phx::open_with` off the UI thread with progress and Cancel
- [x] Read report → Info log and status bar; Load UI (workspace, view, lens, shading)
- [x] Playback ticker at the file's fps; window opens with a self-drawn, themed title bar

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
