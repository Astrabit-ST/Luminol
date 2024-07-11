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

use async_std::io::{BufReader as AsyncBufReader, BufWriter as AsyncBufWriter};
use color_eyre::eyre::WrapErr;
use itertools::Itertools;
use rand::Rng;
use std::io::{prelude::*, BufReader, SeekFrom};

use super::util::{advance_magic, read_file_xor_async, read_header, read_u32_xor};
use super::{Entry, File, Trie, HEADER, MAGIC};
use crate::{Error, Result};

mod impls;

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
    pub fn new(mut file: T) -> Result<Self> {
        file.seek(SeekFrom::Start(0))
            .wrap_err("While detecting archive version")?;
        let mut reader = BufReader::new(&mut file);

        let version = read_header(&mut reader).wrap_err("While detecting archive version")?;

        let mut trie = crate::FileSystemTrie::new();

        let mut base_magic = MAGIC;

        let c = format!(
            "While performing initial parsing of the header of a version {version} archive"
        );

        match version {
            1 | 2 => {
                let mut magic = MAGIC;

                let mut i = 0;

                while let Ok(path_len) = read_u32_xor(&mut reader, advance_magic(&mut magic)) {
                    let mut path = vec![0; path_len as usize];
                    reader.read_exact(&mut path).wrap_err("").wrap_err_with(|| format!("While reading the path (path length = {path_len}) of file #{i} in the archive")).wrap_err_with(|| c.clone())?;
                    for byte in path.iter_mut() {
                        let char = *byte ^ advance_magic(&mut magic) as u8;
                        if char == b'\\' {
                            *byte = b'/';
                        } else {
                            *byte = char;
                        }
                    }
                    let path = camino::Utf8PathBuf::from(String::from_utf8(path).wrap_err_with(|| format!("While reading the path (path length = {path_len}) of file #{i} in the archive)")).wrap_err_with(|| c.clone())?);

                    let entry_len = read_u32_xor(&mut reader, advance_magic(&mut magic))
                        .wrap_err_with(|| {
                            format!("While reading the file length (path = {path:?}) of file #{i} in the archive")
                        })
                        .wrap_err_with(|| c.clone())?;

                    let stream_position = reader
                        .stream_position()
                        .wrap_err_with(|| {
                            format!("While reading the file length (path = {path:?}) of file #{i} in the archive")
                        })
                        .wrap_err_with(|| c.clone())?;
                    let entry = Entry {
                        size: entry_len as u64,
                        header_offset: stream_position
                            .checked_sub(path_len as u64 + 8)
                            .ok_or(Error::InvalidHeader).wrap_err_with(|| format!("While reading the file length (path = {path:?}) of file #{i} in the archive")).wrap_err_with(|| c.clone())?,
                        body_offset: stream_position,
                        start_magic: magic,
                    };

                    trie.create_file(path, entry);

                    reader
                        .seek(SeekFrom::Start(entry.body_offset + entry.size))
                        .wrap_err_with(|| {
                            format!(
                                "While seeking to offset {} to read file #{} in the archive",
                                entry.body_offset + entry.size,
                                i + 1
                            )
                        })
                        .wrap_err_with(|| c.clone())?;
                    i += 1;
                }
            }
            3 => {
                let mut u32_buf = [0; 4];
                reader
                    .read_exact(&mut u32_buf)
                    .wrap_err("While reading the base magic value of the archive")
                    .wrap_err_with(|| c.clone())?;

                base_magic = u32::from_le_bytes(u32_buf);
                base_magic = base_magic.wrapping_mul(9).wrapping_add(3);

                let mut i = 0;

                while let Ok(body_offset) = read_u32_xor(&mut reader, base_magic) {
                    if body_offset == 0 {
                        break;
                    }
                    let header_offset = reader
                        .stream_position()
                        .wrap_err_with(|| {
                            format!("While reading the file offset of file #{i} in the archive")
                        })
                        .wrap_err_with(|| c.clone())?
                        .checked_sub(4)
                        .ok_or(Error::InvalidHeader)
                        .wrap_err_with(|| {
                            format!("While reading the file offset of file #{i} in the archive")
                        })
                        .wrap_err_with(|| c.clone())?;

                    let entry_len = read_u32_xor(&mut reader, base_magic).wrap_err_with(|| format!("While reading the file length (file offset = {body_offset}) of file #{i} in the archive")).wrap_err_with(|| c.clone())?;
                    let magic = read_u32_xor(&mut reader, base_magic).wrap_err_with(|| format!("While reading the magic value (file offset = {body_offset}, file length = {entry_len}) of file #{i} in the archive")).wrap_err_with(|| c.clone())?;
                    let path_len = read_u32_xor(&mut reader, base_magic).wrap_err_with(|| format!("While reading the path length (file offset = {body_offset}, file length = {entry_len}) of file #{i} in the archive")).wrap_err_with(|| c.clone())?;

                    let mut path = vec![0; path_len as usize];
                    reader.read_exact(&mut path).wrap_err_with(|| format!("While reading the path (file offset = {body_offset}, file length = {entry_len}, path length = {path_len}) of file #{i} in the archive")).wrap_err_with(|| c.clone())?;
                    for (i, byte) in path.iter_mut().enumerate() {
                        let char = *byte ^ (base_magic >> (8 * (i % 4))) as u8;
                        if char == b'\\' {
                            *byte = b'/';
                        } else {
                            *byte = char;
                        }
                    }
                    let path = camino::Utf8PathBuf::from(String::from_utf8(path).wrap_err_with(|| format!("While reading the path (file offset = {body_offset}, file length = {entry_len}, path length = {path_len}) of file #{i} in the archive")).wrap_err_with(|| c.clone())?);

                    let entry = Entry {
                        size: entry_len as u64,
                        header_offset,
                        body_offset: body_offset as u64,
                        start_magic: magic,
                    };
                    trie.create_file(path, entry);
                    i += 1;
                }
            }
            _ => return Err(Error::InvalidArchiveVersion(version).into()),
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
    ) -> Result<Self>
    where
        T: futures_lite::AsyncWrite + futures_lite::AsyncSeek + Unpin,
        I: Iterator<Item = Result<(&'a P, u32, R)>>,
        P: AsRef<camino::Utf8Path> + 'a,
        R: futures_lite::AsyncRead + Unpin,
    {
        use futures_lite::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

        let c = format!("While creating a new version {version} archive");

        buffer
            .set_len(0)
            .wrap_err("While clearing the archive")
            .wrap_err_with(|| c.clone())?;
        AsyncSeekExt::seek(&mut buffer, SeekFrom::Start(0))
            .await
            .wrap_err("While clearing the archive")
            .wrap_err_with(|| c.clone())?;

        let mut writer = AsyncBufWriter::new(&mut buffer);
        writer
            .write_all(HEADER)
            .await
            .wrap_err("While writing the archive version")
            .wrap_err_with(|| c.clone())?;
        writer
            .write_all(&[version])
            .await
            .wrap_err("While writing the archive version")
            .wrap_err_with(|| c.clone())?;

        let mut trie = Trie::new();

        match version {
            1 | 2 => {
                let mut magic = MAGIC;
                let mut header_offset = 8;

                for (i, result) in files.enumerate() {
                    let (path, size, file) = result
                        .wrap_err_with(|| {
                            format!(
                                "While getting file #{i} to add to the archive from the iterator"
                            )
                        })
                        .wrap_err_with(|| c.clone())?;
                    let reader = AsyncBufReader::new(file.take(size as u64));
                    let path = path.as_ref();
                    let header_size = path.as_str().bytes().len() as u64 + 8;

                    // Write the header
                    writer
                        .write_all(
                            &(path.as_str().bytes().len() as u32 ^ advance_magic(&mut magic))
                                .to_le_bytes(),
                        )
                        .await.wrap_err_with(|| format!("While writing the path length of file #{i} (path = {path:?}, file length = {size}) to the archive")).wrap_err_with(|| c.clone())?;
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
                        .await.wrap_err_with(|| format!("While writing the path of file #{i} (path = {path:?}, file length = {size}) to the archive")).wrap_err_with(|| c.clone())?;
                    writer
                        .write_all(&(size ^ advance_magic(&mut magic)).to_le_bytes())
                        .await.wrap_err_with(|| format!("While writing the file length of file #{i} (path = {path:?}, file length = {size}) to the archive")).wrap_err_with(|| c.clone())?;

                    // Write the file contents
                    async_std::io::copy(&mut read_file_xor_async(reader, magic), &mut writer)
                        .await.wrap_err_with(|| format!("While writing the contents of file #{i} (path = {path:?}, file length = {size}) to the archive")).wrap_err_with(|| c.clone())?;

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

                writer
                    .flush()
                    .await
                    .wrap_err("While flushing the archive after writing its contents")
                    .wrap_err_with(|| c.clone())?;
                drop(writer);
                Ok(Self {
                    trie: std::sync::Arc::new(parking_lot::RwLock::new(trie)),
                    archive: std::sync::Arc::new(parking_lot::Mutex::new(buffer)),
                    version,
                    base_magic: MAGIC,
                })
            }

            3 => {
                let mut tmp = crate::host::File::new()
                    .wrap_err("While creating a temporary file")
                    .wrap_err_with(|| c.clone())?;
                let mut tmp_writer = AsyncBufWriter::new(&mut tmp);
                let mut entries = if let (_, Some(upper_bound)) = files.size_hint() {
                    Vec::with_capacity(upper_bound)
                } else {
                    Vec::new()
                };

                let base_magic: u32 = rand::thread_rng().gen();
                writer
                    .write_all(&(base_magic.wrapping_sub(3).wrapping_mul(954437177)).to_le_bytes())
                    .await
                    .wrap_err("While writing the archive base magic value")
                    .wrap_err_with(|| c.clone())?;
                let mut header_offset = 12;
                let mut body_offset = 0;

                for (i, result) in files.enumerate() {
                    let (path, size, file) = result
                        .wrap_err_with(|| {
                            format!(
                                "While getting file #{i} to write to the archive from the iterator"
                            )
                        })
                        .wrap_err_with(|| c.clone())?;
                    let reader = AsyncBufReader::new(file.take(size as u64));
                    let path = path.as_ref();
                    let entry_magic: u32 = rand::thread_rng().gen();

                    // Write the header to the buffer, except for the offset
                    writer.seek(SeekFrom::Current(4)).await.wrap_err_with(|| format!("While writing the file length of file #{i} (path = {path:?}, file length = {size}) to the archive")).wrap_err_with(|| c.clone())?;
                    writer.write_all(&(size ^ base_magic).to_le_bytes()).await.wrap_err_with(|| format!("While writing the file length of file #{i} (path = {path:?}, file length = {size}) to the archive")).wrap_err_with(|| c.clone())?;
                    writer
                        .write_all(&(entry_magic ^ base_magic).to_le_bytes())
                        .await.wrap_err_with(|| format!("While writing the magic value of file #{i} (path = {path:?}, file length = {size}) to the archive")).wrap_err_with(|| c.clone())?;
                    writer
                        .write_all(&(path.as_str().bytes().len() as u32 ^ base_magic).to_le_bytes())
                        .await.wrap_err_with(|| format!("While writing the path length of file #{i} (path = {path:?}, file length = {size}) to the archive")).wrap_err_with(|| c.clone())?;
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
                        .await.wrap_err_with(|| format!("While writing the path of file #{i} (path = {path:?}, file length = {size}) to the archive")).wrap_err_with(|| c.clone())?;

                    // Write the actual file contents to a temporary file
                    async_std::io::copy(
                        &mut read_file_xor_async(reader, entry_magic),
                        &mut tmp_writer,
                    )
                    .await.wrap_err_with(|| format!("While writing the contents of file #{i} (path = {path:?}, file length = {size}) to a temporary file before writing it to the archive")).wrap_err_with(|| c.clone())?;

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
                writer
                    .write_all(&base_magic.to_le_bytes())
                    .await
                    .wrap_err("While writing the header terminator to the archive")
                    .wrap_err_with(|| c.clone())?;

                // Write the contents of the temporary file to the buffer after the terminator
                tmp_writer
                    .flush()
                    .await
                    .wrap_err("While flushing a temporary file containing the archive body")
                    .wrap_err_with(|| c.clone())?;
                drop(tmp_writer);
                AsyncSeekExt::seek(&mut tmp, SeekFrom::Start(0)).await.wrap_err("While copying a temporary file containing the archive body into the archive").wrap_err_with(|| c.clone())?;
                async_std::io::copy(&mut tmp, &mut writer).await.wrap_err("While copying a temporary file containin the archive body into the archive").wrap_err_with(|| c.clone())?;

                // Write the offsets into the header now that we know the total size of the files
                let header_size = header_offset + 4;
                for (i, (path, mut entry)) in entries.into_iter().enumerate() {
                    entry.body_offset += header_size;
                    writer.seek(SeekFrom::Start(entry.header_offset)).await.wrap_err_with(|| format!("While writing the file offset of file #{i} (path = {path:?}, file length = {}, file offset = {})", entry.size, entry.body_offset)).wrap_err_with(|| c.clone())?;
                    writer
                        .write_all(&(entry.body_offset as u32 ^ base_magic).to_le_bytes())
                        .await.wrap_err_with(|| format!("While writing the file offset of file #{i} (path = {path:?}, file length = {}, file offset = {}) to the archive", entry.size, entry.body_offset)).wrap_err_with(|| c.clone())?;
                    trie.create_file(path, entry);
                }

                writer
                    .flush()
                    .await
                    .wrap_err("While flushing the archive after writing its contents")
                    .wrap_err_with(|| c.clone())?;
                drop(writer);
                Ok(Self {
                    trie: std::sync::Arc::new(parking_lot::RwLock::new(trie)),
                    archive: std::sync::Arc::new(parking_lot::Mutex::new(buffer)),
                    version,
                    base_magic,
                })
            }

            _ => Err(Error::NotSupported.into()),
        }
    }
}
