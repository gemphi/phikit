# Organisation of `crates/phiber/src/v2`

How the tree is meant to be organised, every file in it, and where it does
not follow its own rule. The rule first, so the deviations can be judged
against it.

## The rule

The tree is Blender's window model, one folder per level, top to bottom:

```
Space  →  Zone  →  Area  →  Host  →  Region  →  Pane  →  Part  →  atoms
(11)      (7)      slot     (36)     (17)       (3)      (30)
```

Every level folder has the same five files, and nothing else at its root:

| file      | holds                                                        |
|-----------|--------------------------------------------------------------|
| `type.rs` | the enum: every kind this level can be, with `slug()`/`label()` |
| `spec.rs` | the static descriptor for one kind (`BSpec`)                  |
| `set.rs`  | the table: one spec per kind, in Blender's order              |
| `item.rs` | builds one `BView` for one kind (`BViewItem`)                 |
| `list.rs` | builds the children of one item (`BViewList`)                 |

A kind that has content of its own gets a folder named after it (`host/file/`,
`part/menubar/`, `modal/splash/`) holding a `mod.rs` and, when it needs them,
its own `set.rs` / `item.rs` / `list.rs`. Content is **declared** in `set.rs`
tables; `item.rs` turns tables into views; nothing at these levels knows what
a pixel is.

Everything visual is in `view/`: `view/prim/` is the vocabulary a level may
speak (`BView::row`, `panel`, `table_row`, `prose`…), `view/render/` decides
layout per level (the `B…Layout` traits) and `view/draw/` draws —
one walker per level, `flex/` as the one layout element, `atom/` and
`prim/` as the leaves. `state/` is the one string key–value store every
action writes; `theme/` is every number and colour; `scene/` is the loaded
file; `app/` is the window.

## Every file

Each folder lists its files first, then its sub-folders.

