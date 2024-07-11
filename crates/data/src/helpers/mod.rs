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

pub mod id_serde;
pub mod id_vec_serde;
pub mod nil_padded_serde;
pub mod optional_id_serde;
pub mod optional_path_serde;

pub mod id_alox;
pub mod id_vec_alox;
pub mod nil_padded_alox;
pub mod optional_id_alox;
pub mod optional_path_alox;

mod parameter_type;

pub use parameter_type::*;
