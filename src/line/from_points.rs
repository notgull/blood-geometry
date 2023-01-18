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

//! Convert points to line segments and vice versa.

use core::iter::FusedIterator;

use super::LineSegment;
use crate::point::Point;

impl<T: Copy> LineSegment<T> {
    /// Convert an iterator of points into an iterator of line segments that
    /// connect each point to the next.
    pub fn from_points<I>(points: I) -> FromPoints<T, I>
    where
        I: Iterator<Item = Point<T>>,
    {
        FromPoints {
            iterator: points,
            previous: None,
            next: None,
        }
    }
}

/// The iterator returned by [`LineSegment::from_points`].
#[derive(Debug, Clone)]
pub struct FromPoints<T: Copy, I> {
    /// The iterator over the points.
    iterator: I,

    /// The previous point.
    previous: Option<Point<T>>,

    /// The next point, for backwards iteration.
    next: Option<Point<T>>,
}

impl<T: Copy, I: Iterator<Item = Point<T>>> Iterator for FromPoints<T, I> {
    type Item = LineSegment<T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.previous {
                Some(previous) => {
                    let current = self.iterator.next().or_else(|| self.next.take())?;
                    let segment = LineSegment::new(previous, current);
                    self.previous = Some(current);
                    return Some(segment);
                }
                None => {
                    self.previous = Some(self.iterator.next()?);
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (mut lo, mut hi) = self.iterator.size_hint();

        // Subtract one if we haven't pulled the previous yet.
        if self.previous.is_none() {
            lo = lo.saturating_sub(1);
            hi = hi.and_then(|hi| hi.checked_sub(1));
        }

        // Add one if we have a next.
        if self.next.is_some() {
            lo = lo.saturating_add(1);
            hi = hi.map(|hi| hi.saturating_add(1));
        }

        (lo, hi)
    }
}

impl<T: Copy, I: FusedIterator<Item = Point<T>>> FusedIterator for FromPoints<T, I> {}

impl<T: Copy, I: DoubleEndedIterator<Item = Point<T>>> DoubleEndedIterator for FromPoints<T, I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            match self.next {
                Some(next) => {
                    let current = self.iterator.next_back().or_else(|| self.previous.take())?;
                    let segment = LineSegment::new(current, next);
                    self.next = Some(current);
                    return Some(segment);
                }
                None => {
                    self.next = Some(self.iterator.next_back()?);
                }
            }
        }
    }
}
