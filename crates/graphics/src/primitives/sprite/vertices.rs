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

use crate::Quad;

#[derive(Debug)]
pub struct Vertices {
    pub vertex_buffer: wgpu::Buffer,
}

impl Vertices {
    pub fn from_quads(
        render_state: &luminol_egui_wgpu::RenderState,
        quads: &[Quad],
        extents: wgpu::Extent3d,
    ) -> Self {
        let (vertex_buffer, _) = Quad::into_buffer(render_state, quads, extents);
        Self { vertex_buffer }
    }

    pub fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..1)
    }
}
