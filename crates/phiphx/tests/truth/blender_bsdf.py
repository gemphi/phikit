"""Regenerate tests/truth/bsdf.txt with real Blender (pip install bpy):

    python tests/truth/blender_bsdf.py -- <files...> | grep '\\.blend|' > tests/truth/bsdf.txt

One line per material with a Principled BSDF: file|material|base rgba|rough|metal|alpha|ior|transmission|emission rgb|strength.
Linked inputs are folded like phiphx `blend/shade/eval.rs` (colour ramps are
evaluated by Blender itself): image colour = white, procedural = 0.5; alpha,
emission, IOR and transmission keep their defaults unless the fold is exact.
"""
import os
import sys

import bpy

BLEND = {"MIX": 0, "ADD": 1, "MULTIPLY": 2, "SUBTRACT": 3, "SCREEN": 4, "DIVIDE": 5, "DIFFERENCE": 6,
         "DARKEN": 7, "LIGHTEN": 8, "OVERLAY": 9, "LINEAR_LIGHT": 17, "EXCLUSION": 18}
FOLDED = {"Base Color", "Roughness", "Metallic"}


def mix(blend, t, a, b):
    m = 1 - t
    def ch(x, y):
        if blend == 1: return x + t * y
        if blend == 2: return x * (m + t * y)
        if blend == 3: return x - t * y
        if blend == 4: return 1 - (m + t * (1 - y)) * (1 - x)
        if blend == 5: return m * x + t * x / y if y != 0 else x
        if blend == 6: return m * x + t * abs(x - y)
        if blend == 7: return m * x + t * min(x, y)
        if blend == 8: return m * x + t * max(x, y)
        if blend == 9: return x * (m + 2 * t * y) if x < 0.5 else 1 - (m + 2 * t * (1 - y)) * (1 - x)
        if blend == 17: return x + t * (2 * y - 1)
        if blend == 18: return m * x + t * (x + y - 2 * x * y)
        return m * x + t * y
    return [ch(a[0], b[0]), ch(a[1], b[1]), ch(a[2], b[2]), a[3]]


def hue_sat(c, hue, sat, val, fac):
    r, g, b, a = c
    mx, mn = max(r, g, b), min(r, g, b)
    d = mx - mn
    if d == 0: h = 0.0
    elif mx == r: h = ((g - b) / d) % 6 / 6
    elif mx == g: h = ((b - r) / d + 2) / 6
    else: h = ((r - g) / d + 4) / 6
    s = d / mx if mx != 0 else 0.0
    h = (h + hue + 0.5) % 1.0
    s, v = min(max(s * sat, 0), 1), mx * val
    def k(n):
        kk = (n + h * 6) % 6
        return v - v * s * min(max(min(kk, 4 - kk), 0), 1)
    rgb = [k(5), k(3), k(1)]
    return [r + fac * (rgb[0] - r), g + fac * (rgb[1] - g), b + fac * (rgb[2] - b), a]


def val(v, fl, exact): return (list(v), fl, exact)
def grey(x, exact): return ([x, x, x, 1.0], True, exact)
def value(v): return v[0][0] if v[1] else 0.2126 * v[0][0] + 0.7152 * v[0][1] + 0.0722 * v[0][2]


def default(s):
    if s.type in ("VALUE", "INT"): return grey(float(s.default_value), True)
    if s.type == "VECTOR": return grey(sum(s.default_value) / 3, True)
    if s.type == "RGBA": return val(s.default_value, False, True)
    return None


def inp(s, depth=0):
    if not s.is_linked: return default(s)
    if depth > 64: return None
    l = s.links[0]
    v = out(l.from_node, l.from_socket, depth + 1)
    # A link from a node this fold (or Blender: `Undefined[...]`, saved by a
    # Blender without the type) does not know is dead: the socket keeps its
    # own value, which is what Cycles renders.
    return v if v is not None else default(s)


MATH = ["ADD", "SUBTRACT", "MULTIPLY", "DIVIDE", "SINE", "COSINE", "TANGENT", "ARCSINE", "ARCCOSINE", "ARCTANGENT",
        "POWER", "LOGARITHM", "MINIMUM", "MAXIMUM", "ROUND", "LESS_THAN", "GREATER_THAN", "MODULO", "ABSOLUTE",
        "ARCTAN2", "FLOOR", "CEIL", "FRACT", "SQRT", "INVERSE_SQRT", "SIGN", "EXPONENT", "RADIANS", "DEGREES",
        "SINH", "COSH", "TANH", "TRUNC", "SNAP", "WRAP", "COMPARE", "MULTIPLY_ADD", "PINGPONG", "SMOOTH_MIN",
        "SMOOTH_MAX", "FLOORED_MODULO"]


