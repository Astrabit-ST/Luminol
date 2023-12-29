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

use pin_project::pin_project;
use std::io::{
    prelude::*,
    BufReader,
    ErrorKind::{InvalidData, PermissionDenied},
    SeekFrom,
};
use std::{pin::Pin, task::Poll};

use super::util::{move_file_and_truncate, read_file_xor, regress_magic};
use super::Trie;
use crate::Metadata;
use crate::{File as _, StdIoErrorContext};

#[derive(Debug)]
#[pin_project]
pub struct File<T>
where
    T: crate::File,
{
    pub(super) archive: Option<std::sync::Arc<parking_lot::Mutex<T>>>,
    pub(super) trie: Option<std::sync::Arc<parking_lot::RwLock<Trie>>>,
    pub(super) path: camino::Utf8PathBuf,
    pub(super) read_allowed: bool,
    pub(super) modified: parking_lot::Mutex<bool>,
    pub(super) version: u8,
    pub(super) base_magic: u32,
    #[pin]
    pub(super) tmp: crate::host::File,
}

impl<T> std::io::Write for File<T>
where
    T: crate::File,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let c = format!(
            "While writing to file {:?} within a version {} archive",
            self.path, self.version
        );
        if self.archive.is_some() {
            let mut modified = self.modified.lock();
            *modified = true;
            self.tmp.write(buf).with_io_context(|| c.clone())
        } else {
            Err(std::io::Error::new(
                PermissionDenied,
                "Attempted to write to file with no write permissions",
            ))
            .with_io_context(|| c.clone())
        }
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        let c = format!(
            "While writing (vectored) to file {:?} within a version {} archive",
            self.path, self.version
        );
        if self.archive.is_some() {
            let mut modified = self.modified.lock();
            *modified = true;
            self.tmp.write_vectored(bufs).with_io_context(|| c.clone())
        } else {
            Err(std::io::Error::new(
                PermissionDenied,
                "Attempted to write to file with no write permissions",
            ))
            .with_io_context(|| c.clone())
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut modified = self.modified.lock();
        if !*modified {
            return Ok(());
        }
        let c = format!(
            "While flushing file {:?} within a version {} archive",
            self.path, self.version
        );

        let mut archive = self
            .archive
            .as_ref()
            .ok_or(std::io::Error::new(
                PermissionDenied,
                "Attempted to write to file with no write permissions",
            ))
            .with_io_context(|| c.clone())?
            .lock();
        let mut trie = self
            .trie
            .as_ref()
            .ok_or(std::io::Error::new(
                PermissionDenied,
                "Attempted to write to file with no write permissions",
            ))
            .with_io_context(|| c.clone())?
            .write();
        let archive_len = archive.metadata()?.size;

        let tmp_stream_position = self.tmp.stream_position().with_io_context(|| c.clone())?;
        self.tmp.flush().with_io_context(|| c.clone())?;
        self.tmp
            .seek(SeekFrom::Start(0))
            .with_io_context(|| c.clone())?;

        // If the size of the file has changed, rotate the archive to place the file at the end of
        // the archive before writing the new contents of the file
        let mut entry = *trie
            .get_file(&self.path)
            .ok_or(std::io::Error::new(
                InvalidData,
                "Could not find the file within the archive",
            ))
            .with_io_context(|| c.clone())?;
        let old_size = entry.size;
        let new_size = self.tmp.metadata().with_io_context(|| c.clone())?.size;
        if old_size != new_size {
            move_file_and_truncate(
                &mut archive,
                &mut trie,
                &self.path,
                self.version,
                self.base_magic,
            )
            .io_context("While relocating the file header to the end of the archive")
            .with_io_context(|| c.clone())?;
            entry = *trie
                .get_file(&self.path)
                .ok_or(std::io::Error::new(
                    InvalidData,
                    "Could not find the file within the archive",
                ))
                .with_io_context(|| c.clone())?;

            // Write the new length of the file to the archive
            match self.version {
                1 | 2 => {
                    let mut magic = entry.start_magic;
                    regress_magic(&mut magic);
                    archive
                        .seek(SeekFrom::Start(
                            entry.body_offset.checked_sub(4).ok_or(InvalidData)?,
                        ))
                        .io_context("While writing the file length to the archive")
                        .with_io_context(|| c.clone())?;
                    archive
                        .write_all(&(new_size as u32 ^ magic).to_le_bytes())
                        .io_context("While writing the base magic value of the file to the archive")
                        .with_io_context(|| c.clone())?;
                }

                3 => {
                    archive
                        .seek(SeekFrom::Start(entry.header_offset + 4))
                        .io_context("While writing the file length to the archive")
                        .with_io_context(|| c.clone())?;
                    archive
                        .write_all(&(new_size as u32 ^ self.base_magic).to_le_bytes())
                        .io_context("While writing the base magic value of the file to the archive")
                        .with_io_context(|| c.clone())?;
                }

                _ => {
                    return Err(std::io::Error::new(
                        InvalidData,
                        format!(
                            "Invalid archive version: {} (supported versions are 1, 2 and 3)",
                            self.version
                        ),
                    ))
                }
            }

            // Write the new length of the file to the trie
            trie.get_file_mut(&self.path)
                .ok_or(std::io::Error::new(
                    InvalidData,
                    "Could not find the file within the archive",
                ))
                .io_context("After changing the file length within the archive")
                .with_io_context(|| c.clone())?
                .size = new_size;
        }

        // Now write the new contents of the file
        archive
            .seek(SeekFrom::Start(entry.body_offset))
            .io_context("While writing the file contents to the archive")
            .with_io_context(|| c.clone())?;
        let mut reader = BufReader::new(&mut self.tmp);
        std::io::copy(
            &mut read_file_xor(&mut reader, entry.start_magic),
            archive.as_file(),
        )
        .io_context("While writing the file contents to the archive")
        .with_io_context(|| c.clone())?;
        drop(reader);
        self.tmp
            .seek(SeekFrom::Start(tmp_stream_position))
            .io_context("While writing the file contents to the archive")
            .with_io_context(|| c.clone())?;

        if old_size > new_size {
            archive
                .set_len(
                    archive_len
                        .checked_sub(old_size)
                        .ok_or(std::io::Error::new(
                            InvalidData,
                            "Archive header is corrupt",
                        ))
                        .io_context("While truncating the archive")
                        .with_io_context(|| c.clone())?
                        .checked_add(new_size)
                        .ok_or(std::io::Error::new(
                            InvalidData,
                            "Archive header is corrupt",
                        ))
                        .io_context("While truncating the archive")
                        .with_io_context(|| c.clone())?,
                )
                .io_context("While truncating the archive")
                .with_io_context(|| c.clone())?;
        }
        archive
            .flush()
            .io_context("While flushing the archive after writing its contents")
            .with_io_context(|| c.clone())?;
        *modified = false;
        Ok(())
    }
}

