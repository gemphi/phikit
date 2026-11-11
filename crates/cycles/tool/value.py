"""What the table tools share: reading Cycles' files, its enums, and working
out the number a C expression such as `DENOISER_PASS_ALBEDO | (1 << 2)`
stands for.
"""
import glob
import io
import os
import re

# Constants Cycles takes from the C library or defines as macros.
KNOWN = {
    'FLT_MAX': 3.4028234663852886e38,
    'INT_MAX': 2147483647,
    'M_PI_F': 3.141592653589793,
    'M_PI_2_F': 1.5707963267948966,
    'M_PI_4_F': 0.7853981633974483,
    'M_2PI_F': 6.283185307179586,
    'M_SQRT2_F': 1.4142135623730951,
    'true': 1,
    'false': 0,
}


def read(root, rel):
    return io.open(os.path.join(root, rel), encoding='utf-8').read()


def strip_comment(text):
    text = re.sub(r'/\*.*?\*/', '', text, flags=re.S)
    return re.sub(r'//[^\n]*', '', text)


class Names:
    """Every enumerator and integer constant of Cycles' headers, by name."""

    def __init__(self, root):
        self.scoped = {}
        self.bare = {}
        pattern = ['kernel/types.h', 'kernel/svm/types.h', 'device/denoise.h', 'scene/*.h', 'util/*.h', 'graph/*.h', 'bvh/*.h']
        for item in pattern:
            for path in sorted(glob.glob(os.path.join(root, item))):
                self.scan(strip_comment(io.open(path, encoding='utf-8').read()))

    def scan(self, text):
        # A define that stands for a number, such as LIGHT_LINK_MASK_ALL.
        for name, given in re.findall(r'^#\s*define (\w+) ([^\n\\]+)$', text, flags=re.M):
            value = self.number(given)
            if value is not None:
                self.bare.setdefault(name, set()).add(value)
        for found in re.finditer(r'static const (?:int|uint|uint32_t) (\w+) = ([^;]+);', text):
            value = self.number(found.group(2))
            if value is not None:
                self.bare.setdefault(found.group(1), set()).add(value)
        for found in re.finditer(r'\benum(?:\s+class)?\s*(\w*)\s*(?::\s*\w+\s*)?\{(.*?)\}', text, flags=re.S):
            owner = re.findall(r'\b(?:class|struct)\s+(\w+)[^;{]*\{', text[:found.start()])
            owner = owner[-1] if owner else ''
            value = -1
            for item in found.group(2).split(','):
                item = item.strip()
                if not item or not re.match(r'^\w+(\s*=.*)?$', item, flags=re.S):
                    continue
                name, _, given = item.partition('=')
                name = name.strip()
                value = self.number(given, owner) if given.strip() else (None if value is None else value + 1)
                if value is None:
                    continue
                self.bare.setdefault(name, set()).add(value)
                self.scoped[(owner, name)] = value
                if found.group(1):
                    self.scoped[(found.group(1), name)] = value

    def lookup(self, name, owner=''):
        scope, _, last = name.rpartition('::')
        for key in ((scope, last), (owner, last)):
            if key[0] and key in self.scoped:
                return self.scoped[key]
        if last in KNOWN:
            return KNOWN[last]
        found = self.bare.get(last, set())
        return next(iter(found)) if len(found) == 1 else None

    def number(self, text, owner=''):
        """The number `text` stands for, or None when it is not one."""
        # An expression may run over several lines, as PRIMITIVE_ALL does.
        text = ' '.join(text.split())
        text = re.sub(r'\(\s*(?:u?int\d*_t|uint|int|float)\s*\)', '', text)
        text = re.sub(r'\b(?:static_cast<\w+>|u?int\d+_t|int|uint|float)\s*\(', '(', text)
        text = re.sub(r'(?<![\w.])(\d+\.?\d*(?:e[+-]?\d+)?|\.\d+)[fFuU]\b', r'\1', text)
        text = re.sub(r'\b(\d+)\.(?!\d)', r'\1.0', text)
        failed = []

        def swap(found):
            value = self.lookup(found.group(0), owner)
            if value is None:
                failed.append(found.group(0))
                return '0'
            return repr(value)

        text = re.sub(r'(?<![\w.])[A-Za-z_][\w:]*', swap, text)
        if failed or not text or not re.fullmatch(r'[0-9a-fA-FxX.+\-*/()<>|&~ e]+', text):
            return None
        try:
            return eval(text, {'__builtins__': {}})  # digits and operators only, checked above
        except Exception:
            return None


def show(value):
    """A number as the tables write it."""
    if value is None:
        return ''
    if isinstance(value, float) and value != int(value):
        return repr(value)
    if isinstance(value, float) and abs(value) >= 1e15:
        return repr(value)
    return str(int(value))
