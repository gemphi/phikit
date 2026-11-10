# phiecs — architecture

Solid boxes are in the crate today. Dashed boxes are to build; the number is
their place in the order of work, and `RF:n` is the line of Apple's
RealityFoundation interface the box answers. Three main folders, `entity`,
`comp` and `system`; every other name is a subfolder of one of them.

## The three folders and what they hold

```mermaid
graph LR
    subgraph entity["entity: what a thing is, how things are linked"]
        e_word["word<br/>Word · Lexicon"]
        e_rel["relation · plan<br/>Relation · Register · Edge · Graph"]
        e_world["world<br/>World: coin · bind · relate · step"]
        e_kind["4 kind · spawn<br/>Kind · spawn(kind) · Enabled<br/>RF:10560-10610"]
        e_tree["2 tree<br/>parent · children · loop refused<br/>RF:15395-15414"]
        e_query["7 query<br/>has · and · not · related<br/>RF:1517 · 13375"]
    end
    subgraph comp["comp: data bound to a thing"]
        c_aspect["aspect<br/>Aspect · Dense&lt;A&gt;"]
        c_res["resources<br/>one value per type"]
        c_names["4 aspect names · change ticks<br/>RF:6537 · 7242"]
        c_stored["3 stored<br/>Stored · Value · stable id<br/>RF:6568 · 12125"]
        c_space["2 space<br/>Pose · WorldMatrix<br/>RF:15094"]
        c_time["1, 3 time<br/>Clock · Track&lt;A&gt; · Journal<br/>RF:14752, then past Apple"]
        c_anim["9 anim<br/>Bind · Animated · Sample ·<br/>Playing · Handoff · ChannelSet<br/>RF:463-625 · 10419-10497 · 13799"]
        c_asset["10 asset<br/>Asset · Loaded<br/>RF:12124 · 13729"]
    end
    subgraph system["system: logic, in stage order"]
        s_stage["stage<br/>System · FnSystem · Stage::ORDER"]
        s_sched["1, 6 schedule<br/>step · after / before · when<br/>RF:14757 · 10308 · 14773"]
        s_layer["6 layer<br/>PerKind · Intake · Animate ·<br/>Simulate · Constrain · Readout · Plugin"]
        s_events["1 event<br/>typed double buffer<br/>RF:7086-7114 · 7221-7242"]
        s_space["2 space/propagate<br/>PostCouple"]
        s_time["3 time/record<br/>Readout: rows to the sinks"]
        s_anim["9 anim/advance · handoff<br/>PreCouple"]
    end
    e_world --> e_word
    e_world --> e_rel
    e_world --> c_aspect
    e_world --> c_res
    e_world --> s_stage
    c_aspect --> e_word
    e_rel --> e_word
    e_kind -.-> e_word
    e_tree -.-> e_word
    e_query -.-> c_aspect
    e_query -.-> e_rel
    c_names -.-> c_aspect
    c_stored -.-> c_names
    c_time -.-> c_stored
    c_anim -.-> c_names
    s_sched -.-> s_stage
    s_layer -.-> s_sched
    s_layer -.-> e_kind
    s_space -.-> c_space
    s_space -.-> e_tree
    s_time -.-> c_time
    s_anim -.-> c_anim
    s_anim -.-> c_time
    classDef plan stroke-dasharray: 6 4;
    class e_kind,e_tree,e_query,c_names,c_stored,c_space,c_time,c_anim,c_asset,s_sched,s_layer,s_events,s_space,s_time,s_anim plan;
```

## Systems are layered traits, chosen by entity kind

```mermaid
graph TB
    l0["layer 0: System<br/>name · stage · run(world, dt)<br/>in the code today"]
    l1["layer 1: order<br/>after · before · when, defaults on System"]
    l2["layer 2: PerKind<br/>kind() · each(world, word, dt)<br/>run once for every word of one kind"]
    subgraph l3["layer 3: roles, one a stage. A new role is a new trait"]
        r_in["Intake<br/>Ingest"]
        r_an["Animate<br/>PreCouple"]
        r_si["Simulate<br/>Couple"]
        r_co["Constrain<br/>PostCouple"]
        r_re["Readout<br/>Readout"]
    end
    l4["layer 4: Plugin<br/>build(world): declares its kinds, registers its aspects,<br/>adds its systems. One a crate or a unit"]
    l0 --> l1 --> l2 --> l3 --> l4
    p1["phiphx plugs in<br/>Intake for scene files · Constrain for rig"]
    p2["phiray plugs in<br/>Simulate for body · Readout for drawn things"]
    p3["phiace plugs in<br/>Animate for bot · Readout for sensor"]
    p4["phiplay plugs in<br/>Animate for player"]
    l4 --- p1
    l4 --- p2
    l4 --- p3
    l4 --- p4
    classDef plan stroke-dasharray: 6 4;
    class l1,l2,r_in,r_an,r_si,r_co,r_re,l4,p1,p2,p3,p4 plan;
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
        o_word["entity: word · kind · spawn · Enabled"]
        o_tree["entity: tree"]
        o_aspect["comp: aspect, names, change ticks"]
        o_query["entity: query, with the relation join"]
        o_system["system: stage · schedule · layers"]
        o_events["system: event"]
        o_space["comp: space"]
        o_pin["a pin is a word: a named child of a bone"]
        o_anim["comp and system: anim, for any field"]
        o_skel["bones are words in the tree.<br/>The rig itself is phiphx's asset"]
        o_bridge["Simulate role, plugged in over phiray sim"]
        o_stored["comp: stored · time"]
        o_asset["comp: asset"]
        o_clock["comp: time/clock"]
    end
    subgraph only["ours alone"]
        x_rel["relation: weighted edges, plan, reach"]
        x_stage["five fixed stages, roles by stage"]
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
    a_bind --> o_anim
    a_skel --> o_skel
    a_phys --> o_bridge
    a_save --> o_stored
    a_sync --> o_stored
    a_ref --> o_asset
    a_dt --> o_clock
    classDef plan stroke-dasharray: 6 4;
    class o_tree,o_query,o_events,o_space,o_pin,o_anim,o_skel,o_bridge,o_stored,o_asset,o_clock plan;
```

## Pages, one subject each

- [step](docs/step.md): one tick, the five stages, ordering inside a stage
- [space](docs/space.md): tree, pose, world matrix, pins as words
- [time](docs/time.md): clock, journal, saving, a Blender frame as a tick
- [anim](docs/anim.md): one animation path for any field of any aspect
- [character](docs/character.md): a character on that path, and what keeps it whole
- [bridge](docs/bridge.md): what stands on the world and what it hands to
- [connect](docs/connect.md): the database, the session journal, data files

Folders and order of work: [PLAN.md](PLAN.md) ·
What each part must do: [SPECS.md](SPECS.md) ·
The contract row by row: [docs/APPLE.md](docs/APPLE.md) ·
Where the interfaces are on disk: [reference/README.md](reference/README.md) ·
What is left to do: [TODO.md](TODO.md)
