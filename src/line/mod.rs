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

use crate::iter::Three;
use crate::path::{Path, PathEvent};
use crate::pair::Quad;
use crate::{ApproxEq, Point, Vector};
use num_traits::{real::Real, Signed, Zero};

use core::convert::TryFrom;
use core::fmt;
use core::ops;

mod from_points;

/// An infinite line.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct Line<T: Copy>(
    // Logically, this is the origin line first, then the direction vector.
    Quad<T>,
);

impl<T: fmt::Debug + Copy> fmt::Debug for Line<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Line")
            .field("origin", &self.origin())
            .field("direction", &self.direction())
            .finish()
    }
}

#[cfg(feature = "arbitrary")]
impl<'a, T: Copy + arbitrary::Arbitrary<'a>> arbitrary::Arbitrary<'a> for Line<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let (origin, direction): (Point<T>, Vector<T>) = arbitrary::Arbitrary::arbitrary(u)?;
        Ok(Line::new(origin, direction))
    }
}

/// The logical representation of a line for serde.
#[cfg(feature = "serde")]
#[derive(serde::Serialize, serde::Deserialize, Copy, Clone)]
#[serde(rename = "Line")]
struct LogicalLine<T: Copy> {
    origin: Point<T>,
    direction: Vector<T>,
}

#[cfg(feature = "serde")]
impl<T: Copy + serde::Serialize> serde::Serialize for Line<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        LogicalLine {
            origin: self.origin(),
            direction: self.direction(),
        }
        .serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, T: Copy + serde::Deserialize<'de>> serde::Deserialize<'de> for Line<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let LogicalLine { origin, direction } = serde::Deserialize::deserialize(deserializer)?;
        Ok(Line::new(origin, direction))
    }
}

impl<T: Copy> Line<T> {
    /// Create a new line from an origin point and a direction vector.
    #[inline]
    pub fn new(origin: Point<T>, direction: Vector<T>) -> Self {
        Line(Quad::from_double(origin.0, direction.0))
    }

    /// Get the origin point of the line.
    #[inline]
    pub fn origin(&self) -> Point<T> {
        Point(self.0.lo())
    }

    /// Get the direction vector of the line.
    #[inline]
    pub fn direction(&self) -> Vector<T> {
        Vector(self.0.hi())
    }

    /// Get the line between two points.
    #[inline]
    pub fn between(a: Point<T>, b: Point<T>) -> Self
    where
        T: ops::Sub<Output = T>,
    {
        Line::new(a, b - a)
    }

    /// Tell whether or not this line intersects with another line.
    #[inline]
    pub fn intersects(&self, other: &Self) -> bool
    where
        T: ApproxEq + Signed,
    {
        !self
            .direction()
            .cross(other.direction())
            .abs()
            .approx_eq(&T::zero())
    }

    /// Get the intersection point of two lines.
    ///
    /// Returns `None` if the lines are parallel.
    #[inline]
    pub fn intersection(&self, line: &Self) -> Option<Point<T>>
    where
        T: Real,
    {
        // Taken from: https://docs.rs/lyon_geom/latest/src/lyon_geom/line.rs.html#550-566
        // Get the inverse determinant of our vectors.
        let det = self.direction().cross(line.direction());

        // If the determinant is zero, lines are probably parallel.
        if det <= T::epsilon() {
            return None;
        }

        let self_p2 = self.origin() + self.direction();
        let other_p2 = line.origin() + line.direction();

        let a = self.origin().into_vector().cross(self_p2.into_vector());
        let b = line.origin().into_vector().cross(other_p2.into_vector());

        Some(Point::new(
            (a * line.direction().x() - b * self.direction().x()) / det,
            (a * line.direction().y() - b * self.direction().y()) / det,
        ))
    }

    /// Get the distance from this line to another point.
    #[inline]
    pub fn distance(&self, point: Point<T>) -> T
    where
        T: Real,
    {
        let p = point - self.origin();
        let d = self.direction();
        let cross = p.cross(d);
        (cross / d.length()).abs()
    }

    /// Get the distance squared to another point.
    #[inline]
    pub fn distance_squared(&self, point: Point<T>) -> T
    where
        T: ops::Add<Output = T>
            + ops::Sub<Output = T>
            + ops::Mul<Output = T>
            + ops::Div<Output = T>,
    {
        let p = point - self.origin();
        let d = self.direction();
        let cross = p.cross(d);
        (cross * cross) / (d.length_squared())
    }

