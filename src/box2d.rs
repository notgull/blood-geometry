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

use crate::pair::{Double, Quad};
use crate::path::{Path, PathEvent, Shape};
use crate::{Point, Size, Vector};
use num_traits::real::Real;
use num_traits::{Bounded, One, Zero};

use core::borrow::Borrow;
use core::fmt;
use core::ops::{self, Range};

/// A rectangular space consisting of its minimum and maximum points.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Box<T: Copy>(Quad<T>);

impl<T: fmt::Debug + Copy> fmt::Debug for Box<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Box")
            .field("min", &self.min())
            .field("max", &self.max())
            .finish()
    }
}

#[cfg(feature = "arbitrary")]
impl<'a, T: Copy + arbitrary::Arbitrary<'a>> arbitrary::Arbitrary<'a> for Box<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let value: [T; 4] = arbitrary::Arbitrary::arbitrary(u)?;
        Ok(Self(Quad::new(value)))
    }
}

/// The logical, serde representation of a box.
#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone)]
#[serde(rename = "Box")]
struct LogicalBox<T: Copy> {
    min: Point<T>,
    max: Point<T>,
}

#[cfg(feature = "serde")]
impl<T: Copy + serde::Serialize> serde::Serialize for Box<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let (min, max) = self.min_max();

        LogicalBox { min, max }.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Copy + serde::Deserialize<'de>> serde::Deserialize<'de> for Box<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let LogicalBox { min, max } = LogicalBox::deserialize(deserializer)?;

        Ok(Self::new(min, max))
    }
}

impl<T: Copy> Box<T> {
    /// Get the minimum point of the box.
    pub fn min(&self) -> Point<T> {
        Point(self.0.lo())
    }

    /// Get the maximum point of the box.
    pub fn max(&self) -> Point<T> {
        Point(self.0.hi())
    }

    /// Get the top right point of the box.
    pub fn bottom_right(&self) -> Point<T> {
        let [x, _, _, y] = self.0.into_inner();
        Point::new(x, y)
    }

    /// Get the bottom left point of the box.
    pub fn top_left(&self) -> Point<T> {
        let [_, y, x, _] = self.0.into_inner();
        Point::new(x, y)
    }

    /// Get the minimum and maximum points of the box.
    pub fn min_max(&self) -> (Point<T>, Point<T>) {
        let (min, max) = self.0.split();
        (Point(min), Point(max))
    }

    /// Create a new `Box` from the minimum and maximum points.
    pub fn new(min: Point<T>, max: Point<T>) -> Self {
        Box(Quad::from_double(min.0, max.0))
    }

    /// Get a `Box` with no bounds.
    pub fn unbounded() -> Self
    where
        T: Bounded,
    {
        Box::new(Point::splat(T::max_value()), Point::splat(T::min_value()))
    }

    /// `unbounded()` but uses the `Real` trait.
    pub fn unbounded_real() -> Self
    where
        T: Real,
    {
        Box::new(Point::splat(T::max_value()), Point::splat(T::min_value()))
    }

    /// Create a new `Box` from an origin point and its size.
    pub fn from_origin_and_size(origin: Point<T>, size: Size<T>) -> Self
    where
        T: ops::Add<Output = T>,
    {
        let max = origin + size;
        Self::new(origin, max)
    }

    /// Create a new `Box` at the origin from a size.
    pub fn from_size(size: Size<T>) -> Self
    where
        T: Zero,
    {
        Self::new(Point::zero(), Point(size.0))
    }

    /// Create an empty `Box` at the origin.
    pub fn zero() -> Self
    where
        T: Zero,
    {
        Self(Quad::splat(T::zero()))
    }
}

impl<T: Copy + PartialOrd> Box<T> {
    /// Tell whether or not this box has a negative area.
    pub fn is_negative(&self) -> bool {
        let min = self.min();
        let max = self.max();
        min > max
    }

    /// Tell whether or not this box has a zero area.
    pub fn is_empty(&self) -> bool {
        let min = self.min();
        let max = self.max();
        min >= max
    }

    /// Tell if this box contains a point.
    pub fn contains(&self, point: &Point<T>) -> bool
    where
        T: PartialOrd,
    {
        let point_repeated = Quad::from_double(point.0, point.0);
        let packed_lt = self.0.packed_lt(point_repeated);
        let (min_cmp, max_cmp) = packed_lt.split();

        // The point should be greater than or equal to the minimum point (i.e lt is false)
        // and less than the maximum point (i.e lt is true).
        !min_cmp.any() && max_cmp.all()
    }

