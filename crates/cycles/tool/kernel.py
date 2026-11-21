"""Write data/kernel.tsv and data/array.tsv from Cycles' own source.

  python tool/kernel.py <path to intern/cycles>

data/kernel.tsv is the constant data every kernel reads: the structs of
kernel/data_template.h (background, bvh, film, integrator, image and the
shader node usage counts of kernel/svm/node_types_template.h), and the
plain structs of `KernelData` in kernel/types.h: the camera, baking, the
lookup tables' offsets and the scene's time.
Each member gets its place, in 32-bit words, in a struct that starts on a
16-byte boundary, as Cycles aligns them.

data/array.tsv is the list of arrays a scene is flattened into
(kernel/data_arrays.h).

data/record.tsv is the structs some of those arrays hold one of for each
element, such as `KernelObject` (kernel/types.h), laid out the same way:
a member takes whole words, a 64-bit one two.
"""
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from value import read, strip_comment  # noqa: E402

CYCLES = sys.argv[1]
HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

# Words a member takes, and the word boundary it starts on.
SIZE = {'int': (1, 1), 'uint': (1, 1), 'float': (1, 1), 'float2': (2, 1), 'float4': (4, 4), 'Transform': (12, 4), 'ProjectionTransform': (16, 4)}
# The plain structs of KernelData, and the member each is in `KernelData`.
PLAIN = [('KernelCamera', 'cam'), ('KernelBake', 'bake'), ('KernelTables', 'tables'), ('KernelSceneTime', 'scene_time')]
# A member of a plain struct: its type, then one name or several.
MEMBER = r'^\s*(uint|int|float|float4|Transform|ProjectionTransform) ([\w, ]+);'


# The structs an array holds one of for each element.
RECORD = ['KernelObject', 'AttributeMap', 'KernelShader']
RECORD_SIZE = {'int': (1, 1), 'uint': (1, 1), 'float': (1, 1), 'uint16_t': (1, 1), 'uint8_t': (1, 1), 'uint64_t': (2, 2), 'Transform': (12, 4)}
RECORD_MEMBER = r'^\s*(uint64_t|uint16_t|uint8_t|uint|int|float|Transform) ((?:\w+(?:\[\d+\])?(?:, )?)+);'


