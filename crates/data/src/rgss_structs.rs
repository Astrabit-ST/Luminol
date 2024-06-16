#![allow(missing_docs)]

/// **A struct representing an RGBA color.**
///
/// Used all over the place in RGSS.
#[derive(Debug, Clone, Copy, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(from = "alox_48::Userdata", into = "alox_48::Userdata")]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Color {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    pub alpha: f64,
}

impl From<alox_48::Userdata> for Color {
    fn from(value: alox_48::Userdata) -> Self {
        *bytemuck::from_bytes(&value.data)
    }
}

impl From<Color> for alox_48::Userdata {
    fn from(value: Color) -> Self {
        alox_48::Userdata {
            class: "Color".into(),
            data: bytemuck::bytes_of(&value).to_vec(),
        }
    }
}

impl From<Color> for alox_48::Value {
    fn from(value: Color) -> Self {
        Self::Userdata(value.into())
    }
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
/// Its members are f64 but must not exceed the range of 255..-255.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(from = "alox_48::Userdata", into = "alox_48::Userdata")]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Tone {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
    pub gray: f64,
}

impl From<alox_48::Userdata> for Tone {
    fn from(value: alox_48::Userdata) -> Self {
        *bytemuck::from_bytes(&value.data)
    }
}

impl From<Tone> for alox_48::Userdata {
    fn from(value: Tone) -> Self {
        alox_48::Userdata {
            class: "Tone".into(),
            data: bytemuck::bytes_of(&value).to_vec(),
        }
    }
}

impl From<Tone> for alox_48::Value {
    fn from(value: Tone) -> Self {
        Self::Userdata(value.into())
    }
}

use std::ops::{Index, IndexMut};

/// Normal RGSS has dynamically dimensioned arrays, but in practice that does not map well to Rust.
/// We don't particularly need dynamically sized arrays anyway.
/// 1D Table.
#[derive(Debug, Default, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(from = "alox_48::Userdata", into = "alox_48::Userdata")]
pub struct Table1 {
    xsize: usize,
    data: Vec<i16>,
}

impl From<alox_48::Userdata> for Table1 {
    fn from(value: alox_48::Userdata) -> Self {
        let u32_slice: &[u32] =
            bytemuck::cast_slice(&value.data[0..std::mem::size_of::<u32>() * 5]);

        assert_eq!(u32_slice[0], 1);
        let xsize = u32_slice[1] as usize;
        let ysize = u32_slice[2] as usize;
        let zsize = u32_slice[3] as usize;
        let len = u32_slice[4] as usize;

        assert_eq!(xsize * ysize * zsize, len);
        let data = bytemuck::cast_slice(&value.data[(std::mem::size_of::<u32>() * 5)..]).to_vec();
        assert_eq!(data.len(), len);

        Self { xsize, data }
    }
}

impl From<Table1> for alox_48::Userdata {
    fn from(value: Table1) -> Self {
        let header = &[1, value.xsize as u32, 1, 1, value.len() as u32];
        let mut data = bytemuck::pod_collect_to_vec(header);
        data.extend_from_slice(bytemuck::cast_slice(&value.data));

        Self {
            class: "Table".into(),
            data,
        }
    }
}

impl Table1 {
    /// Create a new 1d array with a width of xsize.
    #[must_use]
    pub fn new(xsize: usize) -> Self {
        Self {
            xsize,
            data: vec![0; xsize],
        }
    }

    /// Width of the table.
    #[must_use]
    pub fn xsize(&self) -> usize {
        self.xsize
    }

    /// Total number of elements in the table.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Is the table empty?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn resize(&mut self, xsize: usize) {
        self.data.resize(xsize, 0);
        self.xsize = xsize;
    }

    pub fn resize_with_value(&mut self, xsize: usize, value: i16) {
        self.data.resize(xsize, value);
        self.xsize = xsize;
    }

    pub fn iter(&self) -> impl Iterator<Item = &i16> {
        self.data.iter()
    }

    pub fn as_slice(&self) -> &[i16] {
        self.data.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [i16] {
        self.data.as_mut_slice()
    }
}

impl Index<usize> for Table1 {
    type Output = i16;

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
#[derive(Debug, Default, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(from = "alox_48::Userdata", into = "alox_48::Userdata")]
pub struct Table2 {
    xsize: usize,
    ysize: usize,
    data: Vec<i16>,
}

impl From<alox_48::Userdata> for Table2 {
    fn from(value: alox_48::Userdata) -> Self {
        let u32_slice: &[u32] =
            bytemuck::cast_slice(&value.data[0..std::mem::size_of::<u32>() * 5]);

        assert_eq!(u32_slice[0], 2);
        let xsize = u32_slice[1] as usize;
        let ysize = u32_slice[2] as usize;
        let zsize = u32_slice[3] as usize;
        let len = u32_slice[4] as usize;

        assert_eq!(xsize * ysize * zsize, len);
        let data = bytemuck::cast_slice(&value.data[(std::mem::size_of::<u32>() * 5)..]).to_vec();
        assert_eq!(data.len(), len);

        Self { xsize, ysize, data }
    }
}

impl From<Table2> for alox_48::Userdata {
    fn from(value: Table2) -> Self {
        let header = &[
            2,
            value.xsize as u32,
            value.ysize as u32,
            1,
            value.len() as u32,
        ];
        let mut data = bytemuck::pod_collect_to_vec(header);
        data.extend_from_slice(bytemuck::cast_slice(&value.data));

        Self {
            class: "Table".into(),
            data,
        }
    }
}

impl Table2 {
    /// Create a new 2D table with a width of xsize and a height of ysize.
    #[must_use]
    pub fn new(xsize: usize, ysize: usize) -> Self {
        Self {
            xsize,
            ysize,
            data: vec![0; xsize * ysize],
        }
    }