    /// Tell if two boxes intersect.
    pub fn intersects(&self, other: &Self) -> bool
    where
        T: PartialOrd,
    {
        // To intersect, all of the mins have to be less than all of the maxes.
        let (self_min, self_max) = self.0.split();
        let (other_min, other_max) = other.0.split();
        let mins = Quad::from_double(self_min, other_min);
        let maxs = Quad::from_double(other_max, self_max);

        let packed_lt = mins.packed_lt(maxs);
        packed_lt.all()
    }

    /// Tell if we contain another box.
    pub fn contains_box(&self, other: &Self) -> bool
    where
        T: PartialOrd,
    {
        other.is_empty() || (self.contains(&other.min()) && self.contains(&other.max()))
    }

    /// Get the intersection of two boxes.
    pub fn intersection(&self, other: &Self) -> Self
    where
        T: PartialOrd + Copy,
    {
        let (self_min, self_max) = self.0.split();
        let (other_min, other_max) = other.0.split();

        Self(Quad::from_double(
            self_min.max(other_min),
            self_max.min(other_max),
        ))
    }

    /// Get the union of two boxes.
    pub fn union(&self, other: &Self) -> Self
    where
        T: PartialOrd + Copy,
    {
        let (self_min, self_max) = self.0.split();
        let (other_min, other_max) = other.0.split();

        Self(Quad::from_double(
            self_min.min(other_min),
            self_max.max(other_max),
        ))
    }

    /// Get a version of this box that also contains the given point.
    pub fn with_point(&self, point: &Point<T>) -> Self
    where
        T: PartialOrd + Copy,
    {
        let (self_min, self_max) = self.0.split();

        Self(Quad::from_double(
            self_min.min(point.0),
            self_max.max(point.0),
        ))
    }

    /// Create a box that contains all of the given points.
    pub fn with_points<I: IntoIterator>(&self, points: I) -> Self
    where
        I::Item: Borrow<Point<T>>,
        T: PartialOrd + Copy,
    {
        points
            .into_iter()
            .fold(*self, |acc, point| acc.with_point(point.borrow()))
    }

    /// Create a new box that contains all of the given points.
    pub fn of_points<I: IntoIterator>(points: I) -> Self
    where
        I::Item: Borrow<Point<T>>,
        T: PartialOrd + Copy + Zero,
    {
        let mut iter = points.into_iter();
        let first = match iter.next() {
            Some(first) => first,
            None => return Self::zero(),
        };

        let first = *first.borrow();
        iter.fold(Self::new(first, first), |acc, point| {
            acc.with_point(point.borrow())
        })
    }
}

impl<T: Copy> Box<T> {
    /// Linearly interpolate between two boxes.
    pub fn lerp(self, other: Self, t: T) -> Self
    where
        T: One + ops::Sub<Output = T> + ops::Mul<Output = T> + ops::Add<Output = T>,
    {
        let one_t = T::one() - t;

        // Take advantage of SIMD during operations.
        let box1 = self.0 * Quad::splat(one_t);
        let box2 = other.0 * Quad::splat(t);
        Self(box1 + box2)
    }

    /// Get the center of this box.
    pub fn center(&self) -> Point<T>
    where
        T: ops::Add<Output = T> + ops::Div<Output = T> + One + Copy,
    {
        let two = T::one() + T::one();
        let (min, max) = self.0.split();
        let center = (min + max) / Double::splat(two);
        Point(center)
    }

    /// Get the size of this box.
    pub fn size(&self) -> Size<T>
    where
        T: ops::Sub<Output = T>,
    {
        let (min, max) = self.0.split();
        let size = max - min;
        Size(size)
    }

    /// Get the area covered by this `Box`.
    pub fn area(&self) -> T
    where
        T: ops::Sub<Output = T> + ops::Mul<Output = T> + Copy,
    {
        let (min, max) = self.0.split();
        let size = max - min;
        let [x, y] = size.into_inner();
        x * y
    }

    /// Get the range of values covered by this `Box` in the X direction.
    pub fn x_range(&self) -> Range<T>
    where
        T: Copy,
    {
        let (min, max) = self.0.split();
        let [min_x, _] = min.into_inner();
        let [max_x, _] = max.into_inner();
        min_x..max_x
    }

    /// Get the range of values covered by this `Box` in the Y direction.
    pub fn y_range(&self) -> Range<T>
    where
        T: Copy,
    {
        let (min, max) = self.0.split();
        let [_, min_y] = min.into_inner();
        let [_, max_y] = max.into_inner();
        min_y..max_y
    }

    /// Round all of the box's values to the nearest integer.
    pub fn round(self) -> Self
    where
        T: Real,
    {
        Self(self.0.round())
    }

    /// Round all of the box's values to the nearest integer, such that the new box is
    /// containing the old box.
    pub fn round_out(self) -> Self
    where
        T: Real,
    {
        let (min, max) = self.0.split();
        Self(Quad::from_double(min.ceil(), max.floor()))
    }

