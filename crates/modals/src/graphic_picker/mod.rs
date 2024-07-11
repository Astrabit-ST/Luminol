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

use luminol_components::UiExt;
use luminol_core::prelude::*;

pub mod actor;
pub mod basic;
pub mod event;
pub mod hue;

#[derive(Default)]
enum Selected {
    #[default]
    None,
    Entry {
        path: camino::Utf8PathBuf,
        sprite: PreviewSprite,
    },
}

struct ButtonSprite {
    sprite: Sprite,
    sprite_size: egui::Vec2,
    viewport: Viewport,
}

struct PreviewSprite {
    sprite: Sprite,
    sprite_size: egui::Vec2,
    viewport: Viewport,
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone)]
struct Entry {
    path: camino::Utf8PathBuf,
    invalid: bool,
}

impl ButtonSprite {
    pub fn ui(
        this: Option<&mut Self>,
        ui: &mut egui::Ui,
        update_state: &UpdateState<'_>,
        is_open: bool,
        desired_size: egui::Vec2,
    ) -> egui::Response {
        let (rect, response) = ui.allocate_at_least(desired_size, egui::Sense::click());

        let visuals = ui.style().interact_selectable(&response, is_open);
        let rect = rect.expand(visuals.expansion);
        ui.painter()
            .rect(rect, visuals.rounding, visuals.bg_fill, visuals.bg_stroke);

        if let Some(this) = this {
            let viewport_size = rect.size();
            let translation = (viewport_size - this.sprite_size) / 2.;
            this.viewport.set(
                &update_state.graphics.render_state,
                glam::vec2(viewport_size.x, viewport_size.y),
                glam::vec2(translation.x, translation.y),
                glam::Vec2::ONE,
            );
            let callback = luminol_egui_wgpu::Callback::new_paint_callback(
                rect,
                Painter::new(this.sprite.prepare(&update_state.graphics)),
            );
            ui.painter().add(callback);
        }

        response
    }
}

impl Entry {
    fn load(
        // FIXME error handling
        update_state: &UpdateState<'_>,
        directory: &camino::Utf8Path,
    ) -> Vec<Self> {
        let mut entries: Vec<_> = update_state
            .filesystem
            .read_dir(directory)
            .unwrap()
            .into_iter()
            .map(|m| {
                let path = m
                    .path
                    .strip_prefix(directory)
                    .unwrap_or(&m.path)
                    .with_extension("");
                Entry {
                    path,
                    invalid: false,
                }
            })
            .collect();
        entries.sort_unstable();
        entries
    }

    fn filter(entries: &[Self], filter: &str) -> Vec<Entry> {
        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
        entries
            .iter()
            .filter(|entry| matcher.fuzzy(entry.path.as_str(), filter, false).is_some())
            .cloned()
            .collect()
    }

    fn ui(
        entries: &mut [Self],
        ui: &mut egui::Ui,
        rows: std::ops::Range<usize>,
        selected: &mut Selected,
        load_preview_sprite: impl Fn(&camino::Utf8Path) -> PreviewSprite,
    ) {
        for i in entries[rows.clone()].iter_mut().enumerate() {
            let (i, Self { path, invalid }) = i;
            let checked = matches!(selected, Selected::Entry { path: p, .. } if p == path);
            let mut text = egui::RichText::new(path.as_str());
            if *invalid {
                text = text.color(egui::Color32::LIGHT_RED);
            }
            let faint = (i + rows.start) % 2 == 1;
            ui.with_stripe(faint, |ui| {
                let res = ui.add_enabled(!*invalid, egui::SelectableLabel::new(checked, text));

                if res.clicked() {
                    *selected = Selected::Entry {
                        path: path.clone(),
                        sprite: load_preview_sprite(path),
                    };
                }
            });
        }
    }
}

impl PreviewSprite {
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        viewport: egui::Rect,
        update_state: &UpdateState<'_>,
    ) -> egui::Response {
        let (canvas_rect, response) =
            ui.allocate_exact_size(self.sprite_size, egui::Sense::click());

        let absolute_scroll_rect = ui
            .ctx()
            .screen_rect()
            .intersect(viewport.translate(canvas_rect.min.to_vec2()));
        let scroll_rect = absolute_scroll_rect.translate(-canvas_rect.min.to_vec2());
        self.sprite.transform.set_position(
            &update_state.graphics.render_state,
            glam::vec2(-scroll_rect.left(), -scroll_rect.top()),
        );

        self.viewport.set(
            &update_state.graphics.render_state,
            glam::vec2(absolute_scroll_rect.width(), absolute_scroll_rect.height()),
            glam::Vec2::ZERO,
            glam::Vec2::ONE,
        );

        let painter = Painter::new(self.sprite.prepare(&update_state.graphics));
        ui.painter()
            .add(luminol_egui_wgpu::Callback::new_paint_callback(
                absolute_scroll_rect,
                painter,
            ));

        response
    }
}
