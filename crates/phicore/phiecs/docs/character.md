# character

A character is not a special case here: its bones are words in the tree and
it moves by the path in [anim](anim.md). Every box is to build unless it
names a file. The split between what is shared and what is a frame's state
is momentum's (`phiphx/reference/momentum`): its `Character` is shared and
const, a pose is one vector a frame, everything else is recomputed.

```mermaid
graph LR
    subgraph asset["phiphx: the rig, shared and immutable"]
        a_rest["bones, parents before children<br/>name · parent · rest transform<br/>blend/rig/mod.rs:78-125 · momentum character/skeleton.h:22-25"]
        a_bind["inverse rest matrices<br/>derived from the rest pose, never authored<br/>momentum character/character.cpp:783-792"]
        a_skin["weights a vertex<br/>blend/modifier/armature.rs"]
    end
    subgraph words["phiecs: words"]
        w_root["the character<br/>Asset { rig } · Playing"]
        w_bone["a word a bone, in the Tree<br/>Pose: the frame's state"]
        w_pin["a pin: a named child of a bone<br/>Pose: the offset"]
        w_part["a part: a child of a pin"]
    end
    subgraph derived["derived each tick, never stored"]
        d_world["WorldMatrix of every bone, pin and part<br/>system/space/propagate"]
        d_skin["skinning matrices = bone world x inverse rest<br/>gathered at Readout in the rig's bone order"]
    end
    a_rest -->|"spawned once"| w_bone
    w_root --> w_bone --> w_pin --> w_part
    w_bone --> d_world --> d_skin
    a_bind --> d_skin
    a_skin --> d_skin
```

```mermaid
graph TB
    subgraph pre["PreCouple: Animate"]
        p_play["Playing on the character"]
        p_write["bone Poses written through binds"]
    end
    subgraph post["PostCouple: Constrain, then propagate"]
        q_ik["constraints and IK<br/>phiphx blend/rig, blend/rig/ik"]
        q_prop["propagate: world matrices,<br/>bones, pins and parts alike"]
    end
    subgraph read["Readout"]
        r_skin["skinning matrices gathered"]
        r_gpu["phiray kernels::Skin, then Tracer::repose"]
        r_row["bone Poses to the journal"]
    end
    p_play --> p_write --> q_ik --> q_prop --> r_skin --> r_gpu
    p_write --> r_row
```

```mermaid
graph TB
    subgraph today["phiplay today, play/world.rs"]
        t_act["act(): keeps the moment the old move was cut at,<br/>in the one was slot, :140-149"]
        t_rec["record(): overwrites it with t = 1.0,<br/>the old move's end, :216"]
        t_pop["the blend starts from a pose the body was not in: a pop"]
        t_act --> t_rec --> t_pop
    end
    h_snap["Handoff::Snapshot, on the general path:<br/>the blend starts from the bone Poses as last written"]
    h_math["a bone: translation and scale by lerp, rotation by slerp<br/>momentum math/transform.cpp:156-161"]
    t_pop -.->|"replaced by"| h_snap --> h_math
    classDef plan stroke-dasharray: 6 4;
    class h_snap,h_math plan;
```

```mermaid
graph LR
    key["what must agree for a character not to fracture,<br/>checked once by the rig's owner, phiphx, at Intake"]
    k_order["parents before children<br/>momentum character/skeleton.cpp:16-22"]
    k_bind["inverse rest derived from this rig's rest pose"]
    k_index["every influence names a bone that exists"]
    k_sum["weights: a vertex with none is reported.<br/>momentum's FBX reader renormalises, its glTF reader does not<br/>io/fbx/openfbx_loader.cpp:926-949 · io/gltf/gltf_mesh_io.cpp:451-466"]
    k_name["motion bound to bones by name, never by index<br/>momentum io/motion/mmo_io.cpp:174-219"]
    k_tree["a part is a child of its bone:<br/>no second matrix that could disagree"]
    k_refuse["a failing rig is refused: Loaded stays NotLoaded,<br/>an event says why"]
    k_order --> k_refuse
    k_bind --> k_refuse
    k_index --> k_refuse
    k_sum --> k_refuse
    k_name --> k_refuse
    k_tree --> k_refuse
```
