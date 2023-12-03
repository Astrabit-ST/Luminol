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

pub use atlas::Atlas;

use autotiles::Autotiles;
use instance::Instances;
use opacity::Opacity;

mod atlas;
mod autotile_ids;
pub(crate) mod autotiles;
mod instance;
pub(crate) mod opacity;
pub(crate) mod shader;

#[derive(Debug)]
pub struct Tiles {
    pub autotiles: Autotiles,
    pub atlas: Atlas,
    pub instances: Instances,
    pub opacity: Opacity,
    pub use_push_constants: bool,
}

impl Tiles {
    pub fn new(
        graphics_state: &crate::GraphicsState,
        atlas: Atlas,
        tiles: &luminol_data::Table3,
        use_push_constants: bool,
    ) -> Self {
        let autotiles = Autotiles::new(graphics_state, &atlas, use_push_constants);
        let instances = Instances::new(
            &graphics_state.render_state,
            tiles,
            atlas.atlas_texture.size(),
        );
        let opacity = Opacity::new(graphics_state, use_push_constants);

        Self {
            autotiles,
            atlas,
            instances,
            opacity,
            use_push_constants,
        }
    }

    pub fn set_tile(
        &self,
        render_state: &luminol_egui_wgpu::RenderState,
        tile_id: i16,
        position: (usize, usize, usize),
    ) {
        self.instances.set_tile(render_state, tile_id, position)
    }

    pub fn draw<'rpass>(
        &'rpass self,
        graphics_state: &'rpass crate::GraphicsState,
        viewport: &crate::viewport::Viewport,
        enabled_layers: &[bool],
        selected_layer: Option<usize>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        #[repr(C)]
        #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        struct VertexPushConstant {
            viewport: [u8; 64],
            autotiles: [u8; 48],
        }

        render_pass.push_debug_group("tilemap tiles renderer");
        render_pass.set_pipeline(&graphics_state.pipelines.tiles);
        self.autotiles.bind(render_pass);
        if self.use_push_constants {
            render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                0,
                bytemuck::bytes_of(&VertexPushConstant {
                    viewport: viewport.as_bytes(),
                    autotiles: self.autotiles.as_bytes(),
                }),
            );
        }

        self.atlas.bind(render_pass);
        self.opacity.bind(render_pass);

        for (layer, enabled) in enabled_layers.iter().copied().enumerate() {
            let opacity = match selected_layer {
                Some(selected_layer) if selected_layer == layer => 1.0,
                Some(_) => 0.5,
                None => 1.0,
            };
            self.opacity
                .set_opacity(&graphics_state.render_state, layer, opacity);
            if self.use_push_constants {
                render_pass.set_push_constants(
                    wgpu::ShaderStages::FRAGMENT,
                    64 + 48,
                    bytemuck::bytes_of::<f32>(&opacity),
                );
            }
            if enabled {
                self.instances.draw(render_pass, layer);
            }
        }
        render_pass.pop_debug_group();
    }
}
