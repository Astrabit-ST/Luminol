// Copyright (C) 2023 Lily Lyons
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

use itertools::Itertools;
use std::io::{prelude::*, BufReader, SeekFrom};

use crate::{DirEntry, Error, Metadata, OpenFlags};

#[derive(Debug, Default)]
pub struct FileSystem<T> {
    trie: parking_lot::RwLock<crate::FileSystemTrie<Entry>>,
    archive: parking_lot::Mutex<T>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Entry {
    offset: u64,
    size: u64,
    start_magic: u32,
}

const MAGIC: u32 = 0xDEADCAFE;
const HEADER: &[u8] = b"RGSSAD\0";

impl<T> FileSystem<T>
where
    T: Read + Write + Seek + Send + Sync,
{
    pub fn new(mut file: T) -> Result<Self, Error> {
        let version = Self::read_header(&mut file)?;

        let mut trie = crate::FileSystemTrie::new();

        fn read_u32<F>(file: &mut F) -> Result<u32, Error>
        where
            F: Read + Write + Seek + Send + Sync,
        {
            let mut buffer = [0; 4];
            file.read_exact(&mut buffer)?;
            Ok(u32::from_le_bytes(buffer))
        }

        fn read_u32_xor<F>(file: &mut F, key: u32) -> Result<u32, Error>
        where
            F: Read + Write + Seek + Send + Sync,
        {
            let result = read_u32(file)?;
            Ok(result ^ key)
        }

        match version {
            1 | 2 => {
                let mut magic = MAGIC;

                while let Ok(name_len) = read_u32_xor(&mut file, Self::advance_magic(&mut magic)) {
                    let mut name = vec![0; name_len as usize];
                    file.read_exact(&mut name).unwrap();
                    for byte in name.iter_mut() {
                        let char = *byte ^ Self::advance_magic(&mut magic) as u8;
                        if char == b'\\' {
                            *byte = b'/';
                        } else {
                            *byte = char;
                        }
                    }
                    let name = camino::Utf8PathBuf::from(String::from_utf8(name)?);

                    let entry_len = read_u32_xor(&mut file, Self::advance_magic(&mut magic))?;

                    let entry = Entry {
                        size: entry_len as u64,
                        offset: file.stream_position()?,
                        start_magic: magic,
                    };
                    trie.create_file(&name, entry);

                    file.seek(SeekFrom::Start(entry.offset + entry.size))?;
                }
            }
            3 => {
                let mut u32_buf = [0; 4];
                file.read_exact(&mut u32_buf)?;

                let base_magic = u32::from_le_bytes(u32_buf);
                let base_magic = (base_magic * 9) + 3;

                while let Ok(offset) = read_u32_xor(&mut file, base_magic) {
                    if offset == 0 {
                        break;
                    }

                    let entry_len = read_u32_xor(&mut file, base_magic)?;
                    let magic = read_u32_xor(&mut file, base_magic)?;
                    let name_len = read_u32_xor(&mut file, base_magic)?;

                    let mut name = vec![0; name_len as usize];
                    file.read_exact(&mut name)?;
                    for (i, byte) in name.iter_mut().enumerate() {
                        let char = *byte ^ (base_magic >> (8 * (i % 4))) as u8;
                        if char == b'\\' {
                            *byte = b'/';
                        } else {
                            *byte = char;
                        }
                    }
                    let name = camino::Utf8PathBuf::from(String::from_utf8(name)?);

                    let entry = Entry {
                        size: entry_len as u64,
                        offset: offset as u64,
                        start_magic: magic,
                    };
                    trie.create_file(&name, entry);
                }
            }
            _ => return Err(Error::InvalidHeader),
        }

        /*
        for dir in directories.iter() {
            println!("===========");
            println!("{}", dir.key());
            for i in dir.value().iter() {
                println!("{}", &*i);
            }
            println!("----------");
        }
        */

        Ok(FileSystem {
            trie: parking_lot::RwLock::new(trie),
            archive: parking_lot::Mutex::new(file),
        })
    }

    fn advance_magic(magic: &mut u32) -> u32 {
        let old = *magic;

        *magic = magic.wrapping_mul(7).wrapping_add(3);

        old
    }

    fn read_header(file: &mut T) -> Result<u8, Error> {
        let mut header_buf = [0; 8];

        file.read_exact(&mut header_buf)?;

        if !header_buf.starts_with(HEADER) {
            return Err(Error::InvalidHeader);
        }

        Ok(header_buf[7])
    }
}

#[derive(Debug)]
pub struct File {
    tmp: crate::host::File,
}

impl std::io::Write for File {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.tmp.write(buf)
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.tmp.write_vectored(bufs)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl std::io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.tmp.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        self.tmp.read_vectored(bufs)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        self.tmp.read_exact(buf)
    }
}

