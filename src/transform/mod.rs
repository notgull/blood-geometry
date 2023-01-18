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

use crate::{Point, Triangle, Vector};

mod affine;
mod rotation;
mod scale;
mod transformable;
mod translation;

pub use affine::Affine;
pub use rotation::Rotation;
pub use scale::Scale;
pub use transformable::Transformable;
pub use translation::Translation;

/// Represents a transformation that can be applied to a geometric type.
pub trait Transform<T: Copy> {
    /// Apply the transformation to a point.
    fn transform_point(&self, point: Point<T>) -> Point<T>;

    /// Apply the transformation to a vector.
    #[inline]
    fn transform_vector(&self, vector: Vector<T>) -> Vector<T> {
        Vector(self.transform_point(Point(vector.0)).0)
    }

    /// Apply the transformation to a triangle.
    #[inline]
    fn transform_triangle(&self, triangle: Triangle<T>) -> Triangle<T> {
        Triangle::new(
            self.transform_point(triangle.a()),
            self.transform_point(triangle.b()),
            self.transform_point(triangle.c()),
        )
    }
}

impl<T: Copy, Tr: Transform<T> + ?Sized> Transform<T> for &Tr {
    #[inline]
    fn transform_point(&self, point: Point<T>) -> Point<T> {
        (**self).transform_point(point)
    }
}

#[cfg(feature = "alloc")]
impl<T: Copy, Tr: Transform<T> + ?Sized> Transform<T> for alloc::boxed::Box<Tr> {
    #[inline]
    fn transform_point(&self, point: Point<T>) -> Point<T> {
        (**self).transform_point(point)
    }
}
