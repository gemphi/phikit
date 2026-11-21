# tick

Built so far: `drive` (Couple) and `hinge` (PostCouple), then phiecs `propagate`; the rest is to build. The three loops on the left are read from the
references; that they share one order is our reading, not a claim any of
them makes.

```mermaid
graph TB
    subgraph refs["one tick in three references"]
        r1["project_superdex<br/>controller, actuator effort, set forces,<br/>scene.step, sensor signal<br/>superdex_robotics/examples/advanced/example_custom_components.py:349-369"]
        r2["habitat-sim<br/>agent.act, step_world, sensor observations<br/>src_python/habitat_sim/simulator.py:605-632"]
        r3["fairo polymetis<br/>fill RobotState, ControlUpdate, clamp, return torques<br/>polymetis/polymetis/src/clients/franka_panda_client/franka_panda_client.cpp:129-159"]
    end
    subgraph stages["phiecs Stage::ORDER"]
        s1["Ingest<br/>an action or a target arrives:<br/>from env, from link, from a player"]
        s2["PreCouple<br/>actuate: state + target to efforts,<br/>clamped to the limits"]
        s3["Couple<br/>physics: efforts in, joint state,<br/>link transforms and contacts out"]
        s4["PostCouple<br/>derived state: Tree propagate,<br/>sensor poses follow their links"]
        s5["Readout<br/>sense: observations, measures,<br/>camera sensors rendered by phiray"]
    end
    r1 --> s1
    r2 --> s1
    r3 --> s1
    s1 --> s2 --> s3 --> s4 --> s5
```

```mermaid
graph TB
    key["two rates: control and simulation"]
    e_step["env.step(action)"]
    e_clip["clip the action to its space"]
    e_loop["substeps = simulation rate / control rate"]
    e_apply["apply the action"]
    e_world["World::step(dt)"]
    e_obs["observation"]
    e_rew["reward terms"]
    e_stop["stop criteria"]
    e_step --> e_clip --> e_loop --> e_apply --> e_world
    e_world -->|"again, substeps times"| e_apply
    e_world --> e_obs --> e_rew --> e_stop
    src["project_superdex: the ratio of two configured rates<br/>superdex_lab/superdex/lab/gym/envs/mochi_env.py:333, 459-524, 829-833<br/>habitat-lab: control 120 Hz, 4 physics steps an action<br/>habitat-lab/habitat/config/default_structured_configs.py:1789-1834<br/>fairo polymetis: 1000 Hz on the arm, 240 Hz in simulation<br/>polymetis/polymetis/conf/robot_client"]
    e_loop --- src
```

```mermaid
graph TB
    key["the wire to a real robot, after fairo polymetis/polymetis/protos/polymetis.proto"]
    l_user["user side<br/>send a policy · update its parameters · end it ·<br/>read state, now or as a stream"]
    l_server["server<br/>holds the running controller,<br/>falls back to the default one<br/>when updates stop for a second"]
    l_robot["robot side, the real-time loop<br/>state up, torque down, every tick"]
    l_safe["safety on the robot side, last:<br/>reflex torques, then the clamp"]
    l_log["a ring of past states"]
    l_user -->|"SetController · UpdateController"| l_server
    l_robot -->|"ControlUpdate(RobotState)"| l_server
    l_server -->|"TorqueCommand"| l_robot
    l_robot --> l_safe
    l_server --> l_log
    ours["in phikit: link is an Ingest system that writes the state<br/>and a Readout system that sends the command.<br/>The journal is the ring of past states"]
    l_log --- ours
```