```
v2/
    mod.rs
    spec.rs
    app/
        README.md
        handle.rs
        import.rs
        import_tests.rs
        jobs.md
        load.rs
        load_ui.rs
        mod.rs
        playback.rs
        seed.rs
        settle_tests.rs
        trace.rs
    area/
        README.md
        current.rs
        item.rs
        jobs.md
        list.rs
        mod.rs
        spec.rs
        type.rs
    host/
        README.md
        header.rs
        item.rs
        jobs.md
        list.rs
        mod.rs
        set.rs
        spec.rs
        type.rs
        action/
            mod.rs
        activity/
            mod.rs
        asset/
            mod.rs
        buts/
            mod.rs
        clip/
            mod.rs
        compositor/
            mod.rs
        console/
            mod.rs
        drivers/
            mod.rs
        file/
            mod.rs
        geonode/
            mod.rs
        image/
            mod.rs
        info/
            mod.rs
        ipo/
            mod.rs
        logic/
            mod.rs
        nla/
            mod.rs
        node/
            mod.rs
        outliner/
            mod.rs
        prefs/
            mod.rs
        seq/
            mod.rs
        shader/
            mod.rs
        spreadsheet/
            mod.rs
        statusbar/
            mod.rs
        text/
            mod.rs
        time/
            mod.rs
        topbar/
            mod.rs
        uv/
            mod.rs
        view3d/
            mod.rs
    modal/
        README.md
        item.rs
        jobs.md
        list.rs
        mod.rs
        set.rs
        spec.rs
        type.rs
        about/
            mod.rs
            set.rs
        collab/
            mod.rs
        command/
            mod.rs
            palette.rs
            set.rs
        file_browser/
            mod.rs
        help/
            mod.rs
            set.rs
        profile/
            mod.rs
            set.rs
        progress/
            mod.rs
        reader/
            mod.rs
            set.rs
            store/
                about.rs
                community.rs
                credits.rs
                get_involved.rs
                help_system.rs
                license.rs
                mod.rs
                release_notes.rs
                report_bug.rs
                support.rs
                tutorials.rs
                whats_new.rs
        render/
            mod.rs
        settings/
            mod.rs
            set.rs
        spaces/
            mod.rs
        splash/
            mod.rs
            recent.rs
            set.rs
    pane/
        README.md
        item.rs
        jobs.md
        list.rs
        mod.rs
        spec.rs
        type.rs
    part/
        README.md
        item.rs
        jobs.md
        list.rs
        mod.rs
        set.rs
        spec.rs
        strip.rs
        type.rs
        activity/
            list.rs
            mod.rs
            options.rs
            set.rs
        assetshelf/
            list.rs
            mod.rs
            set.rs
        caption/
            mod.rs
        channels/
            item.rs
            list.rs
            mod.rs
            set.rs
        custom/
            item.rs
            list.rs
            mod.rs
            set.rs
            action/
                item.rs
                list.rs
                mod.rs
            buts/
                item.rs
                list.rs
                mod.rs
            clip/
                item.rs
                list.rs
                mod.rs
            file/
                asset_sidebar.rs
                execute.rs
                fsmenu.rs
                header.rs
                history.rs
                item.rs
                keys.rs
                kind.rs
                list.rs
                listing.rs
                mod.rs
                options.rs
                path_bar.rs
                preview.rs
                purpose.rs
                set.rs
                sidebar.rs
                table.rs
                thumbs.rs
            image/
                item.rs
                list.rs
                mod.rs
            node/
                item.rs
                list.rs
                mod.rs
            outliner/
                item.rs
                keys.rs
                layer.rs
                list.rs
                mod.rs
                set.rs
                tree.rs
                type_set.rs
            prefs/
                item.rs
                list.rs
                mod.rs
            seq/
                item.rs
                list.rs
                mod.rs
            spreadsheet/
                item.rs
                list.rs
                mod.rs
            text/
                info.rs
                item.rs
                list.rs
                mod.rs
            time/
                item.rs
                list.rs
                mod.rs
            view3d/
                item.rs
                list.rs
                mod.rs
        execute/
            list.rs
            mod.rs
            set.rs
        filter/
            mod.rs
        hud/
            item.rs
            list.rs
            mod.rs
            set.rs
        menubar/
            item.rs
            list.rs
            mod.rs
            set.rs
            item/
                tree.rs
        modebar/
            list.rs
            mod.rs
            set.rs
            shading.rs
        navbar/
            list.rs
            mod.rs
            set.rs
        panel/
            item.rs
            list.rs
            mod.rs
            prop.rs
            prop_view.rs
            set.rs
            collection/
                mod.rs
                set.rs
            constraint/
                mod.rs
                set.rs
            data/
                mod.rs
                set.rs
            material/
                mod.rs
                set.rs
            modifier/
                mod.rs
                set.rs
            object/
                mod.rs
                set.rs
            output/
                mod.rs
                set.rs
            particles/
                mod.rs
                set.rs
            physics/
                mod.rs
                set.rs
            prefs/
                control.rs
                mod.rs
                set.rs
                control/
                    field.rs
                    selector.rs
                    toggle.rs
            render/
                mod.rs
                set.rs
            scene/
                mod.rs
                set.rs
            texture/
                mod.rs
                set.rs
            tool/
                mod.rs
                set.rs
            view_layer/
                mod.rs
                set.rs
            world/
                mod.rs
                set.rs
        search/
            mod.rs
        sidebar/
            list.rs
            mod.rs
            set.rs
        spacebar/
            list.rs
            mod.rs
            set.rs
        statusbar/
            item.rs
            list.rs
            mod.rs
            set.rs
        statuslog/
            mod.rs
        switcher/
            list.rs
            mod.rs
            set.rs
        toolbar/
            list.rs
            mod.rs
            set.rs
        toolprops/
            item.rs
            list.rs
            mod.rs
            set.rs
        zonebar/
            mod.rs
    region/
        README.md
        item.rs
        jobs.md
        list.rs
        mod.rs
        spec.rs
        type.rs
    scene/
        README.md
        camera.rs
        color.rs
        jobs.md
        keys.rs
        keys_tests.rs
        mod.rs
        project.rs
        shading.rs
        skin.rs
        source.rs
        tests_project.rs
        tests_scene.rs
        tests_state.rs
        extras/
            mod.rs
            set.rs
            tests.rs
    space/
        README.md
        animation.rs
        compositing.rs
        geometry_nodes.rs
        item.rs
        jobs.md
        layout.rs
        list.rs
        mod.rs
        modeling.rs
        rendering.rs
        scripting.rs
        sculpting.rs
        set.rs
        shading.rs
        spec.rs
        texture_paint.rs
        type.rs
        uv_editing.rs
    state/
        README.md
        action.rs
        area_resize.rs
        edit.rs
        file_nav.rs
        follow.rs
        jobs.md
        key.rs
        log.rs
        menu.rs
        mod.rs
        path.rs
        seed.rs
        space_switch.rs
        val.rs
        zone_size.rs
    theme/
        README.md
        descriptor.rs
        jobs.md
        mod.rs
        resolver.rs
        semantic.rs
        set.rs
        style.rs
        vendor.rs
        descriptor/
            parser.rs
            table.rs
            tests.rs
        palette/
            mod.rs
            set.rs
        style/
            colors.rs
            effects.rs
            metrics.rs
            radii.rs
            tests.rs
    view/
        README.md
        jobs.md
        mod.rs
        prim/
            accordion.rs
            action.rs
            canvas.rs
            control.rs
            field.rs
            icon.rs
            menu.rs
            mod.rs
            panel.rs
            prose.rs
            rail.rs
            row.rs
            table.rs
            thumb.rs
            tree.rs
        render/
            area.rs
            ascii.rs
            host.rs
            mod.rs
            pane.rs
            region.rs
            space.rs
            zone.rs
            gpui/
                area.rs
                atom.rs
                backend.rs
                color.rs
                ctx.rs
                host.rs
                mod.rs
                pane.rs
                part.rs
                popover.rs
                region.rs
                space.rs
                surface.rs
                zone.rs
                atom/
                    anchor.rs
                    brand.rs
                    button.rs
                    caption.rs
                    chrome.rs
                    field.rs
                    log.rs
                    menu.rs
                    progress.rs
                    prose.rs
                    rail.rs
                    search.rs
                    tab.rs
                    thumb.rs
                    toggle.rs
                    vlabel.rs
                    canvas/
                        fallback.rs
                        gizmo.rs
                        image.rs
                        mod.rs
                        node.rs
                        overlay.rs
                        surface.rs
                        view3d.rs
                        gizmo/
                            compass.rs
                            compass/
                                axis.rs
                        view3d/
                            cache.rs
                            corner.rs
                            draw.rs
                            extras.rs
                            floor.rs
                            hit.rs
                            mesh.rs
                            object.rs
                            orbit.rs
                            rendered.rs
                            mesh/
                                hull.rs
                                kernel.rs
                                paint.rs
                                raster.rs
                                studio.rs
                                types.rs
                                raster/
                                    tests.rs
                                studio/
                                    tests.rs
                            rendered/
                                key.rs
                                scene.rs
                                settle.rs
                                tests.rs
                                verts.rs
                                worker.rs
                    tab/
                        cap.rs
                flex/
                    apply.rs
                    element.rs
                    mod.rs
                    surface.rs
                    token.rs
                modal/
                    card.rs
                    fullscreen.rs
                    mod.rs
                    part.rs
                    title.rs
                    welcome.rs
                popover/
                    place.rs
                prim/
                    accordion.rs
                    column.rs
                    list.rs
                    mod.rs
                    render_contract.rs
                    row.rs
                    sort.rs
                    strip.rs
                    table.rs
                    tree.rs
                    panel/
                        body.rs
                        card.rs
                        footer.rs
                        header.rs
                        heading.rs
                        mod.rs
                        modal.rs
                        resize.rs
                        scroll.rs
                        section.rs
                        sidebar.rs
                        style.rs
                        toolbar.rs
                        viewport.rs
                    tree/
                        disclosure.rs
                        guide.rs
                region/
                    tests.rs
                space/
                    part.rs
                    seam.rs
                surface/
                    corners.rs
                    treatment.rs
                zone/
                    seam.rs
    zone/
        README.md
        item.rs
        jobs.md
        list.rs
        mod.rs
        set.rs
        spec.rs
        type.rs
        visibility.rs
```

