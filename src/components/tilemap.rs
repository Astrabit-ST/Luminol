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
#![allow(unsafe_code)]

use std::mem::size_of;
use std::sync::Arc;

use crate::image_cache::GlTexture;
use crate::prelude::*;
use glow::HasContext;

pub struct Tilemap {
    /// The tilemap pan.
    pub pan: egui::Vec2,
    /// The scale of the tilemap.
    pub scale: f32,
    /// Toggle to display the visible region in-game.
    pub visible_display: bool,
    /// Toggle move route preview
    pub move_preview: bool,

    textures: Arc<Textures>,

    vbo: glow::NativeBuffer,
    vao: glow::VertexArray,
}

impl Drop for Tilemap {
    fn drop(&mut self) {
        unsafe {
            state!().gl.delete_buffer(self.vbo);
            state!().gl.delete_vertex_array(self.vao);
        }
    }
}

struct Textures {
    atlas: GlTexture,
    event_texs: HashMap<String, Arc<GlTexture>>,
    fog_tex: Option<Arc<GlTexture>>,
    pano_tex: Option<Arc<GlTexture>>,
}

static_assertions::assert_impl_all!(Textures: Send, Sync);

macro_rules! check_for_gl_error {
    () => {{
        if cfg!(debug_assertions) {
            check_for_gl_error_impl(file!(), line!(), "")
        }
    }};
    ($context: literal) => {{
        if cfg!(debug_assertions) {
            check_for_gl_error_impl(file!(), line!(), $context)
        }
    }};
}

static TILEMAP_SHADER: once_cell::sync::Lazy<glow::Program> =
    once_cell::sync::Lazy::new(|| unsafe {
        let gl = &state!().gl;

        let vertex = gl
            .create_shader(glow::VERTEX_SHADER)
            .expect("failed to create shader");
        gl.shader_source(vertex, include_str!("tilemap.vert"));
        gl.compile_shader(vertex);

        if !gl.get_shader_compile_status(vertex) {
            println!("failed to compile tilemap vertex shader");
            println!("{}", gl.get_shader_info_log(vertex));
            std::process::exit(1);
        }

        let fragment = gl
            .create_shader(glow::FRAGMENT_SHADER)
            .expect("failed to create shader");
        gl.shader_source(fragment, include_str!("tilemap.frag"));
        gl.compile_shader(fragment);

        if !gl.get_shader_compile_status(fragment) {
            println!("failed to compile tilemap frag shader");
            println!("{}", gl.get_shader_info_log(fragment));
            std::process::exit(1);
        }

        let program = gl
            .create_program()
            .expect("failed to create shader program");

        gl.attach_shader(program, vertex);
        gl.attach_shader(program, fragment);
        gl.link_program(program);

        if !gl.get_program_completion_status(program) {
            println!("failed to link tilemap shader");
            println!("{}", gl.get_program_info_log(program));
            std::process::exit(1);
        }

        gl.delete_shader(vertex);
        gl.delete_shader(fragment);

        program
    });

impl Tilemap {
    pub fn new(id: i32) -> Result<Tilemap, String> {
        let textures = Arc::new(Self::load_data(id)?);

        let gl = &state!().gl;

        let vao = unsafe {
            let vao = gl.create_vertex_array().expect("failed to create vao");
            gl.bind_vertex_array(Some(vao));
            vao
        };
        check_for_gl_error!("generating vao");

        let vbo = unsafe {
            let vbo = gl.create_buffer().expect("failed to create vbo");
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(&[-0.5_f32, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0]),
                glow::STATIC_DRAW,
            );

            gl.vertex_attrib_pointer_f32(0, 3, glow::FLOAT, false, 3 * size_of::<f32>() as i32, 0);
            gl.enable_vertex_attrib_array(0);

            vbo
        };
        check_for_gl_error!("generating vbo");

