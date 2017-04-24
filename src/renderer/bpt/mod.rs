// Copyright 2017 Dasein Phaos aka. Luxko
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A bidirectional path tracing renderer

use bxdf::*;
use sample::Sampler;
use filming::Camera;
use super::Renderer;
use std::sync::Arc;
use super::scene::Scene;
use filming::film::FilmTile;
use spectrum::{RGBSpectrumf, Spectrum};
use rayon::prelude::*;
use copy_arena::{Allocator, Arena};
use geometry::prelude::*;
use std::path::{PathBuf, Path};
use self::node::Node;

/// A bidirectional path tracing renderer
pub struct BPTRenderer<S> {
    sampler: S,
    camera: Arc<Camera>,
    path_len: usize,
    path: PathBuf,
    max_depth: usize,
    
}

mod node;