    /// Create a parallel line to this one, but with a different origin.
    #[inline]
    pub fn parallel(&self, origin: Point<T>) -> Self {
        Line::new(origin, self.direction())
    }

    /// Get the point on this line at the given X coordinate.
    ///
    /// Returns `None` if the line is horizontal.
    #[inline]
    pub fn point_at_x(&self, x: T) -> Option<Point<T>>
    where
        T: Real + ApproxEq,
    {
        if self.direction().x().approx_eq(&T::zero()) {
            return None;
        }

        let y = (x - self.origin().x()) * self.direction().y() / self.direction().x()
            + self.origin().y();
        Some(Point::new(x, y))
    }

    /// Get this point on this line at the given Y coordinate.
    ///
    /// Returns `None` if the line is vertical.
    #[inline]
    pub fn point_at_y(&self, y: T) -> Option<Point<T>>
    where
        T: Real + ApproxEq,
    {
        if self.direction().y().approx_eq(&T::zero()) {
            return None;
        }

        let x = (y - self.origin().y()) * self.direction().x() / self.direction().y()
            + self.origin().x();
        Some(Point::new(x, y))
    }

    /// Is this line horizontal?
    #[inline]
    pub fn is_horizontal(&self) -> bool
    where
        T: ApproxEq + Zero,
    {
        self.direction().y().approx_eq(&T::zero())
    }

    /// Is this line vertical?
    #[inline]
    pub fn is_vertical(&self) -> bool
    where
        T: ApproxEq + Zero,
    {
        self.direction().x().approx_eq(&T::zero())
    }
}

#[cfg(feature = "lyon_geom")]
impl<T: Copy> From<lyon_geom::Line<T>> for Line<T> {
    #[inline]
    fn from(line: lyon_geom::Line<T>) -> Self {
        Line::new(line.point.into(), line.vector.into())
    }
}

#[cfg(feature = "lyon_geom")]
impl<T: Copy> From<Line<T>> for lyon_geom::Line<T> {
    #[inline]
    fn from(line: Line<T>) -> Self {
        lyon_geom::Line {
            point: line.origin().into(),
            vector: line.direction().into(),
        }
    }
}

/// A line segment, bounded by two points.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LineSegment<T: Copy> {
    /// The point where this segment begins.
    from: Point<T>,

    /// The point where this segment ends.
    to: Point<T>,
}

/// A line segment that is not a horizontal line.
///
/// This structure assumes that the line segment is not horizontal. In exchange, its structure
/// is marginally more applicable to certain use cases.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NhLineSegment<T: Copy> {
    /// The line that this line segment is a part of.
    line: Line<T>,

    // We bound the line using the Y coordinates of the points.
    top: T,
    bottom: T,
}

/// An error indicating that a line segment is horizontal.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HorizontalLineSegmentError<T: Copy>(LineSegment<T>);

#[cfg(feature = "arbitrary")]
impl<
        'a,
        T: PartialOrd + Copy + ApproxEq + Zero + ops::Sub<Output = T> + arbitrary::Arbitrary<'a>,
    > arbitrary::Arbitrary<'a> for LineSegment<T>
{
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let a = Point::arbitrary(u)?;
        let b = Point::arbitrary(u)?;
        Ok(LineSegment::new(a, b))
    }
}

impl<T: Copy> LineSegment<T> {
    /// Create a new line segment from two points.
    #[inline]
    pub fn new(p1: Point<T>, p2: Point<T>) -> Self {
        Self { from: p1, to: p2 }
    }

    /// Get the two points that make up this line segment.
    #[inline]
    pub fn points(&self) -> (Point<T>, Point<T>) {
        (self.from, self.to)
    }

    /// Get the line that this line segment is a part of.
    #[inline]
    pub fn line(&self) -> Line<T>
    where
        T: ops::Sub<Output = T>,
    {
        Line::new(self.from, self.to - self.from)
    }

    /// Get the point where this line segment begins.
    #[inline]
    pub fn from(&self) -> Point<T> {
        self.from
    }