## Where the structure is wrong

Judged against the rule above. Ordered by how much they mislead a reader.

### 1. Two hosts still build their regions inside their `Custom` part

`part/custom/time/item.rs` builds a `BPanelRole::Toolbar` (the transport:
skip / step / play / frame) and stacks it over the channel body by hand. That
is exactly what `part/custom/file/item.rs` did until sync107, and it is the
same mistake: the transport is Blender's `RGN_TYPE_HEADER` of the timeline
(`TIME_HT_editor_buttons`) — a region the host should declare, drawn by the
region walker, with its controls centred by the same strip rule as every
other header. `host/time/mod.rs` declares only Header + Center, so the
transport is a strip nobody at the region level knows about.

`part/custom/pref/item.rs` builds a `BPanelRole::Sidebar` for the
preferences sections — Blender's `space_userpref` has a NavBar region for
that, and `host/pref/mod.rs` already declares it. So the same list exists
twice: once as a region, once inside the body.

**Fix:** declare `ToolHeader` (transport) in `host/time`, drop the panel from
the custom item; drop the sidebar from `part/custom/prefs` and let the NavBar
region own it.

### 2. `part/custom/file/` is a host, not a part

Nineteen files: a listing, a history, a file-system menu, a purpose enum,
options, previews, thumbnails, a table, a sidebar, an asset sidebar, a path
bar, an execute bar, a header. After sync107 four of these are *regions'*
parts (`PathBar`, `SourceList`, `Execute`, the table) that `part/item.rs`
reaches by `crate::v2::part::custom::file::…` paths, which is a `Custom`
folder serving non-custom parts. `header.rs` in there is now dead (the
dialog uses the host header). And the listing / history / navigation code is
state and I/O, not a part at all — `state/file_nav.rs` already holds half of
it.

