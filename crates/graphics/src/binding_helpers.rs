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

pub struct BindGroupLayoutBuilder {
    entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl Default for BindGroupLayoutBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BindGroupLayoutBuilder {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn append(
        &mut self,
        visibility: wgpu::ShaderStages,
        ty: wgpu::BindingType,
        count: Option<std::num::NonZeroU32>,
    ) -> &mut Self {
        self.entries.push(wgpu::BindGroupLayoutEntry {
            binding: self.entries.len() as u32,
            visibility,
            ty,
            count,
        });
        self
    }

    #[must_use]
    pub fn build(self, device: &wgpu::Device, label: wgpu::Label<'_>) -> wgpu::BindGroupLayout {
        let descriptor = wgpu::BindGroupLayoutDescriptor {
            label,
            entries: &self.entries,
        };
        device.create_bind_group_layout(&descriptor)
    }
}

pub struct BindGroupBuilder<'res> {
    entries: Vec<wgpu::BindGroupEntry<'res>>,
}

impl<'res> Default for BindGroupBuilder<'res> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'res> BindGroupBuilder<'res> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn append(&mut self, resource: wgpu::BindingResource<'res>) -> &mut Self {
        self.entries.push(wgpu::BindGroupEntry {
            binding: self.entries.len() as u32,
            resource,
        });
        self
    }

    pub fn append_buffer(&mut self, buffer: &'res wgpu::Buffer) -> &mut Self {
        self.append(buffer.as_entire_binding())
    }

    pub fn append_buffer_with_size(&mut self, buffer: &'res wgpu::Buffer, size: u64) -> &mut Self {
        self.append(wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer,
            offset: 0,
            size: std::num::NonZeroU64::new(size),
        }))
    }

    pub fn append_sampler(&mut self, sampler: &'res wgpu::Sampler) -> &mut Self {
        self.append(wgpu::BindingResource::Sampler(sampler))
    }

    pub fn append_sampler_array(
        &mut self,
        sampler_array: &'res [&'res wgpu::Sampler],
    ) -> &mut Self {
        self.append(wgpu::BindingResource::SamplerArray(sampler_array))
    }

    pub fn append_texture_view(&mut self, texture: &'res wgpu::TextureView) -> &mut Self {
        self.append(wgpu::BindingResource::TextureView(texture))
    }

    pub fn append_texture_view_array(
        &mut self,
        texture_view_array: &'res [&'res wgpu::TextureView],
    ) -> &mut Self {
        self.append(wgpu::BindingResource::TextureViewArray(texture_view_array))
    }

    #[must_use]
    pub fn build(
        self,
        device: &wgpu::Device,
        label: wgpu::Label<'_>,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        let descriptor = wgpu::BindGroupDescriptor {
            label,
            layout,
            entries: &self.entries,
        };
        device.create_bind_group(&descriptor)
    }
}
