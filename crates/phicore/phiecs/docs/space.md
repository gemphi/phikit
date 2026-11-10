# space

Every box on this page is to build. Today each consumer keeps its own
placement: `BBody` in phiplay, `Placement` in simkit, name pairs in phiphx's
`SceneModel.parents`.

```mermaid
graph TB
    t_tree["Tree<br/>child to parent, a loop is refused<br/>from phinet phiecs node/tree.rs"]
    t_pose["Pose { t, r, s }<br/>local, plain f32 arrays<br/>written by animation, physics, the user"]
    t_prop["propagate, a PostCouple system<br/>roots first: world = parent world x local"]
    t_world["WorldMatrix<br/>derived every step, never written by hand"]
    t_change["changed tick on WorldMatrix"]
    t_out["Readout<br/>phiray Tracer::reinstance(word's instance, matrix)"]
    t_tree --> t_prop
    t_pose --> t_prop
    t_prop --> t_world --> t_change --> t_out
```

```mermaid
graph TB
    r_call["World::set_parent(child, parent, keep_world)"]
    r_keep{"keep_world"}
    r_same["local unchanged<br/>the child jumps with its new parent"]
    r_solve["local = inverse(new parent world) x old world<br/>the child stays where it is"]
    r_event["Events: DidReparent(child, old, new)"]
    r_call --> r_keep
    r_keep -->|false| r_same
    r_keep -->|true| r_solve
    r_same --> r_event
    r_solve --> r_event
```

```mermaid
graph LR
    p_skel["rig: Skeleton + JointPose<br/>the parent's joints in world space"]
    p_pin["Pin { joint, offset }<br/>a named point on a joint<br/>GeometricPin, RF:9088"]
    p_att["Attached { to, pin }<br/>on the child<br/>AttachedTransformComponent, RF:13669"]
    p_derive["attach, a PostCouple system after propagate<br/>child world = joint world x offset"]
    p_world["child's WorldMatrix<br/>its own Pose is ignored while Attached"]
    p_nogap["a part cannot drift from its joint:<br/>there is no second matrix to disagree"]
    p_skel --> p_derive
    p_pin --> p_derive
    p_att --> p_derive
    p_derive --> p_world --> p_nogap
```
