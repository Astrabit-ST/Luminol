use eframe::wgpu::util::DeviceExt;

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
use super::shader::Shader;
use crate::prelude::*;
use crossbeam::atomic::AtomicCell;

#[derive(Debug)]
pub struct Uniform {
    viewport: AtomicCell<Viewport>,
    viewport_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, PartialEq)]
struct Viewport {
    proj: cgmath::Matrix4<f32>,
    // The scale of the tilemap.
    scale: f32,
}

// SAFETY:
//
//
#[allow(unsafe_code)]
unsafe impl bytemuck::Pod for Viewport {}
#[allow(unsafe_code)]
unsafe impl bytemuck::Zeroable for Viewport {}

impl Uniform {
    pub fn new() -> Self {
        let viewport = Viewport {
            proj: cgmath::ortho(0.0, 800.0, 600.0, 0.0, -1.0, 1.0),
            // pan: egui::Vec2::ZERO,
            scale: 100.,
        };

        let render_state = &state!().render_state;

        let viewport_buffer =
            render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("tilemap viewport buffer"),
                    contents: bytemuck::cast_slice(&[viewport]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let bind_group = render_state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("tilemap uniform bind group"),
                layout: Shader::uniform_layout(),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: viewport_buffer.as_entire_binding(),
                }],
            });

        Uniform {
            viewport: AtomicCell::new(viewport),
            viewport_buffer,

            bind_group,
        }
    }

    pub fn set_proj(&self, proj: cgmath::Matrix4<f32>) {
        self.viewport.store(Viewport {
            proj,
            ..self.viewport.load()
        });
        self.regen_buffer();
    }

    pub fn scale(&self) -> f32 {
        self.viewport.load().scale
    }

    pub fn set_scale(&self, scale: f32) {
        self.viewport.store(Viewport {
            scale,
            ..self.viewport.load()
        });
        self.regen_buffer();
    }

    /*
    pub fn pan(&self) -> egui::Vec2 {
        self.viewport.load().pan
    }

    pub fn set_pan(&self, pan: egui::Vec2) {
        self.viewport.store(Viewport {
            pan,
            ..self.viewport.load()
        });
        self.regen_buffer();
    }
    */

    fn regen_buffer(&self) {
        let render_state = &state!().render_state;
        render_state.queue.write_buffer(
            &self.viewport_buffer,
            0,
            bytemuck::cast_slice(&[self.viewport.load()]),
        );
    }

    pub fn bind<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.set_bind_group(1, &self.bind_group, &[]);
    }
}
