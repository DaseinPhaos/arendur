// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Defines `Renderer` which can render a scene

use self::scene::Scene;

/// A renderer
pub trait Renderer {
    /// render a scene
    fn render(&mut self, scene: &Scene);
}

pub mod scene;
pub mod whitted;
pub mod bpt;
pub mod pt;
pub mod prelude {
    pub use super::Renderer;
    pub use super::scene::Scene;
    pub use super::whitted::WhittedRenderer;
    pub use super::bpt::BPTRenderer;
    pub use super::pt::PTRenderer;
}
