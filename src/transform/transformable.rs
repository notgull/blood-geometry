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

//! Trait for types that can be transformed.

use super::{Affine, Rotation, Scale, Transform, Translation};
use crate::{Point, Vector};
use num_traits::real::Real;

use core::ops;

/// An object that can have transformations applied to it.
pub trait Transformable<T: Copy>: Sized {
    /// Apply a transformation to this object.
    fn transform(&self, transform: impl Transform<T>) -> Self;

    /// Translate this object along a vector;
    fn translate(&self, translation: impl Into<Translation<T>>) -> Self
    where
        T: ops::Add<Output = T>,
    {
        self.transform(translation.into())
    }

    /// Rotate this object relative to the origin
    fn rotate(&self, rotation: impl Into<Rotation<T>>) -> Self
    where
        T: Real,
    {
        self.transform(rotation.into())
    }

    /// Scale this object relative to the origin.
    fn scale(&self, scale: impl Into<Scale<T>>) -> Self
    where
        T: ops::Mul<Output = T>,
    {
        self.transform(scale.into())
    }

    /// Uniformly scale this object relative to the origin.
    fn scale_uniform(&self, scale: T) -> Self
    where
        T: ops::Mul<Output = T>,
    {
        self.transform(Scale::uniform(scale))
    }

    /// Apply an affine transformation to this object.
    fn affine(&self, affine: impl Into<Affine<T>>) -> Self
    where
        T: ops::Mul<Output = T> + ops::Add<Output = T>,
    {
        self.transform(affine.into())
    }
}

impl<T: Copy> Transformable<T> for Point<T> {
    fn transform(&self, transform: impl Transform<T>) -> Self {
        transform.transform_point(*self)
    }
}

impl<T: Copy> Transformable<T> for Vector<T> {
    fn transform(&self, transform: impl Transform<T>) -> Self {
        transform.transform_vector(*self)
    }
}
