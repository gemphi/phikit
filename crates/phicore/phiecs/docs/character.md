# character

Every box is to build unless it names a file. The split between asset and
state is momentum's (`phiphx/reference/momentum`): its `Character` is shared
and const, a pose is one vector a frame, and everything else is recomputed.

```mermaid
graph LR
    subgraph asset["shared and immutable: one a kind of character"]
        a_skel["Skeleton<br/>joints, parents before children<br/>name · parent · pre-rotation · offset · a name index<br/>momentum character/skeleton.h:22-25 · RF:8030"]
        a_bind["inverse bind<br/>derived: the inverse of the rest pose, never authored<br/>momentum character/character.cpp:783-792"]
        a_skin["skin weights<br/>8 influences a vertex, packed at the front, summing to one<br/>momentum character/skin_weights.h:19-43"]
        a_pins["Pin table<br/>a parent joint and an offset<br/>momentum character/locator.h:19-26 · RF:9088"]
        a_limits["joint limits"]
    end
    subgraph word["on the word"]
        w_rig["Rig { asset }"]
        w_id["Identity<br/>constant for one individual: bone lengths, scales, shape<br/>momentum character/types.h:156-166"]
        w_pose["JointPose<br/>the one vector a tick<br/>RF:5182"]
        w_play["Playing · Reach"]
    end
    subgraph cache["derived each tick, never stored, never journalled"]
        c_world["joint world matrices<br/>one forward pass, parents first<br/>momentum character/skeleton_state.cpp:104-118"]
        c_skinm["skinning matrices = world x inverse bind<br/>momentum character/linear_skinning.cpp:23-37"]
    end
    a_skel --> a_bind
    w_rig --> a_skel
    w_id --> c_world
    w_pose --> c_world
    a_skel --> c_world
    c_world --> c_skinm
    a_bind --> c_skinm
```

```mermaid
graph TB
    subgraph pre["PreCouple"]
        p_play["Playing<br/>clip · time · speed · weight"]
        p_sample["sample the clip at Clock.time<br/>phiphx blend/anim"]
        p_hand["Handoff blends from its source"]
        p_write["JointPose written"]
    end
    subgraph post["PostCouple"]
        q_reach["Reach { joint, target, weight }"]
        q_ik["IK solve<br/>phiphx blend/rig/ik"]
        q_fk["forward pass: joint world matrices"]
        q_pin["Pin and Attached resolve"]
    end
    subgraph read["Readout"]
        r_skin["skinning matrices"]
        r_gpu["phiray kernels::Skin"]
        r_repose["Tracer::repose"]
        r_row["JointPose to the journal"]
    end
    p_play --> p_sample --> p_hand --> p_write --> q_reach --> q_ik --> q_fk --> q_pin --> r_skin --> r_gpu --> r_repose
    p_write --> r_row
```

```mermaid
graph TB
    subgraph today["phiplay today, play/world.rs"]
        t_act["act(): keeps the moment the old move was cut at,<br/>in the one was slot, :140-149"]
        t_rec["record(): overwrites it with t = 1.0,<br/>the old move's end, :216"]
        t_pop["the blend starts from a pose the body was not in: a pop"]
        t_third["a third move inside the blend drops the blend"]
        t_act --> t_rec --> t_pop
        t_act --> t_third
    end
    subgraph handoff["Handoff, RF:13799-13812"]
        h_snap["Snapshot<br/>source = the JointPose last written<br/>Apple: snapshotAndReplace"]
        h_rep["Replace<br/>cut, no blend"]
        h_comp["Compose<br/>layer on top by weight"]
        h_stop["Stop"]
        h_chain["a new move during a blend snapshots the blended pose:<br/>no slot to run out of"]
        h_math["a joint: translation and scale by lerp, rotation by slerp<br/>momentum math/transform.cpp:156-161<br/>never linear in Euler angles"]
        h_snap --> h_chain --> h_math
    end
    t_pop -.->|"replaced by"| h_snap
    classDef plan stroke-dasharray: 6 4;
    class h_snap,h_rep,h_comp,h_stop,h_chain,h_math plan;
```

```mermaid
graph LR
    key["what must agree for a character not to fracture,<br/>checked once when the asset is built"]
    k_order["parents before children<br/>momentum character/skeleton.cpp:16-22"]
    k_size["pose length = what the skeleton expects"]
    k_bind["inverse bind derived from this skeleton's rest pose"]
    k_index["every influence names a joint that exists"]
    k_sum["weights sum to one: momentum's FBX reader renormalises,<br/>its glTF reader copies them raw<br/>io/fbx/openfbx_loader.cpp:926-949 · io/gltf/gltf_mesh_io.cpp:451-466"]
    k_name["motion mapped to joints by name, never by index<br/>momentum io/motion/mmo_io.cpp:174-219"]
    k_refuse["Skeleton::check at Ingest: a failing asset is refused,<br/>Loaded stays NotLoaded, an event says why"]
    k_order --> k_refuse
    k_size --> k_refuse
    k_bind --> k_refuse
    k_index --> k_refuse
    k_sum --> k_refuse
    k_name --> k_refuse
```
