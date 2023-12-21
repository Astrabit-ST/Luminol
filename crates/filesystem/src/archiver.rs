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
use rand::Rng;
use std::io::{
    prelude::*,
    BufReader, BufWriter,
    ErrorKind::{InvalidData, PermissionDenied},
    SeekFrom,
};

use crate::File as _;
use crate::{DirEntry, Error, Metadata, OpenFlags};

type Trie = crate::FileSystemTrie<Entry>;

#[derive(Debug, Default)]
pub struct FileSystem<T> {
    trie: std::sync::Arc<parking_lot::RwLock<Trie>>,
    archive: std::sync::Arc<parking_lot::Mutex<T>>,
    version: u8,
    base_magic: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Entry {
    header_offset: u64,
    body_offset: u64,
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

                    let stream_position = reader.stream_position()?;
                    let entry = Entry {
                        size: entry_len as u64,
                        header_offset: stream_position
                            .checked_sub(path_len as u64 + 8)
                            .ok_or(Error::IoError(InvalidData.into()))?,
                        body_offset: stream_position,
                        start_magic: magic,
                    };

                    trie.create_file(path, entry);

                    reader.seek(SeekFrom::Start(entry.body_offset + entry.size))?;
                }
            }
            3 => {
                let mut u32_buf = [0; 4];
                reader.read_exact(&mut u32_buf)?;

                base_magic = u32::from_le_bytes(u32_buf);
                base_magic = (base_magic * 9) + 3;

                while let Ok(body_offset) = read_u32_xor(&mut reader, base_magic) {
                    if body_offset == 0 {
                        break;
                    }
                    let header_offset = reader
                        .stream_position()?
                        .checked_sub(4)
                        .ok_or(Error::IoError(InvalidData.into()))?;

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
                        header_offset,
                        body_offset: body_offset as u64,
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
    trie: Option<std::sync::Arc<parking_lot::RwLock<Trie>>>,
    path: camino::Utf8PathBuf,
    read_allowed: bool,
    tmp: crate::host::File,
    modified: parking_lot::Mutex<bool>,
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

/// Moves a file within an archive to the end of the archive and truncates the file's length to 0.
/// Does NOT truncate the actual archive to the correct length afterwards.
fn move_file_and_truncate<T>(
    archive: &mut parking_lot::MutexGuard<'_, T>,
    trie: &mut parking_lot::RwLockWriteGuard<'_, Trie>,
    path: impl AsRef<camino::Utf8Path>,
    version: u8,
    base_magic: u32,
) -> std::io::Result<()>
where
    T: crate::File,
{
    let path = path.as_ref();
    let path_len = path.as_str().bytes().len() as u64;
    let archive_len = archive.metadata()?.size;
    archive.seek(SeekFrom::Start(0))?;

    let mut entry = *trie.get_file(&path).ok_or(InvalidData)?;
    match version {
        1 | 2 => {
            let mut tmp = crate::host::File::new()?;
            archive.seek(SeekFrom::Start(entry.body_offset + entry.size))?;
            let mut reader = BufReader::new(archive.as_file());
            let mut writer = BufWriter::new(&mut tmp);

            let mut reader_magic = entry.start_magic;

            // Determine what the magic value was for the beginning of the modified file's
            // header
            let mut writer_magic = entry.start_magic;
            regress_magic(&mut writer_magic);
            regress_magic(&mut writer_magic);
            for _ in path.as_str().bytes() {
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
                let current_path = String::from_utf8(current_path).map_err(|_| InvalidData)?;
                let current_entry = trie.get_mut_file(&current_path).ok_or(InvalidData)?;
                reader.seek(SeekFrom::Start(current_entry.body_offset))?;
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
                    &(current_entry.size as u32 ^ advance_magic(&mut writer_magic)).to_le_bytes(),
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

                current_entry.header_offset = current_entry
                    .header_offset
                    .checked_sub(entry.size + path_len + 8)
                    .ok_or(InvalidData)?;
                current_entry.body_offset = current_entry
                    .body_offset
                    .checked_sub(entry.size + path_len + 8)
                    .ok_or(InvalidData)?;
                current_entry.start_magic = writer_magic;
            }

            // Write the header of the modified file at the end of the temporary file
            writer
                .write_all(&(path_len as u32 ^ advance_magic(&mut writer_magic)).to_le_bytes())?;
            writer.write_all(
                &path
                    .as_str()
                    .bytes()
                    .map(|b| {
                        let b = if b == b'/' { b'\\' } else { b };
                        b ^ advance_magic(&mut writer_magic) as u8
                    })
                    .collect_vec(),
            )?;
            writer.write_all(&advance_magic(&mut writer_magic).to_le_bytes())?;
            writer.flush()?;
            drop(reader);
            drop(writer);

            // Write the contents of the temporary file into the archive, starting from
            // where the modified file's header was
            tmp.seek(SeekFrom::Start(0))?;
            archive.seek(SeekFrom::Start(entry.header_offset))?;
            std::io::copy(&mut tmp, archive.as_file())?;

            entry.start_magic = writer_magic;
            entry.body_offset = archive_len.checked_sub(entry.size).ok_or(InvalidData)?;
            entry.header_offset = entry
                .body_offset
                .checked_sub(path_len + 8)
                .ok_or(InvalidData)?;
            entry.size = 0;
            *trie.get_mut_file(&path).ok_or(InvalidData)? = entry;

            Ok(())
        }

        3 => {
            let mut tmp = crate::host::File::new()?;

            // Copy the contents of the files after the modified file into a temporary file
            archive.seek(SeekFrom::Start(entry.body_offset + entry.size))?;
            std::io::copy(archive.as_file(), &mut tmp)?;
            tmp.flush()?;

            // Copy the contents of the temporary file back into the archive starting from where
            // the modified file was
            tmp.seek(SeekFrom::Start(0))?;
            archive.seek(SeekFrom::Start(entry.body_offset))?;
            std::io::copy(&mut tmp, archive.as_file())?;

            // Find all of the files in the archive with offsets greater than the original
            // offset of the modified file and decrement them accordingly
            archive.seek(SeekFrom::Start(12))?;
            let mut reader = BufReader::new(archive.as_file());
            let mut headers = Vec::new();
            while let Ok(current_body_offset) = read_u32_xor(&mut reader, base_magic) {
                if current_body_offset == 0 {
                    break;
                }
                let current_header_offset = reader
                    .stream_position()?
                    .checked_sub(4)
                    .ok_or(InvalidData)?;
                let current_body_offset = current_body_offset as u64;
                reader.seek(SeekFrom::Current(8))?;
                let current_path_len = read_u32_xor(&mut reader, base_magic)?;
                let should_truncate = current_header_offset == entry.header_offset;
                if current_body_offset <= entry.body_offset && !should_truncate {
                    reader.seek(SeekFrom::Current(current_path_len as i64))?;
                    continue;
                }

                let mut current_path = vec![0; current_path_len as usize];
                reader.read_exact(&mut current_path)?;
                for (i, byte) in current_path.iter_mut().enumerate() {
                    let char = *byte ^ (base_magic >> (8 * (i % 4))) as u8;
                    if char == b'\\' {
                        *byte = b'/';
                    } else {
                        *byte = char;
                    }
                }
                let current_path = String::from_utf8(current_path).map_err(|_| InvalidData)?;

                let current_body_offset = if should_truncate {
                    archive_len.checked_sub(entry.size).ok_or(InvalidData)?
                } else {
                    current_body_offset
                        .checked_sub(entry.size)
                        .ok_or(InvalidData)?
                };

                trie.get_mut_file(current_path)
                    .ok_or(InvalidData)?
                    .body_offset = current_body_offset;
                headers.push((
                    current_header_offset,
                    current_body_offset as u32,
                    should_truncate,
                ));
            }
            drop(reader);
            let mut writer = BufWriter::new(archive.as_file());
            for (position, offset, should_truncate) in headers {
                writer.seek(SeekFrom::Start(position))?;
                writer.write_all(&(offset ^ base_magic).to_le_bytes())?;
                if should_truncate {
                    writer.write_all(&base_magic.to_le_bytes())?;
                }
            }
            writer.flush()?;
            drop(writer);

            trie.get_mut_file(&path).ok_or(InvalidData)?.size = 0;

            Ok(())
        }

        _ => Err(InvalidData.into()),
    }
}

impl<T> std::io::Write for File<T>
where
    T: crate::File,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.archive.is_some() {
            let mut modified = self.modified.lock();
            *modified = true;
            let count = self.tmp.write(buf)?;
            Ok(count)
        } else {
            Err(PermissionDenied.into())
        }
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        if self.archive.is_some() {
            let mut modified = self.modified.lock();
            *modified = true;
            let count = self.tmp.write_vectored(bufs)?;
            Ok(count)
        } else {
            Err(PermissionDenied.into())
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut modified = self.modified.lock();
        if !*modified {
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
        let archive_len = archive.metadata()?.size;

        let tmp_stream_position = self.tmp.stream_position()?;
        self.tmp.flush()?;
        self.tmp.seek(SeekFrom::Start(0))?;

        // If the size of the file has changed, rotate the archive to place the file at the end of
        // the archive before writing the new contents of the file
        let mut entry = *trie.get_file(&self.path).ok_or(InvalidData)?;
        let old_size = entry.size;
        let new_size = self.tmp.metadata()?.size;
        if old_size != new_size {
            move_file_and_truncate(
                &mut archive,
                &mut trie,
                &self.path,
                self.version,
                self.base_magic,
            )?;
            entry = *trie.get_file(&self.path).ok_or(InvalidData)?;

            // Write the new length of the file to the archive
            match self.version {
                1 | 2 => {
                    let mut magic = entry.start_magic;
                    regress_magic(&mut magic);
                    archive.seek(SeekFrom::Start(
                        entry.body_offset.checked_sub(4).ok_or(InvalidData)?,
                    ))?;
                    archive.write_all(&(new_size as u32 ^ magic).to_le_bytes())?;
                }

                3 => {
                    archive.seek(SeekFrom::Start(entry.header_offset as u64 + 4))?;
                    archive.write_all(&(new_size as u32 ^ self.base_magic).to_le_bytes())?;
                }

                _ => return Err(InvalidData.into()),
            }

            // Write the new length of the file to the trie
            trie.get_mut_file(&self.path).ok_or(InvalidData)?.size = new_size;
        }

        // Now write the new contents of the file
        archive.seek(SeekFrom::Start(entry.body_offset))?;
        let mut reader = BufReader::new(&mut self.tmp);
        std::io::copy(
            &mut read_file_xor(&mut reader, entry.start_magic),
            <T as Write>::by_ref(&mut archive),
        )?;
        drop(reader);
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
        *modified = false;
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
        if self.archive.is_some() {
            let mut modified = self.modified.lock();
            *modified = true;
            self.tmp.set_len(new_size)
        } else {
            Err(PermissionDenied.into())
        }
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
        let path = path.as_ref();
        let mut tmp = crate::host::File::new()?;
        let mut created = false;

        {
            let mut archive = self.archive.lock();
            let mut trie = self.trie.write();
            let entry = *trie.get_file(path).ok_or(Error::NotExist)?;
            archive.seek(SeekFrom::Start(entry.body_offset))?;

            if flags.contains(OpenFlags::Create) && !trie.contains_file(&path) {
                created = true;
                match self.version {
                    1 | 2 => {
                        archive.seek(SeekFrom::Start(8))?;
                        let mut reader = BufReader::new(<T as Read>::by_ref(&mut archive));
                        let mut magic = MAGIC;
                        while let Ok(path_len) =
                            read_u32_xor(&mut reader, advance_magic(&mut magic))
                        {
                            for _ in 0..path_len {
                                advance_magic(&mut magic);
                            }
                            reader.seek(SeekFrom::Current(path_len as i64))?;
                            let entry_len = read_u32_xor(&mut reader, advance_magic(&mut magic))?;
                            reader.seek(SeekFrom::Current(entry_len as i64))?;
                        }
                        drop(reader);

                        let archive_len = archive.seek(SeekFrom::End(0))?;
                        let mut writer = BufWriter::new(<T as Write>::by_ref(&mut archive));
                        writer.write_all(
                            &mut (path.as_str().bytes().len() as u32 ^ advance_magic(&mut magic))
                                .to_le_bytes(),
                        )?;
                        writer.write_all(
                            &path
                                .as_str()
                                .bytes()
                                .map(|b| {
                                    let b = if b == b'/' { b'\\' } else { b };
                                    b ^ advance_magic(&mut magic) as u8
                                })
                                .collect_vec(),
                        )?;
                        writer.write_all(&mut advance_magic(&mut magic).to_le_bytes())?;
                        writer.flush()?;
                        drop(writer);

                        trie.create_file(
                            path,
                            Entry {
                                header_offset: archive_len,
                                body_offset: archive_len + path.as_str().bytes().len() as u64 + 8,
                                size: 0,
                                start_magic: magic,
                            },
                        );
                    }

                    3 => {
                        let mut tmp = crate::host::File::new()?;

                        let extra_data_len = path.as_str().bytes().len() as u32 + 16;
                        let mut headers = Vec::new();

                        archive.seek(SeekFrom::Start(12))?;
                        let mut reader = BufReader::new(<T as Read>::by_ref(&mut archive));
                        let mut position = 12;
                        while let Ok(offset) = read_u32_xor(&mut reader, self.base_magic) {
                            if offset == 0 {
                                break;
                            }
                            headers.push((position, offset));
                            reader.seek(SeekFrom::Current(8))?;
                            let path_len = read_u32_xor(&mut reader, self.base_magic)?;
                            position = reader.seek(SeekFrom::Current(path_len as i64))?;
                        }
                        drop(reader);

                        archive.seek(SeekFrom::Start(position))?;
                        std::io::copy(<T as Read>::by_ref(&mut archive), &mut tmp)?;
                        tmp.flush()?;

                        let magic: u32 = rand::thread_rng().gen();
                        let archive_len = archive.metadata()?.size as u32 + extra_data_len;
                        let mut writer = BufWriter::new(<T as Write>::by_ref(&mut archive));
                        for (position, offset) in headers {
                            writer.seek(SeekFrom::Start(position))?;
                            writer.write_all(
                                &mut ((offset + extra_data_len) ^ self.base_magic).to_le_bytes(),
                            )?;
                        }
                        writer.seek(SeekFrom::Start(position))?;
                        writer.write_all(&mut (archive_len ^ self.base_magic).to_le_bytes())?;
                        writer.write_all(&mut self.base_magic.to_le_bytes())?;
                        writer.write_all(&mut (magic ^ self.base_magic).to_le_bytes())?;
                        writer.write_all(
                            &mut (path.as_str().bytes().len() as u32 ^ self.base_magic)
                                .to_le_bytes(),
                        )?;
                        writer.write_all(
                            &path
                                .as_str()
                                .bytes()
                                .enumerate()
                                .map(|(i, b)| {
                                    let b = if b == b'/' { b'\\' } else { b };
                                    b ^ (self.base_magic >> (8 * (i % 4))) as u8
                                })
                                .collect_vec(),
                        )?;
                        tmp.seek(SeekFrom::Start(0))?;
                        std::io::copy(&mut tmp, &mut writer)?;
                        writer.flush()?;
                        drop(writer);

                        trie.create_file(
                            path,
                            Entry {
                                header_offset: position,
                                body_offset: archive_len as u64,
                                size: 0,
                                start_magic: magic,
                            },
                        );
                    }

                    _ => return Err(Error::NotSupported),
                }
            } else if !flags.contains(OpenFlags::Truncate) {
                let mut adapter =
                    BufReader::new(<T as Read>::by_ref(&mut archive).take(entry.size));
                std::io::copy(
                    &mut read_file_xor(&mut adapter, entry.start_magic),
                    &mut tmp,
                )?;
                tmp.flush()?;
            }
        }

        tmp.seek(SeekFrom::Start(0))?;
        Ok(File {
            archive: flags
                .contains(OpenFlags::Write)
                .then(|| self.archive.clone()),
            trie: flags.contains(OpenFlags::Write).then(|| self.trie.clone()),
            path: path.to_owned(),
            read_allowed: flags.contains(OpenFlags::Read),
            tmp,
            modified: parking_lot::Mutex::new(
                !created && flags.contains(OpenFlags::Write) && flags.contains(OpenFlags::Truncate),
            ),
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

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        let path = path.as_ref();
        if !self.trie.read().contains_dir(path) {
            return Err(Error::NotExist);
        }

        let paths = self
            .trie
            .read()
            .iter_prefix(path)
            .ok_or(Error::NotExist)?
            .map(|(k, _)| k)
            .collect_vec();
        for file_path in paths {
            self.remove_file(file_path)?;
        }

        self.trie
            .write()
            .remove_dir(path)
            .then_some(())
            .ok_or(Error::NotExist)?;
        Ok(())
    }

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        let path = path.as_ref();
        let path_len = path.as_str().bytes().len() as u64;
        let mut archive = self.archive.lock();
        let mut trie = self.trie.write();

        let entry = *trie.get_file(path).ok_or(Error::NotExist)?;
        let archive_len = archive.metadata()?.size;

        move_file_and_truncate(
            &mut archive,
            &mut trie,
            &path,
            self.version,
            self.base_magic,
        )?;

        match self.version {
            1 | 2 => {
                archive.set_len(
                    archive_len
                        .checked_sub(entry.size + path_len + 8)
                        .ok_or(Error::IoError(InvalidData.into()))?,
                )?;
                archive.flush()?;
            }

            3 => {
                // Remove the header of the deleted file
                let mut tmp = crate::host::File::new()?;
                archive.seek(SeekFrom::Start(entry.header_offset + path_len + 16))?;
                std::io::copy(archive.as_file(), &mut tmp)?;
                tmp.flush()?;
                tmp.seek(SeekFrom::Start(0))?;
                archive.seek(SeekFrom::Start(entry.header_offset))?;
                std::io::copy(&mut tmp, archive.as_file())?;

                archive.set_len(
                    archive_len
                        .checked_sub(entry.size + path_len + 16)
                        .ok_or(Error::IoError(InvalidData.into()))?,
                )?;
                archive.flush()?;
            }

            _ => return Err(Error::NotSupported),
        }

        trie.remove_file(path);
        Ok(())
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>, Error> {
        let path = path.as_ref();
        let trie = self.trie.read();
        if let Some(iter) = trie.iter_dir(path) {
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
