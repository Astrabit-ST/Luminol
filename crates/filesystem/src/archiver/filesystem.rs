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

use async_std::io::{BufReader as AsyncBufReader, BufWriter as AsyncBufWriter};
use itertools::Itertools;
use rand::Rng;
use std::io::{
    prelude::*,
    BufReader, BufWriter,
    ErrorKind::{AlreadyExists, InvalidData},
    SeekFrom,
};

use super::{
    util::{
        advance_magic, move_file_and_truncate, read_file_xor, read_header, read_u32_xor,
        regress_magic,
    },
    HEADER,
};
use super::{Entry, File, Trie, MAGIC};
use crate::{archiver::util::read_file_xor_async, DirEntry, Error, Metadata, OpenFlags};

#[derive(Debug, Default)]
pub struct FileSystem<T> {
    pub(super) trie: std::sync::Arc<parking_lot::RwLock<Trie>>,
    pub(super) archive: std::sync::Arc<parking_lot::Mutex<T>>,
    pub(super) version: u8,
    pub(super) base_magic: u32,
}

impl<T> Clone for FileSystem<T> {
    fn clone(&self) -> Self {
        Self {
            trie: self.trie.clone(),
            archive: self.archive.clone(),
            version: self.version,
            base_magic: self.base_magic,
        }
    }
}