def record_rows():
    text = strip_comment(read(CYCLES, 'kernel/types.h'))
    rows = []
    for struct in RECORD:
        body = re.search(r'struct ' + struct + r' \{(.*?)\n\};', text, flags=re.S).group(1)
        offset, widest, member = 0, 1, []
        for kind, names in re.findall(RECORD_MEMBER, body, flags=re.M):
            for item in names.split(', '):
                name, _, count = item.partition('[')
                size, align = RECORD_SIZE[kind]
                size *= int(count.rstrip(']') or 1)
                offset = (offset + align - 1) // align * align
                member.append((name, kind, offset, size))
                offset += size
                widest = max(widest, align)
        # Every member of the struct is one this reads.
        assert len(member) == sum(1 + names.count(',') for names in re.findall(r'^\s*[\w:]+ ([\w, \[\]]+);', body, flags=re.M)), struct
        rows.append(('struct', struct, (offset + widest - 1) // widest * widest))
        rows.extend(('member', struct, name, kind, word, words) for name, kind, word, words in member)
    return rows


def place(member):
    """(type, name, specialize) rows with word offsets, and the struct's size."""
    offset, out = 0, []
    for kind, name, specialize in member:
        size, align = SIZE[kind]
        offset = (offset + align - 1) // align * align
        out.append((kind, name, offset, size, specialize))
        offset += size
    return out, offset


def template_structs():
    text = strip_comment(read(CYCLES, 'kernel/data_template.h'))
    usage = strip_comment(read(CYCLES, 'kernel/svm/node_types_template.h'))
    node = []
    for macro, name in re.findall(r'^(SHADER_NODE_TYPE(?:_DERIVATIVE)?)\((\w+)\)', usage, flags=re.M):
        node.append(name)
        if macro.endswith('DERIVATIVE'):
            node.append(name + '_DERIVATIVE')
    out = []
    for struct, parent, body in re.findall(r'KERNEL_STRUCT_BEGIN\((Kernel\w+), (\w+)\)(.*?)KERNEL_STRUCT_END', text, flags=re.S):
        member, keep = [], '1'
        for line in body.splitlines():
            if 'KERNEL_STRUCT_MEMBER_DONT_SPECIALIZE' in line:
                keep = '0'
            found = re.match(r'\s*KERNEL_STRUCT_MEMBER\(\w+, (\w+), (\w+)\)', line)
            if found:
                member.append((found.group(1), found.group(2), keep))
                keep = '1'
        if parent == 'svm_usage':
            member = [('int', name, '1') for name in node]
        out.append((struct, parent, member))
    return out


def plain_structs():
    text = strip_comment(read(CYCLES, 'kernel/types.h'))
    data = re.search(r'struct ccl_align\(16\) KernelData \{(.*?)KernelCamera', text, flags=re.S).group(1)
    out = [('KernelData', 'data', [(kind, name, '0') for kind, name in re.findall(r'^\s*(uint|int|float) (\w+);', data, flags=re.M)])]
    for struct, parent in PLAIN:
        body = re.search(r'struct ' + struct + r' \{(.*?)\n\};', text, flags=re.S).group(1)
        member = [(kind, name.strip(), '0') for kind, names in re.findall(MEMBER, body, flags=re.M) for name in names.split(',')]
        out.append((struct, parent, member))
    return out


def write(name, head, rows):
    out = os.path.join(HERE, 'data', name)
    with open(out, 'w', encoding='utf-8', newline='\n') as file:
        file.write('\n'.join(head + ['\t'.join(str(cell) for cell in row) for row in rows]) + '\n')
    print(out, len(rows), 'rows')


def main():
    rows, start = [], 0
    for struct, parent, member in plain_structs() + template_structs():
        placed, size = place(member)
        # A struct is aligned to 16 bytes, so its size is a multiple of 16.
        size = (size + 3) // 4 * 4
        rows.append(('struct', parent, struct, start, size))
        rows.extend(('member', parent, name, kind, start + offset, words, specialize) for kind, name, offset, words, specialize in placed)
        start += size
    head = [
        '# Generated by tool/kernel.py from Cycles: kernel/data_template.h,',
        '# kernel/svm/node_types_template.h and the plain structs of KernelData',
        '# in kernel/types.h. Do not edit by hand. Places are 32-bit',
        '# words from the start of the data; a struct starts on a 16-byte',
        '# boundary.',
        '#',
        '# struct	parent	C++ struct	first word	words',
        '# member	parent	name	type	word	words	may become a constant when a kernel is specialized',
    ]
    write('kernel.tsv', head, rows)

    arrays = strip_comment(read(CYCLES, 'kernel/data_arrays.h'))
    rows = [(name, kind, '1' if macro.endswith('WRITABLE') else '0') for macro, kind, name in re.findall(r'^(KERNEL_DATA_ARRAY(?:_WRITABLE)?)\((\w+), (\w+)\)', arrays, flags=re.M)]
    head = [
        '# Generated by tool/kernel.py from Cycles: kernel/data_arrays.h. Do not',
        '# edit by hand. The arrays a scene is flattened into, in Cycles\' order.',
        '#',
        '# name	element type	the kernel may write it',
    ]
    write('array.tsv', head, rows)
    head = [
        '# Generated by tool/kernel.py from Cycles: the structs of kernel/types.h',
        '# that a device array holds one of for each element. Do not edit by',
        '# hand. Places are 32-bit words from the start of an element.',
        '#',
        '# struct	C++ struct	words',
        '# member	C++ struct	name	C++ type	word	words',
    ]
    write('record.tsv', head, record_rows())


main()