impl<T> std::io::Read for File<T>
where
    T: crate::File,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let c = format!(
            "While reading from file {:?} within a version {} archive",
            self.path, self.version
        );
        if self.read_allowed {
            self.tmp.read(buf).with_io_context(|| c.clone())
        } else {
            Err(std::io::Error::new(
                PermissionDenied,
                "Attempted to read from file with no read permissions",
            ))
            .with_io_context(|| c.clone())
        }
    }

    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        let c = format!(
            "While reading (vectored) from file {:?} within a version {} archive",
            self.path, self.version
        );
        if self.read_allowed {
            self.tmp.read_vectored(bufs).with_io_context(|| c.clone())
        } else {
            Err(std::io::Error::new(
                PermissionDenied,
                "Attempted to read from file with no read permissions",
            ))
            .with_io_context(|| c.clone())
        }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        let c = format!(
            "While reading (exact) from file {:?} within a version {} archive",
            self.path, self.version
        );
        if self.read_allowed {
            self.tmp.read_exact(buf).with_io_context(|| c.clone())
        } else {
            Err(std::io::Error::new(
                PermissionDenied,
                "Attempted to read from file with no read permissions",
            ))
            .with_io_context(|| c.clone())
        }
    }
}

impl<T> futures_lite::AsyncRead for File<T>
where
    T: crate::File + futures_lite::AsyncRead,
{
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let c = format!(
            "While asynchronously reading from file {:?} within a version {} archive",
            self.path, self.version
        );
        if self.read_allowed {
            self.project()
                .tmp
                .poll_read(cx, buf)
                .map(|r| r.with_io_context(|| c.clone()))
        } else {
            Poll::Ready(
                Err(std::io::Error::new(
                    PermissionDenied,
                    "Attempted to read from file with no read permissions",
                ))
                .with_io_context(|| c.clone()),
            )
        }
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        bufs: &mut [std::io::IoSliceMut<'_>],
    ) -> Poll<std::io::Result<usize>> {
        let c = format!(
            "While asynchronously reading (vectored) from file {:?} within a version {} archive",
            self.path, self.version
        );
        if self.read_allowed {
            self.project()
                .tmp
                .poll_read_vectored(cx, bufs)
                .map(|r| r.with_io_context(|| c.clone()))
        } else {
            Poll::Ready(
                Err(std::io::Error::new(
                    PermissionDenied,
                    "Attempted to read from file with no read permissions",
                ))
                .with_io_context(|| c.clone()),
            )
        }
    }
}

impl<T> std::io::Seek for File<T>
where
    T: crate::File,
{
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let c = format!(
            "While asynchronously seeking file {:?} within a version {} archive",
            self.path, self.version
        );
        self.tmp.seek(pos).io_context(c)
    }

    fn stream_position(&mut self) -> std::io::Result<u64> {
        let c = format!(
            "While getting stream position for file {:?} within a version {} archive",
            self.path, self.version
        );
        self.tmp.stream_position().io_context(c)
    }
}

impl<T> futures_lite::AsyncSeek for File<T>
where
    T: crate::File + futures_lite::AsyncSeek,
{
    fn poll_seek(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        pos: SeekFrom,
    ) -> Poll<std::io::Result<u64>> {
        let c = format!(
            "While asynchronously seeking file {:?} within a version {} archive",
            self.path, self.version
        );
        self.project()
            .tmp
            .poll_seek(cx, pos)
            .map(|r| r.io_context(c))
    }
}

impl<T> crate::File for File<T>
where
    T: crate::File,
{
    fn metadata(&self) -> std::io::Result<Metadata> {
        let c = format!(
            "While getting metadata for file {:?} within a version {} archive",
            self.path, self.version
        );
        self.tmp.metadata().io_context(c)
    }

    fn set_len(&self, new_size: u64) -> std::io::Result<()> {
        let c = format!(
            "While setting length for file {:?} within a version {} archive",
            self.path, self.version
        );
        if self.archive.is_some() {
            let mut modified = self.modified.lock();
            *modified = true;
            self.tmp.set_len(new_size).with_io_context(|| c.clone())
        } else {
            Err(std::io::Error::new(
                PermissionDenied,
                "Attempted to write to file with no write permissions",
            ))
            .with_io_context(|| c.clone())
        }
    }
}
