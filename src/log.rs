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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use std::io::Write;
use std::sync::mpsc::Sender;
use std::sync::Arc;

use once_cell::sync::OnceCell;

#[derive(Clone)]
struct LogWriter {
    sender: Sender<u8>,
    context: Arc<OnceCell<egui::Context>>,
}

impl std::io::Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for &byte in buf {
            if byte == b'\n' {
                let _ = self.sender.send(b'\r');
            }
            let _ = self.sender.send(byte);
        }

        if let Some(ctx) = self.context.get() {
            ctx.request_repaint();
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct CopyWriter<A, B>(A, B);

impl<A, B> Write for CopyWriter<A, B>
where
    A: Write,
    B: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(self.0.write(buf)?.min(self.1.write(buf)?))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()?;
        self.1.flush()?;
        Ok(())
    }
}

pub fn initialize_log(sender: Sender<u8>, context: Arc<OnceCell<egui::Context>>) {
    let log_writer = LogWriter { sender, context };
    tracing_subscriber::fmt()
        // we clone + move the log_writer so this closure impls Fn()
        // the cost of doing this clone is pretty minimal (in comparison to locking stderr) so this is ok
        .with_writer(move || CopyWriter(std::io::stderr(), log_writer.clone()))
        .init();
}
