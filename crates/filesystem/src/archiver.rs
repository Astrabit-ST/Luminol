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
use std::io::{
    prelude::*,
    BufReader,
    ErrorKind::{InvalidData, PermissionDenied},
    SeekFrom,
};

use crate::File as _;
use crate::{DirEntry, Error, Metadata, OpenFlags};

#[derive(Debug, Default)]
pub struct FileSystem<T> {
    trie: parking_lot::RwLock<crate::FileSystemTrie<Entry>>,
    archive: std::sync::Arc<parking_lot::Mutex<T>>,
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
    T: crate::File,
{
    pub fn new(mut file: T) -> Result<Self, Error> {
        let version = read_header(&mut file)?;

        let mut trie = crate::FileSystemTrie::new();

        for (name, entry) in read_file_table(&mut file, version)? {
            trie.create_file(name, entry);
        }

        Ok(FileSystem {
            trie: parking_lot::RwLock::new(trie),
            archive: std::sync::Arc::new(parking_lot::Mutex::new(file)),
        })
    }
}

fn read_u32<F>(file: &mut F) -> Result<u32, Error>
where
    F: Read,
{
    let mut buffer = [0; 4];
    file.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

fn read_u32_xor<F>(file: &mut F, key: u32) -> Result<u32, Error>
where
    F: Read,
{
    let result = read_u32(file)?;
    Ok(result ^ key)
}

fn read_file_table<T>(file: &mut T, version: u8) -> Result<Vec<(camino::Utf8PathBuf, Entry)>, Error>
where
    T: Read + Seek,
{
    let mut entries = Vec::new();

    match version {
        1 | 2 => {
            let mut magic = MAGIC;

            while let Ok(name_len) = read_u32_xor(file, advance_magic(&mut magic)) {
                let mut name = vec![0; name_len as usize];
                file.read_exact(&mut name)?;
                for byte in name.iter_mut() {
                    let char = *byte ^ advance_magic(&mut magic) as u8;
                    if char == b'\\' {
                        *byte = b'/';
                    } else {
                        *byte = char;
                    }
                }
                let name = camino::Utf8PathBuf::from(String::from_utf8(name)?);

                let entry_len = read_u32_xor(file, advance_magic(&mut magic))?;

                let entry = Entry {
                    size: entry_len as u64,
                    offset: file.stream_position()?,
                    start_magic: magic,
                };

                entries.push((name, entry));

                file.seek(SeekFrom::Start(entry.offset + entry.size))?;
            }
        }
        3 => {
            let mut u32_buf = [0; 4];
            file.read_exact(&mut u32_buf)?;

            let base_magic = u32::from_le_bytes(u32_buf);
            let base_magic = (base_magic * 9) + 3;

            while let Ok(offset) = read_u32_xor(file, base_magic) {
                if offset == 0 {
                    break;
                }

                let entry_len = read_u32_xor(file, base_magic)?;
                let magic = read_u32_xor(file, base_magic)?;
                let name_len = read_u32_xor(file, base_magic)?;

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
                entries.push((name, entry));
            }
        }
        _ => return Err(Error::InvalidHeader),
    }

    Ok(entries)
}

fn read_file_xor<T>(archive: &mut T, start_magic: u32) -> impl Read + '_
where
    T: Read,
{
    let iter = archive.bytes().scan((start_magic, 0), |state, maybe_byte| {
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
    iter_read::IterRead::new(iter)
}

fn advance_magic(magic: &mut u32) -> u32 {
    let old = *magic;

    *magic = magic.wrapping_mul(7).wrapping_add(3);

    old
}

fn regress_magic(magic: &mut u32) -> u32 {
    let old = *magic;

    *magic = magic.wrapping_sub(3).wrapping_mul(3067833783);

    old
}

fn read_header<T>(file: &mut T) -> Result<u8, Error>
where
    T: Read,
{
    let mut header_buf = [0; 8];

    file.read_exact(&mut header_buf)?;

    if !header_buf.starts_with(HEADER) {
        return Err(Error::InvalidHeader);
    }

    Ok(header_buf[7])
}

#[derive(Debug)]
pub struct File<T>
where
    T: crate::File,
{
    archive: Option<std::sync::Arc<parking_lot::Mutex<T>>>,
    path: camino::Utf8PathBuf,
    read_allowed: bool,
    tmp: crate::host::File,
}

impl<T> Drop for File<T>
where
    T: crate::File,
{
    fn drop(&mut self) {
        if self.archive.is_some() {
            let _ = self.flush();
        }
    }
}

impl<T> std::io::Write for File<T>
where
    T: crate::File,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.archive.is_some() {
            self.tmp.write(buf)
        } else {
            Err(PermissionDenied.into())
        }
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        if self.archive.is_some() {
            self.tmp.write_vectored(bufs)
        } else {
            Err(PermissionDenied.into())
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let Some(archive) = &self.archive else {
            return Err(PermissionDenied.into());
        };

        let mut archive = archive.lock();
        let stream_position = archive.stream_position()?;
        let archive_length = archive.metadata().map_err(|_| PermissionDenied)?.size;
        archive.seek(SeekFrom::Start(0))?;

        let version =
            read_header(&mut <T as Read>::by_ref(&mut archive)).map_err(|_| InvalidData)?;
        let mut entries = read_file_table(&mut <T as Read>::by_ref(&mut archive), version)
            .map_err(|_| InvalidData)?;
        let (entry_position, (path, mut entry)) = entries
            .iter()
            .find_position(|(path, _)| path == &self.path)
            .ok_or(InvalidData)?
            .clone();

        // If the size of the file has changed, rotate the archive to place the file at the end of
        // the archive before writing the new contents of the file
        let old_size = entry.size;
        let new_size = self.tmp.metadata().map_err(|_| PermissionDenied)?.size;
        if old_size != new_size {
            match version {
                1 | 2 if entry_position + 1 != entries.len() => {
                    let mut tmp = crate::host::File::new()?;

                    // Determine what the magic value was for the beginning of the modified file's
                    // header
                    let mut magic = entry.start_magic;
                    regress_magic(&mut magic);
                    regress_magic(&mut magic);
                    for _ in path.as_str().as_bytes() {
                        regress_magic(&mut magic);
                    }

                    // Re-encrypt the headers and data for the files after the modified file into a
                    // temporary file
                    for (path, entry) in entries[entry_position + 1..].iter() {
                        tmp.write_all(
                            &(path.as_str().as_bytes().len() as u32 ^ advance_magic(&mut magic))
                                .to_le_bytes(),
                        )?;
                        tmp.write_all(
                            &path
                                .as_str()
                                .bytes()
                                .map(|b| {
                                    let b = if b == b'/' { b'\\' } else { b };
                                    b ^ advance_magic(&mut magic) as u8
                                })
                                .collect_vec(),
                        )?;
                        tmp.write_all(
                            &(entry.size as u32 ^ advance_magic(&mut magic)).to_le_bytes(),
                        )?;

                        archive.seek(SeekFrom::Start(entry.offset))?;
                        let mut adapter =
                            BufReader::new(<T as Read>::by_ref(&mut archive).take(entry.size));
                        std::io::copy(
                            &mut read_file_xor(
                                &mut read_file_xor(&mut adapter, entry.start_magic),
                                magic,
                            ),
                            &mut tmp,
                        )?;
                    }

                    // Write the header of the modified file at the end of the temporary file
                    tmp.write_all(
                        &(path.as_str().as_bytes().len() as u32 ^ advance_magic(&mut magic))
                            .to_le_bytes(),
                    )?;
                    tmp.write_all(
                        &path
                            .as_str()
                            .bytes()
                            .map(|b| {
                                let b = if b == b'/' { b'\\' } else { b };
                                b ^ advance_magic(&mut magic) as u8
                            })
                            .collect_vec(),
                    )?;
                    tmp.write_all(&(new_size as u32 ^ advance_magic(&mut magic)).to_le_bytes())?;

                    // Write the contents of the temporary file into the archive, starting from
                    // where the modified file's header was
                    tmp.flush()?;
                    tmp.seek(SeekFrom::Start(0))?;
                    archive.seek(SeekFrom::Start(
                        entry
                            .offset
                            .checked_sub(path.as_str().bytes().len() as u64 + 8)
                            .ok_or(InvalidData)?,
                    ))?;
                    std::io::copy(&mut tmp, &mut <T as Write>::by_ref(&mut archive))?;

                    entry.start_magic = magic;
                }

                1 | 2 => {
                    // The file is already at the end of the archive, so we just need to change the
                    // length of the file
                    let mut magic = entry.start_magic;
                    regress_magic(&mut magic);
                    archive.seek(SeekFrom::Start(
                        entry.offset.checked_sub(4).ok_or(InvalidData)?,
                    ))?;
                    archive.write_all(&(new_size as u32 ^ magic).to_le_bytes())?;
                }

                3 if entry_position + 1 != entries.len() => {
                    let mut tmp = crate::host::File::new()?;

                    archive.seek(SeekFrom::Start(entry.offset + entry.size))?;
                    std::io::copy(&mut <T as Read>::by_ref(&mut archive), &mut tmp)?;
                    tmp.flush()?;

                    archive.seek(SeekFrom::Start(entry.offset))?;
                    tmp.seek(SeekFrom::Start(0))?;
                    std::io::copy(&mut tmp, &mut <T as Write>::by_ref(&mut archive))?;

                    // TODO write these modified offsets to the archive
                    for (_, e) in entries.iter_mut() {
                        if e.offset > entry.offset {
                            e.offset = e.offset.checked_sub(entry.size).ok_or(InvalidData)?;
                        }
                    }
                }

                3 => {}

                _ => return Err(InvalidData.into()),
            }

            entry.offset = archive_length.checked_sub(old_size).ok_or(InvalidData)?;
        }

        let tmp_stream_position = self.tmp.stream_position()?;
        self.tmp.flush()?;
        self.tmp.seek(SeekFrom::Start(0))?;
        archive.seek(SeekFrom::Start(entry.offset))?;
        let mut adapter = BufReader::new(&mut self.tmp);
        std::io::copy(
            &mut read_file_xor(&mut adapter, entry.start_magic),
            &mut <T as Write>::by_ref(&mut archive),
        )?;
        self.tmp.seek(SeekFrom::Start(tmp_stream_position))?;

        if old_size > new_size {
            archive.set_len(
                archive_length
                    .checked_sub(old_size)
                    .ok_or(InvalidData)?
                    .checked_add(new_size)
                    .ok_or(InvalidData)?,
            )?;
        }
        archive.flush()?;
        archive.seek(SeekFrom::Start(stream_position))?;
        Ok(())
    }
}

impl<T> std::io::Read for File<T>
where
    T: crate::File,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.read_allowed {
            self.tmp.read(buf)
        } else {
            Err(PermissionDenied.into())
        }
    }

    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        if self.read_allowed {
            self.tmp.read_vectored(bufs)
        } else {
            Err(PermissionDenied.into())
        }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        if self.read_allowed {
            self.tmp.read_exact(buf)
        } else {
            Err(PermissionDenied.into())
        }
    }
}

