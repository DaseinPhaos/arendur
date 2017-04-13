//! Defines stratified samplers

use super::sink::{Sink, Sinkf, Sink2f};

/// Represents a stratified sampler
pub struct StrataSampler {
    sinkf: Sinkf,
    sink2f: Sink2f,
    sampledx: u32,
    sampledy: u32,
    jitter: bool,
}

impl StrataSampler {
    // /// Construction
    // TODO: pub fn new(sampledx: u32, sampledy: u32, dimension: u32, jitter)
}