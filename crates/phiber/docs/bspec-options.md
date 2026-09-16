# BSpec Abstraction — Two Options

> Comparing two approaches for unifying the store hierarchy.  
> Examples in TypeScript for clarity.

## Current Decision

We are taking the dumb-relational approach from **Option B**, but with per-level `B...Def` traits for default sets and a generic `BMap<K, S>` for lookups.

- One `BSpec` trait: `idx()` + `items()`.
- One `BMap<K, S>` per level with `spec_for(key)`.
- Each `B...Def` trait has one `default()` method and optional `with_*` variants.
- Modals stay as a separate `BModalType` level, but `BModalSpec` is space-like and can be rendered through `BSpaceDef`.

See [`bspec.md`](bspec.md) for the canonical spec-of-specs description and `BMap` details.

---

## Option A - Two Traits (Type-Safe Tree)

Each level keeps its own marker type via generics.
Two traits: `BSpecItem` (leaf-capable) and `BSpec` (container).

```typescript
// --- Marker types (per level) ---
enum BSpaceType { Layout, Modeling, Scripting, Rendering }
enum BZoneType  { Left, Center, Right }
enum BHostType  { View3D, Outliner, Properties, /* ... */ }
enum BRegionType { Header, Center, Footer }
enum BPaneType  { Left, Center, Right }
enum BPartType  { Menubar, Modebar, HostBar, Toolbar, /* ... */ }

// --- BSpecItem: every node has a key + label + visible ---
interface BSpecItem<K> {
  key:   K;
  label: string;
  visible: boolean;
}

// --- BSpec: container nodes also have items ---
interface BSpec<K, C extends BSpecItem<any>> extends BSpecItem<K> {
  items: C[];
}

// --- Structs ---
interface SpaceSpec extends BSpec<BSpaceType, BArea> {}
interface BArea     extends BSpec<BZoneType, HostSpec> {}
interface HostSpec  extends BSpec<BHostType, RegionSpec> {}
interface RegionSpec extends BSpec<BRegionType, PaneSpec> {}
interface PaneSpec  extends BSpec<BPaneType, PartSpec> {}
interface PartSpec  extends BSpecItem<BPartType> {}  // leaf - no items

// --- The tree ---
const layout: SpaceSpec = {
  key: BSpaceType.Layout,
  label: "Layout",
  visible: true,
  items: [
    {
      key: BZoneType.Left,
      label: "Left",
      visible: true,
      items: [
        {
          key: BHostType.Outliner,
          label: "Outliner",
          visible: true,
          items: [
            {
              key: BRegionType.Header,
              label: "Header",
              visible: true,
              items: [
                {
                  key: BPaneType.Center,
                  label: "Center",
                  visible: true,
                  items: [
                    { key: BPartType.Menubar, label: "Menubar", visible: true },
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
  ],
};

// --- Render walk needs a wrapper enum for polymorphism ---
type SpecNode =
  | { level: "space";  spec: SpaceSpec }
  | { level: "area";   spec: BArea }
  | { level: "host";   spec: HostSpec }
  | { level: "region"; spec: RegionSpec }
  | { level: "pane";   spec: PaneSpec }
  | { level: "part";   spec: PartSpec };

function renderTree(node: SpecNode): string {
  const { spec } = node;
  if (!spec.visible) return "";

  if (node.level === "part") {
    return renderPart(spec.key as BPartType);
  }

  // Must map each level's items to SpecNode - boilerplate per level
  const children = (spec as any).items.map((c: any) => wrapInSpecNode(c));
  return `Container(${spec.label}: [${children.map(renderTree).join(", ")}])`;
}
```

### Pros
- Compile-time type safety - `HostSpec.items` can only contain `RegionSpec`
- Each level's key is its own enum - no mega-enum
- Direct field access on concrete structs

### Cons
- Two traits (`BSpecItem` + `BSpec`)
- Needs `SpecNode` wrapper enum for generic traversal
- Render walk still needs per-level wrapping boilerplate
- Adding a level = new generic param + new SpecNode variant + new wrap fn

---

## Option B - One Trait (Dumb Relational Tree)

One trait. One unified key enum. The store is purely key→items.
Reverse-engineer the tree by walking keys.

