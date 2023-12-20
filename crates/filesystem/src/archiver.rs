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
    BufReader, BufWriter,
    ErrorKind::{InvalidData, PermissionDenied},
    SeekFrom,
};

use crate::File as _;
use crate::{DirEntry, Error, Metadata, OpenFlags};

#[derive(Debug, Default)]
pub struct FileSystem<T> {
    trie: std::sync::Arc<parking_lot::RwLock<crate::FileSystemTrie<Entry>>>,
    archive: std::sync::Arc<parking_lot::Mutex<T>>,
    version: u8,
    base_magic: u32,
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
        let mut reader = BufReader::new(&mut file);

        let version = read_header(&mut reader)?;

        let mut trie = crate::FileSystemTrie::new();

        let mut base_magic = MAGIC;

        match version {
            1 | 2 => {
                let mut magic = MAGIC;

                while let Ok(path_len) = read_u32_xor(&mut reader, advance_magic(&mut magic)) {
                    let mut path = vec![0; path_len as usize];
                    reader.read_exact(&mut path)?;
                    for byte in path.iter_mut() {
                        let char = *byte ^ advance_magic(&mut magic) as u8;
                        if char == b'\\' {
                            *byte = b'/';
                        } else {
                            *byte = char;
                        }
                    }
                    let path = camino::Utf8PathBuf::from(String::from_utf8(path)?);

                    let entry_len = read_u32_xor(&mut reader, advance_magic(&mut magic))?;

                    let entry = Entry {
                        size: entry_len as u64,
                        offset: reader.stream_position()?,
                        start_magic: magic,
                    };

                    trie.create_file(path, entry);

                    reader.seek(SeekFrom::Start(entry.offset + entry.size))?;
                }
            }
            3 => {
                let mut u32_buf = [0; 4];
                reader.read_exact(&mut u32_buf)?;

                base_magic = u32::from_le_bytes(u32_buf);
                base_magic = (base_magic * 9) + 3;

                while let Ok(offset) = read_u32_xor(&mut reader, base_magic) {
                    if offset == 0 {
                        break;
                    }

                    let entry_len = read_u32_xor(&mut reader, base_magic)?;
                    let magic = read_u32_xor(&mut reader, base_magic)?;
                    let path_len = read_u32_xor(&mut reader, base_magic)?;

                    let mut path = vec![0; path_len as usize];
                    reader.read_exact(&mut path)?;
                    for (i, byte) in path.iter_mut().enumerate() {
                        let char = *byte ^ (base_magic >> (8 * (i % 4))) as u8;
                        if char == b'\\' {
                            *byte = b'/';
                        } else {
                            *byte = char;
                        }
                    }
                    let path = camino::Utf8PathBuf::from(String::from_utf8(path)?);

                    let entry = Entry {
                        size: entry_len as u64,
                        offset: offset as u64,
                        start_magic: magic,
                    };
                    trie.create_file(path, entry);
                }
            }
            _ => return Err(Error::InvalidHeader),
        }

        Ok(FileSystem {
            trie: std::sync::Arc::new(parking_lot::RwLock::new(trie)),
            archive: std::sync::Arc::new(parking_lot::Mutex::new(file)),
            version,
            base_magic,
        })
    }
}

