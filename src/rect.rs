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

use crate::pair::Quad;
use crate::path::{Path, PathEvent, Shape};
use crate::{Box, Point, Size, Vector};
use num_traits::Zero;

use core::fmt;
use core::ops;

/// A two-dimensional rectangle consisting of a point and its size.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Rect<T: Copy>(pub(crate) Quad<T>);

impl<T: fmt::Debug + Copy> fmt::Debug for Rect<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Rect")
            .field("origin", &self.origin())
            .field("size", &self.size())
            .finish()
    }
}

#[cfg(feature = "arbitrary")]
impl<'a, T: Copy + arbitrary::Arbitrary<'a>> arbitrary::Arbitrary<'a> for Rect<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let value: [T; 4] = arbitrary::Arbitrary::arbitrary(u)?;
        Ok(Rect(Quad::new(value)))
    }
}

/// The logical representation of a rectangle for serde.
#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone)]
#[serde(rename = "Rect")]
struct LogicalRect<T: Copy> {
    origin: Point<T>,
    size: Size<T>,
}

#[cfg(feature = "serde")]
impl<T: Copy + serde::Serialize> serde::Serialize for Rect<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        LogicalRect {
            origin: self.origin(),
            size: self.size(),
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Copy + serde::Deserialize<'de>> serde::Deserialize<'de> for Rect<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let LogicalRect { origin, size } = serde::Deserialize::deserialize(deserializer)?;
        Ok(Rect::new(origin, size))
    }
}

impl<T: Copy> Rect<T> {
    /// Get the origin of the rectangle.
    #[inline]
    pub fn origin(self) -> Point<T> {
        Point(self.0.lo())
    }

    /// Get the size of the rectangle.
    #[inline]
    pub fn size(self) -> Size<T> {
        Size(self.0.hi())
    }

    /// Get the origin and size of the rectangle.
    #[inline]
    pub fn origin_and_size(self) -> (Point<T>, Size<T>) {
        (self.origin(), self.size())
    }

    /// Get the top left corner of the rectangle.
    #[inline]
    pub fn top_left(self) -> Point<T> {
        self.origin()
    }

    /// Get the top right corner of the rectangle.
    #[inline]
    pub fn top_right(self) -> Point<T>
    where
        T: ops::Add<Output = T> + Zero,
    {
        self.origin() + Vector::new(self.size().width(), T::zero())
    }

    /// Get the bottom left corner of the rectangle.
    #[inline]
    pub fn bottom_left(self) -> Point<T>
    where
        T: ops::Add<Output = T> + Zero,
    {
        self.origin() + Vector::new(T::zero(), self.size().height())
    }

    /// Get the bottom right corner of the rectangle.
    #[inline]
    pub fn bottom_right(self) -> Point<T>
    where
        T: ops::Add<Output = T> + Zero,
    {
        self.origin() + self.size()
    }

    /// Create a new rectangle from an origin and a size.
    #[inline]
    pub fn new(origin: Point<T>, size: Size<T>) -> Self {
        Rect(Quad::from_double(origin.0, size.0))
    }

    /// Create a new rectangle from the top left corner and the bottom right corner.
    #[inline]
    pub fn from_corners(top_left: Point<T>, bottom_right: Point<T>) -> Self
    where
        T: ops::Sub<Output = T>,
    {
        Rect::new(top_left, Size(bottom_right.0 - top_left.0))
    }

    /// Create a new rectangle from a `Box`.
    #[inline]
    pub fn from_box(box_: Box<T>) -> Self
    where
        T: ops::Sub<Output = T>,
    {
        Rect::new(box_.min(), box_.size())
    }

    /// Create a new rectangle with zero coordinates.
    #[inline]
    pub fn zero() -> Self
    where
        T: Zero,
    {
        Rect::new(Point::zero(), Size::zero())
    }

    /// Convert the rectangle to a `Box`.
    #[inline]
    pub fn to_box(self) -> Box<T>
    where
        T: ops::Add<Output = T> + Zero,
    {
        Box::new(self.origin(), self.bottom_right())
    }

    /// Get the area of the rectangle.
    #[inline]
    pub fn area(self) -> T
    where
        T: ops::Mul<Output = T> + ops::Add<Output = T>,
    {
        let [_, _, w, h] = self.0.into_inner();
        w * h
    }
}

impl<T: Copy> From<Rect<T>> for Box<T>
where
    T: ops::Add<Output = T> + Zero,
{
    #[inline]
    fn from(rect: Rect<T>) -> Self {
        rect.to_box()
    }
}

impl<T: Copy> From<Box<T>> for Rect<T>
where
    T: ops::Sub<Output = T>,
{
    #[inline]
    fn from(box_: Box<T>) -> Self {
        Rect::from_box(box_)
    }
}

impl<T: Copy + ops::Div<Output = T>> ops::Div<Vector<T>> for Rect<T> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Vector<T>) -> Self::Output {
        Rect::new(self.origin() / rhs, self.size() / rhs)
    }
}

impl<T: Copy + ops::Div<Output = T>> ops::Div<T> for Rect<T> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: T) -> Self::Output {
        Rect::new(self.origin() / rhs, self.size() / rhs)
    }
}

impl<T: Copy + ops::Div<Output = T>> ops::DivAssign<Vector<T>> for Rect<T> {
    #[inline]
    fn div_assign(&mut self, rhs: Vector<T>) {
        *self = *self / rhs;
    }
}

impl<T: Copy + ops::Div<Output = T>> ops::DivAssign<T> for Rect<T> {
    #[inline]
    fn div_assign(&mut self, rhs: T) {
        *self = *self / rhs;
    }
}

#[cfg(feature = "euclid")]
impl<T: Copy, U> From<euclid::Rect<T, U>> for Rect<T> {
    #[inline]
    fn from(rect: euclid::Rect<T, U>) -> Self {
        Rect::new(Point::from(rect.origin), Size::from(rect.size))
    }
}

#[cfg(feature = "euclid")]
impl<T: Copy, U> From<Rect<T>> for euclid::Rect<T, U> {
    #[inline]
    fn from(rect: Rect<T>) -> Self {
        euclid::Rect::new(
            euclid::Point2D::from(rect.origin()),
            euclid::Size2D::from(rect.size()),
        )
    }
}

impl<T: ops::Add<Output = T> + Zero + Copy> crate::BoundingBox<T> for Rect<T> {
    fn bounding_box(&self) -> Box<T> {
        self.to_box()
    }
}

impl<T: Copy + ops::Add<Output = T> + Zero> Path<T> for Rect<T> {
    type Iter = crate::iter::Five<PathEvent<T>>;

    fn path_iter(self) -> Self::Iter {
        self.to_box().path_iter()
    }

    fn rectilinear(self) -> bool {
        true
    }

    fn approximate_length(self, _accuracy: T) -> T {
        let size = self.size();
        let [w, h] = size.0.into_inner();

        w + w + h + h
    }
}

impl<T: Copy + ops::Add<Output = T> + ops::Mul<Output = T> + Zero> Shape<T> for Rect<T> {
    #[cfg(feature = "alloc")]
    fn area(self, _accuracy: T) -> T {
        Rect::area(self)
    }

    fn perimeter(self, _accuracy: T) -> T {
        let size = self.size();
        let [w, h] = size.0.into_inner();

        w + w + h + h
    }
}