    /// Width of the table.
    #[must_use]
    pub fn xsize(&self) -> usize {
        self.xsize
    }

    /// Height of the table.
    #[must_use]
    pub fn ysize(&self) -> usize {
        self.ysize
    }

    /// Total number of elements in the table.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Is the table empty?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn resize(&mut self, xsize: usize, ysize: usize) {
        let mut new_data = vec![0; xsize * ysize];

        for y in 0..self.ysize.min(ysize) {
            for x in 0..self.xsize.min(xsize) {
                new_data[xsize * y + x] = self[(x, y)]
            }
        }

        self.xsize = xsize;
        self.ysize = ysize;

        self.data = new_data;
    }

    pub fn iter(&self) -> impl Iterator<Item = &i16> {
        self.data.iter()
    }

    pub fn as_slice(&self) -> &[i16] {
        self.data.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [i16] {
        self.data.as_mut_slice()
    }
}

impl Index<(usize, usize)> for Table2 {
    type Output = i16;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.data[index.0 + index.1 * self.xsize]
    }
}

impl IndexMut<(usize, usize)> for Table2 {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.data[index.0 + index.1 * self.xsize]
    }
}

#[derive(Debug, Default, Clone)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(from = "alox_48::Userdata", into = "alox_48::Userdata")]
/// 3D table. See [`Table2`].
pub struct Table3 {
    xsize: usize,
    ysize: usize,
    zsize: usize,
    data: Vec<i16>,
}

impl From<alox_48::Userdata> for Table3 {
    fn from(value: alox_48::Userdata) -> Self {
        let u32_slice: &[u32] =
            bytemuck::cast_slice(&value.data[0..std::mem::size_of::<u32>() * 5]);

        assert_eq!(u32_slice[0], 3);
        let xsize = u32_slice[1] as usize;
        let ysize = u32_slice[2] as usize;
        let zsize = u32_slice[3] as usize;
        let len = u32_slice[4] as usize;

        assert_eq!(xsize * ysize * zsize, len);
        let data = bytemuck::cast_slice(&value.data[(std::mem::size_of::<u32>() * 5)..]).to_vec();
        assert_eq!(data.len(), len);

        Self {
            xsize,
            ysize,
            zsize,
            data,
        }
    }
}

impl From<Table3> for alox_48::Userdata {
    fn from(value: Table3) -> Self {
        let header = &[
            3,
            value.xsize as u32,
            value.ysize as u32,
            value.zsize as u32,
            value.len() as u32,
        ];
        let mut data = bytemuck::pod_collect_to_vec(header);
        data.extend_from_slice(bytemuck::cast_slice(&value.data));

        Self {
            class: "Table".into(),
            data,
        }
    }
}

impl Table3 {
    /// Create a new 3D table with a width of xsize, a height of ysize, and a depth of zsize.
    #[must_use]
    pub fn new(xsize: usize, ysize: usize, zsize: usize) -> Self {
        Self {
            xsize,
            ysize,
            zsize,
            data: vec![0; xsize * ysize * zsize],
        }
    }

    #[must_use]
    pub fn new_data(xsize: usize, ysize: usize, zsize: usize, data: Vec<i16>) -> Self {
        assert_eq!(xsize * ysize * zsize, data.len());
        Self {
            xsize,
            ysize,
            zsize,
            data,
        }
    }

    /// Width of the table.
    #[must_use]
    pub fn xsize(&self) -> usize {
        self.xsize
    }

    /// Height of the table.
    #[must_use]
    pub fn ysize(&self) -> usize {
        self.ysize
    }

    /// Depth of the table.
    #[must_use]
    pub fn zsize(&self) -> usize {
        self.zsize
    }

    /// Total number of elements in the table.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Is the table empty?
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn resize(&mut self, xsize: usize, ysize: usize, zsize: usize) {
        let mut new_data = vec![0; xsize * ysize];

        // A naive for loop like this is optimized to a handful of memcpys.
        for z in 0..self.zsize.min(zsize) {
            for y in 0..self.ysize.min(ysize) {
                for x in 0..self.xsize.min(xsize) {
                    new_data[(xsize * ysize * z) + (xsize * y) + x] = self[(x, y, z)]
                }
            }
        }

        self.xsize = xsize;
        self.ysize = ysize;
        self.zsize = zsize;

        self.data = new_data;
    }

    pub fn iter(&self) -> impl Iterator<Item = &i16> {
        self.data.iter()
    }

    pub fn as_slice(&self) -> &[i16] {
        self.data.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [i16] {
        self.data.as_mut_slice()
    }

    pub fn layer_as_slice(&self, layer: usize) -> &[i16] {
        let layer_size = self.xsize * self.ysize;
        &self.data[(layer_size * layer)..(layer_size * (layer + 1))]
    }
}

impl Index<(usize, usize, usize)> for Table3 {
    type Output = i16;

    fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
        &self.data[index.0 + self.xsize * (index.1 + self.ysize * index.2)]
    }
}

impl IndexMut<(usize, usize, usize)> for Table3 {
    fn index_mut(&mut self, index: (usize, usize, usize)) -> &mut Self::Output {
        &mut self.data[index.0 + self.xsize * (index.1 + self.ysize * index.2)]
    }
}
