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

/// Egui inspection window.
#[derive(Default)]
pub struct EguiInspection {}

impl luminol_core::Window for EguiInspection {
    fn id(&self) -> egui::Id {
        egui::Id::new("Egui Inspection")
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        _update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        egui::Window::new("Egui Inspection")
            .open(open)
            .show(ctx, |ui| ctx.inspection_ui(ui));
    }
}

/// Egui memory display.
#[derive(Default)]
pub struct EguiMemory {}

impl luminol_core::Window for EguiMemory {
    fn id(&self) -> egui::Id {
        egui::Id::new("Egui Memory")
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        _update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        egui::Window::new("Egui Memory")
            .open(open)
            .show(ctx, |ui| ctx.memory_ui(ui));
    }
}

#[derive(Default)]
pub struct FilesystemDebug {}

impl luminol_core::Window for FilesystemDebug {
    fn id(&self) -> egui::Id {
        egui::Id::new("Filesystem Debug Window")
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        egui::Window::new("Filesystem Debug")
            .open(open)
            .show(ctx, |ui| update_state.filesystem.debug_ui(ui));
    }
}

pub struct WgpuDebugInfo {
    adapter_info: wgpu::AdapterInfo,
    adapter_features: wgpu::Features,
    adapter_limits: wgpu::Limits,
    downlevel_caps: wgpu::DownlevelCapabilities,
}

impl WgpuDebugInfo {
    pub fn new(update_state: &luminol_core::UpdateState<'_>) -> Self {
        let adapter_info = update_state.graphics.render_state.adapter.get_info();
        let adapter_features = update_state.graphics.render_state.adapter.features();
        let adapter_limits = update_state.graphics.render_state.adapter.limits();
        let downlevel_caps = update_state
            .graphics
            .render_state
            .adapter
            .get_downlevel_capabilities();

        Self {
            adapter_info,
            adapter_features,
            adapter_limits,
            downlevel_caps,
        }
    }
}

impl luminol_core::Window for WgpuDebugInfo {
    fn id(&self) -> egui::Id {
        egui::Id::new("wgpu debug info window")
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        _: &mut luminol_core::UpdateState<'_>,
    ) {
        egui::Window::new("WGPU Debug Info")
            .open(open)
            .scroll([false, true])
            .show(ctx, |ui| {
                if !self.downlevel_caps.is_webgpu_compliant() {
                    ui.label(
                        egui::RichText::new("🔥 Adapter is not WebGPU compliant")
                            .color(egui::Color32::RED),
                    );
                }

                ui.heading("Adapter info");
                ui.separator();

                ui.add_space(16.);
                ui.label(format!("{:#?}", self.adapter_info));
                ui.add_space(16.);

                ui.heading("Device features");
                ui.separator();

                ui.add_space(16.);
                for (name, _) in self.adapter_features.iter_names() {
                    ui.label(name);
                }
                ui.add_space(16.);

                ui.heading("Device limits");
                ui.separator();

                ui.add_space(16.);
                ui.label(format!("{:#?}", self.adapter_limits));
                ui.add_space(16.);

                ui.heading("Downlevel capabilities");
                ui.separator();

                ui.add_space(16.);
                ui.label(format!("{:#?}", self.downlevel_caps.shader_model));
                ui.add_space(16.);

                for (name, _) in self.downlevel_caps.flags.iter_names() {
                    ui.label(name);
                }
            });
    }
}
