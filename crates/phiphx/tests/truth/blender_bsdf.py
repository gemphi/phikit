"""Regenerate tests/truth/bsdf.txt with real Blender (pip install bpy):

    python tests/truth/blender_bsdf.py -- <files...> | grep '\\.blend|' > tests/truth/bsdf.txt

One line per material with a Principled BSDF: file|material|base rgba|rough|metal|alpha|ior|transmission|emission rgb|strength.
Linked inputs are folded like phiphx `blend/shade/eval.rs` and
`blend/shade/fold.rs` (colour ramps are evaluated by Blender itself): image
colour = white, procedural = 0.5; alpha, emission, IOR and transmission keep
their defaults unless the fold is exact. Node groups are expanded as
`blend/shade/group.rs` expands them — a link out of a group node is followed
to what feeds its Group Output, a link out of a Group Input back to the
instance's own socket — and a vector socket keeps its three components, which
read as a float the way `svm_node_convert` converts them (their mean).
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


class VEC:
    """The flag of a vector value: its three components in `v[0]`, read as a
    float by their mean (`svm_node_convert`), not by luminance."""


def val(v, fl, exact): return (list(v), fl, exact)
def grey(x, exact): return ([x, x, x, 1.0], True, exact)
def vec3(xyz, exact): return ([xyz[0], xyz[1], xyz[2], 1.0], VEC, exact)


def value(v):
    if v[1] is VEC: return (v[0][0] + v[0][1] + v[0][2]) / 3
    return v[0][0] if v[1] else 0.2126 * v[0][0] + 0.7152 * v[0][1] + 0.0722 * v[0][2]


def xyz(v): return v[0][:3]


def sock(sockets, ident):
    return next((x for x in sockets if x.identifier == ident), None) or sockets.get(ident)


def group_output(tree):
    outs = [n for n in tree.nodes if n.bl_idname == "NodeGroupOutput"]
    return next((n for n in outs if n.is_active_output), outs[0] if outs else None)


def default(s):
    if s.type in ("VALUE", "INT"): return grey(float(s.default_value), True)
    if s.type == "VECTOR": return vec3(list(s.default_value), True)
    if s.type == "RGBA": return val(s.default_value, False, True)
    return None


def inp(s, depth=0, stack=()):
    if not s.is_linked: return default(s)
    if depth > 64: return None
    l = s.links[0]
    n = l.from_node
    # A node group is expanded, not folded: its output socket is whatever
    # feeds the matching input of the group's active Group Output, evaluated
    # inside the group; a Group Input climbs back out to the instance's own
    # socket, whose value it takes when the instance leaves it unlinked.
    if n.bl_idname == "ShaderNodeGroup":
        go = group_output(n.node_tree) if n.node_tree else None
        inner = sock(go.inputs, l.from_socket.identifier) if go else None
        v = inp(inner, depth + 1, stack + (n,)) if inner is not None else None
        return v if v is not None else default(s)
    if n.bl_idname == "NodeGroupInput":
        outer = sock(stack[-1].inputs, l.from_socket.identifier) if stack else None
        v = inp(outer, depth + 1, stack[:-1]) if outer is not None else None
        return v if v is not None else default(s)
    v = out(n, l.from_socket, depth + 1, stack)
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


def arg(n, ident, depth, stack=()):
    s = sock(n.inputs, ident)
    return inp(s, depth, stack) if s is not None else None


def map_range(ty, v, fmin, fmax, tmin, tmax, steps, clamp):
    """`svm_node_map_range` with `MapRangeNode::expand`'s Clamp."""
    import math
    if fmax == fmin: return 0.0
    t = (v - fmin) / (fmax - fmin)
    if ty == "STEPPED": f = math.floor(t * (steps + 1.0)) / steps if steps > 0 else 0.0
    elif ty == "SMOOTHSTEP":
        x = min(max(t, 0.0), 1.0)
        f = x * x * (3.0 - 2.0 * x)
    elif ty == "SMOOTHERSTEP":
        x = min(max(t, 0.0), 1.0)
        f = x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
    else: f = t
    r = tmin + f * (tmax - tmin)
    if not clamp: return r
    lo, hi = (tmax, tmin) if tmin > tmax else (tmin, tmax)
    return min(max(r, lo), hi)


def rgb_to_hsl(c):
    cmax, cmin = max(c), min(c)
    l = min(1.0, (cmax + cmin) / 2.0)
    if cmax == cmin: return [0.0, 0.0, l]
    d = cmax - cmin
    sat = d / (2.0 - cmax - cmin) if l > 0.5 else d / (cmax + cmin)
    if cmax == c[0]: h = (c[1] - c[2]) / d + (6.0 if c[1] < c[2] else 0.0)
    elif cmax == c[1]: h = (c[2] - c[0]) / d + 2.0
    else: h = (c[0] - c[1]) / d + 4.0
    return [h / 6.0, sat, l]


