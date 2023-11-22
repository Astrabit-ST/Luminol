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

use instance::Instances;
use itertools::Itertools;
use vertex::Vertex;

mod instance;
pub(crate) mod shader;
mod vertex;

#[derive(Debug)]
pub struct Collision {
    pub instances: Instances,
    pub use_push_constants: bool,
}

impl Collision {
    pub fn new(
        graphics_state: &crate::GraphicsState,
        passages: &luminol_data::Table1,
        priorities: &luminol_data::Table1,
        tiles: &luminol_data::Table3,
        events: &luminol_data::OptionVec<luminol_data::rpg::Event>,
        use_push_constants: bool,
    ) -> Self {
        let mut event_map = events
            .iter()
            .filter_map(|(_, event)| {
                let Some(page) = event.pages.first() else {
                    return None;
                };
                if page.through {
                    return None;
                }
                let tile_event = page
                    .graphic
                    .tile_id
                    .map_or((15, 1), |id| (passages[id + 1], priorities[id + 1]));
                Some(((event.x as usize, event.y as usize), tile_event))
            })
            .collect::<std::collections::HashMap<_, _>>();

        let mut calculated_passages = luminol_data::Table2::new(tiles.xsize(), tiles.ysize());
        for (y, x) in (0..tiles.ysize()).cartesian_product(0..tiles.xsize()) {
            let tile_event = event_map.remove(&(x, y));

            calculated_passages[(x, y)] = Self::calculate_passage(tile_event.into_iter().chain(
                (0..tiles.zsize()).rev().map(|z| {
                    let tile_id = tiles[(x, y, z)].try_into().unwrap_or_default();
                    (passages[tile_id], priorities[tile_id])
                }),
            ));
        }

        let instances = Instances::new(&graphics_state.render_state, &calculated_passages);

        Self {
            instances,
            use_push_constants,
        }
    }

    /// Determines the passage value for a position on the map given an iterator over the
    /// `(passage, priority)` values for the tiles in each layer on that position.
    /// The iterator should iterate over the layers from top to bottom.
    pub fn calculate_passage(layers: impl Iterator<Item = (i16, i16)> + Clone) -> i16 {
        let mut computed_passage = 0;

        for direction in [1, 2, 4, 8] {
            for (passage, priority) in layers.clone() {
                if passage & direction != 0 {
                    computed_passage |= direction;
                    break;
                } else if priority == 0 {
                    break;
                }
            }
        }

        computed_passage
    }

    pub fn set_passage(
        &self,
        render_state: &egui_wgpu::RenderState,
        passage: i16,
        position: (usize, usize),
    ) {
        self.instances.set_passage(render_state, passage, position)
    }

    pub fn draw<'rpass>(
        &'rpass self,
        graphics_state: &'rpass crate::GraphicsState,
        viewport: &crate::viewport::Viewport,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        #[repr(C)]
        #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        struct VertexPushConstant {
            viewport: [u8; 64],
        }

        render_pass.push_debug_group("tilemap collision renderer");
        render_pass.set_pipeline(&graphics_state.pipelines.collision);
        if self.use_push_constants {
            render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                0,
                bytemuck::bytes_of(&VertexPushConstant {
                    viewport: viewport.as_bytes(),
                }),
            );
        }
        self.instances.draw(render_pass);
        render_pass.pop_debug_group();
    }
}
