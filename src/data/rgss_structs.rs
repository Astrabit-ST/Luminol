#![allow(dead_code)]
use serde::{Deserialize, Serialize};

/// **A struct representing an RGBA color.**
///
/// Used all over the place in RGSS.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Tone {
    red: f32,
    green: f32,
    blue: f32,
    gray: f32,
}

use std::ops::{Index, IndexMut};

/// Normal RGSS has dynamically dimensioned arrays, but in practice that does not map well to Rust.
/// We don't particularly need dynamically sized arrays anyway.
/// 1D Table.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Table1 {
    xsize: usize,
    data: Vec<i32>,
}

impl Table1 {
    /// Create a new 1d array with a width of xsize.
    pub fn new(xsize: usize) -> Self {
        Self {
            xsize,
            data: vec![0; xsize],
        }
    }

    /// Width of the table.
    pub fn xsize(&self) -> usize {
        self.xsize
    }

    /// Total number of elements in the table.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Is the table empty?
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Return an iterator over all the elements in the table.
    pub fn iter(&self) -> Iter<'_, i32> {
        self.data.iter()
    }
}

impl Index<usize> for Table1 {
    type Output = i32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl IndexMut<usize> for Table1 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

/// 2D table. See [`Table1`].
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Table2 {
    xsize: usize,

    ysize: usize,
    data: Vec<i32>,
}

impl Table2 {
    /// Create a new 2D table with a width of xsize and a height of ysize.
    pub fn new(xsize: usize, ysize: usize) -> Self {
        Self {
            xsize,
            ysize,
            data: vec![0; xsize * ysize],
        }
    }

    /// Width of the table.
    pub fn xsize(&self) -> usize {
        self.xsize
    }

    /// Height of the table.
    pub fn ysize(&self) -> usize {
        self.ysize
    }

    /// Total number of elements in the table.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Is the table empty?
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Return an iterator over all the elements in the table.
    pub fn iter(&self) -> Iter<'_, i32> {
        self.data.iter()
    }
}

impl Index<(usize, usize)> for Table2 {
    type Output = i32;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.data[index.0 + index.1 * self.xsize]
    }
}

impl IndexMut<(usize, usize)> for Table2 {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.data[index.0 + index.1 * self.xsize]
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
/// 3D table. See [`Table2`].
pub struct Table3 {
    xsize: usize,
    ysize: usize,
    zsize: usize,
    data: Vec<i32>,
}

use std::slice::Iter;

impl Table3 {
    /// Create a new 3D table with a width of xsize, a height of ysize, and a depth of zsize.
    pub fn new(xsize: usize, ysize: usize, zsize: usize) -> Self {
        Self {
            xsize,
            ysize,
            zsize,
            data: vec![0; xsize * ysize * zsize],
        }
    }

    /// Width of the table.
    pub fn xsize(&self) -> usize {
        self.xsize
    }

    /// Height of the table.
    pub fn ysize(&self) -> usize {
        self.ysize
    }

    /// Depth of the table.
    pub fn zsize(&self) -> usize {
        self.zsize
    }

    /// Total number of elements in the table.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Is the table empty?
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Return an iterator over all the elements in the table.
    pub fn iter(&self) -> Iter<'_, i32> {
        self.data.iter()
    }
}

impl Index<(usize, usize, usize)> for Table3 {
    type Output = i32;

    fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
        &self.data[index.0 + (index.1 * self.xsize) + (index.2 * self.ysize)]
    }
}

impl IndexMut<(usize, usize, usize)> for Table3 {
    fn index_mut(&mut self, index: (usize, usize, usize)) -> &mut Self::Output {
        &mut self.data[index.0 + (index.1 * self.xsize) + (index.2 * self.ysize)]
    }
}
