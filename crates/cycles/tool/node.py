"""Write data/node.tsv from Cycles' own source, so no socket is typed in.

  python tool/node.py <path to intern/cycles>

Reads every `NODE_DEFINE` and `NODE_ABSTRACT_DEFINE` block of scene/*.cpp and
device/denoise.cpp: the node's name and base, its sockets with their labels
and defaults, and the enums a socket chooses from.
"""
import glob
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from value import Names, read, show, strip_comment  # noqa: E402

CYCLES = sys.argv[1]
HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
NAMES = Names(CYCLES)
LINKABLE = int(NAMES.scoped[('SocketType', 'LINKABLE')])


def split_top(text):
    """`text` cut at the commas that are not inside brackets or quotes."""
    out, depth, start, quote = [], 0, 0, False
    for index, char in enumerate(text):
        if char == '"' and text[index - 1] != '\\':
            quote = not quote
        if quote:
            continue
        depth += char in '(<{['
        depth -= char in ')>}]'
        if char == ',' and depth == 0:
            out.append(text[start:index].strip())
            start = index + 1
    out.append(text[start:].strip())
    return out


def call_list(body, pattern):
    """Every `NAME(args)` in `body` whose name matches: (name, args)."""
    out = []
    for found in re.finditer(pattern + r'\(', body):
        depth, index = 1, found.end()
        while depth:
            depth += body[index] == '('
            depth -= body[index] == ')'
            index += 1
        out.append((found.group(0)[:-1], split_top(body[found.end():index - 1])))
    return out


def unquote(text):
    return text.strip().strip('"')


def expand(text):
    """Socket macros such as TEXTURE_MAPPING_DEFINE, written out in place."""
    macro = {}
    for found in re.finditer(r'#define (\w+)\((\w*)\)((?:[^\n]*\\\n)+[^\n]*)', text):
        if 'SOCKET_' in found.group(3):
            macro[found.group(1)] = found.group(3).replace('\\\n', '\n')
    text = re.sub(r'#define \w+\(\w*\)(?:[^\n]*\\\n)+[^\n]*', '', text)
    for name, body in macro.items():
        text = re.sub(r'\b' + name + r'\(\w*\);', lambda _: body, text)
    return text


def default_of(kind, rest):
    """The raw default of a socket and the number it stands for."""
    raw = rest[1] if kind == 'enum' and len(rest) > 1 else (rest[0] if rest else '')
    raw = '' if kind in ('node', 'node_array', 'closure') else raw
    vector = re.fullmatch(r'make_float[234]\((.*)\)', raw)
    named = {'zero_float3()': '0 0 0', 'one_float3()': '1 1 1', 'zero_float2()': '0 0'}
    if raw in named:
        return raw, named[raw]
    if vector:
        part = [show(NAMES.number(item)) for item in split_top(vector.group(1))]
        return raw, ' '.join(part) if all(part) else ''
    return raw, show(NAMES.number(raw))


# The enum tables Cycles builds in a function of their own, such as
# `Pass::get_type_enum`: (class, getter) to its (key, C++ name) entries.
GETTER = {}


def read_getters(text):
    for owner, getter, body in re.findall(r'const NodeEnum \*(\w+)::get_(\w+)_enum\(\)\s*\{(.*?)\n\}', text, flags=re.S):
        GETTER[(owner, getter)] = re.findall(r'\w+\.insert\(\s*"([^"]*)"\s*,\s*(.+?)\);', body)


