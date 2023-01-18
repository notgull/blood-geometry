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

//! A rotation relative to the origin.

use super::Transform;
use crate::angle::Angle;
use crate::point::Point;
use num_traits::real::Real;

/// A rotation around the origin.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(transparent)]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Rotation<T: Copy>(Angle<T>);

#[cfg(feature = "arbitrary")]
impl<'a, T: Copy + arbitrary::Arbitrary<'a>> arbitrary::Arbitrary<'a> for Rotation<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Rotation(Angle::arbitrary(u)?))
    }
}

impl<T: Copy> Rotation<T> {
    /// Create a new rotation.
    #[inline]
    pub fn new(angle: Angle<T>) -> Self {
        Rotation(angle)
    }

    /// Get the rotation angle.
    #[inline]
    pub fn angle(&self) -> Angle<T> {
        self.0
    }
}

impl<T: Copy> From<Angle<T>> for Rotation<T> {
    #[inline]
    fn from(angle: Angle<T>) -> Self {
        Rotation::new(angle)
    }
}

#[cfg(feature = "euclid")]
impl<T: Copy, Src, Dst> From<euclid::Rotation2D<T, Src, Dst>> for Rotation<T> {
    #[inline]
    fn from(rotation: euclid::Rotation2D<T, Src, Dst>) -> Self {
        Rotation::new(Angle::from_radians(rotation.angle))
    }
}

#[cfg(feature = "euclid")]
impl<T: Copy, Src, Dst> From<Rotation<T>> for euclid::Rotation2D<T, Src, Dst> {
    #[inline]
    fn from(rotation: Rotation<T>) -> euclid::Rotation2D<T, Src, Dst> {
        euclid::Rotation2D::new(euclid::Angle::radians(rotation.0.radians()))
    }
}

impl<T: Copy + Real> Transform<T> for Rotation<T> {
    fn transform_point(&self, point: Point<T>) -> Point<T> {
        let sin = self.0.sin();
        let cos = self.0.cos();

        Point::new(
            point.x() * cos - point.y() * sin,
            point.x() * sin + point.y() * cos,
        )
    }
}