impl<T> std::io::Seek for File<T>
where
    T: crate::File,
{
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.tmp.seek(pos)
    }

    fn stream_position(&mut self) -> std::io::Result<u64> {
        self.tmp.stream_position()
    }
}

impl<T> crate::File for File<T>
where
    T: crate::File,
{
    fn metadata(&self) -> crate::Result<Metadata> {
        self.tmp.metadata()
    }

    fn set_len(&self, new_size: u64) -> std::io::Result<()> {
        self.tmp.set_len(new_size)
    }
}

impl<T> crate::FileSystem for FileSystem<T>
where
    T: crate::File,
{
    type File = File<T>;

    fn open_file(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        flags: OpenFlags,
    ) -> Result<Self::File, Error> {
        if flags.contains(OpenFlags::Create) {
            return Err(Error::NotSupported);
        }

        let path = path.as_ref();
        let trie = self.trie.read();
        let entry = trie.get_file(path).ok_or(Error::NotExist)?;
        let mut tmp = crate::host::File::new()?;

        {
            let mut archive = self.archive.lock();
            archive.seek(SeekFrom::Start(entry.offset))?;

            if !flags.contains(OpenFlags::Truncate) {
                let mut adapter =
                    BufReader::new(<T as Read>::by_ref(&mut archive).take(entry.size));
                std::io::copy(
                    &mut read_file_xor(&mut adapter, entry.start_magic),
                    &mut tmp,
                )?;
            }
        }

        tmp.flush()?;
        tmp.seek(SeekFrom::Start(0))?;
        Ok(File {
            archive: flags
                .contains(OpenFlags::Write)
                .then(|| self.archive.clone()),
            path: path.to_owned(),
            read_allowed: flags.contains(OpenFlags::Read),
            tmp,
        })
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
