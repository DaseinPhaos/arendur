use cgmath;

pub type Float = f32;
pub type Point2f = cgmath::Point2<Float>;
pub type Point3f = cgmath::Point3<Float>;
pub type Vector2f = cgmath::Vector2<Float>;
pub type Vector3f = cgmath::Vector3<Float>;
pub use cgmath::{Point2, Point3, Vector2, Vector3, BaseNum, BaseFloat, PartialOrd};
pub use cgmath::prelude::*;
//pub use num_traits::{Num, NumCast};