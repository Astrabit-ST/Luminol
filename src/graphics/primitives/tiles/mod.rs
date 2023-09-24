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
use crate::prelude::*;

pub use atlas::Atlas;

use autotiles::Autotiles;
use instance::Instances;
use shader::Shader;

mod atlas;
mod autotile_ids;
mod autotiles;
mod instance;
mod shader;

#[derive(Debug)]
pub struct Tiles {
    pub autotiles: Autotiles,
    pub atlas: Atlas,
    pub instances: Instances,
}

impl Tiles {
    pub fn new(atlas: Atlas, tiles: &Table3) -> Self {
        let autotiles = Autotiles::new(&atlas);
        let instances = Instances::new(tiles, atlas.atlas_texture.size());

        Self {
            autotiles,
            atlas,
            instances,
        }
    }

    pub fn set_tile(&self, tile_id: i16, position: (usize, usize, usize)) {
        self.instances.set_tile(tile_id, position)
    }

    pub fn draw<'rpass>(
        &'rpass self,
        viewport: &primitives::Viewport,
        enabled_layers: &[bool],
        selected_layer: Option<usize>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        #[repr(C)]
        #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        struct VertexPushConstant {
            viewport: [u8; 64],
            autotiles: [u8; 36],
        }

        render_pass.push_debug_group("tilemap tiles renderer");
        Shader::bind(render_pass);
        render_pass.set_push_constants(
            wgpu::ShaderStages::VERTEX,
            0,
            bytemuck::bytes_of(&VertexPushConstant {
                viewport: viewport.as_bytes(),
                autotiles: self.autotiles.as_bytes(),
            }),
        );

        self.atlas.bind(render_pass);

        for (layer, enabled) in enabled_layers.iter().copied().enumerate() {
            let opacity = match selected_layer {
                Some(selected_layer) if selected_layer == layer => 1.0,
                Some(_) => 0.5,
                None => 1.0,
            };
            render_pass.set_push_constants(
                wgpu::ShaderStages::FRAGMENT,
                64 + 36,
                bytemuck::bytes_of::<f32>(&opacity),
            );
            if enabled {
                self.instances.draw(render_pass, layer);
            }
        }
        render_pass.pop_debug_group();
    }
}
