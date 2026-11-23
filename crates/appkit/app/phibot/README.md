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

The customiser stands in the bar's second column, as Phiogy's rooms
do (`app.tsv` `side`, a `side` row in `menu.tsv`, `activity.width`). It
is the kit's form (`appkit/pui/form.rs`), with no drawing of its own.
`data/form.tsv` names its blocks: tools, the part's tabs over the
chosen part's bands and swatches, and the robot's lights under no tabs.
`src/part/choose.rs` only answers what each list holds and what each
path keeps, from phiace's tables. The kit lays it out the same as for
any app: the tabs off any ground over one ground of their children
(`tabbed`), each group on its own (`stack`), and a sliding tab light.

The botanist opens in the dark look (`data/look.tsv`, first row). In
the Liquid Glass looks every part stands on a glass pane
(`part.glass`), the corners are rounder, and the glass's edge is a thin
shine in nearly its own colour. Text stands a point off the glass
(`text.lift`, `text.shade`). The other looks are in the profile's
Appearance.

Files: `data/` (the window's tables, words and theme), `src/app.rs` (the
app as appkit sees it), `src/part.rs` (the desk the parts share),
`src/part/*.rs` (the parts; `part/choose.rs` is the customiser's source, `part/view.rs` frames the 3D cameras and turns them by hand, `part/scene.rs` draws the greenhouse), `src/test.rs` (the screens tested as a
person meets them). The user steps it is tested on are `FLOW.md`.
