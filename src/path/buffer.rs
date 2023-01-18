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

use super::{Path, PathEvent};
use crate::point::Point;

use core::borrow::Borrow;
use core::fmt;
use core::iter::FromIterator;
use core::mem;
use core::slice::Iter as SliceIter;

/// A verb associated with a path.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Verb<T: Copy> {
    /// This path is the beginning of a new subpath.
    Begin {
        /// Whether or not the previous subpath should be closed.
        close: bool,
    },

    /// This path forms a line from the previous point to the given point.
    Line,

    /// This line forms a quadratic Bezier curve from the previous point to the given
    /// point, using the provided control point.
    Quadratic {
        /// The control point for the quadratic curve.
        control: Point<T>,
    },

    /// This line forms a cubic Bezier curve from the previous point to the given
    /// point, using the provided control points.
    Cubic {
        /// The first control point for the cubic curve.
        control1: Point<T>,

        /// The second control point for the cubic curve.
        control2: Point<T>,
    },

    #[doc(hidden)]
    __NonExhaustive,
}

/// Type alias for a path's unsized buffer.
type UnsizedBuffer<T> = [(Point<T>, Verb<T>)];

/// A path is a series of connected lines and curves.
pub struct PathBuffer<T: Copy, Buf: ?Sized = UnsizedBuffer<T>> {
    /// The first point in the path.
    first: Point<T>,

    /// The remaining points in the path.
    buffer: Buf,
}

impl<T: Copy, Buf: Borrow<UnsizedBuffer<T>>> PathBuffer<T, Buf> {
    /// Create a new `Path` from the first point and the remaining actions.
    pub const fn new(first: Point<T>, buffer: Buf) -> Self {
        PathBuffer { first, buffer }
    }
}

impl<T: Copy + fmt::Debug, Buf: FromIterator<(Point<T>, Verb<T>)>> FromIterator<PathEvent<T>>
    for PathBuffer<T, Buf>
{
    fn from_iter<Iter: IntoIterator<Item = PathEvent<T>>>(iter: Iter) -> Self {
        // Take the first point, if applicable.
        let mut iter = iter.into_iter();
        let first = match iter.next() {
            Some(PathEvent::Begin { at }) => at,
            Some(pe) => panic!("Expected a Begin event, got {:?}", pe),
            None => panic!("Expected at least one event"),
        };

        // Take the remaining points.
        let mut close_begin = false;
        let buffer = iter
            .filter_map(|event| match event {
                PathEvent::Begin { at } => Some((at, Verb::Begin { close: close_begin })),
                PathEvent::Line { to, .. } => Some((to, Verb::Line)),
                PathEvent::Quadratic { control, to, .. } => Some((to, Verb::Quadratic { control })),
                PathEvent::Cubic {
                    control1,
                    control2,
                    to,
                    ..
                } => Some((to, Verb::Cubic { control1, control2 })),
                PathEvent::End { close, .. } => {
                    close_begin = close;
                    None
                }
                _ => unreachable!(),
            })
            .collect();

        PathBuffer { first, buffer }
    }
}

impl<Seg: Borrow<(Point<T>, Verb<T>)>, T: Copy, Buf: IntoIterator<Item = Seg>> Path<T>
    for PathBuffer<T, Buf>
{
    type Iter = PathBufferIterator<T, Buf::IntoIter>;

    fn path_iter(self) -> Self::Iter {
        PathBufferIterator {
            last: self.first,
            begin: self.first,
            is_first: true,
            remaining: self.buffer.into_iter(),
            begin_event: None,
        }
    }
}

impl<'a, T: Copy, Buf: Borrow<UnsizedBuffer<T>> + ?Sized> Path<T> for &'a PathBuffer<T, Buf> {
    type Iter = PathBufferIterator<T, SliceIter<'a, (Point<T>, Verb<T>)>>;

    fn path_iter(self) -> Self::Iter {
        PathBufferIterator {
            last: self.first,
            begin: self.first,
            is_first: true,
            remaining: self.buffer.borrow().iter(),
            begin_event: None,
        }
    }
}

/// An iterator that iterates over the events in a path.
pub struct PathBufferIterator<T: Copy, I> {
    /// The point that the next event will start from.
    last: Point<T>,

    /// The beginning of the current subpath.
    begin: Point<T>,

    /// Whether or not this is the first point.
    is_first: bool,

    /// The iterator over the remaining points in the path.
    remaining: I,

    /// The "Begin" verb is split into an "End" and "Begin" event. This is the "End"
    /// event that will be returned next.
    begin_event: Option<PathEvent<T>>,
}

impl<T: Copy, I> PathBufferIterator<T, I> {
    #[inline]
    fn parse_verb(&mut self, to: Point<T>, verb: Verb<T>) -> PathEvent<T> {
        let from = mem::replace(&mut self.last, to);
        self.last = to;

        match verb {
            Verb::Begin { close } => {
                // Set the end event.
                self.begin_event = Some(PathEvent::Begin { at: to });

                PathEvent::End {
                    first: mem::replace(&mut self.begin, to),
                    last: from,
                    close,
                }
            }
            Verb::Line => PathEvent::Line { from, to },
            Verb::Quadratic { control } => PathEvent::Quadratic { from, control, to },
            Verb::Cubic { control1, control2 } => PathEvent::Cubic {
                from,
                control1,
                control2,
                to,
            },
            _ => unreachable!(),
        }
    }
}

impl<Seg: Borrow<(Point<T>, Verb<T>)>, T: Copy, I: Iterator<Item = Seg>> Iterator
    for PathBufferIterator<T, I>
{
    type Item = PathEvent<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(end_event) = self.begin_event.take() {
            return Some(end_event);
        }

        if self.is_first {
            self.is_first = false;
            Some(PathEvent::Begin { at: self.last })
        } else {
            self.remaining.next().map(|seg| {
                let (to, verb) = *seg.borrow();
                self.parse_verb(to, verb)
            })
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (mut lo, mut hi) = self.remaining.size_hint();

        // Check for additional events.
        let add = (self.is_first as usize) + (self.begin_event.is_some() as usize);
        lo = lo.saturating_add(add);

        // The remaining events could all be Begin events which, while incomprehensible,
        // will each yield two events.
        hi = hi
            .and_then(|hi| hi.checked_mul(2))
            .and_then(|hi| hi.checked_add(add));

        (lo, hi)
    }
}