    /// Round all of the box's values to the nearest integer, such that the new box is
    /// contained by the old box.
    pub fn round_in(self) -> Self
    where
        T: Real,
    {
        let (min, max) = self.0.split();
        Self(Quad::from_double(min.floor(), max.ceil()))
    }
}

impl<T: Copy + ops::Add<Output = T>> ops::Add<Vector<T>> for Box<T> {
    type Output = Self;

    fn add(self, rhs: Vector<T>) -> Self::Output {
        let translation = Quad::from_double(rhs.0, rhs.0);
        Self(self.0 + translation)
    }
}

impl<T: Copy + ops::AddAssign> ops::AddAssign<Vector<T>> for Box<T> {
    fn add_assign(&mut self, rhs: Vector<T>) {
        let translation = Quad::from_double(rhs.0, rhs.0);
        self.0 += translation;
    }
}

impl<T: Copy + ops::Sub<Output = T>> ops::Sub<Vector<T>> for Box<T> {
    type Output = Self;

    fn sub(self, rhs: Vector<T>) -> Self::Output {
        let translation = Quad::from_double(rhs.0, rhs.0);
        Self(self.0 - translation)
    }
}

impl<T: Copy + ops::SubAssign> ops::SubAssign<Vector<T>> for Box<T> {
    fn sub_assign(&mut self, rhs: Vector<T>) {
        let translation = Quad::from_double(rhs.0, rhs.0);
        self.0 -= translation;
    }
}

impl<T: Copy + Zero> From<Size<T>> for Box<T> {
    fn from(size: Size<T>) -> Self {
        Self::from_size(size)
    }
}

#[cfg(feature = "euclid")]
impl<T: Copy + Zero, U> From<euclid::Box2D<T, U>> for Box<T> {
    fn from(box_: euclid::Box2D<T, U>) -> Self {
        Self::new(box_.min.into(), box_.max.into())
    }
}

#[cfg(feature = "euclid")]
impl<T: Copy + Zero, U> From<Box<T>> for euclid::Box2D<T, U> {
    fn from(box_: Box<T>) -> Self {
        Self::new(box_.min().into(), box_.max().into())
    }
}

#[cfg(feature = "kurbo")]
impl From<kurbo::Rect> for Box<f64> {
    fn from(rect: kurbo::Rect) -> Self {
        let kurbo::Rect { x0, y0, x1, y1 } = rect;
        Self(Quad::new([x0, y0, x1, y1]))
    }
}

#[cfg(feature = "kurbo")]
impl From<Box<f64>> for kurbo::Rect {
    fn from(box_: Box<f64>) -> Self {
        let [x0, y0, x1, y1] = box_.0.into_inner();
        Self { x0, y0, x1, y1 }
    }
}

/// An object that has a bounding box.
pub trait BoundingBox<T: Copy> {
    /// Return the bounding box of the object.
    fn bounding_box(&self) -> Box<T>;
}

impl<T: Copy> BoundingBox<T> for Box<T> {
    fn bounding_box(&self) -> Box<T> {
        *self
    }
}

impl<T: Copy> Path<T> for Box<T> {
    type Iter = crate::iter::Five<PathEvent<T>>;

    fn path_iter(self) -> Self::Iter {
        crate::iter::Five::from([
            PathEvent::Begin { at: self.min() },
            PathEvent::Line {
                from: self.min(),
                to: self.bottom_right(),
            },
            PathEvent::Line {
                from: self.bottom_right(),
                to: self.max(),
            },
            PathEvent::Line {
                from: self.max(),
                to: self.top_left(),
            },
            PathEvent::End {
                last: self.top_left(),
                first: self.min(),
                close: true,
            },
        ])
    }

    fn approximate_length(self, _: T) -> T
    where
        Self: Sized,
        T: Real,
    {
        let size = self.size();
        let [width, height] = size.0.into_inner();
        width + height + width + height
    }

    fn rectilinear(self) -> bool
    where
        Self: Sized,
        T: crate::ApproxEq,
    {
        true
    }
}

impl<T: Copy> Shape<T> for Box<T> {
    #[cfg(feature = "alloc")]
    fn area(self, _: T) -> T
    where
        Self: Sized,
        T: Real + crate::ApproxEq,
    {
        let size = self.size();
        let [width, height] = size.0.into_inner();
        width * height
    }

    #[cfg(feature = "alloc")]
    fn perimeter(self, _: T) -> T
    where
        Self: Sized,
        T: Real,
    {
        let size = self.size();
        let [width, height] = size.0.into_inner();
        width + height + width + height
    }

    #[cfg(feature = "alloc")]
    fn bounding_box(self, _: T) -> Box<T> {
        self
    }
}