```typescript
// --- Marker types stay the same ---
enum BSpaceType { Layout, Modeling, Scripting, Rendering }
enum BZoneType  { Left, Center, Right }
enum BHostType  { View3D, Outliner, Properties, /* ... */ }
enum BRegionType { Header, Center, Footer }
enum BPaneType  { Left, Center, Right }
enum BPartType  { Menubar, Modebar, HostBar, Toolbar, /* ... */ }

// --- One unified key ---
type BKey =
  | { level: "space";  variant: BSpaceType }
  | { level: "zone";   variant: BZoneType }
  | { level: "host";   variant: BHostType }
  | { level: "region"; variant: BRegionType }
  | { level: "pane";   variant: BPaneType }
  | { level: "part";   variant: BPartType };

// --- One trait ---
interface BSpec {
  key:   BKey;
  items: BSpec[];   // children - empty for leaves
}

// --- Structs carry their own fields, trait carries only relations ---
interface SpaceSpec  extends BSpec { columns: number[] }
interface BArea      extends BSpec { fraction: number }
interface HostSpec   extends BSpec { icon: string }
interface RegionSpec extends BSpec { units: number | null }
interface PaneSpec   extends BSpec { dock: string; width: number | null }
interface PartSpec   extends BSpec {}  // leaf - items: []

// --- The tree ---
const layout: SpaceSpec = {
  key:   { level: "space", variant: BSpaceType.Layout },
  items: [
    {
      key: { level: "zone", variant: BZoneType.Left },
      items: [
        {
          key: { level: "host", variant: BHostType.Outliner },
          items: [
            {
              key: { level: "region", variant: BRegionType.Header },
              items: [
                {
                  key: { level: "pane", variant: BPaneType.Center },
                  items: [
                    {
                      key: { level: "part", variant: BPartType.Menubar },
                      items: [],  // leaf
                    },
                  ],
                },
              ],
            },
          ],
        },
      ],
    },
  ],
  columns: [0.25, 0.5, 0.25],
};

// --- Generic render walk - no wrapper, no per-level code ---
function renderTree(node: BSpec): string {
  // The key tells you the level + variant
  const { key, items } = node;

  if (items.length === 0) {
    // Leaf - dispatch on key
    if (key.level === "part") {
      return renderPart(key.variant);
    }
    return "";
  }

  // Container - just recurse, no level-specific logic
  const children = items.map(renderTree);
  return `Container(${key.level}:${key.variant} → [${children.join(", ")}])`;
}

// --- Reverse-engineer: given any key, find the node ---
function findByKey(root: BSpec, target: BKey): BSpec | null {
  if (root.key.level === target.level && root.key.variant === target.variant) {
    return root;
  }
  for (const child of root.items) {
    const found = findByKey(child, target);
    if (found) return found;
  }
  return null;
}
```

### Pros
- **One trait** - `BSpec` with `key()` + `items()`
- **No wrapper enum** - `items: BSpec[]` is already polymorphic
- **Generic render walk** - one function, zero per-level boilerplate
- **Reverse-engineer the tree** from any key via `findByKey()`
- Adding a level = add a `BKey` variant + impl `BSpec` - no wrapper changes
- Store is maximally dumb - just keys and items

### Cons
- Loses compile-time type safety on `items` (any `BSpec` can contain any `BSpec`)
- Level-specific fields require downcast/narrowing via `key.level`
- `BKey` is a mega-enum (but it's just a marker - not behavioral)

---

## Side-by-Side

| | Option A (Two Traits) | Option B (One Trait) |
|---|---|---|
| **Traits** | `BSpecItem<K>` + `BSpec<K,C>` | `BSpec` |
| **Key type** | Per-level generic `K` | Unified `BKey` enum |
| **Items type** | `C[]` (concrete child type) | `BSpec[]` (polymorphic) |
| **Type safety** | ✅ Compile-time enforced | ❌ Runtime via key check |
| **Wrapper needed** | `SpecNode` enum for traversal | ❌ Not needed |
| **Render walk** | Per-level wrapping + recurse | One generic function |
| **Reverse-engineer** | Needs SpecNode mapping | `findByKey(root, key)` |
| **Add new level** | New generic + SpecNode variant + wrap fn | New BKey variant + impl |
| **Store dumbness** | Moderate (trait has label/visible) | Maximum (key + items only) |
| **Field access** | Direct on concrete struct | Narrow via `key.level` |

---

## The Core Question

**Do you want the compiler to enforce that `HostSpec.items` only contains `RegionSpec`?**

- **Yes** → Option A. You get type safety but pay for it with a wrapper enum and per-level boilerplate.
- **No, the store is just a dumb key→items tree** → Option B. One trait, one generic walk, reverse-engineer anything from keys. The hierarchy is implicit in the key→items association.
