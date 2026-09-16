# BSpec - Spec of Specs

A `BSpec` is a spec of specs. Each spec has an `idx` and `items`.

```rust
pub trait BSpec {
    fn idx(&self) -> BSpecType;
    fn items(&self) -> Vec<&dyn BSpec>;
}
```

8 spec types. The rest is data.

---

## BSpecIdx — the markers

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BSpecType {
    BSpace(BSpaceType),
    BZone(BZoneType),
    BArea(BAreaType),
    Host(BHostType),
    BRegion(BRegionType),
    BPane(BPaneType),
    Part(BPartType),
    Modal(BModalType),
}
```

`Modal` is a separate level because modals can be triggered from any space. They are still *space-like*: a `BModalSpec` implements the same `BSpec` trait and can be rendered through the same `BSpaceDef` pipeline as a workspace space.

## BMap and `spec_for`

Every level has one `BMap` from its key type to its spec type, and one `spec_for(key)` lookup.

```rust
pub struct BMap<K, S> {
    pub map: HashMap<K, S>,
}

impl<K: Eq + Hash + Copy, S: Clone> BMap<K, S> {
    pub fn new(pairs: &[(K, S)]) -> Self {
        Self { map: pairs.iter().copied().collect() }
    }

    pub fn spec_for(&self, key: K) -> Option<S> {
        self.map.get(&key).copied()
    }
}
```

Each `set.rs` builds its `BMap` from `def::ALL` and the corresponding spec constructors:

```rust
pub fn host_spec(host: BHostType) -> BHostSpec {
    HOST_MAP.spec_for(host).unwrap()
}
```

## Default sets — one `default()` plus `with_*` variants

Each level defines a `B...Def` trait with a single `default()` method. Optional variants keep the `with_*` prefix.

```rust
pub trait BRegionDef {
    fn default() -> Vec<BRegionSpec>;
    fn with_footer() -> Vec<BRegionSpec>;
}

pub trait BPaneDef {
    fn default() -> Vec<BPaneSpec>;
    fn with_header() -> Vec<BPaneSpec>;
    fn with_footer() -> Vec<BPaneSpec>;
}
```

No more `default_regions`, `default_panes`, or repeated level names inside the method name. The module already tells you the level.

## BSpec impls

### `store/spaces/mod.rs`

```rust
impl BSpec for BSpaceSpec {
    fn idx(&self) -> BSpecType { BSpecType::BSpace(self.space_type) }
    fn items(&self) -> Vec<&dyn BSpec> {
        self.areas.iter().map(|a| a as &dyn BSpec).collect()
    }
}
```

### `store/zones/mod.rs`

```rust
impl BSpec for BZoneSpec {
    fn idx(&self) -> BSpecType { BSpecType::BZone(self.zone) }
    fn items(&self) -> Vec<&dyn BSpec> { Vec::new() }
}
```

### `store/areas/mod.rs`

```rust
impl BSpec for BArea {
    fn idx(&self) -> BSpecType { BSpecType::BArea(self.idx) }
    fn items(&self) -> Vec<&dyn BSpec> {
        self.hosts.iter().map(|h| h as &dyn BSpec).collect()
    }
}
```

### `store/hosts/mod.rs`

```rust
impl BSpec for BHostSpec {
    fn idx(&self) -> BSpecType { BSpecType::Host(self.idx) }
    fn items(&self) -> Vec<&dyn BSpec> {
        self.regions.iter().map(|r| r as &dyn BSpec).collect()
    }
}
```

### `store/regions/mod.rs`

```rust
impl BSpec for BRegionSpec {
    fn idx(&self) -> BSpecType { BSpecType::BRegion(self.region) }
    fn items(&self) -> Vec<&dyn BSpec> {
        self.panes.iter().map(|p| p as &dyn BSpec).collect()
    }
}
```

### `store/panes/mod.rs`

```rust
impl BSpec for BPaneSpec {
    fn idx(&self) -> BSpecType { BSpecType::BPane(self.pane) }
    fn items(&self) -> Vec<&dyn BSpec> {
        self.parts.iter().map(|p| p as &dyn BSpec).collect()
    }
}
```

### `store/parts/mod.rs`

```rust
impl BSpec for BPart {
    fn idx(&self) -> BSpecType { BSpecType::Part(self.part) }
    fn items(&self) -> Vec<&dyn BSpec> { Vec::new() }
}
```

### `store/modals/mod.rs`

```rust
impl BSpec for BModalSpec {
    fn idx(&self) -> BSpecType { BSpecType::Modal(self.modal_type) }
    fn items(&self) -> Vec<&dyn BSpec> { Vec::new() }
}
```
