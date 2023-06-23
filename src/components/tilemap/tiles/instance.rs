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
use wgpu::util::DeviceExt;

use super::Quad;
use crate::prelude::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    position: [f32; 3],
    tile_id: i32, // force this to be an i32 to avoid padding issues
}

#[derive(Debug)]
pub struct Instances {
    instance_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    instances: u32,
}

const TILE_QUAD: Quad = Quad::new(
    egui::Rect::from_min_max(egui::pos2(0., 0.), egui::pos2(32., 32.0)),
    egui::Rect::from_min_max(egui::pos2(0., 0.), egui::pos2(32., 32.0)),
    0.0,
);

impl Instances {
    pub fn new(map_data: &Table3, atlas_size: wgpu::Extent3d) -> Self {
        let instances = Self::calculate_instances(map_data);
        let instance_buffer =
            state!()
                .render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("tilemap tiles instance buffer"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });
        let instances = instances.len() as u32;

        let (vertex_buffer, _) = Quad::into_buffer(&[TILE_QUAD], atlas_size);

        Self {
            instance_buffer,
            vertex_buffer,
            instances,
        }
    }

    fn calculate_instances(map_data: &Table3) -> Vec<Instance> {
        map_data
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, tile_id)| *tile_id >= 48)
            .map(|(index, tile_id)| {
                // We reset the x every xsize elements.
                let map_x = index % map_data.xsize();
                // We reset the y every ysize elements, but only increment it every xsize elements.
                let map_y = (index / map_data.xsize()) % map_data.ysize();
                // We change the z every xsize * ysize elements.
                let map_z = index / (map_data.xsize() * map_data.ysize());

                Instance {
                    position: [
                        map_x as f32,
                        map_y as f32,
                        1. - (map_z as f32 / map_data.zsize() as f32), // reverse tile order (higher z is closer?)
                    ],
                    tile_id: tile_id as i32,
                }
            })
            .collect_vec()
    }

    pub fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.draw(0..6, 0..self.instances);
    }

    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ARRAY: &[wgpu::VertexAttribute] =
            &wgpu::vertex_attr_array![2 => Float32x3, 3 => Sint32];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ARRAY,
        }
    }
}
