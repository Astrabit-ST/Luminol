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

pub mod collision;
pub mod grid;
pub mod sprite;
pub mod tiles;

pub struct BindGroupLayouts {
    sprite: wgpu::BindGroupLayout,
    tiles: wgpu::BindGroupLayout,
    collision: wgpu::BindGroupLayout,
    grid: wgpu::BindGroupLayout,
}

pub struct Pipelines {
    sprites: std::collections::HashMap<luminol_data::BlendMode, wgpu::RenderPipeline>,
    tiles: wgpu::RenderPipeline,
    collision: wgpu::RenderPipeline,
    grid: wgpu::RenderPipeline,
}

impl BindGroupLayouts {
    pub fn new(render_state: &luminol_egui_wgpu::RenderState) -> Self {
        Self {
            sprite: sprite::create_bind_group_layout(render_state),
            tiles: tiles::create_bind_group_layout(render_state),
            collision: collision::create_bind_group_layout(render_state),
            grid: grid::create_bind_group_layout(render_state),
        }
    }
}

macro_rules! create_pipelines {
(
    $render_state:ident, $bind_group_layouts:ident,
    $($name:ident: $fun:path),*
) => {{
    let mut composer = naga_oil::compose::Composer::default();
    $(
        let $name = match $fun(&mut composer, $render_state, $bind_group_layouts) {
            Ok(p) => p,
            Err(err) => {
                let err = err.emit_to_string(&composer);
                panic!("Error creating {} render pipeline:\n{err}", stringify!($name))
            }
        };
    )*
    Pipelines {
        $($name,)*
    }
}};
}

impl Pipelines {
    pub fn new(
        render_state: &luminol_egui_wgpu::RenderState,
        bind_group_layouts: &BindGroupLayouts,
    ) -> Self {
        create_pipelines! {
            render_state, bind_group_layouts,
            sprites: sprite::shader::create_sprite_shaders,
            tiles: tiles::shader::create_render_pipeline,
            collision: collision::shader::create_render_pipeline,
            grid: grid::shader::create_render_pipeline
        }
    }
}
