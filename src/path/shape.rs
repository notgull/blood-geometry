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

//! The closed version of a path.

use super::Path;
use crate::box2d::Box;
use crate::{ApproxEq, FillRule};
use num_traits::real::Real;

/// Represents a closed path, or a specific shape.
///
/// This is, by and large, a marker trait for `Path`s that are closed.
pub trait Shape<T: Copy>: Path<T> {
    /// Tesselate this shape into a form that can be represented by a series of
    /// trapezoids.
    #[cfg(feature = "alloc")]
    fn trapezoids(self, tolerance: T) -> crate::bentley_ottman::Trapezoids<T>
    where
        Self: Sized,
        T: Real + ApproxEq,
    {
        crate::bentley_ottman::trapezoids(self.segments(tolerance), FillRule::Winding)
    }

    /// Get the area of the shape.
    #[cfg(feature = "alloc")]
    fn area(self, accuracy: T) -> T
    where
        Self: Sized,
        T: Real + ApproxEq,
    {
        self.trapezoids(accuracy)
            .fold(T::zero(), |area, trapezoid| area + trapezoid.area(accuracy))
    }

    /// Get the perimeter of the shape.
    ///
    /// By default, this is implemented by just calling the `length` method on the path.
    fn perimeter(self, accuracy: T) -> T
    where
        Self: Sized,
        T: Real + ApproxEq,
    {
        self.approximate_length(accuracy)
    }

    /// Get the bounding box of the shape.
    #[cfg(feature = "alloc")]
    fn bounding_box(self, accuracy: T) -> Box<T>
    where
        Self: Sized,
        T: Real + ApproxEq,
    {
        self.trapezoids(accuracy)
            .fold(Box::unbounded_real(), |box_, trapezoid| {
                box_.union(&crate::BoundingBox::bounding_box(&trapezoid))
            })
    }
}
