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
| Greenhouse | `ExplorationView` in a volume | `part/ground.rs`, the greenhouse from above |
| Score | `OrnamentView` | `part/score.rs`: the ring, Replay, Exit Game |

Run it: `target/main-ui/quick/phibot.exe` (built with `cargo build -p
phibot --target-dir target/main-ui --profile quick`). `--show garden`
opens on the greenhouse.

What is a mock until the window draws meshes: the robot is drawn flat
from its look (its parts' shapes, materials, colours, lights and eyes),
and the greenhouse from above (its floor, its planters where Apple's
scene puts them, the robot facing where it goes). Keys are held as the
system repeats them (`garden.hold`), as the window sends no key going up.

Files: `data/` (the window's tables, words and theme), `src/app.rs` (the
app as appkit sees it), `src/part.rs` (the desk the parts share),
`src/part/*.rs` (the parts), `src/test.rs` (the screens tested as a
person meets them). The user steps it is tested on are `FLOW.md`.
