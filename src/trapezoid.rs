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

//! A trapezoid with horizontal top and bottom edges.

use num_traits::real::Real;

use crate::box2d::{BoundingBox, Box};
use crate::line::{Line, LineSegment, NhLineSegment};
use crate::path::{Path, PathEvent, Shape};
use crate::ApproxEq;

/// A trapezoid with horizontal top and bottom edges.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Trapezoid<T: Copy> {
    /// The top edge of the trapezoid.
    top: T,

    /// The bottom edge of the trapezoid.
    bottom: T,

    /// The left edge of the trapezoid.
    left: Line<T>,

    /// The right edge of the trapezoid.
    right: Line<T>,
}

#[cfg(feature = "arbitrary")]
impl<'a, T: Copy + arbitrary::Arbitrary<'a>> arbitrary::Arbitrary<'a> for Trapezoid<T> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let top: T = u.arbitrary()?;
        let bottom: T = u.arbitrary()?;
        let left: Line<T> = u.arbitrary()?;
        let right: Line<T> = u.arbitrary()?;
        Ok(Trapezoid {
            top,
            bottom,
            left,
            right,
        })
    }
}

impl<T: Copy> Trapezoid<T> {
    /// Create a new trapezoid.
    pub fn new(top: T, bottom: T, left: Line<T>, right: Line<T>) -> Self {
        Trapezoid {
            top,
            bottom,
            left,
            right,
        }
    }

    /// Get the top edge of the trapezoid.
    pub fn top(&self) -> T {
        self.top
    }

    /// Get the bottom edge of the trapezoid.
    pub fn bottom(&self) -> T {
        self.bottom
    }

    /// Get the left edge of the trapezoid.
    pub fn left(&self) -> Line<T> {
        self.left
    }

    /// Get the left edge of the trapezoid as a line segment.
    pub fn left_segment(&self) -> LineSegment<T>
    where
        T: ApproxEq + Real,
    {
        NhLineSegment::from_line_and_y(self.left, self.top, self.bottom)
            .expect("horizontal line")
            .into()
    }

    /// Get the right edge of the trapezoid.
    pub fn right(&self) -> Line<T> {
        self.right
    }

    /// Get the right edge of the trapezoid as a line segment.
    pub fn right_segment(&self) -> LineSegment<T>
    where
        T: ApproxEq + Real,
    {
        NhLineSegment::from_line_and_y(self.right, self.top, self.bottom)
            .expect("horizontal line")
            .into()
    }

    /// Get the top side of the trapezoid as a line segment.
    pub fn top_segment(&self) -> LineSegment<T>
    where
        T: ApproxEq + Real,
    {
        let right_point = self.right.point_at_y(self.top).expect("horizontal line");
        let left_point = self.left.point_at_y(self.top).expect("horizontal line");
        LineSegment::new(left_point, right_point)
    }

    /// Get the bottom side of the trapezoid as a line segment.
    pub fn bottom_segment(&self) -> LineSegment<T>
    where
        T: ApproxEq + Real,
    {
        let right_point = self.right.point_at_y(self.bottom).expect("horizontal line");
        let left_point = self.left.point_at_y(self.bottom).expect("horizontal line");
        LineSegment::new(left_point, right_point)
    }

    /// Get the perimeter of the trapezoid.
    pub fn perimeter(&self) -> T
    where
        T: ApproxEq + Real,
    {
        self.top_segment().length()
            + self.bottom_segment().length()
            + self.left_segment().length()
            + self.right_segment().length()
    }

    /// Get the area of the trapezoid.
    pub fn area(&self) -> T
    where
        T: ApproxEq + Real,
    {
        let top = self.top_segment().length();
        let bottom = self.bottom_segment().length();
        let height = self.bottom - self.top;
        (top + bottom) * height / (T::one() + T::one())
    }
}

impl<T: Real + ApproxEq> Path<T> for Trapezoid<T> {
    type Iter = crate::iter::Five<PathEvent<T>>;

    fn path_iter(self) -> Self::Iter {
        let top = self.top_segment();
        let bottom = self.bottom_segment();

        let top_left = top.from();
        let top_right = top.to();
        let bottom_left = bottom.from();
        let bottom_right = bottom.to();

        crate::iter::Five::from([
            PathEvent::Begin { at: top_left },
            PathEvent::Line {
                from: top_left,
                to: top_right,
            },
            PathEvent::Line {
                from: top_right,
                to: bottom_right,
            },
            PathEvent::Line {
                from: bottom_right,
                to: bottom_left,
            },
            PathEvent::End {
                last: bottom_left,
                first: top_left,
                close: true,
            },
        ])
    }

    fn rectilinear(self) -> bool
    where
        Self: Sized,
        T: Copy + ApproxEq,
    {
        self.left.is_vertical() && self.right.is_vertical()
    }

    fn approximate_length(self, _accuracy: T) -> T
    where
        Self: Sized,
        T: Real + ApproxEq,
    {
        Trapezoid::perimeter(&self)
    }
}

impl<T: Real + ApproxEq> Shape<T> for Trapezoid<T> {
    #[cfg(feature = "alloc")]
    fn area(self, _accuracy: T) -> T
    where
        Self: Sized,
        T: Real + ApproxEq,
    {
        Trapezoid::area(&self)
    }

    fn perimeter(self, _accuracy: T) -> T
    where
        Self: Sized,
        T: Real + ApproxEq,
    {
        Trapezoid::perimeter(&self)
    }

    #[cfg(feature = "alloc")]
    fn bounding_box(self, _accuracy: T) -> Box<T>
    where
        Self: Sized,
        T: Real + ApproxEq,
    {
        BoundingBox::bounding_box(&self)
    }
}

impl<T: Real + ApproxEq> BoundingBox<T> for Trapezoid<T> {
    fn bounding_box(&self) -> Box<T> {
        // Get the points making up the trapezoid.
        let top_segment = self.top_segment();
        let bottom_segment = self.bottom_segment();

        let top_left = top_segment.from();
        let top_right = top_segment.to();
        let bottom_left = bottom_segment.from();
        let bottom_right = bottom_segment.to();

        Box::of_points(crate::iter::Four::from([
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        ]))
    }
}
