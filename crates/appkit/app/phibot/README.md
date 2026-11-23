# phibot: the botanist

Apple's BOT-anist sample (visionOS, SwiftUI and RealityKit) as an app on
appkit. The robot and its greenhouse are phiace's world on phiecs
(`crates/phiace/src/bot/botanist*`): what the robot is and does is
phiace's tables (`crates/phiace/data/botanist`), its scenes Apple's own
USD, read by phiecs (`crates/phicore/phiecs/src/usd`). This app is only
the screens, as BOT-anist's are.

| Screen | BOT-anist | Here |
|---|---|---|
| Loading | `StartScreenView`: "Loading assets…" | the greenhouse's part, while the scene is read away from the window |
| Creator | `RobotCustomizationView` beside `RobotView` | `part/pick.rs` (customiser) beside `part/robot.rs` (the robot, Start Planting) |
| Greenhouse | `ExplorationView` in a volume | `part/ground.rs`, the greenhouse in 3D from the sample's camera |
| Score | `OrnamentView` | `part/score.rs`: the ring, Replay, Exit Game |

Run it: `target/main-ui/quick/phibot.exe` (built with `cargo build -p
phibot --target-dir target/main-ui --profile quick`). `--show garden`
opens on the greenhouse.

Both 3D views are drawn from Apple's own meshes. phiecs composes the
`.usda` scenes; phiusd (`crates/phicore/phiusd`, on openusd) reads the
binary assets they name. The kit's raster (`appkit/pui/core/raster.rs`,
with `lens.rs`, `sun.rs`, `scan.rs`) draws the triangles on the
processor into a moving picture:
- **Edges:** it draws at twice the size and averages down (`scene.smooth`), so no edge is jagged.
- **Shading:** each corner carries its own normal, so curves look round, while edges sharper than `smooth.crease` stay sharp.
- **Light:** a sky and ground light from all round, a key light with a shine, and the robot's shadow on the floor (`game.tsv`).
- **Near plane:** a triangle reaching behind the camera is cut where it crosses, not dropped.

The robot is its three parts, standing where Apple's rig puts them. A
drag turns and tips either view; the wheel brings it nearer
(`scene.drag`, `scene.zoom`, `scene.nearest`, `scene.farthest`). A
press that does not drag, in the greenhouse, sends the robot there.

The greenhouse (403k triangles) is read while the loading screen
shows. It is drawn once for its view and size; while it is dragged it is
drawn coarser (`garden.moving`). Each frame only the patch the robot and
its shadow cover is drawn again, with as many of the robot's triangles
as its size on the picture shows (`garden.detail` to `garden.whole`).
The robot's colours come from `paint.tsv`, the greenhouse's from
`scenery.tsv`, until Apple's materials and textures are read. Keys are
held as the system repeats them (`garden.hold`), because the window
sends no key-up.

The customiser is data on the kit's layout, the same for every app. The
part's tabs and the groups they choose stand on one ground (`tabbed`).
The light on the chosen tab slides to the next (`PTabMove`,
`tab.step`). Each group stands on its own section ground under its
heading, and the room left in the pane is shared out between them
(`stack`, `section.side`, `section.top`, `section.radius`,
`section.space`). Options fill the width (`PBand::draw_full`,
`pill.high`), and colours are lit, ringed swatches (`PSwatch`,
`swatch.size`, `swatch.ring`). Each ground is lit against the one it
stands on, so the edges of headings and options line up.

The botanist opens in the Liquid Glass look (`data/look.tsv`, first
row). Every part stands on a glass pane (`part.glass`), the corners are
rounder, and the glass's edge is a thin shine in nearly its own colour.
Text stands a point off the glass (`text.lift`, `text.shade`). The
light, dark, glass and frost looks are still in the profile's Appearance.

Files: `data/` (the window's tables, words and theme), `src/app.rs` (the
app as appkit sees it), `src/part.rs` (the desk the parts share),
`src/part/*.rs` (the parts; `part/view.rs` frames the 3D cameras and turns them by hand, `part/scene.rs` draws the greenhouse), `src/test.rs` (the screens tested as a
person meets them). The user steps it is tested on are `FLOW.md`.
