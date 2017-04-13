//! Contains foundamental geometry definitions.
//!
//! - `foundamental` ports some foundamental definitions from `cgmath`.
//! - `float` defines functions dealing with basic type `Float`.
//! - `ray` defines the ray interface.
//! - `bbox` defines the bounding box interface.
//! - `transform` defines the transform interface.
//! - `interaction` defines the interaction interface.

pub mod float;
pub mod ray;
pub mod bbox;
pub mod transform;
pub mod foundamental;
pub mod interaction;
pub mod prelude;

pub use self::foundamental::*;
pub use self::ray::{Ray, RawRay, RayDifferential};
pub use self::transform::TransformExt;
pub use self::bbox::{BBox2, BBox3, BBox2f, BBox3f};
pub use self::interaction::{DerivativeInfo2D, InteractInfo, SurfaceInteraction};