    /// Get the point where this line segment ends.
    #[inline]
    pub fn to(&self) -> Point<T> {
        self.to
    }

    /// Get the length of this line segment.
    #[inline]
    pub fn length(&self) -> T
    where
        T: Real,
    {
        (self.to - self.from).length()
    }
}

impl<T: ApproxEq + Real> From<NhLineSegment<T>> for LineSegment<T> {
    #[inline]
    fn from(line: NhLineSegment<T>) -> Self {
        let (from, to) = line.points();
        Self { from, to }
    }
}

impl<T: Copy + PartialOrd + ops::Sub<Output = T> + ApproxEq + Zero> TryFrom<LineSegment<T>>
    for NhLineSegment<T>
{
    type Error = HorizontalLineSegmentError<T>;

    fn try_from(value: LineSegment<T>) -> Result<Self, Self::Error> {
        NhLineSegment::new(value.from, value.to).ok_or(HorizontalLineSegmentError(value))
    }
}

impl<T: PartialOrd + Copy> NhLineSegment<T> {
    /// Try to create a new line segment from two points.
    ///
    /// Returns `None` if the line segment is horizontal.
    #[inline]
    pub fn new(p1: Point<T>, p2: Point<T>) -> Option<Self>
    where
        T: ops::Sub<Output = T> + ApproxEq + Zero,
    {
        let line = Line::between(p1, p2);

        if line.is_horizontal() {
            None
        } else {
            let (top, bottom) = order(p1.y(), p2.y());
            Some(NhLineSegment { line, top, bottom })
        }
    }

    /// Create a new line segment from a line and two Y coordinates.
    ///
    /// Returns `None` if the line segment is horizontal.
    #[inline]
    pub fn from_line_and_y(line: Line<T>, top: T, bottom: T) -> Option<Self>
    where
        T: ops::Sub<Output = T> + ApproxEq + Zero,
    {
        if line.is_horizontal() {
            None
        } else {
            Some(NhLineSegment { line, top, bottom })
        }
    }

    /// Get the top Y coordinate of this line segment.
    #[inline]
    pub fn top(&self) -> T {
        self.top
    }

    /// Get the bottom Y coordinate of this line segment.
    #[inline]
    pub fn bottom(&self) -> T {
        self.bottom
    }

    /// Get the two points that make up this line segment.
    #[inline]
    pub fn points(&self) -> (Point<T>, Point<T>)
    where
        T: ApproxEq + Real,
    {
        let p1 = self.line.point_at_y(self.top).unwrap();
        let p2 = self.line.point_at_y(self.bottom).unwrap();
        (p1, p2)
    }

    /// Get the line that this line segment is a part of.
    #[inline]
    pub fn line(&self) -> Line<T> {
        self.line
    }

    /// Get the intersection between this line segment and another line segment.
    #[inline]
    pub fn intersection(&self, other: &NhLineSegment<T>) -> Option<Point<T>>
    where
        T: ApproxEq + Real,
    {
        self.line
            .intersection(&other.line)
            .and_then(|intersection| {
                if self.top >= intersection.y()
                    && self.bottom <= intersection.y()
                    && other.top >= intersection.y()
                    && other.bottom <= intersection.y()
                {
                    Some(intersection)
                } else {
                    None
                }
            })
    }
}

impl<T: Copy> Path<T> for LineSegment<T> {
    type Iter = Three<PathEvent<T>>;

    fn path_iter(self) -> Self::Iter {
        Three::from([
            PathEvent::Begin { at: self.from },
            PathEvent::Line {
                from: self.from,
                to: self.to,
            },
            PathEvent::End {
                last: self.to,
                first: self.from,
                close: false,
            },
        ])
    }

    fn rectilinear(self) -> bool
    where
        Self: Sized,
        T: ApproxEq,
    {
        self.from.x().approx_eq(&self.to.x()) || self.from.y().approx_eq(&self.to.y())
    }
}

impl<T: Real + ApproxEq> Path<T> for NhLineSegment<T> {
    type Iter = Three<PathEvent<T>>;

    fn path_iter(self) -> Self::Iter {
        let seg: LineSegment<_> = self.into();
        seg.path_iter()
    }
}

#[inline]
fn order<T: PartialOrd>(a: T, b: T) -> (T, T) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}
