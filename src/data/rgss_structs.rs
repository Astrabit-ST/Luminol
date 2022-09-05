#![allow(dead_code)]
use ndarray::{Array1, Array2, Array3};
use serde::{Deserialize, Serialize};

/// **A struct representing an RGBA color.**
///
/// Used all over the place in RGSS.
#[derive(Debug, Serialize, Deserialize)]
pub struct Color {
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
}

// Default values
impl Default for Color {
    fn default() -> Self {
        Self {
            red: 255.0,
            green: 255.0,
            blue: 255.0,
            alpha: 255.0,
        }
    }
}

/// **A struct representing an offset to an RGBA color.**
///
/// Its members are f32 but must not exceed the range of 255..-255.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Tone {
    red: f32,
    green: f32,
    blue: f32,
    gray: f32,
}

/// Normal RGSS has dynamically dimensioned arrays, but in practice that does not map well to Rust.
/// We don't particularly need dynamically sized arrays anyway.
pub type Table1 = Array1<i16>;
pub type Table2 = Array2<i16>;
pub type Table3 = Array3<i16>;
