// Copyright 2023 John Nunley
//
// This file is part of blood-geometry.
// 
// blood-geometry is free software: you can redistribute it and/or modify it 
// under the terms of the GNU Affero General Public License as published by 
// the Free Software Foundation, either version 3 of the License, or (at your 
// option) any later version.
// 
// blood-geometry is distributed in the hope that it will be useful, but 
// WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY 
// or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License 
// for more details.
// 
// You should have received a copy of the GNU Affero General Public License 
// along with blood-geometry. If not, see <https://www.gnu.org/licenses/>. 

//! A scaling relative to the origin.

use super::Transform;
use crate::point::{Point, Vector};

use core::ops;

/// A scaling relative to the origin.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Scale<T: Copy>(Vector<T>);

#[cfg(feature = "arbitrary")]
impl<'a, T: Copy + arbitrary::Arbitrary<'a>> arbitrary::Arbitrary<'a> for Scale<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Scale(Vector::arbitrary(u)?))
    }
}

impl<T: Copy> Scale<T> {
    /// Create a new scaling.
    #[inline]
    pub fn new(vector: Vector<T>) -> Self {
        Scale(vector)
    }

    /// Create a new uniform scaling.
    #[inline]
    pub fn uniform(scale: T) -> Self {
        Scale(Vector::splat(scale))
    }

    /// Get the scaling vector.
    #[inline]
    pub fn vector(&self) -> Vector<T> {
        self.0
    }
}

impl<T: Copy> From<Vector<T>> for Scale<T> {
    #[inline]
    fn from(vector: Vector<T>) -> Self {
        Scale::new(vector)
    }
}

#[cfg(feature = "euclid")]
impl<T: Copy, Src, Dst> From<euclid::Scale<T, Src, Dst>> for Scale<T> {
    #[inline]
    fn from(scale: euclid::Scale<T, Src, Dst>) -> Self {
        Scale::uniform(scale.0)
    }
}

impl<T: Copy + ops::Mul<Output = T>> Transform<T> for Scale<T> {
    #[inline]
    fn transform_point(&self, point: Point<T>) -> Point<T> {
        Point(point.0 * self.0 .0)
    }
}
