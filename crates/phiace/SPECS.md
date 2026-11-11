# phiace — specs

What each part must do. One row is one requirement, with the Apple source
it answers, where there is one, and the reference it was read from.
Nothing is met today: the crate is a doc comment.

## Sources

| Short | What | Where |
|---|---|---|
| **RF** | RealityFoundation: joints, pins, IK, physics bodies | `gemphi/phi/crates/kitpro/src/examples/headers/MacOSX-20260419T054331Z-3-001/MacOSX/RealityFoundation/SwiftInterfaces/arm64e-apple-macos.swiftinterface` |
| **SDX** | project_superdex | `reference/project_superdex/` |
| **HS** | habitat-sim | `reference/habitat-sim/` |
| **HL** | habitat-lab | `reference/habitat-lab/habitat-lab/habitat/` |
| **PM** | fairo's polymetis | `ext/fairo/polymetis/polymetis/` (to be `reference/fairo/`) |
| **TH** | theseus | `reference/theseus/theseus/` |
| index | every reference, one line each | [reference/README.md](reference/README.md) |
| index | the world phiace stands on | [../phicore/phiecs/SPECS.md](../phicore/phiecs/SPECS.md) |

Apple has no robotics framework among the interfaces read. Its joints,
pins and IK are the nearest parts, and are cited where they apply.

## bot

| Part | Must | Apple | Reference |
|---|---|---|---|
| description | be data in a file: a default pose, links, joints | - | SDX `assets/bots/fun/example_bot_2dof/example_bot_2dof.superdex_bot` |
| link | carry a shape, a render model, and its own sensors and actuators | - | SDX `superdex_robotics/superdex_robotics.mochi_gen:89-93` |
| joint | be one of free, prismatic, revolute, spherical, fixed, with limits and an effort limit | physics joints with `pin0`, `pin1` RF:2230 | SDX `superdex_physics/libraries/mochi/mochi_core/include/mochi_core/articulated_body/articulated_body_params.h:75-84`; HS `src/esp/physics/ArticulatedObject.h:40-45` |
| tree | give each link one parent, parents first | `Skeleton` RF:8030-8045 | SDX `superdex_physics/libraries/mochi/mochi_physics/include/mochi_physics/cpp_api/mochi_structs.h:206-280` |
| joined bots | build a bot from a base and attachments, a hand on an arm | `GeometricPin` RF:9088 | SDX `superdex_robotics.mochi_gen:219-255` |
| URDF | read URDF into the same description | - | SDX `superdex_robotics.mochi_gen:278-294`; HS `src/esp/physics/PhysicsManager.h:541` |

## actuate

| Part | Must | Apple | Reference |
|---|---|---|---|
| controller | be pure: the same state and target give the same efforts, with no side effect | - | SDX `superdex_robotics.mochi_gen:717, 1090` |
| PD | hold a joint to a target by position and velocity gains, with a force cap | - | HS `src/esp/physics/ArticulatedObject.h:60-115` |
| fallback | switch to the default controller when no update arrives for a set time | - | PM `src/polymetis_server.cpp:150-155` |
| clamp | apply limits last, after every controller | - | PM `src/clients/franka_panda_client/franka_panda_client.cpp:136-142, 302-333` |
| reach | move an end to a target through IK | `IKComponent` RF:8238, constraints RF:4413-4498 | phiphx `blend/rig/ik` |

## sense

| Part | Must | Apple | Reference |
|---|---|---|---|
| spec | name a sensor by id, type, pose and noise; a visual one adds resolution, near and far | - | HS `src/esp/sensor/Sensor.h:57-71`, `VisualSensor.h:36-56` |
| mount | sit on a link as a leaf and follow it | `AttachedTransformComponent` RF:13669 | HS `src/esp/sensor/SensorFactory.cpp:18-22` |
| observation | be a table from sensor id to buffer, read at Readout | - | HS `src_python/habitat_sim/simulator.py:339-363` |
| joint state | give positions, velocities and efforts | - | PM `protos/polymetis.proto:120-136` |
| contact | give points and forces between bodies | `CollisionEvents` RF:7163-7180 | SDX `mochi_physics/include/mochi_physics/cpp_api/mochi_structs.h:518-544` |
| camera | render colour, depth and normal from the sensor's pose | - | HS `src_python/habitat_sim/sensors/sensor_wrapper.py:174-219` |

## env and task

| Part | Must | Apple | Reference |
|---|---|---|---|
| step | clip the action, run substeps, then give observation, reward terms and stop | - | SDX `superdex_lab/superdex/lab/gym/envs/mochi_env.py:459-524` |
| two rates | hold a control rate and a simulation rate; substeps is their ratio | - | SDX `mochi_env.py:333, 829-833`; HL `config/default_structured_configs.py:1789-1834` |
| reset | return to a saved state | - | SDX `mochi_env.py:788-790` |
| task | own the measures, the task sensors and the actions, each registered by name | - | HL `core/embodied_task.py:207-266` |
| measure | be updated after every step and read by name for reward and success | - | HL `core/embodied_task.py:83-126`; `core/environments.py:42-85` |

## solve, retarget, link

| Part | Must | Apple | Reference |
|---|---|---|---|
| objective | be a sum of cost functions over named variables, each with error and Jacobian | - | TH `core/cost_function.py:64-100`, `core/objective.py:42` |
| solver | linearise, solve for a step, retract, and repeat; the linear solver is a part that can be swapped | - | TH `optimizer/nonlinear/nonlinear_least_squares.py:57-137`, `optimizer/linear/linear_solver.py:15-37` |
| retarget | map human fingertip points to robot joint angles within limits | - | GeoRT `geort/model.py:84-93`, `geort/export.py:31-36` |
| wire | carry robot state up and a command down every tick, and let a policy be replaced while the loop runs | - | PM `protos/polymetis.proto:11-56`, `src/polymetis_server.cpp:145-292` |
| log | keep a ring of past states | - | PM `src/polymetis_server.cpp:217-222` |

## Not taken

Network policies and their training, planners over language, and world
models: those sit above phiace, in phinet. Bullet, Mochi and libfranka as
dependencies: the references are read, not linked.
