# Animation — Mock

Timeline + Dope Sheet + viewport for keyframe animation.

## Composition

| Zone     | Slot | Default host | Notes                                    |
|----------|-----:|--------------|------------------------------------------|
| `Top`    |    1 | `Topbar`     |                                          |
| `Left`   |    1 | `Outliner`   | Data_Api display mode (Req 8.2)          |
| `Center` |    1 | `View3D`     | Pose Mode for armatures, else Object     |
| `Right`  |    1 | `Buts`       |                                          |
| `Bottom` |    1 | `Action`     | `Action`, `Ipo`, `Nla`, `Drivers` (Req 8.4) |

## Blender parity references

- `bl_ui/space_dopesheet.py` timeline + dope sheet.
- Playback tick fires at project fps (24 / 30 / 60) per Requirement 8.5.
