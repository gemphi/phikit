# phiace — to do

Nothing is implemented. The diagrams are in
[ARCHITECTURE.md](ARCHITECTURE.md); each module names the reference it is
read from. phiace stands on phiecs, so the first three items of
[phiecs's list](../phicore/phiecs/TODO.md) come first: clock and events,
tree and space, the journal.

## botanist (2026-11-23)

Apple's BOT-anist sample played out on phiecs: `src/bot/botanist*`, its
tables in `data/botanist`, its scenes Apple's own USD where `game.tsv` says.

- [x] the robot from its parts' USD scenes, its look from the tables (shapes, materials, colours, lights, eyes), reset
- [x] the animation states and which may follow which, a clip that plays once going on to the next
- [x] walking with the keys, facing where it goes, the walk ending after a key is let go
- [x] the greenhouse from `volume.usda`, its planters by their plant component, a planter the robot comes to propagated, celebrating at three, replay, exit
- [x] a test that plays a session through (`tests/botanist.rs`)
- [ ] the screens, as an app on appkit: loading, the creator (customiser and the robot), the greenhouse with its score, Replay and Exit
- [ ] the clips' lengths and poses from the binary animation files (a mock length now, `clip.length`); waving when the viewer moves
- [ ] the head and backpack pinned to the body's joints (JointPinSystem), once the body's skeleton is read
- [ ] the robot drawn from its meshes (`.usdz`) by phiray, as the stage gives them, live
- [ ] `touch.reach` is a mock: from the character controller's capsule once the robot's size is read

## bot

- [ ] a bot description as a data file: links with a parent, joints, limits
- [ ] spawn it into a `World`: a word a link in the `Tree`, a `Pose` each, joint angles in `ChannelSet`
- [ ] a bot made of bots, joined at pins (named child words)
- [ ] URDF in

## actuate

- [ ] `Controller`: state and target in, efforts out, no side effects
- [ ] a joint PD controller first
- [ ] a default controller to fall back on, and the clamp last

## sense

- [ ] `Sensor { spec }` on a word under a pin of a link
- [ ] joint state and contact from the physics bridge
- [ ] camera and depth from a `phiray::Tracer` view at Readout

## env, task

- [ ] `Env::step(action)`: substeps of `World::step`, then observation, reward terms, stop
- [ ] control rate and simulation rate, both in the config
- [ ] reset from a journal tick
- [ ] `Measure` and an episode list

## physics under it

- [ ] joints between bodies in `phiray::sim` (it holds rigid bodies and contact today; `sim/` was not read line by line)

## link, solve, retarget

- [ ] `link`: state up and command down over a wire, a policy swapped while it runs
- [ ] `solve`: one objective of cost functions; phiphx's `blend/rig/ik` is the solver that exists
- [ ] `retarget`: human keypoints to joint targets

## housekeeping

- [ ] `git mv crates/phiace/ext/fairo crates/phiace/reference/fairo` once the folder is free
- [ ] decide whether simkit joins the workspace or its sensors and controllers move here
