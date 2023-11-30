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

#[derive(Debug)]
pub struct Graphic {
    data: AtomicCell<Data>,
    uniform: Option<wgpu::Buffer>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Data {
    hue: f32,
    opacity: f32,
    opacity_multiplier: f32,
    _padding: u32,
}

impl Graphic {
    pub fn new(
        graphics_state: &crate::GraphicsState,
        hue: i32,
        opacity: i32,
        use_push_constants: bool,
    ) -> Self {
        let hue = (hue % 360) as f32 / 360.0;
        let opacity = opacity as f32 / 255.;
        let data = Data {
            hue,
            opacity,
            opacity_multiplier: 1.,
            _padding: 0,
        };

        let uniform = (!use_push_constants).then(|| {
            graphics_state.render_state.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("tilemap sprite graphic buffer"),
                    contents: bytemuck::cast_slice(&[data]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                },
            )
        });

        Self {
            data: AtomicCell::new(data),
            uniform,
        }
    }

    pub fn hue(&self) -> i32 {
        (self.data.load().hue * 360.) as i32
    }

    pub fn set_hue(&self, render_state: &egui_wgpu::RenderState, hue: i32) {
        let hue = (hue % 360) as f32 / 360.0;
        let data = self.data.load();

        if data.hue != hue {
            self.data.store(Data { hue, ..data });
            self.regen_buffer(render_state);
        }
    }

    pub fn opacity(&self) -> i32 {
        (self.data.load().opacity * 255.) as i32
    }

    pub fn set_opacity(&self, render_state: &egui_wgpu::RenderState, opacity: i32) {
        let opacity = opacity as f32 / 255.0;
        let data = self.data.load();

        if data.opacity != opacity {
            self.data.store(Data { opacity, ..data });
            self.regen_buffer(render_state);
        }
    }

    pub fn opacity_multiplier(&self) -> f32 {
        self.data.load().opacity_multiplier
    }

    pub fn set_opacity_multiplier(
        &self,
        render_state: &egui_wgpu::RenderState,
        opacity_multiplier: f32,
    ) {
        let data = self.data.load();

        if data.opacity_multiplier != opacity_multiplier {
            self.data.store(Data {
                opacity_multiplier,
                ..data
            });
            self.regen_buffer(render_state);
        }
    }

    pub fn as_bytes(&self) -> [u8; std::mem::size_of::<Data>()] {
        bytemuck::cast(self.data.load())
    }

    pub fn as_buffer(&self) -> Option<&wgpu::Buffer> {
        self.uniform.as_ref()
    }

    fn regen_buffer(&self, render_state: &egui_wgpu::RenderState) {
        if let Some(uniform) = &self.uniform {
            render_state
                .queue
                .write_buffer(uniform, 0, bytemuck::cast_slice(&[self.data.load()]));
        }
    }
}

pub fn add_to_bind_group_layout(
    layout_builder: &mut crate::BindGroupLayoutBuilder,
) -> &mut crate::BindGroupLayoutBuilder {
    layout_builder.append(
        wgpu::ShaderStages::VERTEX_FRAGMENT,
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        None,
    )
}