impl std::io::Seek for File {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.tmp.seek(pos)
    }

    fn stream_position(&mut self) -> std::io::Result<u64> {
        self.tmp.stream_position()
    }
}

impl crate::File for File {
    fn metadata(&self) -> crate::Result<Metadata> {
        self.tmp.metadata()
    }
}

impl<T> crate::FileSystem for FileSystem<T>
where
    T: Read + Write + Seek + Send + Sync + 'static,
{
    type File = File;

    fn open_file(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        flags: OpenFlags,
    ) -> Result<Self::File, Error> {
        if flags.contains(OpenFlags::Write) {
            return Err(Error::NotSupported);
        }

        let path = path.as_ref();
        let trie = self.trie.read();
        let entry = trie.get_file(path).ok_or(Error::NotExist)?;
        let mut tmp = crate::host::File::new()?;

        {
            let mut archive = self.archive.lock();
            archive.seek(SeekFrom::Start(entry.offset))?;

            let adapter = BufReader::new(<T as Read>::by_ref(&mut archive).take(entry.size));
            let iter = adapter
                .bytes()
                .scan((entry.start_magic, 0), |state, maybe_byte| {
                    let Ok(byte) = maybe_byte else { return None };
                    let (mut magic, mut j) = *state;

                    if j == 4 {
                        j = 0;
                        magic = magic.wrapping_mul(7).wrapping_add(3);
                    }
                    let byte = byte ^ magic.to_le_bytes()[j];
                    j += 1;

                    *state = (magic, j);
                    Some(byte)
                });

            std::io::copy(&mut iter_read::IterRead::new(iter), &mut tmp)?;
        }

        tmp.flush()?;
        tmp.seek(SeekFrom::Start(0))?;
        Ok(File { tmp })
    }

    fn metadata(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Metadata, Error> {
        let path = path.as_ref();
        let trie = self.trie.read();
        if let Some(entry) = trie.get_file(path) {
            Ok(Metadata {
                is_file: true,
                size: entry.size,
            })
        } else if let Some(size) = trie.get_dir_size(path) {
            Ok(Metadata {
                is_file: false,
                size: size as u64,
            })
        } else {
            Err(Error::NotExist)
        }
    }

    fn rename(
        &self,
        _from: impl AsRef<camino::Utf8Path>,
        _to: impl AsRef<camino::Utf8Path>,
    ) -> std::result::Result<(), Error> {
        Err(Error::NotSupported)
    }

    fn create_dir(&self, _path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        Err(Error::NotSupported)
    }

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool, Error> {
        let trie = self.trie.read();
        Ok(trie.contains(path))
    }

    fn remove_dir(&self, _path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        Err(Error::NotSupported)
    }

    fn remove_file(&self, _path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        Err(Error::NotSupported)
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>, Error> {
        let path = path.as_ref();
        let trie = self.trie.read();
        if let Some(iter) = trie.iter(path) {
            iter.map(|(name, _)| {
                let path = path.join(name);
                let metadata = self.metadata(&path)?;
                Ok(DirEntry { path, metadata })
            })
            .try_collect()
        } else {
            Err(Error::NotExist)
        }
    }
}
