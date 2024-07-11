// Copyright (C) 2024 Melody Madeline Lyons
//
// This file is part of Luminol.
//
// Luminol is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Luminol is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Luminol.  If not, see <http://www.gnu.org/licenses/>.

mod filesystem;
pub use filesystem::FileSystem;

mod file;
pub use file::File;

mod util;

type Trie = crate::FileSystemTrie<Entry>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Entry {
    header_offset: u64,
    body_offset: u64,
    size: u64,
    start_magic: u32,
}

const MAGIC: u32 = 0xDEADCAFE;
const HEADER: &[u8] = b"RGSSAD\0";
