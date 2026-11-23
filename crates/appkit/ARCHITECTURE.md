# appkit - architecture

Diagrams only. The words are in [`PLAN.md`](PLAN.md) and [`SPECS.md`](SPECS.md).

## The layers

```mermaid
graph TB
    subgraph app["an app: app/phigen"]
        a_data["data/<br/>tables: space · layout · host · region · part · modal ·<br/>activity · menu · user · theme · text · model"]
        a_app["its PApp"]
        a_part["its parts: PView"]
    end
    subgraph pui["pui: the kit, no gpui"]
        p_core["core<br/>PDraw · PView · PEvent · PPaint · PSheet · PMode · PAsset · PIcon"]
        p_control["control<br/>PControl: PButton · PToggle · PField"]
        p_layout["layout<br/>wrap · PScroll"]
        p_window["window<br/>PWindow · PStage · PFrame · PDeed · PPanel ·<br/>space · zone · area · host · region · pane · part · modal"]
        p_bar["window/part<br/>PCaption · PActivity and its menus · PTab · PProfile"]
    end
    subgraph src["src: beside the kit"]
        s_theme["theme<br/>PTheme · PLook"]
        s_service["service<br/>PTable · PFolder · PBundle · PStore · PCall"]
        s_screen["screen<br/>the one place that names gpui"]
    end
    subgraph gpu["gpu: Zed's gpui"]
        g_core["gpui<br/>window, text, executor"]
        g_win["gpui_windows: DirectX"]
        g_web["gpui_web + gpui_wgpu: WebGPU"]
    end
    a_data --> a_app --> p_window
    a_app --> a_part --> p_core
    a_part --> p_control
    a_part --> p_layout
    p_window --> p_bar --> p_core
    p_control --> p_core
    p_core --> s_theme
    a_part --> s_service
    p_window --> s_screen --> g_core
    g_core --> g_win
    g_core --> g_web
```

## From tables to a window

```mermaid
graph LR
    t["an app's tables"] --> s["PSet of each level<br/>PSpace::arrange · PHost::declare"]
    s --> w["PWindow<br/>the space shown, the area in front, the modal open"]
    w --> st["PStage<br/>the window, the caption, the bar, the tabs, the view of every part"]
    st --> f["PFrame<br/>caption · bar · tabs · body"]
    f --> pl["PWindow::place<br/>desktop: zones by share · mobile: one area · spatial: an arc"]
    pl --> l["PHost::lay<br/>regions · panes · parts"]
    l --> v["every view has its rectangle"]
```

## A frame

```mermaid
graph TB
    r["render"] --> k["PStage::tick<br/>every view takes in what arrived"]
    k --> la["PStage::lay<br/>for the window's size and mode"]
    la --> d["PStage::draw on a PSheet"]
    d --> m["marks<br/>fill · text · icon · clip"]
    m --> p["screen::play<br/>quads, shaped lines, svg, content masks"]
    k -- "a view has more to come" --> n["another frame is asked for"]
    n --> r
```

## An event

```mermaid
graph TB
    e["a press, a key, the wheel"] --> q{"a modal open?"}
    q -- yes --> pn["PPanel: a square, a drawer, a sheet or the whole window<br/>Close · Expand · the modal's content"]
    q -- no --> sp{"on the bar, its menu or a tab?"}
    sp -- yes --> dd["PDeed<br/>Show · Front · Open · Pick · Side · Fold · Tell"]
    sp -- no --> pa["the part under it, or the part that has the keys"]
    pa --> wi["PWish<br/>what the part asks the window for"]
    dd --> ly["laid out again"]
    wi --> ly
    pn --> ly
```

## An answer over time

```mermaid
graph LR
    c["PCall::post"] --> th["a thread<br/>plain HTTP, a line at a time"]
    th --> ps["PStream"]
    ps --> tk["PView::tick<br/>once a frame"]
    tk --> vw["the view shows what has arrived"]
    stop["PStream::stop"] --> th
```

## At a desk and in a browser

```mermaid
graph TB
    run["an app's run"] --> sc["screen::run"]
    sc --> desk["at a desk<br/>application().run: stays in the system's loop"]
    sc --> web["in a browser<br/>single_threaded_web().run_embedded:<br/>the handle is kept for as long as the page"]
    desk --> as1["assets: the app's folder, then PBundle"]
    web --> as2["assets: PBundle, built in by build.rs"]
    desk --> st1["PStore: a folder of the person's"]
    web --> st2["no store, no thread: nothing kept, no call"]
```
