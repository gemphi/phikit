# glass — architecture

Solid boxes are in the code today. Dashed boxes are to build.

## One crate, every renderer

```mermaid
graph TB
    subgraph host["who asks for glass"]
        h_gpui["gpui: BackdropStyle on an element<br/>phiber, any gpui app"]
        h_view["an NSView tree<br/>appkit/old/glass_view.rs, not built"]
        h_page["a page: a canvas on WebGPU"]
    end
    subgraph crate["gpui_glass"]
        c_effect["effect<br/>GlassEffect · GlassContainer<br/>NSGlassEffectView.h:14-57"]
        c_material["material + data/style.tsv<br/>GlassStyle · GlassMaterial"]
        c_shape["shape<br/>GlassShape · GlassBounds"]
        c_cpu["cpu<br/>capture · blur · shade<br/>the reference"]
        c_plan["plan<br/>GlassRequest to GlassPlan:<br/>one copy, four draws"]
        c_hlsl["src/glass.hlsl"]
        c_wgsl["src/glass.wgsl"]
    end
    subgraph back["who draws it"]
        b_dx["gpui_windows: DirectX 11<br/>directx_backdrop.rs, from the plan"]
        b_wgpu["gpui_wgpu: wgpu, Linux and the web"]
        b_soft["a software canvas"]
    end
    h_gpui --> c_material
    h_view --> c_effect --> c_material
    h_page --> c_plan
    c_material --> c_cpu
    c_shape --> c_cpu
    c_material --> c_plan
    c_cpu --> b_soft
    c_plan --> c_hlsl --> b_dx
    c_plan --> c_wgsl --> b_wgpu
    b_dx -.->|"takes its numbers from plan, not its own copy"| c_plan
    classDef plan stroke-dasharray: 6 4;
    class b_wgpu plan;
```

## The stages, the same on the CPU and on a GPU

```mermaid
graph TB
    s0["what is drawn so far"]
    s1["copy the region under the glass,<br/>with room for the blur and the lens"]
    s2["shrink: a half or a quarter"]
    s3["blur across, then down"]
    s4["the shape: lens at the rim, saturation,<br/>tint that thickens over contrast, edge light"]
    s5["drawn back over the window,<br/>content on top stays sharp"]
    s0 --> s1 --> s2 --> s3 --> s4 --> s5
```

## Where each backend stands

```mermaid
graph TB
    d1["DirectX: draws glass in phiber today.<br/>Its pass keeps a private copy of the plan arithmetic"]
    d2["DirectX takes GlassPlan from the crate"]
    w1["WGSL: the shader and the plan run on WebGPU.<br/>Checked in a browser: blurred inside, sharp outside,<br/>rim lit, rounded corner respected"]
    w2["gpui_wgpu draws Backdrops with it.<br/>Today it skips them: wgpu_renderer.rs:1307"]
    w3["GPU output compared with the CPU reference, in a test"]
    c1["CPU: 12 tests against real pixels, running again"]
    d1 -.-> d2
    w1 -.-> w2 -.-> w3
    c1 --> w3
    classDef plan stroke-dasharray: 6 4;
    class d2,w2,w3 plan;
```
