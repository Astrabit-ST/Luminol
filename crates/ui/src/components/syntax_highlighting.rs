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

#![allow(missing_docs)]

use egui::text::LayoutJob;

impl egui::util::cache::ComputerMut<(luminol_config::CodeTheme, &str, &str), LayoutJob>
    for Highlighter
{
    fn compute(
        &mut self,
        (theme, code, lang): (luminol_config::CodeTheme, &str, &str),
    ) -> LayoutJob {
        self.highlight(theme, code, lang)
    }
}

/// View some code with syntax highlighting and selection.
pub fn code_view_ui(ui: &mut egui::Ui, mut code: &str, theme: luminol_config::CodeTheme) {
    let language = "rb";

    let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
        let layout_job = highlight(ui.ctx(), theme, string, language);
        // layout_job.wrap.max_width = wrap_width; // no wrapping
        ui.fonts(|f| f.layout_job(layout_job))
    };

    ui.add(
        egui::TextEdit::multiline(&mut code)
            .font(egui::TextStyle::Monospace) // for cursor height
            .code_editor()
            .desired_rows(1)
            .lock_focus(true)
            .layouter(&mut layouter),
    );
}

/// Memoized Code highlighting
#[must_use]
pub fn highlight(
    ctx: &egui::Context,
    theme: luminol_config::CodeTheme,
    code: &str,
    language: &str,
) -> LayoutJob {
    type HighlightCache = egui::util::cache::FrameCache<LayoutJob, Highlighter>;

    ctx.memory_mut(|m| {
        let highlight_cache = m.caches.cache::<HighlightCache>();
        highlight_cache.get((theme, code, language))
    })
}

struct Highlighter {
    ps: syntect::parsing::SyntaxSet,
    ts: syntect::highlighting::ThemeSet,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self {
            ps: syntect::parsing::SyntaxSet::load_defaults_newlines(),
            ts: syntect::highlighting::ThemeSet::load_defaults(),
        }
    }
}

impl Highlighter {
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn highlight(&self, theme: luminol_config::CodeTheme, code: &str, lang: &str) -> LayoutJob {
        self.highlight_impl(theme, code, lang).unwrap_or_else(|| {
            // Fallback:
            LayoutJob::simple(
                code.into(),
                egui::FontId::monospace(12.0),
                if theme.dark_mode {
                    egui::Color32::LIGHT_GRAY
                } else {
                    egui::Color32::DARK_GRAY
                },
                f32::INFINITY,
            )
        })
    }

    fn highlight_impl(
        &self,
        theme: luminol_config::CodeTheme,
        text: &str,
        language: &str,
    ) -> Option<LayoutJob> {
        use egui::text::{LayoutSection, TextFormat};
        use syntect::easy::HighlightLines;
        use syntect::highlighting::FontStyle;
        use syntect::util::LinesWithEndings;

        let syntax = self
            .ps
            .find_syntax_by_name(language)
            .or_else(|| self.ps.find_syntax_by_extension(language))?;

        let theme = theme.syntect_theme.syntect_key_name();
        let mut h = HighlightLines::new(syntax, &self.ts.themes[theme]);

        let mut job = LayoutJob {
            text: text.into(),
            ..Default::default()
        };

        for line in LinesWithEndings::from(text) {
            for (style, range) in h.highlight_line(line, &self.ps).ok()? {
                let fg = style.foreground;
                let text_color = egui::Color32::from_rgb(fg.r, fg.g, fg.b);
                let italics = style.font_style.contains(FontStyle::ITALIC);
                let underline = style.font_style.contains(FontStyle::ITALIC);
                let underline = if underline {
                    egui::Stroke::new(1.0, text_color)
                } else {
                    egui::Stroke::NONE
                };
                job.sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: as_byte_range(text, range),
                    format: TextFormat {
                        font_id: egui::FontId::monospace(12.0),
                        color: text_color,
                        italics,
                        underline,
                        ..Default::default()
                    },
                });
            }
        }

        Some(job)
    }
}

fn as_byte_range(whole: &str, range: &str) -> std::ops::Range<usize> {
    let whole_start = whole.as_ptr() as usize;
    let range_start = range.as_ptr() as usize;
    assert!(whole_start <= range_start);
    assert!(range_start + range.len() <= whole_start + whole.len());
    let offset = range_start - whole_start;
    offset..(offset + range.len())
}
