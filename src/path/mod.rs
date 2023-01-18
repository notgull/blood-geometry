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

use crate::curve::Curve;
use crate::{ApproxEq, Point};

use core::slice::Iter as SliceIter;
use num_traits::real::Real;

mod buffer;
pub use buffer::{PathBuffer, Verb};

mod flatten;
pub use flatten::Flattened;

mod line_segments;
pub use line_segments::LineSegments;

mod shape;
pub use shape::Shape;

/// An object that can be represented by a series of `PathEvent`s.
pub trait Path<T: Copy> {
    /// The type of the iterator returned by `path_iter`.
    type Iter: Iterator<Item = PathEvent<T>>;

    /// Get an iterator over the path events that make up this path.
    fn path_iter(self) -> Self::Iter;

    /// Determine if this path is rectilinear.
    fn rectilinear(self) -> bool
    where
        Self: Sized,
        T: ApproxEq,
    {
        self.path_iter().all(|event| match event {
            PathEvent::Begin { .. } | PathEvent::End { close: false, .. } => true,
            PathEvent::End {
                first: to,
                last: from,
                close: true,
            }
            | PathEvent::Line { from, to } => {
                from.x().approx_eq(&to.x()) || from.y().approx_eq(&to.y())
            }
            _ => false,
        })
    }

    /// Flatten the path into a series of straight line segments.
    fn flatten(self, tolerance: T) -> Flattened<T, Self::Iter>
    where
        Self: Sized,
        T: Real + ApproxEq,
    {
        Flattened::new(self.path_iter(), tolerance)
    }

    /// Get the flattened line segments of the path.
    fn segments(self, tolerance: T) -> LineSegments<T, Self::Iter>
    where
        Self: Sized,
        T: Real + ApproxEq,
    {
        LineSegments(self.flatten(tolerance))
    }

    /// Get the total length of this path.
    fn approximate_length(self, accuracy: T) -> T
    where
        Self: Sized,
        T: Real + ApproxEq,
    {
        self.path_iter().fold(T::zero(), |sum, event| match event {
            PathEvent::Begin { .. } | PathEvent::End { close: false, .. } => sum,
            PathEvent::Line { from, to }
            | PathEvent::End {
                first: to,
                last: from,
                close: true,
            } => sum + (to - from).length(),
            PathEvent::Quadratic { from, control, to } => {
                sum + crate::QuadraticBezier::new(from, control, to).length(accuracy)
            }
            PathEvent::Cubic {
                from,
                control1,
                control2,
                to,
            } => sum + crate::CubicBezier::new(from, control1, control2, to).length(accuracy),
            _ => unreachable!(),
        })
    }
}

/// A single event in a path.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PathEvent<T: Copy> {
    /// This path is the beginning of a new subpath.
    Begin {
        /// The starting point of the subpath.
        at: Point<T>,
    },

    /// This path forms a line from the previous point to the given point.
    Line {
        /// The starting point of the line.
        from: Point<T>,

        /// The end point of the line.
        to: Point<T>,
    },

    /// This path forms a quadratic Bezier curve from the previous point to the given
    /// point, using the provided control point.
    Quadratic {
        /// The starting point of the quadratic curve.
        from: Point<T>,

        /// The control point for the quadratic curve.
        control: Point<T>,

        /// The end point of the quadratic curve.
        to: Point<T>,
    },

    /// This path forms a cubic Bezier curve from the previous point to the given
    /// point, using the provided control points.
    Cubic {
        /// The starting point of the cubic curve.
        from: Point<T>,

        /// The first control point for the cubic curve.
        control1: Point<T>,

        /// The second control point for the cubic curve.
        control2: Point<T>,

        /// The end point of the cubic curve.
        to: Point<T>,
    },

    /// Close the current subpath.
    End {
        /// The starting point of the subpath.
        first: Point<T>,

        /// The end point of the subpath.
        last: Point<T>,

        /// Whether or not the subpath was closed.
        close: bool,
    },

    #[doc(hidden)]
    __NonExhaustive,
}

/// Events that can occur when a path consists only of straight lines.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StraightPathEvent<T: Copy> {
    /// This path is the beginning of a new subpath.
    Begin {
        /// The starting point of the subpath.
        at: Point<T>,
    },

    /// This path forms a line from the previous point to the given point.
    Line {
        /// The starting point of the line.
        from: Point<T>,

        /// The end point of the line.
        to: Point<T>,
    },

    /// Close the current subpath.
    End {
        /// The starting point of the subpath.
        first: Point<T>,

        /// The end point of the subpath.
        last: Point<T>,

        /// Whether or not the subpath was closed.
        close: bool,
    },

    #[doc(hidden)]
    __NonExhaustive,
}

impl<T: Copy> From<StraightPathEvent<T>> for PathEvent<T> {
    fn from(value: StraightPathEvent<T>) -> Self {
        match value {
            StraightPathEvent::Begin { at } => PathEvent::Begin { at },
            StraightPathEvent::Line { from, to } => PathEvent::Line { from, to },
            StraightPathEvent::End { first, last, close } => PathEvent::End { first, last, close },
            StraightPathEvent::__NonExhaustive => PathEvent::__NonExhaustive,
        }
    }
}

impl<'a, T: Copy, P> Path<T> for &'a [P]
where
    &'a P: Path<T>,
{
    type Iter = PathConnector<T, &'a P, SliceIter<'a, P>>;

    fn path_iter(self) -> Self::Iter {
        PathConnector {
            paths: self.iter(),
            current: None,
        }
    }
}

/// An iterator that connects many paths together.
pub struct PathConnector<T: Copy, P: Path<T>, I> {
    /// The iterator over the paths to connect.
    paths: I,

    /// The current path iterator we are iterating over.
    current: Option<P::Iter>,
}

impl<T: Copy, P: Path<T>, I: Iterator<Item = P>> Iterator for PathConnector<T, P, I> {
    type Item = PathEvent<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.current.as_mut() {
                Some(current) => match current.next() {
                    Some(event) => return Some(event),
                    None => self.current = None,
                },
                None => match self.paths.next() {
                    Some(path) => self.current = Some(path.path_iter()),
                    None => return None,
                },
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lo, hi) = self.paths.size_hint();

        match (lo, hi, self.current.as_ref()) {
            (0, Some(0), Some(current)) => current.size_hint(),
            (_, _, Some(current)) => {
                let (lo, _) = current.size_hint();
                (lo, None)
            }
            _ => (0, None),
        }
    }
}
