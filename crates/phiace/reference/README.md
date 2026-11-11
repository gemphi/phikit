# reference

Trees phiace reads and ports from. None is built. Meta (facebookresearch)
robotics and embodied-AI repositories, as shallow git submodules:

- `project_superdex/`: a physics engine, robotics authoring tools and an RL interface in one simulation platform
- `habitat-lab/`: a library to train embodied AI agents across tasks and environments
- `habitat-sim/`: a 3D simulator for embodied AI research (archived upstream)
- `home-robot/`: mobile manipulation research tools
- `partnr-planner/`: the PARTNR benchmark: planning models for human-robot tasks
- `vjepa2/`: V-JEPA 2, a self-supervised video world model
- `nwm/`: Navigation World Models
- `eai-vc/`: visual foundation models for embodied AI
- `theseus/`: differentiable nonlinear optimisation
- `digit360/`: the Digit 360 tactile sensor platform
- `neuralfeels/`: visuo-tactile perception for in-hand manipulation
- `GeoRT/`: geometric hand retargeting
- `BenchMARL/`: multi-agent reinforcement learning benchmarks
- `fairo/`: a modular embodied agent architecture (holds Polymetis). **Still
  at `../ext/fairo`**: a process held its folder open when the others moved
  on 2026-11-10. It follows with
  `git mv crates/phiace/ext/fairo crates/phiace/reference/fairo`.

Renderers sit with `phiray` in `crates/phiray/reference`; characters, rigs
and geometry with `phiphx` in `crates/phiphx/reference`.
