#![allow(dead_code)]
use ndarray::{Array1, Array2, Array3};
use serde::{Deserialize, Serialize};

/// **A struct representing an RGBA color.**
/// 
/// Used all over the place in RGSS.
#[derive(Debug, Serialize, Deserialize)]
pub struct Color {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

// Default values
impl Default for Color {
    fn default() -> Self {
        Self {
            red: 255,
            green: 255,
            blue: 255,
            alpha: 255,
        }
    }
}

/// **A struct representing an offset to an RGBA color.**
/// 
/// Its members are i16 but must not exceed the range of 255..-255.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Tone {
    red: i16,
    green: i16,
    blue: i16,
    gray: i16,
}

/// Normal RGSS has dynamically dimensioned arrays, but in practice that does not map well to Rust. 
/// We don't particularly need dynamically sized arrays anyway.
pub type Table1 = Array1<i16>;
pub type Table2 = Array2<i16>;
pub type Table3 = Array3<i16>;