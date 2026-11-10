# phiecs — architecture

Solid boxes are in the crate today. Dashed boxes are to build; the number is
their place in the order of work, and `RF:n` is the line of Apple's
RealityFoundation interface the box answers.

## The modules and what they use

```mermaid
graph LR
    subgraph today["in the crate today"]
        m_word["word<br/>Word · Lexicon"]
        m_relation["relation<br/>Relation · Register · Edge · Graph"]
        m_aspect["aspect<br/>Aspect · Dense&lt;A&gt;"]
        m_resources["resources<br/>one value per type"]
        m_system["system<br/>System · FnSystem · Stage::ORDER"]
        m_world["world<br/>World: coin · bind · relate · plan · step"]
    end
    subgraph build["to build"]
        m_clock["1 clock<br/>Clock: tick · time · dt · fps<br/>RF:14752"]
        m_events["1 events<br/>typed double buffer, read for one step<br/>RF:7086-7114 · 7221-7242"]
        m_schedule["1, 6 world/schedule<br/>step · after / before · when<br/>RF:14757 · 10308 · 14773"]
        m_tree["2 tree<br/>parent · children · loop refused<br/>RF:15395-15414"]
        m_space["2 space<br/>Pose · WorldMatrix · propagate · Pin · Attached<br/>RF:15094 · 9088 · 13669"]
        m_stored["3 stored<br/>Stored: KIND · fields · Value<br/>RF:6568 · 12125 · 15151"]
        m_journal["3 journal<br/>Track&lt;A&gt; · at(word, tick) · memory impl<br/>past Apple, which keeps no history"]
        m_names["4 aspect names and change ticks<br/>RF:6537 · 7242"]
        m_query["7 query<br/>has · and · not · related<br/>RF:1517 · 13375"]
        m_rig["9 rig<br/>Skeleton · JointPose · Reach<br/>RF:8030 · 5182 · 4413"]
        m_anim["9 anim<br/>Clip · Playing · Handoff · Bind · Animated<br/>RF:10419-10497 · 13799 · 463-625"]
        m_asset["10 asset<br/>Asset · Loaded<br/>RF:12124 · 13729"]
    end
    m_aspect --> m_word
    m_relation --> m_word
    m_system --> m_world
    m_world --> m_aspect
    m_world --> m_relation
    m_world --> m_resources
    m_world --> m_system
    m_world --> m_word
    m_world -.-> m_schedule
    m_schedule -.-> m_clock
    m_schedule -.-> m_events
    m_schedule -.-> m_system
    m_clock -.-> m_resources
    m_events -.-> m_resources
    m_tree -.-> m_word
    m_space -.-> m_tree
    m_space -.-> m_aspect
    m_names -.-> m_aspect
    m_stored -.-> m_names
    m_journal -.-> m_stored
    m_journal -.-> m_clock
    m_query -.-> m_aspect
    m_query -.-> m_relation
    m_query -.-> m_tree
    m_rig -.-> m_space
    m_anim -.-> m_rig
    m_anim -.-> m_clock
    m_anim -.-> m_journal
    m_asset -.-> m_aspect
    classDef plan stroke-dasharray: 6 4;
    class m_clock,m_events,m_schedule,m_tree,m_space,m_stored,m_journal,m_names,m_query,m_rig,m_anim,m_asset plan;
```

## Apple's contract, and where each part lands

```mermaid
graph LR
    subgraph apple["RealityFoundation"]
        a_entity["Entity<br/>id · name · isEnabled · clone"]
        a_tree["parent · children · setParent"]
        a_comp["Component · TransientComponent"]
        a_query["EntityQuery · QueryPredicate"]
        a_system["System · SystemDependency"]
        a_events["SceneEvents · ComponentEvents"]
        a_xf["Transform · convert · look"]
        a_pin["GeometricPin · AttachedTransformComponent"]
        a_play["AnimationPlaybackController · AnimationHandoffType"]
        a_bind["BindTarget · BindableValue"]
        a_skel["Skeleton · SkeletalPosesComponent · IKComponent"]
        a_phys["PhysicsBodyComponent · CollisionEvents"]
        a_save["registerComponent · Entity.write"]
        a_sync["SynchronizationComponent"]
        a_ref["ReferenceComponent · LowLevelMesh"]
        a_dt["SceneUpdateContext.deltaTime"]
    end
    subgraph ours["phiecs"]
        o_word["word + World::spawn · Enabled"]
        o_tree["tree"]
        o_aspect["aspect + names · change ticks · Transient"]
        o_query["query, with the relation join"]
        o_system["system + schedule"]
        o_events["events"]
        o_space["space: Pose · WorldMatrix"]
        o_pin["space: Pin · Attached"]
        o_anim["anim: Playing · Handoff"]
        o_bind["anim: Bind · Animated"]
        o_rig["rig"]
        o_bridge["Body · Collider · Velocity · Contact<br/>bridge over phiray sim::World"]
        o_stored["stored + journal"]
        o_owner["stored: stable id · Owner"]
        o_asset["asset, mesh replace = Tracer::repose"]
        o_clock["clock"]
    end
    subgraph only["ours alone"]
        x_rel["relation: weighted edges, plan, reach"]
        x_stage["five fixed stages"]
        x_hist["journal: every entity over time"]
    end
    a_entity --> o_word
    a_tree --> o_tree
    a_comp --> o_aspect
    a_query --> o_query
    a_system --> o_system
    a_events --> o_events
    a_xf --> o_space
    a_pin --> o_pin
    a_play --> o_anim
    a_bind --> o_bind
    a_skel --> o_rig
    a_phys --> o_bridge
    a_save --> o_stored
    a_sync --> o_owner
    a_ref --> o_asset
    a_dt --> o_clock
    classDef plan stroke-dasharray: 6 4;
    class o_tree,o_query,o_events,o_space,o_pin,o_anim,o_bind,o_rig,o_bridge,o_stored,o_owner,o_asset,o_clock plan;
```

## Pages, one subject each

- [step](docs/step.md): one tick, the five stages, ordering inside a stage
- [space](docs/space.md): tree, pose, world matrix, pins
- [time](docs/time.md): clock, journal, saving, a Blender frame as a tick
- [character](docs/character.md): skeleton, joint pose, playing, handoff, reach
- [bridges](docs/bridges.md): what stands on the world and what it hands to

Folders and order of work: [PLAN.md](PLAN.md) ·
What each part must do: [SPECS.md](SPECS.md) ·
The contract row by row: [docs/APPLE.md](docs/APPLE.md) ·
Where the interfaces are on disk: [reference/README.md](reference/README.md) ·
What is left to do: [TODO.md](TODO.md)