def cpow(x, y):
    if y == 0: return 1.0
    if x < 0: return (-x) ** y if (-y) % 2 == 0 else -((-x) ** y)
    if x == 0: return 0.0
    return x ** y


def cmod(a, b): return a - int(a / b) * b if b != 0 else 0.0


def math_op(op, a, b, c):
    import math
    t = {
        "ADD": a + b, "SUBTRACT": a - b, "MULTIPLY": a * b, "DIVIDE": a / b if b != 0 else 0.0,
        "SINE": math.sin(a), "COSINE": math.cos(a), "TANGENT": math.tan(a),
        "ARCSINE": math.asin(a) if abs(a) <= 1 else 0.0, "ARCCOSINE": math.acos(a) if abs(a) <= 1 else 0.0,
        "ARCTANGENT": math.atan(a), "POWER": cpow(a, b),
        "LOGARITHM": math.log(a) / math.log(b) if a > 0 and b > 0 else 0.0,
        "MINIMUM": min(a, b), "MAXIMUM": max(a, b), "ROUND": math.floor(a + 0.5),
        "LESS_THAN": float(a < b), "GREATER_THAN": float(a > b), "MODULO": cmod(a, b), "ABSOLUTE": abs(a),
        "ARCTAN2": math.atan2(a, b), "FLOOR": math.floor(a), "CEIL": math.ceil(a), "FRACT": a - math.floor(a),
        "SQRT": math.sqrt(max(a, 0.0)), "INVERSE_SQRT": 1 / math.sqrt(a) if a > 0 else 0.0,
        "SIGN": (a > 0) - (a < 0), "EXPONENT": math.exp(a), "RADIANS": math.radians(a), "DEGREES": math.degrees(a),
        "SINH": math.sinh(a), "COSH": math.cosh(a), "TANH": math.tanh(a), "TRUNC": math.trunc(a),
        "SNAP": math.floor(a / b) * b if b != 0 else 0.0,
        "WRAP": a - (c - b) * math.floor((a - b) / (c - b)) if c != b else b,
        "COMPARE": float(abs(a - b) <= max(c, 1e-5)), "MULTIPLY_ADD": a * b + c,
        "PINGPONG": b - abs(abs(cmod(a - b, 2 * b)) - b) if b != 0 else 0.0,
        "FLOORED_MODULO": a - math.floor(a / b) * b if b != 0 else 0.0,
    }
    if op in ("SMOOTH_MIN", "SMOOTH_MAX"):
        aa, bb = (-a, -b) if op == "SMOOTH_MAX" else (a, b)
        if c != 0:
            h = min(max((c - abs(aa - bb)) / c, 0.0), 1.0)
            r = min(aa, bb) - h * h * h * c / 6.0
        else:
            r = min(aa, bb)
        return -r if op == "SMOOTH_MAX" else r
    return t.get(op)


def arg(n, ident, depth):
    s = next((x for x in n.inputs if x.identifier == ident), None) or n.inputs.get(ident)
    return inp(s, depth) if s is not None else None


