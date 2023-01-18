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

use crate::{Point, Vector};
use crate::pair::{Double, Quad};
use num_traits::real::Real;
use num_traits::{One, Signed, Zero};

use core::cmp;
use core::fmt;
use core::hash::{self, Hash};
use core::ops;

/// A two-dimensional size describing the width and height of something.
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct Size<T: Copy>(pub(crate) Double<T>);

impl<T: fmt::Debug + Copy> fmt::Debug for Size<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Size")
            .field(&self.width())
            .field(&self.height())
            .finish()
    }
}

impl<T: Copy> Size<T> {
    /// Create a new size with the given width and height.
    pub fn new(width: T, height: T) -> Self {
        Size(Double::new([width, height]))
    }

    /// Create a new size with two of the same value.
    pub fn splat(value: T) -> Self {
        Size(Double::splat(value))
    }

    /// Get the width of the size.
    pub fn width(&self) -> T {
        self.0[0]
    }

    /// Get the height of the size.
    pub fn height(&self) -> T {
        self.0[1]
    }

    /// Create a new `Size` from an array.
    pub fn from_array(array: [T; 2]) -> Self {
        Size(Double::new(array))
    }

    /// Create a new `Size` from a tuple.
    pub fn from_tuple((width, height): (T, T)) -> Self {
        Size(Double::new([width, height]))
    }
}

impl<T: Copy + Zero> Size<T> {
    /// Create a new size with zero width and height.
    pub fn zero() -> Self {
        Size(Double::splat(T::zero()))
    }
}

impl<T: PartialEq + Copy> PartialEq for Size<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Eq + Copy> Eq for Size<T> {}

impl<T: PartialOrd + Copy> PartialOrd for Size<T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T: Ord + Copy> Ord for Size<T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T: Hash + Copy> Hash for Size<T> {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T: Default + Copy> Default for Size<T> {
    fn default() -> Self {
        Size(Double::default())
    }
}

#[cfg(feature = "arbitrary")]
impl<'a, T: arbitrary::Arbitrary<'a> + Copy> arbitrary::Arbitrary<'a> for Size<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let (width, height) = arbitrary::Arbitrary::arbitrary(u)?;
        Ok(Size::new(width, height))
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize + Copy> serde::Serialize for Size<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        (&self.width(), &self.height()).serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de> + Copy> serde::Deserialize<'de> for Size<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let (width, height) = serde::Deserialize::deserialize(deserializer)?;
        Ok(Size::new(width, height))
    }
}

impl<T: Copy> From<[T; 2]> for Size<T> {
    fn from(array: [T; 2]) -> Self {
        Size::from_array(array)
    }
}

impl<T: Copy> From<(T, T)> for Size<T> {
    fn from(tuple: (T, T)) -> Self {
        Size::from_tuple(tuple)
    }
}

impl<T: Copy> From<Size<T>> for [T; 2] {
    fn from(size: Size<T>) -> Self {
        size.0.into_inner()
    }
}

impl<T: Copy> From<Size<T>> for (T, T) {
    fn from(size: Size<T>) -> Self {
        let [width, height] = size.0.into_inner();
        (width, height)
    }
}

#[cfg(feature = "euclid")]
impl<T: Copy, U> From<euclid::Size2D<T, U>> for Size<T> {
    fn from(size: euclid::Size2D<T, U>) -> Self {
        Size::new(size.width, size.height)
    }
}

#[cfg(feature = "euclid")]
impl<T: Copy, U> From<Size<T>> for euclid::Size2D<T, U> {
    fn from(size: Size<T>) -> Self {
        euclid::Size2D::from(size.0.into_inner())
    }
}

#[cfg(feature = "kurbo")]
impl From<kurbo::Size> for Size<f64> {
    fn from(size: kurbo::Size) -> Self {
        Size::new(size.width, size.height)
    }
}

#[cfg(feature = "kurbo")]
impl From<Size<f64>> for kurbo::Size {
    fn from(size: Size<f64>) -> Self {
        kurbo::Size::new(size.width(), size.height())
    }
}

impl<T: Copy + ops::Add<Output = T>> ops::Add for Size<T> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Size(self.0 + other.0)
    }
}

