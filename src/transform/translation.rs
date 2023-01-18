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

//! Translation along a vector.

use super::Transform;
use crate::point::{Point, Vector};

use core::ops;

/// A translation along a vector.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Translation<T: Copy>(Vector<T>);

#[cfg(feature = "arbitrary")]
impl<'a, T: Copy + arbitrary::Arbitrary<'a>> arbitrary::Arbitrary<'a> for Translation<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Translation(Vector::arbitrary(u)?))
    }
}

impl<T: Copy> Translation<T> {
    /// Create a new translation.
    #[inline]
    pub fn new(vector: Vector<T>) -> Self {
        Translation(vector)
    }

    /// Get the translation vector.
    #[inline]
    pub fn vector(&self) -> Vector<T> {
        self.0
    }
}

impl<T: Copy> From<Vector<T>> for Translation<T> {
    #[inline]
    fn from(vector: Vector<T>) -> Self {
        Translation::new(vector)
    }
}

#[cfg(feature = "euclid")]
impl<T: Copy, Src, Dst> From<euclid::Translation2D<T, Src, Dst>> for Translation<T> {
    #[inline]
    fn from(translation: euclid::Translation2D<T, Src, Dst>) -> Self {
        Translation::new(Vector::new(translation.x, translation.y))
    }
}

#[cfg(feature = "euclid")]
impl<T: Copy, Src, Dst> From<Translation<T>> for euclid::Translation2D<T, Src, Dst> {
    #[inline]
    fn from(tx: Translation<T>) -> euclid::Translation2D<T, Src, Dst> {
        euclid::Translation2D::new(tx.0.x(), tx.0.y())
    }
}

impl<T: Copy + ops::Add<Output = T>> Transform<T> for Translation<T> {
    #[inline]
    fn transform_point(&self, point: Point<T>) -> Point<T> {
        point + self.0
    }
}
