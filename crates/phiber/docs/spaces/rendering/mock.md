# Rendering — Mock

Image editor maximised for render output, with an info strip.

## Composition

| Zone     | Slot | Default host | Notes                                  |
|----------|-----:|--------------|----------------------------------------|
| `Top`    |    1 | `Topbar`     |                                        |
| `Center` |    1 | `Image`      | `Render_Result` mode (Req 9.2)         |
| `Bottom` |    1 | `Info`       | Render log strip                       |
| `Bottom` |    2 | `Statusbar`  |                                        |

## Blender parity references

- `Image.image_user.frame_current` reflects the active render frame.
- Requirement 9.4/9.5: render state transitions.