def block_rows(body, owner):
    add = call_list(body, r'NodeType::add')
    if not add:
        return []
    arg = add[0][1]
    node = unquote(arg[0])
    kind = arg[2].replace('NodeType::', '').lower() if len(arg) > 2 else 'none'
    base = re.sub(r'::get_node(_base)?_type\(\)', '', arg[3]) if len(arg) > 3 else ''
    rows = [('node', node, kind, base)]
    for table, key, name in re.findall(r'(\w+)\.insert\(\s*"([^"]*)"\s*,\s*([^)]+)\)', body):
        rows.append(('enum', node, table, key, name.strip(), show(NAMES.number(name))))
    for table, other, getter in re.findall(r'const NodeEnum \*(\w+) = (?:(\w+)::)?get_(\w+)_enum\(\);', body):
        for key, name in GETTER.get((other or owner, getter), []):
            rows.append(('enum', node, table, key, name.strip(), show(NAMES.number(name))))
    for macro, arg in call_list(body, r'SOCKET_(?!OFFSETOF|SIZEOF|DEFINE)[A-Z_0-9]+'):
        direction = {'IN': 'in', 'OUT': 'out'}.get(macro.split('_')[1], '-')
        kind = macro.replace('SOCKET_', '').replace('IN_', '').replace('OUT_', '').lower()
        name, label, rest = arg[0], unquote(arg[1]) if len(arg) > 1 else '', arg[2:]
        table = rest[0].lstrip('*') if kind == 'enum' and rest else ''
        target = re.sub(r'::get_node(_base)?_type\(\)', '', rest[0]) if kind in ('node', 'node_array') and rest else ''
        raw, number = default_of(kind, rest) if direction != 'out' else ('', '')
        # `SocketType::LINK_NORMAL` and the like, after the default; an
        # input is linkable besides.
        given = [item for item in rest[1:] if 'SocketType::' in item]
        flag = (NAMES.number(given[0]) or 0 if given else 0) | (LINKABLE if direction == 'in' else 0)
        rows.append(('socket', node, direction, kind, name, label, raw, number, table or target, str(int(flag))))
    return rows


def convert_rows(text):
    """The node types `ConvertNode::get_node_types` makes: one from each type to each."""
    body = re.search(r'ConvertNode::get_node_types\(\)\)\[.*?\n\}', text, flags=re.S).group(0)
    kind = re.findall(r'SocketType::([A-Z0-9_]+)', re.search(r'types\[num_types\] = \{(.*?)\};', body, flags=re.S).group(1))
    rows = []
    for source in (item.lower() for item in kind):
        for target in (item.lower() for item in kind):
            node = 'convert_%s_to_%s' % (source, target)
            rows.append(('class', node, 'ConvertNode', 'scene/shader_nodes.cpp'))
            rows.append(('node', node, 'shader', ''))
            rows.append(('socket', node, 'in', source, 'value_' + source, 'value_' + source, '', '', '', str(LINKABLE)))
            rows.append(('socket', node, 'out', target, 'value_' + target, 'value_' + target, '', '', '', '0'))
    return rows


def main():
    rows = []
    path_list = sorted(glob.glob(os.path.join(CYCLES, 'scene', '*.cpp'))) + [os.path.join(CYCLES, 'device', 'denoise.cpp')]
    for path in path_list:
        read_getters(strip_comment(read(CYCLES, os.path.relpath(path, CYCLES))))
    for path in path_list:
        text = expand(strip_comment(read(CYCLES, os.path.relpath(path, CYCLES))))
        text = re.sub(r'#ifdef WITH_CYCLES_DEBUG.*?#endif', '', text, flags=re.S)
        for found in re.finditer(r'\nNODE_(?:ABSTRACT_)?DEFINE\((\w+)\)\s*\{(.*?)\n\}', text, flags=re.S):
            block = block_rows(found.group(2), found.group(1))
            if block:
                rows.append(('class', block[0][1], found.group(1), os.path.relpath(path, CYCLES).replace(os.sep, '/')))
                rows.extend(block)
        if path.endswith('shader_nodes.cpp'):
            rows.extend(convert_rows(text))
    head = [
        '# Generated by tool/node.py from Cycles: the NODE_DEFINE blocks of scene/*.cpp',
        '# and device/denoise.cpp, and the node types ConvertNode::get_node_types makes.',
        '# Do not edit by hand.',
        '#',
        '# class	node	C++ class	file',
        '# node	node	kind	base class',
        '# enum	node	table	key	C++ name	number',
        '# socket	node	in|out|-	type	name	label	default as written	default as numbers	enum table or node type	SocketType flags',
    ]
    out = os.path.join(HERE, 'data', 'node.tsv')
    with open(out, 'w', encoding='utf-8', newline='\n') as file:
        file.write('\n'.join(head + ['\t'.join(row) for row in rows]) + '\n')
    count = {kind: sum(1 for row in rows if row[0] == kind) for kind in ('node', 'socket', 'enum')}
    blank = sum(1 for row in rows if row[0] == 'enum' and not row[5])
    print(out, count, 'enum numbers not found:', blank)


main()
