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

//! A geometric arc.

use num_traits::real::Real;

use crate::angle::Angle;
use crate::path::{Path, PathEvent};
use crate::point::Point;

/// A geometric arc.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Arc<T: Copy> {
    /// The center of the arc.
    center: Point<T>,

    /// The radius of the arc.
    radius: T,

    /// The start angle of the arc.
    start_angle: Angle<T>,

    /// The end angle of the arc.
    end_angle: Angle<T>,
}

#[cfg(feature = "arbitrary")]
impl<'a, T: Copy + arbitrary::Arbitrary<'a>> arbitrary::Arbitrary<'a> for Arc<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Arc {
            center: arbitrary::Arbitrary::arbitrary(u)?,
            radius: arbitrary::Arbitrary::arbitrary(u)?,
            start_angle: arbitrary::Arbitrary::arbitrary(u)?,
            end_angle: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

impl<T: Copy> Arc<T> {
    /// Create a new `Arc` from the center, radius, start angle, and end angle.
    pub fn new(center: Point<T>, radius: T, start_angle: Angle<T>, end_angle: Angle<T>) -> Self {
        Arc {
            center,
            radius,
            start_angle,
            end_angle,
        }
    }

    /// Get the center of the arc.
    pub fn center(self) -> Point<T> {
        self.center
    }

    /// Get the radius of the arc.
    pub fn radius(self) -> T {
        self.radius
    }

    /// Get the start angle of the arc.
    pub fn start_angle(self) -> Angle<T> {
        self.start_angle
    }

    /// Get the end angle of the arc.
    pub fn end_angle(self) -> Angle<T> {
        self.end_angle
    }

    /// Reverse the direction of the arc.
    pub fn reverse(self) -> Self
    where
        T: Real,
    {
        let full_circle = T::from(core::f32::consts::PI * 2.0).unwrap();

        Self {
            center: self.center,
            radius: self.radius,
            start_angle: self.end_angle,
            end_angle: Angle::from_radians(
                (self.start_angle().radians() + full_circle) % full_circle,
            ),
        }
    }
}

impl<T: Real> Path<T> for Arc<T> {
    type Iter = ArcPathIter<T>;

    fn path_iter(self) -> Self::Iter {
        ArcPathIter { arc: self }
    }
}

#[doc(hidden)]
pub struct ArcPathIter<T: Copy> {
    /// The inner arc.
    arc: Arc<T>,
}

impl<T: Real> Iterator for ArcPathIter<T> {
    type Item = PathEvent<T>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