fn read_u32<F>(file: &mut F) -> std::io::Result<u32>
where
    F: Read,
{
    let mut buffer = [0; 4];
    file.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

fn read_u32_xor<F>(file: &mut F, key: u32) -> std::io::Result<u32>
where
    F: Read,
{
    let result = read_u32(file)?;
    Ok(result ^ key)
}

fn read_file_xor<T>(file: &mut T, start_magic: u32) -> impl Read + '_
where
    T: Read,
{
    let iter = file.bytes().scan((start_magic, 0), |state, maybe_byte| {
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
    trie: Option<std::sync::Arc<parking_lot::RwLock<crate::FileSystemTrie<Entry>>>>,
    path: camino::Utf8PathBuf,
    read_allowed: bool,
    tmp: crate::host::File,
    modified: bool,
    version: u8,
    base_magic: u32,
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
            let count = self.tmp.write(buf)?;
            if count != 0 {
                self.modified = true;
            }
            Ok(count)
        } else {
            Err(PermissionDenied.into())
        }
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        if self.archive.is_some() {
            let count = self.tmp.write_vectored(bufs)?;
            if count != 0 {
                self.modified = true;
            }
            Ok(count)
        } else {
            Err(PermissionDenied.into())
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        if !self.modified {
            return Ok(());
        }

        let Some(archive) = &self.archive else {
            return Err(PermissionDenied.into());
        };
        let Some(trie) = &self.trie else {
            return Err(PermissionDenied.into());
        };
        let mut archive = archive.lock();
        let mut trie = trie.write();
        let path_len = self.path.as_str().bytes().len() as u64;
        let stream_position = archive.stream_position()?;
        let archive_len = archive.metadata()?.size;
        archive.seek(SeekFrom::Start(0))?;

        // If the size of the file has changed, rotate the archive to place the file at the end of
        // the archive before writing the new contents of the file
        let mut entry = *trie.get_file(&self.path).ok_or(InvalidData)?;
        let old_size = entry.size;
        let new_size = self.tmp.metadata()?.size;
        if old_size != new_size {
            let is_last = entry.offset + entry.size >= archive_len;
            match self.version {
                1 | 2 if !is_last => {
                    let mut tmp = crate::host::File::new()?;
                    archive.seek(SeekFrom::Start(entry.offset + entry.size))?;
                    let mut reader = BufReader::new(<T as Read>::by_ref(&mut archive));
                    let mut writer = BufWriter::new(&mut tmp);

                    let mut reader_magic = entry.start_magic;

                    // Determine what the magic value was for the beginning of the modified file's
                    // header
                    let mut writer_magic = entry.start_magic;
                    regress_magic(&mut writer_magic);
                    regress_magic(&mut writer_magic);
                    for _ in self.path.as_str().bytes() {
                        regress_magic(&mut writer_magic);
                    }

                    // Re-encrypt the headers and data for the files after the modified file into a
                    // temporary file
                    while let Ok(current_path_len) =
                        read_u32_xor(&mut reader, advance_magic(&mut reader_magic))
                    {
                        let mut current_path = vec![0; current_path_len as usize];
                        reader.read_exact(&mut current_path)?;
                        for byte in current_path.iter_mut() {
                            let char = *byte ^ advance_magic(&mut reader_magic) as u8;
                            if char == b'\\' {
                                *byte = b'/';
                            } else {
                                *byte = char;
                            }
                        }
                        let current_path =
                            String::from_utf8(current_path).map_err(|_| InvalidData)?;
                        let current_entry = trie.get_mut_file(&current_path).ok_or(InvalidData)?;
                        reader.seek(SeekFrom::Start(current_entry.offset))?;
                        advance_magic(&mut reader_magic);

                        writer.write_all(
                            &(current_path_len ^ advance_magic(&mut writer_magic)).to_le_bytes(),
                        )?;
                        writer.write_all(
                            &current_path
                                .as_str()
                                .bytes()
                                .map(|b| {
                                    let b = if b == b'/' { b'\\' } else { b };
                                    b ^ advance_magic(&mut writer_magic) as u8
                                })
                                .collect_vec(),
                        )?;
                        writer.write_all(
                            &(current_entry.size as u32 ^ advance_magic(&mut writer_magic))
                                .to_le_bytes(),
                        )?;
                        std::io::copy(
                            &mut read_file_xor(
                                &mut read_file_xor(
                                    &mut (&mut reader).take(current_entry.size),
                                    reader_magic,
                                ),
                                writer_magic,
                            ),
                            &mut writer,
                        )?;

                        current_entry.offset = current_entry
                            .offset
                            .checked_sub(entry.size + path_len + 8)
                            .ok_or(InvalidData)?;
                        current_entry.start_magic = writer_magic;
                    }

                    // Write the header of the modified file at the end of the temporary file
                    writer.write_all(
                        &(path_len as u32 ^ advance_magic(&mut writer_magic)).to_le_bytes(),
                    )?;
                    writer.write_all(
                        &self
                            .path
                            .as_str()
                            .bytes()
                            .map(|b| {
                                let b = if b == b'/' { b'\\' } else { b };
                                b ^ advance_magic(&mut writer_magic) as u8
                            })
                            .collect_vec(),
                    )?;
                    writer.write_all(
                        &(new_size as u32 ^ advance_magic(&mut writer_magic)).to_le_bytes(),
                    )?;
                    writer.flush()?;
                    drop(reader);
                    drop(writer);

                    // Write the contents of the temporary file into the archive, starting from
                    // where the modified file's header was
                    tmp.seek(SeekFrom::Start(0))?;
                    archive.seek(SeekFrom::Start(
                        entry.offset.checked_sub(path_len + 8).ok_or(InvalidData)?,
                    ))?;
                    std::io::copy(&mut tmp, <T as Write>::by_ref(&mut archive))?;

                    entry.start_magic = writer_magic;
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

                3 if !is_last => {
                    let mut tmp = crate::host::File::new()?;

                    archive.seek(SeekFrom::Start(entry.offset + entry.size))?;
                    std::io::copy(<T as Read>::by_ref(&mut archive), &mut tmp)?;
                    tmp.flush()?;

                    tmp.seek(SeekFrom::Start(0))?;
                    archive.seek(SeekFrom::Start(entry.offset))?;
                    std::io::copy(&mut tmp, <T as Write>::by_ref(&mut archive))?;

                    // Find all of the files in the archive with offsets greater than the original
                    // offset of the modified file and decrement them accordingly
                    archive.seek(SeekFrom::Start(12))?;
                    let mut reader = BufReader::new(<T as Read>::by_ref(&mut archive));
                    let mut headers = Vec::new();
                    while let Ok(current_offset) = read_u32_xor(&mut reader, self.base_magic) {
                        if current_offset == 0 {
                            break;
                        }
                        let current_offset = current_offset as u64;
                        reader.seek(SeekFrom::Current(8))?;
                        let current_path_len = read_u32_xor(&mut reader, self.base_magic)?;
                        if current_offset <= entry.offset {
                            reader.seek(SeekFrom::Current(current_path_len as i64))?;
                            continue;
                        }
                        let current_offset =
                            current_offset.checked_sub(entry.size).ok_or(InvalidData)?;

                        let mut current_path = vec![0; current_path_len as usize];
                        reader.read_exact(&mut current_path)?;
                        for (i, byte) in current_path.iter_mut().enumerate() {
                            let char = *byte ^ (self.base_magic >> (8 * (i % 4))) as u8;
                            if char == b'\\' {
                                *byte = b'/';
                            } else {
                                *byte = char;
                            }
                        }
                        let current_path =
                            String::from_utf8(current_path).map_err(|_| InvalidData)?;
                        trie.get_mut_file(current_path).ok_or(InvalidData)?.offset = current_offset;
                        headers.push((
                            reader
                                .stream_position()?
                                .checked_sub(current_path_len as u64 + 16)
                                .ok_or(InvalidData)?,
                            current_offset as u32,
                        ));
                    }
                    drop(reader);
                    let mut writer = BufWriter::new(<T as Write>::by_ref(&mut archive));
                    for (position, offset) in headers {
                        writer.seek(SeekFrom::Start(position))?;
                        writer.write_all(&(offset ^ self.base_magic).to_le_bytes())?;
                    }
                    writer.flush()?;
                    drop(writer);
                }

                3 => {}

                _ => return Err(InvalidData.into()),
            }

            entry.offset = archive_len.checked_sub(old_size).ok_or(InvalidData)?;
            entry.size = new_size;
            *trie.get_mut_file(&self.path).ok_or(InvalidData)? = entry;
        }

        let tmp_stream_position = self.tmp.stream_position()?;
        self.tmp.flush()?;
        self.tmp.seek(SeekFrom::Start(0))?;

        archive.seek(SeekFrom::Start(entry.offset))?;
        let mut reader = BufReader::new(&mut self.tmp);
        let mut writer = BufWriter::new(<T as Write>::by_ref(&mut archive));
        std::io::copy(
            &mut read_file_xor(&mut reader, entry.start_magic),
            &mut writer,
        )?;
        drop(reader);
        drop(writer);

        self.tmp.seek(SeekFrom::Start(tmp_stream_position))?;

        if old_size > new_size {
            archive.set_len(
                archive_len
                    .checked_sub(old_size)
                    .ok_or(InvalidData)?
                    .checked_add(new_size)
                    .ok_or(InvalidData)?,
            )?;
        }
        archive.flush()?;
        archive.seek(SeekFrom::Start(stream_position))?;
        self.modified = false;
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
    fn metadata(&self) -> std::io::Result<Metadata> {
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
        let mut tmp = crate::host::File::new()?;

        {
            let trie = self.trie.read();
            let mut archive = self.archive.lock();
            let entry = *trie.get_file(path).ok_or(Error::NotExist)?;
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
            trie: flags.contains(OpenFlags::Write).then(|| self.trie.clone()),
            path: path.to_owned(),
            read_allowed: flags.contains(OpenFlags::Read),
            tmp,
            modified: false,
            version: self.version,
            base_magic: self.base_magic,
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
