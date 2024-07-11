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

use egui::InnerResponse;

/// An extension trait that contains some helper methods for `egui::Ui`.
pub trait UiExt {
    /// Determines the width of text in points when displayed with a specific font.
    fn text_width(
        &self,
        text: impl Into<egui::WidgetText>,
        font: impl Into<egui::FontSelection>,
    ) -> f32;

    /// Displays widgets with cross justify, i.e. widgets will expand horizontally to take up all
    /// available space in vertical layouts and widgets will expand vertically to take up all
    /// available space in horizontal layouts.
    fn with_cross_justify<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> InnerResponse<R>;

    /// This is the same as `.with_cross_justify` except it also centers widgets.
    fn with_cross_justify_center<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> InnerResponse<R>;

    /// Displays contents inside a container with spacing on the left side.
    fn with_left_margin<R>(&mut self, m: f32, f: impl FnOnce(&mut Self) -> R) -> InnerResponse<R>;

    /// Displays contents inside a container with spacing on the right side.
    fn with_right_margin<R>(&mut self, m: f32, f: impl FnOnce(&mut Self) -> R) -> InnerResponse<R>;

    /// Displays contents with a normal or faint background (useful for tables with striped rows).
    fn with_stripe<R>(&mut self, faint: bool, f: impl FnOnce(&mut Self) -> R) -> InnerResponse<R>;

    /// Displays contents with a normal or faint background as well as a little bit of horizontal
    /// padding.
    fn with_padded_stripe<R>(
        &mut self,
        faint: bool,
        f: impl FnOnce(&mut Self) -> R,
    ) -> InnerResponse<R>;

    /// Modifies the given `egui::WidgetText` to truncate when the text is too long to fit in the
    /// current layout, rather than wrapping the text or expanding the layout.
    fn truncate_text(&self, text: impl Into<egui::WidgetText>) -> egui::WidgetText;
}

impl UiExt for egui::Ui {
    fn text_width(
        &self,
        text: impl Into<egui::WidgetText>,
        font: impl Into<egui::FontSelection>,
    ) -> f32 {
        Into::<egui::WidgetText>::into(text)
            .into_galley(self, None, f32::INFINITY, font)
            .rect
            .width()
    }

    fn with_cross_justify<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> egui::InnerResponse<R> {
        self.with_layout(
            egui::Layout {
                cross_justify: true,
                ..*self.layout()
            },
            f,
        )
    }

    fn with_cross_justify_center<R>(
        &mut self,
        f: impl FnOnce(&mut Self) -> R,
    ) -> egui::InnerResponse<R> {
        self.with_layout(
            egui::Layout {
                cross_justify: true,
                cross_align: egui::Align::Center,
                ..*self.layout()
            },
            f,
        )
    }

    fn with_left_margin<R>(&mut self, m: f32, f: impl FnOnce(&mut Self) -> R) -> InnerResponse<R> {
        egui::Frame::none()
            .outer_margin(egui::Margin {
                left: m,
                ..egui::Margin::ZERO
            })
            .show(self, f)
    }

    fn with_right_margin<R>(&mut self, m: f32, f: impl FnOnce(&mut Self) -> R) -> InnerResponse<R> {
        egui::Frame::none()
            .outer_margin(egui::Margin {
                right: m,
                ..egui::Margin::ZERO
            })
            .show(self, f)
    }

    fn with_stripe<R>(&mut self, faint: bool, f: impl FnOnce(&mut Self) -> R) -> InnerResponse<R> {
        let frame = egui::containers::Frame::none();
        if faint {
            frame.fill(self.visuals().faint_bg_color)
        } else {
            frame
        }
        .show(self, f)
    }

    fn with_padded_stripe<R>(
        &mut self,
        faint: bool,
        f: impl FnOnce(&mut Self) -> R,
    ) -> InnerResponse<R> {
        let frame = egui::containers::Frame::none()
            .inner_margin(egui::Margin::symmetric(self.spacing().item_spacing.x, 0.));
        if faint {
            frame.fill(self.visuals().faint_bg_color)
        } else {
            frame
        }
        .show(self, f)
    }

    fn truncate_text(&self, text: impl Into<egui::WidgetText>) -> egui::WidgetText {
        let mut job = Into::<egui::WidgetText>::into(text).into_layout_job(
            self.style(),
            egui::TextStyle::Body.into(),
            self.layout().vertical_align(),
        );
        job.wrap.max_width = self.available_width();
        job.wrap.max_rows = 1;
        job.wrap.break_anywhere = true;
        job.into()
    }
}