def hsl_to_rgb(hsl):
    h, sat, l = hsl
    nr = min(max(abs(h * 6.0 - 3.0) - 1.0, 0.0), 1.0)
    ng = min(max(2.0 - abs(h * 6.0 - 2.0), 0.0), 1.0)
    nb = min(max(2.0 - abs(h * 6.0 - 4.0), 0.0), 1.0)
    chroma = (1.0 - abs(2.0 * l - 1.0)) * sat
    return [(nr - 0.5) * chroma + l, (ng - 0.5) * chroma + l, (nb - 0.5) * chroma + l]


def rgb_to_hsv(c):
    mx, mn = max(c), min(c)
    d = mx - mn
    if d == 0: h = 0.0
    elif mx == c[0]: h = ((c[1] - c[2]) / d) % 6 / 6
    elif mx == c[1]: h = ((c[2] - c[0]) / d + 2) / 6
    else: h = ((c[0] - c[1]) / d + 4) / 6
    return [h, d / mx if mx != 0 else 0.0, mx]


def hsv_to_rgb(hsv):
    h, sat, v = hsv
    def k(n):
        kk = (n + h * 6) % 6
        return v - v * sat * min(max(min(kk, 4 - kk), 0), 1)
    return [k(5), k(3), k(1)]


def sep_color(mode, c):
    return rgb_to_hsv(c) if mode == "HSV" else rgb_to_hsl(c) if mode == "HSL" else list(c)


def comb_color(mode, c):
    return hsv_to_rgb(c) if mode == "HSV" else hsl_to_rgb(c) if mode == "HSL" else list(c)


def safe_normalize(v):
    import math
    l = math.sqrt(sum(x * x for x in v))
    return [x / l for x in v] if l > 0 else list(v)


def vector_math(op, A, B, C, param):
    """`svm_vector_math`: `(vector, value)`, or None for an unknown op."""
    import math
    sd = lambda x, y: x / y if y != 0 else 0.0
    fmod = lambda x, y: x - math.floor(x / y) * y if y != 0 else 0.0
    dot = sum(A[i] * B[i] for i in range(3))
    z = [0.0, 0.0, 0.0]
    if op == "ADD": return ([A[i] + B[i] for i in range(3)], 0.0)
    if op == "SUBTRACT": return ([A[i] - B[i] for i in range(3)], 0.0)
    if op == "MULTIPLY": return ([A[i] * B[i] for i in range(3)], 0.0)
    if op == "DIVIDE": return ([sd(A[i], B[i]) for i in range(3)], 0.0)
    if op == "CROSS_PRODUCT":
        return ([A[1] * B[2] - A[2] * B[1], A[2] * B[0] - A[0] * B[2], A[0] * B[1] - A[1] * B[0]], 0.0)
    if op == "PROJECT":
        l2 = sum(x * x for x in B)
        return ([B[i] * dot / l2 for i in range(3)] if l2 != 0 else z, 0.0)
    if op == "REFLECT":
        nb = safe_normalize(B)
        d = sum(A[i] * nb[i] for i in range(3))
        return ([A[i] - 2 * d * nb[i] for i in range(3)], 0.0)
    if op == "REFRACT":
        nb = safe_normalize(B)
        d = sum(nb[i] * A[i] for i in range(3))
        k = 1.0 - param * param * (1.0 - d * d)
        return (z if k < 0 else [param * A[i] - (param * d + math.sqrt(k)) * nb[i] for i in range(3)], 0.0)
    if op == "FACEFORWARD":
        return (list(A) if sum(C[i] * B[i] for i in range(3)) < 0 else [-x for x in A], 0.0)
    if op == "MULTIPLY_ADD": return ([A[i] * B[i] + C[i] for i in range(3)], 0.0)
    if op == "DOT_PRODUCT": return (z, dot)
    if op == "DISTANCE": return (z, math.sqrt(sum((A[i] - B[i]) ** 2 for i in range(3))))
    if op == "LENGTH": return (z, math.sqrt(sum(x * x for x in A)))
    if op == "SCALE": return ([x * param for x in A], 0.0)
    if op == "NORMALIZE": return (safe_normalize(A), 0.0)
    if op == "SNAP": return ([math.floor(sd(A[i], B[i])) * B[i] for i in range(3)], 0.0)
    if op == "ROUND": return ([math.floor(x + 0.5) for x in A], 0.0)
    if op == "FLOOR": return ([math.floor(x) for x in A], 0.0)
    if op == "CEIL": return ([math.ceil(x) for x in A], 0.0)
    if op == "MODULO": return ([cmod(A[i], B[i]) for i in range(3)], 0.0)
    if op == "WRAP": return ([fmod(A[i] - C[i], B[i] - C[i]) + C[i] for i in range(3)], 0.0)
    if op == "FRACTION": return ([x - math.floor(x) for x in A], 0.0)
    if op == "ABSOLUTE": return ([abs(x) for x in A], 0.0)
    if op == "POWER": return ([cpow(A[i], B[i]) for i in range(3)], 0.0)
    if op == "SIGN": return ([(x > 0) - (x < 0) for x in A], 0.0)
    if op == "MINIMUM": return ([min(A[i], B[i]) for i in range(3)], 0.0)
    if op == "MAXIMUM": return ([max(A[i], B[i]) for i in range(3)], 0.0)
    if op == "SINE": return ([math.sin(x) for x in A], 0.0)
    if op == "COSINE": return ([math.cos(x) for x in A], 0.0)
    if op == "TANGENT": return ([math.tan(x) for x in A], 0.0)
    return None