def out(n, o, depth):
    t = n.bl_idname
    a = lambda i: arg(n, i, depth)
    if t == "ShaderNodeMix":
        f = a("Factor_Float")
        A, B = (a("A_Color"), a("B_Color")) if n.data_type == "RGBA" else (a("A_Float"), a("B_Float"))
        if None in (f, A, B): return None
        tt = min(max(value(f), 0), 1) if n.clamp_factor else value(f)
        c = mix(BLEND.get(n.blend_type, 0), tt, A[0], B[0])
        if n.clamp_result: c = [min(max(x, 0), 1) for x in c]
        return (c, A[1], f[2] and A[2] and B[2])
    if t == "ShaderNodeMixRGB":
        f, A, B = a("Fac"), a("Color1"), a("Color2")
        if None in (f, A, B): return None
        return (mix(BLEND.get(n.blend_type, 0), min(max(value(f), 0), 1), A[0], B[0]), False, f[2] and A[2] and B[2])
    if t == "ShaderNodeValToRGB":
        f = a("Fac")
        if f is None: return None
        c = list(n.color_ramp.evaluate(value(f)))
        return grey(c[3], f[2]) if o.name == "Alpha" else (c, False, f[2])
    if t == "ShaderNodeHueSaturation":
        xs = [a("Hue"), a("Saturation"), a("Value"), a("Fac"), a("Color")]
        if None in xs: return None
        h, s_, v, f, c = xs
        return (hue_sat(c[0], value(h), value(s_), value(v), value(f)), False, all(x[2] for x in xs))
    if t == "ShaderNodeBrightContrast":
        c, br, ct = a("Color"), a("Bright"), a("Contrast")
        if None in (c, br, ct): return None
        k, d = 1 + value(ct), value(br) - value(ct) * 0.5
        return ([max(k * c[0][i] + d, 0.0) for i in range(3)] + [c[0][3]], False, c[2] and br[2] and ct[2])
    if t == "ShaderNodeGamma":
        c, g = a("Color"), a("Gamma")
        if None in (c, g): return None
        return ([cpow(x, value(g)) if x > 0 else x for x in c[0][:3]] + [c[0][3]], False, c[2] and g[2])
    if t == "ShaderNodeMath":
        x, y, z = a("Value"), a("Value_001"), a("Value_002")
        if x is None: return None
        r = math_op(n.operation, value(x), value(y) if y else 0.0, value(z) if z else 0.0)
        if r is None: return None
        if n.use_clamp: r = min(max(r, 0.0), 1.0)
        return grey(r, x[2] and (y is None or y[2]) and (z is None or z[2]))
    if t == "ShaderNodeClamp":
        v, lo, hi = a("Value"), a("Min"), a("Max")
        if None in (v, lo, hi): return None
        r = (min(max(value(v), min(value(lo), value(hi))), max(value(lo), value(hi))) if n.clamp_type == "RANGE"
             else min(max(value(v), value(lo)), value(hi)))
        return grey(r, v[2] and lo[2] and hi[2])
    if t == "ShaderNodeInvert":
        f, c = a("Fac"), a("Color")
        if None in (f, c): return None
        tt = value(f)
        rgb = [c[0][k] + tt * (1 - 2 * c[0][k]) for k in range(3)]
        return (rgb + [c[0][3]], False, f[2] and c[2])
    if t in ("ShaderNodeRGB", "ShaderNodeValue"): return default(n.outputs[0])
    if t == "NodeReroute": return inp(n.inputs[0], depth)
    if t == "ShaderNodeRGBCurve":
        c = a("Color")
        return (c[0], c[1], False) if c else None
    if t == "ShaderNodeTexImage": return grey(1.0, False) if o.identifier == "Alpha" else ([1.0] * 4, False, False)
    if t.startswith("ShaderNodeTex"): return ([0.5, 0.5, 0.5, 1.0], False, False) if o.name == "Color" else grey(0.5, False)
    return None


def fmt(v): return ",".join(f"{x:.4f}" for x in v) if hasattr(v, "__len__") else f"{v:.4f}"


def principled(node):
    b = {"Base Color": [0.8, 0.8, 0.8, 1.0], "Roughness": 0.5, "Metallic": 0.0, "Alpha": 1.0, "IOR": 1.5,
         "Transmission": 0.0, "Emission": [0.0, 0.0, 0.0, 1.0], "Emission Strength": 1.0}
    alias = {"Transmission Weight": "Transmission", "Emission Color": "Emission"}
    for s in node.inputs:
        key = alias.get(s.name, s.name)
        if key not in b: continue
        v = inp(s)
        if v is None: continue
        if s.is_linked and not v[2] and key not in FOLDED: continue
        b[key] = list(v[0]) if s.type == "RGBA" else value(v)
    return b


for p in sys.argv[sys.argv.index("--") + 1:]:
    try:
        bpy.ops.wm.open_mainfile(filepath=p, load_ui=False)
    except RuntimeError as err:
        print(f"# {os.path.basename(p)}: {str(err).splitlines()[0]}", file=sys.stderr)
    for m in sorted(bpy.data.materials, key=lambda m: m.name):
        if m.node_tree is None:
            continue
        node = next((n for n in m.node_tree.nodes if n.type == "BSDF_PRINCIPLED"), None)
        if node is None:
            continue
        b = principled(node)
        fields = [fmt(b["Base Color"]), fmt(b["Roughness"]), fmt(b["Metallic"]), fmt(b["Alpha"]), fmt(b["IOR"]),
                  fmt(b["Transmission"]), fmt(b["Emission"][:3]), fmt(b["Emission Strength"])]
        print("|".join([os.path.basename(p), m.name] + fields))
