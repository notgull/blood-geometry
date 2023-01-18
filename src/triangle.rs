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

use crate::path::{Path, PathEvent, Shape};
use crate::{ApproxEq, Box, LineSegment, Point};
use num_traits::real::Real;
use num_traits::Zero;

use core::fmt;

/// A triangle.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Triangle<T: Copy>([Point<T>; 3]);

/// A triangle where two of its points are on the same horizontal line.
///
/// This is useful for computing the area or bounding box of a triangle.
struct HalfTriangle<T: Copy> {
    y: T,
    x1: T,
    x2: T,
    free: Point<T>,
}

impl<T: fmt::Debug + Copy> fmt::Debug for Triangle<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Triangle")
            .field("a", &self.0[0])
            .field("b", &self.0[1])
            .field("c", &self.0[2])
            .finish()
    }
}

#[cfg(feature = "arbitrary")]
impl<'a, T: Copy + arbitrary::Arbitrary<'a>> arbitrary::Arbitrary<'a> for Triangle<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let (a, b, c): (Point<T>, Point<T>, Point<T>) = arbitrary::Arbitrary::arbitrary(u)?;
        Ok(Triangle([a, b, c]))
    }
}

/// The logical representation of a triangle for serde.
#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone)]
#[serde(rename = "Triangle")]
struct LogicalTriangle<T: Copy> {
    a: Point<T>,
    b: Point<T>,
    c: Point<T>,
}

#[cfg(feature = "serde")]
impl<T: Copy + serde::Serialize> serde::Serialize for Triangle<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        LogicalTriangle {
            a: self.a(),
            b: self.b(),
            c: self.c(),
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Copy + serde::Deserialize<'de>> serde::Deserialize<'de> for Triangle<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let LogicalTriangle { a, b, c } = serde::Deserialize::deserialize(deserializer)?;
        Ok(Triangle([a, b, c]))
    }
}

impl<T: Copy> Triangle<T> {
    /// Create a new triangle.
    pub fn new(a: Point<T>, b: Point<T>, c: Point<T>) -> Self {
        Triangle([a, b, c])
    }

    /// Get the first point of the triangle.
    pub fn a(&self) -> Point<T> {
        self.0[0]
    }

    /// Get the second point of the triangle.
    pub fn b(&self) -> Point<T> {
        self.0[1]
    }

    /// Get the third point of the triangle.
    pub fn c(&self) -> Point<T> {
        self.0[2]
    }

    /// Get the line segment from `a` to `b`.
    pub fn ab(&self) -> LineSegment<T> {
        LineSegment::new(self.a(), self.b())
    }

    /// Get the line segment from `b` to `c`.
    pub fn bc(&self) -> LineSegment<T> {
        LineSegment::new(self.b(), self.c())
    }

    /// Get the line segment from `c` to `a`.
    pub fn ca(&self) -> LineSegment<T> {
        LineSegment::new(self.c(), self.a())
    }

    /// Get the points of the triangle as an array.
    pub fn into_array(self) -> [Point<T>; 3] {
        self.0
    }

    /// Get the line segments of the triangle as an array.
    pub fn into_segments(self) -> [LineSegment<T>; 3] {
        [self.ab(), self.bc(), self.ca()]
    }

    /// Get the points of the triangle as a tuple.
    pub fn into_tuple(self) -> (Point<T>, Point<T>, Point<T>) {
        (self.0[0], self.0[1], self.0[2])
    }

    /// Create a zeroed triangle.
    pub fn zero() -> Self
    where
        T: Zero,
    {
        Triangle([Point::zero(), Point::zero(), Point::zero()])
    }

    /// Break this triangle into one or more half-triangles.
    fn half_triangles(self) -> crate::iter::Two<HalfTriangle<T>>
    where
        T: Real + ApproxEq,
    {
        let [a, b, c] = self.0;

        let single_half = if a.y().approx_eq(&b.y()) {
            Some(HalfTriangle {
                y: a.y(),
                x1: a.x(),
                x2: b.x(),
                free: c,
            })
        } else if a.y().approx_eq(&c.y()) {
            Some(HalfTriangle {
                y: a.y(),
                x1: a.x(),
                x2: c.x(),
                free: b,
            })
        } else if b.y().approx_eq(&c.y()) {
            Some(HalfTriangle {
                y: b.y(),
                x1: b.x(),
                x2: c.x(),
                free: a,
            })
        } else {
            None
        };

        if let Some(single_half) = single_half {
            return crate::iter::Two::from([single_half]);
        }

        // Sort points by Y coordinate.
        let mut points = [a, b, c];
        points.sort_by(|a, b| a.y().partial_cmp(&b.y()).unwrap());

        // Find the X coordinate where we split the triangle.
        let ac = LineSegment::new(a, c);
        let split_x = ac.line().point_at_y(points[1].y()).unwrap().x();

        let half1 = HalfTriangle {
            y: points[1].y(),
            x1: points[1].x(),
            x2: split_x,
            free: points[0],
        };
        let half2 = HalfTriangle {
            y: points[1].y(),
            x1: split_x,
            x2: points[1].x(),
            free: points[2],
        };

        crate::iter::Two::from([half1, half2])
    }
}

impl<T: PartialOrd + Copy> crate::BoundingBox<T> for Triangle<T> {
    /// Get the bounding box of the triangle.
    fn bounding_box(&self) -> Box<T> {
        let min = self.a().min(self.b()).min(self.c());
        let max = self.a().max(self.b()).max(self.c());

        Box::new(min, max)
    }
}

impl<T: Copy> Path<T> for Triangle<T> {
    type Iter = crate::iter::Four<PathEvent<T>>;

    fn path_iter(self) -> Self::Iter {
        let [a, b, c] = self.0;
        crate::iter::Four::from([
            PathEvent::Begin { at: a },
            PathEvent::Line { from: a, to: b },
            PathEvent::Line { from: b, to: c },
            PathEvent::End {
                last: c,
                first: a,
                close: true,
            },
        ])
    }

    fn rectilinear(self) -> bool {
        false
    }

    fn approximate_length(self, _accuracy: T) -> T
    where
        T: Real + ApproxEq,
    {
        crate::iter::Three::from(self.into_segments())
            .fold(T::zero(), |acc, segment| acc + segment.length())
    }
}

impl<T: Copy> Shape<T> for Triangle<T> {
    fn area(self, _accuracy: T) -> T
    where
        T: Real + ApproxEq,
    {
        self.half_triangles()
            .fold(T::zero(), |acc, half| acc + half.area())
    }

    fn bounding_box(self, _accuracy: T) -> Box<T>
    where
        T: Real,
    {
        crate::BoundingBox::bounding_box(&self)
    }
}

impl<T: Copy> HalfTriangle<T> {
    fn area(&self) -> T
    where
        T: Real,
    {
        let b = (self.x2 - self.x1).abs();
        let h = (self.free.y() - self.y).abs();
        b * h / (T::one() + T::one())
    }
}