def euler_rows(e):
    """`euler_to_transform`'s rows."""
    import math
    cx, cy, cz = (math.cos(x) for x in e)
    sx, sy, sz = (math.sin(x) for x in e)
    return [[cy * cz, sy * sx * cz - cx * sz, sy * cx * cz + sx * sz],
            [cy * sz, sy * sx * sz + cx * cz, sy * cx * sz - sx * cz],
            [-sy, cy * sx, cy * cx]]


def vector_rotate(ty, invert, v, centre, axis, angle, rotation):
    """`svm_node_vector_rotate`."""
    import math
    p = [v[i] - centre[i] for i in range(3)]
    if ty == "EULER_XYZ":
        r = euler_rows(rotation)
        q = ([sum(r[i][k] * p[k] for k in range(3)) for i in range(3)] if not invert
             else [sum(r[k][i] * p[k] for k in range(3)) for i in range(3)])
        return [q[i] + centre[i] for i in range(3)]
    axis = {"X_AXIS": [1.0, 0, 0], "Y_AXIS": [0, 1.0, 0], "Z_AXIS": [0, 0, 1.0]}.get(ty, list(axis))
    l = math.sqrt(sum(x * x for x in axis))
    if l == 0: return list(v)
    x, y, zz = [c / l for c in axis]
    ang = -angle if invert else angle
    c, sn = math.cos(ang), math.sin(ang)
    k = 1.0 - c
    r = [[c + k * x * x, k * x * y - zz * sn, k * x * zz + y * sn],
         [k * x * y + zz * sn, c + k * y * y, k * y * zz - x * sn],
         [k * x * zz - y * sn, k * y * zz + x * sn, c + k * zz * zz]]
    return [sum(r[i][j] * p[j] for j in range(3)) + centre[i] for i in range(3)]


def fresnel_fss(eta):
    """`fresnel_dielectric_Fss`: the hemispherical mean of the Fresnel."""
    if eta < 1.0: return 0.997118 + eta * (0.1014 - eta * (0.965241 + eta * 0.130607))
    return (eta - 1.0) / (4.08567 + 1.00071 * eta)


def facing_mean(blend):
    """Layer Weight ▸ Facing averaged over the hemisphere: `1 − 2/(p + 2)`."""
    if blend == 0.5: p = 1.0
    else:
        b = min(max(blend, 0.0), 1.0 - 1e-5)
        p = 2.0 * b if b < 0.5 else 0.5 / (1.0 - b)
    return 1.0 - 2.0 / (p + 2.0)


