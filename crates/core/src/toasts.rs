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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use itertools::Itertools;

/// A toasts management struct.
#[derive(Default)]
pub struct Toasts {
    inner: egui_notify::Toasts,
}

// We wrap the toasts structs in a RefCell to maintain interior mutability.
#[allow(dead_code)]
impl Toasts {
    /// Add a custom toast.
    pub fn add(&mut self, toast: egui_notify::Toast) {
        self.inner.add(toast);
    }

    /// Display all toasts.
    pub fn show(&mut self, ctx: &egui::Context) {
        self.inner.show(ctx);
    }

    #[doc(hidden)]
    pub fn _i_inner(&mut self, caption: impl Into<String>) {
        self.inner
            .info(caption)
            .set_duration(Some(std::time::Duration::from_secs(7)));
    }

    #[doc(hidden)]
    pub fn _w_inner(&mut self, caption: impl Into<String>) {
        self.inner
            .warning(caption)
            .set_duration(Some(std::time::Duration::from_secs(7)));
    }

    #[doc(hidden)]
    pub fn _b_inner(&mut self, caption: impl Into<String>) {
        self.inner
            .basic(caption)
            .set_duration(Some(std::time::Duration::from_secs(7)));
    }

    #[doc(hidden)]
    pub fn _e_inner(&mut self, error: &color_eyre::Report) {
        #[cfg(not(target_arch = "wasm32"))]
        let help = "Check the output window (Debug > Output) for more details";
        #[cfg(target_arch = "wasm32")]
        let help = "Check the browser developer console for more details";

        if error.chain().len() <= 1 {
            self.inner.error(format!("{}\n\n{}", error, help,))
        } else {
            self.inner.error(format!(
                "{}\n\n{}\n\n{}",
                error,
                error.chain().skip(1).map(|e| e.to_string()).join("\n"),
                help
            ))
        }
        .set_duration(Some(std::time::Duration::from_secs(7)));
    }
}

/// Display an info toast.
#[macro_export]
macro_rules! info {
    ($toasts:expr, $caption:expr $(,)?) => {{
        let caption = String::from($caption);
        $crate::tracing::info!("{caption}");
        $crate::Toasts::_i_inner($toasts, $caption);
    }};
}

/// Display a warning toast.
#[macro_export]
macro_rules! warn {
    ($toasts:expr, $caption:expr $(,)?) => {{
        let caption = String::from($caption);
        $crate::tracing::warn!("{caption}");
        $crate::Toasts::_w_inner($toasts, caption);
    }};
}

/// Display a generic toast.
#[macro_export]
macro_rules! basic {
    ($toasts:expr, $caption:expr $(,)?) => {{
        let caption = String::from($caption);
        $crate::tracing::info!("{caption}");
        $crate::Toasts::_b_inner($toasts, caption);
    }};
}

/// Format a `color_eyre::Report` and display it as an error toast.
#[macro_export]
macro_rules! error {
    ($toasts:expr, $error:expr $(,)?) => {{
        let error = &$error;
        $crate::tracing::error!("Luminol error:{error:?}");
        $crate::Toasts::_e_inner($toasts, error);
    }};
}