**Fix:** split it: `part/pathbar/`, `part/sourcelist/` and
`part/execute/` (which already exists, with a dummy `set.rs` of actions the
file browser no longer uses — delete `FILE`/`ASSET` from it or delete the
file) become real part folders with their own `set.rs`; the listing,
history, fsmenu, purpose and keys move to `state/file/` beside `file_nav.rs`;
`part/custom/file/` keeps only the table and the thumbnail grid.

### 3. Loose files at level roots that are not one of the five

- `host/header.rs` — the `EDITOR` pane template every host header starts
  from. It is a *region* fact (what a Header region's panes are), so it
  belongs in `region/` (as `region/header.rs`, or as the Header entry's
  default panes in `region/set.rs`).
- `part/strip.rs` — expand/collapse keys and label cycling for rails. Half of
  it is state (`expanded_key`, `is_expanded`) and belongs in `state/`, half is
  a rail helper and belongs in `part/navbar/` or `part/toolbar/`.
- `zone/visibility.rs`, `area/current.rs` — state readers living in the level
  folder instead of `state/`. `state/zone_size.rs` and
  `state/area_resize.rs` already exist for the same levels, so a reader has
  to look in two places for one level's state.
- `modal/splash/recent.rs` — the recent-files list is app state (it is
  written by the action reducer in `state/action.rs`), not a splash concern.

### 4. Modals compose panels, not regions — but only some of them

`modal/settings`, `modal/file_browser`, `modal/render` raise a host and get
the region walkers for free. `modal/about`, `modal/profile`, `modal/collab`,
`modal/help`, `modal/reader` and `modal/splash` hand-build `Card` /
`Section` / `Footer` panels. The profile modal's `Footer` panel *is* an
Execute region; the splash's cards *are* what a `Center` region with two
panes would give. Two ways to build a dialog means two sets of padding and
alignment rules — which is the header-height inconsistency you saw. The
title bar (`view/draw/modal/title.rs`) is a hand-built Header region
too (see `bflex.md`, "the structural step for modals").

**Fix:** a modal body should be `BView::Host { .. }` or a list of
`BView::Region { .. }`, always; the small dialogs get a `BHostType` of their
own (Blender has `SPACE_USERPREF` for Preferences; About and Splash are
`WM_MT_splash*` menus, which map to a host with one Center region).

### 5. `theme/set.rs` (`METRICS`) duplicates `theme/style/metric.rs`

`METRICS` is a 40-field struct of `h_header`, `w_toolbar`, `h_bottom_strip`…
each initialised from the matching `DEFAULT_DIMENSIONS`/`DEFAULT_SPACING`
field. Every new dimension has to be added in four places (`metrics.rs`,
`set.rs`, `descriptor/table.rs`, `state/key.rs`) and the `h_`/`w_` names
differ from the descriptor names (`h_banner` vs `banner_height`). `METRICS`
exists for tests to read defaults; they can read `DEFAULT_DIMENSIONS`
directly.

**Fix:** delete `theme/set.rs`; point the ~20 test uses at
`DEFAULT_DIMENSIONS` / `DEFAULT_SPACING`.

### 6. The `state/` folder is a flat bag

`area_resize.rs`, `zone_size.rs`, `space_switch.rs`, `file_nav.rs`,
`menu.rs`, `edit.rs`, `log.rs`, `seed.rs` — reducers, readers and helpers
for six different levels, all at one depth. The level folders meanwhile keep
their own readers (§3). The pattern the rest of the tree uses would be
`state/<level>/` mirroring `space/`, `zone/`, `area/`… so a reader finds a
level's state where they find the level.

### 7. Renderer: `render_contract.rs` (`BFocusedRenderer`) is dead

Twenty-five files implement it; nothing calls through it; it is
`#[allow(dead_code)]`. It certifies that each file styles itself, which is
the opposite of the `BFlex` direction, and `file_browser_accordion_refinement`
is the only test still checking for it. Delete the trait and the impls.

### 8. Test files inside `src/`

`app/import_tests.rs`, `app/settle_tests.rs`, `scene/keys_tests.rs`,
`scene/tests_project.rs`, `scene/tests_scene.rs`, `scene/tests_state.rs`,
`theme/style/tests.rs`, `theme/descriptor/tests.rs`, `scene/extra/tests.rs`,
and the `raster/tests.rs` / `studio/tests.rs` folders under `view3d/mesh`.
Two naming conventions (`x_tests.rs` and `tests_x.rs`) for one thing, and
`crates/phiber/tests/` already holds 80 integration test files. Unit tests
that need private items should be a `mod tests` at the foot of the file they
test; everything else goes to `tests/`.

### 9. Small inconsistencies

- `part/menubar/item/tree.rs` is the only part with an `item/` *folder*;
  every other part has `item.rs`.
- `part/props/` has `prop.rs` + `prop_view.rs` at its root while its 17
  property tabs are folders — `prop*` should be a folder like them.
- `modal/file_browser/mod.rs` still re-exports `BFileCategory`, `COLUMNS`,
  `FILTERS`… from `part::custom::file::set` — leftover from when it built the
  browser itself. Callers should import from the set directly.
- `host/spec.rs`, `region/spec.rs`, `pane/spec.rs` each define `B…Spec` with
  the same `kind` / `label` / `icon` shape; `spec.rs` at the v2 root already
  has `BSpec`. That is fine as a trait — but three of the five levels'
  `spec.rs` files are under ten lines, which suggests the descriptor belongs
  in `type.rs` and `spec.rs` only earns a file when it carries real fields
  (`BModalSpec` does: width, height, shape).
- `view/draw/canvas/view3d/` is 20 files of viewport drawing
  under `atom/`. It is not an atom; it is a renderer of its own and reads
  better as `view/draw/viewport/`, beside `modal/` and `popover/`.
- Every level has a `README.md` and a `jobs.md`; `host/jobs.md` and
  `part/jobs.md` describe work finished several syncs ago. A stale jobs file
  is worse than none.

## What is right, for the record

The five-file pattern holds at `space/`, `zone/`, `area/`, `region/`,
`pane/` and `modal/` without exception. Every host is one `mod.rs`
declaring regions and nothing else — 36 of 36. The renderer has one walker
per level and the level walkers carry no raw style (`tests/style_budget.rs`
pins it). `theme/descriptor/table.rs` is the single list of every tunable
number, and Preferences is generated from it. The `reader/store/` shape —
one declared document per file, a `mod.rs` that lists them — is the model
the file browser's set and the menubar's tables should follow too.
