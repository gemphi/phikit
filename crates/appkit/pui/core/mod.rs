//! What everything is: something drawn, a view, an event, the painter a
//! view draws with. `PDraw` and `PView` are defined here;
//! Zed's gpui is transformed into this folder, a part at a time.
//!
//! So far: an event, the painter and the sheet of marks it keeps, `PDraw`,
//! `PView`, the mode an app is used in, an asset and an icon; and how a
//! surface is lit (`light`) and what glass does (`PGlass`). The
//! system window, the renderer, the executor and animation are not here:
//! each has an owner of its own (`../../PLAN.md`, "Who owns what").

mod asset;
pub mod ball;
mod draw;
mod event;
pub mod figure;
pub mod letter;
mod film;
mod glass;
pub mod haze;
mod icon;
mod lens;
pub mod light;
mod mode;
pub mod orbit;
mod paint;
mod raster;
mod scan;
mod sheet;
mod sun;
mod view;
pub mod wire;

pub use asset::PAsset;
pub use draw::PDraw;
pub use event::PEvent;
pub use film::{PFilm, PStill};
pub use glass::PGlass;
pub use icon::PIcon;
pub use light::PLight;
pub use mode::{PMode, setting};
pub use orbit::{POrbit, PPiece};
pub use paint::{PColor, PFit, PPaint, PPoint, PPress, PRect, PSize, within};
pub use lens::PCamera;
pub use raster::{PRaster, PSolid};
pub use sun::PSun;
pub use sheet::{PMark, PSheet};
pub use view::{PView, send, show};
