# phiace — architecture

Nothing in phiace is implemented: `src/lib.rs` is a doc comment. Every
phiace box below is to build, named with the reference it is read from
(`reference/<repo>`; `fairo` is still under `ext/`). Solid boxes are the
crates it will stand on.

## The modules and what they stand on

```mermaid
graph TB
    subgraph under["in phikit today"]
        u_ecs["phiecs<br/>World · five stages"]
        u_sim["phiray sim<br/>rigid bodies: broad ·<br/>narrow · contact · world"]
        u_trace["phiray trace<br/>Tracer: what a camera sees"]
        u_phx["phiphx<br/>meshes · armatures ·<br/>blend/rig/ik"]
        u_simkit["simkit<br/>sensors · controllers ·<br/>vehicles · mavlink<br/>not a workspace member"]
    end
    subgraph phiace["phiace, to build"]
        m_bot["bot<br/>links with a parent, joints,<br/>limits, sensors and actuators<br/>on a link, bots joined to bots<br/>project_superdex · habitat-sim"]
        m_act["actuate<br/>a controller is pure:<br/>state + target to efforts.<br/>a default to fall back on,<br/>a clamp last<br/>project_superdex · fairo polymetis"]
        m_sense["sense<br/>a sensor is a spec and a pose<br/>on a link: camera · depth ·<br/>joint state · contact<br/>habitat-sim"]
        m_env["env<br/>action in, substeps,<br/>observation · reward · stop out<br/>project_superdex · habitat-lab"]
        m_task["task<br/>episodes, measures, success<br/>habitat-lab"]
        m_link["link<br/>the wire to a real robot:<br/>state up, torque down, a policy<br/>swapped under a running loop<br/>fairo polymetis"]
        m_solve["solve<br/>cost functions over variables,<br/>Gauss-Newton, a linear solver<br/>theseus · momentum"]
        m_retarget["retarget<br/>human keypoints to robot joints<br/>GeoRT"]
    end
    subgraph above["above phiace, in phinet"]
        a_policy["policy<br/>observation to action"]
        a_plan["planner over a world graph<br/>partnr-planner · home-robot"]
        a_model["world model:<br/>predict, score, pick<br/>vjepa2 · nwm"]
    end
    m_bot --> u_ecs
    m_bot --> u_phx
    m_act --> m_bot
    m_act --> m_solve
    m_sense --> m_bot
    m_sense --> u_trace
    m_sense --> u_sim
    m_env --> m_act
    m_env --> m_sense
    m_env --> u_ecs
    m_task --> m_env
    m_link --> m_act
    m_solve --> u_phx
    m_retarget --> m_solve
    m_bot --> u_sim
    u_simkit -.->|"nearest existing code"| m_sense
    u_simkit -.-> m_act
    a_policy --> m_env
    a_plan --> a_policy
    a_model --> a_policy
    classDef plan stroke-dasharray: 6 4;
    class m_bot,m_act,m_sense,m_env,m_task,m_link,m_solve,m_retarget plan;
```

## Pages, one subject each

- [tick](docs/tick.md): one control step on the five stages, and the two rates
- [layers](docs/layers.md): the robotics stack as the references divide it, and what crosses each boundary

Folders and order of work: [PLAN.md](PLAN.md) ·
What each part must do: [SPECS.md](SPECS.md) ·
The references: [reference/README.md](reference/README.md) ·
The world it stands on: [phiecs](../phicore/phiecs/ARCHITECTURE.md) ·
What is left to do: [TODO.md](TODO.md)
