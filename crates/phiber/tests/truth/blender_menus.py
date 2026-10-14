"""Blender's own topbar menus, with the operator each entry runs, the
properties it sets on it, and the key bound to that same call.

A menu is not data in Blender. `TOPBAR_MT_render` is a Python class whose
`draw(self, context)` calls `layout.operator(...)`, `layout.prop(...)`,
`layout.menu(...)` and `layout.separator()` in order -- so the only way to
know what a menu really holds, in the real order with the real labels, is to
run that draw against a layout that records instead of drawing. That is what
`Recorder` is.

The properties matter as much as the operator. "Render Image" and "Render
Animation" are *the same operator*, `render.render`; what tells them apart is
that the second sets `animation = True`. A dump without the properties cannot
distinguish them, and neither can a test built on it.

The shortcut column comes from the keymap, because that is where it comes
from in Blender too -- a menu entry shows whatever key is bound to its
operator. The `bpy` module does not populate `keyconfigs` in background, so
this reads the keymap's *source*, `presets/keyconfig/keymap_data/
blender_default.py`, and matches a binding to a menu entry by operator **and
by the properties both set**: F12 is `render.render` with no `animation`, and
Ctrl F12 is `render.render` with `animation = True`.

Writes `menus.txt`, tab separated:

    menu, order, kind, identifier, label, properties, shortcut

`kind` is `op`, `prop`, `menu` or `sep`. `identifier` is the operator's
`bl_idname`, the property's identifier, or the submenu's class name. `label`
is the `text=` the menu passes, empty when it lets Blender use the
operator's own name. `properties` is `name=value` pairs, comma separated.

Run (Blender 5.x, or the `bpy` module at the same version):

    blender --background --factory-startup --python blender_menus.py > menus.txt
"""

import importlib.util
import os
import sys

import bpy
import bpy.utils

# The topbar menus, in the order `TOPBAR_MT_editor_menus.draw` lays them out.
MENU = ("TOPBAR_MT_file", "TOPBAR_MT_edit", "TOPBAR_MT_render", "TOPBAR_MT_window", "TOPBAR_MT_help")


# Two topbar submenus are registered in C rather than Python
# (`space_topbar.cc:214` and `:272`, `STRNCPY_UTF8(mt->label, ...)`), so the
# `bpy` module does not carry them as `bpy.types` classes in background and
# their labels cannot be read the way every other one is. They are quoted
# here from those two lines rather than derived from the identifier, which
# would be a guess that happens to look right.
C_MENU = {
    "TOPBAR_MT_file_open_recent": "Open Recent",
    "TOPBAR_MT_undo_history": "Undo History",
}


class Recorder:
    """A `layout` that writes down what a `draw` asks it to draw.

    Menus assign to an operator's return value (`props.animation = True`),
    chain sub-layouts (`layout.column()`, `layout.row()`) and set layout
    state (`layout.operator_context = ...`), so all three have to work and
    only the first is a recording.
    """

    def __init__(self, out, row=None):
        object.__setattr__(self, "_out", out)
        object.__setattr__(self, "_row", row)

    def operator(self, idname, text=None, **_kw):
        row = {"kind": "op", "id": idname, "label": text, "props": {}}
        self._out.append(row)
        return Recorder(self._out, row)

    def operator_menu_enum(self, idname, _property, text=None, **_kw):
        row = {"kind": "op", "id": idname, "label": text, "props": {}}
        self._out.append(row)
        return Recorder(self._out, row)

    def prop(self, _data, prop, text=None, **_kw):
        self._out.append({"kind": "prop", "id": prop, "label": text, "props": {}})
        return Recorder(self._out)

    def menu(self, idname, text=None, **_kw):
        self._out.append({"kind": "menu", "id": idname, "label": text, "props": {}})

    def separator(self, **_kw):
        self._out.append({"kind": "sep", "id": "", "label": None, "props": {}})

    def __getattr__(self, _name):
        # Any other layout call -- column(), row(), template_*() -- just has
        # to hand back something a menu can keep drawing into.
        return lambda *a, **kw: Recorder(self._out)

    def __setattr__(self, name, value):
        # `props.animation = True` on an operator's return value is the
        # property assignment this dump exists to capture; anything else is
        # layout state and is ignored.
        row = object.__getattribute__(self, "_row")
        if row is not None:
            row["props"][name] = value


class Shell:
    """What a `draw` sees as `self`."""

    def __init__(self, out):
        self.layout = Recorder(out)