        Ok(Self {
            pan: egui::Vec2::ZERO,
            scale: 100.,
            visible_display: false,
            move_preview: false,

            textures,
            vbo,
            vao,
        })
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        map: &rpg::Map,
        cursor_pos: &mut egui::Pos2,
        toggled_layers: &[bool],
        selected_layer: usize,
        dragging_event: bool,
    ) -> egui::Response {
        // Allocate the largest size we can for the tilemap
        let canvas_rect = ui.max_rect();
        let canvas_center = canvas_rect.center();
        ui.set_clip_rect(canvas_rect);

        let vao = self.vao;
        ui.painter().add(egui::PaintCallback {
            rect: canvas_rect,
            callback: Arc::new(eframe::egui_glow::CallbackFn::new(move |_i, painter| {
                let gl = painter.gl();
                unsafe {
                    gl.clear_color(0.2, 0.3, 0.3, 1.0);
                    gl.clear(glow::COLOR_BUFFER_BIT);

                    gl.bind_vertex_array(Some(vao));
                    gl.use_program(Some(*TILEMAP_SHADER));
                    gl.draw_arrays(glow::TRIANGLES, 0, 3);

                    check_for_gl_error!("painting tilemap");
                }
            })),
        });

        let mut response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());

        response
    }

    pub fn tilepicker(&self, ui: &mut egui::Ui, selected_tile: &mut i16) {
        let (canvas_rect, response) =
            ui.allocate_exact_size(self.textures.atlas.size_vec2(), egui::Sense::click());

        let vao = self.vao;
        ui.painter().add(egui::PaintCallback {
            rect: canvas_rect,
            callback: Arc::new(eframe::egui_glow::CallbackFn::new(move |_i, painter| {
                let gl = painter.gl();
            })),
        });
    }

    #[allow(unused_variables, unused_assignments)]
    fn load_data(id: i32) -> Result<Textures, String> {
        let state = state!();
        // Load the map.

        let map = state.data_cache.load_map(id)?;
        // Get tilesets.
        let tilesets = state.data_cache.tilesets();

        // We subtract 1 because RMXP is stupid and pads arrays with nil to start at 1.
        let tileset = &tilesets[map.tileset_id as usize - 1];

        let atlas = Self::build_atlas(tileset)?;

        let event_texs = map
            .events
            .iter()
            .filter_map(|(_, e)| e.pages.first().map(|p| p.graphic.character_name.clone()))
            .filter(|s| !s.is_empty())
            .dedup()
            .map(|char_name| unsafe {
                state
                    .image_cache
                    .load_glow_image("Graphics/Characters", &char_name)
                    .map(|texture| (char_name, texture))
            })
            .try_collect()?;

        // These two are pretty simple.
        let fog_tex = unsafe {
            state
                .image_cache
                .load_glow_image("Graphics/Fogs", &tileset.fog_name)
        }
        .ok();

        let pano_tex = unsafe {
            state
                .image_cache
                .load_glow_image("Graphics/Panoramas", &tileset.panorama_name)
        }
        .ok();

        // Finally create and return the struct.
        Ok(Textures {
            atlas,
            event_texs,
            fog_tex,
            pano_tex,
        })
    }

    fn calc_atlas_dimensions(
        tileset: &rpg::Tileset,
        tileset_height: i32,
    ) -> Result<(i32, i32), String> {
        // Limit the size of the tileset to 8192
        const maxAtlasSize: i32 = 8192;
        const atBuffer: i32 = 32;
        /* Autotile area width */
        const atAreaW: i32 = 32 * 3 * 8;
        /* Autotile area height */
        const atAreaH: i32 = 32 * 4 * 7 + atBuffer;
        const tilesetW: i32 = 32 * 8;
        const tsLaneW: i32 = tilesetW / 1;
        const underAtLanes: i32 = atAreaW / tsLaneW + !!(atAreaW % tsLaneW);

        /* Autotile area */
        const atArea: i32 = underAtLanes * tsLaneW * atAreaH;

        let mut width = underAtLanes * tsLaneW;
        let mut height = atAreaH;
        let tsArea = tilesetW * tileset_height;

        fn freeArea(width: i32, height: i32) -> i32 {
            width * height - atArea
        }

        /* Expand vertically */
        while freeArea(width, height) < tsArea && height < maxAtlasSize {
            height += 32;
        }

        if freeArea(width, height) >= tsArea && height <= maxAtlasSize {
            return Ok((width, height));
        }

        /* Expand horizontally */
        while freeArea(width, height) < tsArea && width < maxAtlasSize {
            width += tsLaneW;
        }

        if (freeArea(width, height) >= tsArea && width <= maxAtlasSize) {
            return Ok((width, height));
        }

        Err("cannot fit tileset into an 8192x8192 texture".to_string())
    }

    fn build_atlas(tileset: &rpg::Tileset) -> Result<GlTexture, String> {
        let gl = &state!().gl;
        unsafe {
            let tileset_img = state!()
                .image_cache
                .load_image("Graphics/Tilesets", &tileset.tileset_name)?;

            let (width, height) =
                Self::calc_atlas_dimensions(tileset, tileset_img.height() as i32)?;

            let texture = gl.create_texture()?;
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width,
                height,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                None,
            );

            Ok(GlTexture::new(texture, width as u32, height as u32))
        }
    }
}

pub fn check_for_gl_error_impl(file: &str, line: u32, context: &str) {
    let gl = &state!().gl;
    #[allow(unsafe_code)]
    let error_code = unsafe { gl.get_error() };
    if error_code != glow::NO_ERROR {
        let error_str = match error_code {
            glow::INVALID_ENUM => "GL_INVALID_ENUM",
            glow::INVALID_VALUE => "GL_INVALID_VALUE",
            glow::INVALID_OPERATION => "GL_INVALID_OPERATION",
            glow::STACK_OVERFLOW => "GL_STACK_OVERFLOW",
            glow::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW",
            glow::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
            glow::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
            glow::CONTEXT_LOST => "GL_CONTEXT_LOST",
            0x8031 => "GL_TABLE_TOO_LARGE1",
            0x9242 => "CONTEXT_LOST_WEBGL",
            _ => "<unknown>",
        };

        if context.is_empty() {
            tracing::error!(
                "GL error, at {}:{}: {} (0x{:X}). Please file a bug at https://github.com/Astrabit-ST/Luminol/issues",
                file,
                line,
                error_str,
                error_code,
            );
        } else {
            tracing::error!(
                "GL error, at {}:{} ({}): {} (0x{:X}). Please file a bug at https://github.com/Astrabit-ST/Luminol/issues",
                file,
                line,
                context,
                error_str,
                error_code,
            );
        }
    }
}