def out(n, o, depth, stack=()):
    t = n.bl_idname
    a = lambda i: arg(n, i, depth, stack)
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
    if t == "NodeReroute": return inp(n.inputs[0], depth, stack)
    if t == "ShaderNodeRGBCurve":
        c = a("Color")
        return (c[0], c[1], False) if c else None
    if t == "ShaderNodeMapRange":
        if n.data_type == "FLOAT_VECTOR":
            xs = [a(i) for i in ("Vector", "From_Min_FLOAT3", "From_Max_FLOAT3", "To_Min_FLOAT3", "To_Max_FLOAT3", "Steps_FLOAT3")]
            if None in xs: return None
            r = [map_range(n.interpolation_type, *[x[0][k] for x in xs], n.clamp) for k in range(3)]
            return vec3(r, all(x[2] for x in xs))
        xs = [a(i) for i in ("Value", "From Min", "From Max", "To Min", "To Max", "Steps")]
        if None in xs: return None
        return grey(map_range(n.interpolation_type, *[value(x) for x in xs], n.clamp), all(x[2] for x in xs))
    if t in ("ShaderNodeSeparateColor", "ShaderNodeSeparateRGB", "ShaderNodeSeparateHSV"):
        mode = {"ShaderNodeSeparateRGB": "RGB", "ShaderNodeSeparateHSV": "HSV"}.get(t) or n.mode
        c = a("Image" if t == "ShaderNodeSeparateRGB" else "Color")
        if c is None: return None
        names = {"ShaderNodeSeparateColor": ["Red", "Green", "Blue"]}.get(t, ["R", "G", "B"]) if t != "ShaderNodeSeparateHSV" else ["H", "S", "V"]
        k = names.index(o.name) if o.name in names else 0
        return grey(sep_color(mode, xyz(c))[k], c[2])
    if t in ("ShaderNodeCombineColor", "ShaderNodeCombineRGB", "ShaderNodeCombineHSV"):
        mode = {"ShaderNodeCombineRGB": "RGB", "ShaderNodeCombineHSV": "HSV"}.get(t) or n.mode
        ids = {"ShaderNodeCombineColor": ["Red", "Green", "Blue"]}.get(t, ["R", "G", "B"]) if t != "ShaderNodeCombineHSV" else ["H", "S", "V"]
        xs = [a(i) for i in ids]
        if None in xs: return None
        return val(comb_color(mode, [value(x) for x in xs]) + [1.0], False, all(x[2] for x in xs))
    if t == "ShaderNodeSeparateXYZ":
        v = a("Vector")
        if v is None: return None
        return grey(xyz(v)["XYZ".index(o.name) if o.name in "XYZ" else 0], v[2])
    if t == "ShaderNodeCombineXYZ":
        xs = [a(i) for i in ("X", "Y", "Z")]
        if None in xs: return None
        return vec3([value(x) for x in xs], all(x[2] for x in xs))
    if t == "ShaderNodeVectorMath":
        xs = [a(i) for i in ("Vector", "Vector_001", "Vector_002", "Scale")]
        if None in xs: return None
        r = vector_math(n.operation, xyz(xs[0]), xyz(xs[1]), xyz(xs[2]), value(xs[3]))
        if r is None: return None
        ok = all(x[2] for x in xs)
        return grey(r[1], ok) if o.name == "Value" else vec3(r[0], ok)
    if t == "ShaderNodeVectorRotate":
        xs = [a(i) for i in ("Vector", "Center", "Axis", "Angle", "Rotation")]
        if None in xs: return None
        r = vector_rotate(n.rotation_type, n.invert, xyz(xs[0]), xyz(xs[1]), xyz(xs[2]), value(xs[3]), xyz(xs[4]))
        return vec3(r, all(x[2] for x in xs))
    if t == "ShaderNodeNormal":
        d = safe_normalize(list(n.outputs["Normal"].default_value))
        if o.name != "Dot": return vec3(d, True)
        v = a("Normal")
        if v is None: return None
        nv = safe_normalize(xyz(v))
        return grey(sum(d[i] * nv[i] for i in range(3)), v[2])
    if t == "ShaderNodeFresnel":
        ior = a("IOR")
        return grey(fresnel_fss(max(value(ior) if ior else 1.45, 1e-5)), False)
    if t == "ShaderNodeLayerWeight":
        b = a("Blend")
        bl = value(b) if b else 0.5
        return grey(facing_mean(bl) if o.name == "Facing" else fresnel_fss(1.0 / max(1.0 - bl, 1e-5)), False)
    if t in ("ShaderNodeVertexColor", "ShaderNodeAttribute"):
        return grey(1.0, False) if o.name in ("Alpha", "Fac") else ([1.0] * 4, False, False)
    # A Mapping is affine, so the mean maps to the mean (`eval.rs`).
    if t == "ShaderNodeMapping":
        v = a("Vector")
        if v is None: return None
        sc, lo = a("Scale"), a("Location")
        return grey(value(v) * (value(sc) if sc else 1.0) + (value(lo) if lo else 0.0), False)
    if t in ("ShaderNodeUVMap", "ShaderNodeNewGeometry"): return grey(0.5, False)
    if t == "ShaderNodeLightPath":
        return grey(1.0 if o.name in ("Is Camera Ray", "Ray Length") else 0.0, True)
    if t == "ShaderNodeAmbientOcclusion":
        if o.name == "AO": return grey(1.0, False)
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
