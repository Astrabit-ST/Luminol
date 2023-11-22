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

#[cfg(target_arch = "wasm32")]
pub mod bindings;
#[cfg(target_arch = "wasm32")]
pub mod web_worker_runner;
#[cfg(target_arch = "wasm32")]
pub use web_worker_runner::WebWorkerRunner;
#[cfg(target_arch = "wasm32")]
pub use web_worker_runner::WebWorkerRunnerEvent;
#[cfg(target_arch = "wasm32")]
pub use web_worker_runner::WebWorkerRunnerOutput;
