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

use std::sync::Arc;

use crate::{
    viewport::{self, Viewport},
    BindGroupBuilder, BindGroupLayoutBuilder, GraphicsState,
};

use instance::Instances;
use itertools::Itertools;
use vertex::Vertex;

mod instance;
pub(crate) mod shader;
mod vertex;

#[derive(Debug)]
pub struct Collision {
    pub instances: Instances,
    pub viewport: Arc<Viewport>,
    pub bind_group: Option<wgpu::BindGroup>,
}

#[derive(Debug, Clone)]
pub enum CollisionType {
    /// An event
    Event,
    /// A tile whose ID is less than 48 (i.e. a blank autotile)
    BlankTile,
    /// A tile whose ID is greater than or equal to 48
    Tile,
}

/// Determines the passage values for every position on the map, running `f(x, y, passage)` for
/// every position.
///
/// `layers` should be an iterator over the enabled layer numbers of the map from top to bottom.
pub fn calculate_passages(
    passages: &luminol_data::Table1,
    priorities: &luminol_data::Table1,
    tiles: &luminol_data::Table3,
    events: Option<&luminol_data::OptionVec<luminol_data::rpg::Event>>,
    layers: impl Iterator<Item = usize> + Clone,
    mut f: impl FnMut(usize, usize, i16),
) {
    let tileset_size = passages.len().min(priorities.len());

    let mut event_map = if let Some(events) = events {
        events
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
                    .map_or((15, 1, CollisionType::Event), |id| {
                        let tile_id = id + 1;
                        if tile_id >= tileset_size {
                            (0, 0, CollisionType::Event)
                        } else {
                            (passages[tile_id], priorities[tile_id], CollisionType::Event)
                        }
                    });
                Some(((event.x as usize, event.y as usize), tile_event))
            })
            .collect()
    } else {
        std::collections::HashMap::new()
    };

    for (y, x) in (0..tiles.ysize()).cartesian_product(0..tiles.xsize()) {
        let tile_event = event_map.remove(&(x, y));

        f(
            x,
            y,
            calculate_passage(tile_event.into_iter().chain(layers.clone().map(|z| {
                let tile_id = tiles[(x, y, z)].try_into().unwrap_or_default();
                let collision_type = if tile_id < 48 {
                    CollisionType::BlankTile
                } else {
                    CollisionType::Tile
                };
                if tile_id >= tileset_size {
                    (0, 0, collision_type)
                } else {
                    (passages[tile_id], priorities[tile_id], collision_type)
                }
            }))),
        );
    }
}

/// Determines the passage value for a position on the map given an iterator over the
/// `(passage, priority, collision_type)` values for the tiles in each layer on that position.
/// The iterator should iterate over the layers from top to bottom.
pub fn calculate_passage(layers: impl Iterator<Item = (i16, i16, CollisionType)> + Clone) -> i16 {
    let mut computed_passage = 0;

    for direction in [1, 2, 4, 8] {
        let mut at_least_one_layer_not_blank = false;
        let mut layers = layers.clone().peekable();
        while let Some((passage, priority, collision_type)) = layers.next() {
            if matches!(
                collision_type,
                CollisionType::Tile | CollisionType::BlankTile
            ) {
                if matches!(collision_type, CollisionType::BlankTile)
                    && (at_least_one_layer_not_blank || layers.peek().is_some())
                {
                    continue;
                } else {
                    at_least_one_layer_not_blank = true;
                }
            }
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

impl Collision {
    pub fn new(
        graphics_state: &GraphicsState,
        viewport: Arc<Viewport>,
        passages: &luminol_data::Table2,
    ) -> Self {
        let instances = Instances::new(&graphics_state.render_state, passages);

        let bind_group = (!graphics_state.push_constants_supported()).then(|| {
            let mut bind_group_builder = BindGroupBuilder::new();
            bind_group_builder.append_buffer(viewport.as_buffer().unwrap());
            bind_group_builder.build(
                &graphics_state.render_state.device,
                Some("collision bind group"),
                &graphics_state.bind_group_layouts.collision,
            )
        });

        Self {
            instances,
            viewport,
            bind_group,
        }
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
        graphics_state: &'rpass GraphicsState,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        render_pass.push_debug_group("tilemap collision renderer");
        render_pass.set_pipeline(&graphics_state.pipelines.collision);

        if let Some(bind_group) = &self.bind_group {
            render_pass.set_bind_group(0, bind_group, &[])
        } else {
            render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                0,
                &self.viewport.as_bytes(),
            );
        }

        self.instances.draw(render_pass);
        render_pass.pop_debug_group();
    }
}

pub fn create_bind_group_layout(render_state: &egui_wgpu::RenderState) -> wgpu::BindGroupLayout {
    let mut builder = BindGroupLayoutBuilder::new();

    if !crate::push_constants_supported(render_state) {
        viewport::add_to_bind_group_layout(&mut builder);
    }

    builder.build(&render_state.device, Some("collision bind group layout"))
}
