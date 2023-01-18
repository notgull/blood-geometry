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

//! Affine transformations.

use super::Transform;
use crate::angle::Angle;
use crate::point::Point;
use crate::pair::{Double, Quad};
use num_traits::{real::Real, One, Zero};

use core::ops;

/// An affine transformation.
// Most code here is taken from
// https://docs.rs/kurbo/latest/src/kurbo/affine.rs.html
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Affine<T: Copy> {
    /// The 4x4 top-left matrix.
    matrix: Quad<T>,

    /// The transformation applied.
    transform: Double<T>,
}

#[cfg(feature = "arbitrary")]
impl<'a, T: Copy + arbitrary::Arbitrary<'a>> arbitrary::Arbitrary<'a> for Affine<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let matrix: [T; 4] = u.arbitrary()?;
        let transform: [T; 2] = u.arbitrary()?;
        Ok(Affine {
            matrix: Quad::new(matrix),
            transform: Double::new(transform),
        })
    }
}

#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone)]
#[serde(rename = "Affine", transparent)]
#[repr(transparent)]
struct LogicalAffine<T>([T; 6]);

#[cfg(feature = "serde")]
impl<T: Copy + serde::Serialize> serde::Serialize for Affine<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        LogicalAffine(self.as_coefficients()).serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Copy + serde::Deserialize<'de>> serde::Deserialize<'de> for Affine<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        LogicalAffine::deserialize(deserializer)
            .map(|LogicalAffine(coefficients)| Self::new(coefficients))
    }
}

impl<T: Copy> Affine<T> {
    /// Create a new affine transformation.
    #[inline]
    pub fn new(coefficients: [T; 6]) -> Self {
        let [a, b, c, d, e, f] = coefficients;

        Affine {
            matrix: Quad::new([a, b, c, d]),
            transform: Double::new([e, f]),
        }
    }

    /// Get the coefficients of the affine transformation.
    #[inline]
    pub fn as_coefficients(&self) -> [T; 6] {
        let [a, b, c, d] = self.matrix.into_inner();
        let [e, f] = self.transform.into_inner();

        [a, b, c, d, e, f]
    }

    /// Get an affine transformation that represents a scaling.
    #[inline]
    pub fn scale(x: T, y: T) -> Self
    where
        T: Zero,
    {
        Self::new([x, T::zero(), T::zero(), y, T::zero(), T::zero()])
    }

    /// Get an affine transformation that represents a rotation.
    #[inline]
    pub fn rotate(angle: Angle<T>) -> Self
    where
        T: Zero + Real,
    {
        let sin = angle.sin();
        let cos = angle.cos();

        Self::new([cos, -sin, sin, cos, T::zero(), T::zero()])
    }

    /// Get an affine transformation that represents a translation.
    #[inline]
    pub fn translate(x: T, y: T) -> Self
    where
        T: Zero + One,
    {
        Self::new([T::one(), T::zero(), T::zero(), T::one(), x, y])
    }

    /// Get the determinant of the affine transformation.
    #[inline]
    pub fn determinant(&self) -> T
    where
        T: ops::Sub<Output = T> + ops::Mul<Output = T>,
    {
        let [a, b, c, d] = self.matrix.into_inner();

        a * d - b * c
    }

    /// Get the inverse of the affine transformation.
    #[inline]
    pub fn inverse(&self) -> Self
    where
        T: Real,
    {
        let inverse_det = self.determinant().recip();
        let [a, b, c, d, e, f] = self.as_coefficients();

        Self::new([
            d * inverse_det,
            -b * inverse_det,
            -c * inverse_det,
            a * inverse_det,
            (c * f - d * e) * inverse_det,
            (b * e - a * f) * inverse_det,
        ])
    }
}

impl<T: Copy + Zero + One> Default for Affine<T> {
    #[inline]
    fn default() -> Self {
        Self::new([
            T::one(),
            T::zero(),
            T::zero(),
            T::one(),
            T::zero(),
            T::zero(),
        ])
    }
}

impl<T: Copy + ops::Mul<Output = T> + ops::Add<Output = T>> Transform<T> for Affine<T> {
    fn transform_point(&self, point: Point<T>) -> Point<T> {
        let (lo, hi) = self.matrix.split();
        let point_swapped = point.0.swap();

        Point(((lo * point.0) + (hi * point_swapped)) + self.transform)
    }
}
