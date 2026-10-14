# Jobs — `phiber/v2/area`

| State | Open | Done |
|---|---|---|
| 🟡 in progress | 1 | 2 |

## Todo

- [ ] Area sizes restored from the file's screen

## Done

- [x] Toggle Maximize Area (`screen.screen_full_area`, Ctrl Space, every editor's View menu — `area::maximize`): the area showing the editor whose menu fired it grows to fill the workspace (one Center zone, one area), and the same press — from any editor's menu — puts the areas back; per workspace, not resumed with the session. Blender maximizes the area under the pointer; a menu entry knows its editor, so the op carries the editor's slug and the rule finds its area — deliberate, recorded. Replaces eight `op()` placeholders. (`area::maximize` tests)
- [x] Area context for editor switching

Format: one line per job, `- [ ]` open / `- [x]` done; move a line from Todo to Done when it lands, with its test.
