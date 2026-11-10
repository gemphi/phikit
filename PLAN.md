# phikit — plan

phikit is the reality kit. This page says which crate is which part of it,
how the crates stand on each other, and the folders every crate keeps. Each
crate's own `PLAN.md` gives its folder tree and order of work, and its
`SPECS.md` says what each part must do, with the Apple source it answers.

Everything marked "suggested" is a proposal to accept or strike, not a
decision already taken.

## The crates

| Crate | Repo | What it is | Apple's counterpart | Stands on | Pages |
|---|---|---|---|---|---|
| `phicore/phiecs` | phikit | the world: words, relations, aspects, staged systems | RealityFoundation: Entity, Component, System, Scene | nothing | [architecture](crates/phicore/phiecs/ARCHITECTURE.md) · [plan](crates/phicore/phiecs/PLAN.md) · [specs](crates/phicore/phiecs/SPECS.md) |
| `phiray` | phiray | the renderer: a compute path tracer, and a rigid-body `sim` | Metal ray tracing, MetalFX, Metal Performance Shaders | nothing in phikit | in its own repo: [architecture](https://github.com/gemphi/phiray/blob/main/ARCHITECTURE.md) · [plan](https://github.com/gemphi/phiray/blob/main/PLAN.md) · [specs](https://github.com/gemphi/phiray/blob/main/SPECS.md) |
| `phiphx` | phikit | the asset core: `.blend`, USD, glTF, PLY in, one scene out | Model I/O; `Entity(named:)` loading | gpui (feature `bui`) | [architecture](crates/phiphx/ARCHITECTURE.md) · [plan](crates/phiphx/PLAN.md) · [specs](crates/phiphx/SPECS.md) |
| `phiace` | phikit | robotics: a robot as words in the world. Not implemented | RealityFoundation joints, pins, IK, in part | phiecs | [architecture](crates/phiace/ARCHITECTURE.md) · [plan](crates/phiace/PLAN.md) · [specs](crates/phiace/SPECS.md) |
| `phiplay` | phikit | the game: sports, teams, kits, the play | none read | phiecs | - |
| `phiber` | phiber | the app: views, Project Browser, the rendered viewport | none read | phiecs, phiphx, phiray, phiplay, philog, gpui | - |
| `appkit` | phikit | the common app layer: GPU, pui, glass | AppKit | - | - |
| `phipro` | phikit | a project bundle from any asset phiphx reads | none read | phiphx | - |
| `philog` | phikit | the session journal for the `log` facade | none read | nothing | - |
| `simkit` | phikit | vehicles, sensors, controllers after AirSim. Not a workspace member | none | phiecs, philog | - |
| `apple` | phikit | Rust transcriptions of Apple frameworks: shapes, nothing runs | - | - | - |
| `phient` | phinet | learning: the orchestrator and the learner | Core ML, Create ML, ML Compute | phical, phiecs (phinet's) | in phinet: [plan](https://github.com/gemphi/phinet/blob/main/docs/phient/plan.md) · [specs](https://github.com/gemphi/phinet/blob/main/docs/phient/specs.md) |

## How they stand on each other

```mermaid
graph TB
    phiber["phiber<br/>the app"]
    phiplay["phiplay"]
    phiace["phiace"]
    simkit["simkit"]
    phipro["phipro"]
    phiphx["phiphx"]
    phiray["phiray<br/>trace · sim"]
    phiecs["phiecs"]
    appkit["appkit: gpui"]
    philog["philog"]
    phiber --> phiplay
    phiber --> phiphx
    phiber --> phiray
    phiber --> phiecs
    phiber --> appkit
    phiber --> philog
    phiplay --> phiecs
    phiace --> phiecs
    simkit --> phiecs
    simkit --> philog
    phipro --> phiphx
    phiphx --> appkit
    phiace -.->|"sense: camera views, contact"| phiray
    phiace -.->|"bot: meshes, rigs"| phiphx
    phiber -.->|"bridges: the world drives the tracer and the sim"| phiecs
    classDef plan stroke-dasharray: 6 4;
    class phiace,simkit plan;
```

## A frame through the crates

```mermaid
graph LR
    f_file["a file<br/>.blend · usd · gltf"]
    f_phx["phiphx<br/>SceneModel · rigs · timeline"]
    f_world["phiecs World<br/>Ingest · PreCouple · Couple ·<br/>PostCouple · Readout"]
    f_sim["phiray sim<br/>bodies and contact"]
    f_trace["phiray trace<br/>repose · reinstance · sample"]
    f_view["phiber<br/>the viewport"]
    f_journal["journal<br/>every entity over time"]
    f_file --> f_phx -->|"spawn words"| f_world
    f_world <-->|"Couple"| f_sim
    f_world -->|"Readout: changed poses"| f_trace --> f_view
    f_world -->|"Readout: timed aspects"| f_journal
    f_phx -->|"today, directly"| f_trace
```

## The folders every crate keeps

| Path | Holds | Rule |
|---|---|---|
| `README.md` | what the crate is and where to start | short |
| `ARCHITECTURE.md` | diagrams: modules, flows, links to pages | diagrams, not prose; under 200 lines |
| `PLAN.md` | the folder tree, today and suggested, and the order of work | tables and flow diagrams |
| `SPECS.md` | what each part must do, with its Apple source and reference | one row a requirement |
| `TODO.md` | the checklist, by module | ticked as work lands |
| `docs/` | one page a subject, lowercase; longer studies in capitals (`APPLE.md`) | a page is diagrams first |
| `reference/` | the repositories that guide the crate, as shallow submodules, and a `README.md` index | never built, never in the workspace |
| `src/` | code | no file over 250 lines; a folder is `mod.rs` plus named parts; tables live in data files |

## Suggested moves between crates

| Suggested | From | To | Why | When |
|---|---|---|---|---|
| One phiecs, not two | the hand copy in `phicore/phiecs` | a feature gate in the phiecs repo that phikit takes with no phinet path | the two have already drifted: relation numbering, `relate` | before the journal, which persists relation ids |
| Physics out of the renderer | `phiray/src/sim` | its own crate | phiace and the phiecs bridge need bodies and contact, not a tracer | after the physics bridge; `sim` uses `bvh`, so the CPU tree moves with it or is shared |
| simkit joins or folds | `crates/simkit`, outside the workspace | a member, or its sensors and controllers into phiace | it is the nearest code to phiace and is not compiled today | when phiace's `sense` starts |
| `fairo` follows the others | `phiace/ext/fairo` | `phiace/reference/fairo` | one place a crate | when the folder is free |
| Old reference trees fold in | `gemphi/ext/phiber-refs`, `phinet/crates/phiano/refs` | each crate's `reference/` | one place a crate | when the sessions using those paths are done |
| The transcriptions stay an index | `crates/apple` | - | the SDK headers under `gemphi/phi/apple-source` are the specification; the Rust shapes run nothing | now |

## Order across crates

```mermaid
graph LR
    r1["phiray: measure, then the two inner-loop fixes"]
    r2["phiray: adaptive viewport, Preview warm start"]
    r3["phiray: vis buffer"]
    r4["phiray: splat kind, then media"]
    e1["phiecs: clock, events"]
    e2["phiecs: tree, space, pins"]
    e3["phiecs: stored, journal"]
    e4["phiplay: snapshot handoff"]
    e5["phiber: physics and render bridges"]
    a1["phiace: bot, actuate"]
    a2["phiace: sense, env"]
    x1["phiphx: character check, rig as asset"]
    r1 --> r2 --> r3 --> r4
    e1 --> e2 --> e3
    e1 --> e4
    e2 --> e5
    e2 --> x1
    e5 --> a1 --> a2
    x1 --> a1
    r3 --> a2
```