def keymap_source():
    """`blender_default.py`'s own `generate_keymaps`, imported by path.

    The `bpy` module leaves `keyconfigs` empty in background, so the
    generated keymaps are read from the preset that generates them.
    """
    path = os.path.join(
        bpy.utils.resource_path("LOCAL"), "scripts/presets/keyconfig/keymap_data/blender_default.py"
    )
    spec = importlib.util.spec_from_file_location("blender_default", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module.generate_keymaps(module.Params(legacy=False))


def rna_name(kind, identifier):
    """An entry's own name, for the entries that pass no `text=`.

    `layout.operator("ed.undo")` shows "Undo" because that is
    `RNA_struct_ui_name` of the operator; a property with no `text=` shows
    `RNA_property_ui_name`. Reading them here is what keeps the dump a
    record of what the menu *reads* rather than what it was handed.
    """
    if kind == "op":
        module, _, name = identifier.partition(".")
        try:
            return getattr(getattr(bpy.ops, module), name).get_rna_type().name
        except (AttributeError, RuntimeError):
            return ""
    if kind == "prop":
        for owner in (bpy.types.Scene, bpy.types.RenderSettings, bpy.types.ToolSettings, bpy.types.Screen, bpy.types.Preferences):
            prop = owner.bl_rna.properties.get(identifier)
            if prop is not None:
                return prop.name
    if kind == "menu":
        # `layout.menu("TOPBAR_MT_file_recover")` with no `text=` reads the
        # submenu's own `bl_label`, which is "Recover".
        menu = getattr(bpy.types, identifier, None)
        label = getattr(menu, "bl_label", None) if menu is not None else None
        return label if label is not None else C_MENU.get(identifier, "")
    return ""


# `WM_key_event_string` special-cases these ahead of the RNA enum
# (`wm_keymap.cc:1131`), and their text is not the enum's name.
SPECIAL = {
    "TAB": "Tab",
    "BACK_SPACE": "Bksp",
    "ESC": "Esc",
    "RET": "Enter",
    "SPACE": "Space",
    "LEFT_ARROW": "Left",
    "UP_ARROW": "Up",
    "RIGHT_ARROW": "Right",
    "DOWN_ARROW": "Down",
}


def key_name(identifier):
    """`WM_key_event_string(type, compact=false)`: the key's own text.

    A menu's shortcut goes through `WM_key_event_operator_string`, which ends
    in `WM_keymap_item_to_string(kmi, false)` -- **compact false**
    (`interface.cc:1678`). So the text is the event enum's `name`, not the
    short `description` the compact form uses: `COMMA` reads "," and
    `PAGE_DOWN` reads "Page Down", not "PgDown".
    """
    if identifier in SPECIAL:
        return SPECIAL[identifier]
    item = bpy.types.KeyMapItem.bl_rna.properties["type"].enum_items.get(identifier)
    return item.name if item is not None else identifier


def spell(binding):
    """A key binding as a menu prints it.

    The modifier order is `WM_keymap_item_raw_to_string`'s, which appends
    **Shift, Ctrl, Alt, OS** in that order (`wm_keymap.cc:1222`) -- so
    Blender prints "Shift Ctrl S", never "Ctrl Shift S". Getting this
    backwards is the easy mistake, and it is visible in every menu.
    """
    part = [name for name, flag in (("Shift", "shift"), ("Ctrl", "ctrl"), ("Alt", "alt"), ("Cmd", "oskey")) if binding.get(flag)]
    part.append(key_name(binding.get("type", "")))
    return " ".join(part)


def bindings():
    """operator idname -> [(properties, key spelling)], most specific first."""
    found = {}
    for _name, _poll, data in keymap_source():
        for entry in data.get("items", []):
            idname, binding, extra = entry[0], entry[1], entry[2] if len(entry) > 2 else None
            if not isinstance(idname, str) or binding.get("value") != "PRESS":
                continue
            prop = dict((extra or {}).get("properties") or [])
            found.setdefault(idname, []).append((prop, spell(binding)))
    # A binding that sets more properties is the more specific match, so it
    # is tried first: Ctrl F12 (animation=True) before plain F12.
    for value in found.values():
        value.sort(key=lambda pair: -len(pair[0]))
    return found


def shortcut_for(bound, row):
    """The key bound to this exact call -- same operator, same properties.

    The match has to be exact, not a subset. `render.render` carries four
    bindings that differ only in properties: F12 sets `use_viewport`, Ctrl
    F12 adds `animation`, Alt F12 adds `use_sequencer_scene`, Ctrl Alt F12
    adds both. Treating any of those properties as noise makes "Render
    Image" match Alt F12 -- a different command -- so none of them are.
    """
    for prop, key in bound.get(row["id"], []):
        if prop == row["props"]:
            return key
    return ""


bound = bindings()
for name in MENU:
    menu = getattr(bpy.types, name, None)
    if menu is None:
        print(f"# {name}: not in this build", file=sys.stderr)
        continue
    out = []
    try:
        menu.draw(Shell(out), bpy.context)
    except Exception as why:  # a menu needing more context than background gives
        print(f"# {name}: {type(why).__name__}: {why}", file=sys.stderr)
        continue
    for order, row in enumerate(out):
        prop = ",".join(f"{key}={value}" for key, value in sorted(row["props"].items()))
        key = shortcut_for(bound, row) if row["kind"] == "op" else ""
        label = row["label"] or rna_name(row["kind"], row["id"])
        print(f"{name}\t{order}\t{row['kind']}\t{row['id']}\t{label}\t{prop}\t{key}")
    print(f"# {name}: {len(out)} entries", file=sys.stderr)
