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

use async_std::stream::StreamExt;
use itertools::Itertools;
use std::io::{prelude::*, BufReader, BufWriter, ErrorKind::InvalidData, SeekFrom};

use super::{Trie, HEADER};
use crate::Error;

fn read_u32(mut file: impl Read) -> std::io::Result<u32> {
    let mut buffer = [0; 4];
    file.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

pub(super) fn read_u32_xor(file: impl Read, key: u32) -> std::io::Result<u32> {
    let result = read_u32(file)?;
    Ok(result ^ key)
}

pub(super) fn read_file_xor(file: impl Read, start_magic: u32) -> impl Read {
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

pub(super) fn read_file_xor_async(
    file: impl futures_lite::AsyncRead + Unpin,
    start_magic: u32,
) -> impl futures_lite::AsyncRead + Unpin {
    use futures_lite::AsyncReadExt;

    let stream = file.bytes().scan((start_magic, 0), |state, maybe_byte| {
        let Ok(byte) = maybe_byte else { return None };
        let (mut magic, mut j) = *state;

        if j == 4 {
            j = 0;
            magic = magic.wrapping_mul(7).wrapping_add(3);
        }
        let byte = byte ^ magic.to_le_bytes()[j];
        j += 1;

        *state = (magic, j);
        Some(Ok([byte]))
    });

    async_io_stream::IoStream::new(stream)
}

pub(super) fn advance_magic(magic: &mut u32) -> u32 {
    let old = *magic;

    *magic = magic.wrapping_mul(7).wrapping_add(3);

    old
}

pub(super) fn regress_magic(magic: &mut u32) -> u32 {
    let old = *magic;

    *magic = magic.wrapping_sub(3).wrapping_mul(3067833783);

    old
}

pub(super) fn read_header(mut file: impl Read) -> Result<u8, Error> {
    let mut header_buf = [0; 8];

    file.read_exact(&mut header_buf)?;

    if !header_buf.starts_with(HEADER) {
        return Err(Error::InvalidHeader);
    }

    Ok(header_buf[7])
}

/// Moves a file within an archive to the end of the archive and truncates the file's length to 0.
/// Does NOT truncate the actual archive to the correct length afterwards.
pub(super) fn move_file_and_truncate<T>(
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

    let mut entry = *trie.get_file(path).ok_or(InvalidData)?;
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
                let current_entry = trie.get_file_mut(&current_path).ok_or(InvalidData)?;
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
                        read_file_xor(&mut (&mut reader).take(current_entry.size), reader_magic),
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
            *trie.get_file_mut(path).ok_or(InvalidData)? = entry;

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
            let mut current_header_offset = 12;
            archive.seek(SeekFrom::Start(12))?;
            let mut reader = BufReader::new(archive.as_file());
            let mut headers = Vec::new();
            while let Ok(current_body_offset) = read_u32_xor(&mut reader, base_magic) {
                if current_body_offset == 0 {
                    break;
                }
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

                trie.get_file_mut(current_path)
                    .ok_or(InvalidData)?
                    .body_offset = current_body_offset;
                headers.push((
                    current_header_offset,
                    current_body_offset as u32,
                    should_truncate,
                ));

                current_header_offset += current_path_len as u64 + 16;
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

            trie.get_file_mut(path).ok_or(InvalidData)?.size = 0;

            Ok(())
        }

        _ => Err(InvalidData.into()),
    }
}
