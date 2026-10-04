//! Which host is which: every `BHost` implementation, in one place.
//!
//! A host is a module and a module cannot implement a trait, so each one gets a
//! zero-sized type here. That is the only reason these exist; they carry no
//! state and are never constructed.
//!
//! The macro is the point. Adding a host is one line, and the compiler then
//! requires its folder to have a `SLUG`, a `SPEC` and a `REGIONS` — the three
//! things that were a convention held together by nothing but an exhaustive
//! match on two of them.

use super::{BHost, BHostSpec, BHostType, BRegionSpec};

macro_rules! hosts {
    ($($variant:ident => $module:ident,)*) => {
        $(
            #[allow(non_camel_case_types)]
            pub struct $variant;

            impl BHost for $variant {
                const KIND: BHostType = BHostType::$variant;
                const SLUG: &'static str = super::$module::SLUG;
                fn spec() -> &'static BHostSpec {
                    &super::$module::SPEC
                }
                fn regions() -> &'static [BRegionSpec] {
                    super::$module::REGIONS
                }
            }
        )*
    };
}

hosts! {
    Topbar => topbar,
    View3D => view3d,
    Outliner => outliner,
    Buts => buts,
    Time => time,
    Statusbar => statusbar,
    File => file,
    Image => image,
    Info => info,
    Seq => seq,
    Text => text,
    Action => action,
    Nla => nla,
    Node => node,
    Logic => logic,
    Console => console,
    Uv => uv,
    Compositor => compositor,
    Shader => shader,
    GeoNode => geonode,
    Clip => clip,
    Drivers => drivers,
    Asset => asset,
    Spreadsheet => spreadsheet,
    Ipo => ipo,
    Prefs => pref,
    Activity => activity,
}
