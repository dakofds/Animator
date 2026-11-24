use crate::drawing::stroke::Stroke;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frame {
    pub layers: Vec<Vec<Stroke>>,
}

impl Frame {
    pub fn new(layer_count: usize) -> Self {
        Self {
            layers: vec![Vec::new(); layer_count],
        }
    }
}
