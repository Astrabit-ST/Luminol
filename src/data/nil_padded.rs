use std::ops::{Deref, DerefMut};

// Copyright (C) 2022 Lily Lyons
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
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(from = "Vec<Option<T>>")]
pub struct NilPadded<T>(Vec<T>);

impl<T> Deref for NilPadded<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for NilPadded<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Default> Default for NilPadded<T> {
    fn default() -> Self {
        Self(vec![])
    }
}

impl<T> From<Vec<Option<T>>> for NilPadded<T> {
    fn from(value: Vec<Option<T>>) -> Self {
        let mut iter = value.into_iter();

        assert!(
            iter.next()
                .expect("there should be at least one element")
                .is_none(),
            "the array should be padded with nil at the first index"
        );
        Self(iter.flatten().collect())
    }
}

impl<T> From<Vec<T>> for NilPadded<T> {
    fn from(value: Vec<T>) -> Self {
        Self(value)
    }
}
