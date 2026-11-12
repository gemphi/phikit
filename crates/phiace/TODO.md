# phiace — to do

The first slice of `bot` and `actuate` is built (a two-joint arm, `tests/arm.rs`). The diagrams are in
[ARCHITECTURE.md](ARCHITECTURE.md); each module names the reference it is
read from. phiace stands on phiecs, so the first three items of
[phiecs's list](../phicore/phiecs/TODO.md) come first: clock and events,
tree and space, the journal.

## bot

- [ ] a bot description as a data file: links with a parent, joints, limits (the `Bot` type exists; built in code, not read from a file)
- [x] spawn it into a `World`: a word a link in the `Tree`, a `Pose` each (joint angles are `Angle` aspects until `ChannelSet` exists, phiecs order 9)
- [ ] a bot made of bots, joined at pins (named child words; `World::pin` exists in phiecs)
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