impl<T: Copy + ops::Add<Output = T>> ops::Add<Size<T>> for Point<T> {
    type Output = Point<T>;

    fn add(self, other: Size<T>) -> Point<T> {
        Point(self.0 + other.0)
    }
}

impl<T: Copy + ops::AddAssign> ops::AddAssign for Size<T> {
    fn add_assign(&mut self, other: Self) {
        self.0 += other.0;
    }
}

impl<T: Copy + ops::Sub<Output = T>> ops::Sub for Size<T> {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Size(self.0 - other.0)
    }
}

impl<T: Copy + ops::Sub<Output = T>> ops::Sub<Size<T>> for Point<T> {
    type Output = Point<T>;

    fn sub(self, other: Size<T>) -> Point<T> {
        Point(self.0 - other.0)
    }
}

impl<T: Copy + ops::SubAssign> ops::SubAssign for Size<T> {
    fn sub_assign(&mut self, other: Self) {
        self.0 -= other.0;
    }
}

impl<T: Copy + ops::MulAssign> ops::MulAssign<T> for Size<T> {
    fn mul_assign(&mut self, other: T) {
        self.0 *= Double::splat(other);
    }
}

impl<T: Copy + ops::Div<Output = T>> ops::Div<T> for Size<T> {
    type Output = Self;

    fn div(self, other: T) -> Self {
        Size(self.0 / Double::splat(other))
    }
}

impl<T: Copy + ops::Div<Output = T>> ops::Div<Vector<T>> for Size<T> {
    type Output = Size<T>;

    fn div(self, other: Vector<T>) -> Size<T> {
        Size(self.0 / other.0)
    }
}

impl<T: Copy + ops::DivAssign> ops::DivAssign<T> for Size<T> {
    fn div_assign(&mut self, other: T) {
        self.0 /= Double::splat(other);
    }
}

impl<T: Copy + ops::Neg<Output = T>> ops::Neg for Size<T> {
    type Output = Self;

    fn neg(self) -> Self {
        Size(-self.0)
    }
}

impl<T: Copy> Size<T> {
    /// Get the absolute value of this size.
    pub fn abs(self) -> Self
    where
        T: Signed,
    {
        Size(self.0.abs())
    }

    /// Get the minimum values for both sizes.
    pub fn min(self, other: Self) -> Self
    where
        T: PartialOrd,
    {
        Size(self.0.min(other.0))
    }

    /// Get the maximum values for both sizes.
    pub fn max(self, other: Self) -> Self
    where
        T: PartialOrd,
    {
        Size(self.0.max(other.0))
    }

    /// Clamp the coordinates to `[min, max]`.
    pub fn clamp(self, min: Self, max: Self) -> Self
    where
        T: PartialOrd,
    {
        Size(self.0.clamp(min.0, max.0))
    }

    /// Linearly interpolate between two sets of coordinates.
    #[inline]
    pub fn lerp(self, other: Self, t: T) -> Self
    where
        T: One + ops::Sub<Output = T> + ops::Mul<Output = T> + ops::Add<Output = T>,
    {
        let one_t = T::one() - t;

        // Combine them into a quad for a multiplication, then add them together.
        let points = Quad::from_double(self.0, other.0);
        let multiplier = Quad::new([one_t, one_t, t, t]);
        let result = points * multiplier;

        let (point1, point2) = result.split();
        Self(point1 + point2)
    }

    /// Round the coordinates to the nearest integer.
    #[inline]
    pub fn round(self) -> Self
    where
        T: Real,
    {
        Self(self.0.round())
    }

    /// Round the coordinates down.
    #[inline]
    pub fn floor(self) -> Self
    where
        T: Real,
    {
        Self(self.0.floor())
    }

    /// Round the coordinates up.
    #[inline]
    pub fn ceil(self) -> Self
    where
        T: Real,
    {
        Self(self.0.ceil())
    }

    /// Get the total area of the size.
    #[inline]
    pub fn area(self) -> T
    where
        T: ops::Mul<Output = T>,
    {
        let [width, height] = self.0.into_inner();
        width * height
    }
}
