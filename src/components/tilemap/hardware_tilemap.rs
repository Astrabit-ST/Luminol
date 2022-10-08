// Copyright (C) 2022 Lily Lyons
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

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use eframe::glow::NativeTexture;
use egui::{Pos2, Response, Vec2};

use super::TilemapDef;
use crate::data::rmxp_structs::rpg;
use crate::glow::{self, HasContext};
use crate::UpdateInfo;

#[allow(dead_code)]
pub struct Textures {
    pub tileset_tex: NativeTexture,
    pub autotile_texs: Vec<Option<NativeTexture>>,
    pub event_texs: HashMap<(String, i32), Option<NativeTexture>>,
    pub fog_tex: Option<NativeTexture>,
    pub fog_zoom: i32,
    pub pano_tex: Option<NativeTexture>,
}

pub struct Tilemap {
    pub scale: f32,
    pub visible_display: bool,
    pub pan: Vec2,
    vao: glow::NativeVertexArray,
    load_data: poll_promise::Promise<()>,
}

// We only want to create shaders once. This setup allows us to do that.
unsafe fn with_hardware_shaders(gl: Arc<glow::Context>, mut f: impl FnMut(glow::NativeProgram)) {
    thread_local! {
        static SHADERS: RefCell<Option<glow::NativeProgram>> = RefCell::new(None)
    }
    SHADERS.with(|s| {
        f(*s.borrow_mut().get_or_insert_with(|| {
            let vert_source = r#"
            #version 330 core
            layout (location = 0) in vec3 aPos;
            layout (location = 1) in vec3 aColor;

            out vec3 ourColor; // output a color to the fragment shader
                    
            void main()
            {
                gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
                ourColor = aColor;
            }
            "#;

            let vert = gl.create_shader(glow::VERTEX_SHADER).unwrap();
            gl.shader_source(vert, vert_source);
            gl.compile_shader(vert);

            if !gl.get_shader_compile_status(vert) {
                println!("SHDER LOG: {}", gl.get_shader_info_log(vert));
            }

            let frag_source = r#"
            #version 330 core
            out vec4 FragColor;

            in vec3 ourColor;
                    
            void main()
            {
                FragColor = vec4(ourColor, 1.0f);
            } 
            "#;

            let frag = gl.create_shader(glow::FRAGMENT_SHADER).unwrap();
            gl.shader_source(frag, frag_source);
            gl.compile_shader(frag);

            if !gl.get_shader_compile_status(frag) {
                println!("SHDER LOG: {}", gl.get_shader_info_log(frag));
            }

            let program = gl.create_program().unwrap();
            gl.attach_shader(program, vert);
            gl.attach_shader(program, frag);
            gl.link_program(program);

            gl.delete_shader(vert);
            gl.delete_shader(frag);

            program
        }));
    })
}

#[allow(dead_code)]
impl TilemapDef for Tilemap {
    fn new(info: &'static UpdateInfo, id: i32) -> Self {
        let vao = unsafe {
            let gl = info.gl.clone();

            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));

            let vertices: [f32; 18] = [
                // positions         // colors
                0.5, -0.5, 0.0, 1.0, 0.0, 0.0, // bottom right
                -0.5, -0.5, 0.0, 0.0, 1.0, 0.0, // bottom left
                0.0, 0.5, 0.0, 0.0, 0.0, 1.0, // top
            ];

            let vert_buffer = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vert_buffer));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                std::slice::from_raw_parts(
                    vertices.as_ptr() as _,
                    vertices.len() * std::mem::size_of::<f32>(),
                ), // Holy *shit* this is BAD. we are converting a slice of f32s to bytes by reading their memory AS bytes.
                // At least rust makes this behavior obvious rather than C's void*...
                glow::STATIC_DRAW,
            );

            gl.vertex_attrib_pointer_f32(
                0,
                3,
                glow::FLOAT,
                false,
                6 * std::mem::size_of::<f32>() as i32,
                0,
            ); // We have 3 floats per 'index'? so that's why we multiply 3 by the sizeof float.
            gl.enable_vertex_attrib_array(0);

            gl.vertex_attrib_pointer_f32(
                1,
                3,
                glow::FLOAT,
                false,
                6 * std::mem::size_of::<f32>() as i32,
                3 * std::mem::size_of::<f32>() as i32, // We need to offset it by 3 floats since our color is after positions
            ); // We have 3 floats per 'index'? so that's why we multiply 3 by the sizeof float.
            gl.enable_vertex_attrib_array(1);

            vao
        };

        Self {
            scale: 100.,
            visible_display: false,
            pan: Vec2::ZERO,
            vao,
            load_data: poll_promise::Promise::spawn_local(async move {
                Self::load_data(info, id).await.unwrap()
            }),
        }
    }

    #[allow(unused_variables)]
    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        map: &rpg::Map,
        cursor_pos: &mut Pos2,
        toggled_layers: &[bool],
        selected_layer: usize,
    ) -> Response {
        let canvas_rect = ui.max_rect();
        let canvas_center = canvas_rect.center();

        let response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());

        let vao = self.vao;
        ui.painter().add(egui::PaintCallback {
            rect: canvas_rect,
            callback: std::sync::Arc::new(eframe::egui_glow::CallbackFn::new(
                move |_info, painter| unsafe {
                    with_hardware_shaders(painter.gl().clone(), |program| {
                        let gl = painter.gl().clone();

                        gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);

                        gl.use_program(Some(program));

                        gl.bind_vertex_array(Some(vao));
                        gl.draw_arrays(glow::TRIANGLES, 0, 3);

                        gl.polygon_mode(glow::FRONT_AND_BACK, glow::FILL);
                    })
                },
            )),
        });

        response
    }

    fn tilepicker(&self, ui: &mut egui::Ui, selected_tile: &mut i16) {
        let (rect, response) = ui.allocate_exact_size(egui::vec2(256., 256.), egui::Sense::click()); // textures.tileset_tex.size_vec2(), egui::Sense::click());

        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let mut pos = (pos - rect.min) / 32.;
                pos.x = pos.x.floor();
                pos.y = pos.y.floor();
                *selected_tile = (pos.x + pos.y * 8.) as i16;
            }
        }

        let cursor_x = *selected_tile % 8 * 32;
        let cursor_y = *selected_tile / 8 * 32;
        ui.painter().rect_stroke(
            egui::Rect::from_min_size(
                rect.min + egui::vec2(cursor_x as f32, cursor_y as f32),
                egui::Vec2::splat(32.),
            ),
            5.0,
            egui::Stroke::new(1.0, egui::Color32::WHITE),
        );
    }

    fn textures_loaded(&self) -> bool {
        self.load_data.ready().is_some()
    }
}

impl Tilemap {
    async fn load_data(info: &'static UpdateInfo, id: i32) -> Result<(), String> {
        let _map = info.data_cache.load_map(&info.filesystem, id).await?;

        Ok(())
    }
}
