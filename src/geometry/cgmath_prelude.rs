use cgmath;

pub type Float = f32;
pub type Point2f = cgmath::Point2<Float>;
pub type Point3f = cgmath::Point3<Float>;
pub type Vector2f = cgmath::Vector2<Float>;
pub type Vector3f = cgmath::Vector3<Float>;
pub type Vector4f = cgmath::Vector4<Float>;
pub type Matrix3f = cgmath::Matrix3<Float>;
pub type Matrix4f = cgmath::Matrix4<Float>;
pub type Basis3f = cgmath::Basis3<Float>;
pub use cgmath::{Point2, Point3, Vector2, Vector3, Vector4, Basis3, BaseNum, BaseFloat, Matrix4, PartialOrd};
pub use cgmath::prelude::*;
//pub use num_traits::{Num, NumCast};