impl<T> FileSystem<T>
where
    T: crate::File,
{
    /// Creates a new archiver filesystem from a file containing an existing archive.
    pub fn new(mut file: T) -> Result<Self, Error> {
        file.seek(SeekFrom::Start(0))?;
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
                base_magic = base_magic.wrapping_mul(9).wrapping_add(3);

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

        Ok(Self {
            trie: std::sync::Arc::new(parking_lot::RwLock::new(trie)),
            archive: std::sync::Arc::new(parking_lot::Mutex::new(file)),
            version,
            base_magic,
        })
    }

    /// Creates a new archiver filesystem from the given files.
    /// The contents of the archive itself will be stored in `buffer`.
    pub async fn from_buffer_and_files<'a, I, P, R>(
        mut buffer: T,
        version: u8,
        files: I,
    ) -> Result<Self, Error>
    where
        T: futures_lite::AsyncWrite + futures_lite::AsyncSeek + Unpin,
        I: Iterator<Item = Result<(&'a P, u32, R), Error>>,
        P: AsRef<camino::Utf8Path> + 'a,
        R: futures_lite::AsyncRead + Unpin,
    {
        use futures_lite::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

        buffer.set_len(0)?;
        AsyncSeekExt::seek(&mut buffer, SeekFrom::Start(0)).await?;

        let mut writer = AsyncBufWriter::new(&mut buffer);
        writer.write_all(HEADER).await?;
        writer.write_all(&[version]).await?;

        let mut trie = Trie::new();

        match version {
            1 | 2 => {
                let mut magic = MAGIC;
                let mut header_offset = 8;

                for result in files {
                    let (path, size, file) = result?;
                    let reader = AsyncBufReader::new(file.take(size as u64));
                    let path = path.as_ref();
                    let header_size = path.as_str().bytes().len() as u64 + 8;

                    // Write the header
                    writer
                        .write_all(
                            &(path.as_str().bytes().len() as u32 ^ advance_magic(&mut magic))
                                .to_le_bytes(),
                        )
                        .await?;
                    writer
                        .write_all(
                            &path
                                .as_str()
                                .bytes()
                                .map(|b| {
                                    let b = if b == b'/' { b'\\' } else { b };
                                    b ^ advance_magic(&mut magic) as u8
                                })
                                .collect_vec(),
                        )
                        .await?;
                    writer
                        .write_all(&(size ^ advance_magic(&mut magic)).to_le_bytes())
                        .await?;

                    // Write the file contents
                    async_std::io::copy(&mut read_file_xor_async(reader, magic), &mut writer)
                        .await?;

                    trie.create_file(
                        path,
                        Entry {
                            header_offset,
                            body_offset: header_offset + header_size,
                            size: size as u64,
                            start_magic: magic,
                        },
                    );

                    header_offset += header_size + size as u64;
                }

                writer.flush().await?;
                drop(writer);
                Ok(Self {
                    trie: std::sync::Arc::new(parking_lot::RwLock::new(trie)),
                    archive: std::sync::Arc::new(parking_lot::Mutex::new(buffer)),
                    version,
                    base_magic: MAGIC,
                })
            }

            3 => {
                let mut tmp = crate::host::File::new()?;
                let mut tmp_writer = AsyncBufWriter::new(&mut tmp);
                let mut entries = if let (_, Some(upper_bound)) = files.size_hint() {
                    Vec::with_capacity(upper_bound)
                } else {
                    Vec::new()
                };

                let base_magic: u32 = rand::thread_rng().gen();
                writer
                    .write_all(&(base_magic.wrapping_sub(3).wrapping_mul(954437177)).to_le_bytes())
                    .await?;
                let mut header_offset = 12;
                let mut body_offset = 0;

                for result in files {
                    let (path, size, file) = result?;
                    let reader = AsyncBufReader::new(file.take(size as u64));
                    let path = path.as_ref();
                    let entry_magic: u32 = rand::thread_rng().gen();

                    // Write the header to the buffer, except for the offset
                    writer.seek(SeekFrom::Current(4)).await?;
                    writer.write_all(&(size ^ base_magic).to_le_bytes()).await?;
                    writer
                        .write_all(&(entry_magic ^ base_magic).to_le_bytes())
                        .await?;
                    writer
                        .write_all(&(path.as_str().bytes().len() as u32 ^ base_magic).to_le_bytes())
                        .await?;
                    writer
                        .write_all(
                            &path
                                .as_str()
                                .bytes()
                                .enumerate()
                                .map(|(i, b)| {
                                    let b = if b == b'/' { b'\\' } else { b };
                                    b ^ (base_magic >> (8 * (i % 4))) as u8
                                })
                                .collect_vec(),
                        )
                        .await?;

                    // Write the actual file contents to a temporary file
                    async_std::io::copy(
                        &mut read_file_xor_async(reader, entry_magic),
                        &mut tmp_writer,
                    )
                    .await?;

                    entries.push((
                        path.to_owned(),
                        Entry {
                            header_offset,
                            body_offset,
                            size: size as u64,
                            start_magic: entry_magic,
                        },
                    ));

                    header_offset += path.as_str().bytes().len() as u64 + 16;
                    body_offset += size as u64;
                }

                // Write the terminator at the end of the buffer
                writer.write_all(&base_magic.to_le_bytes()).await?;

                // Write the contents of the temporary file to the buffer after the terminator
                tmp_writer.flush().await?;
                drop(tmp_writer);
                AsyncSeekExt::seek(&mut tmp, SeekFrom::Start(0)).await?;
                async_std::io::copy(&mut tmp, &mut writer).await?;

                // Write the offsets into the header now that we know the total size of the files
                let header_size = header_offset + 4;
                for (path, mut entry) in entries {
                    entry.body_offset += header_size;
                    writer.seek(SeekFrom::Start(entry.header_offset)).await?;
                    writer
                        .write_all(&(entry.body_offset as u32 ^ base_magic).to_le_bytes())
                        .await?;
                    trie.create_file(path, entry);
                }

                writer.flush().await?;
                drop(writer);
                Ok(Self {
                    trie: std::sync::Arc::new(parking_lot::RwLock::new(trie)),
                    archive: std::sync::Arc::new(parking_lot::Mutex::new(buffer)),
                    version,
                    base_magic,
                })
            }

            _ => Err(Error::NotSupported),
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

            if flags.contains(OpenFlags::Create) && !trie.contains_file(path) {
                created = true;
                match self.version {
                    1 | 2 => {
                        archive.seek(SeekFrom::Start(8))?;
                        let mut reader = BufReader::new(archive.as_file());
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
                        regress_magic(&mut magic);

                        let archive_len = archive.seek(SeekFrom::End(0))?;
                        let mut writer = BufWriter::new(archive.as_file());
                        writer.write_all(
                            &(path.as_str().bytes().len() as u32 ^ advance_magic(&mut magic))
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
                        writer.write_all(&advance_magic(&mut magic).to_le_bytes())?;
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
                        let mut reader = BufReader::new(archive.as_file());
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
                        std::io::copy(archive.as_file(), &mut tmp)?;
                        tmp.flush()?;

                        let magic: u32 = rand::thread_rng().gen();
                        let archive_len = archive.metadata()?.size as u32 + extra_data_len;
                        let mut writer = BufWriter::new(archive.as_file());
                        for (position, offset) in headers {
                            writer.seek(SeekFrom::Start(position))?;
                            writer.write_all(
                                &((offset + extra_data_len) ^ self.base_magic).to_le_bytes(),
                            )?;
                        }
                        writer.seek(SeekFrom::Start(position))?;
                        writer.write_all(&(archive_len ^ self.base_magic).to_le_bytes())?;
                        writer.write_all(&self.base_magic.to_le_bytes())?;
                        writer.write_all(&(magic ^ self.base_magic).to_le_bytes())?;
                        writer.write_all(
                            &(path.as_str().bytes().len() as u32 ^ self.base_magic).to_le_bytes(),
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
                let entry = *trie.get_file(path).ok_or(Error::NotExist)?;
                archive.seek(SeekFrom::Start(entry.body_offset))?;

                let mut adapter = BufReader::new(archive.as_file().take(entry.size));
                std::io::copy(
                    &mut read_file_xor(&mut adapter, entry.start_magic),
                    &mut tmp,
                )?;
                tmp.flush()?;
            } else if !trie.contains_file(path) {
                return Err(Error::NotExist);
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
        from: impl AsRef<camino::Utf8Path>,
        to: impl AsRef<camino::Utf8Path>,
    ) -> std::result::Result<(), Error> {
        let from = from.as_ref();
        let to = to.as_ref();

        let mut archive = self.archive.lock();
        let mut trie = self.trie.write();

        if trie.contains_dir(from) {
            return Err(Error::NotSupported);
        }
        if trie.contains(to) {
            return Err(Error::IoError(AlreadyExists.into()));
        }
        if !trie.contains_dir(from.parent().ok_or(Error::NotExist)?) {
            return Err(Error::NotExist);
        }
        let Some(old_entry) = trie.get_file(from).copied() else {
            return Err(Error::NotExist);
        };

        let archive_len = archive.metadata()?.size;
        let from_len = from.as_str().bytes().len();
        let to_len = to.as_str().bytes().len();

        if from_len != to_len {
            match self.version {
                1 | 2 => {
                    // Move the file contents into a temporary file
                    let mut tmp = crate::host::File::new()?;
                    archive.seek(SeekFrom::Start(old_entry.body_offset))?;
                    let mut reader = BufReader::new(archive.as_file().take(old_entry.size));
                    std::io::copy(
                        &mut read_file_xor(&mut reader, old_entry.start_magic),
                        &mut tmp,
                    )?;
                    tmp.flush()?;
                    drop(reader);

                    // Move the file to the end so that we can change the header size
                    move_file_and_truncate(
                        &mut archive,
                        &mut trie,
                        from,
                        self.version,
                        self.base_magic,
                    )?;
                    let mut new_entry = *trie
                        .get_file(from)
                        .ok_or(Error::IoError(InvalidData.into()))?;
                    trie.remove_file(from)
                        .ok_or(Error::IoError(InvalidData.into()))?;
                    new_entry.size = old_entry.size;

                    let mut magic = new_entry.start_magic;
                    regress_magic(&mut magic);
                    regress_magic(&mut magic);
                    for _ in from.as_str().bytes() {
                        regress_magic(&mut magic);
                    }

                    // Regenerate the header
                    archive.seek(SeekFrom::Start(new_entry.header_offset))?;
                    let mut writer = BufWriter::new(archive.as_file());
                    writer.write_all(&(to_len as u32 ^ advance_magic(&mut magic)).to_le_bytes())?;
                    writer.write_all(
                        &to.as_str()
                            .bytes()
                            .map(|b| {
                                let b = if b == b'/' { b'\\' } else { b };
                                b ^ advance_magic(&mut magic) as u8
                            })
                            .collect_vec(),
                    )?;
                    writer.write_all(
                        &(old_entry.size as u32 ^ advance_magic(&mut magic)).to_le_bytes(),
                    )?;

                    new_entry.start_magic = magic;

                    // Move the file contents to the end
                    tmp.seek(SeekFrom::Start(0))?;
                    let mut reader = BufReader::new(&mut tmp);
                    std::io::copy(&mut read_file_xor(&mut reader, magic), &mut writer)?;
                    writer.flush()?;
                    drop(writer);

                    trie.create_file(to, new_entry);
                }

                3 => {
                    // Move everything after the header into a temporary file
                    let mut tmp = crate::host::File::new()?;
                    archive.seek(SeekFrom::Start(
                        old_entry.header_offset + from_len as u64 + 16,
                    ))?;
                    std::io::copy(archive.as_file(), &mut tmp)?;
                    tmp.flush()?;

                    // Change the path
                    archive.seek(SeekFrom::Start(old_entry.header_offset + 12))?;
                    let mut writer = BufWriter::new(archive.as_file());
                    writer.write_all(&(to_len as u32 ^ self.base_magic).to_le_bytes())?;
                    writer.write_all(
                        &to.as_str()
                            .bytes()
                            .enumerate()
                            .map(|(i, b)| {
                                let b = if b == b'/' { b'\\' } else { b };
                                b ^ (self.base_magic >> (8 * (i % 4))) as u8
                            })
                            .collect_vec(),
                    )?;
                    trie.remove_file(from)
                        .ok_or(Error::IoError(InvalidData.into()))?;
                    trie.create_file(to, old_entry);

                    // Move everything else back
                    tmp.seek(SeekFrom::Start(0))?;
                    std::io::copy(&mut tmp, &mut writer)?;
                    writer.flush()?;
                    drop(writer);

                    // Update all of the offsets in the headers
                    archive.seek(SeekFrom::Start(12))?;
                    let mut reader = BufReader::new(archive.as_file());
                    let mut headers = Vec::new();
                    while let Ok(current_body_offset) = read_u32_xor(&mut reader, self.base_magic) {
                        if current_body_offset == 0 {
                            break;
                        }
                        let current_header_offset = reader
                            .stream_position()?
                            .checked_sub(4)
                            .ok_or(Error::IoError(InvalidData.into()))?;
                        reader.seek(SeekFrom::Current(8))?;
                        let current_path_len = read_u32_xor(&mut reader, self.base_magic)?;

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
                        let current_path = String::from_utf8(current_path)
                            .map_err(|_| Error::IoError(InvalidData.into()))?;

                        let current_body_offset = (current_body_offset as u64)
                            .checked_add_signed(to_len as i64 - from_len as i64)
                            .ok_or(Error::IoError(InvalidData.into()))?;
                        trie.get_mut_file(current_path)
                            .ok_or(Error::IoError(InvalidData.into()))?
                            .body_offset = current_body_offset;
                        headers.push((current_header_offset, current_body_offset as u32));
                    }
                    drop(reader);
                    let mut writer = BufWriter::new(archive.as_file());
                    for (position, offset) in headers {
                        writer.seek(SeekFrom::Start(position))?;
                        writer.write_all(&(offset ^ self.base_magic).to_le_bytes())?;
                    }
                    writer.flush()?;
                    drop(writer);
                }

                _ => return Err(Error::IoError(InvalidData.into())),
            }

            if to_len < from_len {
                archive.set_len(
                    archive_len
                        .checked_add_signed(to_len as i64 - from_len as i64)
                        .ok_or(Error::IoError(InvalidData.into()))?,
                )?;
                archive.flush()?;
            }
        } else {
            match self.version {
                1 | 2 => {
                    let mut magic = old_entry.start_magic;
                    for _ in from.as_str().bytes() {
                        regress_magic(&mut magic);
                    }
                    archive.seek(SeekFrom::Start(old_entry.header_offset + 4))?;
                    archive.write_all(
                        &to.as_str()
                            .bytes()
                            .map(|b| {
                                let b = if b == b'/' { b'\\' } else { b };
                                b ^ advance_magic(&mut magic) as u8
                            })
                            .collect_vec(),
                    )?;
                    archive.flush()?;
                }

                3 => {
                    archive.seek(SeekFrom::Start(old_entry.header_offset + 16))?;
                    archive.write_all(
                        &to.as_str()
                            .bytes()
                            .enumerate()
                            .map(|(i, b)| {
                                let b = if b == b'/' { b'\\' } else { b };
                                b ^ (self.base_magic >> (8 * (i % 4))) as u8
                            })
                            .collect_vec(),
                    )?;
                    archive.flush()?;
                }

                _ => return Err(Error::IoError(InvalidData.into())),
            }
        }

        Ok(())
    }

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        let path = path.as_ref();
        let mut trie = self.trie.write();
        if trie.contains_file(path) {
            return Err(Error::IoError(AlreadyExists.into()));
        }
        trie.create_dir(path);
        Ok(())
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

        move_file_and_truncate(&mut archive, &mut trie, path, self.version, self.base_magic)?;

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
                let path = if path == "" {
                    name.into()
                } else {
                    format!("{path}/{name}").into()
                };
                let metadata = self.metadata(&path)?;
                Ok(DirEntry { path, metadata })
            })
            .try_collect()
        } else {
            Err(Error::NotExist)
        }
    }
}
