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
use crate::File as _;
use crate::Metadata;

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
                    archive.seek(SeekFrom::Start(entry.header_offset + 4))?;
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
            archive.as_file(),
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

impl<T> futures_lite::AsyncRead for File<T>
where
    T: crate::File + futures_lite::AsyncRead,
{
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        if self.read_allowed {
            self.project().tmp.poll_read(cx, buf)
        } else {
            Poll::Ready(Err(PermissionDenied.into()))
        }
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        bufs: &mut [std::io::IoSliceMut<'_>],
    ) -> Poll<std::io::Result<usize>> {
        if self.read_allowed {
            self.project().tmp.poll_read_vectored(cx, bufs)
        } else {
            Poll::Ready(Err(PermissionDenied.into()))
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

impl<T> futures_lite::AsyncSeek for File<T>
where
    T: crate::File + futures_lite::AsyncSeek,
{
    fn poll_seek(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        pos: SeekFrom,
    ) -> Poll<std::io::Result<u64>> {
        self.project().tmp.poll_seek(cx, pos)
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
