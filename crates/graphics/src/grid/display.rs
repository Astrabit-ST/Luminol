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

use crossbeam::atomic::AtomicCell;
use wgpu::util::DeviceExt;

use crate::{BindGroupLayoutBuilder, GraphicsState};

#[derive(Debug)]
pub struct Display {
    data: AtomicCell<Data>,
    uniform: Option<wgpu::Buffer>,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Data {
    viewport_size_in_pixels: [f32; 2],
    pixels_per_point: f32,
    inner_thickness_in_points: f32,
}

impl Display {
    pub fn new(graphics_state: &GraphicsState) -> Self {
        let display = Data {
            viewport_size_in_pixels: [0., 0.],
            pixels_per_point: 1.,
            inner_thickness_in_points: 1.,
        };

        let uniform = (!graphics_state.push_constants_supported()).then(|| {
            graphics_state.render_state.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("grid display buffer"),
                    contents: bytemuck::bytes_of(&display),
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                },
            )
        });

        Display {
            data: AtomicCell::new(display),
            uniform,
        }
    }

    pub fn as_bytes(&self) -> [u8; std::mem::size_of::<Data>()] {
        bytemuck::cast(self.data.load())
    }

    pub fn as_buffer(&self) -> Option<&wgpu::Buffer> {
        self.uniform.as_ref()
    }

    pub fn set_inner_thickness(
        &self,
        render_state: &luminol_egui_wgpu::RenderState,
        inner_thickness_in_points: f32,
    ) {
        let data = self.data.load();
        if data.inner_thickness_in_points != inner_thickness_in_points {
            self.data.store(Data {
                inner_thickness_in_points,
                ..data
            });
            self.regen_buffer(render_state);
        }
    }

    pub(super) fn update_viewport_size(
        &self,
        render_state: &luminol_egui_wgpu::RenderState,
        info: &egui::PaintCallbackInfo,
    ) {
        let viewport_size = info.viewport_in_pixels();
        let viewport_size = [
            viewport_size.width_px as f32,
            viewport_size.height_px as f32,
        ];
        let pixels_per_point = info.pixels_per_point.max(1.).floor();
        let data = self.data.load();
        if data.viewport_size_in_pixels != viewport_size
            || data.pixels_per_point != pixels_per_point
        {
            self.data.store(Data {
                viewport_size_in_pixels: viewport_size,
                pixels_per_point,
                ..data
            });
            self.regen_buffer(render_state);
        }
    }

    fn regen_buffer(&self, render_state: &luminol_egui_wgpu::RenderState) {
        if let Some(uniform) = &self.uniform {
            render_state
                .queue
                .write_buffer(uniform, 0, bytemuck::bytes_of(&self.data.load()));
        }
    }
}

pub fn add_to_bind_group_layout(
    layout_builder: &mut BindGroupLayoutBuilder,
) -> &mut BindGroupLayoutBuilder {
    layout_builder.append(
        wgpu::ShaderStages::FRAGMENT,
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        None,
    )
}
